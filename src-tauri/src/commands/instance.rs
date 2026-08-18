use crate::instance::{Instance, ModLoader};
use crate::instance::manager;
use crate::mc::install;
use crate::mc::modloader;
use crate::utils::io;
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};
use tauri::Emitter;

fn install_in_progress() -> &'static Mutex<HashSet<String>> {
    static S: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    S.get_or_init(|| Mutex::new(HashSet::new()))
}

struct InstallGuard(String);

impl InstallGuard {
    fn try_new(key: String) -> Result<Self, String> {
        let mut set = install_in_progress()
            .lock()
            .map_err(|_| "内部错误: 安装状态锁失效".to_string())?;
        if set.contains(&key) {
            return Err("该实例正在下载中，请等待完成后再试".to_string());
        }
        set.insert(key.clone());
        Ok(InstallGuard(key))
    }
}

impl Drop for InstallGuard {
    fn drop(&mut self) {
        if let Ok(mut set) = install_in_progress().lock() {
            set.remove(&self.0);
        }
    }
}

fn install_dedupe_key(mc_version: &str, loaderrs: &[LoaderrSelection]) -> String {
    let mut parts: Vec<String> = Vec::new();
    for l in loaderrs {
        if l.loaderr == "vanilla" {
            continue;
        }
        parts.push(format!("{}-{}", l.loaderr, l.version.clone().unwrap_or_else(|| "latest".into())));
    }
    if parts.is_empty() {
        mc_version.to_string()
    } else {
        format!("{}|{}", mc_version, parts.join("+"))
    }
}

#[tauri::command]
pub async fn list_instances() -> Result<Vec<Instance>, String> {
    manager::list_instances()
}

#[tauri::command]
pub async fn get_instance(id: String) -> Result<Option<Instance>, String> {
    manager::get_instance(&id)
}

#[tauri::command]
#[allow(deprecated)]
pub async fn open_instance_folder(
    instance_id: String,
    subdir: Option<String>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    
    use tauri_plugin_shell::ShellExt;

    let instance = manager::get_instance(&instance_id)?
        .ok_or_else(|| "Instance not found".to_string())?;
    let base = if let Some(sd) = &subdir {
        manager::get_instance_mc_dir(&instance_id)?.join(sd)
    } else {
        manager::get_instance_launch_dir(&instance)
    };
    std::fs::create_dir_all(&base).map_err(|e| e.to_string())?;
    app_handle
        .shell()
        .open(base.to_string_lossy(), None)
        .map_err(|e| e.to_string())?;
    Ok(())
}

pub(crate) fn load_use_mirror() -> bool {
    crate::commands::settings::load_config()
        .unwrap_or_else(|_| crate::commands::settings::LauncherConfig::default())
        .download_source
        .as_str()
        != "official"
}

pub(crate) fn load_download_threads() -> usize {
    let n = crate::commands::settings::load_config()
        .unwrap_or_else(|_| crate::commands::settings::LauncherConfig::default())
        .download_threads;
    if n == 0 {
        8
    } else {
        n as usize
    }
}

#[tauri::command]
pub async fn install_game(
    name: String,
    mc_version: String,
    loaderr: String,
    loaderr_version: Option<String>,
    app_handle: tauri::AppHandle,
) -> Result<Instance, String> {
    let use_mirror = load_use_mirror();

    let loaderrs = if loaderr == "vanilla" {
        vec![]
    } else {
        vec![LoaderrSelection {
            loaderr: loaderr.clone(),
            version: loaderr_version.clone(),
        }]
    };
    let _guard = InstallGuard::try_new(install_dedupe_key(&mc_version, &loaderrs))?;

    let threads = load_download_threads();
    install::install_minecraft(&mc_version, use_mirror, threads, move |progress| {
        let _ = app_handle.emit("install-progress", &progress);
    })
    .await
    .map_err(|e| e.to_string())?;

    let ml = match loaderr.as_str() {
        "forge" => ModLoader::Forge(loaderr_version.clone().unwrap_or_default()),
        "fabric" | "fabric-api" => ModLoader::Fabric(loaderr_version.clone().unwrap_or_default()),
        "quilt" | "qsl" => ModLoader::Quilt(loaderr_version.clone().unwrap_or_default()),
        "neoforge" => ModLoader::NeoForge(loaderr_version.clone().unwrap_or_default()),
        _ => ModLoader::Vanilla,
    };

    let shared = io::get_shared_dir();
    let version_id: String = match loaderr.as_str() {
        "forge" => {
            let v = loaderr_version.as_deref().ok_or("缺少 Forge 版本")?;
            modloader::install_forge(&shared, &mc_version, v, use_mirror).await?;
            modloader::get_forge_version_id(&mc_version, v)
        }
        "neoforge" => {
            let v = loaderr_version.as_deref().ok_or("缺少 NeoForge 版本")?;
            modloader::install_neoforge(&shared, &mc_version, v, use_mirror).await?;
            modloader::get_neoforge_version_id(&mc_version, v)
        }
        "fabric" => {
            let v = loaderr_version.unwrap_or_else(|| "latest".to_string());
            modloader::install_fabric(&shared, &mc_version, &v, use_mirror).await?
        }
        "quilt" => {
            let v = loaderr_version.unwrap_or_else(|| "latest".to_string());
            modloader::install_quilt(&shared, &mc_version, &v, use_mirror).await?
        }
        "optifine" => {
            let v = loaderr_version.as_deref().ok_or("缺少 OptiFine 版本")?;
            modloader::install_optifine(&shared, v).await?;
            format!("{}-OptiFine_{}", mc_version, v)
        }
        "fabric-api" => {
            let v = loaderr_version.as_deref().ok_or("缺少 Fabric API 版本")?;
            let fabric_id = modloader::install_fabric(&shared, &mc_version, "latest", use_mirror).await?;
            modloader::install_api_mod(&io::get_instance_dir(&fabric_id), v).await?;
            fabric_id
        }
        "qsl" => {
            let v = loaderr_version.as_deref().ok_or("缺少 QSL 版本")?;
            let quilt_id = modloader::install_quilt(&shared, &mc_version, "latest", use_mirror).await?;
            modloader::install_api_mod(&io::get_instance_dir(&quilt_id), v).await?;
            quilt_id
        }
        _ => mc_version.clone(),
    };

    let instance = Instance {
        id: version_id.clone(),
        name,
        version_id,
        modloader: ml,
        ..Default::default()
    };
    manager::create_instance(&instance)?;

    Ok(instance)
}

#[derive(serde::Deserialize)]
pub struct LoaderrSelection {
    pub loaderr: String,
    #[serde(default)]
    pub version: Option<String>,
}

#[tauri::command]
pub async fn install_game_multi(
    name: String,
    mc_version: String,
    loaderrs: Vec<LoaderrSelection>,
    app_handle: tauri::AppHandle,
) -> Result<Instance, String> {
    let use_mirror = load_use_mirror();

    let _guard = InstallGuard::try_new(install_dedupe_key(&mc_version, &loaderrs))?;

    let app_handle2 = app_handle.clone();
    let threads = load_download_threads();
    install::install_minecraft(&mc_version, use_mirror, threads, move |progress| {
        let _ = app_handle2.emit("install-progress", &progress);
    })
    .await
    .map_err(|e| e.to_string())?;

    let mut ordered = loaderrs;
    ordered.sort_by_key(|l| match l.loaderr.as_str() {
        "optifine" => 2,
        "fabric-api" | "qsl" => 1,
        _ => 0,
    });

    let first = ordered
        .iter()
        .find(|l| l.loaderr != "vanilla")
        .and_then(|l| match l.loaderr.as_str() {
            "forge" => Some(ModLoader::Forge(l.version.clone().unwrap_or_default())),
            "fabric" | "fabric-api" => Some(ModLoader::Fabric(l.version.clone().unwrap_or_default())),
            "quilt" | "qsl" => Some(ModLoader::Quilt(l.version.clone().unwrap_or_default())),
            "neoforge" => Some(ModLoader::NeoForge(l.version.clone().unwrap_or_default())),
            _ => None,
        })
        .unwrap_or(ModLoader::Vanilla);

    let name_clone = name.clone();
    let first_clone = first.clone();
    let shared = io::get_shared_dir();
    let app_handle3 = app_handle.clone();
    let version_id: String = async move {
        let mut final_id: String = mc_version.clone();
        for sel in &ordered {
            let stage_msg = |loaderr_name: &str, ver: &str| {
                let _ = app_handle3.emit(
                    "install-progress",
                    &install::InstallProgress {
                        stage: "loaderr".into(),
                        progress: 0.92,
                        message: format!("正在安装 {} {}", loaderr_name, ver),
                    },
                );
            };
            match sel.loaderr.as_str() {
                "forge" => {
                    let v = sel.version.as_deref().ok_or_else(|| "缺少 Forge 版本".to_string())?;
                    stage_msg("Forge", v);
                    modloader::install_forge(&shared, &mc_version, v, use_mirror).await?;
                    final_id = modloader::get_forge_version_id(&mc_version, v);
                }
                "neoforge" => {
                    let v = sel.version.as_deref().ok_or_else(|| "缺少 NeoForge 版本".to_string())?;
                    stage_msg("NeoForge", v);
                    modloader::install_neoforge(&shared, &mc_version, v, use_mirror).await?;
                    final_id = modloader::get_neoforge_version_id(&mc_version, v);
                }
                "fabric" => {
                    let v = sel.version.clone().unwrap_or_else(|| "latest".to_string());
                    stage_msg("Fabric", &v);
                    final_id = modloader::install_fabric(&shared, &mc_version, &v, use_mirror).await?;
                }
                "quilt" => {
                    let v = sel.version.clone().unwrap_or_else(|| "latest".to_string());
                    stage_msg("Quilt", &v);
                    final_id = modloader::install_quilt(&shared, &mc_version, &v, use_mirror).await?;
                }
                "optifine" => {
                    let v = sel.version.as_deref().ok_or_else(|| "缺少 OptiFine 版本".to_string())?;
                    stage_msg("OptiFine", v);
                    modloader::install_optifine(&shared, v).await?;
                    final_id = format!("{}-OptiFine_{}", mc_version, v);
                }
                "fabric-api" => {
                    let v = sel.version.as_deref().ok_or_else(|| "缺少 Fabric API 版本".to_string())?;
                    stage_msg("Fabric API", v);
                    let fabric_id = modloader::install_fabric(&shared, &mc_version, "latest", use_mirror).await?;
                    final_id = fabric_id;
                    modloader::install_api_mod(&io::get_instance_dir(&final_id), v).await?;
                }
                "qsl" => {
                    let v = sel.version.as_deref().ok_or_else(|| "缺少 QSL 版本".to_string())?;
                    stage_msg("QSL", v);
                    let quilt_id = modloader::install_quilt(&shared, &mc_version, "latest", use_mirror).await?;
                    final_id = quilt_id;
                    modloader::install_api_mod(&io::get_instance_dir(&final_id), v).await?;
                }
                _ => {}
            }
        }
        Ok::<String, String>(final_id)
    }
    .await?;

    let instance = Instance {
        id: version_id.clone(),
        name: name_clone,
        version_id,
        modloader: first_clone,
        ..Default::default()
    };
    manager::create_instance(&instance)?;

    Ok(instance)
}

#[tauri::command]
pub async fn create_instance(
    name: String,
    version_id: String,
    modloader: String,
    modloader_version: Option<String>,
) -> Result<Instance, String> {
    let ml = match modloader.as_str() {
        "forge" => ModLoader::Forge(modloader_version.unwrap_or_default()),
        "fabric" => ModLoader::Fabric(modloader_version.unwrap_or_default()),
        "quilt" => ModLoader::Quilt(modloader_version.unwrap_or_default()),
        "neoforge" => ModLoader::NeoForge(modloader_version.unwrap_or_default()),
        _ => ModLoader::Vanilla,
    };

    let instance = Instance {
        id: version_id.clone(),
        name,
        version_id,
        modloader: ml,
        ..Default::default()
    };

    manager::create_instance(&instance)?;
    Ok(instance)
}

#[tauri::command]
pub async fn delete_instance(id: String) -> Result<(), String> {
    if id.starts_with("ext-") {
        return Err("外部实例请在“实例文件夹”中移除".into());
    }
    manager::delete_instance(&id)
}

#[tauri::command]
pub async fn list_instance_folders() -> Result<Vec<String>, String> {
    Ok(crate::commands::settings::load_config()
        .unwrap_or_else(|_| crate::commands::settings::LauncherConfig::default())
        .instance_folders)
}

#[tauri::command]
pub async fn auto_scan_instance_folders() -> Result<Vec<String>, String> {
    tokio::task::spawn_blocking(manager::auto_scan_instance_folders)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_instance_folder(folder: String) -> Result<Vec<String>, String> {
    let mut config = crate::commands::settings::load_config()
        .unwrap_or_else(|_| crate::commands::settings::LauncherConfig::default());
    let path = std::path::PathBuf::from(&folder);
    if !path.is_dir() {
        return Err("所选文件夹不存在".into());
    }
    let canonical = path
        .canonicalize()
        .map(|p| io::strip_extended_prefix(&p.to_string_lossy()))
        .unwrap_or_else(|_| io::strip_extended_prefix(&folder));
    if !config.instance_folders.iter().any(|f| f == &canonical) {
        config.instance_folders.push(canonical);
    }
    crate::commands::settings::save_config(config)?;
    list_instance_folders().await
}

#[tauri::command]
pub async fn remove_instance_folder(folder: String) -> Result<Vec<String>, String> {
    let mut config = crate::commands::settings::load_config()
        .unwrap_or_else(|_| crate::commands::settings::LauncherConfig::default());
    config.instance_folders.retain(|f| f != &folder);
    if let Some(active) = &config.active_instance_folder {
        if active == &folder {
            config.active_instance_folder = None;
        }
    }
    crate::commands::settings::save_config(config)?;
    manager::purge_external_residue(&std::path::PathBuf::from(&folder));
    list_instance_folders().await
}

#[tauri::command]
pub async fn list_home_instances() -> Result<Vec<Instance>, String> {
    manager::list_home_instances()
}

#[tauri::command]
pub async fn get_active_instance_folder() -> Result<Option<String>, String> {
    Ok(crate::commands::settings::load_config()
        .unwrap_or_else(|_| crate::commands::settings::LauncherConfig::default())
        .active_instance_folder)
}

#[tauri::command]
pub async fn set_active_instance_folder(folder: Option<String>) -> Result<Option<String>, String> {
    let mut config = crate::commands::settings::load_config()
        .unwrap_or_else(|_| crate::commands::settings::LauncherConfig::default());
    let valid = folder
        .as_ref()
        .filter(|f| !f.trim().is_empty())
        .map(|f| std::path::PathBuf::from(f).is_dir())
        .unwrap_or(false);
    config.active_instance_folder = if valid {
        folder.map(|f| io::strip_extended_prefix(&f))
    } else {
        None
    };
    crate::commands::settings::save_config(config)?;
    get_active_instance_folder().await
}

#[tauri::command]
pub async fn update_instance(instance: Instance) -> Result<(), String> {
    manager::update_instance(&instance)
}

#[tauri::command]
pub async fn fetch_versions() -> Result<crate::mc::version::VersionManifest, String> {
    install::fetch_versions(load_use_mirror()).await
}

#[tauri::command]
pub async fn install_version(
    version_id: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let _guard = InstallGuard::try_new(format!("mc|{}", version_id))?;
    let use_mirror = load_use_mirror();
    let threads = load_download_threads();
    install::install_minecraft(
        &version_id,
        use_mirror,
        threads,
        move |progress| {
            let _ = app_handle.emit("install-progress", &progress);
        },
    ).await.map(|_| ())
}

#[derive(serde::Serialize)]
pub struct InstalledVersion {
    pub id: String,
    pub has_jar: bool,
}

#[tauri::command]
pub async fn list_installed_versions() -> Result<Vec<InstalledVersion>, String> {
    let versions_dir = io::get_versions_dir();
    if !versions_dir.exists() {
        return Ok(Vec::new());
    }
    let mut versions = Vec::new();
    for entry in std::fs::read_dir(&versions_dir).map_err(|e| e.to_string())?.flatten() {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let id = entry.file_name().to_string_lossy().to_string();
        if !dir.join(format!("{}.json", id)).exists() {
            continue;
        }
        versions.push(InstalledVersion {
            has_jar: dir.join(format!("{}.jar", id)).exists(),
            id,
        });
    }
    versions.sort_by(|a, b| b.id.cmp(&a.id));
    Ok(versions)
}

#[tauri::command]
pub async fn analyze_crash(instance_id: String) -> Result<Option<crate::mc::crash::CrashAnalysis>, String> {
    let instance = manager::get_instance(&instance_id)?
        .ok_or_else(|| "Instance not found".to_string())?;
    let launch_dir = manager::get_instance_launch_dir(&instance);
    Ok(crate::mc::crash::analyze_latest_crash(&launch_dir)?)
}

#[tauri::command]
pub async fn list_instance_worlds(instance_id: String) -> Result<Vec<crate::mc::world::WorldInfo>, String> {
    let game_dir = manager::get_instance_mc_dir(&instance_id)?;
    let mut worlds = crate::mc::world::scan_worlds(&game_dir)?;
    for w in &mut worlds {
        if let Some(icon_path) = w.icon.clone() {
            w.icon = crate::mc::world::icon_as_data_uri(&std::path::PathBuf::from(icon_path));
        }
    }
    Ok(worlds)
}

#[tauri::command]
pub async fn list_multiplayer_servers(
    instance_id: String,
) -> Result<Vec<crate::mc::multiplayer::MultiplayerServer>, String> {
    let game_dir = manager::get_instance_mc_dir(&instance_id)?;
    let servers_dat = game_dir.join("servers.dat");
    if !servers_dat.exists() {
        return Ok(Vec::new());
    }
    crate::mc::multiplayer::read_servers_dat(&servers_dat)
}

/// Get world info including seed
#[tauri::command]
pub async fn get_world_info(_instance_id: String, world_path: String) -> Result<crate::mc::world::WorldInfo, String> {
    let path = std::path::PathBuf::from(&world_path);
    crate::mc::world::get_world_info(&path)
}

/// Generate map preview for a world
#[tauri::command]
pub async fn generate_map_preview(
    _instance_id: String,
    world_path: String,
    center_chunk_x: i32,
    center_chunk_z: i32,
    radius: usize,
) -> Result<crate::mc::world::MapPreview, String> {
    let path = std::path::PathBuf::from(&world_path);
    crate::mc::world::generate_map_preview(&path, center_chunk_x, center_chunk_z, radius)
}

/// Render a single region tile (512x512 blocks) for infiniter map loading.
/// Returns None when the region file does not exist.
#[tauri::command]
pub async fn world_map_region(
    world_path: String,
    region_x: i32,
    region_z: i32,
) -> Result<Option<crate::mc::region::RegionTile>, String> {
    let path = std::path::PathBuf::from(&world_path);
    Ok(crate::mc::region::render_region(&path, region_x, region_z))
}

/// Render a region tile with structure/ruin overlay from the world seed.
/// Returns the terrain tile plus structures near this region for overlay rendering.
#[tauri::command]
pub async fn world_map_region_with_structures(
    world_path: String,
    region_x: i32,
    region_z: i32,
) -> Result<Option<WorldMapTileWithStructures>, String> {
    let path = std::path::PathBuf::from(&world_path);
    let world_info = crate::mc::world::get_world_info(&path).map_err(|e| e.to_string())?;
    let tile = crate::mc::region::render_region(&path, region_x, region_z)
        .ok_or_else(|| "Region file not found".to_string())?;
    let structures = if let Some(seed) = world_info.seed {
        // Get all structures from seed, then filter to those near this region
        let _nearby = region_x.abs() + region_z.abs();
        let range = 3i32; // structures within ±3 regions of the requested tile
        let min_rx = region_x - range;
        let max_rx = region_x + range;
        let min_rz = region_z - range;
        let max_rz = region_z + range;
        let rresults = crate::mc::seedmap::seed_results(seed, "normal");
        rresults.structures.into_iter()
            .filter(|s| {
                let sx = s.x / 16;
                let sz = s.z / 16;
                sx >= min_rx && sx <= max_rx && sz >= min_rz && sz <= max_rz
            })
            .collect()
    } else {
        Vec::new()
    };
    Ok(Some(WorldMapTileWithStructures {
        tile,
        structures,
        seed: world_info.seed,
    }))
}

#[derive(serde::Serialize)]
pub struct WorldMapTileWithStructures {
    pub tile: crate::mc::region::RegionTile,
    pub structures: Vec<crate::mc::seedmap::SeedStructure>,
    pub seed: Option<i64>,
}

#[tauri::command]
pub fn seed_map_region(
    seed: i64,
    world_type: String,
    region_x: i32,
    region_z: i32,
) -> Result<crate::mc::seedmap::RegionTile, String> {
    Ok(crate::mc::seedmap::seed_map_region(seed, &world_type, region_x, region_z))
}

#[tauri::command]
pub fn seed_biome_region(
    seed: i64,
    world_type: String,
    region_x: i32,
    region_z: i32,
) -> Result<crate::mc::seedmap::RegionTile, String> {
    Ok(crate::mc::seedmap::seed_biome_region(seed, &world_type, region_x, region_z))
}

#[tauri::command]
pub fn seed_biome_at(
    seed: i64,
    world_type: String,
    x: i32,
    z: i32,
) -> Result<crate::mc::seedmap::SeedBiome, String> {
    Ok(crate::mc::seedmap::seed_biome_at(seed, &world_type, x, z))
}

#[tauri::command]
pub fn seed_results(
    seed: i64,
    world_type: String,
) -> Result<crate::mc::seedmap::SeedResults, String> {
    Ok(crate::mc::seedmap::seed_results(seed, &world_type))
}

#[derive(serde::Serialize)]
pub struct ScreenshotInfo {
    pub path: String,
    pub file_name: String,
    pub size_kb: u64,
    pub modified_at: Option<String>,
}
fn screenshot_ext_ok(ext: &str) -> bool {
    matches!(ext, "png" | "jpg" | "jpeg" | "bmp" | "gif" | "webp")
}

/// List screenshot files in the instance's game directory.
#[tauri::command]
pub async fn list_screenshots(instance_id: String) -> Result<Vec<ScreenshotInfo>, String> {
    let game_di = manager::get_instance_mc_dir(&instance_id)?;
    let shots_di = game_di.join("screenshots");
    if !shots_di.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for entry in std::fs::read_dir(&shots_di).map_err(|e| e.to_string())?.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let ext = path
            .extension()
            .map(|e| e.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        if !screenshot_ext_ok(&ext) {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let size_kb = (meta.len() / 1024).max(1);
        let modified_at = meta.modified().ok().and_then(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .ok()
                .map(|d| {
                    chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
                        .unwrap_or_default()
                })
        });
        out.push(ScreenshotInfo {
            path: path.to_string_lossy().to_string(),
            file_name: entry.file_name().to_string_lossy().to_string(),
            size_kb,
            modified_at,
        });
    }
    out.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
    Ok(out)
}



#[tauri::command]
pub async fn delete_screenshot(instance_id: String, file_name: String) -> Result<(), String> {
    if file_name.is_empty()
        || file_name.contains('/')
        || file_name.contains('\\')
        || file_name.contains("..")
    {
        return Err("非法文件名".into());
    }
    let game_di = manager::get_instance_mc_dir(&instance_id)?;
    let path = game_di.join("screenshots").join(&file_name);
    if !path.is_file() {
        return Err("截图不存在".into());
    }
    std::fs::remove_file(&path).map_err(|e| e.to_string())?;
    Ok(())
}



#[tauri::command]
pub async fn read_screenshot_base64(instance_id: String, file_name: String) -> Result<Option<String>, String> {
    if file_name.is_empty()
        || file_name.contains('/')
        || file_name.contains('\\')
        || file_name.contains("..")
    {
        return Err("非法文件名".into());
    }
    let game_di = manager::get_instance_mc_dir(&instance_id)?;
    let path = game_di.join("screenshots").join(&file_name);
    if !path.is_file() {
        return Ok(None);
    }
    const MAX: u64 = 16 * 1024 * 1024;
    let meta = std::fs::metadata(&path).map_err(|e| e.to_string())?;
    if meta.len() > MAX {
        return Ok(None);
    }
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    if bytes.is_empty() {
        return Ok(None);
    }
    use base64::Engine;
    let mime = if bytes.starts_with(b"\x89PNG") {
        "image/png"
    } else if bytes.starts_with(b"\xFF\xD8") {
        "image/jpeg"
    } else if bytes.starts_with(b"GIF8") {
        "image/gif"
    } else if bytes.starts_with(b"RIFF") && bytes.len() > 12 && &bytes[8..12] == b"WEBP" {
        "image/webp"
    } else {
        "image/png"
    };
    Ok(Some(format!(
        "data:{};base64,{}",
        mime,
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    )))
}


#[tauri::command]
#[allow(deprecated)]
pub async fn open_screenshot(
    instance_id: String,
    file_name: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    if file_name.is_empty()
        || file_name.contains('/')
        || file_name.contains('\\')
        || file_name.contains("..")
    {
        return Err("非法文件名".into());
    }
    use tauri_plugin_shell::ShellExt;
    let game_di = manager::get_instance_mc_dir(&instance_id)?;
    let path = game_di.join("screenshots").join(&file_name);
    if !path.is_file() {
        return Err("截图不存在".into());
    }
    app_handle
        .shell()
        .open(path.to_string_lossy().to_string(), None)
        .map_err(|e| e.to_string())?;
    Ok(())
}


#[tauri::command]
#[allow(deprecated)]
pub async fn open_file(path: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_shell::ShellExt;
    if path.contains('/') || path.contains('\\') {
        
    }
    app_handle
        .shell()
        .open(path, None)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn scan_data_packs(instance_id: String) -> Result<Vec<crate::instance::mods::ResourcePackInfo>, String> {
    let di = manager::get_instance_mc_dir(&instance_id)?;
    crate::instance::mods::scan_data_packs(&di)
}

#[tauri::command]
pub async fn toggle_data_pack(path: String, enable: bool) -> Result<(), String> {
    crate::instance::mods::toggle_data_pack(&path, enable)
}

#[tauri::command]
pub async fn delete_data_pack(path: String) -> Result<(), String> {
    crate::instance::mods::delete_data_pack(&path)
}

#[tauri::command]
pub async fn import_world_zip(
    instance_id: String,
    zip_path: String,
) -> Result<String, String> {
    let game_di = manager::get_instance_mc_dir(&instance_id)?;
    let path = std::path::PathBuf::from(&zip_path);
    let world_name = crate::mc::world::import_world(&game_di, &path)?;
    Ok(world_name)
}

#[tauri::command]
pub async fn import_world_from_url(
    url: String,
    filename: String,
    instance_id: String,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let temp = std::env::temp_dir();
    let threads = crate::commands::instance::load_download_threads();
    let app2 = app_handle.clone();
    let path = crate::modpack::download_file_to(&url, &filename, &temp, move |dl, all| {
        let pct = if all > 0 { dl as f64 / all as f64 } else { 0.0 };
        let _ = app2.emit(
            "install-progress",
            &crate::mc::install::InstallProgress {
                stage: "world".into(),
                progress: pct * 0.9,
                message: format!("正在下载地图... ({:.0}%)", pct * 100.0),
            },
        );
    }, threads)
    .await?;

    let game_di = manager::get_instance_mc_dir(&instance_id)?;
    let world_name = crate::mc::world::import_world(&game_di, &std::path::PathBuf::from(&path))?;
    std::fs::remove_file(&path).ok();
    let _ = app_handle.emit(
        "install-progress",
        &crate::mc::install::InstallProgress {
            stage: "world".into(),
            progress: 1.0,
            message: format!("地图「{}」安装完成", world_name),
        },
    );
    Ok(world_name)
}

#[tauri::command]
pub async fn delete_world(instance_id: String, world_name: String) -> Result<(), String> {
    let game_di = manager::get_instance_mc_dir(&instance_id)?;
    crate::mc::world::delete_world(&game_di, &world_name)
}
