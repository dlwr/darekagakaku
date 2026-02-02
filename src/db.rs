use worker::d1::{D1Database, D1Type};
use worker::Result;

use crate::models::DiaryEntry;
use crate::time::{now_iso8601, today_jst};

/// 指定日の日記エントリを取得
pub async fn get_entry(db: &D1Database, date: &str) -> Result<Option<DiaryEntry>> {
    let stmt = db.prepare("SELECT date, content, created_at, updated_at FROM diary_entries WHERE date = ?1");
    let stmt = stmt.bind_refs(&D1Type::Text(date))?;
    stmt.first::<DiaryEntry>(None).await
}

/// 今日の日記エントリを作成または更新
pub async fn upsert_today_entry(db: &D1Database, content: &str) -> Result<()> {
    let today = today_jst();
    let now = now_iso8601();

    let stmt = db.prepare(
        "INSERT INTO diary_entries (date, content, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?3)
         ON CONFLICT(date) DO UPDATE SET
           content = excluded.content,
           updated_at = excluded.updated_at"
    );

    let stmt = stmt.bind_refs(&[
        D1Type::Text(&today),
        D1Type::Text(content),
        D1Type::Text(&now),
    ])?;

    stmt.run().await?;
    Ok(())
}

/// 過去の日記エントリ一覧を取得（今日を除く、新しい順）
pub async fn list_past_entries(db: &D1Database, limit: i32) -> Result<Vec<DiaryEntry>> {
    let today = today_jst();

    let stmt = db.prepare(
        "SELECT date, content, created_at, updated_at
         FROM diary_entries
         WHERE date < ?1
         ORDER BY date DESC
         LIMIT ?2"
    );

    let stmt = stmt.bind_refs(&[
        D1Type::Text(&today),
        D1Type::Integer(limit),
    ])?;

    let result = stmt.all().await?;
    result.results::<DiaryEntry>()
}

/// すべての日記エントリ一覧を取得（新しい順）
pub async fn list_all_entries(db: &D1Database, limit: i32) -> Result<Vec<DiaryEntry>> {
    let stmt = db.prepare(
        "SELECT date, content, created_at, updated_at
         FROM diary_entries
         ORDER BY date DESC
         LIMIT ?1"
    );

    let stmt = stmt.bind_refs(&D1Type::Integer(limit))?;

    let result = stmt.all().await?;
    result.results::<DiaryEntry>()
}
