use worker::kv::KvStore;
use worker::{Request, Result};

const MAX_REQUESTS: u32 = 60;
const WINDOW_SECONDS: u64 = 3600;

/// リクエスト数がレート制限に達しているかチェック（純粋関数）
fn is_rate_limited(count: u32) -> bool {
    count >= MAX_REQUESTS
}

async fn get_count(kv: &KvStore, key: &str) -> Result<u32> {
    let count = kv
        .get(key)
        .text()
        .await?
        .and_then(|v| v.parse().ok())
        .unwrap_or(0);
    Ok(count)
}

pub async fn check_rate_limit(kv: &KvStore, ip: &str) -> Result<bool> {
    let key = format!("rate:{}", ip);
    let count = get_count(kv, &key).await?;
    Ok(is_rate_limited(count))
}

pub async fn increment_rate_limit(kv: &KvStore, ip: &str) -> Result<()> {
    let key = format!("rate:{}", ip);
    let count = get_count(kv, &key).await?;
    kv.put(&key, (count + 1).to_string())?
        .expiration_ttl(WINDOW_SECONDS)
        .execute()
        .await?;
    Ok(())
}

pub fn get_client_ip(req: &Request) -> String {
    req.headers()
        .get("CF-Connecting-IP")
        .ok()
        .flatten()
        .unwrap_or_else(|| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_requests_constant() {
        assert_eq!(MAX_REQUESTS, 60);
    }

    #[test]
    fn test_window_seconds_constant() {
        assert_eq!(WINDOW_SECONDS, 3600);
    }

    #[test]
    fn test_is_rate_limited_under_limit() {
        assert!(!is_rate_limited(0));
        assert!(!is_rate_limited(59));
    }

    #[test]
    fn test_is_rate_limited_at_limit() {
        assert!(is_rate_limited(60));
    }

    #[test]
    fn test_is_rate_limited_over_limit() {
        assert!(is_rate_limited(61));
        assert!(is_rate_limited(100));
        assert!(is_rate_limited(u32::MAX));
    }
}
