-- 誰かが書く日記 - データベーススキーマ
CREATE TABLE IF NOT EXISTS diary_entries (
    date TEXT PRIMARY KEY,           -- YYYY-MM-DD (JST)
    content TEXT NOT NULL,           -- 日記本文 (最大10000文字)
    created_at TEXT NOT NULL,        -- ISO8601タイムスタンプ
    updated_at TEXT NOT NULL         -- ISO8601タイムスタンプ
) STRICT;

CREATE INDEX IF NOT EXISTS idx_entries_date_desc
ON diary_entries(date DESC);
