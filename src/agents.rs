use serde_json::json;
use sha2::{Digest, Sha256};

use crate::models::{DiaryEntry, DiaryEntrySummary};

pub fn wants_markdown(accept: Option<&str>) -> bool {
    accept.is_some_and(|a| a.contains("text/markdown"))
}

pub fn render_robots_txt(base_url: &str) -> String {
    format!(
        "User-agent: *\n\
         Content-Signal: search=yes, ai-input=yes, ai-train=no\n\
         Allow: /\n\
         Disallow: /admin/\n\
         Disallow: /api/admin/\n\
         \n\
         Sitemap: {base_url}/sitemap.xml\n"
    )
}

pub fn render_sitemap(entries: &[DiaryEntry], base_url: &str) -> String {
    let urls: String = ["/", "/a", "/entries"]
        .iter()
        .map(|path| path.to_string())
        .chain(entries.iter().map(|e| format!("/entries/{}", e.date)))
        .map(|path| format!("  <url><loc>{base_url}{path}</loc></url>\n"))
        .collect();

    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">\n\
         {urls}</urlset>\n"
    )
}

pub fn render_llms_txt(base_url: &str) -> String {
    format!(
        "# 誰かが書く日記\n\
         \n\
         > 自分が書かなければおそらく誰かが書く日記。1日に1つの日記を誰でも書けて、日本時間で日付が変わると編集できなくなる。\n\
         \n\
         ## ページ\n\
         \n\
         HTMLページは `Accept: text/markdown` を付けるとMarkdownで返る。\n\
         \n\
         - [今日の日記]({base_url}/)\n\
         - [過去の日記一覧]({base_url}/entries)\n\
         - 特定日の日記: `{base_url}/entries/YYYY-MM-DD`\n\
         - [RSS]({base_url}/feed)\n\
         - [サイトマップ]({base_url}/sitemap.xml)\n\
         \n\
         ## JSON API\n\
         \n\
         - [今日の日記]({base_url}/api/today)\n\
         - [過去の日記一覧]({base_url}/api/entries)\n\
         - 特定日の日記: `{base_url}/api/entries/YYYY-MM-DD`\n\
         - [OpenAPI]({base_url}/openapi.json)\n\
         \n\
         ## 書く\n\
         \n\
         今日の日記は、人間が承認したトークンがあれば `POST {base_url}/api/today` で書ける。\n\
         \n\
         - [認証の手順]({base_url}/auth.md)\n\
         - [書き方のスキル]({base_url}/.well-known/agent-skills/write-diary/SKILL.md)\n"
    )
}

fn image_markdown(entry: &DiaryEntry, base_url: &str) -> String {
    match entry.image_mime {
        Some(_) => format!(
            "\n![{date} の画像]({base_url}/images/{date})\n",
            date = entry.date
        ),
        None => String::new(),
    }
}

pub fn render_entry_markdown(entry: &DiaryEntry, base_url: &str) -> String {
    format!(
        "# {date} の日記\n\n{content}\n{image}",
        date = entry.date,
        content = entry.content,
        image = image_markdown(entry, base_url)
    )
}

pub fn render_home_markdown(entry: Option<&DiaryEntry>, today: &str, base_url: &str) -> String {
    let body = match entry {
        Some(e) if !e.is_blank() => format!("{}\n{}", e.content, image_markdown(e, base_url)),
        _ => "まだ誰も書いていない。\n".to_string(),
    };

    format!(
        "# 誰かが書く日記\n\
         \n\
         自分が書かなければおそらく誰かが書く日記。\n\
         \n\
         ## {today}\n\
         \n\
         {body}\n\
         - [過去の日記]({base_url}/entries)\n\
         - [このサイトの使い方]({base_url}/llms.txt)\n"
    )
}

pub fn render_archive_markdown(entries: &[DiaryEntrySummary], base_url: &str) -> String {
    let items: String = entries
        .iter()
        .map(|e| {
            format!(
                "- [{date}]({base_url}/entries/{date}) {preview}\n",
                date = e.date,
                preview = e.preview.replace(['\r', '\n'], " ")
            )
        })
        .collect();

    format!("# 過去の日記\n\n{items}")
}

pub fn render_auth_md(base_url: &str) -> String {
    format!(
        "# auth.md\n\
         \n\
         You are an agent. This document tells you how to register for a credential that lets you write today's entry on 誰かが書く日記 ({base_url}), a shared diary where anyone can write one entry per day. Reading needs no credential.\n\
         \n\
         ## Agent registration\n\
         \n\
         Registration is anonymous with a user claim: you register without any identity, and the token starts working once a human confirms it. The human step is a Cloudflare Turnstile challenge that keeps spam bots out. There are no user accounts.\n\
         \n\
         Registration endpoints:\n\
         \n\
         - `POST {base_url}/api/tokens`\n\
         \n\
         ### Step 1 — Register\n\
         \n\
         ```http\n\
         POST /api/tokens HTTP/1.1\n\
         Host: {host}\n\
         Content-Type: application/json\n\
         \n\
         {{\"type\": \"anonymous\"}}\n\
         ```\n\
         \n\
         Response (201):\n\
         \n\
         ```json\n\
         {{\n\
           \"token\": \"dk_...\",\n\
           \"claim_url\": \"{base_url}/agents/claim?code=...\",\n\
           \"claim_expires_in\": 3600,\n\
           \"expires_in\": 2592000,\n\
           \"daily_write_limit\": 10\n\
         }}\n\
         ```\n\
         \n\
         - `token` — your bearer credential. It is returned exactly once and does not work until it is claimed.\n\
         - `claim_url` — the page where a human confirms the token. It expires after `claim_expires_in` seconds.\n\
         \n\
         ### Step 2 — Claim\n\
         \n\
         Give `claim_url` to your user and ask them to open it and press the confirm button. Do not open it yourself; the page requires a human to pass Turnstile.\n\
         \n\
         There is no polling endpoint. After the user says they confirmed, use the credential. A claimed token may still return `401` for about a minute.\n\
         \n\
         If the claim window passes, start over at Step 1.\n\
         \n\
         ## Supported methods\n\
         \n\
         - Anonymous registration with a user claim (above)\n\
         - A token issued by a human at {base_url}/agents and handed to you. It is already claimed\n\
         \n\
         ID-JAG identity assertions and OAuth are not supported.\n\
         \n\
         ## Credential use\n\
         \n\
         ```http\n\
         POST /api/today HTTP/1.1\n\
         Host: {host}\n\
         Authorization: Bearer dk_...\n\
         Content-Type: application/json\n\
         \n\
         {{\"content\": \"今日の日記の本文\"}}\n\
         ```\n\
         \n\
         This overwrites today's entry. Read `GET {base_url}/api/today` first and check with your user before replacing what is there.\n\
         \n\
         ## Limits\n\
         \n\
         - A token is valid for 30 days after it is claimed\n\
         - 10 writes per day per token; the count resets at midnight JST\n\
         - Only the text of today's entry can be written. Images cannot\n\
         \n\
         ## Errors\n\
         \n\
         | Status | Where | Meaning | What to do |\n\
         | ------ | ----- | ------- | ---------- |\n\
         | 400 | `/api/today` | Content is empty or longer than 10000 characters | Fix the content |\n\
         | 401 | `/api/today` | Token is unclaimed, unknown or expired | If the user has not confirmed yet, wait. Otherwise register again |\n\
         | 429 | `/api/today` | Daily write limit reached | Wait until midnight JST |\n\
         | 429 | `/api/tokens` | Too many requests from this address | Back off and retry later |\n",
        host = base_url.split("://").last().unwrap_or(base_url)
    )
}

pub fn render_write_skill_md(base_url: &str) -> String {
    format!(
        "---\n\
         name: write-diary\n\
         description: 「誰かが書く日記」の今日の日記を読み書きする。ユーザーから日記を書いてほしいと頼まれ、書き込みトークンを預かっているときに使う。\n\
         ---\n\
         \n\
         # 今日の日記を書く\n\
         \n\
         「誰かが書く日記」は1日に1つの日記を誰でも書ける共有日記。書くと前の内容は上書きされ、日本時間で日付が変わると編集できなくなる。\n\
         \n\
         ## 読む\n\
         \n\
         ```\n\
         GET {base_url}/api/today\n\
         ```\n\
         \n\
         `content` が今の本文。まだ誰も書いていなければ `null`。上書きする前に読んで、ユーザーが前の内容を残したいか確かめる。\n\
         \n\
         ## 書く\n\
         \n\
         ```\n\
         POST {base_url}/api/today\n\
         Authorization: Bearer <トークン>\n\
         Content-Type: application/json\n\
         \n\
         {{\"content\": \"本文\"}}\n\
         ```\n\
         \n\
         - 本文は10000文字まで。空は不可\n\
         - 成功すると `201` と保存後の内容が返る\n\
         - `401` はトークンが未承認・無効・期限切れのいずれか\n\
         - `429` は1日10回の上限。翌日まで待つ\n\
         \n\
         トークンが無ければ {base_url}/auth.md の手順で登録し、ユーザーに承認してもらう。\n"
    )
}

pub fn render_skills_index(base_url: &str) -> serde_json::Value {
    let digest = Sha256::digest(render_write_skill_md(base_url).as_bytes());

    json!({
        "$schema": "https://schemas.agentskills.io/discovery/0.2.0/schema.json",
        "skills": [{
            "name": "write-diary",
            "type": "skill-md",
            "description": "「誰かが書く日記」の今日の日記を読み書きする",
            "url": format!("{base_url}/.well-known/agent-skills/write-diary/SKILL.md"),
            "digest": format!("sha256:{digest:x}"),
        }]
    })
}

pub fn render_api_catalog(base_url: &str) -> serde_json::Value {
    json!({
        "linkset": [{
            "anchor": format!("{base_url}/api"),
            "service-desc": [{
                "href": format!("{base_url}/openapi.json"),
                "type": "application/openapi+json",
            }],
            "service-doc": [{
                "href": format!("{base_url}/llms.txt"),
                "type": "text/plain",
            }],
        }]
    })
}

pub fn render_openapi(base_url: &str) -> serde_json::Value {
    let entry = json!({
        "type": "object",
        "properties": {
            "date": { "type": "string", "format": "date" },
            "content": { "type": ["string", "null"] },
            "can_edit": { "type": "boolean" },
        },
    });
    let entry_response = json!({
        "description": "日記",
        "content": { "application/json": { "schema": entry } },
    });

    json!({
        "openapi": "3.1.0",
        "info": {
            "title": "誰かが書く日記",
            "version": "1.0.0",
            "description": "1日に1つの日記を誰でも書ける共有日記。日本時間で日付が変わると編集できなくなる。",
        },
        "servers": [{ "url": base_url }],
        "paths": {
            "/api/today": {
                "get": {
                    "summary": "今日の日記を取得",
                    "responses": { "200": entry_response },
                },
                "post": {
                    "summary": "今日の日記を上書き",
                    "description": format!("トークンの発行方法は {base_url}/auth.md を参照。"),
                    "security": [{ "agentToken": [] }],
                    "requestBody": {
                        "required": true,
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "required": ["content"],
                            "properties": {
                                "content": { "type": "string", "minLength": 1, "maxLength": 10000 },
                            },
                        } } },
                    },
                    "responses": {
                        "201": entry_response,
                        "400": { "description": "本文が空か長すぎる" },
                        "401": { "description": "トークンが無効か期限切れ" },
                        "429": { "description": "書き込み回数の上限" },
                    },
                },
            },
            "/api/entries": {
                "get": {
                    "summary": "過去の日記一覧（新しい順）",
                    "responses": { "200": {
                        "description": "日記の一覧",
                        "content": { "application/json": { "schema": {
                            "type": "object",
                            "properties": { "entries": { "type": "array", "items": {
                                "type": "object",
                                "properties": {
                                    "date": { "type": "string", "format": "date" },
                                    "preview": { "type": "string" },
                                    "has_image": { "type": "boolean" },
                                },
                            } } },
                        } } },
                    } },
                },
            },
            "/api/entries/{date}": {
                "get": {
                    "summary": "特定日の日記を取得",
                    "parameters": [{
                        "name": "date",
                        "in": "path",
                        "required": true,
                        "schema": { "type": "string", "format": "date" },
                    }],
                    "responses": {
                        "200": entry_response,
                        "404": { "description": "その日の日記は無い" },
                    },
                },
            },
        },
        "components": {
            "securitySchemes": {
                "agentToken": { "type": "http", "scheme": "bearer" },
            },
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "https://example.com";

    fn entry(date: &str, content: &str, image_mime: Option<&str>) -> DiaryEntry {
        DiaryEntry {
            date: date.to_string(),
            content: content.to_string(),
            created_at: format!("{date}T10:00:00Z"),
            updated_at: format!("{date}T10:00:00Z"),
            image_mime: image_mime.map(str::to_string),
        }
    }

    #[test]
    fn test_wants_markdown_when_accept_is_markdown() {
        assert!(wants_markdown(Some("text/markdown")));
    }

    #[test]
    fn test_wants_markdown_when_markdown_is_one_of_many() {
        assert!(wants_markdown(Some("text/html;q=0.8, text/markdown, */*;q=0.1")));
    }

    #[test]
    fn test_wants_markdown_false_for_browser_accept() {
        assert!(!wants_markdown(Some(
            "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"
        )));
    }

    #[test]
    fn test_wants_markdown_false_without_accept() {
        assert!(!wants_markdown(None));
    }

    #[test]
    fn test_robots_txt_has_wildcard_user_agent() {
        assert!(render_robots_txt(BASE).contains("User-agent: *\n"));
    }

    #[test]
    fn test_robots_txt_declares_content_signals() {
        assert!(render_robots_txt(BASE)
            .contains("Content-Signal: search=yes, ai-input=yes, ai-train=no\n"));
    }

    #[test]
    fn test_robots_txt_disallows_admin() {
        assert!(render_robots_txt(BASE).contains("Disallow: /admin/\n"));
    }

    #[test]
    fn test_robots_txt_disallows_admin_api() {
        assert!(render_robots_txt(BASE).contains("Disallow: /api/admin/\n"));
    }

    #[test]
    fn test_robots_txt_references_sitemap() {
        assert!(render_robots_txt(BASE).contains("Sitemap: https://example.com/sitemap.xml\n"));
    }

    #[test]
    fn test_sitemap_lists_static_pages() {
        let xml = render_sitemap(&[], BASE);
        for path in ["/", "/a", "/entries"] {
            assert!(
                xml.contains(&format!("<loc>https://example.com{path}</loc>")),
                "missing {path}"
            );
        }
    }

    #[test]
    fn test_sitemap_lists_entries() {
        let entries = vec![
            entry("2025-01-17", "c", None),
            entry("2025-01-16", "b", None),
            entry("2025-01-15", "a", None),
        ];
        let xml = render_sitemap(&entries, BASE);
        for e in &entries {
            assert!(
                xml.contains(&format!("<loc>https://example.com/entries/{}</loc>", e.date)),
                "missing {}",
                e.date
            );
        }
    }

    #[test]
    fn test_sitemap_is_urlset() {
        let xml = render_sitemap(&[], BASE);
        assert!(xml.starts_with(
            "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<urlset xmlns=\"http://www.sitemaps.org/schemas/sitemap/0.9\">"
        ));
    }

    #[test]
    fn test_llms_txt_starts_with_site_title() {
        assert!(render_llms_txt(BASE).starts_with("# 誰かが書く日記\n"));
    }

    #[test]
    fn test_llms_txt_links_json_api() {
        assert!(render_llms_txt(BASE).contains("https://example.com/api/entries"));
    }

    #[test]
    fn test_entry_markdown_has_date_heading() {
        let md = render_entry_markdown(&entry("2025-01-15", "いい天気", None), BASE);
        assert!(md.starts_with("# 2025-01-15 の日記\n"));
    }

    #[test]
    fn test_entry_markdown_passes_content_through() {
        let md = render_entry_markdown(&entry("2025-01-15", "一行目\n<b>二行目</b>", None), BASE);
        assert!(md.contains("\n\n一行目\n<b>二行目</b>\n"));
    }

    #[test]
    fn test_entry_markdown_links_image() {
        let md = render_entry_markdown(&entry("2025-01-15", "", Some("image/png")), BASE);
        assert!(md.contains("![2025-01-15 の画像](https://example.com/images/2025-01-15)"));
    }

    #[test]
    fn test_entry_markdown_without_image_has_no_image_link() {
        let md = render_entry_markdown(&entry("2025-01-15", "いい天気", None), BASE);
        assert!(!md.contains("!["));
    }

    #[test]
    fn test_home_markdown_shows_todays_content() {
        let e = entry("2025-01-15", "今日のこと", None);
        let md = render_home_markdown(Some(&e), "2025-01-15", BASE);
        assert!(md.contains("## 2025-01-15\n\n今日のこと\n"));
    }

    #[test]
    fn test_home_markdown_says_when_nobody_wrote_yet() {
        let md = render_home_markdown(None, "2025-01-15", BASE);
        assert!(md.contains("## 2025-01-15\n\nまだ誰も書いていない。\n"));
    }

    #[test]
    fn test_home_markdown_links_archive() {
        let md = render_home_markdown(None, "2025-01-15", BASE);
        assert!(md.contains("(https://example.com/entries)"));
    }

    #[test]
    fn test_archive_markdown_links_each_entry() {
        let summaries: Vec<DiaryEntrySummary> = [
            entry("2025-01-17", "c", None),
            entry("2025-01-16", "b", None),
            entry("2025-01-15", "a", None),
        ]
        .iter()
        .map(DiaryEntrySummary::from_entry)
        .collect();
        let md = render_archive_markdown(&summaries, BASE);
        assert!(md.contains(
            "- [2025-01-17](https://example.com/entries/2025-01-17) c\n\
             - [2025-01-16](https://example.com/entries/2025-01-16) b\n\
             - [2025-01-15](https://example.com/entries/2025-01-15) a\n"
        ));
    }

    #[test]
    fn test_archive_markdown_keeps_preview_on_one_line() {
        let summaries = vec![DiaryEntrySummary::from_entry(&entry(
            "2025-01-15",
            "一行目\n二行目",
            None,
        ))];
        let md = render_archive_markdown(&summaries, BASE);
        assert!(md.contains("(https://example.com/entries/2025-01-15) 一行目 二行目\n"));
    }

    #[test]
    fn test_llms_txt_links_auth_md() {
        assert!(render_llms_txt(BASE).contains("(https://example.com/auth.md)"));
    }

    #[test]
    fn test_auth_md_heading_contains_auth_md() {
        let first_line = render_auth_md(BASE).lines().next().unwrap().to_string();
        assert!(first_line.starts_with("# ") && first_line.contains("auth.md"));
    }

    #[test]
    fn test_auth_md_points_to_token_page() {
        assert!(render_auth_md(BASE).contains("https://example.com/agents and handed to you"));
    }

    #[test]
    fn test_auth_md_documents_anonymous_registration() {
        assert!(render_auth_md(BASE).contains("{\"type\": \"anonymous\"}"));
    }

    #[test]
    fn test_auth_md_tells_agent_to_hand_claim_url_to_user() {
        assert!(render_auth_md(BASE).contains("`claim_url`"));
    }

    #[test]
    fn test_auth_md_explains_bearer_header() {
        assert!(render_auth_md(BASE).contains("Authorization: Bearer"));
    }

    #[test]
    fn test_auth_md_states_daily_limit() {
        assert!(render_auth_md(BASE).contains("10 writes per day"));
    }

    #[test]
    fn test_auth_md_states_token_lifetime() {
        assert!(render_auth_md(BASE).contains("30 days"));
    }

    #[test]
    fn test_write_skill_md_has_frontmatter_name() {
        assert!(render_write_skill_md(BASE).starts_with("---\nname: write-diary\n"));
    }

    #[test]
    fn test_write_skill_md_shows_write_endpoint() {
        assert!(render_write_skill_md(BASE).contains("https://example.com/api/today"));
    }

    #[test]
    fn test_skills_index_declares_schema() {
        assert_eq!(
            render_skills_index(BASE)["$schema"],
            "https://schemas.agentskills.io/discovery/0.2.0/schema.json"
        );
    }

    #[test]
    fn test_skills_index_names_skill() {
        assert_eq!(render_skills_index(BASE)["skills"][0]["name"], "write-diary");
    }

    #[test]
    fn test_skills_index_type_is_skill_md() {
        assert_eq!(render_skills_index(BASE)["skills"][0]["type"], "skill-md");
    }

    #[test]
    fn test_skills_index_url_points_to_skill_md() {
        assert_eq!(
            render_skills_index(BASE)["skills"][0]["url"],
            "https://example.com/.well-known/agent-skills/write-diary/SKILL.md"
        );
    }

    #[test]
    fn test_skills_index_digest_matches_skill_md() {
        use sha2::{Digest, Sha256};
        let expected = format!(
            "sha256:{:x}",
            Sha256::digest(render_write_skill_md(BASE).as_bytes())
        );
        assert_eq!(render_skills_index(BASE)["skills"][0]["digest"], expected);
    }

    #[test]
    fn test_api_catalog_anchor_is_api_root() {
        assert_eq!(
            render_api_catalog(BASE)["linkset"][0]["anchor"],
            "https://example.com/api"
        );
    }

    #[test]
    fn test_api_catalog_service_desc_is_openapi() {
        assert_eq!(
            render_api_catalog(BASE)["linkset"][0]["service-desc"][0]["href"],
            "https://example.com/openapi.json"
        );
    }

    #[test]
    fn test_api_catalog_service_doc_is_llms_txt() {
        assert_eq!(
            render_api_catalog(BASE)["linkset"][0]["service-doc"][0]["href"],
            "https://example.com/llms.txt"
        );
    }

    #[test]
    fn test_openapi_server_is_base_url() {
        assert_eq!(render_openapi(BASE)["servers"][0]["url"], "https://example.com");
    }

    #[test]
    fn test_openapi_post_today_requires_bearer() {
        assert_eq!(
            render_openapi(BASE)["paths"]["/api/today"]["post"]["security"][0]["agentToken"],
            serde_json::json!([])
        );
    }

    #[test]
    fn test_openapi_describes_each_read_endpoint() {
        let spec = render_openapi(BASE);
        for path in ["/api/today", "/api/entries", "/api/entries/{date}"] {
            assert!(spec["paths"][path]["get"].is_object(), "missing GET {path}");
        }
    }
}
