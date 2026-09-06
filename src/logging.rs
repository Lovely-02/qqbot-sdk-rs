//! 日志配置。

use crate::error::{Result, SdkError};
use std::{fmt as stdfmt, io::Write, path::PathBuf};
use time::OffsetDateTime;
use time::macros::format_description;
use tracing::{
    Event, Subscriber,
    field::{Field, Visit},
};
use tracing_appender::{non_blocking, rolling};
use tracing_subscriber::{
    EnvFilter,
    fmt::{
        self, FmtContext, FormattedFields,
        format::{FormatEvent, FormatFields, Writer},
    },
    registry::LookupSpan,
};

/// 业务事件日志目标。
pub(crate) const EVENT_LOG_TARGET: &str = "qqbot_sdk_rs::event";

/// API 原始错误日志目标。
pub(crate) const API_ERROR_LOG_TARGET: &str = "qqbot_sdk_rs::api_error";

/// 单行日志格式器。
#[derive(Debug, Clone, Copy)]
struct TextEventFormatter;

/// 日志字段。
#[derive(Default)]
struct EventFields {
    message: Option<String>,
    fields: Vec<(String, String)>,
}

impl EventFields {
    fn record(&mut self, field: &Field, value: String) {
        if field.name() == "message" {
            self.message = Some(value);
        } else {
            self.fields.push((field.name().to_owned(), value));
        }
    }

    fn debug_value(value: &dyn stdfmt::Debug) -> String {
        let value = format!("{value:?}");
        value
            .strip_prefix('"')
            .and_then(|value| value.strip_suffix('"'))
            .map_or(value.clone(), str::to_owned)
    }
}

impl Visit for EventFields {
    fn record_str(&mut self, field: &Field, value: &str) {
        self.record(field, value.to_owned());
    }

    fn record_debug(&mut self, field: &Field, value: &dyn stdfmt::Debug) {
        self.record(field, Self::debug_value(value));
    }
}

/// 将日志正文压成一行。
fn one_line(value: &str) -> String {
    value.replace(['\r', '\n'], " ").trim().to_owned()
}

impl<S, N> FormatEvent<S, N> for TextEventFormatter
where
    S: Subscriber + for<'lookup> LookupSpan<'lookup>,
    N: for<'writer> FormatFields<'writer> + 'static,
{
    fn format_event(
        &self,
        context: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> stdfmt::Result {
        let timestamp = OffsetDateTime::now_utc()
            .format(format_description!(
                "[year]-[month]-[day] [hour]:[minute]:[second]"
            ))
            .map_err(|_| stdfmt::Error)?;
        write!(writer, "{timestamp}")?;

        let mut fields = EventFields::default();
        event.record(&mut fields);
        if event.metadata().target() != EVENT_LOG_TARGET {
            write!(writer, " [{}]", event.metadata().level())?;
        }

        let preserve_message = event.metadata().target() == API_ERROR_LOG_TARGET;
        if let Some(message) = fields.message {
            if preserve_message {
                write!(writer, " {message}")?;
            } else {
                write!(writer, " {}", one_line(&message))?;
            }
        }
        for (name, value) in fields.fields {
            write!(writer, " {name}={}", one_line(&value))?;
        }
        if event.metadata().target() != EVENT_LOG_TARGET
            && event.metadata().target() != API_ERROR_LOG_TARGET
            && let Some(scope) = context.event_scope()
        {
            for span in scope.from_root() {
                let extensions = span.extensions();
                if let Some(fields) = extensions.get::<FormattedFields<N>>()
                    && !fields.is_empty()
                {
                    write!(writer, " {fields}")?;
                }
            }
        }
        writeln!(writer)
    }
}

/// 日志输出格式。
#[derive(Debug, Clone, Copy)]
pub enum Format {
    Pretty,
    Json,
}

/// 日志输出目标。
#[derive(Debug, Clone)]
pub enum LogTarget {
    /// 标准输出。
    Stdout,
    /// 标准错误。
    Stderr,
    /// 按天滚动的文件。
    FileDaily(PathBuf),
    /// 按小时滚动的文件。
    FileHourly(PathBuf),
}

/// 日志写入守卫。
pub type WorkerGuard = tracing_appender::non_blocking::WorkerGuard;

/// 日志配置器。
#[derive(Debug, Clone)]
pub struct SakuraLogger {
    format: Format,
    ansi: bool,
    level: String,
    target: LogTarget,
}

impl Default for SakuraLogger {
    fn default() -> Self {
        Self {
            format: Format::Pretty,
            ansi: true,
            level: "info".into(),
            target: LogTarget::Stdout,
        }
    }
}

impl SakuraLogger {
    /// 创建默认配置。
    pub fn builder() -> Self {
        Self::default()
    }

    /// 初始化默认日志。
    pub fn init() -> Result<WorkerGuard> {
        Self::default().try_init()
    }

    /// 切换输出格式。
    pub fn with_format(mut self, format: Format) -> Self {
        self.format = format;
        self
    }
    /// 设置 ANSI 彩色输出。
    pub fn with_ansi(mut self, ansi: bool) -> Self {
        self.ansi = ansi;
        self
    }
    /// 设置默认等级，`RUST_LOG` 优先。
    pub fn with_level(mut self, level: impl Into<String>) -> Self {
        self.level = level.into();
        self
    }
    /// 设置输出目标。
    pub fn with_target(mut self, target: LogTarget) -> Self {
        self.target = target;
        self
    }

    /// 初始化日志并返回写入守卫。
    pub fn try_init(self) -> Result<WorkerGuard> {
        let writer: Box<dyn Write + Send> = match &self.target {
            LogTarget::Stdout => Box::new(std::io::stdout()),
            LogTarget::Stderr => Box::new(std::io::stderr()),
            LogTarget::FileDaily(path) => Box::new(rolling::daily(path, "qqbot.log")),
            LogTarget::FileHourly(path) => Box::new(rolling::hourly(path, "qqbot.log")),
        };
        let (writer, guard) = non_blocking(writer);
        let filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(self.level));
        let result = match self.format {
            Format::Pretty => fmt::Subscriber::builder()
                .with_env_filter(filter)
                .with_writer(writer)
                .with_ansi(self.ansi)
                .event_format(TextEventFormatter)
                .try_init(),
            Format::Json => fmt::Subscriber::builder()
                .with_env_filter(filter)
                .with_writer(writer)
                .with_ansi(false)
                .with_timer(tracing_subscriber::fmt::time::UtcTime::new(
                    format_description!(
                        "[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"
                    ),
                ))
                .json()
                .with_current_span(true)
                .with_span_list(true)
                .try_init(),
        };
        result.map_err(|e| SdkError::Logging(e.to_string()))?;
        Ok(guard)
    }
}
