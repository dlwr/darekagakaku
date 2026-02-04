use serde::{Deserialize, Serialize};
use worker::*;

#[derive(Serialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
struct TurnstileRequest {
    secret: String,
    response: String,
    remoteip: Option<String>,
}

#[derive(Deserialize)]
#[cfg_attr(test, derive(Debug, PartialEq))]
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turnstile_request_serialization() {
        let req = TurnstileRequest {
            secret: "test-secret".to_string(),
            response: "test-token".to_string(),
            remoteip: Some("192.168.1.1".to_string()),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"secret\":\"test-secret\""));
        assert!(json.contains("\"response\":\"test-token\""));
        assert!(json.contains("\"remoteip\":\"192.168.1.1\""));
    }

    #[test]
    fn test_turnstile_request_serialization_without_ip() {
        let req = TurnstileRequest {
            secret: "test-secret".to_string(),
            response: "test-token".to_string(),
            remoteip: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"remoteip\":null"));
    }

    #[test]
    fn test_turnstile_response_deserialization_success() {
        let json = r#"{"success": true}"#;
        let resp: TurnstileResponse = serde_json::from_str(json).unwrap();
        assert!(resp.success);
    }

    #[test]
    fn test_turnstile_response_deserialization_failure() {
        let json = r#"{"success": false}"#;
        let resp: TurnstileResponse = serde_json::from_str(json).unwrap();
        assert!(!resp.success);
    }

    #[test]
    fn test_turnstile_response_deserialization_with_extra_fields() {
        // Turnstile APIは追加フィールドを返すことがある
        let json = r#"{"success": true, "challenge_ts": "2025-01-15T00:00:00Z", "hostname": "example.com"}"#;
        let resp: TurnstileResponse = serde_json::from_str(json).unwrap();
        assert!(resp.success);
    }
}
