use worker::kv::KvStore;
use worker::{Request, Result};

const MAX_REQUESTS: u32 = 60;
const WINDOW_SECONDS: u64 = 3600;

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
    Ok(count >= MAX_REQUESTS)
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
