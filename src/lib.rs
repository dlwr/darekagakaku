use worker::*;

mod db;
mod handlers;
mod models;
mod pages;
mod templates;
mod time;

#[event(fetch, respond_with_errors)]
async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    Router::new()
        // HTMLページ
        .get_async("/", pages::home)
        .post_async("/", pages::post_home)
        .get_async("/a", pages::about)
        .get_async("/entries", pages::entries_list)
        .get_async("/entries/:date", pages::entry_page)
        // JSON API
        .get_async("/api/today", handlers::get_today)
        .post_async("/api/today", handlers::post_today)
        .get_async("/api/entries", handlers::get_entries)
        .get_async("/api/entries/:date", handlers::get_entry_by_date)
        .run(req, env)
        .await
}
