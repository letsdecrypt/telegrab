use crate::Result;
use anyhow::Context;
use reqwest::Client;
use scraper::{Html, Selector};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TelegraphPost {
    pub title: String,
    pub date: Option<String>,
    pub image_urls: Vec<String>,
}

pub async fn parse_telegraph_post(client: &Client, url: &str) -> Result<TelegraphPost> {
    // 获取网页内容
    let html_content = client.get(url).send().await?.text().await?;

    // 解析HTML
    let document = Html::parse_document(&html_content);

    // 提取标题
    let title_selector = Selector::parse("h1").expect("Failed to parse h1 selector");
    let title = document
        .select(&title_selector)
        .next()
        .context("Failed to find title")
        .expect("Failed to find title element")
        .text()
        .collect::<String>()
        .trim()
        .to_string();

    // 提取日期
    let date_selector = Selector::parse("time").expect("Failed to parse time selector");
    let date = document.select(&date_selector).next().map(|element| {
        let s = element.text().collect::<String>().trim().to_string();
        let l = element.attr("datetime");
        if let Some(datetime) = l {
            datetime.to_string()
        } else {
            s
        }
    });

    // 提取图片URL
    let img_selector = Selector::parse("img").expect("Failed to parse img selector");
    let mut image_urls = Vec::new();
    let mut seen_urls = HashSet::new();

    for img_element in document.select(&img_selector) {
        if let Some(src) = img_element.value().attr("src") {
            let full_url = if src.starts_with("http") {
                src.to_string()
            } else if src.starts_with("/") {
                format!("https://telegra.ph{}", src)
            } else {
                continue;
            };

            // 避免重复URL
            if seen_urls.insert(full_url.clone()) {
                image_urls.push(full_url);
            }
        }
    }

    Ok(TelegraphPost {
        title,
        date,
        image_urls,
    })
}
