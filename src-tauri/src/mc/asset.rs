use crate::mc::version::AssetIndexRef;
use crate::mc::mirror::DownloadSource;
use crate::utils::io;
use crate::utils::crypto;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub async fn download_asset_index(asset_ef: &AssetIndexRef, use_mirror: bool) -> Result<(), String> {
    let assets_di = io::get_assets_dir();
    let indexes_di = assets_di.join("indexes");
    std::fs::create_dir_all(&indexes_di).map_err(|e| e.to_string())?;

    let index_path = indexes_di.join(format!("{}.json", asset_ef.id));
    if index_path.exists() {
        let content = std::fs::read_to_string(&index_path).map_err(|e| e.to_string())?;
        let actual_sha1 = crypto::sha1_hex(content.as_bytes()).map_err(|e| e.to_string())?;
        if actual_sha1 == asset_ef.sha1 {
            return Ok(());
        }
    }

    let client = crate::mc::mirror::http_client();
    let bytes = crate::mc::mirror::download_bytes(&client, &asset_ef.url, use_mirror).await?;

    std::fs::write(&index_path, &bytes).map_err(|e| e.to_string())?;
    Ok(())
}

pub async fn download_assets(asset_index_id: &str, use_mirror: bool, concurrency: usize, on_progress: impl Fn(u64, u64) + Send + Sync + 'static) -> Result<(), String> {
    let assets_dir = io::get_assets_dir();
    let index_path = assets_dir.join("indexes").join(format!("{asset_index_id}.json"));

    if !index_path.exists() {
        return Err("Asset index not found".to_string());
    }

    let content = std::fs::read_to_string(&index_path).map_err(|e| e.to_string())?;
    let index: serde_json::Value = serde_json::from_str(&content).map_err(|e| e.to_string())?;

    let objects = index["objects"].as_object().ok_or("Invalid asset index")?;
    let total = objects.len() as u64;

    let semaphore = Arc::new(Semaphore::new(concurrency.max(1)));
    let client = Arc::new(crate::mc::mirror::http_client());
    let done = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let source = if use_mirror {
        DownloadSource::Auto
    } else {
        DownloadSource::Official
    };

    let on_progress = Arc::new(on_progress);

    let mut handles = Vec::new();
    for (_, obj) in objects {
        let hash = obj["hash"].as_str().ok_or("Missing hash")?.to_string();
        let object_dir = assets_dir.join("objects").join(&hash[..2]);
        let object_path = object_dir.join(&hash);

        if object_path.exists() {
            done.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            continue;
        }

        let sem = semaphore.clone();
        let cl = client.clone();
        let d = done.clone();
        let p = on_progress.clone();
        let url = format!("https://resources.download.minecraft.net/{}/{}", &hash[..2], &hash);
        let dl_source = source;

        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            if let Err(e) = download_single_asset(&cl, &url, dl_source, &object_path, &hash).await {
                log::warn!("Failed to download asset {}: {}", hash, e);
            }
            let prev = d.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            (p)(prev + 1, total);
        }));
    }

    for handle in handles {
        let _ = handle.await;
    }
    Ok(())
}

async fn download_single_asset(client: &reqwest::Client, url: &str, source: DownloadSource, path: &Path, expected_hash: &str) -> Result<(), String> {
    let candidates = crate::mc::mirror::inject_url_with_candidates(url, source);

    for cand in &candidates {
        match client.get(cand).send().await {
            Ok(resp) if resp.status().is_success() => {
                if let Ok(bytes) = resp.bytes().await {
                    let actual_hash = crypto::sha1_hex(std::io::Cursor::new(&bytes)).map_err(|e| e.to_string())?;
                    if actual_hash == expected_hash {
                        log::debug!("Asset download success from: {}", cand);
                        return writer_asset(path, &bytes);
                    } else {
                        log::warn!("Asset hash mismatch from {}, expected {} got {}", cand, expected_hash, actual_hash);
                    }
                }
            }
            Ok(resp) => {
                log::warn!("Asset download HTTP {} from {}", resp.status(), cand);
            }
            Err(e) => {
                log::warn!("Asset download error from {}: {}", cand, e);
            }
        }
    }

    Err(format!("Failed to download asset {} from all candidates", expected_hash))
}

fn writer_asset(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, bytes).map_err(|e| e.to_string())
}
