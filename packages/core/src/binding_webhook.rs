use crate::Result;
use serde_json::{Value, json};

pub fn webhook(secret: &str, timestamp: &str, signature: &str, body: &[u8]) -> Result<Value> {
    let webhook = crate::Webhook::new(crate::WebhookVerifier::from_secret(secret)?);
    let payload = webhook.parse(timestamp, signature, body)?;
    if payload.op == 13 {
        return Ok(json!({ "response": crate::Webhook::validation_response(body, secret)? }));
    }
    Ok(json!({
        "response": crate::Webhook::acknowledgement(),
        "event": { "id": payload.id, "name": payload.t.unwrap_or_default(),
            "sequence": payload.s.map(|value| value.to_string()), "data": payload.d }
    }))
}
