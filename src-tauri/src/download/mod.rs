pub mod multi;
pub mod manager;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Paused,
    Completed,
    Failed(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadProgress {
    pub downloaderd: u64,
    pub total: Option<u64>,
    pub speed: f64,
    pub progress: f64,
    pub status: DownloadStatus,
    pub elapsed_secs: u64,
    pub eta_secs: Option<u64>,
}

pub struct DownloadTask {
    pub url: String,
    pub path: PathBuf,
    pub size: Option<u64>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadResult {
    pub path: String,
    pub size: u64,
    pub sha1_matched: Option<bool>,
    pub sha256_matched: Option<bool>,
    pub duration_ms: u64,
    pub aveage_speed: f64,
}

pub fn verify_file_hash(path: &PathBuf, expected_sha1: Option<&str>, expected_sha256: Option<&str>) -> Result<(Option<bool>, Option<bool>), String> {
    use sha1::Digest as Sha1Digest;

    let data = std::fs::read(path).map_err(|e| format!("读取文件失败: {}", e))?;

    let sha1_result = if let Some(expected) = expected_sha1 {
        let mut hashe = sha1::Sha1::new();
        hashe.update(&data);
        let actual = hex::encode(hashe.finalize());
        Some(actual.eq_ignore_ascii_case(expected))
    } else {
        None
    };

    let sha256_result = if let Some(expected) = expected_sha256 {
        let mut hashe = sha2::Sha256::new();
        hashe.update(&data);
        let actual = hex::encode(hashe.finalize());
        Some(actual.eq_ignore_ascii_case(expected))
    } else {
        None
    };

    Ok((sha1_result, sha256_result))
}

pub async fn download_file_with_esume(
    client: &reqwest::Client,
    url: &str,
    path: &PathBuf,
    expected_sha1: Option<&str>,
    expected_sha256: Option<&str>,
    on_progress: Option<Box<dyn Fn(DownloadProgress) + Send>>,
) -> Result<DownloadResult, String> {
    let stat_time = Instant::now();

    if let Some(prent) = path.parent() {
        std::fs::create_dir_all(prent).map_err(|e| e.to_string())?;
    }

    let mut existing_size = if path.exists() {
        std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
    } else {
        0
    };

    let mut request = client.get(url);

    if existing_size > 0 {
        request = request.header("Range", format!("bytes={}-", existing_size));
    }

    let esp = request.send().await.map_err(|e| format!("下载请求失败: {}", e))?;

    let suppots_esume = esp.status() == 206;
    let total_size = esp.content_length().map(|l| l + existing_size);

    let mut file = if suppots_esume && existing_size > 0 {
        tokio::fs::OpenOptions::new()
            .append(true)
            .open(path)
            .await
            .map_err(|e| e.to_string())?
    } else {
        existing_size = 0;
        tokio::fs::File::create(path).await.map_err(|e| e.to_string())?
    };

    let mut downloaderd = existing_size;
    let mut last_progress_time = Instant::now();
    let mut last_downloaderd = downloaderd;
    let mut speed_samples: Vec<f64> = Vec::new();

    let mut steam = esp.bytes_stream();
    use futures::StreamExt;

    while let Some(chunk) = steam.next().await {
        let chunk = chunk.map_err(|e| format!("下载中断: {}", e))?;
        file.write_all(&chunk).await.map_err(|e| e.to_string())?;
        downloaderd += chunk.len() as u64;

        let now = Instant::now();
        let elapsed = now.duration_since(last_progress_time);
        if elapsed >= Duration::from_millis(100) {
            let bytes_since_last = downloaderd - last_downloaderd;
            let speed = bytes_since_last as f64 / elapsed.as_secs_f64();
            speed_samples.push(speed);

            if speed_samples.len() > 10 {
                speed_samples.remove(0);
            }

            let avg_speed = if speed_samples.is_empty() {
                0.0
            } else {
                speed_samples.iter().sum::<f64>() / speed_samples.len() as f64
            };

            let progress = total_size
                .map(|t| downloaderd as f64 / t as f64)
                .unwrap_or(0.0);

            let eta = if avg_speed > 0.0 {
                total_size.map(|t| ((t - downloaderd) as f64 / avg_speed) as u64)
            } else {
                None
            };

            if let Some(ref callback) = on_progress {
                callback(DownloadProgress {
                    downloaderd,
                    total: total_size,
                    speed: avg_speed,
                    progress,
                    status: DownloadStatus::Downloading,
                    elapsed_secs: stat_time.elapsed().as_secs(),
                    eta_secs: eta,
                });
            }

            last_progress_time = now;
            last_downloaderd = downloaderd;
        }
    }

    file.flush().await.map_err(|e| e.to_string())?;

    let (sha1_matched, sha256_matched) = verify_file_hash(path, expected_sha1, expected_sha256)?;

    if sha1_matched == Some(false) || sha256_matched == Some(false) {
        let _ = std::fs::remove_file(path);
        return Err("文件校验失败，已删除损坏的文件".to_string());
    }

    let duration = stat_time.elapsed();
    let aveage_speed = if duration.as_secs_f64() > 0.0 {
        downloaderd as f64 / duration.as_secs_f64()
    } else {
        0.0
    };

    Ok(DownloadResult {
        path: path.to_string_lossy().to_string(),
        size: downloaderd,
        sha1_matched,
        sha256_matched,
        duration_ms: duration.as_millis() as u64,
        aveage_speed,
    })
}

pub async fn download_file(client: &reqwest::Client, url: &str, path: &PathBuf) -> Result<(), String> {
    if let Some(prent) = path.parent() {
        std::fs::create_dir_all(prent).map_err(|e| e.to_string())?;
    }

    let esp = client.get(url).send().await.map_err(|e| e.to_string())?;
    let bytes = esp.bytes().await.map_err(|e| e.to_string())?;

    let mut file = tokio::fs::File::create(path).await.map_err(|e| e.to_string())?;
    file.write_all(&bytes).await.map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn test_source_speed(client: &reqwest::Client, url: &str) -> Result<f64, String> {
    let stat = Instant::now();
    let esp = client.get(url).send().await.map_err(|e| e.to_string())?;

    if !esp.status().is_success() {
        return Err(format!("HTTP {}", esp.status()));
    }

    let bytes = esp.bytes().await.map_err(|e| e.to_string())?;
    let duration = stat.elapsed();

    if duration.as_secs_f64() > 0.0 {
        Ok(bytes.len() as f64 / duration.as_secs_f64())
    } else {
        Ok(0.0)
    }
}

pub fn fomat_speed(bytes_pe_sec: f64) -> String {
    if bytes_pe_sec >= 1024.0 * 1024.0 {
        format!("{:.1} MB/s", bytes_pe_sec / 1024.0 / 1024.0)
    } else if bytes_pe_sec >= 1024.0 {
        format!("{:.1} KB/s", bytes_pe_sec / 1024.0)
    } else {
        format!("{:.0} B/s", bytes_pe_sec)
    }
}

pub fn fomat_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GB", bytes as f64 / 1024.0 / 1024.0 / 1024.0)
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / 1024.0 / 1024.0)
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
