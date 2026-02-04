use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Serialize)]
struct TurnstileRequest {
    secret: String,
    response: String,
    remoteip: Option<String>,
}

#[derive(Deserialize)]
struct TurnstileResponse {
    success: bool,
}

pub async fn verify_turnstile(secret: &str, token: &str, ip: Option<&str>) -> Result<bool> {
    let body = TurnstileRequest {
        secret: secret.to_string(),
        response: token.to_string(),
        remoteip: ip.map(|s| s.to_string()),
    };

    let headers = Headers::new();
    headers.set("Content-Type", "application/json")?;

    let req = Request::new_with_init(
        "https://challenges.cloudflare.com/turnstile/v0/siteverify",
        RequestInit::new()
            .with_method(Method::Post)
            .with_headers(headers)
            .with_body(Some(serde_json::to_string(&body)?.into())),
    )?;

    let mut resp = Fetch::Request(req).send().await?;
    let result: TurnstileResponse = resp.json().await?;

    Ok(result.success)
}
