use worker::{Env, Request, Response, Result};

use crate::models::ErrorResponse;

/// Bearer tokenまたはクエリパラメーターから管理者認証を検証
pub fn verify_admin_token(req: &Request, env: &Env) -> Result<bool> {
    let expected_token = match env.secret("ADMIN_TOKEN") {
        Ok(secret) => secret.to_string(),
        Err(_) => return Ok(false),
    };

    // まずAuthorizationヘッダーをチェック（API用）
    if let Some(auth_header) = req.headers().get("Authorization")? {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            return Ok(token == expected_token);
        }
    }

    // 次にクエリパラメーターをチェック（HTML画面用）
    let url = req.url()?;
    for (key, value) in url.query_pairs() {
        if key == "token" && value == expected_token {
            return Ok(true);
        }
    }

    Ok(false)
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
