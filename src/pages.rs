use worker::d1::D1Database;
use worker::{Headers, Request, Response, Result, RouteContext};

use crate::db;
use crate::models::DiaryEntrySummary;
use crate::templates;
use crate::time::{is_today, is_valid_date, today_jst};

/// GET /a - Aboutページ（これはなにか）
pub async fn about(_req: Request, _ctx: RouteContext<()>) -> Result<Response> {
    let html = templates::render_about();
    Response::from_html(html)
}

const MAX_CONTENT_LENGTH: usize = 10000;

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

    let html = templates::render_home(entry.as_ref());
    Response::from_html(html)
}

/// POST / - 今日の日記を保存してリダイレクト
pub async fn post_home(mut req: Request, ctx: RouteContext<()>) -> Result<Response> {
    let db: D1Database = ctx.env.d1("DB")?;

    // フォームデータをパース
    let form_data = req.form_data().await?;
    let content = match form_data.get("content") {
        Some(worker::FormEntry::Field(value)) => value,
        _ => String::new(),
    };

    // バリデーション
    if content.chars().count() > MAX_CONTENT_LENGTH {
        // エラーの場合はフォームを再表示（エラーメッセージ付き）
        let html = format!(
            r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>エラー</title></head>
<body><p>日記が長すぎます（最大{}文字）</p><a href="/">戻る</a></body></html>"#,
            MAX_CONTENT_LENGTH
        );
        return Response::from_html(html).map(|r| r.with_status(400));
    }

    // 保存
    if let Err(e) = db::upsert_today_entry(&db, &content).await {
        worker::console_error!("Failed to save entry: {:?}", e);
        let html = r#"<!DOCTYPE html>
<html><head><meta charset="UTF-8"><title>エラー</title></head>
<body><p>保存に失敗しました</p><a href="/">戻る</a></body></html>"#;
        return Response::from_html(html).map(|r| r.with_status(500));
    }

    // 成功したらホームにリダイレクト
    Response::redirect_with_status(req.url()?, 303)
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

    let entries = match db::list_all_entries(&db, 20).await {
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
