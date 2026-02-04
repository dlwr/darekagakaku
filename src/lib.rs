use worker::*;

mod auth;
mod db;
mod handlers;
mod models;
mod pages;
mod rate_limit;
mod templates;
mod time;
mod turnstile;

#[event(fetch, respond_with_errors)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        // HTMLページ
        .get_async("/", pages::home)
        .get_async("/a", pages::about)
        .get_async("/feed", pages::feed)
        .get_async("/entries", pages::entries_list)
        .get_async("/entries/:date", pages::entry_page)
        // JSON API
        .get_async("/api/today", handlers::get_today)
        .post_async("/api/today", handlers::post_today)
        .get_async("/api/entries", handlers::get_entries)
        .get_async("/api/entries/:date", handlers::get_entry_by_date)
        // 管理者用HTML画面
        .get_async("/admin/versions", pages::admin_versions_index)
        .get_async("/admin/entries/:date/versions", pages::admin_versions_list)
        .get_async(
            "/admin/entries/:date/versions/:version",
            pages::admin_version_detail,
        )
        // 管理者用API
        .get_async(
            "/api/admin/entries/:date/versions",
            handlers::admin_list_versions,
        )
        .get_async(
            "/api/admin/entries/:date/versions/:version",
            handlers::admin_get_version,
        )
        .run(req, env)
        .await
}
