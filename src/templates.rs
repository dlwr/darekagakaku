use crate::models::{DiaryEntry, DiaryEntrySummary};
use crate::time::today_jst;

/// HTMLをエスケープする
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
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
