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
         - 特定日の日記: `{base_url}/api/entries/YYYY-MM-DD`\n"
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
}
