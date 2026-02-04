use worker::{Env, Request, Response, Result};

use crate::models::ErrorResponse;

const ADMIN_COOKIE_NAME: &str = "admin_token";

/// Bearerトークンをチェック（純粋関数）
fn check_bearer_token(auth_header: Option<&str>, expected: &str) -> bool {
    auth_header
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(|t| t == expected)
        .unwrap_or(false)
}

/// Cookieヘッダーからトークンを抽出（純粋関数）
fn extract_cookie_token<'a>(cookie_header: Option<&'a str>, cookie_name: &str) -> Option<&'a str> {
    cookie_header?
        .split(';')
        .map(|s| s.trim())
        .filter_map(|pair| pair.split_once('='))
        .find(|(name, _)| *name == cookie_name)
        .map(|(_, value)| value)
}

/// Cookieからトークンをチェック（純粋関数）
fn check_cookie_token(cookie_header: Option<&str>, expected: &str) -> bool {
    extract_cookie_token(cookie_header, ADMIN_COOKIE_NAME)
        .map(|t| t == expected)
        .unwrap_or(false)
}

/// Bearer tokenまたはCookieから管理者認証を検証
pub fn verify_admin_token(req: &Request, env: &Env) -> Result<bool> {
    let expected_token = match env.secret("ADMIN_TOKEN") {
        Ok(secret) => secret.to_string(),
        Err(_) => return Ok(false),
    };

    // まずAuthorizationヘッダーをチェック（API用）
    let auth_header = req.headers().get("Authorization")?;
    if check_bearer_token(auth_header.as_deref(), &expected_token) {
        return Ok(true);
    }

    // 次にCookieをチェック（HTML画面用）
    let cookie_header = req.headers().get("Cookie")?;
    if check_cookie_token(cookie_header.as_deref(), &expected_token) {
        return Ok(true);
    }

    Ok(false)
}

/// 認証Cookie設定用のSet-Cookieヘッダー値を生成
pub fn create_auth_cookie(token: &str, secure: bool) -> String {
    let secure_flag = if secure { "; Secure" } else { "" };
    format!(
        "{}={}; HttpOnly; SameSite=Strict; Path=/admin; Max-Age=86400{}",
        ADMIN_COOKIE_NAME, token, secure_flag
    )
}

/// 認証Cookie削除用のSet-Cookieヘッダー値を生成
pub fn create_logout_cookie() -> String {
    format!(
        "{}=; HttpOnly; SameSite=Strict; Path=/admin; Max-Age=0",
        ADMIN_COOKIE_NAME
    )
}

/// 認証失敗時のJSONレスポンスを生成
pub fn unauthorized_response() -> Result<Response> {
    Response::from_json(&ErrorResponse::new("Unauthorized", "UNAUTHORIZED"))
        .map(|r| r.with_status(401))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_bearer_token_valid() {
        assert!(check_bearer_token(Some("Bearer secret123"), "secret123"));
    }

    #[test]
    fn test_check_bearer_token_invalid_token() {
        assert!(!check_bearer_token(Some("Bearer wrong"), "secret123"));
    }

    #[test]
    fn test_check_bearer_token_no_header() {
        assert!(!check_bearer_token(None, "secret123"));
    }

    #[test]
    fn test_check_bearer_token_wrong_scheme() {
        assert!(!check_bearer_token(Some("Basic secret123"), "secret123"));
    }

    #[test]
    fn test_check_bearer_token_no_space() {
        assert!(!check_bearer_token(Some("Bearersecret123"), "secret123"));
    }

    #[test]
    fn test_check_bearer_token_empty() {
        assert!(!check_bearer_token(Some("Bearer "), "secret123"));
    }

    #[test]
    fn test_extract_cookie_token_valid() {
        let cookie = "admin_token=secret123";
        assert_eq!(
            extract_cookie_token(Some(cookie), "admin_token"),
            Some("secret123")
        );
    }

    #[test]
    fn test_extract_cookie_token_multiple() {
        let cookie = "other=value; admin_token=secret123; another=thing";
        assert_eq!(
            extract_cookie_token(Some(cookie), "admin_token"),
            Some("secret123")
        );
    }

    #[test]
    fn test_extract_cookie_token_not_found() {
        let cookie = "other=value";
        assert_eq!(extract_cookie_token(Some(cookie), "admin_token"), None);
    }

    #[test]
    fn test_extract_cookie_token_none() {
        assert_eq!(extract_cookie_token(None, "admin_token"), None);
    }

    #[test]
    fn test_check_cookie_token_valid() {
        assert!(check_cookie_token(Some("admin_token=secret123"), "secret123"));
    }

    #[test]
    fn test_check_cookie_token_invalid() {
        assert!(!check_cookie_token(Some("admin_token=wrong"), "secret123"));
    }

    #[test]
    fn test_check_cookie_token_no_cookie() {
        assert!(!check_cookie_token(None, "secret123"));
    }

    #[test]
    fn test_check_cookie_token_wrong_name() {
        assert!(!check_cookie_token(Some("other=secret123"), "secret123"));
    }

    #[test]
    fn test_create_auth_cookie_secure() {
        let cookie = create_auth_cookie("mytoken", true);
        assert!(cookie.contains("admin_token=mytoken"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("SameSite=Strict"));
        assert!(cookie.contains("Path=/admin"));
        assert!(cookie.contains("Secure"));
    }

    #[test]
    fn test_create_auth_cookie_insecure() {
        let cookie = create_auth_cookie("mytoken", false);
        assert!(cookie.contains("admin_token=mytoken"));
        assert!(!cookie.contains("Secure"));
    }

    #[test]
    fn test_create_logout_cookie() {
        let cookie = create_logout_cookie();
        assert!(cookie.contains("admin_token="));
        assert!(cookie.contains("Max-Age=0"));
    }
}
