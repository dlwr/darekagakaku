use sha2::{Digest, Sha256};
use wasm_bindgen::prelude::*;
use worker::kv::KvStore;
use worker::Result;

pub const TOKEN_TTL_SECONDS: u64 = 30 * 24 * 60 * 60;
pub const DAILY_WRITE_LIMIT: u32 = 10;
const COUNT_TTL_SECONDS: u64 = 2 * 24 * 60 * 60;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = crypto, js_name = randomUUID)]
    fn random_uuid() -> String;
}

#[derive(Debug, PartialEq)]
pub enum TokenStatus {
    Usable,
    Unknown,
    DailyLimitReached,
}

pub fn bearer_token(authorization: Option<&str>) -> Option<&str> {
    let (scheme, token) = authorization?.split_once(' ')?;
    let token = token.trim();
    (scheme.eq_ignore_ascii_case("Bearer") && !token.is_empty()).then_some(token)
}

fn is_write_limited(count: u32) -> bool {
    count >= DAILY_WRITE_LIMIT
}

fn token_hash(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

fn token_key(token: &str) -> String {
    format!("token:{}", token_hash(token))
}

fn count_key(token: &str, date: &str) -> String {
    format!("token_writes:{}:{}", token_hash(token), date)
}

async fn write_count(kv: &KvStore, token: &str, date: &str) -> Result<u32> {
    Ok(kv
        .get(&count_key(token, date))
        .text()
        .await?
        .and_then(|v| v.parse().ok())
        .unwrap_or(0))
}

pub async fn issue(kv: &KvStore) -> Result<String> {
    let token = format!("dk_{}", random_uuid().replace('-', ""));
    kv.put(&token_key(&token), "1")?
        .expiration_ttl(TOKEN_TTL_SECONDS)
        .execute()
        .await?;
    Ok(token)
}

pub async fn status(kv: &KvStore, token: &str, date: &str) -> Result<TokenStatus> {
    if kv.get(&token_key(token)).text().await?.is_none() {
        return Ok(TokenStatus::Unknown);
    }
    if is_write_limited(write_count(kv, token, date).await?) {
        return Ok(TokenStatus::DailyLimitReached);
    }
    Ok(TokenStatus::Usable)
}

// KVは読んでから書くまでが不可分ではないので、同時書き込みでは上限を数回超えうる。
pub async fn record_write(kv: &KvStore, token: &str, date: &str) -> Result<()> {
    let count = write_count(kv, token, date).await?;
    kv.put(&count_key(token, date), (count + 1).to_string())?
        .expiration_ttl(COUNT_TTL_SECONDS)
        .execute()
        .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bearer_token_extracts_token() {
        assert_eq!(bearer_token(Some("Bearer dk_abc")), Some("dk_abc"));
    }

    #[test]
    fn test_bearer_token_scheme_is_case_insensitive() {
        assert_eq!(bearer_token(Some("bearer dk_abc")), Some("dk_abc"));
    }

    #[test]
    fn test_bearer_token_none_without_header() {
        assert_eq!(bearer_token(None), None);
    }

    #[test]
    fn test_bearer_token_none_for_other_scheme() {
        assert_eq!(bearer_token(Some("Basic dXNlcjpwYXNz")), None);
    }

    #[test]
    fn test_bearer_token_none_when_token_is_empty() {
        assert_eq!(bearer_token(Some("Bearer ")), None);
    }

    #[test]
    fn test_is_write_limited_under_limit() {
        assert!(!is_write_limited(DAILY_WRITE_LIMIT - 1));
    }

    #[test]
    fn test_is_write_limited_at_limit() {
        assert!(is_write_limited(DAILY_WRITE_LIMIT));
    }

    #[test]
    fn test_token_key_does_not_contain_raw_token() {
        assert!(!token_key("dk_secret").contains("dk_secret"));
    }

    #[test]
    fn test_token_key_is_sha256_of_token() {
        assert_eq!(
            token_key("abc"),
            "token:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn test_count_key_is_per_date() {
        assert_ne!(
            count_key("dk_a", "2025-01-15"),
            count_key("dk_a", "2025-01-16")
        );
    }

    #[test]
    fn test_count_key_is_per_token() {
        assert_ne!(
            count_key("dk_a", "2025-01-15"),
            count_key("dk_b", "2025-01-15")
        );
    }

    #[test]
    fn test_count_key_does_not_contain_raw_token() {
        assert!(!count_key("dk_secret", "2025-01-15").contains("dk_secret"));
    }
}
