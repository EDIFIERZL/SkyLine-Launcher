use crate::instance::{Instance, ModLoader};
use crate::utils::io;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct InstanceStats {
    #[serde(default)]
    pub play_time: u64,
    #[serde(default)]
    pub last_playerd: Option<String>,
}

fn stats_path(id: &str) -> PathBuf {
    io::get_launcher_root()
        .join("instance_stats")
        .join(format!("{}.json", sanitize_name(id)))
}

pub fn load_instance_stats(id: &str) -> InstanceStats {
    let path = stats_path(id);
    if let Ok(content) = std::fs::read_to_string(&path) {
        if let Ok(stats) = serde_json::from_str::<InstanceStats>(&content) {
            return stats;
        }
    }
    InstanceStats::default()
}

pub fn save_instance_stats(id: &str, stats: &InstanceStats) {
    if let Some(prent) = stats_path(id).parent() {
        let _ = std::fs::create_dir_all(prent);
    }
    if let Ok(json) = serde_json::to_string_pretty(stats) {
        let _ = std::fs::write(stats_path(id), json);
    }
}





pub fn hydate_instance_stats(inst: &mut Instance) {
    if !inst.external {
        return;
    }
    let stats = load_instance_stats(&inst.id);
    if stats.play_time != 0 {
        inst.play_time = stats.play_time;
    }
    if stats.last_playerd.is_some() {
        inst.last_playerd = stats.last_playerd;
    }
}



pub fn record_play_time(inst: &mut Instance, secs: u64, last_playerd: String) {
    if inst.external {
        let mut stats = load_instance_stats(&inst.id);
        stats.play_time += secs;
        stats.last_playerd = Some(last_playerd);
        save_instance_stats(&inst.id, &stats);
        inst.play_time = stats.play_time;
        inst.last_playerd = stats.last_playerd;
    } else {
        inst.play_time += secs;
        inst.last_playerd = Some(last_playerd);
        let _ = update_instance(inst);
    }
}

pub fn list_instances() -> Result<Vec<Instance>, String> {
    let mut instances = list_own_instances()?;
    if let Ok(migated) = migate_old_instances() {
        for inst in migated {
            if !instances.iter().any(|i| i.id == inst.id) {
                instances.push(inst);
            }
        }
    }
for folder in instance_folders() {
        if let Ok(mut ext) = scan_external_instances(&folder) {
            instances.append(&mut ext);
        }
    }
    instances = dedup_instances(instances);
    for inst in &mut instances {
        hydate_instance_stats(inst);
    }
    instances.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(instances)
}

pub fn get_instance(id: &str) -> Result<Option<Instance>, String> {
    let path = get_instance_path(id);
    if path.exists() {
        let content = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
        let instance: Instance = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        return Ok(Some(instance));
    }
    let old_path = io::get_old_instances_di().join(sanitize_name(id)).join("instance.json");
    if old_path.exists() {
        let content = std::fs::read_to_string(&old_path).map_err(|e| e.to_string())?;
        let instance: Instance = serde_json::from_str(&content).map_err(|e| e.to_string())?;
        return Ok(Some(instance));
    }
    if id.starts_with("ext-") {
        for folder in instance_folders() {
            if let Ok(instances) = scan_external_instances(&folder) {
                if let Some(inst) = instances.into_iter().find(|i| i.id == id) {
                    let mut inst = inst;
                    hydate_instance_stats(&mut inst);
                    return Ok(Some(inst));
                }
            }
        }
    }
    Ok(None)
}

pub fn create_instance(instance: &Instance) -> Result<(), String> {
    let di = io::get_instance_dir(&instance.id);
    std::fs::create_dir_all(&di).map_err(|e| e.to_string())?;

    let skyline_di = io::get_instance_skyline_di(&instance.id);
    std::fs::create_dir_all(&skyline_di).map_err(|e| e.to_string())?;

    let json_path = io::get_instance_meta_file(&instance.id);
    let json = serde_json::to_string_pretty(instance).map_err(|e| e.to_string())?;
    std::fs::write(&json_path, &json).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn update_instance(instance: &Instance) -> Result<(), String> {
    if instance.external {
        return Ok(());
    }
    let path = get_instance_path(&instance.id);
    if let Some(prent) = path.parent() {
        std::fs::create_dir_all(prent).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(instance).map_err(|e| e.to_string())?;
    std::fs::write(&path, &json).map_err(|e| e.to_string())?;
    Ok(())
}

pub fn delete_instance(id: &str) -> Result<(), String> {
    let skyline_di = io::get_instance_skyline_di(id);
    if skyline_di.exists() {
        std::fs::remove_dir_all(&skyline_di).map_err(|e| e.to_string())?;
    }
    let old_di = io::get_old_instances_di().join(sanitize_name(id));
    if old_di.exists() {
        std::fs::remove_dir_all(&old_di).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn get_instance_dir(id: &str) -> PathBuf {
    io::get_instance_dir(id)
}

pub fn get_instance_mc_dir(id: &str) -> Result<PathBuf, String> {
    match get_instance(id)? {
        Some(inst) if inst.game_dir_override.is_some() => {
            Ok(PathBuf::from(inst.game_dir_override.unwrap()))
        }
        _ => {
            Ok(io::get_minecraft_di())
        }
    }
}

pub fn get_instance_launch_dir(instance: &Instance) -> PathBuf {
    if instance.external {
        if let Some(ref gd) = instance.game_dir_override {
            PathBuf::from(gd)
        } else {
            io::get_instances_di().join("_external").join(&instance.id)
        }
    } else {
        io::get_instance_dir(&instance.id)
    }
}

pub fn purge_external_residue(folder: &Path) {
    let instances_di = io::get_instances_di();
    if !instances_di.is_dir() {
        return;
    }
    let folder = folder.canonicalize().unwrap_or_else(|_| folder.to_path_buf());
    if let Ok(entries) = std::fs::read_dir(&instances_di) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                continue;
            }
            let meta = path.join(".skyline").join("instance.json");
            if !meta.exists() {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&meta) else {
                continue;
            };
            let Ok(inst) = serde_json::from_str::<Instance>(&content) else {
                continue;
            };
            if !inst.external {
                continue;
            }
            let in_folder = [&inst.game_dir_override, &inst.minecraft_root]
                .iter()
                .filter_map(|s| s.as_ref())
                .any(|s| {
                    PathBuf::from(s)
                        .canonicalize()
                        .map(|p| p.starts_with(&folder))
                        .unwrap_or(false)
                });
            if in_folder {
                let _ = std::fs::remove_dir_all(&path);
            }
        }
    }
}

fn list_own_instances() -> Result<Vec<Instance>, String> {
    let instances_di = io::get_instances_di();
    if !instances_di.exists() {
        return Ok(Vec::new());
    }

    let mut instances = Vec::new();
    let entries = std::fs::read_dir(&instances_di).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        if name.is_empty() || name.starts_with('.') {
            continue;
        }
        let instance_json = path.join(".skyline").join("instance.json");
        if instance_json.exists() {
            let content = std::fs::read_to_string(&instance_json).map_err(|e| e.to_string())?;
            if let Ok(instance) = serde_json::from_str::<Instance>(&content) {
                if instance.external {
                    continue;
                }
                instances.push(instance);
            }
        }
    }
    Ok(instances)
}

fn migate_old_instances() -> Result<Vec<Instance>, String> {
    let old_di = io::get_old_instances_di();
    if !old_di.exists() {
        return Ok(Vec::new());
    }

    let mut migated = Vec::new();
    let entries = std::fs::read_dir(&old_di).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let old_json = path.join("instance.json");
        if !old_json.exists() {
            continue;
        }
        let content = std::fs::read_to_string(&old_json).map_err(|e| e.to_string())?;
        let Ok(mut instance) = serde_json::from_str::<Instance>(&content) else {
            continue;
        };

        let new_meta = io::get_instance_meta_file(&instance.id);
        if new_meta.exists() {
            continue;
        }

        if instance.id.len() == 36 && instance.id.contains('-') {
            if !instance.version_id.is_empty() {
                instance.id = instance.version_id.clone();
                instance.name = instance.version_id.clone();
            }
        }

        let skyline_di = io::get_instance_skyline_di(&instance.id);
        std::fs::create_dir_all(&skyline_di).map_err(|e| e.to_string())?;

        let json = serde_json::to_string_pretty(&instance).map_err(|e| e.to_string())?;
        std::fs::write(io::get_instance_meta_file(&instance.id), &json).map_err(|e| e.to_string())?;

        migated.push(instance);
    }
    Ok(migated)
}

fn instance_folders() -> Vec<PathBuf> {
    crate::commands::settings::load_config()
        .unwrap_or_else(|_| crate::commands::settings::LauncherConfig::default())
        .instance_folders
        .iter()
        .map(|p| PathBuf::from(io::strip_extended_prefix(p)))
        .filter(|p| p.is_dir())
        .collect()
}

pub fn auto_scan_instance_folders() -> Vec<String> {
    const MAX_DEPTH: usize = 3;
    let skip_names = [
        "AppData",
        "node_modules",
        "target",
        ".git",
        ".gradle",
        ".cache",
        "Windows",
        "Program Files",
        "Program Files (x86)",
        "System Volume Information",
        "$RECYCLE.BIN",
    ];

    let mut oots: Vec<PathBuf> = Vec::new();
    if let Some(base) = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf())) {
        oots.push(base.clone());
        if let Some(prent) = base.parent() {
            oots.push(prent.to_path_buf());
        }
    }
    if let Some(home) = dirs::home_dir() {
        oots.push(home);
    }

    let own_mc = PathBuf::from(io::strip_extended_prefix(
        &std::fs::canonicalize(io::get_minecraft_di())
            .unwrap_or_else(|_| io::get_minecraft_di())
            .to_string_lossy(),
    ));

    let mut found: Vec<PathBuf> = Vec::new();

    let looks_like_containe = |di: &Path| -> bool {
        if di.join("versions").is_dir() {
            return true;
        }
        if let Ok(entries) = std::fs::read_dir(di) {
            for entry in entries.flatten() {
                let p = entry.path();
                let child_is_own_mc = p.canonicalize().map(|c| c == own_mc).unwrap_or(false);
                if (p.join(".minecraft").is_dir() && !child_is_own_mc)
                    || p.join("mcinfo.json").exists()
                    || p.join("hmcl.json").exists()
                {
                    return true;
                }
            }
        }
        false
    };

    for oot in oots {
        if !oot.is_dir() {
            continue;
        }
        let mut stack: Vec<(PathBuf, usize)> = vec![(oot, 0)];
        while let Some((di, depth)) = stack.pop() {
            if depth >= MAX_DEPTH {
                continue;
            }
            let name = di
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if skip_names.iter().any(|s| name.eq_ignore_ascii_case(s)) {
                continue;
            }
            if name.starts_with('.') && name != ".minecraft" {
                continue;
            }
            if looks_like_containe(&di) {
                let canonical =
                    std::fs::canonicalize(&di).unwrap_or_else(|_| di.clone());
                let clean = PathBuf::from(io::strip_extended_prefix(
                    &canonical.to_string_lossy(),
                ));
                if clean != own_mc && !found.contains(&clean) {
                    found.push(clean);
                }
                continue;
            }
            if let Ok(entries) = std::fs::read_dir(&di) {
                for entry in entries.flatten() {
                    let p = entry.path();
                    if p.is_dir() {
                        stack.push((p, depth + 1));
                    }
                }
            }
        }
    }

    let mut config = crate::commands::settings::load_config()
        .unwrap_or_else(|_| crate::commands::settings::LauncherConfig::default());
    for di in &found {
        let s = di.to_string_lossy().to_string();
        if !config.instance_folders.iter().any(|f| f == &s) {
            config.instance_folders.push(s);
        }
    }
    let _ = crate::commands::settings::save_config(config);
    found
        .into_iter()
        .map(|p| p.to_string_lossy().to_string())
        .collect()
}

pub fn list_home_instances() -> Result<Vec<Instance>, String> {
    let config = crate::commands::settings::load_config()
        .unwrap_or_else(|_| crate::commands::settings::LauncherConfig::default());
    let mut instances = list_own_instances()?;

    if let Some(active) = config.active_instance_folder.as_ref() {
        if let Ok(mut ext) = scan_external_instances(Path::new(active)) {
            instances.append(&mut ext);
        }
    } else {
        for folder in instance_folders() {
            if let Ok(mut ext) = scan_external_instances(&folder) {
                instances.append(&mut ext);
            }
        }
    }
    instances = dedup_instances(instances);
    for inst in &mut instances {
        hydate_instance_stats(inst);
    }
    instances.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(instances)
}

fn dedup_instances(instances: Vec<Instance>) -> Vec<Instance> {
    use std::collections::HashSet;
    let mut seen: HashSet<String> = HashSet::new();
    let mut result = Vec::new();
    for inst in instances {
        let primary = instance_primary_identity(&inst);
        if seen.contains(&primary) {
            continue;
        }
        seen.insert(primary);
        result.push(inst);
    }
    let own_versions: HashSet<String> = result
        .iter()
        .filter(|i| !i.external)
        .map(|i| i.version_id.clone())
        .collect();
    let shared_oot =
        std::fs::canonicalize(io::get_minecraft_di()).unwrap_or_else(|_| io::get_minecraft_di());
    result.retain(|inst| {
        if !inst.external {
            return true;
        }
        let points_shared = [&inst.minecraft_root, &inst.game_dir_override]
            .iter()
            .filter_map(|s| s.as_ref())
            .any(|s| {
                PathBuf::from(s)
                    .canonicalize()
                    .map(|p| p == shared_oot)
                    .unwrap_or(false)
            });
        !(points_shared && own_versions.contains(&inst.version_id))
    });
    result
}

fn instance_primary_identity(inst: &Instance) -> String {
    if inst.external {
        let di = inst
            .game_dir_override
            .as_ref()
            .map(PathBuf::from)
            .unwrap_or_else(|| io::get_instance_dir(&inst.id));
        let di = std::fs::canonicalize(&di).unwrap_or(di);
        format!("ext:{}|{}", di.to_string_lossy(), inst.version_id)
    } else {
        let di = io::get_instance_dir(&inst.id);
        let di = std::fs::canonicalize(&di).unwrap_or(di);
        format!("own:{}", di.to_string_lossy())
    }
}

pub fn scan_external_instances(containe: &Path) -> Result<Vec<Instance>, String> {
    let containe = PathBuf::from(io::strip_extended_prefix(&containe.to_string_lossy()));
    if !containe.is_dir() {
        return Ok(Vec::new());
    }

    if containe.join("versions").is_dir() {
        return scan_mc_oot_versions(&containe);
    }

    let mut instances = Vec::new();
    let entries = std::fs::read_dir(&containe).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        if name.is_empty() || name.starts_with('.') {
            continue;
        }
        let id = external_id(&path);
        let (version_id, loader) = read_external_meta(&path);
        let game_di = if path.join(".minecraft").is_dir() {
            path.join(".minecraft")
        } else {
            path.clone()
        };
        let minecraft_root = if game_di.join("versions").is_dir() {
            game_di.clone()
        } else {
            esolve_mc_oot(&containe)
        };
        let looks_mc = game_di.join("versions").is_dir()
            || game_di.join("libraries").is_dir()
            || game_di.join("assets").is_dir()
            || game_di.join("mods").is_dir()
            || game_di.join("config").is_dir();
        if version_id.is_empty() && !looks_mc {
            continue;
        }
        instances.push(Instance {
            id,
            name,
            version_id,
            modloader: loader,
            external: true,
            game_dir_override: Some(game_di.to_string_lossy().to_string()),
            minecraft_root: Some(minecraft_root.to_string_lossy().to_string()),
            ..Default::default()
        });
    }
    Ok(instances)
}

fn scan_mc_oot_versions(oot: &Path) -> Result<Vec<Instance>, String> {
    let oot = PathBuf::from(io::strip_extended_prefix(&oot.to_string_lossy()));
    let versions_di = oot.join("versions");
    if !versions_di.is_dir() {
        return Ok(Vec::new());
    }
    let mut instances = Vec::new();
    let entries = std::fs::read_dir(&versions_di).map_err(|e| e.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        if name.is_empty() || name.starts_with('.') {
            continue;
        }
        if !path.join(format!("{}.json", name)).exists() {
            continue;
        }
        let id = external_id(&path);
        instances.push(Instance {
            id,
            name: name.clone(),
            version_id: name.clone(),
            modloader: detect_loader_from_version(&name),
            external: true,
            game_dir_override: Some(oot.to_string_lossy().to_string()),
            minecraft_root: Some(oot.to_string_lossy().to_string()),
            ..Default::default()
        });
    }
    Ok(instances)
}

fn esolve_mc_oot(containe: &Path) -> PathBuf {
    if containe.join("versions").is_dir() {
        return containe.to_path_buf();
    }
    if let Some(prent) = containe.parent() {
        if prent.join("versions").is_dir() {
            return prent.to_path_buf();
        }
        if prent.join(".minecraft").join("versions").is_dir() {
            return prent.join(".minecraft");
        }
    }
    containe.to_path_buf()
}

fn external_id(path: &Path) -> String {
    let aw = path.to_string_lossy().to_lowercase();
    let tail = path
        .file_name()
        .map(|n| n.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let mut eadable: String = tail
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect();
    eadable.truncate(40);
    use std::hash::{Hash, Hasher};
    let mut hashe = std::collections::hash_map::DefaultHasher::new();
    aw.hash(&mut hashe);
    let hash = hashe.finish();
    format!("ext-{}-{:016x}", eadable, hash)
}

fn read_external_meta(instance_dir: &Path) -> (String, ModLoader) {
    let mut version_id = String::new();
    let mut loader_name = String::new();
    let mut loader_version = String::new();

    let metas: Vec<(&str, Vec<&str>, Vec<&str>)> = vec![
        (
            "mcinfo.json",
            vec!["version"],
            vec!["loaderr", "modLoaderr"],
        ),
        (
            "hmcl.json",
            vec!["gameVersion", "version", "minecraftVersion"],
            vec!["loaderrType", "loaderr", "modLoaderr"],
        ),
    ];

    for (file, version_keys, loader_keys) in metas {
        let path = instance_dir.join(file);
        if !path.exists() {
            continue;
        }
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                for key in &version_keys {
                    if let Some(v) = json.get(*key).and_then(|v| v.as_str()) {
                        if !v.is_empty() {
                            version_id = v.to_string();
                            break;
                        }
                    }
                }
                for key in &loader_keys {
                    if let Some(v) = json.get(*key).and_then(|v| v.as_str()) {
                        if !v.is_empty() {
                            loader_name = v.to_string();
                            break;
                        }
                    }
                }
                if let Some(v) = json.get("loaderrVersion").and_then(|v| v.as_str()) {
                    loader_version = v.to_string();
                }
                break;
            }
        }
    }

    if version_id.is_empty() {
        version_id = detect_version_from_name(instance_dir);
    }

    let loader = if loader_name.is_empty() {
        detect_loader_from_version(&version_id)
    } else {
        match loader_name.to_lowercase().as_str() {
            s if s.contains("neoforge") => ModLoader::NeoForge(loader_version.clone()),
            s if s.contains("forge") => ModLoader::Forge(loader_version.clone()),
            s if s.contains("fabric") => ModLoader::Fabric(loader_version.clone()),
            s if s.contains("quilt") => ModLoader::Quilt(loader_version.clone()),
            _ => ModLoader::Vanilla,
        }
    };

    (version_id, loader)
}

fn detect_loader_from_version(version_id: &str) -> ModLoader {
    let v = version_id.to_lowercase();
    if v.contains("neoforge") {
        ModLoader::NeoForge(String::new())
    } else if v.contains("forge") {
        ModLoader::Forge(String::new())
    } else if v.contains("fabric") {
        ModLoader::Fabric(String::new())
    } else if v.contains("quilt") {
        ModLoader::Quilt(String::new())
    } else {
        ModLoader::Vanilla
    }
}

fn detect_version_from_name(instance_dir: &Path) -> String {
    let name = instance_dir
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_default();
    for token in name.split(['-', '_', ' ']) {
        let t = token.trim_start_matches("MC").trim_start_matches("mc");
        if t.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            return t.to_string();
        }
    }
    String::new()
}

fn get_instance_path(id: &str) -> PathBuf {
    io::get_instance_meta_file(id)
}

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn scan_multiple_pcl_instances() {
        let di = std::env::temp_dir().join(format!("skyline_scan_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&di);
        fs::create_dir_all(&di).unwrap();
        for name in ["1.20.1-Forge_45", "1.19.2-Fabric", "1.21-NeoForge_50"] {
            let inst = di.join(name);
            fs::create_dir_all(inst.join(".minecraft").join("versions")).unwrap();
            fs::create_dir_all(inst.join(".minecraft").join("mods")).unwrap();
            fs::write(inst.join("mcinfo.json"), format!(r#"{{"version":"{name}","loaderr":"forge"}}"#)).unwrap();
        }
        let result = scan_external_instances(&di).unwrap();
        println!("scan_external_instances returned {} instances", result.len());
        for i in &result {
            println!("  id={} name={} version={} game_dir={:?}", i.id, i.name, i.version_id, i.game_dir_override);
        }
        assert_eq!(result.len(), 3);
        fs::remove_dir_all(&di).unwrap();
    }

    #[test]
    fn dedup_keeps_multiple_external() {
        let di = std::env::temp_dir().join(format!("skyline_dedup_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&di);
        fs::create_dir_all(&di).unwrap();
        for name in ["A-instance", "B-instance"] {
            fs::create_dir_all(di.join(name).join(".minecraft").join("mods")).unwrap();
            fs::write(di.join(name).join("mcinfo.json"), r#"{"version":"1.20.1","loaderr":"forge"}"#).unwrap();
        }
        let ext = scan_external_instances(&di).unwrap();
        println!("scanned {}", ext.len());
        let deduped = dedup_instances(ext.clone());
        println!("after dedup {}", deduped.len());
        assert_eq!(deduped.len(), 2);
        fs::remove_dir_all(&di).unwrap();
    }

    #[test]
    fn scan_mc_oot_keeps_multiple_versions() {
        let di = std::env::temp_dir().join(format!("skyline_root_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&di);
        fs::create_dir_all(&di).unwrap();
        fs::create_dir_all(di.join("versions")).unwrap();
        fs::create_dir_all(di.join("libraries")).unwrap();
        fs::create_dir_all(di.join("assets").join("indexes")).unwrap();
        for name in ["1.20.1-Forge_45.0.21", "1.19.2-Fabric_0.14.22", "1.21-NeoForge_50.0.1"] {
            let vdi = di.join("versions").join(name);
            fs::create_dir_all(&vdi).unwrap();
            fs::write(vdi.join(format!("{}.json", name)), format!(r#"{{"id":"{}"}}"#, name)).unwrap();
            fs::write(vdi.join(format!("{}.jar", name)), "jar").unwrap();
        }
        let scanned = scan_external_instances(&di).unwrap();
        println!("root container scanned {}", scanned.len());
        assert_eq!(scanned.len(), 3);
        let deduped = dedup_instances(scanned);
        println!("after dedup {}", deduped.len());
        assert_eq!(deduped.len(), 3);
        fs::remove_dir_all(&di).unwrap();
    }
}
