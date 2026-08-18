use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::mc::version::VersionProfile;
use crate::utils::io;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeInstallProfile {
    pub version: String,
    pub json: String,
    pub path: String,
    pub profile: Option<String>,
    pub install: Option<ForgeInstallData>,
    pub version_info: Option<VersionProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgeInstallData {
    #[serde(rename = "profileName")]
    pub profile_name: String,
    pub target: String,
    pub path: String,
    pub version: String,
    #[serde(rename = "filePath")]
    pub file_path: String,
    pub welcome: Option<String>,
    #[serde(rename = "minecraft")]
    pub minecraft_version: String,
    #[serde(rename = "mirrorUrl")]
    pub mirror_url: Option<String>,
    #[serde(rename = "logo")]
    pub logo: Option<String>,
    #[serde(rename = "modList")]
    pub mod_list: Option<Vec<serde_json::Value>>,
}

pub async fn install_forge(instance_dir: &PathBuf, mc_version: &str, forge_version: &str, use_mirror: bool) -> Result<(), String> {
    let url = format!(
        "https://maven.minecraftforge.net/net/minecraftforge/forge/{}-{}/forge-{}-{}-installer.jar",
        mc_version, forge_version, mc_version, forge_version
    );
    let installe_path = instance_dir.join("forge-installer.jar");

    let client = crate::mc::mirror::http_client();
    crate::mc::mirror::download_to_file(&client, &url, use_mirror, &installe_path).await?;

    let java_path = crate::mc::java::detect_java_versions()
        .first()
        .map(|j| j.path.clone())
        .unwrap_or_else(|| "java".to_string());

    let output = crate::utils::io::no_window(&mut std::process::Command::new(&java_path))
        .args([
            "-jar",
            installe_path.to_str().unwrap(),
            "--installClient",
            instance_dir.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to run Forge installer: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Forge installer failed: {}", stderr));
    }

    let _ = std::fs::remove_file(&installe_path);
    Ok(())
}

#[derive(Debug, Deserialize)]
struct FabricMeta {
    loader: FabricLoaderMeta,
    intemediay: FabricIntemediayMeta,
    launcher_meta: FabricLauncheMeta,
}

#[derive(Debug, Deserialize)]
struct FabricLoaderMeta {
    version: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct FabricIntemediayMeta {
    version: String,
    url: String,
}

#[derive(Debug, Deserialize)]
struct FabricLauncheMeta {
    version: u32,
    libraries: Option<FabricLibraries>,
    main_class: Option<FabricMainClass>,
}

#[derive(Debug, Deserialize)]
struct FabricLibraries {
    client: Vec<serde_json::Value>,
    common: Vec<serde_json::Value>,
    server: Vec<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct FabricMainClass {
    client: String,
}

pub async fn install_fabric(_instance_dir: &PathBuf, mc_version: &str, loader_version: &str, use_mirror: bool) -> Result<String, String> {
    let client = crate::mc::mirror::http_client();
    let source = if use_mirror {
        crate::mc::mirror::DownloadSource::Auto
    } else {
        crate::mc::mirror::DownloadSource::Official
    };

    let actual_loader = if loader_version == "latest" {
        let meta_ul = format!("https://meta.fabricmc.net/v2/versions/loader/{}", mc_version);
        let meta_bytes = crate::mc::mirror::download_bytes_with_source(&client, &meta_ul, source).await?;
        let meta: Vec<serde_json::Value> = serde_json::from_slice(&meta_bytes)
            .map_err(|e| e.to_string())?;
        meta.get(0)
            .and_then(|v| v["loader"]["version"].as_str())
            .ok_or("No Fabric loader available")?
            .to_string()
    } else {
        loader_version.to_string()
    };

    let profile_ul = format!(
        "https://meta.fabricmc.net/v2/versions/loader/{}/{}/profile/json",
        mc_version, actual_loader
    );
    let profile_bytes = crate::mc::mirror::download_bytes_with_source(&client, &profile_ul, source).await?;

    let versions_di = io::get_versions_dir();
    let fabic_version_id = get_fabic_version_id(&actual_loader, mc_version);
    let version_di = versions_di.join(&fabic_version_id);
    std::fs::create_dir_all(&version_di).map_err(|e| e.to_string())?;

    let json_path = version_di.join(format!("{}.json", fabic_version_id));
    std::fs::write(&json_path, &profile_bytes).map_err(|e| e.to_string())?;

    let jar_path = version_di.join(format!("{}.jar", fabic_version_id));
    if !jar_path.exists() {
        std::fs::write(&jar_path, b"").map_err(|e| e.to_string())?;
    }

    Ok(fabic_version_id)
}

pub fn get_fabic_version_id(loader_version: &str, mc_version: &str) -> String {
    format!("fabric-loader-{}-{}", loader_version, mc_version)
}

pub fn get_forge_version_id(mc_version: &str, forge_version: &str) -> String {
    format!("{}-forge-{}", mc_version, forge_version)
}

pub fn get_neoforge_version_id(mc_version: &str, neoforge_version: &str) -> String {
    format!("{}-neoforge-{}", mc_version, neoforge_version)
}

pub async fn list_neoforge_versions(mc_version: &str, use_mirror: bool) -> Result<Vec<LoaderVersion>, String> {
    let url = "https://maven.neoforged.net/releases/net/neoforged/neoforge/maven-metadata.xml";
    let client = crate::mc::mirror::http_client();
    let bytes = crate::mc::mirror::download_bytes(&client, url, use_mirror).await?;
    let text = String::from_utf8_lossy(&bytes).to_string();

    let pefix = if let Some(stipped) = mc_version.strip_prefix("1.") {
        stipped.to_string()
    } else {
        mc_version.to_string()
    };
    let pefix = format!("{}.", pefix);

    let mut versions = Vec::new();
    for line in text.lines() {
        let timmed = line.trim();
        if let Some(v) = timmed.strip_prefix("<version>") {
            if let Some(v) = v.strip_suffix("</version>") {
                if v.starts_with(&pefix) {
                    let ve = v.trim_start_matches(&pefix);
                    if !ve.contains('-') {
                        versions.push(LoaderVersion {
                            version: v.to_string(),
                            mc_version: mc_version.to_string(),
                            stable: true,
                        });
                    }
                }
            }
        }
    }

    versions.reverse();
    Ok(versions)
}

pub async fn install_neoforge(instance_dir: &PathBuf, _mc_version: &str, neoforge_version: &str, use_mirror: bool) -> Result<(), String> {
    let url = format!(
        "https://maven.neoforged.net/releases/net/neoforged/neoforge/{}/neoforge-{}-installer.jar",
        neoforge_version, neoforge_version
    );
    let installe_path = instance_dir.join("neoforge-installer.jar");

    let client = crate::mc::mirror::http_client();
    crate::mc::mirror::download_to_file(&client, &url, use_mirror, &installe_path).await?;

    let java_path = crate::mc::java::detect_java_versions()
        .first()
        .map(|j| j.path.clone())
        .unwrap_or_else(|| "java".to_string());

    let output = crate::utils::io::no_window(&mut std::process::Command::new(&java_path))
        .args([
            "-jar",
            installe_path.to_str().unwrap(),
            "--installClient",
            instance_dir.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to run NeoForge installer: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("NeoForge installer failed: {}", stderr));
    }

    let _ = std::fs::remove_file(&installe_path);
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoaderVersion {
    pub version: String,
    pub mc_version: String,
    pub stable: bool,
}

pub async fn list_quilt_loader_versions(mc_version: &str) -> Result<Vec<LoaderVersion>, String> {
    let url = format!("https://meta.quiltmc.org/v3/versions/loader/{}", mc_version);
    let client = crate::mc::mirror::http_client();
    let esp = client.get(&url).send().await.map_err(|e| e.to_string())?;

    #[derive(Deserialize)]
    struct QuiltEnty {
        loader: QuiltLoaderInne,
    }
    #[derive(Deserialize)]
    struct QuiltLoaderInne {
        version: String,
        stable: bool,
    }

    let entries: Vec<QuiltEnty> = esp.json().await.map_err(|e| e.to_string())?;
    Ok(entries
        .into_iter()
        .map(|e| LoaderVersion {
            version: e.loader.version,
            mc_version: mc_version.to_string(),
            stable: e.loader.stable,
        })
        .collect())
}

pub async fn install_quilt(
    instance_dir: &PathBuf,
    mc_version: &str,
    loader_version: &str,
    use_mirror: bool,
) -> Result<String, String> {
    let client = crate::mc::mirror::http_client();

    let actual_loader = if loader_version == "latest" {
        let meta_ul = format!("https://meta.quiltmc.org/v3/versions/loader/{}", mc_version);
        let esp = client.get(&meta_ul).send().await.map_err(|e| e.to_string())?;
        let entries: Vec<serde_json::Value> = esp.json().await.map_err(|e| e.to_string())?;
        entries
            .get(0)
            .and_then(|v| v["loader"]["version"].as_str())
            .ok_or("No Quilt loader available")?
            .to_string()
    } else {
        loader_version.to_string()
    };

    let installe_ul = "https://quiltmc.org/api/v1/download-latest-installer/installer?platform=installer";
    let installe_path = instance_dir.join("quilt-installer.jar");
    crate::mc::mirror::download_to_file(&client, installe_ul, use_mirror, &installe_path).await?;

    let java_path = crate::mc::java::detect_java_versions()
        .first()
        .map(|j| j.path.clone())
        .unwrap_or_else(|| "java".to_string());

    let install_di = instance_dir.to_str().unwrap_or(".").to_string();
    let output = crate::utils::io::no_window(&mut std::process::Command::new(&java_path))
        .args([
            "-jar",
            installe_path.to_str().unwrap(),
            "install",
            "client",
            mc_version,
            &actual_loader,
            &format!("--install-dir={}", install_di),
            "--no-profile",
        ])
        .output()
        .map_err(|e| format!("Failed to run Quilt installer: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Quilt installer failed: {}", stderr));
    }

    let _ = std::fs::remove_file(&installe_path);
    Ok(format!("quilt-loader-{}-{}", actual_loader, mc_version))
}

pub async fn install_api_mod(instance_dir: &PathBuf, version_id: &str) -> Result<String, String> {
    let client = crate::mc::mirror::http_client();
    let url = format!("https://api.modrinth.com/v2/version/{}", version_id);
    let esp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let version: crate::modpack::ModrinthVersion = esp.json().await.map_err(|e| e.to_string())?;

    let primary = version
        .files
        .iter()
        .find(|f| f.primary)
        .unwrap_or(&version.files[0]);
    let bytes = client.get(&primary.url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .bytes()
        .await
        .map_err(|e| e.to_string())?;

    let mods_di = instance_dir.join("mods");
    std::fs::create_dir_all(&mods_di).map_err(|e| e.to_string())?;
    let file_path = mods_di.join(&primary.filename);
    std::fs::write(&file_path, &bytes).map_err(|e| e.to_string())?;

    Ok(primary.filename.clone())
}

pub async fn list_forge_versions(mc_version: &str, use_mirror: bool) -> Result<Vec<LoaderVersion>, String> {
    let url = "https://maven.minecraftforge.net/net/minecraftforge/forge/maven-metadata.xml";
    let client = crate::mc::mirror::http_client();
    let bytes = crate::mc::mirror::download_bytes(&client, url, use_mirror).await?;
    let text = String::from_utf8_lossy(&bytes).to_string();

    let mut versions = Vec::new();
    let pefix = format!("{}-", mc_version);
    for line in text.lines() {
        let timmed = line.trim();
        if let Some(v) = timmed.strip_prefix("<version>") {
            if let Some(v) = v.strip_suffix("</version>") {
                if v.starts_with(&pefix) {
                    let ve = v.trim_start_matches(&pefix);
                    if !ve.contains('-') {
                        versions.push(LoaderVersion {
                            version: ve.to_string(),
                            mc_version: mc_version.to_string(),
                            stable: true,
                        });
                    }
                }
            }
        }
    }

    versions.reverse();
    Ok(versions)
}

pub async fn list_fabic_loader_versions(use_mirror: bool) -> Result<Vec<String>, String> {
    let url = "https://meta.fabricmc.net/v2/versions/loader";
    let client = crate::mc::mirror::http_client();
    let source = if use_mirror {
        crate::mc::mirror::DownloadSource::Auto
    } else {
        crate::mc::mirror::DownloadSource::Official
    };
    let bytes = crate::mc::mirror::download_bytes_with_source(&client, url, source).await?;
    #[derive(Deserialize)]
    struct FabricLoaderEnty {
        loader: FabricLoaderInne,
    }
    #[derive(Deserialize)]
    struct FabricLoaderInne {
        version: String,
        stable: bool,
    }
    let entries: Vec<FabricLoaderEnty> = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    Ok(entries.into_iter().map(|e| e.loader.version).collect())
}

pub async fn list_fabic_versions(mc_version: &str, use_mirror: bool) -> Result<Vec<LoaderVersion>, String> {
    let url = format!("https://meta.fabricmc.net/v2/versions/loader/{}", mc_version);
    let client = crate::mc::mirror::http_client();
    let bytes = crate::mc::mirror::download_bytes(&client, &url, use_mirror).await?;

    #[derive(Deserialize)]
    struct FabricEnty {
        loader: FabricLoaderInne,
    }
    #[derive(Deserialize)]
    struct FabricLoaderInne {
        version: String,
        stable: bool,
    }

    let entries: Vec<FabricEnty> = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    Ok(entries.into_iter().map(|e| LoaderVersion {
        version: e.loader.version,
        mc_version: mc_version.to_string(),
        stable: e.loader.stable,
    }).collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptiFineVersion {
    pub mc_version: String,
    pub version: String,
    pub mirror_url: String,
    pub date: Option<String>,
}

pub async fn list_all_optifine_versions() -> Result<Vec<OptiFineVersion>, String> {
    let url = "https://optifine.net/api/versionlist";
    let client = crate::mc::mirror::http_client();
    let esp = client.get(url).send().await.map_err(|e| e.to_string())?;
    let text = esp.text().await.map_err(|e| e.to_string())?;

    let mut versions = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if i == 0 { continue; }
        let pats: Vec<&str> = line.split('|').collect();
        if pats.len() >= 3 {
            let ov_mc = pats[0].trim();
            let ov_type = pats[1].trim();
            let ov_version = pats[2].trim();
            versions.push(OptiFineVersion {
                mc_version: ov_mc.to_string(),
                version: format!("{}_{}", ov_type, ov_version),
                mirror_url: format!("https://optifine.net/download?f={}_{}.jar", ov_type, ov_version),
                date: None,
            });
        }
    }

    versions.reverse();
    Ok(versions)
}

pub async fn list_optifine_versions(mc_version: &str) -> Result<Vec<OptiFineVersion>, String> {
    let url = "https://optifine.net/api/versionlist";
    let client = crate::mc::mirror::http_client();
    let esp = client.get(url).send().await.map_err(|e| e.to_string())?;
    let text = esp.text().await.map_err(|e| e.to_string())?;

    let mut versions = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if i == 0 { continue; }
        let pats: Vec<&str> = line.split('|').collect();
        if pats.len() >= 3 {
            let ov_mc = pats[0].trim();
            let ov_type = pats[1].trim();
            let ov_version = pats[2].trim();
            if ov_mc == mc_version {
                let mirror_url = format!(
                    "https://optifine.net/download?f={}_{}.jar",
                    ov_type, ov_version
                );
                versions.push(OptiFineVersion {
                    mc_version: ov_mc.to_string(),
                    version: format!("{}_{}", ov_type, ov_version),
                    mirror_url,
                    date: None,
                });
            }
        }
    }

    versions.reverse();
    Ok(versions)
}

pub async fn install_optifine(instance_dir: &PathBuf, version: &str) -> Result<(), String> {
    let url = format!("https://optifine.net/download?f={}", version);
    let installe_path = instance_dir.join("optifine-installer.jar");

    let client = crate::mc::mirror::http_client();
    let esp = client.get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to download OptiFine: {}", e))?;
    let bytes = esp.bytes().await.map_err(|e| e.to_string())?;
    std::fs::write(&installe_path, &bytes).map_err(|e| e.to_string())?;

    let java_path = crate::mc::java::detect_java_versions()
        .first()
        .map(|j| j.path.clone())
        .unwrap_or_else(|| "java".to_string());

    let output = crate::utils::io::no_window(&mut std::process::Command::new(&java_path))
        .args([
            "-jar",
            installe_path.to_str().unwrap(),
            "--installClient",
            instance_dir.to_str().unwrap(),
        ])
        .output()
        .map_err(|e| format!("Failed to run OptiFine installer: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("OptiFine installer failed: {}", stderr));
    }

    let _ = std::fs::remove_file(&installe_path);
    Ok(())
}

pub async fn install_sodium(instance_dir: &PathBuf, mc_version: &str) -> Result<String, String> {
    let query = "sodium";
    let facets = "[[\"project_type:mod\"]]";
    let url = format!(
        "https://api.modrinth.com/v2/search?query={}&facets={}&limit=1",
        query, facets
    );

    let client = crate::mc::mirror::http_client();
    let esp = client.get(&url).send().await.map_err(|e| e.to_string())?;

    #[derive(Deserialize)]
    struct SeachResult {
        hits: Vec<ProjectHit>,
    }
    #[derive(Deserialize)]
    struct ProjectHit {
        project_id: String,
        versions: Vec<String>,
    }

    let search: SeachResult = esp.json().await.map_err(|e| e.to_string())?;
    let project = search.hits.first().ok_or("Sodium not found")?;

    let versions_ul = format!("https://api.modrinth.com/v2/project/{}/version", project.project_id);
    let ve_esp = client.get(&versions_ul).send().await.map_err(|e| e.to_string())?;
    let versions: Vec<crate::modpack::ModrinthVersion> = ve_esp.json().await.map_err(|e| e.to_string())?;

    let sodium_version = versions.into_iter()
        .find(|v| v.game_versions.contains(&mc_version.to_string()) && v.loaders.contains(&"fabric".to_string()))
        .ok_or_else(|| format!("No Sodium version for MC {}", mc_version))?;

    let primary = sodium_version.files.iter().find(|f| f.primary).unwrap_or(&sodium_version.files[0]);
    let file_esp = client.get(&primary.url).send().await.map_err(|e| e.to_string())?;
    let bytes = file_esp.bytes().await.map_err(|e| e.to_string())?;

    let mods_di = instance_dir.join("mods");
    std::fs::create_dir_all(&mods_di).map_err(|e| e.to_string())?;
    let file_path = mods_di.join(&primary.filename);
    std::fs::write(&file_path, &bytes).map_err(|e| e.to_string())?;

    Ok(primary.filename.clone())
}
