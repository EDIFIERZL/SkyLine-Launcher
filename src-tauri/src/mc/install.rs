use crate::mc::version::{self, VersionManifest};
use crate::mc::library;
use crate::mc::asset;
use crate::utils::io;
use serde::Serialize;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone, Serialize)]
pub struct InstallProgress {
    pub stage: String,
    pub progress: f64,
    pub message: String,
}

pub async fn fetch_versions(use_mirror: bool) -> Result<VersionManifest, String> {
    version::fetch_version_manifest(use_mirror).await.map_err(|e| e.to_string())
}

pub async fn install_minecraft(
    version_id: &str,
    use_mirror: bool,
    concurrency: usize,
    on_progress: impl Fn(InstallProgress) + Send + Sync + Clone + 'static,
) -> Result<PathBuf, String> {
    let on_progress = Arc::new(on_progress);
    let manifest = fetch_versions(use_mirror).await?;
    let entry = manifest.versions.iter()
        .find(|v| v.id == version_id)
        .ok_or_else(|| format!("Version {} not found", version_id))?;

    (on_progress)(InstallProgress {
        stage: "fetching_profile".into(),
        progress: 0.05,
        message: "Fetching version profile...".into(),
    });

    let profile = version::fetch_version_profile(&entry.url, use_mirror).await?;

    let versions_dir = io::get_versions_dir();
    let version_dir = versions_dir.join(&profile.id);
    std::fs::create_dir_all(&version_dir).map_err(|e| e.to_string())?;

    let profile_path = version_dir.join(format!("{}.json", profile.id));
    let profile_json = serde_json::to_string_pretty(&profile).map_err(|e| e.to_string())?;
    std::fs::write(&profile_path, &profile_json).map_err(|e| e.to_string())?;

    (on_progress)(InstallProgress {
        stage: "downloading_client".into(),
        progress: 0.15,
        message: "Downloading client jar...".into(),
    });

    if let Some(ref downloads) = profile.downloads {
        if let Some(ref client) = downloads.client {
            let jar_path = version_dir.join(format!("{}.jar", profile.id));
            if !jar_path.exists() {
                crate::mc::mirror::download_to_file(
                    &crate::mc::mirror::http_client(),
                    &client.url,
                    use_mirror,
                    &jar_path,
                ).await?;
            }
        }
    }

    (on_progress)(InstallProgress {
        stage: "downloading_asset_index".into(),
        progress: 0.30,
        message: "Downloading asset index...".into(),
    });

    asset::download_asset_index(&profile.asset_index, use_mirror).await?;

    (on_progress)(InstallProgress {
        stage: "downloading_libraries".into(),
        progress: 0.40,
        message: "Downloading libraries...".into(),
    });

    let total_libs = profile.libraries.len() as f64;
    let semaphore = Arc::new(tokio::sync::Semaphore::new(concurrency.max(1)));
    let done_libs = Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut handles = Vec::new();
    for lib in &profile.libraries {
        let sem = semaphore.clone();
        let done = done_libs.clone();
        let on_progress = on_progress.clone();
        let name = lib.name.clone();
        let lib = lib.clone();
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            if let Err(e) = library::download_library(&lib, use_mirror).await {
                log::warn!("Failed to download library {}: {}", name, e);
            }
            let d = done.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1;
            let p = 0.40 + (d as f64 / total_libs) * 0.50;
            (on_progress)(InstallProgress {
                stage: "downloading_libraries".into(),
                progress: p,
                message: format!("Downloading libraries... ({}/{})", d, total_libs),
            });
        }));
    }
    for handle in handles {
        let _ = handle.await;
    }

    (on_progress)(InstallProgress {
        stage: "downloading_assets".into(),
        progress: 0.90,
        message: "Verifying assets...".into(),
    });

    let on_progress_clone = on_progress.clone();
    asset::download_assets(
        &profile.assets,
        use_mirror,
        concurrency,
        move |done, total| {
            (on_progress_clone)(InstallProgress {
                stage: "downloading_assets".into(),
                progress: 0.90 + (done as f64 / total as f64) * 0.10,
                message: format!("Verifying assets... ({}/{})", done, total),
            });
        },
    ).await?;

    (on_progress)(InstallProgress {
        stage: "complete".into(),
        progress: 1.0,
        message: "Installation complete!".into(),
    });

    Ok(version_dir)
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_install_flow_profile() {
        // Test just profile fetch + manifest fetch (not full download)
        let manifest = crate::mc::version::fetch_version_manifest(false).await;
        match manifest {
            Ok(m) => {
                println!("manifest OK latest={} count={}", m.latest.elease, m.versions.len());
                let entry = m.versions.iter().find(|v| v.id == "1.16.5").or_else(|| m.versions.first());
                if let Some(e) = entry {
                    println!("fetching profile for {}", e.id);
                    let prof = crate::mc::version::fetch_version_profile(&e.url, false).await;
                    match prof {
                        Ok(p) => println!("profile OK id={} libs={} mainClass={}", p.id, p.libraries.len(), p.main_class),
                        Err(e2) => println!("profile ERR: {}", e2),
                    }
                }
            }
            Err(e) => println!("manifest ERR: {}", e),
        }
    }
}
