//! CBZ 归档模块 — 将下载的图片打包为 Comic Book Zip 格式。
//!
//! 从 worker.rs 的 `process_cbz_archive_task` 提取而来，
//! 移除了数据库依赖，以便 CLI 独立使用。

use crate::util;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};
use telegrab_model::entity::doc::{ComicInfo, Doc};
use zip::write::SimpleFileOptions;

/// CBZ 归档结果
pub struct CbzArchiveResult {
    /// 生成的 CBZ 文件完整路径
    pub cbz_path: PathBuf,
    /// CBZ 文件名（不含路径）
    pub cbz_filename: String,
    /// 内部包含的文件数
    pub file_count: usize,
}

/// 将指定目录下的图片打包为 CBZ 文件
///
/// # Arguments
/// * `doc` — 文档信息（用于生成 ComicInfo.xml 和 CBZ 文件名）
/// * `pic_dir` — 包含已下载图片的目录
/// * `cbz_dir` — CBZ 输出目录
/// * `page_count` — 总页数
pub async fn archive_to_cbz(
    doc: &Doc,
    pic_dir: &Path,
    cbz_dir: &Path,
    page_count: i16,
) -> Result<CbzArchiveResult, CbzArchiveError> {
    util::ensure_dir_exists(cbz_dir).await?;

    let last_path_segment = util::url_last_segment(&doc.url).unwrap_or_else(|| doc.url.clone());

    // 生成 ComicInfo.xml
    let mut doc_for_xml = doc.clone();
    doc_for_xml.page_count = Some(page_count);
    let comic_info = ComicInfo::from(doc_for_xml);
    let mut xml = String::new();
    quick_xml::se::to_writer(&mut xml, &comic_info).map_err(CbzArchiveError::XmlSerialize)?;
    let xml_with_decl = format!(r#"<?xml version="1.0" encoding="utf-8"?>{}"#, xml);

    // 生成 CBZ 文件名
    let cbz_filename = match (&doc.writer, &doc.title, &doc.page_title) {
        (Some(writer), Some(title), _) => format!("[{}]{}", writer, title),
        (_, None, Some(page_title)) => page_title.to_string(),
        _ => last_path_segment.to_string(),
    };
    let cbz_full_filename = format!("{}.cbz", cbz_filename);
    let cbz_path = cbz_dir.join(&cbz_full_filename);

    // 读取图片文件
    let files = util::get_files_in_dir(pic_dir)?;

    // 在 blocking 线程中打包
    let cbz_path_clone = cbz_path.clone();
    let xml_with_decl_clone = xml_with_decl.clone();
    let file_count = files.len() + 1; // +1 for ComicInfo.xml

    tokio::task::spawn_blocking(move || -> Result<(), CbzArchiveError> {
        let zip_file = std::fs::File::create(&cbz_path_clone)?;
        let mut zip_writer = zip::ZipWriter::new(zip_file);
        let options = SimpleFileOptions::default();

        zip_writer
            .start_file("ComicInfo.xml", options)
            .map_err(CbzArchiveError::Zip)?;
        zip_writer.write_all(xml_with_decl_clone.as_bytes())?;

        for file in &files {
            let filename = file
                .file_name()
                .ok_or_else(|| CbzArchiveError::InvalidFilename(file.clone()))?
                .to_string_lossy()
                .to_string();
            zip_writer
                .start_file(&filename, options)
                .map_err(CbzArchiveError::Zip)?;
            let f = std::fs::File::open(file)?;
            let mut reader = BufReader::new(f);
            std::io::copy(&mut reader, &mut zip_writer)?;
        }

        zip_writer.finish().map_err(CbzArchiveError::Zip)?;
        Ok(())
    })
    .await
    .map_err(|_| CbzArchiveError::JoinError)??;

    Ok(CbzArchiveResult {
        cbz_path,
        cbz_filename: cbz_full_filename,
        file_count,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum CbzArchiveError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("XML serialize error: {0}")]
    XmlSerialize(#[from] quick_xml::se::SeError),
    #[error("Zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("Invalid filename: {0}")]
    InvalidFilename(PathBuf),
    #[error("Blocking task cancelled")]
    JoinError,
}
