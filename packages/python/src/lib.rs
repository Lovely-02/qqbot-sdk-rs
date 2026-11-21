use pyo3::create_exception;
use pyo3::exceptions::{PyException, PyValueError};
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;
use qqbot_sdk_core::{
    Bot, BotMode, ClientConfig, EventRouter, GatewayClient, GatewayConfig, Intents, SdkError,
    binding_api,
};
use serde::Serialize;
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use tokio::sync::watch;

create_exception!(qqbot_sdk, BotError, PyException);
create_exception!(qqbot_sdk, ApiError, BotError);

fn sdk_error(error: SdkError) -> PyErr {
    match error {
        SdkError::InvalidInput(message) => PyValueError::new_err(message),
        SdkError::Api {
            status,
            code,
            ref message,
            ref trace_id,
            ref data,
        } => Python::attach(|py| {
            let exception = ApiError::new_err(error.to_string());
            let instance = exception.value(py);
            let set_details = || -> PyResult<()> {
                instance.setattr("status", status)?;
                instance.setattr("code", code)?;
                instance.setattr("message", message.as_str())?;
                instance.setattr("trace_id", trace_id.as_deref())?;
                instance.setattr("data", pythonize::pythonize(py, data)?)?;
                Ok(())
            };
            set_details().err().unwrap_or(exception)
        }),
        _ => BotError::new_err(error.to_string()),
    }
}

fn response_to_python<T: Serialize>(response: T) -> PyResult<Py<PyAny>> {
    Python::attach(|py| Ok(pythonize::pythonize(py, &response)?.unbind()))
}

#[pyclass(name = "Bot", module = "qqbot_sdk._native")]
struct PythonBot {
    client: Bot,
    secret: String,
    intents: Option<u32>,
    started: AtomicBool,
    shutdown: watch::Sender<bool>,
}

#[pymethods]
impl PythonBot {
    #[new]
    #[pyo3(signature = (app_id, app_secret, *, mode="public", intents=None, api_base_url=None))]
    fn new(
        app_id: String,
        app_secret: String,
        mode: &str,
        intents: Option<u32>,
        api_base_url: Option<String>,
    ) -> PyResult<Self> {
        if app_id.trim().is_empty() || app_secret.trim().is_empty() {
            return Err(PyValueError::new_err(
                "app_id and app_secret must not be empty",
            ));
        }
        let mode = match mode {
            "public" => BotMode::PublicWebSocket,
            "private" => BotMode::PrivateWebSocket,
            "webhook" => BotMode::Webhook,
            _ => {
                return Err(PyValueError::new_err(
                    "mode must be public, private or webhook",
                ));
            }
        };
        let mut config = ClientConfig {
            mode,
            ..Default::default()
        };
        if let Some(url) = api_base_url {
            config.api_base_url = url;
        }
        let client = Bot::with_config(app_id, app_secret.clone(), config).map_err(sdk_error)?;
        Ok(Self {
            client,
            secret: app_secret,
            intents,
            started: AtomicBool::new(false),
            shutdown: watch::channel(false).0,
        })
    }

    #[getter]
    fn api<'py>(slf: &Bound<'py, Self>) -> PyResult<Bound<'py, PyAny>> {
        slf.py()
            .import("qqbot_sdk.api")?
            .getattr("BotApi")?
            .call1((slf,))
    }

    #[pyo3(name = "_call")]
    #[pyo3(signature = (operation, *args))]
    fn call<'py>(
        &self,
        operation: String,
        args: &Bound<'py, pyo3::types::PyTuple>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let values = args
            .iter()
            .map(|arg| {
                pythonize::depythonize(&arg)
                    .map_err(|error| PyValueError::new_err(error.to_string()))
            })
            .collect::<PyResult<Vec<Value>>>()?;
        call_api(args.py(), self.client.clone(), operation, values)
    }

    #[pyo3(name = "_upload_material")]
    fn upload_material<'py>(
        &self,
        py: Python<'py>,
        file_name: String,
        content_type: String,
        data: Vec<u8>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        future_into_py(py, async move {
            let response = client
                .api()
                .materials()
                .upload(&file_name, &content_type, &data)
                .await
                .map_err(sdk_error)?;
            response_to_python(response)
        })
    }

    #[pyo3(name = "_upload_media", signature = (kind, id, file_type, data, srv_send_msg=false))]
    fn upload_media<'py>(
        &self,
        py: Python<'py>,
        kind: String,
        id: String,
        file_type: u8,
        data: Vec<u8>,
        srv_send_msg: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        let target =
            binding_api::MediaTargetInput::parse(serde_json::json!({"kind": kind, "id": id}))
                .map_err(sdk_error)?;
        let file_type =
            serde_json::from_value(file_type.into()).map_err(|error| sdk_error(error.into()))?;
        future_into_py(py, async move {
            let response = client
                .api()
                .messages()
                .upload_media(
                    target.target().map_err(sdk_error)?,
                    file_type,
                    &data,
                    srv_send_msg,
                )
                .await
                .map_err(sdk_error)?;
            response_to_python(response)
        })
    }

    #[pyo3(name = "_send_media")]
    fn send_media<'py>(
        &self,
        kind: String,
        id: String,
        file_type: u8,
        data: Vec<u8>,
        message: &Bound<'py, PyAny>,
    ) -> PyResult<Bound<'py, PyAny>> {
        let client = self.client.clone();
        let target =
            binding_api::MediaTargetInput::parse(serde_json::json!({"kind": kind, "id": id}))
                .map_err(sdk_error)?;
        let file_type =
            serde_json::from_value(file_type.into()).map_err(|error| sdk_error(error.into()))?;
        let body = pythonize::depythonize(message)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        let body = binding_api::sendable(body).map_err(sdk_error)?;
        future_into_py(message.py(), async move {
            let response = client
                .api()
                .messages()
                .send_media(target.target().map_err(sdk_error)?, file_type, &data, body)
                .await
                .map_err(sdk_error)?;
            response_to_python(response)
        })
    }

    fn run<'py>(&self, callback: &Bound<'py, PyAny>) -> PyResult<Bound<'py, PyAny>> {
        if self.client.mode().is_webhook() {
            return Err(PyValueError::new_err("Use handle_webhook in webhook mode"));
        }
        if !callback.is_callable() {
            return Err(pyo3::exceptions::PyTypeError::new_err(
                "callback must be callable",
            ));
        }
        let py = callback.py();
        let event_loop = pyo3_async_runtimes::tokio::get_current_loop(py)?.unbind();
        let callback = callback.clone().unbind();
        if self.started.swap(true, Ordering::AcqRel) {
            return Err(BotError::new_err(
                "Bot already started; create a new Bot after stop",
            ));
        }
        let mut shutdown = self.shutdown.subscribe();
        let client = Arc::new(self.client.clone());
        let intents = self.intents;
        let completed = py.None();
        future_into_py(py, async move {
            if *shutdown.borrow_and_update() {
                return Ok(completed);
            }
            let router = EventRouter::new();
            let callback_context = Arc::new((event_loop, callback));
            router
                .on_raw(move |event, _| {
                    let callback_context = callback_context.clone();
                    async move {
                        let payload = json!({
                            "id": event.id,
                            "name": event.name,
                            "sequence": event.sequence.map(|value| value.to_string()),
                            "data": event.data,
                        });
                        Python::attach(|py| -> PyResult<()> {
                            let (event_loop, callback) = &*callback_context;
                            let event = pythonize::pythonize(py, &payload)?;
                            event_loop
                                .bind(py)
                                .call_method1("call_soon_threadsafe", (callback.bind(py), event))?;
                            Ok(())
                        })
                        .map_err(|error| {
                            SdkError::InvalidInput(format!("Python event callback failed: {error}"))
                        })
                    }
                })
                .await;
            let mut config = GatewayConfig::for_bot(&client);
            if let Some(bits) = intents {
                config.intents = Intents::from_bits(u64::from(bits));
            }
            let gateway = GatewayClient::new(client, router, config).map_err(sdk_error)?;
            tokio::select! {
                result = gateway.run() => result.map_err(sdk_error).map(|()| completed),
                _ = shutdown.changed() => Ok(completed),
            }
        })
    }

    fn stop(&self) {
        self.shutdown.send_replace(true);
    }

    fn handle_webhook(&self, timestamp: &str, signature: &str, body: &[u8]) -> PyResult<Py<PyAny>> {
        if !self.client.mode().is_webhook() {
            return Err(PyValueError::new_err("Bot must use webhook mode"));
        }
        response_to_python(
            qqbot_sdk_core::binding_webhook::webhook(&self.secret, timestamp, signature, body)
                .map_err(sdk_error)?,
        )
    }

    fn user(&self, py: Python<'_>, user_openid: String) -> PyResult<Py<PythonUser>> {
        if user_openid.trim().is_empty() {
            return Err(PyValueError::new_err("user_openid must not be empty"));
        }
        Py::new(
            py,
            PyClassInitializer::from(self.target(
                user_openid,
                "messages.sendC2c",
                "messages.deleteC2c",
            ))
            .add_subclass(PythonUser),
        )
    }

    fn group(&self, id: String) -> PythonTarget {
        self.target(id, "messages.sendGroup", "messages.deleteGroup")
    }

    fn channel(&self, id: String) -> PythonTarget {
        self.target(id, "messages.sendChannel", "channels.recallMessage")
    }

    fn direct(&self, id: String) -> PythonTarget {
        self.target(id, "messages.sendDm", "messages.deleteDm")
    }
}

impl PythonBot {
    fn target(&self, id: String, send: &'static str, recall: &'static str) -> PythonTarget {
        PythonTarget {
            id,
            client: self.client.clone(),
            send,
            recall,
        }
    }
}

fn call_api(
    py: Python<'_>,
    client: Bot,
    operation: String,
    args: Vec<Value>,
) -> PyResult<Bound<'_, PyAny>> {
    future_into_py(py, async move {
        let response = binding_api::call(&client, &operation, args)
            .await
            .map_err(sdk_error)?;
        response_to_python(response)
    })
}

#[pyclass(name = "Target", module = "qqbot_sdk._native", subclass)]
struct PythonTarget {
    id: String,
    client: Bot,
    send: &'static str,
    recall: &'static str,
}

#[pymethods]
impl PythonTarget {
    #[getter]
    fn id(&self) -> &str {
        &self.id
    }

    #[pyo3(signature = (content, *, msg_id=None))]
    fn send<'py>(
        &self,
        content: &Bound<'py, PyAny>,
        msg_id: Option<String>,
    ) -> PyResult<Bound<'py, PyAny>> {
        if self.id.trim().is_empty() {
            return Err(PyValueError::new_err("target id must not be empty"));
        }
        let mut message: Value = pythonize::depythonize(content)
            .map_err(|error| PyValueError::new_err(error.to_string()))?;
        if let Some(id) = msg_id {
            message = match message {
                Value::String(text) => serde_json::json!({"content": text, "msg_id": id}),
                Value::Object(mut body) => {
                    body.insert("msg_id".into(), Value::String(id));
                    Value::Object(body)
                }
                _ => {
                    return Err(PyValueError::new_err(
                        "message must be text or a MessageRequest object",
                    ));
                }
            };
        }
        call_api(
            content.py(),
            self.client.clone(),
            self.send.into(),
            vec![self.id.clone().into(), message],
        )
    }

    #[pyo3(signature = (message_id, hide_tip=false))]
    fn recall<'py>(
        &self,
        py: Python<'py>,
        message_id: String,
        hide_tip: bool,
    ) -> PyResult<Bound<'py, PyAny>> {
        call_api(
            py,
            self.client.clone(),
            self.recall.into(),
            vec![self.id.clone().into(), message_id.into(), hide_tip.into()],
        )
    }
}

#[pyclass(name = "User", module = "qqbot_sdk._native", extends = PythonTarget)]
struct PythonUser;

#[pymodule]
fn _native(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PythonBot>()?;
    module.add_class::<PythonTarget>()?;
    module.add_class::<PythonUser>()?;
    module.add("BotError", module.py().get_type::<BotError>())?;
    module.add("ApiError", module.py().get_type::<ApiError>())?;
    Ok(())
}
