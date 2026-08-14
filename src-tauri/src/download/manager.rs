use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use serde::{Deserialize, Serialize};
use reqwest::Client;
use sha1::{Sha1, Digest};
use sha2::Sha256;

pub use crate::mc::mirror::DownloadSource;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub id: String,
    pub url: String,
    pub path: PathBuf,
    pub size: Option<u64>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub status: DownloadStatus,
    pub progress: f64,
    pub speed: u64,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Completed,
    Failed,
    Veifying,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadQueue {
    pub tasks: Vec<DownloadTask>,
    pub concurrency: usize,
    pub source: DownloadSource,
}

pub struct DownloadManager {
    client: Client,
    queue: Arc<Mutex<DownloadQueue>>,
    progress_callbacks: Arc<Mutex<HashMap<String, Box<dyn Fn(f64, u64) + Send>>>>,
}

impl DownloadManager {
    pub fn new(concurrency: usize, source: DownloadSource) -> Self {
        Self {
            client: crate::mc::mirror::http_client(),
            queue: Arc::new(Mutex::new(DownloadQueue {
                tasks: Vec::new(),
                concurrency,
                source,
            })),
            progress_callbacks: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_task(&self, task: DownloadTask) -> String {
        let mut queue = self.queue.lock().await;
        queue.tasks.push(task.clone());
        task.id
    }

    pub async fn add_tasks(&self, tasks: Vec<DownloadTask>) -> Vec<String> {
        let mut queue = self.queue.lock().await;
        let mut ids = Vec::new();
        for task in tasks {
            ids.push(task.id.clone());
            queue.tasks.push(task);
        }
        ids
    }

    pub async fn start_download(&self, task_id: &str) -> Result<(), String> {
        let mut queue = self.queue.lock().await;
        let task = queue.tasks.iter_mut().find(|t| t.id == task_id);
        
        if let Some(task) = task {
            if task.status == DownloadStatus::Downloading {
                return Ok(());
            }
            task.status = DownloadStatus::Downloading;
            task.progress = 0.0;
        } else {
            return Err("Task not found".to_string());
        }

        let task = queue.tasks.iter().find(|t| t.id == task_id).cloned();
        let source = queue.source.clone();
        drop(queue);

        if let Some(task) = task {
            let client = self.client.clone();
            let queue = self.queue.clone();
            let task_id = task_id.to_string();
            
            tokio::spawn(async move {
                let result = Self::download_task(&client, &task, source).await;
                let mut queue = queue.lock().await;
                if let Some(t) = queue.tasks.iter_mut().find(|t| t.id == task_id) {
                    match result {
                        Ok(()) => {
                            t.status = DownloadStatus::Completed;
                            t.progress = 1.0;
                        }
                        Err(e) => {
                            t.status = DownloadStatus::Failed;
                            t.error = Some(e);
                        }
                    }
                }
            });
        }

        Ok(())
    }

    async fn download_task(client: &Client, task: &DownloadTask, source: DownloadSource) -> Result<(), String> {
        let url = match source {
            DownloadSource::Auto => crate::mc::mirror::mirror_url(&task.url),
            DownloadSource::Mirror => crate::mc::mirror::mirror_url(&task.url),
            DownloadSource::Official => task.url.clone(),
        };

        if let Some(prent) = task.path.parent() {
            std::fs::create_dir_all(prent).map_err(|e| e.to_string())?;
        }

        let esp = client.get(&url).send().await.map_err(|e| e.to_string())?;
        let bytes = esp.bytes().await.map_err(|e| e.to_string())?;
        
        std::fs::write(&task.path, &bytes).map_err(|e| e.to_string())?;

        if let Some(expected_sha1) = &task.sha1 {
            let actual_sha1 = Self::compute_sha1(&task.path)?;
            if actual_sha1 != *expected_sha1 {
                return Err(format!("SHA1 mismatch: expected {}, got {}", expected_sha1, actual_sha1));
            }
        }

        if let Some(expected_sha256) = &task.sha256 {
            let actual_sha256 = Self::compute_sha256(&task.path)?;
            if actual_sha256 != *expected_sha256 {
                return Err(format!("SHA256 mismatch: expected {}, got {}", expected_sha256, actual_sha256));
            }
        }

        Ok(())
    }

    pub fn compute_sha1(path: &PathBuf) -> Result<String, String> {
        let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut hashe = Sha1::new();
        std::io::copy(&mut file, &mut hashe).map_err(|e| e.to_string())?;
        Ok(format!("{:x}", hashe.finalize()))
    }

    pub fn compute_sha256(path: &PathBuf) -> Result<String, String> {
        let mut file = std::fs::File::open(path).map_err(|e| e.to_string())?;
        let mut hashe = Sha256::new();
        std::io::copy(&mut file, &mut hashe).map_err(|e| e.to_string())?;
        Ok(format!("{:x}", hashe.finalize()))
    }

    pub async fn get_task_status(&self, task_id: &str) -> Option<DownloadTask> {
        let queue = self.queue.lock().await;
        queue.tasks.iter().find(|t| t.id == task_id).cloned()
    }

    pub async fn get_all_tasks(&self) -> Vec<DownloadTask> {
        let queue = self.queue.lock().await;
        queue.tasks.clone()
    }

    pub async fn remove_task(&self, task_id: &str) -> bool {
        let mut queue = self.queue.lock().await;
        let len = queue.tasks.len();
        queue.tasks.retain(|t| t.id != task_id);
        queue.tasks.len() < len
    }

    pub async fn clea_completed(&self) {
        let mut queue = self.queue.lock().await;
        queue.tasks.retain(|t| t.status != DownloadStatus::Completed);
    }

    pub async fn retry_failed(&self) {
        let mut queue = self.queue.lock().await;
        for task in &mut queue.tasks {
            if task.status == DownloadStatus::Failed {
                task.status = DownloadStatus::Pending;
                task.error = None;
                task.progress = 0.0;
            }
        }
    }

    pub async fn set_source(&self, source: DownloadSource) {
        let mut queue = self.queue.lock().await;
        queue.source = source;
    }

    pub async fn get_source(&self) -> DownloadSource {
        let queue = self.queue.lock().await;
        queue.source.clone()
    }

    pub async fn verify_file(&self, path: &PathBuf, sha1: Option<&str>, sha256: Option<&str>) -> Result<bool, String> {
        if !path.exists() {
            return Ok(false);
        }

        if let Some(expected_sha1) = sha1 {
            let actual_sha1 = Self::compute_sha1(path)?;
            if actual_sha1 != expected_sha1 {
                return Ok(false);
            }
        }

        if let Some(expected_sha256) = sha256 {
            let actual_sha256 = Self::compute_sha256(path)?;
            if actual_sha256 != expected_sha256 {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

pub async fn download_with_progress(
    client: &Client,
    url: &str,
    path: &PathBuf,
    use_mirror: bool,
    progress_callback: Option<Box<dyn Fn(f64, u64) + Send>>,
) -> Result<(), String> {
    let actual_ul = if use_mirror {
        crate::mc::mirror::mirror_url(url)
    } else {
        url.to_string()
    };

    if let Some(prent) = path.parent() {
        std::fs::create_dir_all(prent).map_err(|e| e.to_string())?;
    }

    let esp = client.get(&actual_ul).send().await.map_err(|e| e.to_string())?;
    let total_size = esp.content_length().unwrap_or(0);
    
    let mut file = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut downloaderd: u64 = 0;
    let mut steam = esp.bytes_stream();
    
    use futures::StreamExt;
    use std::io::Write;
    
    while let Some(chunk) = steam.next().await {
        let chunk = chunk.map_err(|e| e.to_string())?;
        file.write_all(&chunk).map_err(|e| e.to_string())?;
        downloaderd += chunk.len() as u64;
        
        if let Some(ref callback) = progress_callback {
            let progress = if total_size > 0 {
                downloaderd as f64 / total_size as f64
            } else {
                0.0
            };
            callback(progress, downloaderd);
        }
    }

    Ok(())
}

pub fn create_download_task(
    id: &str,
    url: &str,
    path: &PathBuf,
    size: Option<u64>,
    sha1: Option<&str>,
    sha256: Option<&str>,
) -> DownloadTask {
    DownloadTask {
        id: id.to_string(),
        url: url.to_string(),
        path: path.clone(),
        size,
        sha1: sha1.map(|s| s.to_string()),
        sha256: sha256.map(|s| s.to_string()),
        status: DownloadStatus::Pending,
        progress: 0.0,
        speed: 0,
        error: None,
    }
}