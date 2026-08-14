pub const BMCLAPI_BASE: &str = "https://bmclapi2.bangbang93.com";

pub const SKYLINE_USER_AGENT: &str = concat!("SkyLineLauncher/", env!("CARGO_PKG_VERSION"));

pub fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent(SKYLINE_USER_AGENT)
        .connect_timeout(Duration::from_secs(15))
        .timeout(Duration::from_secs(180))
        .redirect(reqwest::redirect::Policy::limited(10))
        .build()
        .expect("failed to build HTTP client")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum DownloadSource {
    Auto,
    Official,
    Mirror,
}

impl DownloadSource {
    pub fn from_str(s: &str) -> Self {
        match s {
            "official" => DownloadSource::Official,
            "mirror" => DownloadSource::Mirror,
            _ => DownloadSource::Auto,
        }
    }

    pub fn to_str(&self) -> &str {
        match self {
            DownloadSource::Auto => "auto",
            DownloadSource::Official => "official",
            DownloadSource::Mirror => "mirror",
        }
    }
}

#[derive(Debug, Clone)]
pub struct SourceSpeedResult {
    pub source: DownloadSource,
    pub speed: f64,  
    pub latency: Duration,
    pub success: bool,
}

use std::time::Duration;

pub fn mirror_url(url: &str) -> String {
    let url = url.replace("https://piston-meta.mojang.com", BMCLAPI_BASE);
    let url = url.replace("https://piston-data.mojang.com", BMCLAPI_BASE);
    let url = url.replace("https://launcher.mojang.com", BMCLAPI_BASE);
    let url = url.replace("https://libraries.minecraft.net", &format!("{}/libraries", BMCLAPI_BASE));
    let url = url.replace("https://resources.download.minecraft.net", &format!("{}/assets", BMCLAPI_BASE));
    let url = url.replace("https://maven.minecraftforge.net", &format!("{}/maven", BMCLAPI_BASE));
    let url = url.replace("https://maven.fabricmc.net", &format!("{}/maven", BMCLAPI_BASE));
    let url = url.replace("https://maven.neoforged.net", &format!("{}/maven", BMCLAPI_BASE));
    let url = url.replace("https://meta.fabricmc.net", &format!("{}/fabric-meta", BMCLAPI_BASE));
    
    
    url.replace("https://sessionserver.mojang.com", BMCLAPI_BASE)
}

fn is_mirror_url(url: &str) -> bool {
    url.contains("bmclapi2.bangbang93.com")
}

pub fn inject_url_with_candidates(url: &str, source: DownloadSource) -> Vec<String> {
    let mirrored = mirror_url(url);

    match source {
        DownloadSource::Auto => {
            if is_mirror_url(url) {
                vec![url.to_string()]
            } else {
                vec![url.to_string(), mirrored]
            }
        }
        DownloadSource::Official => {
            vec![url.to_string()]
        }
        DownloadSource::Mirror => {
            if is_mirror_url(url) {
                vec![url.to_string()]
            } else {
                vec![mirrored]
            }
        }
    }
}

pub async fn test_source_speed(client: &reqwest::Client, url: &str) -> SourceSpeedResult {
    let stat = std::time::Instant::now();

    match client.get(url).send().await {
        Ok(esp) => {
            let latency = stat.elapsed();
            if esp.status().is_success() {
                match esp.bytes().await {
                    Ok(bytes) => {
                        let elapsed = stat.elapsed().as_secs_f64();
                        let speed = if elapsed > 0.0 {
                            bytes.len() as f64 / elapsed
                        } else {
                            0.0
                        };
                        SourceSpeedResult {
                            source: DownloadSource::Auto,
                            speed,
                            latency,
                            success: true,
                        }
                    }
                    Err(_) => SourceSpeedResult {
                        source: DownloadSource::Auto,
                        speed: 0.0,
                        latency,
                        success: false,
                    },
                }
            } else {
                SourceSpeedResult {
                    source: DownloadSource::Auto,
                    speed: 0.0,
                    latency,
                    success: false,
                }
            }
        }
        Err(_) => SourceSpeedResult {
            source: DownloadSource::Auto,
            speed: 0.0,
            latency: Duration::from_secs(999),
            success: false,
        },
    }
}

pub async fn select_fastest_source(client: &reqwest::Client, test_url: &str) -> DownloadSource {
    let official_ul = test_url;
    let mirror_url_st = mirror_url(test_url);

    let (official_result, mirror_result) = tokio::join!(
        test_source_speed(client, official_ul),
        test_source_speed(client, &mirror_url_st)
    );

    match (official_result.success, mirror_result.success) {
        (true, true) => {
            if official_result.speed >= mirror_result.speed {
                DownloadSource::Official
            } else {
                DownloadSource::Mirror
            }
        }
        (true, false) => DownloadSource::Official,
        (false, true) => DownloadSource::Mirror,
        (false, false) => DownloadSource::Auto,
    }
}

pub async fn download_bytes(
    client: &reqwest::Client,
    url: &str,
    use_mirror: bool,
) -> Result<Vec<u8>, String> {
    let source = if use_mirror {
        DownloadSource::Auto
    } else {
        DownloadSource::Official
    };
    download_bytes_with_source(client, url, source).await
}

pub async fn download_bytes_with_source(
    client: &reqwest::Client,
    url: &str,
    source: DownloadSource,
) -> Result<Vec<u8>, String> {
    let candidates = inject_url_with_candidates(url, source);
    let mut last_er: Option<String> = None;

    for cand in &candidates {
        match client.get(cand).send().await {
            Ok(esp) if esp.status().is_success() => {
                match esp.bytes().await {
                    Ok(bytes) => {
                        log::debug!("Download success from: {}", cand);
                        return Ok(bytes.to_vec());
                    }
                    Err(e) => {
                        log::warn!("Download body error from {}: {}", cand, e);
                        last_er = Some(format!("{}: {}", e, cand));
                    }
                }
            }
            Ok(esp) => {
                log::warn!("Download HTTP {} from {}", esp.status(), cand);
                last_er = Some(format!("HTTP {} for {}", esp.status(), cand));
            }
            Err(e) => {
                log::warn!("Download request error from {}: {}", cand, e);
                last_er = Some(format!("{}: {}", e, cand));
            }
        }
    }
    Err(last_er.unwrap_or_else(|| format!("Download failed: {}", url)))
}

pub async fn download_to_file(
    client: &reqwest::Client,
    url: &str,
    use_mirror: bool,
    path: &std::path::Path,
) -> Result<(), String> {
    let bytes = download_bytes(client, url, use_mirror).await?;
    if let Some(prent) = path.parent() {
        std::fs::create_dir_all(prent).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, &bytes).map_err(|e| e.to_string())
}

pub async fn fetch_json<T: serde::de::DeserializeOwned>(
    client: &reqwest::Client,
    url: &str,
    use_mirror: bool,
) -> Result<T, String> {
    let bytes = download_bytes(client, url, use_mirror).await?;
    serde_json::from_slice(&bytes).map_err(|e| format!("Failed to parse JSON from {}: {}", url, e))
}
