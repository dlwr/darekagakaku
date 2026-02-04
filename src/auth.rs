use worker::{Env, Request, Response, Result};

use crate::models::ErrorResponse;

/// Bearerトークンをチェック（純粋関数）
fn check_bearer_token(auth_header: Option<&str>, expected: &str) -> bool {
    auth_header
        .and_then(|h| h.strip_prefix("Bearer "))
        .map(|t| t == expected)
        .unwrap_or(false)
}

/// クエリパラメーターからトークンをチェック（純粋関数）
fn check_query_token<'a, I>(query_pairs: I, expected: &str) -> bool
where
    I: Iterator<Item = (&'a str, &'a str)>,
{
    query_pairs
        .filter(|(k, _)| *k == "token")
        .any(|(_, v)| v == expected)
}

/// Bearer tokenまたはクエリパラメーターから管理者認証を検証
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

    // 次にクエリパラメーターをチェック（HTML画面用）
    let url = req.url()?;
    let has_valid_token = url
        .query_pairs()
        .any(|(k, v)| k == "token" && v == expected_token);

    Ok(has_valid_token)
}

/// クエリパラメーターからトークンを取得
pub fn get_token_from_query(req: &Request) -> Option<String> {
    let url = req.url().ok()?;
    for (key, value) in url.query_pairs() {
        if key == "token" {
            return Some(value.to_string());
        }
    }
    None
}

/// 認証失敗時のJSONレスポンスを生成
pub fn unauthorized_response() -> Result<Response> {
    Response::from_json(&ErrorResponse::new("Unauthorized", "UNAUTHORIZED"))
        .map(|r| r.with_status(401))
}

/// 認証失敗時のHTMLレスポンスを生成
pub fn unauthorized_html_response() -> Result<Response> {
    let html = r#"<!DOCTYPE html>
<html lang="ja">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>認証エラー - 誰かが書く日記</title>
<style>
body { font-family: sans-serif; max-width: 600px; margin: 2rem auto; padding: 0 1rem; }
h1 { color: #c00; }
</style>
</head>
<body>
<h1>認証エラー</h1>
<p>管理者トークンが無効または未指定です。</p>
<p><a href="/">トップページに戻る</a></p>
</body>
</html>"#;

    Response::from_html(html).map(|r| r.with_status(401))
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
    fn test_check_query_token_valid() {
        let pairs = vec![("token", "secret123")];
        assert!(check_query_token(pairs.into_iter(), "secret123"));
    }

    #[test]
    fn test_check_query_token_invalid() {
        let pairs = vec![("token", "wrong")];
        assert!(!check_query_token(pairs.into_iter(), "secret123"));
    }

    #[test]
    fn test_check_query_token_no_token_key() {
        let pairs = vec![("other", "secret123")];
        assert!(!check_query_token(pairs.into_iter(), "secret123"));
    }

    #[test]
    fn test_check_query_token_empty() {
        let pairs: Vec<(&str, &str)> = vec![];
        assert!(!check_query_token(pairs.into_iter(), "secret123"));
    }

    #[test]
    fn test_check_query_token_multiple_params() {
        let pairs = vec![("foo", "bar"), ("token", "secret123"), ("baz", "qux")];
        assert!(check_query_token(pairs.into_iter(), "secret123"));
    }
}
