use crate::models::{DiaryEntry, DiaryEntrySummary};
use crate::time::today_jst;

/// 共通のエスケープ処理（HTML/XML両方で使用）
fn escape_common(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// XMLをエスケープする
fn escape_xml(s: &str) -> String {
    escape_common(s).replace('\'', "&apos;")
}

/// HTMLをエスケープする
fn escape_html(s: &str) -> String {
    escape_common(s).replace('\'', "&#x27;")
}

/// 共通のHTMLヘッダー
fn html_head(title: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} - 誰かが書く日記</title>
    <link rel="alternate" type="application/rss+xml" title="誰かが書く日記 RSS" href="/feed">
    <style>
        * {{ box-sizing: border-box; margin: 0; padding: 0; }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Hiragino Sans", "Noto Sans CJK JP", sans-serif;
            max-width: 800px;
            margin: 0 auto;
            padding: 20px;
            line-height: 1.6;
            background-color: #fafafa;
            color: #333;
        }}
        h1 {{
            font-size: 1.8em;
            margin-bottom: 10px;
            color: #2c3e50;
        }}
        nav {{
            margin-bottom: 20px;
            padding-bottom: 10px;
            border-bottom: 1px solid #ddd;
        }}
        nav a {{
            color: #3498db;
            text-decoration: none;
            margin-right: 15px;
        }}
        nav a:hover {{ text-decoration: underline; }}
        .date {{
            color: #666;
            font-size: 0.95em;
            margin-bottom: 15px;
        }}
        textarea {{
            width: 100%;
            height: 300px;
            font-size: 16px;
            padding: 15px;
            border: 1px solid #ddd;
            border-radius: 8px;
            resize: vertical;
            font-family: inherit;
            line-height: 1.6;
        }}
        textarea:focus {{
            outline: none;
            border-color: #3498db;
            box-shadow: 0 0 0 3px rgba(52, 152, 219, 0.1);
        }}
        button {{
            padding: 12px 24px;
            font-size: 16px;
            cursor: pointer;
            background-color: #3498db;
            color: white;
            border: none;
            border-radius: 6px;
            margin-top: 15px;
        }}
        button:hover {{ background-color: #2980b9; }}
        .hint {{
            font-size: 0.85em;
            color: #888;
            margin-top: 10px;
        }}
        .entry-list {{
            list-style: none;
        }}
        .entry-list li {{
            padding: 15px;
            margin-bottom: 10px;
            background: white;
            border-radius: 8px;
            border: 1px solid #eee;
        }}
        .entry-list li:hover {{
            border-color: #3498db;
        }}
        .entry-list a {{
            text-decoration: none;
            color: inherit;
            display: block;
        }}
        .entry-date {{
            font-weight: bold;
            color: #2c3e50;
            margin-bottom: 5px;
        }}
        .entry-preview {{
            color: #666;
            font-size: 0.9em;
        }}
        .content {{
            background: white;
            padding: 20px;
            border-radius: 8px;
            border: 1px solid #eee;
            white-space: pre-wrap;
            word-wrap: break-word;
        }}
        .empty {{
            color: #888;
            font-style: italic;
            padding: 40px;
            text-align: center;
        }}
    </style>
</head>
<body>"#,
        title = escape_html(title)
    )
}

/// 共通のナビゲーション
fn html_nav() -> &'static str {
    r#"<nav>
        <a href="/">今日の日記を書く</a>
        <a href="/entries">過去の日記</a>
        <a href="/a">これはなにか</a>
        <a href="/feed">RSS</a>
    </nav>"#
}

/// HTMLフッター
fn html_footer() -> &'static str {
    "</body></html>"
}

/// ホームページ（今日の日記フォーム）
pub fn render_home(entry: Option<&DiaryEntry>) -> String {
    let today = today_jst();
    let content = entry.map(|e| escape_html(&e.content)).unwrap_or_default();

    format!(
        r#"{head}
    {nav}
    <h1>誰かが書く日記</h1>
    <p class="date">{today}の日記</p>
    <form method="POST" action="/">
        <textarea name="content" placeholder="今日の日記を書いてください...">{content}</textarea>
        <br>
        <button type="submit">保存する</button>
    </form>
    <p class="hint">0時（JST）になると編集できなくなります</p>
{footer}"#,
        head = html_head("今日の日記"),
        nav = html_nav(),
        today = today,
        content = content,
        footer = html_footer()
    )
}

/// 過去の日記一覧ページ
pub fn render_archive(entries: &[DiaryEntrySummary]) -> String {
    let entries_html = if entries.is_empty() {
        r#"<p class="empty">まだ過去の日記はありません</p>"#.to_string()
    } else {
        let items: Vec<String> = entries
            .iter()
            .map(|e| {
                format!(
                    r#"<li><a href="/entries/{date}">
                        <div class="entry-date">{date}</div>
                        <div class="entry-preview">{preview}</div>
                    </a></li>"#,
                    date = escape_html(&e.date),
                    preview = escape_html(&e.preview)
                )
            })
            .collect();
        format!(r#"<ul class="entry-list">{}</ul>"#, items.join("\n"))
    };

    format!(
        r#"{head}
    {nav}
    <h1>過去の日記</h1>
    {entries}
{footer}"#,
        head = html_head("過去の日記"),
        nav = html_nav(),
        entries = entries_html,
        footer = html_footer()
    )
}

/// 個別の日記ページ（閲覧専用）
pub fn render_entry(entry: &DiaryEntry, can_edit: bool) -> String {
    let edit_link = if can_edit {
        r#"<p><a href="/">編集する</a></p>"#
    } else {
        ""
    };

    format!(
        r#"{head}
    {nav}
    <h1>{date}の日記</h1>
    <div class="content">{content}</div>
    {edit_link}
{footer}"#,
        head = html_head(&format!("{}の日記", entry.date)),
        nav = html_nav(),
        date = escape_html(&entry.date),
        content = escape_html(&entry.content),
        edit_link = edit_link,
        footer = html_footer()
    )
}

/// 404ページ
pub fn render_not_found() -> String {
    format!(
        r#"{head}
    {nav}
    <h1>日記が見つかりません</h1>
    <p class="empty">この日の日記は存在しません。</p>
{footer}"#,
        head = html_head("見つかりません"),
        nav = html_nav(),
        footer = html_footer()
    )
}

/// Aboutページ（これはなにか）
pub fn render_about() -> String {
    format!(
        r#"{head}
    {nav}
    <h1>これはなにか</h1>
    <div class="content">
        <p>「自分が書かなければおそらく誰かが書く日記」</p>
        <p>ここでは日記をつけることができます。しかしその日記は日付がかわるその瞬間までインターネットにアクセスできるすべての存在（それは人間とも限りません）が書くことができます。</p>
        <p>どこかの誰か（重ねていいますがそれは人間とは限りません）が書き残したものを跡形もなく消し去って、今まさにテクストエリアにフォーカスを持っている存在のその瞬間を記録することができます。どんな美しい言葉でさえも今という瞬間に存在する言葉を超えることはできないのです。</p>
        <p>たとえそれがどんなに汚ない言葉でも例外はありません。</p>
        <p>日付を越えるという経験をした言葉は（スーパーユーザではない限り）2度と手をいれることのできない存在になります。</p>
    </div>
    <p style="text-align: right; margin-top: 20px;"><a href="/">トップ</a></p>
{footer}"#,
        head = html_head("これはなにか"),
        nav = html_nav(),
        footer = html_footer()
    )
}

/// RSSフィードを生成
pub fn render_rss(entries: &[DiaryEntry], base_url: &str) -> String {
    let items: Vec<String> = entries
        .iter()
        .map(|entry| {
            let description = if entry.content.chars().count() > 200 {
                let preview: String = entry.content.chars().take(200).collect();
                format!("{}...", preview)
            } else {
                entry.content.clone()
            };

            format!(
                r#"    <item>
      <title>{date}の日記</title>
      <link>{base_url}/entries/{date}</link>
      <guid>{base_url}/entries/{date}</guid>
      <pubDate>{pub_date}</pubDate>
      <description>{description}</description>
    </item>"#,
                date = escape_xml(&entry.date),
                base_url = base_url,
                pub_date = date_to_rfc2822(&entry.date),
                description = escape_xml(&description)
            )
        })
        .collect();

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>誰かが書く日記</title>
    <link>{base_url}</link>
    <description>自分が書かなければおそらく誰かが書く日記</description>
    <language>ja</language>
{items}
  </channel>
</rss>"#,
        base_url = base_url,
        items = items.join("\n")
    )
}

const MONTH_NAMES: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun",
    "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

const WEEKDAY_NAMES: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

/// YYYY-MM-DD形式の日付をRFC2822形式に変換
fn date_to_rfc2822(date: &str) -> String {
    let parts: Vec<&str> = date.split('-').collect();
    if parts.len() != 3 {
        return date.to_string();
    }

    let year: i32 = parts[0].parse().unwrap_or(2025);
    let month: u32 = parts[1].parse().unwrap_or(1);
    let day: u32 = parts[2].parse().unwrap_or(1);

    let month_name = MONTH_NAMES.get((month - 1) as usize).unwrap_or(&"Jan");
    let weekday = calculate_weekday(year, month, day);
    let weekday_name = WEEKDAY_NAMES.get(weekday as usize).unwrap_or(&"Sun");

    format!(
        "{}, {:02} {} {} 00:00:00 +0900",
        weekday_name, day, month_name, year
    )
}

/// 曜日を計算（0=日曜〜6=土曜）
fn calculate_weekday(year: i32, month: u32, day: u32) -> u32 {
    let y = if month <= 2 { year - 1 } else { year };
    let m = if month <= 2 { month + 12 } else { month };
    let d = day as i32;

    let q = y / 100;
    let r = y % 100;

    let h = (d + (13 * (m as i32 + 1)) / 5 + r + r / 4 + q / 4 - 2 * q) % 7;
    ((h + 7) % 7) as u32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_rss_empty() {
        let rss = render_rss(&[], "https://example.com");
        assert!(rss.contains("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));
        assert!(rss.contains("<title>誰かが書く日記</title>"));
        assert!(rss.contains("<link>https://example.com</link>"));
    }

    #[test]
    fn test_render_rss_with_entries() {
        let entries = vec![
            DiaryEntry {
                date: "2025-01-15".to_string(),
                content: "今日はいい天気だった".to_string(),
                created_at: "2025-01-15T10:00:00Z".to_string(),
                updated_at: "2025-01-15T10:00:00Z".to_string(),
            },
        ];
        let rss = render_rss(&entries, "https://example.com");
        assert!(rss.contains("<title>2025-01-15の日記</title>"));
        assert!(rss.contains("<link>https://example.com/entries/2025-01-15</link>"));
        assert!(rss.contains("<description>今日はいい天気だった</description>"));
    }

    #[test]
    fn test_render_rss_escapes_xml() {
        let entries = vec![
            DiaryEntry {
                date: "2025-01-15".to_string(),
                content: "<script>alert('xss')</script>".to_string(),
                created_at: "2025-01-15T10:00:00Z".to_string(),
                updated_at: "2025-01-15T10:00:00Z".to_string(),
            },
        ];
        let rss = render_rss(&entries, "https://example.com");
        assert!(rss.contains("&lt;script&gt;"));
        assert!(!rss.contains("<script>"));
    }

    #[test]
    fn test_render_rss_truncates_long_content() {
        let long_content = "あ".repeat(300);
        let entries = vec![
            DiaryEntry {
                date: "2025-01-15".to_string(),
                content: long_content,
                created_at: "2025-01-15T10:00:00Z".to_string(),
                updated_at: "2025-01-15T10:00:00Z".to_string(),
            },
        ];
        let rss = render_rss(&entries, "https://example.com");
        // 200文字 + "..." = 203文字分のエスケープされた内容が含まれる
        assert!(rss.contains("..."));
    }

    #[test]
    fn test_date_to_rfc2822() {
        let rfc = date_to_rfc2822("2025-01-15");
        assert!(rfc.contains("Jan"));
        assert!(rfc.contains("2025"));
        assert!(rfc.contains("+0900"));
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("<test>"), "&lt;test&gt;");
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("\"quote\""), "&quot;quote&quot;");
    }
}
