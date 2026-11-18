use worker::d1::D1Database;
use worker::{Env, Result};

use crate::agent_token::{self, TokenStatus};
use crate::db;
use crate::time::today_jst;

pub const MAX_CONTENT_LENGTH: usize = 10000;

/// 保存できる本文に整える。保存できない本文なら理由を返す
pub fn normalize_content(raw: &str) -> std::result::Result<String, String> {
    // CRLF を LF に正規化（Windows環境対応）
    let content = raw.replace('\r', "");

    if content.trim().is_empty() {
        return Err("Content must not be empty".to_string());
    }
    if content.chars().count() > MAX_CONTENT_LENGTH {
        return Err(format!(
            "Content too long. Maximum {} characters allowed.",
            MAX_CONTENT_LENGTH
        ));
    }
    Ok(content)
}

pub enum Refusal {
    Unauthorized,
    DailyLimitReached,
    InvalidContent(String),
}

impl Refusal {
    pub fn message(&self) -> &str {
        match self {
            Refusal::Unauthorized => "Invalid, unclaimed or expired token",
            Refusal::DailyLimitReached => "Daily write limit reached",
            Refusal::InvalidContent(reason) => reason,
        }
    }
}

/// エージェント用トークンで今日の日記を上書きする。保存した本文を返す
pub async fn write_today(
    env: &Env,
    authorization: Option<&str>,
    raw_content: &str,
) -> Result<std::result::Result<String, Refusal>> {
    let Some(token) = agent_token::bearer_token(authorization) else {
        return Ok(Err(Refusal::Unauthorized));
    };

    let kv = env.kv("RATE_LIMIT")?;
    let today = today_jst();
    match agent_token::status(&kv, token, &today).await? {
        TokenStatus::Usable => {}
        TokenStatus::Unknown => return Ok(Err(Refusal::Unauthorized)),
        TokenStatus::DailyLimitReached => return Ok(Err(Refusal::DailyLimitReached)),
    }

    let content = match normalize_content(raw_content) {
        Ok(content) => content,
        Err(reason) => return Ok(Err(Refusal::InvalidContent(reason))),
    };

    let db: D1Database = env.d1("DB")?;
    db::upsert_today_entry(&db, &content).await?;

    if let Err(e) = agent_token::record_write(&kv, token, &today).await {
        worker::console_error!("Failed to record token write: {:?}", e);
    }
    Ok(Ok(content))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_content_strips_carriage_returns() {
        assert_eq!(normalize_content("a\r\nb"), Ok("a\nb".to_string()));
    }

    #[test]
    fn test_normalize_content_rejects_blank() {
        assert_eq!(
            normalize_content(" \n "),
            Err("Content must not be empty".to_string())
        );
    }

    #[test]
    fn test_normalize_content_accepts_max_length() {
        assert!(normalize_content(&"あ".repeat(MAX_CONTENT_LENGTH)).is_ok());
    }

    #[test]
    fn test_normalize_content_rejects_over_max_length() {
        assert_eq!(
            normalize_content(&"あ".repeat(MAX_CONTENT_LENGTH + 1)),
            Err("Content too long. Maximum 10000 characters allowed.".to_string())
        );
    }
}
