use std::path::{Path, PathBuf};

/// 确保目录存在，不存在则创建
pub async fn ensure_dir_exists<P: AsRef<Path>>(p: P) -> std::io::Result<()> {
    let pp = p.as_ref();
    if !pp.exists() {
        tokio::fs::create_dir_all(pp).await?;
    }
    Ok(())
}

/// 格式化图片文件名（按页码补零）
pub fn format_page_filename(page_idx: usize, total_pages: usize, ext: &str) -> String {
    let num_digits = ((total_pages as f64).log10().floor() as usize + 1).max(3);
    format!("{:0width$}.{}", page_idx, ext, width = num_digits)
}

/// 获取目录下的所有文件（排除子目录）
pub fn get_files_in_dir<P: AsRef<Path>>(dir_path: P) -> std::io::Result<Vec<PathBuf>> {
    let dir = dir_path.as_ref();

    if !dir.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("dir not exists: {}", dir.display()),
        ));
    }

    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        }
    }
    Ok(files)
}

/// 提取 URL 最后一段路径作为标识符
pub fn url_last_segment(url: &str) -> Option<String> {
    let parsed_url = url::Url::parse(url).ok()?;
    let last_path_segment = parsed_url.path_segments()?.next_back()?;
    Some(
        url::form_urlencoded::parse(last_path_segment.as_bytes())
            .map(|(key, _)| key)
            .collect(),
    )
}
