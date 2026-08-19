use crate::modpack;
use tauri::Emitter;

async fn ensure_game_and_loader(instance_id: &str, app: tauri::AppHandle) -> Result<String, String> {
    let instance = crate::instance::manager::get_instance(instance_id)?
        .ok_or_else(|| "Instance not found".to_string())?;
    let mc_version = instance.version_id.clone();
    let use_mirror = crate::commands::instance::load_use_mirror();
    let shared = crate::utils::io::get_shared_dir();

    let app2 = app.clone();
    let threads = crate::commands::instance::load_download_threads();
    crate::mc::install::install_minecraft(&mc_version, use_mirror, threads, move |p| {
        let _ = app2.emit("install-progress", &p);
    })
    .await
    .map_err(|e| e.to_string())?;

    let final_id: String = match &instance.modloader {
        crate::instance::ModLoader::Vanilla => mc_version.clone(),
        crate::instance::ModLoader::Forge(v) if !v.is_empty() => {
            crate::mc::modloader::install_forge(&shared, &mc_version, v, use_mirror).await?;
            crate::mc::modloader::get_forge_version_id(&mc_version, v)
        }
        crate::instance::ModLoader::NeoForge(v) if !v.is_empty() => {
            crate::mc::modloader::install_neoforge(&shared, &mc_version, v, use_mirror).await?;
            crate::mc::modloader::get_neoforge_version_id(&mc_version, v)
        }
        crate::instance::ModLoader::Fabric(v) => {
            let loader_version = if v.is_empty() { "latest" } else { v };
            crate::mc::modloader::install_fabric(&shared, &mc_version, loader_version, use_mirror).await?
        }
        crate::instance::ModLoader::Quilt(v) => {
            let loader_version = if v.is_empty() { "latest" } else { v };
            crate::mc::modloader::install_quilt(&shared, &mc_version, loader_version, use_mirror).await?
        }
        _ => mc_version.clone(),
    };

    if final_id != mc_version {
        let mut updated = instance.clone();
        updated.id = final_id.clone();
        updated.version_id = final_id.clone();
        
        let new_skyline_di = crate::utils::io::get_instance_skyline_di(&final_id);
        std::fs::create_dir_all(&new_skyline_di).map_err(|e| e.to_string())?;
        
        crate::instance::manager::update_instance(&updated)?;
    }
    Ok(final_id)
}

#[tauri::command]
pub async fn search_modrinth_mods(query: String, limit: u32, offset: u32, game_version: Option<String>, loaders: Option<Vec<String>>) -> Result<Vec<modpack::ModrinthProject>, String> {
    modpack::search_modrinth(&query, limit, offset, game_version.as_deref(), loaders.as_ref().map(|v| &**v)).await
}

#[tauri::command]
pub async fn recommended_mods(limit: u32, game_version: Option<String>, loaders: Option<Vec<String>>) -> Result<Vec<modpack::ModrinthProject>, String> {
    modpack::recommended_mods(limit, game_version.as_deref(), loaders.as_ref().map(|v| &**v)).await
}

#[tauri::command]
pub async fn recommended_resource_packs(limit: u32, game_version: Option<String>) -> Result<Vec<modpack::ModrinthProject>, String> {
    modpack::recommended_resource_packs(limit, game_version.as_deref()).await
}

#[tauri::command]
pub async fn recommended_shader_packs(limit: u32, game_version: Option<String>) -> Result<Vec<modpack::ModrinthProject>, String> {
    modpack::recommended_shader_packs(limit, game_version.as_deref()).await
}

#[tauri::command]
pub async fn recommended_modpacks(limit: u32, game_version: Option<String>) -> Result<Vec<modpack::ModrinthProject>, String> {
    modpack::recommended_modpacks(limit, game_version.as_deref()).await
}

#[tauri::command]
pub async fn recommended_datapacks(limit: u32, game_version: Option<String>) -> Result<Vec<modpack::ModrinthProject>, String> {
    modpack::recommended_datapacks(limit, game_version.as_deref()).await
}

#[tauri::command]
pub async fn recommended_worlds(limit: u32, game_version: Option<String>) -> Result<Vec<modpack::ModrinthProject>, String> {
    modpack::recommended_worlds(limit, game_version.as_deref()).await
}

#[tauri::command]
pub async fn search_resource_packs(query: String, limit: u32, offset: u32, game_version: Option<String>) -> Result<Vec<modpack::ModrinthProject>, String> {
    modpack::search_resource_packs(&query, limit, offset, game_version.as_deref(), None).await
}

#[tauri::command]
pub async fn search_shader_packs(query: String, limit: u32, offset: u32, game_version: Option<String>) -> Result<Vec<modpack::ModrinthProject>, String> {
    modpack::search_shader_packs(&query, limit, offset, game_version.as_deref(), None).await
}

#[tauri::command]
pub async fn search_datapacks(query: String, limit: u32, offset: u32, game_version: Option<String>) -> Result<Vec<modpack::ModrinthProject>, String> {
    modpack::search_datapacks(&query, limit, offset, game_version.as_deref()).await
}

#[tauri::command]
pub async fn search_worlds(query: String, limit: u32, offset: u32, game_version: Option<String>) -> Result<Vec<modpack::ModrinthProject>, String> {
    modpack::search_worlds(&query, limit, offset, game_version.as_deref()).await
}

#[tauri::command]
pub async fn search_modpacks(query: String, limit: u32, offset: u32, game_version: Option<String>) -> Result<Vec<modpack::ModrinthProject>, String> {
    modpack::search_modpacks(&query, limit, offset, game_version.as_deref()).await
}

#[tauri::command]
pub async fn get_modrinth_versions(project_id: String) -> Result<Vec<modpack::ModrinthVersion>, String> {
    modpack::get_modrinth_versions(&project_id).await
}

#[tauri::command]
pub async fn get_modrinth_project_detail(slug: String) -> Result<modpack::ModrinthProjectDetail, String> {
    modpack::get_modrinth_project_detail(&slug).await
}

#[tauri::command]
pub async fn download_file(
    url: String,
    filename: String,
    instance_id: String,
    target: Option<String>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let mc_di = crate::instance::manager::get_instance_mc_dir(&instance_id)?;
    let sub_di = target.as_deref().unwrap_or("mods");
    let di = mc_di.join(sub_di);
    std::fs::create_dir_all(&di).map_err(|e| e.to_string())?;
    let threads = crate::commands::instance::load_download_threads();
    let app = app_handle.clone();
    modpack::download_file_to(&url, &filename, &di, move |done, total| {
        let pct = if total > 0 { done as f64 / total as f64 } else { 0.0 };
        let _ = app.emit(
            "install-progress",
            &crate::mc::install::InstallProgress {
                stage: "mod".into(),
                progress: pct,
                message: format!("正在下载... ({:.0}%)", pct * 100.0),
            },
        );
    }, threads)
    .await
}

#[tauri::command]
pub async fn install_modrinth_modpack(
    version_id: String,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let url = format!("https://api.modrinth.com/v2/version/{}", version_id);
    let client = crate::mc::mirror::http_client();
    let esp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let version: modpack::ModrinthVersion = esp.json().await.map_err(|e| e.to_string())?;
    let primary = version
        .files
        .iter()
        .find(|f| f.primary)
        .unwrap_or(&version.files[0]);

    let temp = std::env::temp_dir();
    let threads = crate::commands::instance::load_download_threads();
    let app2 = app_handle.clone();
    let path = modpack::download_file_to(
        &primary.url,
        &primary.filename,
        &temp,
        move |done, total| {
            let pct = if total > 0 { done as f64 / total as f64 } else { 0.0 };
            let _ = app2.emit(
                "install-progress",
                &crate::mc::install::InstallProgress {
                    stage: "modpack".into(),
                    progress: 0.5 * pct,
                    message: format!("正在下载整合包... ({:.0}%)", pct * 100.0),
                },
            );
        },
        threads,
    )
    .await?;

    let instance_id = modpack::modpack::import_modrinth_pack(&std::path::PathBuf::from(&path)).await?;
    std::fs::remove_file(&path).ok();

    ensure_game_and_loader(&instance_id, app_handle).await?;
    Ok(instance_id)
}

#[tauri::command]
pub async fn export_modrinth_pack(instance_id: String, output_path: String) -> Result<String, String> {
    modpack::modpack::export_modrinth_pack(&instance_id, &std::path::PathBuf::from(&output_path)).await
}

#[tauri::command]
pub async fn import_modrinth_pack(pack_path: String, app_handle: tauri::AppHandle) -> Result<String, String> {
    let instance_id = modpack::modpack::import_modrinth_pack(&std::path::PathBuf::from(&pack_path)).await?;
    ensure_game_and_loader(&instance_id, app_handle).await?;
    Ok(instance_id)
}

#[tauri::command]
pub async fn import_mmc_pack(pack_path: String, app_handle: tauri::AppHandle) -> Result<String, String> {
    let instance_id = modpack::modpack::import_mmc_pack(&std::path::PathBuf::from(&pack_path)).await?;
    ensure_game_and_loader(&instance_id, app_handle).await?;
    Ok(instance_id)
}

#[tauri::command]
pub async fn import_hmcl_pack(pack_path: String, app_handle: tauri::AppHandle) -> Result<String, String> {
    let instance_id = modpack::modpack::import_hmcl_pack(&std::path::PathBuf::from(&pack_path)).await?;
    ensure_game_and_loader(&instance_id, app_handle).await?;
    Ok(instance_id)
}

#[tauri::command]
pub async fn detect_modpack_type(pack_path: String) -> Result<String, String> {
    let pack_type = modpack::modpack::detect_modpack_type(&std::path::PathBuf::from(&pack_path))?;
    Ok(match pack_type {
        modpack::modpack::ModpackType::Modrinth => "modrinth".to_string(),
        modpack::modpack::ModpackType::MMC => "mmc".to_string(),
        modpack::modpack::ModpackType::HMCL => "hmcl".to_string(),
        modpack::modpack::ModpackType::MCBBS => "mcbbs".to_string(),
        modpack::modpack::ModpackType::LaunchePack => "launcherpack".to_string(),
        modpack::modpack::ModpackType::Achive => "archive".to_string(),
        modpack::modpack::ModpackType::Unknown => "unknown".to_string(),
    })
}

#[tauri::command]
pub async fn check_mod_updates(instance_id: String) -> Result<Vec<modpack::ModUpdateInfo>, String> {
    let mc_di = crate::instance::manager::get_instance_mc_dir(&instance_id)?;
    let mods_di = mc_di.join("mods");
    modpack::check_mod_updates(&mods_di).await
}

#[tauri::command]
pub async fn download_modrinth_mod(
    version_id: String,
    instance_id: String,
    target: Option<String>,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let mc_di = crate::instance::manager::get_instance_mc_dir(&instance_id)?;
    
    let sub_di = target.as_deref().unwrap_or("mods");
    let mods_di = mc_di.join(sub_di);
    std::fs::create_dir_all(&mods_di).map_err(|e| e.to_string())?;
    let threads = crate::commands::instance::load_download_threads();
    let app = app_handle.clone();
    modpack::download_modrinth_mod(&version_id, &mods_di, move |done, total| {
        let pct = if total > 0 { done as f64 / total as f64 } else { 0.0 };
        let _ = app.emit(
            "install-progress",
            &crate::mc::install::InstallProgress {
                stage: "mod".into(),
                progress: pct,
                message: format!("正在下载模组... ({:.0}%)", pct * 100.0),
            },
        );
    }, threads)
    .await
}

#[tauri::command]
pub async fn resolve_modrinth_dependencies(
    version_id: String,
    instance_id: String,
    mc_version: String,
    loader: Option<String>,
) -> Result<Vec<modpack::ModrinthDependency>, String> {
    let mc_di = crate::instance::manager::get_instance_mc_dir(&instance_id)?;
    let mods_di = mc_di.join("mods");
    std::fs::create_dir_all(&mods_di).map_err(|e| e.to_string())?;

    let loader = loader.unwrap_or_default();
    let deps = modpack::resolve_modrinth_dependencies(&version_id, &mc_version, &loader).await?;
    filter_existing_deps(&mods_di, deps).await
}

async fn filter_existing_deps(
    mods_di: &std::path::Path,
    deps: Vec<modpack::ModrinthDependency>,
) -> Result<Vec<modpack::ModrinthDependency>, String> {
    let mut existing_sha1: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut existing_names: std::collections::HashSet<String> = std::collections::HashSet::new();
    if let Ok(entries) = std::fs::read_dir(mods_di) {
        for entry in entries.flatten() {
            let p = entry.path();
            if !p.is_file() {
                continue;
            }
            let fname = p
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            existing_names.insert(fname.to_lowercase());
            if let Ok(file) = std::fs::File::open(&p) {
                if let Ok(h) = crate::utils::crypto::sha1_hex(file) {
                    existing_sha1.insert(h);
                }
            }
        }
    }

    let mut remaining: Vec<modpack::ModrinthDependency> = Vec::new();
    for dep in deps {
        let dep_vid = match &dep.version_id {
            Some(v) if !v.is_empty() => v.clone(),
            _ => continue,
        };
        if let Ok(v) = modpack::fetch_modrinth_version(&dep_vid).await {
            let primary = v.files.iter().find(|f| f.primary).unwrap_or(&v.files[0]);
            let fname = primary.filename.to_lowercase();
            let sha1 = &primary.hashes.sha1;
            if existing_names.contains(&fname) || existing_sha1.contains(sha1) {
                continue;
            }
        }
        remaining.push(dep);
    }

    Ok(remaining)
}

#[tauri::command]
pub async fn install_modrinth_content(
    version_id: String,
    instance_id: String,
    target: Option<String>,
    app_handle: tauri::AppHandle,
) -> Result<serde_json::Value, String> {
    let mc_di = crate::instance::manager::get_instance_mc_dir(&instance_id)?;
    let sub_di = target.as_deref().unwrap_or("mods");
    let dir = mc_di.join(sub_di);
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;

    let (mc_version, loader) = match crate::instance::manager::get_instance(&instance_id)? {
        Some(inst) => {
            let l = match &inst.modloader {
                crate::instance::ModLoader::Forge(_) => "forge",
                crate::instance::ModLoader::NeoForge(_) => "neoforge",
                crate::instance::ModLoader::Fabric(_) => "fabric",
                crate::instance::ModLoader::Quilt(_) => "quilt",
                _ => "",
            };
            (inst.version_id.clone(), l.to_string())
        }
        None => (String::new(), String::new()),
    };

    let primary = modpack::fetch_modrinth_version(&version_id).await?;
    let deps = filter_existing_deps(
        &dir,
        modpack::resolve_modrinth_dependencies(&version_id, &mc_version, &loader).await?,
    )
    .await?;

    let threads = crate::commands::instance::load_download_threads();
    let total = (1 + deps.len()) as f64;
    let mut installed: Vec<String> = Vec::new();

    let primary_file = primary
        .files
        .iter()
        .find(|f| f.primary)
        .unwrap_or(&primary.files[0]);

    {
        let app2 = app_handle.clone();
        let url = primary_file.url.clone();
        let filename = primary_file.filename.clone();
        let name = primary.name.clone();
        let dir2 = dir.clone();
        let _ = app_handle.emit(
            "install-progress",
            &crate::mc::install::InstallProgress {
                stage: "mod".into(),
                progress: 0.0,
                message: format!("正在下载 {}...", name),
            },
        );
        modpack::download_file_to(&url, &filename, &dir2, move |dl, all| {
            let pct = if all > 0 { dl as f64 / all as f64 } else { 0.0 };
            let _ = app2.emit(
                "install-progress",
                &crate::mc::install::InstallProgress {
                    stage: "mod".into(),
                    progress: pct / total,
                    message: format!("正在下载 {} ({:.0}%)", name, pct * 100.0),
                },
            );
        }, threads)
        .await?;
        installed.push(primary_file.filename.clone());
    }

    for (i, dep) in deps.iter().enumerate() {
        let dep_vid = match &dep.version_id {
            Some(v) if !v.is_empty() => v.clone(),
            _ => continue,
        };
        let v = modpack::fetch_modrinth_version(&dep_vid).await?;
        let file = v.files.iter().find(|f| f.primary).unwrap_or(&v.files[0]);
        let name = dep
            .file_name
            .clone()
            .unwrap_or_else(|| file.filename.clone());
        let app2 = app_handle.clone();
        let url = file.url.clone();
        let filename = file.filename.clone();
        let dir2 = dir.clone();
        let base = (i as f64) + 1.0;
        let _ = app_handle.emit(
            "install-progress",
            &crate::mc::install::InstallProgress {
                stage: "mod".into(),
                progress: base / total,
                message: format!("正在下载前置 {} ({}/{})...", name, i + 1, deps.len()),
            },
        );
        modpack::download_file_to(&url, &filename, &dir2, move |dl, all| {
            let pct = if all > 0 { dl as f64 / all as f64 } else { 0.0 };
            let _ = app2.emit(
                "install-progress",
                &crate::mc::install::InstallProgress {
                    stage: "mod".into(),
                    progress: (base + pct) / total,
                    message: format!("正在下载前置 {} ({:.0}%)", name, pct * 100.0),
                },
            );
        }, threads)
        .await?;
        installed.push(file.filename.clone());
    }

    let _ = app_handle.emit(
        "install-progress",
        &crate::mc::install::InstallProgress {
            stage: "mod".into(),
            progress: 1.0,
            message: format!("安装完成，共 {} 个文件", installed.len()),
        },
    );

    Ok(serde_json::json!({
        "installed": installed,
        "dependency_count": deps.len(),
    }))
}

#[tauri::command]
pub async fn install_modrinth_map(
    version_id: String,
    instance_id: String,
    app_handle: tauri::AppHandle,
) -> Result<String, String> {
    let version = modpack::fetch_modrinth_version(&version_id).await?;
    let primary = version
        .files
        .iter()
        .find(|f| f.primary)
        .unwrap_or(&version.files[0]);

    let temp = std::env::temp_dir();
    let threads = crate::commands::instance::load_download_threads();
    let app2 = app_handle.clone();
    let url = primary.url.clone();
    let filename = primary.filename.clone();
    let path = modpack::download_file_to(&url, &filename, &temp, move |dl, all| {
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

    let mc_di = crate::instance::manager::get_instance_mc_dir(&instance_id)?;
    let world_name = crate::mc::world::import_world(&mc_di, &std::path::PathBuf::from(&path))?;
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
