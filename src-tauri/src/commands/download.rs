use crate::download::manager::{DownloadManager, DownloadSource, DownloadTask, DownloadStatus};
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::State;

pub struct DownloadState(pub Arc<Mutex<DownloadManager>>);

#[tauri::command]
pub async fn get_download_source(state: State<'_, DownloadState>) -> Result<String, String> {
    let manager = state.0.lock().await;
    let source = manager.get_source().await;
    Ok(match source {
        DownloadSource::Official => "official".to_string(),
        DownloadSource::Mirror => "mirror".to_string(),
        DownloadSource::Auto => "auto".to_string(),
    })
}

#[tauri::command]
pub async fn set_download_source(source: String, state: State<'_, DownloadState>) -> Result<(), String> {
    let manage = state.0.lock().await;
    let download_source = match source.as_str() {
        "official" => DownloadSource::Official,
        "mirror" => DownloadSource::Mirror,
        _ => DownloadSource::Auto,
    };
    manage.set_source(download_source).await;
    Ok(())
}

#[tauri::command]
pub async fn add_download_task(
    id: String,
    url: String,
    path: String,
    size: Option<u64>,
    sha1: Option<String>,
    sha256: Option<String>,
    state: State<'_, DownloadState>,
) -> Result<String, String> {
    let manager = state.0.lock().await;
    let task = DownloadTask {
        id: id.clone(),
        url,
        path: std::path::PathBuf::from(path),
        size,
        sha1,
        sha256,
        status: DownloadStatus::Pending,
        progress: 0.0,
        speed: 0,
        error: None,
    };
    Ok(manager.add_task(task).await)
}

#[tauri::command]
pub async fn start_download(task_id: String, state: State<'_, DownloadState>) -> Result<(), String> {
    let manage = state.0.lock().await;
    manage.start_download(&task_id).await
}

#[tauri::command]
pub async fn get_download_status(task_id: String, state: State<'_, DownloadState>) -> Result<DownloadTask, String> {
    let manager = state.0.lock().await;
    manager.get_task_status(&task_id).await
        .ok_or_else(|| "Task not found".to_string())
}

#[tauri::command]
pub async fn get_all_downloads(state: State<'_, DownloadState>) -> Result<Vec<DownloadTask>, String> {
    let manage = state.0.lock().await;
    Ok(manage.get_all_tasks().await)
}

#[tauri::command]
pub async fn remove_download(task_id: String, state: State<'_, DownloadState>) -> Result<bool, String> {
    let manager = state.0.lock().await;
    Ok(manager.remove_task(&task_id).await)
}

#[tauri::command]
pub async fn clear_completed_downloads(state: State<'_, DownloadState>) -> Result<(), String> {
    let manage = state.0.lock().await;
    manage.clea_completed().await;
    Ok(())
}

#[tauri::command]
pub async fn retry_failed_downloads(state: State<'_, DownloadState>) -> Result<(), String> {
    let manager = state.0.lock().await;
    manager.retry_failed().await;
    Ok(())
}

#[tauri::command]
pub async fn verify_file(
    path: String,
    sha1: Option<String>,
    sha256: Option<String>,
    state: State<'_, DownloadState>,
) -> Result<bool, String> {
    let manage = state.0.lock().await;
    manage.verify_file(
        &std::path::PathBuf::from(path),
        sha1.as_deref(),
        sha256.as_deref(),
    ).await
}