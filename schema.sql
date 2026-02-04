-- 誰かが書く日記 - データベーススキーマ
CREATE TABLE IF NOT EXISTS diary_entries (
    date TEXT PRIMARY KEY,           -- YYYY-MM-DD (JST)
    content TEXT NOT NULL,           -- 日記本文 (最大10000文字)
    created_at TEXT NOT NULL,        -- ISO8601タイムスタンプ
    updated_at TEXT NOT NULL         -- ISO8601タイムスタンプ
) STRICT;

CREATE INDEX IF NOT EXISTS idx_entries_date_desc
ON diary_entries(date DESC);

-- バージョン履歴テーブル
CREATE TABLE IF NOT EXISTS diary_versions (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    entry_date TEXT NOT NULL,           -- diary_entries.dateへの参照
    content TEXT NOT NULL,              -- 保存時点の内容
    version_number INTEGER NOT NULL,    -- バージョン番号（1から始まる連番）
    created_at TEXT NOT NULL,           -- このバージョンが作成された日時
    FOREIGN KEY (entry_date) REFERENCES diary_entries(date)
) STRICT;

CREATE INDEX IF NOT EXISTS idx_versions_entry_date
ON diary_versions(entry_date, version_number DESC);
