use crate::models::{DiaryEntry, DiaryEntrySummary, DiaryVersion, VersionSummary};
use crate::time::today_jst;

fn escape_common(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn escape_xml(s: &str) -> String {
    escape_common(s).replace('\'', "&apos;")
}

fn escape_html(s: &str) -> String {
    escape_common(s).replace('\'', "&#x27;")
}

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
        .toast {{
            position: fixed;
            top: 20px;
            right: 20px;
            padding: 15px 25px;
            background-color: #2ecc71;
            color: white;
            border-radius: 8px;
            box-shadow: 0 4px 12px rgba(0,0,0,0.15);
            font-size: 14px;
            z-index: 1000;
            animation: toast-slide-in 0.3s ease, toast-fade-out 0.3s ease 2.7s;
            opacity: 0;
            animation-fill-mode: forwards;
        }}
        .toast.error {{
            background-color: #e74c3c;
        }}
        @keyframes toast-slide-in {{
            from {{ transform: translateX(100%); opacity: 0; }}
            to {{ transform: translateX(0); opacity: 1; }}
        }}
        @keyframes toast-fade-out {{
            from {{ opacity: 1; }}
            to {{ opacity: 0; }}
        }}
    </style>
</head>
<body>"#,
        title = escape_html(title)
    )
}

fn html_nav() -> &'static str {
    r#"<nav>
        <a href="/">今日の日記を書く</a>
        <a href="/entries">過去の日記</a>
        <a href="/a">これはなにか</a>
        <a href="/feed">RSS</a>
    </nav>"#
}

fn html_footer() -> &'static str {
    "</body></html>"
}

pub fn render_home(entry: Option<&DiaryEntry>, turnstile_site_key: &str) -> String {
    let today = today_jst();
    let content = entry.map(|e| escape_html(&e.content)).unwrap_or_default();
    let turnstile_key = escape_html(turnstile_site_key);

    format!(
        r#"{head}
    {nav}
    <h1>誰かが書く日記</h1>
    <p class="date">{today}の日記</p>
    <form id="diary-form">
        <textarea name="content" placeholder="今日の日記を書いてください...">{content}</textarea>
        <br>
        <div id="turnstile-container"></div>
        <button type="submit">保存する</button>
    </form>
    <p class="hint">0時（JST）になると編集できなくなります</p>
    <script>
    var turnstileWidgetId = null;
    function initTurnstile() {{
        if (typeof turnstile !== 'undefined' && document.getElementById('turnstile-container')) {{
            turnstileWidgetId = turnstile.render('#turnstile-container', {{
                sitekey: '{turnstile_key}',
                callback: function(token) {{}},
                'error-callback': function() {{
                    console.error('Turnstile error');
                }}
            }});
        }}
    }}
    document.getElementById('diary-form').addEventListener('submit', function(e) {{
        e.preventDefault();
        var form = this;
        var btn = form.querySelector('button');
        var token = turnstileWidgetId ? turnstile.getResponse(turnstileWidgetId) : null;
        if (!token) {{
            alert('認証処理中です。少々お待ちください。');
            return;
        }}
        btn.disabled = true;
        btn.textContent = '保存中...';
        fetch('/api/today', {{
            method: 'POST',
            headers: {{ 'Content-Type': 'application/json' }},
            body: JSON.stringify({{
                content: form.content.value,
                turnstile_token: token
            }})
        }}).then(function(res) {{
            if (res.ok) {{
                var toast = document.createElement('div');
                toast.className = 'toast';
                toast.textContent = '保存しました';
                document.body.appendChild(toast);
                setTimeout(function() {{ toast.remove(); }}, 3000);
                turnstile.reset(turnstileWidgetId);
            }} else if (res.status === 429) {{
                alert('投稿制限中です。しばらくお待ちください。');
            }} else {{
                alert('保存に失敗しました');
            }}
        }}).catch(function() {{
            alert('保存に失敗しました');
        }}).finally(function() {{
            btn.disabled = false;
            btn.textContent = '保存する';
        }});
    }});
    </script>
    <script src="https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit&onload=initTurnstile" async defer></script>
{footer}"#,
        head = html_head("今日の日記"),
        nav = html_nav(),
        today = today,
        content = content,
        turnstile_key = turnstile_key,
        footer = html_footer()
    )
}

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
                pub_date = datetime_to_rfc2822(&entry.updated_at),
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

fn datetime_to_rfc2822(datetime: &str) -> String {
    if datetime.len() < 19 {
        return datetime.to_string();
    }

    let date_part = &datetime[0..10];
    let time_part = &datetime[11..19];

    let parts: Vec<&str> = date_part.split('-').collect();
    if parts.len() != 3 {
        return datetime.to_string();
    }

    let year: i32 = parts[0].parse().unwrap_or(2025);
    let month: u32 = parts[1].parse().unwrap_or(1);
    let day: u32 = parts[2].parse().unwrap_or(1);

    let time_parts: Vec<&str> = time_part.split(':').collect();
    let (hour, minute, second) = if time_parts.len() == 3 {
        (
            time_parts[0].parse().unwrap_or(0),
            time_parts[1].parse().unwrap_or(0),
            time_parts[2].parse().unwrap_or(0),
        )
    } else {
        (0, 0, 0)
    };

    let month_name = MONTH_NAMES.get((month - 1) as usize).unwrap_or(&"Jan");
    let weekday = calculate_weekday(year, month, day);
    let weekday_name = WEEKDAY_NAMES.get(weekday as usize).unwrap_or(&"Sun");

    format!(
        "{}, {:02} {} {} {:02}:{:02}:{:02} +0900",
        weekday_name, day, month_name, year, hour, minute, second
    )
}

fn calculate_weekday(year: i32, month: u32, day: u32) -> u32 {
    let y = if month <= 2 { year - 1 } else { year };
    let m = if month <= 2 { month + 12 } else { month };
    let d = day as i32;

    let q = y / 100;
    let r = y % 100;

    let h = (d + (13 * (m as i32 + 1)) / 5 + r + r / 4 + q / 4 - 2 * q) % 7;
    ((h + 7) % 7) as u32
}

fn admin_nav(token: &str) -> String {
    format!(
        r#"<nav>
        <a href="/admin/versions?token={token}">バージョン履歴</a>
        <a href="/">トップページ</a>
    </nav>"#,
        token = token
    )
}

pub fn render_admin_versions_index(token: &str) -> String {
    let today = today_jst();
    format!(
        r#"{head}
    {nav}
    <h1>バージョン履歴 - 管理者ページ</h1>
    <form method="get" action="/admin/entries/{today}/versions">
        <input type="hidden" name="token" value="{token}">
        <label for="date">日付を入力:</label>
        <input type="date" id="date" name="date" value="{today}" required
               onchange="this.form.action='/admin/entries/'+this.value+'/versions'">
        <button type="submit">表示</button>
    </form>
{footer}"#,
        head = html_head("バージョン履歴"),
        nav = admin_nav(token),
        today = today,
        token = escape_html(token),
        footer = html_footer()
    )
}

pub fn render_admin_versions_list(
    date: &str,
    current_content: Option<&str>,
    versions: &[VersionSummary],
    token: &str,
) -> String {
    let current_html = match current_content {
        Some(content) => format!(
            r#"<h2>現在の内容</h2>
            <div class="content">{}</div>"#,
            escape_html(content)
        ),
        None => r#"<p class="empty">この日付の日記はありません</p>"#.to_string(),
    };

    let versions_html = if versions.is_empty() {
        r#"<p class="empty">バージョン履歴はありません</p>"#.to_string()
    } else {
        let items: Vec<String> = versions
            .iter()
            .map(|v| {
                format!(
                    r#"<li><a href="/admin/entries/{date}/versions/{version}?token={token}">
                        <div class="entry-date">バージョン {version} ({created_at})</div>
                        <div class="entry-preview">{preview}</div>
                    </a></li>"#,
                    date = escape_html(date),
                    version = v.version_number,
                    created_at = escape_html(&v.created_at),
                    preview = escape_html(&v.preview),
                    token = escape_html(token)
                )
            })
            .collect();
        format!(r#"<ul class="entry-list">{}</ul>"#, items.join("\n"))
    };

    format!(
        r#"{head}
    {nav}
    <h1>{date}のバージョン履歴</h1>
    {current}
    <h2>過去のバージョン</h2>
    {versions}
    <p><a href="/admin/versions?token={token}">別の日付を選択</a></p>
{footer}"#,
        head = html_head(&format!("{} バージョン履歴", date)),
        nav = admin_nav(token),
        date = escape_html(date),
        current = current_html,
        versions = versions_html,
        token = escape_html(token),
        footer = html_footer()
    )
}

pub fn render_admin_version_detail(version: &DiaryVersion, token: &str) -> String {
    format!(
        r#"{head}
    {nav}
    <h1>{date}の日記 - バージョン {version_number}</h1>
    <p class="date">保存日時: {created_at}</p>
    <div class="content">{content}</div>
    <p><a href="/admin/entries/{date}/versions?token={token}">バージョン一覧に戻る</a></p>
{footer}"#,
        head = html_head(&format!(
            "{} バージョン{}",
            version.entry_date, version.version_number
        )),
        nav = admin_nav(token),
        date = escape_html(&version.entry_date),
        version_number = version.version_number,
        created_at = escape_html(&version.created_at),
        content = escape_html(&version.content),
        token = escape_html(token),
        footer = html_footer()
    )
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
    fn test_datetime_to_rfc2822() {
        let rfc = datetime_to_rfc2822("2025-01-15T10:30:45Z");
        assert!(rfc.contains("Jan"));
        assert!(rfc.contains("2025"));
        assert!(rfc.contains("10:30:45"));
        assert!(rfc.contains("+0900"));
    }

    #[test]
    fn test_datetime_to_rfc2822_preserves_time() {
        let rfc = datetime_to_rfc2822("2025-01-15T10:30:45Z");
        assert_eq!(rfc, "Thu, 15 Jan 2025 10:30:45 +0900");
    }

    #[test]
    fn test_escape_xml() {
        assert_eq!(escape_xml("<test>"), "&lt;test&gt;");
        assert_eq!(escape_xml("a & b"), "a &amp; b");
        assert_eq!(escape_xml("\"quote\""), "&quot;quote&quot;");
    }

    #[test]
    fn test_toast_css_exists() {
        let head = html_head("テスト");
        assert!(head.contains(".toast {"));
        assert!(head.contains("toast-slide-in"));
        assert!(head.contains("toast-fade-out"));
    }
}
