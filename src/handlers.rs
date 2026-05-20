use serde::Deserialize;
use worker::d1::D1Database;
use worker::{Headers, Request, Response, Result, RouteContext};

use crate::auth;
use crate::db;
use crate::image::{detect_image_mime, MAX_IMAGE_SIZE};
use crate::models::{
    DiaryEntrySummary, DiaryEntryResponse, DiaryListResponse,
    ErrorResponse, TodayEmptyResponse, VersionDetailResponse, VersionListResponse, VersionSummary,
};
use crate::rate_limit;
use crate::time::{is_today, is_valid_date, today_jst};
use crate::turnstile;

const MAX_CONTENT_LENGTH: usize = 10000;

#[derive(Deserialize)]
struct PostTodayRequest {
    content: String,
    turnstile_token: Option<String>,
}

/// GET /api/today - 今日の日記を取得
pub async fn get_today(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let db: D1Database = ctx.env.d1("DB")?;
    let today = today_jst();

    match db::get_entry(&db, &today).await {
        Ok(Some(entry)) => {
            let response = DiaryEntryResponse::from_entry(&entry, true);
            Response::from_json(&response)
        }
        Ok(None) => {
            let response = TodayEmptyResponse {
                date: today,
                content: None,
                can_edit: true,
            };
            Response::from_json(&response)
        }
        Err(e) => {
            worker::console_error!("Failed to get today's entry: {:?}", e);
            Response::from_json(&ErrorResponse::internal_error())
                .map(|r| r.with_status(500))
        }
    }
}

/// POST /api/today - 今日の日記を作成/更新
pub async fn post_today(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let kv = ctx.env.kv("RATE_LIMIT")?;
    let ip = rate_limit::get_client_ip(&req);

    if rate_limit::check_rate_limit(&kv, &ip).await? {
        return Response::from_json(&ErrorResponse::bad_request("Too Many Requests"))
            .map(|r| r.with_status(429));
    }

    let db: D1Database = ctx.env.d1("DB")?;

    let body: PostTodayRequest = match req.json().await {
        Ok(body) => body,
        Err(_) => {
            return Response::from_json(&ErrorResponse::bad_request("Invalid JSON"))
                .map(|r| r.with_status(400));
        }
    };

    let token = match &body.turnstile_token {
        Some(t) => t,
        None => {
            return Response::from_json(&ErrorResponse::bad_request("Turnstile token required"))
                .map(|r| r.with_status(400));
        }
    };

    let secret = ctx.env.secret("TURNSTILE_SECRET_KEY")?.to_string();
    match turnstile::verify_turnstile(&secret, token, Some(&ip)).await {
        Ok(true) => {}
        Ok(false) => {
            return Response::from_json(&ErrorResponse::bad_request(
                "Turnstile verification failed",
            ))
            .map(|r| r.with_status(400));
        }
        Err(e) => {
            worker::console_error!("Turnstile verification error: {:?}", e);
            return Response::from_json(&ErrorResponse::internal_error())
                .map(|r| r.with_status(500));
        }
    }

    // CRLF を LF に正規化（Windows環境対応）
    let content = body.content.replace('\r', "");

    if content.trim().is_empty() {
        return Response::from_json(&ErrorResponse::bad_request("Content must not be empty"))
            .map(|r| r.with_status(400));
    }

    if content.chars().count() > MAX_CONTENT_LENGTH {
        return Response::from_json(&ErrorResponse::bad_request(format!(
            "Content too long. Maximum {} characters allowed.",
            MAX_CONTENT_LENGTH
        )))
        .map(|r| r.with_status(400));
    }

    match db::upsert_today_entry(&db, &content).await {
        Ok(()) => {
            if let Err(e) = rate_limit::increment_rate_limit(&kv, &ip).await {
                worker::console_error!("Failed to increment rate limit: {:?}", e);
            }

            let today = today_jst();
            let response = DiaryEntryResponse {
                date: today,
                content,
                can_edit: true,
            };
            Response::from_json(&response).map(|r| r.with_status(201))
        }
        Err(e) => {
            worker::console_error!("Failed to save entry: {:?}", e);
            Response::from_json(&ErrorResponse::internal_error())
                .map(|r| r.with_status(500))
        }
    }
}

/// POST /api/today/image - 今日の画像をアップロード（multipart/form-data）
///
/// フィールド: `image`（File）, `turnstile_token`（Field）
pub async fn post_today_image(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let kv = ctx.env.kv("RATE_LIMIT")?;
    let ip = rate_limit::get_client_ip(&req);

    if rate_limit::check_rate_limit(&kv, &ip).await? {
        return Response::from_json(&ErrorResponse::bad_request("Too Many Requests"))
            .map(|r| r.with_status(429));
    }

    let form = match req.form_data().await {
        Ok(f) => f,
        Err(_) => {
            return Response::from_json(&ErrorResponse::bad_request("Invalid form data"))
                .map(|r| r.with_status(400));
        }
    };

    let token = form
        .get("turnstile_token")
        .and_then(|v| match v {
            worker::FormEntry::Field(s) => Some(s),
            _ => None,
        })
        .unwrap_or_default();

    if token.is_empty() {
        return Response::from_json(&ErrorResponse::bad_request("Turnstile token required"))
            .map(|r| r.with_status(400));
    }

    let secret = ctx.env.secret("TURNSTILE_SECRET_KEY")?.to_string();
    match turnstile::verify_turnstile(&secret, &token, Some(&ip)).await {
        Ok(true) => {}
        Ok(false) => {
            return Response::from_json(&ErrorResponse::bad_request(
                "Turnstile verification failed",
            ))
            .map(|r| r.with_status(400));
        }
        Err(e) => {
            worker::console_error!("Turnstile verification error: {:?}", e);
            return Response::from_json(&ErrorResponse::internal_error())
                .map(|r| r.with_status(500));
        }
    }

    let file = match form.get("image") {
        Some(worker::FormEntry::File(f)) => f,
        _ => {
            return Response::from_json(&ErrorResponse::bad_request("Image file required"))
                .map(|r| r.with_status(400));
        }
    };

    if file.size() > MAX_IMAGE_SIZE {
        return Response::from_json(&ErrorResponse::bad_request(format!(
            "Image too large. Maximum {}MB.",
            MAX_IMAGE_SIZE / (1024 * 1024)
        )))
        .map(|r| r.with_status(413));
    }

    let bytes = file.bytes().await?;
    let mime = match detect_image_mime(&bytes) {
        Some(m) => m,
        None => {
            return Response::from_json(&ErrorResponse::bad_request(
                "Unsupported image format. Use JPEG, PNG, or WebP.",
            ))
            .map(|r| r.with_status(400));
        }
    };

    let bucket = ctx.env.bucket("IMAGES")?;
    let today = today_jst();
    let key = format!("entries/{}", today);

    let metadata = worker::HttpMetadata {
        content_type: Some(mime.to_string()),
        ..Default::default()
    };

    if let Err(e) = bucket
        .put(&key, bytes)
        .http_metadata(metadata)
        .execute()
        .await
    {
        worker::console_error!("Failed to upload image to R2: {:?}", e);
        return Response::from_json(&ErrorResponse::internal_error())
            .map(|r| r.with_status(500));
    }

    let db: D1Database = ctx.env.d1("DB")?;
    if let Err(e) = db::set_image_mime(&db, &today, mime).await {
        worker::console_error!("Failed to set image_mime: {:?}", e);
        return Response::from_json(&ErrorResponse::internal_error())
            .map(|r| r.with_status(500));
    }

    if let Err(e) = rate_limit::increment_rate_limit(&kv, &ip).await {
        worker::console_error!("Failed to increment rate limit: {:?}", e);
    }

    #[derive(serde::Serialize)]
    struct UploadResponse {
        date: String,
        image_mime: String,
        image_url: String,
    }

    let response = UploadResponse {
        date: today.clone(),
        image_mime: mime.to_string(),
        image_url: format!("/images/{}", today),
    };
    Response::from_json(&response).map(|r| r.with_status(201))
}

/// DELETE /api/today/image - 今日の画像を削除
///
/// Turnstileトークンを JSON body で受け取る。レートリミットも適用。
pub async fn delete_today_image(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let kv = ctx.env.kv("RATE_LIMIT")?;
    let ip = rate_limit::get_client_ip(&req);

    if rate_limit::check_rate_limit(&kv, &ip).await? {
        return Response::from_json(&ErrorResponse::bad_request("Too Many Requests"))
            .map(|r| r.with_status(429));
    }

    #[derive(Deserialize)]
    struct DeleteBody {
        turnstile_token: Option<String>,
    }

    let body: DeleteBody = match req.json().await {
        Ok(b) => b,
        Err(_) => {
            return Response::from_json(&ErrorResponse::bad_request("Invalid JSON"))
                .map(|r| r.with_status(400));
        }
    };

    let token = match body.turnstile_token {
        Some(t) if !t.is_empty() => t,
        _ => {
            return Response::from_json(&ErrorResponse::bad_request("Turnstile token required"))
                .map(|r| r.with_status(400));
        }
    };

    let secret = ctx.env.secret("TURNSTILE_SECRET_KEY")?.to_string();
    match turnstile::verify_turnstile(&secret, &token, Some(&ip)).await {
        Ok(true) => {}
        Ok(false) => {
            return Response::from_json(&ErrorResponse::bad_request(
                "Turnstile verification failed",
            ))
            .map(|r| r.with_status(400));
        }
        Err(e) => {
            worker::console_error!("Turnstile verification error: {:?}", e);
            return Response::from_json(&ErrorResponse::internal_error())
                .map(|r| r.with_status(500));
        }
    }

    let today = today_jst();
    let key = format!("entries/{}", today);

    let bucket = ctx.env.bucket("IMAGES")?;
    if let Err(e) = bucket.delete(&key).await {
        worker::console_error!("Failed to delete image from R2: {:?}", e);
        return Response::from_json(&ErrorResponse::internal_error())
            .map(|r| r.with_status(500));
    }

    let db: D1Database = ctx.env.d1("DB")?;
    if let Err(e) = db::clear_image_mime(&db, &today).await {
        worker::console_error!("Failed to clear image_mime: {:?}", e);
        return Response::from_json(&ErrorResponse::internal_error())
            .map(|r| r.with_status(500));
    }

    if let Err(e) = rate_limit::increment_rate_limit(&kv, &ip).await {
        worker::console_error!("Failed to increment rate limit: {:?}", e);
    }

    Response::empty().map(|r| r.with_status(204))
}

/// GET /images/:date - 画像配信
pub async fn get_image(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let date = match ctx.param("date") {
        Some(d) => d.as_str(),
        None => return Response::error("Not Found", 404),
    };

    if !is_valid_date(date) {
        return Response::error("Not Found", 404);
    }

    let db: D1Database = ctx.env.d1("DB")?;
    let mime = match db::get_entry(&db, date).await? {
        Some(entry) => match entry.image_mime {
            Some(m) => m,
            None => return Response::error("Not Found", 404),
        },
        None => return Response::error("Not Found", 404),
    };

    let bucket = ctx.env.bucket("IMAGES")?;
    let object = match bucket.get(format!("entries/{}", date)).execute().await? {
        Some(o) => o,
        None => return Response::error("Not Found", 404),
    };

    let body = match object.body() {
        Some(b) => b.response_body()?,
        None => return Response::error("Not Found", 404),
    };

    let headers = Headers::new();
    headers.set("Content-Type", &mime)?;
    headers.set("Cache-Control", "public, max-age=3600")?;

    Ok(Response::from_body(body)?.with_headers(headers))
}

/// GET /api/entries - 過去の日記一覧を取得
pub async fn get_entries(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let db: D1Database = ctx.env.d1("DB")?;

    match db::list_past_entries(&db, 100).await {
        Ok(entries) => {
            let summaries: Vec<DiaryEntrySummary> = entries
                .iter()
                .map(DiaryEntrySummary::from_entry)
                .collect();
            let response = DiaryListResponse { entries: summaries };
            Response::from_json(&response)
        }
        Err(e) => {
            worker::console_error!("Failed to list entries: {:?}", e);
            Response::from_json(&ErrorResponse::internal_error())
                .map(|r| r.with_status(500))
        }
    }
}

/// GET /api/entries/:date - 特定日の日記を取得
pub async fn get_entry_by_date(_req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let db: D1Database = ctx.env.d1("DB")?;

    let date = match ctx.param("date") {
        Some(d) => d,
        None => {
            return Response::from_json(&ErrorResponse::bad_request("Date parameter required"))
                .map(|r| r.with_status(400));
        }
    };

    // 日付の形式を検証
    if !is_valid_date(date) {
        return Response::from_json(&ErrorResponse::bad_request("Invalid date format. Use YYYY-MM-DD."))
            .map(|r| r.with_status(400));
    }

    match db::get_entry(&db, date).await {
        Ok(Some(entry)) => {
            let can_edit = is_today(date);
            let response = DiaryEntryResponse::from_entry(&entry, can_edit);
            Response::from_json(&response)
        }
        Ok(None) => {
            Response::from_json(&ErrorResponse::not_found())
                .map(|r| r.with_status(404))
        }
        Err(e) => {
            worker::console_error!("Failed to get entry: {:?}", e);
            Response::from_json(&ErrorResponse::internal_error())
                .map(|r| r.with_status(500))
        }
    }
}

/// GET /api/admin/entries/:date/versions - バージョン一覧取得（管理者用）
pub async fn admin_list_versions(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // 認証チェック
    if !auth::verify_admin_token(&req, &ctx.env)? {
        return auth::unauthorized_response();
    }

    let db: D1Database = ctx.env.d1("DB")?;

    let date = match ctx.param("date") {
        Some(d) => d,
        None => {
            return Response::from_json(&ErrorResponse::bad_request("Date parameter required"))
                .map(|r| r.with_status(400));
        }
    };

    if !is_valid_date(date) {
        return Response::from_json(&ErrorResponse::bad_request(
            "Invalid date format. Use YYYY-MM-DD.",
        ))
        .map(|r| r.with_status(400));
    }

    // 現在のエントリを取得
    let current = db::get_entry(&db, date).await?;

    // バージョン一覧を取得
    let versions = db::list_versions(&db, date).await?;

    let response = VersionListResponse {
        entry_date: date.to_string(),
        current_content: current.map(|e| e.content),
        versions: versions.iter().map(VersionSummary::from_version).collect(),
    };

    Response::from_json(&response)
}

/// GET /api/admin/entries/:date/versions/:version - 特定バージョン取得（管理者用）
pub async fn admin_get_version(req: Request, ctx: RouteContext<()>) -> Result<Response> {
    // 認証チェック
    if !auth::verify_admin_token(&req, &ctx.env)? {
        return auth::unauthorized_response();
    }

    let db: D1Database = ctx.env.d1("DB")?;

    let date = match ctx.param("date") {
        Some(d) => d,
        None => {
            return Response::from_json(&ErrorResponse::bad_request("Date parameter required"))
                .map(|r| r.with_status(400));
        }
    };

    let version: i32 = match ctx.param("version").and_then(|v| v.parse().ok()) {
        Some(v) => v,
        None => {
            return Response::from_json(&ErrorResponse::bad_request("Invalid version number"))
                .map(|r| r.with_status(400));
        }
    };

    if !is_valid_date(date) {
        return Response::from_json(&ErrorResponse::bad_request(
            "Invalid date format. Use YYYY-MM-DD.",
        ))
        .map(|r| r.with_status(400));
    }

    match db::get_version(&db, date, version).await? {
        Some(v) => {
            let response = VersionDetailResponse {
                entry_date: v.entry_date,
                version_number: v.version_number,
                content: v.content,
                created_at: v.created_at,
            };
            Response::from_json(&response)
        }
        None => Response::from_json(&ErrorResponse::not_found()).map(|r| r.with_status(404)),
    }
}
