use serde::{Deserialize, Serialize};

/// 日記エントリのデータ構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiaryEntry {
    pub date: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
}

/// APIレスポンス用の日記エントリ
#[derive(Debug, Serialize)]
pub struct DiaryEntryResponse {
    pub date: String,
    pub content: String,
    pub can_edit: bool,
}

impl DiaryEntryResponse {
    pub fn from_entry(entry: &DiaryEntry, can_edit: bool) -> Self {
        Self {
            date: entry.date.clone(),
            content: entry.content.clone(),
            can_edit,
        }
    }
}

/// 今日の日記がない場合のレスポンス
#[derive(Debug, Serialize)]
pub struct TodayEmptyResponse {
    pub date: String,
    pub content: Option<String>,
    pub can_edit: bool,
}

/// 日記作成/更新リクエスト
#[derive(Debug, Deserialize)]
pub struct CreateDiaryRequest {
    pub content: String,
}

/// エラーレスポンス
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

impl ErrorResponse {
    pub fn new(error: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            error: error.into(),
            code: code.into(),
        }
    }

    pub fn not_found() -> Self {
        Self::new("Entry not found", "NOT_FOUND")
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(message, "BAD_REQUEST")
    }

    pub fn internal_error() -> Self {
        Self::new("Internal server error", "INTERNAL_ERROR")
    }
}

/// 日記一覧レスポンス
#[derive(Debug, Serialize)]
pub struct DiaryListResponse {
    pub entries: Vec<DiaryEntrySummary>,
}

/// 日記一覧用のサマリ
#[derive(Debug, Serialize)]
pub struct DiaryEntrySummary {
    pub date: String,
    pub preview: String,
}

impl DiaryEntrySummary {
    pub fn from_entry(entry: &DiaryEntry) -> Self {
        // 最初の100文字をプレビューとして使用
        let preview = if entry.content.chars().count() > 100 {
            let preview: String = entry.content.chars().take(100).collect();
            format!("{}...", preview)
        } else {
            entry.content.clone()
        };
        Self {
            date: entry.date.clone(),
            preview,
        }
    }
}

/// バージョン履歴のデータ構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiaryVersion {
    pub id: i64,
    pub entry_date: String,
    pub content: String,
    pub version_number: i32,
    pub created_at: String,
}

/// バージョン一覧レスポンス
#[derive(Debug, Serialize)]
pub struct VersionListResponse {
    pub entry_date: String,
    pub current_content: Option<String>,
    pub versions: Vec<VersionSummary>,
}

/// バージョンサマリ（一覧用）
#[derive(Debug, Serialize)]
pub struct VersionSummary {
    pub version_number: i32,
    pub created_at: String,
    pub preview: String,
}

impl VersionSummary {
    pub fn from_version(version: &DiaryVersion) -> Self {
        let preview = if version.content.chars().count() > 100 {
            let preview: String = version.content.chars().take(100).collect();
            format!("{}...", preview)
        } else {
            version.content.clone()
        };
        Self {
            version_number: version.version_number,
            created_at: version.created_at.clone(),
            preview,
        }
    }
}

/// 単一バージョンレスポンス
#[derive(Debug, Serialize)]
pub struct VersionDetailResponse {
    pub entry_date: String,
    pub version_number: i32,
    pub content: String,
    pub created_at: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diary_entry_summary_short_content() {
        let entry = DiaryEntry {
            date: "2025-01-15".to_string(),
            content: "短い日記".to_string(),
            created_at: "2025-01-15T00:00:00Z".to_string(),
            updated_at: "2025-01-15T00:00:00Z".to_string(),
        };
        let summary = DiaryEntrySummary::from_entry(&entry);
        assert_eq!(summary.preview, "短い日記");
    }

    #[test]
    fn test_diary_entry_summary_long_content() {
        let long_content = "あ".repeat(150);
        let entry = DiaryEntry {
            date: "2025-01-15".to_string(),
            content: long_content,
            created_at: "2025-01-15T00:00:00Z".to_string(),
            updated_at: "2025-01-15T00:00:00Z".to_string(),
        };
        let summary = DiaryEntrySummary::from_entry(&entry);
        assert!(summary.preview.ends_with("..."));
        assert_eq!(summary.preview.chars().count(), 103); // 100 + "..."
    }
}
