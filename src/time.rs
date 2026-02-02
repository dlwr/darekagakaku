use chrono::{DateTime, NaiveDate};
use chrono_tz::Asia::Tokyo;

/// JavaScriptのDate.now()からミリ秒を取得
fn js_now_millis() -> i64 {
    js_sys::Date::now() as i64
}

/// 現在のUTC DateTimeを取得
fn now_utc() -> DateTime<chrono::Utc> {
    let millis = js_now_millis();
    let secs = millis / 1000;
    let nsecs = ((millis % 1000) * 1_000_000) as u32;
    DateTime::from_timestamp(secs, nsecs).unwrap()
}

/// 現在の日付をJSTでYYYY-MM-DD形式の文字列として返す
pub fn today_jst() -> String {
    let now_utc = now_utc();
    let now_jst = now_utc.with_timezone(&Tokyo);
    now_jst.format("%Y-%m-%d").to_string()
}

/// 現在時刻をISO8601形式で返す
pub fn now_iso8601() -> String {
    now_utc().to_rfc3339()
}

/// 指定された日付が今日かどうかを判定する
pub fn is_today(date: &str) -> bool {
    date == today_jst()
}

/// 日付文字列をパースする
pub fn parse_date(date: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(date, "%Y-%m-%d").ok()
}

/// 日付が有効な形式かどうかを検証する
pub fn is_valid_date(date: &str) -> bool {
    parse_date(date).is_some()
}
