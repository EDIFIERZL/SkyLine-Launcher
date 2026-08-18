use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McNewsItem {
    pub title: String,
    pub image_url: String,
    pub link: String,
    #[serde(default)]
    pub published_at: Option<String>,
}

#[tauri::command]
pub async fn fetch_mc_news() -> Result<Vec<McNewsItem>, String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| format!("创建 HTTP 客户端失败: {e}"))?;

    // 并行获取文章页和 sitemap（sitemap 很大，用字符串匹配代替 DOM 解析）
    let (html_result, sitemap_result) = tokio::join!(
        client.get("https://www.minecraft.net/zh-hans/article").send(),
        client.get("https://www.minecraft.net/sitemap.xml").send()
    );

    let html = html_result
        .map_err(|e| format!("请求失败: {e}"))?
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {e}"))?;

    let sitemap_text = match sitemap_result {
        Ok(resp) => resp.text().await.unwrap_or_default(),
        Err(_) => String::new(),
    };

    // 从 sitemap 中用字符串匹配提取文章 URL（避免解析巨大的 XML DOM）
    let mut sitemap_urls: Vec<String> = Vec::new();
    let mut pos = 0;
    while let Some(start) = sitemap_text[pos..].find("<loc>") {
        let abs_start = pos + start + 5;
        if let Some(end) = sitemap_text[abs_start..].find("</loc>") {
            let url = sitemap_text[abs_start..abs_start + end].trim();
            if url.contains("/zh-hans/article/") {
                sitemap_urls.push(url.to_string());
            }
            pos = abs_start + end + 6;
        } else {
            break;
        }
    }

    let mut items = Vec::new();
    let mut seen_links = std::collections::HashSet::new();

    {
      let document = scraper::Html::parse_document(&html);

    // 1. 解析 Hero 卡片（顶部大图轮播）
    let hero_card_sel = scraper::Selector::parse(".MC_tiledHeroA_card").unwrap();
    let hero_img_sel = scraper::Selector::parse("img").unwrap();
    let hero_title_sel = scraper::Selector::parse("h2").unwrap();
    let hero_link_sel = scraper::Selector::parse("a[href*=\"/article/\"]").unwrap();

    for card in document.select(&hero_card_sel) {
        let title = card.select(&hero_title_sel).next()
            .map(|h| h.text().collect::<String>().trim().to_string())
            .unwrap_or_default();
        let image_url = card.select(&hero_img_sel).next()
            .and_then(|img| img.value().attr("src").map(|s| s.to_string()))
            .unwrap_or_default();
        let link = card.select(&hero_link_sel).find_map(|a| a.value().attr("href").map(|s| s.to_string()))
            .unwrap_or_default();
        if !title.is_empty() && !link.is_empty() && seen_links.insert(link.clone()) {
            items.push(McNewsItem { title, image_url: norm(&image_url), link: norm(&link), published_at: None });
        }
    }

    // 2. 全局扫描所有 /article/ 链接，关联内部图片
    let all_link_sel = scraper::Selector::parse("a[href*=\"/article/\"]").unwrap();
    let all_img_sel = scraper::Selector::parse("img").unwrap();

    for a in document.select(&all_link_sel) {
        let href = a.value().attr("href").unwrap_or("");
        if href.is_empty() || seen_links.contains(href) {
            continue;
        }

        // 从链接自身或内部元素提取标题
        let own_text = a.text().collect::<String>().trim().to_string();
        let title = if own_text.len() >= 3 && own_text.len() <= 200 {
            own_text
        } else {
            let inner_sel = scraper::Selector::parse("h2, h3, span, p, div").unwrap();
            a.select(&inner_sel)
                .map(|el| el.text().collect::<String>().trim().to_string())
                .find(|s| s.len() >= 3 && s.len() <= 200)
                .unwrap_or_default()
        };

        if title.is_empty() {
            continue;
        }

        // 从链接内部找图片
        let image_url = a.select(&all_img_sel).find_map(get_img_src).unwrap_or_default();

        seen_links.insert(href.to_string());
        items.push(McNewsItem { title, image_url: norm(&image_url), link: norm(href), published_at: None });
    }

    // 3. 补充：扫描所有 /content/dam/ 图片，向上查找关联的 /article/ 链接
    if items.len() < 8 {
        let img_sel = scraper::Selector::parse("img[src*=\"/content/dam/\"]").unwrap();
        for img in document.select(&img_sel) {
            let src = img.value().attr("src").unwrap_or("");
            let alt = img.value().attr("alt").unwrap_or("").trim().to_string();
            if src.is_empty() || alt.len() < 3 {
                continue;
            }
            // 查找最近的 a[href*=/article/] 作为父级
            let mut link = String::new();
            let mut current = img;
            for _ in 0..6 {
                let parent_node = current.parent();
                if parent_node.is_none() { break; }
                let parent_ref = scraper::ElementRef::wrap(parent_node.unwrap());
                if parent_ref.is_none() { break; }
                let p = parent_ref.unwrap();
                if let Some(href) = p.value().attr("href") {
                    if href.contains("/article/") {
                        link = href.to_string();
                        break;
                    }
                }
                current = p;
            }
            if link.is_empty() || seen_links.contains(&link) {
                continue;
            }
            seen_links.insert(link.clone());
            items.push(McNewsItem { title: alt, image_url: norm(src), link: norm(&link), published_at: None });
        }
    }
    }

    // 页面列表由官方前端异步加载，部分网络环境只能拿到 Hero 卡片。
    // 用官方 sitemap 作为稳定兜底，保证轮播不会退化为 3~4 条。
    if items.len() < 8 && !sitemap_urls.is_empty() {
        let article_urls: Vec<String> = sitemap_urls.into_iter()
            .filter(|url| !seen_links.contains(url))
            .collect();

        // 优先从 sitemap 中的 URL 提取标题（用 h1/h2），避免逐页网络请求
        for url in article_urls.into_iter().take(30) {
            if items.len() >= 16 { break; }
            let title_from_url = url.rsplit('/').next().unwrap_or("")
                .replace('-', " ")
                .split_whitespace()
                .map(|w| { let mut c = w.chars(); match c.next() { Some(first) => format!("{}{}", first.to_uppercase(), c.as_str()), None => String::new() } })
                .collect::<Vec<_>>()
                .join(" ");
            if title_from_url.len() < 3 { continue; }
            seen_links.insert(url.clone());
            items.push(McNewsItem { title: title_from_url, image_url: String::new(), link: url, published_at: None });
        }
    }

    // 补充：抓取文章详情页获取真实封面和标题
    let need_detail: Vec<(usize, String)> = items.iter().enumerate()
        .filter(|(_, item)| item.image_url.is_empty())
        .map(|(i, item)| (i, item.link.clone()))
        .take(16)
        .collect();

    for (idx, url) in need_detail {
        let page = match client.get(&url).send().await {
            Ok(resp) => match resp.text().await { Ok(t) => t, Err(_) => continue },
            Err(_) => continue,
        };
        let article = scraper::Html::parse_document(&page);
        let meta = |property: &str| -> String {
            let selector = scraper::Selector::parse(&format!("meta[property=\"{property}\"]")).unwrap();
            article.select(&selector).next().and_then(|el| el.value().attr("content")).unwrap_or("").to_string()
        };
        let og_title = meta("og:title");
        let og_image = meta("og:image");
        let published = meta("article:published_time");
        if !og_title.is_empty() { items[idx].title = og_title; }
        if !og_image.is_empty() { items[idx].image_url = norm(&og_image); }
        if !published.is_empty() { items[idx].published_at = Some(published); }
    }

    items.sort_by(|a, b| b.published_at.cmp(&a.published_at));

    if items.is_empty() {
        return Err("未找到资讯内容".into());
    }
    Ok(items)
}

fn get_img_src(img: scraper::ElementRef) -> Option<String> {
    let src = img.value().attr("src").unwrap_or("");
    let data_src = img.value().attr("data-src").unwrap_or("");
    let s = if !src.is_empty() && !src.starts_with("data:") { src } else { data_src };
    if s.is_empty() { None } else { Some(s.to_string()) }
}

fn norm(path: &str) -> String {
    if path.is_empty() { return String::new(); }
    if path.starts_with("http") { path.to_string() } else { format!("https://www.minecraft.net{}", path) }
}
