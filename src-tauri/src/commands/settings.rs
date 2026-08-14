use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::OnceLock;
use crate::instance::IsolationMode;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VersionSetting {
    pub java_path: Option<String>,
    pub min_memory: Option<u32>,
    pub max_memory: Option<u32>,
    #[serde(alias = "version_isolation")]
    pub isolation_mode: Option<IsolationMode>,
    #[serde(default)]
    pub game_dir_override: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LauncherConfig {
    #[serde(default)]
    pub theme: String,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub download_threads: u32,
    #[serde(default)]
    pub max_memory: u32,
    #[serde(default)]
    pub min_memory: u32,
    #[serde(default)]
    pub java_path: Option<String>,
    #[serde(default)]
    pub keep_launcher_open: bool,
    #[serde(default, alias = "close_afte_launch")]
    pub close_after_launch: bool,

    #[serde(default)]
    pub theme_mode: String,
    #[serde(default, alias = "accent_colo")]
    pub accent_color: String,
    #[serde(default, alias = "backgound_type")]
    pub background_type: String,
    #[serde(default, alias = "backgound_value")]
    pub background_value: String,
    #[serde(default)]
    pub ui_scale: f64,

    #[serde(default)]
    pub font_size: String,
    #[serde(default, alias = "sideba_collapsed")]
    pub sidebar_collapsed: bool,
    #[serde(default)]
    pub use_mirror: bool,
    #[serde(default)]
    pub window_size: String,
    #[serde(default)]
    pub window_width: u32,
    #[serde(default)]
    pub window_height: u32,
    #[serde(default)]
    pub version_settings: HashMap<String, VersionSetting>,
    #[serde(default)]
    pub instance_folders: Vec<String>,
    #[serde(default)]
    pub active_instance_folder: Option<String>,
    #[serde(default)]
    pub last_selected_instance: Option<String>,
    #[serde(default)]
    pub download_source: String,
    #[serde(default, alias = "server_addess")]
    pub server_address: String,
    #[serde(default)]
    pub server_name: String,
    #[serde(default, alias = "hide_server_cad")]
    pub hide_server_card: bool,
    #[serde(default, alias = "hide_mp_quick_cad")]
    pub hide_mp_quick_card: bool,
    #[serde(default = "default_server_card_size", alias = "server_cad_size")]
    pub server_card_size: u32,
    #[serde(default)]
    pub liquid_glass: bool,
    #[serde(default)]
    pub liquid_glass_mode: String,
    #[serde(default = "default_glass_intensity")]
    pub liquid_glass_intensity: f64,
    #[serde(default)]
    pub jvm_args: Vec<String>,
    #[serde(default)]
    pub opengl_compat: bool,
    #[serde(default)]
    pub game_folder: Option<String>,
}

fn default_glass_intensity() -> f64 {
    1.0
}

fn default_server_card_size() -> u32 {
    80
}

impl Default for LauncherConfig {
    fn default() -> Self {
        Self {
            theme: "dark".into(),
            language: "zh-CN".into(),
            download_threads: 64,
            max_memory: 4096,
            min_memory: 1024,
            java_path: None,
            keep_launcher_open: true,
            close_after_launch: false,
            theme_mode: "dark".into(),
            accent_color: "#3b82f6".into(),
            background_type: "none".into(),
            background_value: String::new(),
            ui_scale: 1.0,

            font_size: "normal".into(),
            sidebar_collapsed: false,
            use_mirror: true,
            window_size: "1200x720".into(),
            window_width: 1200,
            window_height: 720,
            version_settings: HashMap::new(),
            instance_folders: Vec::new(),
            active_instance_folder: None,
            last_selected_instance: None,
            download_source: "auto".into(),
            server_address: String::new(),
            server_name: String::new(),
            hide_server_card: false,
            hide_mp_quick_card: false,
            server_card_size: 80,
            liquid_glass: false,
            liquid_glass_mode: "normal".into(),
            liquid_glass_intensity: 1.0,
            jvm_args: Vec::new(),
            opengl_compat: false,
            game_folder: None,
        }
    }
}

static CONFIG_CACHE: OnceLock<std::sync::Mutex<(std::path::PathBuf, LauncherConfig)>> = OnceLock::new();

fn config_path() -> std::path::PathBuf {
    crate::utils::io::get_config_file()
}

#[tauri::command]
pub fn load_config() -> Result<LauncherConfig, String> {
    let path = config_path();
    let cache = CONFIG_CACHE.get_or_init(|| {
        std::sync::Mutex::new((std::path::PathBuf::new(), LauncherConfig::default()))
    });
    let mut guard = cache.lock().map_err(|e| e.to_string())?;
    if guard.0 != path {
        let cfg = load_config_from(&path)?;
        guard.0 = path;
        guard.1 = cfg.clone();
        return Ok(cfg);
    }
    Ok(guard.1.clone())
}

pub fn load_config_from(path: &std::path::Path) -> Result<LauncherConfig, String> {
    if !path.exists() {
        return Ok(LauncherConfig::default());
    }
    let content = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
    serde_json::from_str(&content).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_config(config: LauncherConfig) -> Result<(), String> {
    let path = config_path();
    if let Some(prent) = path.parent() {
        std::fs::create_dir_all(prent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&path, &content).map_err(|e| e.to_string())?;
    if let Some(cache) = CONFIG_CACHE.get() {
        let mut guad = cache.lock().map_err(|e| e.to_string())?;
        guad.1 = config;
    }
    Ok(())
}

#[tauri::command]
pub fn set_last_selected_instance(instance_id: Option<String>) -> Result<(), String> {
    let mut config = load_config()?;
    config.last_selected_instance = instance_id;
    save_config(config)
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportedMedia {
    pub kind: String,
    pub data_uri: String,
}

#[tauri::command]
pub fn read_background_media(path: String) -> Result<ImportedMedia, String> {
    use base64::Engine;
    let path = std::path::PathBuf::from(path);
    if !path.exists() {
        return Err("文件不存在".into());
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let (mime, kind) = match ext.as_str() {
        "png" => ("image/png", "image"),
        "jpg" | "jpeg" => ("image/jpeg", "image"),
        "gif" => ("image/gif", "image"),
        "webp" => ("image/webp", "image"),
        "bmp" => ("image/bmp", "image"),
        "svg" => ("image/svg+xml", "image"),
        "avif" => ("image/avif", "image"),
        "ico" => ("image/x-icon", "image"),
        "mp4" => ("video/mp4", "video"),
        "webm" => ("video/webm", "video"),
        "mov" | "m4v" => ("video/quicktime", "video"),
        "ogg" => ("video/ogg", "video"),
        _ => return Err(format!("不支持的文件类型: .{ext}")),
    };
    let bytes = std::fs::read(&path).map_err(|e| e.to_string())?;
    const MAX_SIZE: usize = 50 * 1024 * 1024;
    if bytes.len() > MAX_SIZE {
        return Err(format!("文件过大（{}MB），请选择小于 50MB 的文件", bytes.len() / 1024 / 1024));
    }
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(ImportedMedia {
        kind: kind.into(),
        data_uri: format!("data:{mime};base64,{b64}"),
    })
}

#[tauri::command]
pub fn set_game_folder(folder: String) -> Result<(), String> {
    let mut config = load_config()?;
    config.game_folder = if folder.is_empty() { None } else { Some(folder) };
    save_config(config)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigationResult {
    pub copied_count: u64,
    pub copied_size: u64,
    pub instance_count: usize,
}

#[tauri::command]
pub async fn migrate_game_folder(
    old_path: String,
    new_path: String,
) -> Result<MigationResult, String> {
    let old = std::path::PathBuf::from(&old_path);
    let new = std::path::PathBuf::from(&new_path);

    if !old.exists() {
        return Err(format!("源文件夹不存在: {}", old_path));
    }
    let old_canon = old.canonicalize().map_err(|e| e.to_string())?;
    let new_canon = new.canonicalize().unwrap_or(new.clone());
    if old_canon == new_canon {
        return Err("源文件夹和目标文件夹相同".to_string());
    }

    std::fs::create_dir_all(&new).map_err(|e| format!("创建目标文件夹失败: {}", e))?;

    
    let copy_handle = tokio::task::spawn_blocking({
        let old = old.clone();
        let new = new.clone();
        move || copy_di_sync(&old, &new)
    });

    
    {
        let mut config = load_config()?;
        config.game_folder = Some(new_path.clone());
        save_config(config)?;
    }

    
    let mut instance_count: usize = 0;
    let old_st = old_canon.to_string_lossy().to_string();
    let new_st = new_canon.to_string_lossy().to_string();

    
    if let Ok(entries) = std::fs::read_dir(crate::utils::io::get_instances_di()) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() { continue; }
            let meta = path.join(".skyline").join("instance.json");
            if !meta.exists() { continue; }
            let content = match std::fs::read_to_string(&meta) {
                Ok(c) => c,
                Err(_) => continue,
            };
            let mut inst: crate::instance::Instance = match serde_json::from_str(&content) {
                Ok(i) => i,
                Err(_) => continue,
            };
            let mut changed = false;
            if let Some(ref mut gd) = inst.game_dir_override {
                if gd == &old_st { *gd = new_st.clone(); changed = true; }
            }
            if let Some(ref mut m) = inst.minecraft_root {
                if m == &old_st { *m = new_st.clone(); changed = true; }
            }
            if changed {
                let json = serde_json::to_string_pretty(&inst).map_err(|e| e.to_string())?;
                std::fs::write(&meta, &json).map_err(|e| e.to_string())?;
                instance_count += 1;
            }
        }
    }

    
    let config = load_config()?;
    for folder in &config.instance_folders {
        let folder_path = std::path::PathBuf::from(crate::utils::io::strip_extended_prefix(folder));
        if !folder_path.is_dir() { continue; }
        if let Ok(entries) = std::fs::read_dir(&folder_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() { continue; }
                let meta = path.join(".skyline").join("instance.json");
                if !meta.exists() { continue; }
                let content = match std::fs::read_to_string(&meta) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                let mut inst: crate::instance::Instance = match serde_json::from_str(&content) {
                    Ok(i) => i,
                    Err(_) => continue,
                };
                if !inst.external { continue; }
                let mut changed = false;
                if let Some(ref mut gd) = inst.game_dir_override {
                    if gd == &old_st { *gd = new_st.clone(); changed = true; }
                }
                if let Some(ref mut m) = inst.minecraft_root {
                    if m == &old_st { *m = new_st.clone(); changed = true; }
                }
                if changed {
                    let json = serde_json::to_string_pretty(&inst).map_err(|e| e.to_string())?;
                    std::fs::write(&meta, &json).map_err(|e| e.to_string())?;
                    instance_count += 1;
                }
            }
        }
    }

    
    copy_handle.await.map_err(|e| e.to_string())??;

    Ok(MigationResult {
        copied_count: 0,
        copied_size: 0,
        instance_count,
    })
}

fn copy_di_sync(sc: &std::path::Path, dst: &std::path::Path) -> Result<(), String> {
    if !sc.exists() { return Ok(()); }
    if !dst.exists() {
        std::fs::create_dir_all(dst).map_err(|e| e.to_string())?;
    }
    for entry in std::fs::read_dir(sc).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let sc_path = entry.path();
        let dst_path = dst.join(entry.file_name());
        if sc_path.is_dir() {
            copy_di_sync(&sc_path, &dst_path)?;
        } else {
            std::fs::copy(&sc_path, &dst_path)
                .map_err(|e| format!("复制 {} 失败: {}", sc_path.display(), e))?;
        }
    }
    Ok(())
}
