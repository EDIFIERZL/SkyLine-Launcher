use std::path::PathBuf;

pub fn run_first_time_tasks() {
    
    if !PathBuf::from(r"D:\").is_dir() {
        return;
    }

    let launcher_root = crate::utils::io::get_launcher_root();
    let minecraft_root = crate::utils::io::get_minecraft_di();

    let did_setup = setup_minecraft_di(&minecraft_root)
        || setup_launcher_di(&launcher_root);

    if did_setup {
        std::fs::create_dir_all(&launcher_root).ok();
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            let path = launcher_root.to_string_lossy().replace('/', "\\");
            let _ = Command::new("cmd")
                .args(["/C", "attrib", "+h", &format!("\"{}\"", path)])
                .output();
        }
    }
}

fn setup_minecraft_di(mc: &std::path::Path) -> bool {
    if mc.exists() && has_existing_content(mc) {
        return false;
    }
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
    let mut created = false;
    for di in &dirs {
        if std::fs::create_dir_all(mc.join(di)).is_ok() {
            created = true;
        }
    }
    created
}

fn setup_launcher_di(launche: &std::path::Path) -> bool {
    if launche.exists() && has_existing_content(launche) {
        return false;
    }
    std::fs::create_dir_all(launche).is_ok()
}

fn has_existing_content(di: &std::path::Path) -> bool {
    di.read_dir()
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false)
}
