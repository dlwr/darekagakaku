use worker::d1::{D1Database, D1Type};
use worker::Result;

use crate::models::{DiaryEntry, DiaryVersion};
use crate::time::{now_iso8601, today_jst};

/// 指定日の日記エントリを取得
pub async fn get_entry(db: &D1Database, date: &str) -> Result<Option<DiaryEntry>> {
    let stmt = db.prepare("SELECT date, content, created_at, updated_at FROM diary_entries WHERE date = ?1");
    let stmt = stmt.bind_refs(&D1Type::Text(date))?;
    stmt.first::<DiaryEntry>(None).await
}

/// 今日の日記エントリを作成または更新（変更がある場合はバージョンを保存）
pub async fn upsert_today_entry(db: &D1Database, content: &str) -> Result<()> {
    let today = today_jst();
    let now = now_iso8601();

    // 既存エントリを取得
    let existing = get_entry(db, &today).await?;

    // 既存エントリがあり、内容が異なる場合のみバージョンを保存
    if let Some(entry) = existing {
        if entry.content != content {
            save_version(db, &today, &entry.content).await?;
        }
    }

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

/// 日記のバージョンを履歴に保存
async fn save_version(db: &D1Database, date: &str, content: &str) -> Result<()> {
    let now = now_iso8601();
    let next_version = get_next_version_number(db, date).await?;

    let stmt = db.prepare(
        "INSERT INTO diary_versions (entry_date, content, version_number, created_at)
         VALUES (?1, ?2, ?3, ?4)"
    );

    let stmt = stmt.bind_refs(&[
        D1Type::Text(date),
        D1Type::Text(content),
        D1Type::Integer(next_version),
        D1Type::Text(&now),
    ])?;

    stmt.run().await?;
    Ok(())
}

/// 次のバージョン番号を取得
async fn get_next_version_number(db: &D1Database, date: &str) -> Result<i32> {
    let stmt = db.prepare(
        "SELECT COALESCE(MAX(version_number), 0) + 1 as next_version
         FROM diary_versions WHERE entry_date = ?1"
    );
    let stmt = stmt.bind_refs(&D1Type::Text(date))?;

    #[derive(serde::Deserialize)]
    struct NextVersion {
        next_version: i32,
    }

    match stmt.first::<NextVersion>(None).await? {
        Some(result) => Ok(result.next_version),
        None => Ok(1),
    }
}

/// 特定日のバージョン一覧を取得（新しい順）
pub async fn list_versions(db: &D1Database, date: &str) -> Result<Vec<DiaryVersion>> {
    let stmt = db.prepare(
        "SELECT id, entry_date, content, version_number, created_at
         FROM diary_versions
         WHERE entry_date = ?1
         ORDER BY version_number DESC"
    );
    let stmt = stmt.bind_refs(&D1Type::Text(date))?;
    let result = stmt.all().await?;
    result.results::<DiaryVersion>()
}

/// 特定バージョンを取得
pub async fn get_version(db: &D1Database, date: &str, version: i32) -> Result<Option<DiaryVersion>> {
    let stmt = db.prepare(
        "SELECT id, entry_date, content, version_number, created_at
         FROM diary_versions
         WHERE entry_date = ?1 AND version_number = ?2"
    );
    let stmt = stmt.bind_refs(&[
        D1Type::Text(date),
        D1Type::Integer(version),
    ])?;
    stmt.first::<DiaryVersion>(None).await
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

