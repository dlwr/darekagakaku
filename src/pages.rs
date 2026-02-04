use worker::d1::D1Database;
use worker::{Headers, Request, Response, Result, RouteContext};

use crate::auth;
use crate::db;
use crate::models::{DiaryEntrySummary, VersionSummary};
use crate::templates;
use crate::time::{is_today, is_valid_date, today_jst};

/// GET /a - Aboutページ（これはなにか）
pub async fn about(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let html = templates::render_about();
    Response::from_html(html)
}

/// GET / - ホームページ（今日の日記フォーム）
pub async fn home(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let db: D1Database = ctx.env.d1("DB")?;
    let today = today_jst();

    let entry = match db::get_entry(&db, &today).await {
        Ok(entry) => entry,
        Err(e) => {
            worker::console_error!("Failed to get today's entry: {:?}", e);
            None
        }
    };

    let turnstile_site_key = ctx
        .env
        .var("TURNSTILE_SITE_KEY")
        .map(|v| v.to_string())
        .unwrap_or_default();

    let html = templates::render_home(entry.as_ref(), &turnstile_site_key);
    Response::from_html(html)
}

/// GET /entries - 過去の日記一覧
pub async fn entries_list(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let db: D1Database = ctx.env.d1("DB")?;

    let entries = match db::list_past_entries(&db, 100).await {
        Ok(entries) => entries,
        Err(e) => {
            worker::console_error!("Failed to list entries: {:?}", e);
            vec![]
        }
    };

    let summaries: Vec<DiaryEntrySummary> = entries
        .iter()
        .map(DiaryEntrySummary::from_entry)
        .collect();

    let html = templates::render_archive(&summaries);
    Response::from_html(html)
}

/// GET /entries/:date - 特定日の日記を表示
pub async fn entry_page(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let db: D1Database = ctx.env.d1("DB")?;

    let date = match ctx.param("date") {
        Some(d) => d,
        None => {
            let html = templates::render_not_found();
            return Response::from_html(html).map(|r| r.with_status(404));
        }
    };

    // 日付の形式を検証
    if !is_valid_date(date) {
        let html = templates::render_not_found();
        return Response::from_html(html).map(|r| r.with_status(404));
    }

    match db::get_entry(&db, date).await {
        Ok(Some(entry)) => {
            let can_edit = is_today(date);
            let html = templates::render_entry(&entry, can_edit);
            Response::from_html(html)
        }
        Ok(None) => {
            let html = templates::render_not_found();
            Response::from_html(html).map(|r| r.with_status(404))
        }
        Err(e) => {
            worker::console_error!("Failed to get entry: {:?}", e);
            let html = templates::render_not_found();
            Response::from_html(html).map(|r| r.with_status(500))
        }
    }
}

/// GET /feed - RSSフィード
pub async fn feed(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let db: D1Database = ctx.env.d1("DB")?;

    // 今日の日記は編集中なので、過去の確定した日記のみをRSSに含める
    let entries = match db::list_past_entries(&db, 20).await {
        Ok(entries) => entries,
        Err(e) => {
            worker::console_error!("Failed to list entries for RSS: {:?}", e);
            vec![]
        }
    };

    // ベースURLをリクエストから取得
    let url = req.url()?;
    let base_url = format!("{}://{}", url.scheme(), url.host_str().unwrap_or("localhost"));

    let rss = templates::render_rss(&entries, &base_url);

    let headers = Headers::new();
    headers.set("Content-Type", "application/rss+xml; charset=utf-8")?;

    Ok(Response::ok(rss)?.with_headers(headers))
}

/// GET /admin/login - 管理者ログインページ
pub async fn admin_login_page(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let html = templates::render_admin_login(None);
    Response::from_html(html)
}

/// POST /admin/login - 管理者ログイン処理
pub async fn admin_login_submit(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let expected_token = match ctx.env.secret("ADMIN_TOKEN") {
        Ok(secret) => secret.to_string(),
        Err(_) => {
            let html = templates::render_admin_login(Some("認証が設定されていません"));
            return Response::from_html(html).map(|r| r.with_status(500));
        }
    };

    // フォームデータを取得
    let form_data = req.form_data().await?;
    let submitted_token = form_data
        .get("token")
        .and_then(|v| match v {
            worker::FormEntry::Field(s) => Some(s),
            _ => None,
        })
        .unwrap_or_default();

    if submitted_token != expected_token {
        let html = templates::render_admin_login(Some("トークンが正しくありません"));
        return Response::from_html(html).map(|r| r.with_status(401));
    }

    // httpsかどうかをチェック
    let is_secure = req.url()?.scheme() == "https";

    // 認証成功、Cookieをセット
    let cookie = auth::create_auth_cookie(&expected_token, is_secure);
    let headers = Headers::new();
    headers.set("Set-Cookie", &cookie)?;
    headers.set("Location", "/admin/versions")?;

    Ok(Response::empty()?.with_status(302).with_headers(headers))
}

/// GET /admin/logout - 管理者ログアウト
pub async fn admin_logout(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let cookie = auth::create_logout_cookie();
    let headers = Headers::new();
    headers.set("Set-Cookie", &cookie)?;
    headers.set("Location", "/")?;

    Ok(Response::empty()?.with_status(302).with_headers(headers))
}

/// GET /admin/versions - 管理者用：日付選択ページ
pub async fn admin_versions_index(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // 認証チェック
    if !auth::verify_admin_token(&req, &ctx.env)? {
        // 未認証の場合はログインページにリダイレクト
        let headers = Headers::new();
        headers.set("Location", "/admin/login")?;
        return Ok(Response::empty()?.with_status(302).with_headers(headers));
    }

    let html = templates::render_admin_versions_index();
    Response::from_html(html)
}

/// GET /admin/entries/:date/versions - 管理者用：バージョン一覧ページ
pub async fn admin_versions_list(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // 認証チェック
    if !auth::verify_admin_token(&req, &ctx.env)? {
        let headers = Headers::new();
        headers.set("Location", "/admin/login")?;
        return Ok(Response::empty()?.with_status(302).with_headers(headers));
    }

    let db: D1Database = ctx.env.d1("DB")?;

    let date = match ctx.param("date") {
        Some(d) => d,
        None => {
            let html = templates::render_not_found();
            return Response::from_html(html).map(|r| r.with_status(404));
        }
    };

    if !is_valid_date(date) {
        let html = templates::render_not_found();
        return Response::from_html(html).map(|r| r.with_status(404));
    }

    // 現在のエントリを取得
    let current = db::get_entry(&db, date).await?;

    // バージョン一覧を取得
    let versions = db::list_versions(&db, date).await?;
    let summaries: Vec<VersionSummary> = versions.iter().map(VersionSummary::from_version).collect();

    let html = templates::render_admin_versions_list(
        date,
        current.as_ref().map(|e| e.content.as_str()),
        &summaries,
    );
    Response::from_html(html)
}

/// GET /admin/entries/:date/versions/:version - 管理者用：バージョン詳細ページ
pub async fn admin_version_detail(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // 認証チェック
    if !auth::verify_admin_token(&req, &ctx.env)? {
        let headers = Headers::new();
        headers.set("Location", "/admin/login")?;
        return Ok(Response::empty()?.with_status(302).with_headers(headers));
    }

    let db: D1Database = ctx.env.d1("DB")?;

    let date = match ctx.param("date") {
        Some(d) => d,
        None => {
            let html = templates::render_not_found();
            return Response::from_html(html).map(|r| r.with_status(404));
        }
    };

    let version: i32 = match ctx.param("version").and_then(|v| v.parse().ok()) {
        Some(v) => v,
        None => {
            let html = templates::render_not_found();
            return Response::from_html(html).map(|r| r.with_status(404));
        }
    };

    if !is_valid_date(date) {
        let html = templates::render_not_found();
        return Response::from_html(html).map(|r| r.with_status(404));
    }

    match db::get_version(&db, date, version).await? {
        Some(v) => {
            let html = templates::render_admin_version_detail(&v);
            Response::from_html(html)
        }
        None => {
            let html = templates::render_not_found();
            Response::from_html(html).map(|r| r.with_status(404))
        }
    }
}
