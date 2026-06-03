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

fn linkify(text: &str) -> String {
    use std::fmt::Write;

    let mut result = String::with_capacity(text.len());
    let mut rest = text;

    while let Some(pos) = rest.find("http") {
        let candidate = &rest[pos..];

        // "https://" (8 chars) or "http://" (7 chars) のみマッチ
        let scheme_len = if candidate.starts_with("https://") {
            8
        } else if candidate.starts_with("http://") {
            7
        } else {
            result.push_str(&rest[..pos + 4]);
            rest = &rest[pos + 4..];
            continue;
        };

        // 空白・改行・< でURLの終端を判定
        let url_end = candidate[scheme_len..]
            .find(|c: char| c.is_whitespace() || c == '<')
            .map_or(candidate.len(), |i| i + scheme_len);

        let url = &candidate[..url_end];

        result.push_str(&rest[..pos]);
        let _ = write!(
            result,
            r#"<a href="{url}" target="_blank" rel="noopener noreferrer">{url}</a>"#
        );

        rest = &rest[pos + url_end..];
    }

    result.push_str(rest);
    result
}

fn truncate_for_description(content: &str, max_chars: usize) -> String {
    let trimmed = content.trim();
    if trimmed.is_empty() {
        return "この日の日記".to_string();
    }
    if trimmed.chars().count() <= max_chars {
        trimmed.to_string()
    } else {
        let truncated: String = trimmed.chars().take(max_chars).collect();
        format!("{}...", truncated)
    }
}

fn html_head(title: &str, description: Option<&str>, path: &str, og_image: Option<&str>) -> String {
    let default_description = "誰でも書ける共有日記。日付が変わると編集できなくなります。";
    let desc = escape_html(description.unwrap_or(default_description));
    let full_title = format!("{} - 誰かが書く日記", escape_html(title));
    let url = format!("https://darekagakaku.day{}", path);

    let twitter_card = if og_image.is_some() {
        "summary_large_image"
    } else {
        "summary"
    };

    let og_image_tags = match og_image {
        Some(img) => {
            // アップロード画像(/images/...)はWebPでサイズ可変。型のみ宣言し寸法はクローラーに委ねる。
            // 生成OGカード(/og/*.png)は1200x630 PNG固定。
            let type_and_dims = if img.contains("/images/") {
                r#"    <meta property="og:image:type" content="image/webp">"#.to_string()
            } else {
                r#"    <meta property="og:image:type" content="image/png">
    <meta property="og:image:width" content="1200">
    <meta property="og:image:height" content="630">"#
                    .to_string()
            };
            format!(
                r#"    <meta property="og:image" content="{img}">
{type_and_dims}
    <meta name="twitter:image" content="{img}">"#
            )
        }
        None => String::new(),
    };

    format!(
        r#"<!DOCTYPE html>
<html lang="ja">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta property="og:title" content="{full_title}">
    <meta property="og:description" content="{desc}">
    <meta property="og:type" content="website">
    <meta property="og:url" content="{url}">
    <meta property="og:site_name" content="誰かが書く日記">
{og_image_tags}
    <meta name="twitter:card" content="{twitter_card}">
    <meta name="twitter:title" content="{full_title}">
    <meta name="twitter:description" content="{desc}">
    <title>{full_title}</title>
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
        .masonry {{
            display: grid;
            grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
            column-gap: 14px;
            align-items: start;
        }}
        /* JS無効時のフォールバック: 通常グリッド（行は揃うが左→右の時系列は保つ） */
        .masonry:not(.is-masonry) {{
            row-gap: 14px;
        }}
        /* JS有効時: 8px単位の行に各カードの高さ分をspanさせて隙間を詰める */
        .masonry.is-masonry {{
            grid-auto-rows: 8px;
        }}
        .masonry-card {{
            display: block;
            margin: 0;
            background: white;
            border: 1px solid #eee;
            border-radius: 10px;
            overflow: hidden;
            text-decoration: none;
            color: inherit;
            transition: transform 0.12s ease, box-shadow 0.12s ease, border-color 0.12s ease;
        }}
        .masonry-card:hover {{
            transform: translateY(-2px);
            box-shadow: 0 6px 18px rgba(0,0,0,0.08);
            border-color: #3498db;
        }}
        .masonry-card img {{
            display: block;
            width: 100%;
            height: auto;
        }}
        .masonry-body {{
            padding: 12px 14px;
        }}
        .masonry-date {{
            font-weight: bold;
            font-size: 0.85em;
            color: #2c3e50;
        }}
        .masonry-preview {{
            margin-top: 6px;
            font-size: 0.9em;
            color: #555;
            white-space: pre-wrap;
            word-wrap: break-word;
            display: -webkit-box;
            -webkit-line-clamp: 10;
            -webkit-box-orient: vertical;
            overflow: hidden;
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
        .entry-image {{
            margin: 20px 0;
        }}
        .entry-image img {{
            max-width: 100%;
            height: auto;
            border-radius: 4px;
        }}
        .textarea-wrap {{
            position: relative;
        }}
        .textarea-wrap.has-image textarea {{
            background-size: cover;
            background-position: center;
            background-repeat: no-repeat;
            color: #1a1a1a;
        }}
        .textarea-wrap.has-image textarea::placeholder {{
            color: #555;
        }}
        #image-section {{
            margin: 12px 0;
            display: flex;
            gap: 8px;
        }}
        #image-section button {{
            margin-top: 0;
            padding: 8px 14px;
            font-size: 13px;
            background-color: #7f8c8d;
        }}
        #image-section button:hover {{
            background-color: #6c7a7b;
        }}
        #image-section #delete-image-btn {{
            background-color: #c0392b;
        }}
        #image-section #delete-image-btn:hover {{
            background-color: #a93226;
        }}
        .file-button {{
            display: inline-block;
            padding: 10px 18px;
            font-size: 14px;
            color: #555;
            background-color: #f0f1f3;
            border: 1px dashed #bbb;
            border-radius: 6px;
            cursor: pointer;
        }}
        .file-button:hover {{
            background-color: #e8e9ec;
        }}
    </style>
</head>
<body>"#,
        full_title = full_title,
        desc = desc,
        url = url,
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
    let has_image = entry.and_then(|e| e.image_mime.as_ref()).is_some();
    let today_esc = escape_html(&today);

    let (wrap_class, textarea_style, image_section) = if has_image {
        (
            " has-image",
            format!(
                "background-image: linear-gradient(rgba(255,255,255,0.85), rgba(255,255,255,0.85)), url('/images/{today}');",
                today = today_esc
            ),
            r#"<div id="image-section">
        <button type="button" id="replace-image-btn">画像を差し替え</button>
        <button type="button" id="delete-image-btn">画像を削除</button>
        <input type="file" id="image-input" accept="image/jpeg,image/png,image/webp" hidden>
    </div>"#
                .to_string(),
        )
    } else {
        (
            "",
            String::new(),
            r#"<div id="image-section">
        <label for="image-input" class="file-button">画像を追加（任意・1枚）</label>
        <input type="file" id="image-input" accept="image/jpeg,image/png,image/webp" hidden>
    </div>"#
                .to_string(),
        )
    };

    format!(
        r#"{head}
    {nav}
    <h1>誰かが書く日記</h1>
    <p class="date">{today}の日記</p>
    <form id="diary-form">
        <div class="textarea-wrap{wrap_class}">
            <textarea name="content" placeholder="今日の日記を書いてください..." style="{textarea_style}">{content}</textarea>
        </div>
        {image_section}
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
    function getTurnstileToken() {{
        return turnstileWidgetId ? turnstile.getResponse(turnstileWidgetId) : null;
    }}
    function showToast(msg) {{
        var toast = document.createElement('div');
        toast.className = 'toast';
        toast.textContent = msg;
        document.body.appendChild(toast);
        setTimeout(function() {{ toast.remove(); }}, 3000);
    }}
    document.getElementById('diary-form').addEventListener('submit', function(e) {{
        e.preventDefault();
        var form = this;
        var btn = form.querySelector('button[type="submit"]');
        var token = getTurnstileToken();
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
                showToast('保存しました');
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

    // 長辺2048pxに縮小し、WebP(品質0.85)で再エンコードする。
    // 透過を保持するためWebPを使用。失敗時は元ファイルをそのまま返す。
    function compressImage(file) {{
        return new Promise(function(resolve) {{
            var url = URL.createObjectURL(file);
            var img = new Image();
            img.onload = function() {{
                URL.revokeObjectURL(url);
                var maxSide = 2048;
                var w = img.naturalWidth, h = img.naturalHeight;
                var scale = Math.min(1, maxSide / Math.max(w, h));
                var canvas = document.createElement('canvas');
                canvas.width = Math.round(w * scale);
                canvas.height = Math.round(h * scale);
                canvas.getContext('2d').drawImage(img, 0, 0, canvas.width, canvas.height);
                canvas.toBlob(function(blob) {{
                    resolve(blob || file);
                }}, 'image/webp', 0.85);
            }};
            img.onerror = function() {{ URL.revokeObjectURL(url); resolve(file); }};
            img.src = url;
        }});
    }}

    function uploadImage(file) {{
        if (!file) return;
        var token = getTurnstileToken();
        if (!token) {{
            alert('認証処理中です。少々お待ちください。');
            return;
        }}
        compressImage(file).then(function(blob) {{
            if (blob.size > 3 * 1024 * 1024) {{
                alert('画像サイズが大きすぎます（圧縮後も3MBを超えています）');
                return;
            }}
            var fd = new FormData();
            fd.append('image', blob, 'image.webp');
            fd.append('turnstile_token', token);
            return fetch('/api/today/image', {{ method: 'POST', body: fd }})
                .then(function(res) {{
                    if (res.ok) {{
                        showToast('画像をアップロードしました');
                        turnstile.reset(turnstileWidgetId);
                        setTimeout(function() {{ location.reload(); }}, 800);
                    }} else if (res.status === 413) {{
                        alert('画像サイズが大きすぎます（3MBまで）');
                    }} else if (res.status === 400) {{
                        alert('画像形式が不正です（JPEG/PNG/WebPのみ）');
                    }} else {{
                        alert('アップロードに失敗しました');
                    }}
                }});
        }}).catch(function() {{ alert('アップロードに失敗しました'); }});
    }}

    var imageInput = document.getElementById('image-input');
    if (imageInput) {{
        imageInput.addEventListener('change', function() {{
            uploadImage(imageInput.files && imageInput.files[0]);
        }});
    }}

    var replaceBtn = document.getElementById('replace-image-btn');
    if (replaceBtn && imageInput) {{
        replaceBtn.addEventListener('click', function() {{
            imageInput.click();
        }});
    }}

    var deleteBtn = document.getElementById('delete-image-btn');
    if (deleteBtn) {{
        deleteBtn.addEventListener('click', function() {{
            if (!confirm('画像を削除しますか？')) return;
            var token = getTurnstileToken();
            if (!token) {{
                alert('認証処理中です。少々お待ちください。');
                return;
            }}
            deleteBtn.disabled = true;
            fetch('/api/today/image', {{
                method: 'DELETE',
                headers: {{ 'Content-Type': 'application/json' }},
                body: JSON.stringify({{ turnstile_token: token }})
            }})
                .then(function(res) {{
                    if (res.ok) {{
                        showToast('画像を削除しました');
                        turnstile.reset(turnstileWidgetId);
                        setTimeout(function() {{ location.reload(); }}, 800);
                    }} else {{
                        alert('削除に失敗しました');
                    }}
                }})
                .catch(function() {{ alert('削除に失敗しました'); }})
                .finally(function() {{ deleteBtn.disabled = false; }});
        }});
    }}
    </script>
    <script src="https://challenges.cloudflare.com/turnstile/v0/api.js?render=explicit&onload=initTurnstile" async defer></script>
{footer}"#,
        head = html_head("今日の日記", None, "/", Some("https://darekagakaku.day/og/default.png")),
        nav = html_nav(),
        today = today,
        content = content,
        wrap_class = wrap_class,
        textarea_style = textarea_style,
        image_section = image_section,
        turnstile_key = turnstile_key,
        footer = html_footer()
    )
}

/// マソンリーグリッドの1カードを描画する。
/// 画像エントリは画像を、テキストのみのエントリはプレビュー文を表示し、
/// 日付は常に表示する。本文が空（画像のみ）の場合はプレビューdivを省く。
fn render_archive_card(e: &DiaryEntrySummary) -> String {
    let date = escape_html(&e.date);

    let preview_block = if e.preview.trim().is_empty() {
        String::new()
    } else {
        format!(
            r#"<div class="masonry-preview">{}</div>"#,
            escape_html(&e.preview)
        )
    };

    let image_block = if e.has_image {
        format!(r#"<img src="/images/{date}" alt="" loading="lazy">"#)
    } else {
        String::new()
    };

    let card_class = if e.has_image {
        "masonry-card has-image"
    } else {
        "masonry-card"
    };

    format!(
        r#"<a class="{card_class}" href="/entries/{date}">{image_block}<div class="masonry-body"><div class="masonry-date">{date}</div>{preview_block}</div></a>"#
    )
}

pub fn render_archive(entries: &[DiaryEntrySummary]) -> String {
    let (entries_html, script) = if entries.is_empty() {
        (
            r#"<p class="empty">まだ過去の日記はありません</p>"#.to_string(),
            String::new(),
        )
    } else {
        let cards: Vec<String> = entries.iter().map(render_archive_card).collect();
        (
            format!(r#"<div class="masonry">{}</div>"#, cards.join("\n")),
            masonry_script().to_string(),
        )
    };

    format!(
        r#"{head}
    {nav}
    <h1>過去の日記</h1>
    {entries}
    {script}
{footer}"#,
        head = html_head("過去の日記", Some("過去の日記一覧"), "/entries", Some("https://darekagakaku.day/og/default.png")),
        nav = html_nav(),
        entries = entries_html,
        script = script,
        footer = html_footer()
    )
}

/// 行方向（左→右が新しい順）のマソンリーを実現するスクリプト。
/// 各カードの実際の高さから 8px 行の span 数を計算してグリッドに詰める。
/// JSが無効・実行前は通常グリッド（行が揃う）として読めるフォールバックになる。
fn masonry_script() -> &'static str {
    r#"<script>
    (function() {
        var grid = document.querySelector('.masonry');
        if (!grid) return;
        var ROW = 8, GAP = 14;
        function layout() {
            grid.classList.remove('is-masonry');
            var cards = grid.querySelectorAll('.masonry-card');
            var heights = [];
            for (var i = 0; i < cards.length; i++) {
                heights.push(cards[i].getBoundingClientRect().height);
            }
            grid.classList.add('is-masonry');
            for (var j = 0; j < cards.length; j++) {
                var span = Math.ceil((heights[j] + GAP) / ROW);
                cards[j].style.gridRowEnd = 'span ' + span;
            }
        }
        layout();
        window.addEventListener('resize', layout);
        var imgs = grid.querySelectorAll('img');
        for (var k = 0; k < imgs.length; k++) {
            if (!imgs[k].complete) imgs[k].addEventListener('load', layout);
        }
    })();
    </script>"#
}

pub fn render_entry(entry: &DiaryEntry, can_edit: bool) -> String {
    let edit_link = if can_edit {
        r#"<p><a href="/">編集する</a></p>"#
    } else {
        ""
    };

    let image_html = if entry.image_mime.is_some() {
        format!(
            r#"<div class="entry-image"><img src="/images/{date}" alt=""></div>"#,
            date = escape_html(&entry.date)
        )
    } else {
        String::new()
    };

    // 画像がある日記はアップロード画像をOGPに、なければ生成OGカードを使う
    let og_image = if entry.image_mime.is_some() {
        format!("https://darekagakaku.day/images/{}", entry.date)
    } else {
        format!("https://darekagakaku.day/og/{}.png", entry.date)
    };

    format!(
        r#"{head}
    {nav}
    <h1>{date}の日記</h1>
    {image_html}
    <div class="content">{content}</div>
    {edit_link}
{footer}"#,
        head = html_head(
            &format!("{}の日記", entry.date),
            Some(&truncate_for_description(&entry.content, 150)),
            &format!("/entries/{}", entry.date),
            Some(&og_image),
        ),
        nav = html_nav(),
        date = escape_html(&entry.date),
        image_html = image_html,
        content = linkify(&escape_html(&entry.content)),
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
        head = html_head("見つかりません", None, "/", Some("https://darekagakaku.day/og/default.png")),
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
        head = html_head(
            "これはなにか",
            Some("「自分が書かなければおそらく誰かが書く日記」についての説明"),
            "/a",
            Some("https://darekagakaku.day/og/default.png"),
        ),
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

fn admin_nav() -> String {
    r#"<nav>
        <a href="/admin/versions">バージョン履歴</a>
        <a href="/admin/logout">ログアウト</a>
        <a href="/">トップページ</a>
    </nav>"#
        .to_string()
}

pub fn render_admin_versions_index() -> String {
    let today = today_jst();
    format!(
        r#"{head}
    {nav}
    <h1>バージョン履歴 - 管理者ページ</h1>
    <form method="get" action="/admin/entries/{today}/versions">
        <label for="date">日付を入力:</label>
        <input type="date" id="date" name="date" value="{today}" required
               onchange="this.form.action='/admin/entries/'+this.value+'/versions'">
        <button type="submit">表示</button>
    </form>
{footer}"#,
        head = html_head("バージョン履歴", None, "/admin/versions", None),
        nav = admin_nav(),
        today = today,
        footer = html_footer()
    )
}

pub fn render_admin_versions_list(
    date: &str,
    current_content: Option<&str>,
    versions: &[VersionSummary],
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
                    r#"<li><a href="/admin/entries/{date}/versions/{version}">
                        <div class="entry-date">バージョン {version} ({created_at})</div>
                        <div class="entry-preview">{preview}</div>
                    </a></li>"#,
                    date = escape_html(date),
                    version = v.version_number,
                    created_at = escape_html(&v.created_at),
                    preview = escape_html(&v.preview),
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
    <p><a href="/admin/versions">別の日付を選択</a></p>
{footer}"#,
        head = html_head(
            &format!("{} バージョン履歴", date),
            None,
            &format!("/admin/entries/{}/versions", date),
            None,
        ),
        nav = admin_nav(),
        date = escape_html(date),
        current = current_html,
        versions = versions_html,
        footer = html_footer()
    )
}

pub fn render_admin_version_detail(version: &DiaryVersion) -> String {
    format!(
        r#"{head}
    {nav}
    <h1>{date}の日記 - バージョン {version_number}</h1>
    <p class="date">保存日時: {created_at}</p>
    <div class="content">{content}</div>
    <p><a href="/admin/entries/{date}/versions">バージョン一覧に戻る</a></p>
{footer}"#,
        head = html_head(
            &format!("{} バージョン{}", version.entry_date, version.version_number),
            None,
            &format!("/admin/entries/{}/versions/{}", version.entry_date, version.version_number),
            None,
        ),
        nav = admin_nav(),
        date = escape_html(&version.entry_date),
        version_number = version.version_number,
        created_at = escape_html(&version.created_at),
        content = escape_html(&version.content),
        footer = html_footer()
    )
}

pub fn render_admin_login(error: Option<&str>) -> String {
    let error_html = error
        .map(|e| format!(r#"<p class="error">{}</p>"#, escape_html(e)))
        .unwrap_or_default();

    format!(
        r#"{head}
    <nav><a href="/">トップページ</a></nav>
    <h1>管理者ログイン</h1>
    {error}
    <form method="post" action="/admin/login">
        <label for="token">管理者トークン:</label>
        <input type="password" id="token" name="token" required autocomplete="off">
        <button type="submit">ログイン</button>
    </form>
{footer}"#,
        head = html_head("管理者ログイン", None, "/admin/login", None),
        error = error_html,
        footer = html_footer()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_archive_uses_masonry_container() {
        let entries = vec![DiaryEntrySummary {
            date: "2025-01-15".to_string(),
            preview: "テキストの日記".to_string(),
            has_image: false,
        }];
        let html = render_archive(&entries);
        assert!(html.contains(r#"<div class="masonry">"#));
    }

    #[test]
    fn test_render_archive_masonry_css_present() {
        let html = render_archive(&[]);
        assert!(html.contains(".masonry {"));
        assert!(html.contains("display: grid"));
    }

    #[test]
    fn test_render_archive_includes_masonry_script() {
        // 左→右（行方向）のマソンリーは各カードの高さから行スパンを計算するJSで実現する
        let entries = vec![DiaryEntrySummary {
            date: "2025-01-15".to_string(),
            preview: "本文".to_string(),
            has_image: false,
        }];
        let html = render_archive(&entries);
        assert!(html.contains("gridRowEnd"));
        assert!(html.contains("is-masonry"));
    }

    #[test]
    fn test_render_archive_image_entry_renders_img() {
        let entries = vec![DiaryEntrySummary {
            date: "2025-01-15".to_string(),
            preview: "".to_string(),
            has_image: true,
        }];
        let html = render_archive(&entries);
        assert!(html.contains(r#"<img src="/images/2025-01-15""#));
        assert!(html.contains(r#"loading="lazy""#));
    }

    #[test]
    fn test_render_archive_text_entry_renders_preview() {
        let entries = vec![DiaryEntrySummary {
            date: "2025-01-15".to_string(),
            preview: "今日のできごと".to_string(),
            has_image: false,
        }];
        let html = render_archive(&entries);
        assert!(html.contains("今日のできごと"));
        assert!(!html.contains(r#"<img src="/images/"#));
    }

    #[test]
    fn test_render_archive_image_only_entry_omits_preview_block() {
        let entries = vec![DiaryEntrySummary {
            date: "2025-01-15".to_string(),
            preview: "".to_string(),
            has_image: true,
        }];
        let html = render_archive(&entries);
        // 画像のみ（本文なし）のときは空のプレビューdivを出さない
        assert!(!html.contains(r#"class="masonry-preview""#));
        // 日付は常に表示する
        assert!(html.contains(r#"class="masonry-date">2025-01-15"#));
    }

    #[test]
    fn test_render_archive_escapes_preview() {
        let entries = vec![DiaryEntrySummary {
            date: "2025-01-15".to_string(),
            preview: "<script>alert('x')</script>".to_string(),
            has_image: false,
        }];
        let html = render_archive(&entries);
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains("<script>alert"));
    }

    #[test]
    fn test_render_archive_empty_shows_message() {
        let html = render_archive(&[]);
        assert!(html.contains("まだ過去の日記はありません"));
    }

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
            image_mime: None,
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
            image_mime: None,
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
            image_mime: None,
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
        let head = html_head("テスト", None, "/", None);
        assert!(head.contains(".toast {"));
        assert!(head.contains("toast-slide-in"));
        assert!(head.contains("toast-fade-out"));
    }

    #[test]
    fn test_truncate_for_description_short() {
        let result = truncate_for_description("短い日記", 150);
        assert_eq!(result, "短い日記");
    }

    #[test]
    fn test_truncate_for_description_long() {
        let long_content = "あ".repeat(200);
        let result = truncate_for_description(&long_content, 150);
        assert_eq!(result.chars().count(), 153); // 150 + "..."
        assert!(result.ends_with("..."));
    }

    #[test]
    fn test_truncate_for_description_empty() {
        let result = truncate_for_description("", 150);
        assert_eq!(result, "この日の日記");
    }

    #[test]
    fn test_truncate_for_description_whitespace_only() {
        let result = truncate_for_description("  \n\t  ", 150);
        assert_eq!(result, "この日の日記");
    }

    #[test]
    fn test_html_head_contains_ogp_tags() {
        let html = html_head("テスト", Some("説明文"), "/test", None);
        assert!(html.contains(r#"<meta property="og:title" content="テスト - 誰かが書く日記">"#));
        assert!(html.contains(r#"<meta property="og:description" content="説明文">"#));
        assert!(html.contains(r#"<meta property="og:type" content="website">"#));
        assert!(html.contains(r#"<meta property="og:url" content="https://darekagakaku.day/test">"#));
        assert!(html.contains(r#"<meta property="og:site_name" content="誰かが書く日記">"#));
        assert!(html.contains(r#"<meta name="twitter:card" content="summary">"#));
    }

    #[test]
    fn test_html_head_default_description() {
        let html = html_head("テスト", None, "/", None);
        assert!(html.contains("誰でも書ける共有日記"));
    }

    #[test]
    fn test_html_head_escapes_description() {
        let html = html_head("テスト", Some("<script>alert('xss')</script>"), "/", None);
        assert!(html.contains("&lt;script&gt;"));
        assert!(!html.contains(r#"content="<script>"#));
    }

    #[test]
    fn test_html_head_with_og_image() {
        let html = html_head(
            "テスト",
            None,
            "/test",
            Some("https://darekagakaku.day/og/default.png"),
        );
        assert!(html.contains(
            r#"<meta property="og:image" content="https://darekagakaku.day/og/default.png">"#
        ));
        assert!(html.contains(r#"<meta property="og:image:width" content="1200">"#));
        assert!(html.contains(r#"<meta property="og:image:height" content="630">"#));
        assert!(html.contains(
            r#"<meta name="twitter:image" content="https://darekagakaku.day/og/default.png">"#
        ));
        assert!(html.contains(r#"<meta name="twitter:card" content="summary_large_image">"#));
    }

    #[test]
    fn test_html_head_without_og_image() {
        let html = html_head("テスト", None, "/test", None);
        assert!(!html.contains("og:image"));
        assert!(!html.contains("twitter:image"));
        assert!(html.contains(r#"<meta name="twitter:card" content="summary">"#));
    }

    #[test]
    fn test_linkify_no_url() {
        assert_eq!(linkify("普通のテキスト"), "普通のテキスト");
    }

    #[test]
    fn test_linkify_https_url() {
        assert_eq!(
            linkify("見て https://example.com いいね"),
            r#"見て <a href="https://example.com" target="_blank" rel="noopener noreferrer">https://example.com</a> いいね"#
        );
    }

    #[test]
    fn test_linkify_http_url() {
        assert_eq!(
            linkify("http://example.com"),
            r#"<a href="http://example.com" target="_blank" rel="noopener noreferrer">http://example.com</a>"#
        );
    }

    #[test]
    fn test_linkify_multiple_urls() {
        let result = linkify("https://a.com と https://b.com");
        assert!(result.contains(r#"<a href="https://a.com""#));
        assert!(result.contains(r#"<a href="https://b.com""#));
    }

    #[test]
    fn test_linkify_preserves_escaped_html() {
        let result = linkify("text &amp; https://example.com");
        assert!(result.contains("&amp;"));
        assert!(result.contains(r#"<a href="https://example.com""#));
    }

    #[test]
    fn test_linkify_url_with_path_and_query() {
        let result = linkify("https://example.com/path?q=1&amp;r=2 end");
        assert!(result.contains(r#"<a href="https://example.com/path?q=1&amp;r=2""#));
    }

    #[test]
    fn test_render_entry_links_urls() {
        let entry = DiaryEntry {
            date: "2025-01-15".to_string(),
            content: "見て https://example.com いいね".to_string(),
            created_at: "2025-01-15T10:00:00Z".to_string(),
            updated_at: "2025-01-15T10:00:00Z".to_string(),
            image_mime: None,
        };
        let html = render_entry(&entry, false);
        assert!(html.contains(r#"<a href="https://example.com""#));
    }

    #[test]
    fn test_render_entry_has_ogp_description() {
        let entry = DiaryEntry {
            date: "2025-01-15".to_string(),
            content: "これは日記の内容です。".to_string(),
            created_at: "2025-01-15T10:00:00Z".to_string(),
            updated_at: "2025-01-15T10:00:00Z".to_string(),
            image_mime: None,
        };
        let html = render_entry(&entry, false);
        assert!(html.contains(r#"og:description" content="これは日記の内容です。"#));
        assert!(html.contains(r#"og:url" content="https://darekagakaku.day/entries/2025-01-15"#));
    }

    #[test]
    fn test_render_entry_without_image_uses_og_card() {
        let entry = DiaryEntry {
            date: "2025-01-15".to_string(),
            content: "本文".to_string(),
            created_at: "2025-01-15T10:00:00Z".to_string(),
            updated_at: "2025-01-15T10:00:00Z".to_string(),
            image_mime: None,
        };
        let html = render_entry(&entry, false);
        assert!(html.contains(
            r#"<meta property="og:image" content="https://darekagakaku.day/og/2025-01-15.png">"#
        ));
    }

    #[test]
    fn test_render_entry_with_image_uses_uploaded_image_for_og() {
        let entry = DiaryEntry {
            date: "2025-01-15".to_string(),
            content: "本文".to_string(),
            created_at: "2025-01-15T10:00:00Z".to_string(),
            updated_at: "2025-01-15T10:00:00Z".to_string(),
            image_mime: Some("image/webp".to_string()),
        };
        let html = render_entry(&entry, false);
        assert!(html.contains(
            r#"<meta property="og:image" content="https://darekagakaku.day/images/2025-01-15">"#
        ));
    }

    #[test]
    fn test_render_entry_with_image_declares_webp_type() {
        let entry = DiaryEntry {
            date: "2025-01-15".to_string(),
            content: "本文".to_string(),
            created_at: "2025-01-15T10:00:00Z".to_string(),
            updated_at: "2025-01-15T10:00:00Z".to_string(),
            image_mime: Some("image/webp".to_string()),
        };
        let html = render_entry(&entry, false);
        assert!(html.contains(r#"<meta property="og:image:type" content="image/webp">"#));
    }

    #[test]
    fn test_render_entry_with_image_omits_fixed_dimensions() {
        let entry = DiaryEntry {
            date: "2025-01-15".to_string(),
            content: "本文".to_string(),
            created_at: "2025-01-15T10:00:00Z".to_string(),
            updated_at: "2025-01-15T10:00:00Z".to_string(),
            image_mime: Some("image/webp".to_string()),
        };
        let html = render_entry(&entry, false);
        assert!(!html.contains(r#"<meta property="og:image:width""#));
    }
}
