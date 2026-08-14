use std::path::PathBuf;

pub fn no_window(cmd: &mut std::process::Command) -> &mut std::process::Command {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        
        cmd.creation_flags(0x0800_0000)
    }
    #[cfg(not(target_os = "windows"))]
    {
        cmd
    }
}

pub fn get_base_di() -> PathBuf {
    let d = PathBuf::from(r"D:\");
    if d.is_dir() {
        return d;
    }
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .or_else(|| dirs::data_dir())
        .unwrap_or_else(|| PathBuf::from("."))
}

pub fn strip_extended_prefix(p: &str) -> String {
    if let Some(est) = p.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{}", est)
    } else if let Some(est) = p.strip_prefix(r"\\?\") {
        est.to_string()
    } else {
        p.to_string()
    }
}

pub fn get_minecraft_di() -> PathBuf {
    get_base_di().join(".minecraft")
}

pub fn get_launcher_root() -> PathBuf {
    get_base_di().join(".skyline launcher")
}

pub fn get_instances_di() -> PathBuf {
    get_minecraft_di().join("versions")
}

pub fn get_java_dir() -> PathBuf {
    get_launcher_root().join("java")
}

pub fn get_skins_di() -> PathBuf {
    get_launcher_root().join("skins")
}

pub fn get_shared_dir() -> PathBuf {
    get_minecraft_di()
}

pub fn get_libraries_dir() -> PathBuf {
    get_minecraft_di().join("libraries")
}

pub fn get_assets_dir() -> PathBuf {
    get_minecraft_di().join("assets")
}

pub fn get_versions_dir() -> PathBuf {
    get_minecraft_di().join("versions")
}

pub fn ensure_minecraft_structure() {
    let mc = get_minecraft_di();
    let dirs = [
        "assets/indexes",
        "assets/objects",
        "assets/skins",
        "worlds",
        "config",
        "crash-reports",
        "data",
        "debug",
        "downloads",
        "logs",
        "natives",
        "schematics",
        "screenshots",
        "shaderpacks",
        "versions",
        "libraries",
        "backups",
        "defaultconfigs",
        ".fabric",
        ".cache",
    ];
    for di in dirs {
        std::fs::create_dir_all(mc.join(di)).ok();
    }
}

pub fn get_config_file() -> PathBuf {
    get_launcher_root().join("config.json")
}

pub fn get_instance_dir(id: &str) -> PathBuf {
    get_instances_di().join(id)
}

pub fn get_instance_skyline_di(id: &str) -> PathBuf {
    get_instance_dir(id).join(".skyline")
}

pub fn get_instance_meta_file(id: &str) -> PathBuf {
    get_instance_skyline_di(id).join("instance.json")
}

pub fn get_old_instances_di() -> PathBuf {
    get_launcher_root().join("instances")
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' })
        .collect()
}
