use crate::configuration::Settings;
use crate::graceful::{GracefulShutdown, TaskGuard};
use crate::http_client::HttpClientManager;
use crate::model::entity::doc::ComicInfo;
use crate::model::entity::task::{QueueEvent, TaskStatus, TaskType};
use crate::state::{AppState, QueueState};
use crate::{service, Result};
use sqlx::PgPool;
use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use zip::write::SimpleFileOptions;

#[derive(Debug, Clone)]
pub struct TaskWorker {
    queue_state: QueueState,
    shutdown: Arc<GracefulShutdown>,
    http_client: Arc<HttpClientManager>,
    db_pool: Arc<PgPool>,
    worker_id: usize,
    pic_dir: String,
    cbz_dir: String,
}

impl TaskWorker {
    pub fn new(app_state: &AppState, configuration: Settings, worker_id: usize) -> Self {
        Self {
            queue_state: app_state.queue_state.clone(),
            shutdown: app_state.shutdown.clone(),
            http_client: app_state.http_client.clone(),
            db_pool: app_state.db_pool.clone(),
            pic_dir: configuration.pic_dir.clone(),
            cbz_dir: configuration.cbz_dir.clone(),
            worker_id,
        }
    }
    pub async fn start(&self) {
        tracing::info!("Worker {} started", self.worker_id);
        let mut shutdown_rx = self.shutdown.get_shutdown_rx().await;
        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    tracing::info!("Worker {} received shutdown signal, stop to receive new tasks", self.worker_id);
                    break;
                }
                _ = async {
                    loop{
                        if self.shutdown.is_shutting_down().await{
                            tracing::info!("Worker {} is shutting down, no more waiting  for tasks", self.worker_id);
                            return;
                        }
                        let has_task = self.queue_state.wait_for_task(Some(Duration::from_secs(5))).await;
                        if has_task {
                            break;
                        } else {
                            continue;
                        }

                    }
                    match self.process_queue_with_guard().await{
                        Ok(Some(true)) => {
                            tracing::info!("Worker {} processed a task", self.worker_id);
                        }
                        Ok(Some(false)) => {
                            // empty queue, wait for next task
                        }
                        Ok(None) => {
                            tracing::info!("Worker {} is shutting down, no more new tasks", self.worker_id);
                        }
                        Err(e) => {
                            tracing::error!("Worker {} encountered an error on queue: {}", self.worker_id, e);
                            tokio::time::sleep(Duration::from_secs(1)).await;
                        }
                    }
                }=>{}
            }
        }
        tracing::info!(
            "Worker {} waiting for current tasks to finish",
            self.worker_id
        );
        self.wait_for_current_tasks().await;
        tracing::info!("Worker {} stopped", self.worker_id);
    }
    async fn process_queue_with_guard(&self) -> Result<Option<bool>, String> {
        let _guard = match TaskGuard::new(self.shutdown.clone()).await {
            Some(guard) => guard,
            None => return Ok(None),
        };

        let task = self.queue_state.dequeue().await;
        match task {
            Some(mut task) => {
                tracing::info!("Worker {} processing task: {:?}", self.worker_id, task);
                task.mark_processing();

                if !self.queue_state.update_task(task.clone()).await {
                    tracing::warn!(
                        "Worker {} can not update task {}, it may be processed by other worker",
                        self.worker_id,
                        task.id
                    );
                    return Ok(Some(false));
                }
                self.queue_state
                    .register_active_task(&task, self.worker_id)
                    .await;
                let result = match &task.task_type {
                    TaskType::HtmlParse { id: doc_id } => {
                        self.process_html_parse_task(doc_id).await
                    }
                    TaskType::PicDownload { id: doc_id } => {
                        self.process_pic_download_task(doc_id).await
                    }
                    TaskType::CbzArchive { id: doc_id } => {
                        self.process_cbz_archive_task(doc_id).await
                    }
                    TaskType::ScanDir => self.process_scan_dir_task().await,
                    TaskType::RemoveCbz { id: cbz_id } => {
                        self.process_remove_cbz_task(cbz_id).await
                    }
                };
                self.queue_state.unregister_active_task(&task.id).await;
                match result {
                    Ok(task_result) => {
                        task.mark_completed(task_result);
                        tracing::info!(
                            "Worker {} processed task {} successfully",
                            self.worker_id,
                            task.id
                        );
                    }
                    Err(err) => {
                        task.mark_failed(err.to_string());
                        tracing::warn!(
                            "Worker {} processed task {} failed: {}",
                            self.worker_id,
                            task.id,
                            err
                        );
                    }
                }
                if !self.queue_state.update_task(task.clone()).await {
                    tracing::warn!(
                        "Worker {} can not update task {} to final state",
                        self.worker_id,
                        task.id
                    );
                }
                if let Err(err) = self
                    .queue_state
                    .sender
                    .send(QueueEvent::TaskRemoved(task.id.clone()))
                {
                    tracing::warn!(
                        "Worker {} send TaskRemoved event {} failed: {}",
                        self.worker_id,
                        task.id,
                        err
                    );
                }
                Ok(Some(true))
            }
            None => Ok(Some(false)),
        }
    }
    async fn process_html_parse_task(&self, id: &i32) -> Result<Option<String>> {
        let doc = service::doc::get_doc_by_id(&self.db_pool, *id).await?;
        if doc.status == 1 && doc.page_title.is_some() {
            return Ok(doc.page_title);
        }
        let telegraph_post = self.http_client.parse_telegraph_post(&doc.url).await?;
        let doc = service::doc::update_parsed_doc(&self.db_pool, *id, telegraph_post).await?;
        Ok(doc.page_title)
    }
    async fn process_pic_download_task(&self, id: &i32) -> Result<Option<String>> {
        let doc = service::doc::get_doc_by_id(&self.db_pool, *id).await?;
        let parsed_url = url::Url::parse(&doc.url).expect("Invalid url");
        let last_path_segment = parsed_url.path_segments().unwrap().last().unwrap();
        let save_dir = format!("{}/{}", self.pic_dir, last_path_segment);
        ensure_dir_exists(&save_dir).await?;
        let pics = service::pic::get_pics_by_doc_id(&self.db_pool, *id).await?;
        let total = pics.len();
        let mut succeeded = 0;
        for (i, pic) in pics.iter().enumerate() {
            let pic_url = pic.url.clone();
            let ext = pic_url.split('.').last().unwrap_or("jpg");
            let filename = format_page_filename(i, total, ext);
            let filepath = format!("{}/{}", save_dir, filename);
            if Path::new(&filepath).exists() {
                tracing::info!(
                    "Worker {} pic {} already exists, skip download",
                    self.worker_id,
                    filepath
                );
                succeeded += 1;
                continue;
            }
            if let Err(err) = self
                .http_client
                .download_file(&pic_url, filepath.as_str())
                .await
            {
                tracing::warn!(
                    "Worker {} download pic {} failed: {}",
                    self.worker_id,
                    pic_url,
                    err
                );
            } else {
                succeeded += 1;
            }
        }
        if succeeded == total {
            service::doc::update_doc_status(&self.db_pool, *id, 2).await?;
        }
        Ok(Some(format!("{},{}/{}", save_dir, succeeded, total)))
    }
    async fn process_cbz_archive_task(&self, id: &i32) -> Result<Option<String>> {
        let mut doc = service::doc::get_doc_by_id(&self.db_pool, *id).await?;
        let pics = service::pic::get_pics_by_doc_id(&self.db_pool, *id).await?;
        doc.page_count = Some(pics.len().to_string());
        let doc_xml = ComicInfo::from_doc(doc.clone());
        let mut xml = String::new();
        quick_xml::se::to_writer(&mut xml, &doc_xml).expect("Failed to serialize ComicInfo Xml");
        let xml_with_decl = format!(r#"<?xml version="1.0" encoding="utf-8"?>{}"#, xml);
        let parsed_url = url::Url::parse(&doc.url).expect("Invalid url");
        let last_path_segment = parsed_url.path_segments().unwrap().last().unwrap();
        ensure_dir_exists(&self.cbz_dir).await?;
        let pic_dir = format!("{}/{}", self.pic_dir, last_path_segment);
        let files_result = get_files_in_dir(pic_dir.as_str());
        if let Err(err) = files_result {
            tracing::warn!(
                "Worker {} get files in dir {} failed: {}",
                self.worker_id,
                pic_dir,
                err
            );
            return Ok(None);
        }
        let cbz_filename = match (doc.writer, doc.title, doc.page_title) {
            (Some(writer), Some(title), _) => format!("[{}]{}", writer, title),
            (_, None, Some(page_title)) => format!("{}", page_title),
            _ => last_path_segment.to_string(),
        };
        let zip_file_path = format!("{}/{}.cbz", self.cbz_dir, cbz_filename);
        let zip_file = std::fs::File::create(&zip_file_path)?;
        let mut zip_writer = zip::ZipWriter::new(zip_file);
        let r = zip_writer.start_file("ComicInfo.xml", SimpleFileOptions::default());
        if let Err(err) = r {
            tracing::warn!(
                "Worker {} start file {} in zip failed: {}",
                self.worker_id,
                "ComicInfo.xml",
                err
            );
            return Ok(None);
        }
        let r = zip_writer.write_all(xml_with_decl.as_bytes());
        if let Err(err) = r {
            tracing::warn!(
                "Worker {} write file {} in zip failed: {}",
                self.worker_id,
                "ComicInfo.xml",
                err
            );
            return Ok(None);
        }

        let files = files_result?;
        let simple_options = SimpleFileOptions::default();
        for file in files {
            let filename = file.file_name().unwrap().to_string_lossy().to_string();
            let r = zip_writer.start_file(filename.clone(), simple_options);
            if let Err(err) = r {
                tracing::warn!(
                    "Worker {} add file {} to zip failed: {}",
                    self.worker_id,
                    filename,
                    err
                );
                return Ok(None);
            }
            let img = std::fs::read(file)?;
            let r = zip_writer.write_all(&img);
            if let Err(err) = r {
                tracing::warn!(
                    "Worker {} write file {} in zip failed: {}",
                    self.worker_id,
                    filename,
                    err
                );
                return Ok(None);
            }
        }
        let r = zip_writer.finish();
        if let Err(err) = r {
            tracing::warn!("Worker {} finish zip file failed: {}", self.worker_id, err);
            return Ok(None);
        } else {
            service::doc::update_doc_status(&self.db_pool, *id, 3).await?;
            service::cbz::create_cbz_with_doc_id(&self.db_pool, *id, format!("{}.cbz", cbz_filename)).await?;
        }
        Ok(None)
    }
    async fn process_scan_dir_task(&self) -> Result<Option<String>> {
        let dir = PathBuf::from(self.cbz_dir.as_str());
        let mut files = HashSet::new();
        scan_dir_recursive(&dir, &mut files).await;
        for file in files {
            let filename = file.file_name().unwrap().to_string_lossy().to_string();
            let cbz_in_db = service::cbz::get_cbz_by_path(&self.db_pool, filename.clone()).await?;
            if cbz_in_db.is_none() {
                service::cbz::create_cbz(&self.db_pool, filename.clone()).await?;
            }
        }
        Ok(None)
    }
    async fn process_remove_cbz_task(&self, cbz_id: &i32) -> Result<Option<String>> {
        let cbz = service::cbz::get_cbz_by_id(&self.db_pool, *cbz_id).await?;
        let cbz_path = format!("{}/{}", self.cbz_dir, cbz.path);
        if let Err(err) = std::fs::remove_file(cbz_path) {
            tracing::warn!("Remove cbz {} failed: {}", cbz_id, err);
            return Ok(None);
        }
        service::cbz::remove_cbz_by_id(&self.db_pool, *cbz_id).await?;
        Ok(None)
    }
    async fn wait_for_current_tasks(&self) {
        let active_tasks = self.queue_state.active_task_count().await;
        if active_tasks > 0 {
            tracing::info!(
                "Worker {} waiting for {} active tasks to finish",
                self.worker_id,
                active_tasks
            );
            for i in 0..30 {
                tokio::time::sleep(Duration::from_secs(1)).await;
                let remaining = self.queue_state.active_task_count().await;
                if remaining == 0 {
                    tracing::info!("Worker {} all active tasks finished", self.worker_id);
                    return;
                }
                tracing::info!(
                    "Worker {} awaiting...({}/30s), remaining tasks: {}",
                    self.worker_id,
                    i,
                    remaining
                );
            }
            tracing::warn!(
                "Worker {} timed out waiting, force shutdown",
                self.worker_id
            );
        } else {
            tracing::info!("Worker {} no active task", self.worker_id);
        }
    }
}

pub async fn start_background_workers(state: AppState, configuration: Settings) {
    let worker_count = configuration.worker.count;
    tracing::info!("Start {} worker(s)", worker_count);
    for worker_id in 0..worker_count {
        let worker = TaskWorker::new(&state, configuration.clone(), worker_id);
        tokio::spawn(async move {
            worker.start().await;
        });
    }
}
pub async fn start_auto_cleanup_task(state: AppState, configuration: Settings) {
    let cleanup_interval = configuration.worker.auto_cleanup_interval_secs;
    let max_completed_tasks = configuration.worker.max_completed_tasks;
    tokio::spawn(async move {
        let mut shutdown_rx = state.shutdown.get_shutdown_rx().await;
        tracing::info!(
            "Start Auto Cleanup Task, cleanup in every {}s, remain {} tasks at most",
            cleanup_interval,
            max_completed_tasks
        );
        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    tracing::info!("Auto Cleanup Task received shutdown signal, stop.");
                    break;
                }
                _ = tokio::time::sleep(Duration::from_secs(cleanup_interval)) => {
                    let removed_count = state.queue_state.cleanup_completed_tasks(max_completed_tasks).await;
                    if removed_count>0{
                        tracing::info!("{} tasks cleaned", removed_count);
                        let tasks = state.queue_state.get_tasks().await;
                        let remaining_completed = tasks.iter().filter(|t| matches!(t.status, TaskStatus::Completed)).count();
                        tracing::info!("{} tasks remaining, {} tasks total.",  remaining_completed,tasks.len());
                    }
                }
            }
        }
    });
}

async fn ensure_dir_exists(p: &str) -> Result<()> {
    let pp = Path::new(p);
    if !pp.exists() {
        tokio::fs::create_dir_all(pp).await?;
    }
    Ok(())
}

fn format_page_filename(page_idx: usize, total_pages: usize, ext: &str) -> String {
    let num_digits = ((total_pages as f64).log10().floor() as usize + 1).max(3);
    format!("{:0width$}.{}", page_idx, ext, width = num_digits)
}

fn get_files_in_dir(dir_path: &str) -> Result<Vec<PathBuf>, std::io::Error> {
    let dir = Path::new(dir_path);

    if !dir.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("dir not exists: {}", dir_path),
        ));
    }

    let mut files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        // 只保留文件（排除目录）
        if path.is_file() {
            files.push(path);
        }
    }

    Ok(files)
}

async fn scan_dir_recursive(
    dir_path: &PathBuf,
    files: &mut HashSet<PathBuf>,
)  {
    if !dir_path.is_dir() {
        tracing::error!("dir not exists: {:?}", dir_path);
    }
    let entries = std::fs::read_dir(dir_path).expect("Failed to read dir");

    for entry in entries {
        if let Err(err) = entry {
            tracing::error!("Failed to read entry: {:?}", err);
            continue;
        }
        let entry = entry.expect("Failed to read entry");
        let path = entry.path();

        // 递归处理子目录
        if path.is_dir() {
            let fut = Box::pin(scan_dir_recursive(&path, files));
            fut.await;
        } else if path.is_file() && path.extension() == Some("cbz".as_ref()) {
            files.insert(path);
        }
    }
}
