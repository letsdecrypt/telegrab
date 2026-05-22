//! telegrabber — 独立命令行工具
//!
//! 抓取 telegra.ph 页面中的图片，下载并打包为 CBZ。
//!
//! 用法:
//!   telegrabber <url> [--pic-dir <dir>] [--cbz-dir <dir>]
//!
//! 示例:
//!   telegrabber https://telegra.ph/some-page-01-01 --pic-dir ./pics --cbz-dir ./cbz

use anyhow::Context;
use std::path::PathBuf;
use telegrab_core::archiver;
use telegrab_core::http_client::HttpClientManager;
use telegrab_core::util;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "telegrabber=info".into()),
        )
        .init();

    let args: Vec<String> = std::env::args().collect();
    let (url, pic_dir, cbz_dir) = parse_args(&args)?;

    tracing::info!("Fetching: {}", url);
    tracing::info!("Pic dir:  {}", pic_dir.display());
    tracing::info!("CBZ dir:  {}", cbz_dir.display());

    // 1. 创建 HTTP 客户端
    let http = HttpClientManager::new(None);

    // 2. 解析页面
    let post = http
        .parse_telegraph_post(&url)
        .await
        .context("Failed to parse telegraph post")?;
    tracing::info!("Title: {:?}, Images: {}", post.title, post.image_urls.len());

    // 3. 下载图片
    let last_segment = util::url_last_segment(&url).unwrap_or_else(|| url.clone());
    let save_dir = pic_dir.join(&last_segment);
    util::ensure_dir_exists(&save_dir).await?;

    let total = post.image_urls.len();
    for (i, img_url) in post.image_urls.iter().enumerate() {
        let ext = img_url.split('.').next_back().unwrap_or("jpg");
        let filename = util::format_page_filename(i, total, ext);
        let filepath = save_dir.join(&filename);

        if filepath.exists() {
            tracing::info!("[{}/{}] Skip existing: {}", i + 1, total, filename);
            continue;
        }

        tracing::info!("[{}/{}] Downloading: {}", i + 1, total, img_url);
        http.download_file(img_url, &filepath)
            .await
            .with_context(|| format!("Failed to download: {}", img_url))?;
    }

    // 4. 打包为 CBZ
    let doc = telegrab_model::entity::doc::Doc {
        id: 0,
        cbz_id: None,
        status: 0,
        url: url.clone(),
        page_title: Some(post.title.clone()),
        page_date: None,
        title: None,
        series: None,
        number: None,
        count: None,
        volume: None,
        summary: None,
        notes: None,
        year: None,
        month: None,
        day: None,
        writer: None,
        penciller: None,
        inker: None,
        colorist: None,
        letterer: None,
        cover_artist: None,
        editor: None,
        publisher: None,
        imprint: None,
        genre: None,
        tags: None,
        web: None,
        page_count: None,
        language: None,
        format: None,
        black_and_white: None,
        characters: None,
        teams: None,
        locations: None,
        scan_information: None,
        story_arc: None,
        series_group: None,
        age_rating: None,
        community_rating: None,
        critical_rating: None,
        created_at: time::OffsetDateTime::now_utc(),
        updated_at: time::OffsetDateTime::now_utc(),
    };

    let result = archiver::archive_to_cbz(&doc, &save_dir, &cbz_dir, total as i16)
        .await
        .context("Failed to create CBZ archive")?;

    tracing::info!(
        "CBZ created: {} ({} files)",
        result.cbz_path.display(),
        result.file_count
    );

    Ok(())
}

fn parse_args(args: &[String]) -> anyhow::Result<(String, PathBuf, PathBuf)> {
    let mut url = None;
    let mut pic_dir = PathBuf::from("./pics");
    let mut cbz_dir = PathBuf::from("./cbz");
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--pic-dir" => {
                i += 1;
                pic_dir = PathBuf::from(args.get(i).context("--pic-dir requires a value")?);
            }
            "--cbz-dir" => {
                i += 1;
                cbz_dir = PathBuf::from(args.get(i).context("--cbz-dir requires a value")?);
            }
            arg if !arg.starts_with("--") => {
                url = Some(arg.to_string());
            }
            _ => {
                anyhow::bail!("Unknown argument: {}", args[i]);
            }
        }
        i += 1;
    }

    let url = url.context("URL is required")?;
    Ok((url, pic_dir, cbz_dir))
}
