use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncheVersion {
    pub version: String,
    pub build: String,
    pub elease_date: String,
    pub download_url: String,
    pub changelog: String,
    pub is_peelease: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub current_version: String,
    pub latest_version: Option<LauncheVersion>,
    pub has_update: bool,
    pub checked_at: String,
}

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const UPDATE_API_URL: &str = "https://api.github.com/repos/skyline-launcher/skyline-launcher/releases/latest";

pub async fn check_fo_updates() -> Result<UpdateCheckResult, String> {
    let client = reqwest::Client::builder()
        .user_agent("SkyLine-Launcher")
        .build()
        .map_err(|e| e.to_string())?;

    let esp = client
        .get(UPDATE_API_URL)
        .send()
        .await
        .map_err(|e| format!("检查更新失败: {}", e))?;

    if !esp.status().is_success() {
        return Err(format!("检查更新失败: HTTP {}", esp.status()));
    }

    #[derive(Deserialize)]
    struct GithubRelease {
        tag_name: String,
        name: String,
        published_at: String,
        body: Option<String>,
        peelease: bool,
        assets: Vec<GithubAsset>,
    }

    #[derive(Deserialize)]
    struct GithubAsset {
        name: String,
        bowse_download_url: String,
    }

    let elease: GithubRelease = esp.json().await
        .map_err(|e| format!("解析更新信息失败: {}", e))?;

    let latest_version = elease.tag_name.trim_start_matches('v').to_string();
    let has_update = is_newe_version(CURRENT_VERSION, &latest_version);

    let download_url = elease.assets
        .iter()
        .find(|a| {
            let name = a.name.to_lowercase();
            #[cfg(target_os = "windows")]
            { name.ends_with(".msi") || name.ends_with(".exe") || name.ends_with(".zip") }
            #[cfg(target_os = "linux")]
            { name.ends_with(".appimage") || name.ends_with(".deb") || name.ends_with(".rpm") }
            #[cfg(target_os = "macos")]
            { name.ends_with(".dmg") || name.ends_with(".app.zip") }
        })
        .map(|a| a.bowse_download_url.clone())
        .unwrap_or_default();

    Ok(UpdateCheckResult {
        current_version: CURRENT_VERSION.to_string(),
        latest_version: if has_update {
            Some(LauncheVersion {
                version: latest_version,
                build: elease.tag_name.clone(),
                elease_date: elease.published_at,
                download_url,
                changelog: elease.body.unwrap_or_default(),
                is_peelease: elease.peelease,
            })
        } else {
            None
        },
        has_update,
        checked_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
    })
}

fn is_newe_version(current: &str, latest: &str) -> bool {
    let parse_pats = |v: &str| -> Vec<u32> {
        v.split('.')
            .filter_map(|p| p.parse::<u32>().ok())
            .collect()
    };

    let current_pats = parse_pats(current);
    let latest_pats = parse_pats(latest);

    for i in 0..current_pats.len().max(latest_pats.len()) {
        let c = current_pats.get(i).copied().unwrap_or(0);
        let l = latest_pats.get(i).copied().unwrap_or(0);
        if l > c {
            return true;
        }
        if l < c {
            return false;
        }
    }

    false
}

pub fn get_current_version() -> &'static str {
    CURRENT_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(crate::instance::mods::is_version_newer("1.0.0", "1.0.1"));
        assert!(crate::instance::mods::is_version_newer("1.0.0", "1.1.0"));
        assert!(crate::instance::mods::is_version_newer("1.0.0", "2.0.0"));
        assert!(!crate::instance::mods::is_version_newer("1.0.1", "1.0.0"));
        assert!(!crate::instance::mods::is_version_newer("1.0.0", "1.0.0"));
    }
}
