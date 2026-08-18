use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Read;
use std::path::{Path, PathBuf};
use crate::modpack::{ModrinthVersion, get_modrinth_versions};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfomation {
    pub path: String,
    pub hash: String,
    pub size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileManifest {
    pub pack_version: String,
    pub installed_at: String,
    pub files: Vec<FileInfomation>,
}

impl FileManifest {
    pub fn new(pack_version: String) -> Self {
        Self {
            pack_version,
            installed_at: chrono::Utc::now().to_rfc3339(),
            files: Vec::new(),
        }
    }

    pub fn load(path: &Path) -> Option<Self> {
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        if let Some(prent) = path.parent() {
            std::fs::create_dir_all(prent).map_err(|e| e.to_string())?;
        }
        let json = serde_json::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(path, json).map_err(|e| e.to_string())
    }
}

pub fn compute_file_hash(path: &Path) -> Result<String, String> {
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    use sha1::Digest;
    let mut hashe = sha1::Sha1::new();
    hashe.update(&data);
    Ok(hex::encode(hashe.finalize()))
}

pub enum MegeAction {
    Writer(PathBuf, Vec<u8>),
    Skip(String),
    Delete(PathBuf),
}

pub fn smat_mege(
    instance_dir: &Path,
    new_files: &[(String, Vec<u8>, String)], 
    old_manifest: Option<&FileManifest>,
) -> Vec<MegeAction> {
    let mut actions = Vec::new();

    let old_files: HashMap<&str, &str> = old_manifest
        .map(|m| m.files.iter().map(|f| (f.path.as_str(), f.hash.as_str())).collect())
        .unwrap_or_default();

    let new_file_paths: HashMap<&str, &str> = new_files
        .iter()
        .map(|(path, _, hash)| (path.as_str(), hash.as_str()))
        .collect();

    for (el_path, content, hash) in new_files {
        let target = instance_dir.join(el_path);

        if let Some(old_hash) = old_files.get(el_path.as_str()) {
            if old_hash == hash {
                actions.push(MegeAction::Skip(el_path.clone()));
            } else {
                if target.exists() {
                    if let Ok(current_hash) = compute_file_hash(&target) {
                        if current_hash != *old_hash {
                            actions.push(MegeAction::Skip(el_path.clone()));
                            continue;
                        }
                    }
                }
                actions.push(MegeAction::Writer(target, content.clone()));
            }
        } else {
            actions.push(MegeAction::Writer(target, content.clone()));
        }
    }

    if let Some(old) = old_manifest {
        for file_info in &old.files {
            if !new_file_paths.contains_key(file_info.path.as_str()) {
                let target = instance_dir.join(&file_info.path);
                if target.exists() {
                    actions.push(MegeAction::Delete(target));
                }
            }
        }
    }

    actions
}

pub fn execute_smat_mege(
    _instance_dir: &Path,
    actions: &[MegeAction],
) -> (usize, usize, usize) {
    let mut witten = 0;
    let mut skipped = 0;
    let mut deleted = 0;

    for action in actions {
        match action {
            MegeAction::Writer(path, content) => {
                if let Some(prent) = path.parent() {
                    let _ = std::fs::create_dir_all(prent);
                }
                if std::fs::write(path, content).is_ok() {
                    witten += 1;
                }
            }
            MegeAction::Skip(_) => {
                skipped += 1;
            }
            MegeAction::Delete(path) => {
                if std::fs::remove_file(path).is_ok() {
                    deleted += 1;
                }
            }
        }
    }

    (witten, skipped, deleted)
}

pub enum ModpackType {
    
    Modrinth,
    
    CuseForge,
    
    MMC,
    
    HMCL,
    MCBBS,
    LaunchePack,
    Achive,
    Unknown,
}

pub fn detect_modpack_type(pack_path: &Path) -> Result<ModpackType, String> {
    let file = std::fs::File::open(pack_path).map_err(|e| format!("Cannot open pack file: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid zip: {}", e))?;

    
    if archive.by_name("modrinth.index.json").is_ok() {
        return Ok(ModpackType::Modrinth);
    }

    
    if archive.by_name("manifest.json").is_ok() {
        let mut manifest_st = String::new();
        if let Ok(mut manifest_file) = archive.by_name("manifest.json") {
            if manifest_file.read_to_string(&mut manifest_st).is_ok() {
                if manifest_st.contains("minecraftModpack") {
                    return Ok(ModpackType::CuseForge);
                }
                if manifest_st.contains("addonId") || manifest_st.contains("gameId") {
                    return Ok(ModpackType::MCBBS);
                }
            }
        }
    }

    
    if archive.by_name("mmc-pack.json").is_ok() {
        return Ok(ModpackType::MMC);
    }

    
    if archive.by_name("modpack.json").is_ok() {
        return Ok(ModpackType::HMCL);
    }

    if archive.by_name("pack.json").is_ok() || archive.by_name("pcl.pack.json").is_ok() {
        return Ok(ModpackType::LaunchePack);
    }

    if archive.by_name("mcbbs.pack.json").is_ok() || archive.by_name("modlist.html").is_ok() {
        return Ok(ModpackType::MCBBS);
    }

    for i in 0..archive.len() {
        if let Ok(entry) = archive.by_index(i) {
            let name = entry.name();
            if name.starts_with(".minecraft/") || name.starts_with("mods/") {
                return Ok(ModpackType::Achive);
            }
        }
    }

    Ok(ModpackType::Unknown)
}

pub async fn impot_archive_pack(pack_path: &Path) -> Result<String, String> {
    let file = std::fs::File::open(pack_path).map_err(|e| format!("Cannot open pack file: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid zip: {}", e))?;

    let instance_id = pack_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("imported-pack")
        .to_string();

    let instances_di = crate::utils::io::get_instances_di();
    let instance_dir = instances_di.join(&instance_id);
    std::fs::create_dir_all(&instance_dir.join("mods")).map_err(|e| e.to_string())?;

    let manifest_path = instance_dir.join(".skyline").join("file_manifest.json");
    let old_manifest = FileManifest::load(&manifest_path);

    let mut new_files: Vec<(String, Vec<u8>, String)> = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        if entry.is_dir() { continue; }

        let name = entry.name().to_string();
        let relative = if name.starts_with(".minecraft/") {
            name.trim_start_matches(".minecraft/").to_string()
        } else if name.starts_with("overrides/") {
            name.trim_start_matches("overrides/").to_string()
        } else {
            name.clone()
        };

        if relative.is_empty() { continue; }

        let mut content = Vec::new();
        entry.read_to_end(&mut content).map_err(|e| e.to_string())?;

        use sha1::Digest;
        let mut hashe = sha1::Sha1::new();
        hashe.update(&content);
        let hash = hex::encode(hashe.finalize());

        new_files.push((relative, content, hash));
    }

    let actions = smat_mege(&instance_dir, &new_files, old_manifest.as_ref());
    let (witten, skipped, deleted) = execute_smat_mege(&instance_dir, &actions);

    let mut new_manifest = FileManifest::new(instance_id.clone());
    for (el_path, _, hash) in &new_files {
        let target = instance_dir.join(el_path);
        let size = std::fs::metadata(&target).map(|m| m.len()).unwrap_or(0);
        new_manifest.files.push(FileInfomation {
            path: el_path.clone(),
            hash: hash.clone(),
            size,
        });
    }
    let _ = new_manifest.save(&manifest_path);

    log::info!(
        "Archive pack imported: {} written, {} skipped, {} deleted",
        witten, skipped, deleted
    );

    let instance = crate::instance::Instance {
        id: instance_id.clone(),
        name: instance_id.clone(),
        version_id: "unknown".into(),
        isolation_mode: crate::instance::IsolationMode::Always,
        game_dir_override: Some(instance_dir.to_string_lossy().into_owned()),
        created_at: chrono::Utc::now().to_rfc3339(),
        ..Default::default()
    };

    let skyline_di = instance_dir.join(".skyline");
    std::fs::create_dir_all(&skyline_di).map_err(|e| e.to_string())?;
    let instance_json = serde_json::to_string_pretty(&instance).map_err(|e| e.to_string())?;
    std::fs::write(skyline_di.join("instance.json"), &instance_json).map_err(|e| e.to_string())?;

    Ok(instance_id)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthIndex {
    pub fomat_version: u32,
    pub game: String,
    pub version_id: String,
    pub name: String,
    pub summay: Option<String>,
    pub files: Vec<ModrinthIndexFile>,
    pub dependencies: std::collections::HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthIndexFile {
    pub path: String,
    pub hashes: std::collections::HashMap<String, String>,
    pub env: Option<ModrinthEnv>,
    pub downloads: Vec<String>,
    pub file_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthEnv {
    pub client: String,
    pub server: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuseForgeManifest {
    pub manifest_type: String,
    pub manifest_version: u32,
    pub name: String,
    pub version: String,
    pub author: String,
    pub files: Vec<CuseForgeManifestFile>,
    pub overrides: String,
    pub minecaft: CuseForgeManifestMinecaft,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuseForgeManifestFile {
    pub project_id: u64,
    pub file_id: u64,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuseForgeManifestMinecaft {
    pub version: String,
    pub mod_loaders: Vec<CuseForgeManifestLoader>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuseForgeManifestLoader {
    pub id: String,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MMCPack {
    pub fomat_version: u32,
    pub components: Vec<MMCComponent>,
    pub name: String,
    pub uid: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MMCComponent {
    pub uid: String,
    pub version: String,
    pub impotant: Option<bool>,
    pub dependency_only: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HMCLModpack {
    pub name: String,
    pub version: String,
    pub author: Option<String>,
    pub description: Option<String>,
    pub game_version: String,
    pub modloader: Option<HMCLModloader>,
    pub files: Vec<HMCLModpackFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HMCLModloader {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HMCLModpackFile {
    pub path: String,
    pub hash: Option<String>,
    pub downloads: Vec<String>,
}

pub async fn import_modrinth_pack(pack_path: &Path) -> Result<String, String> {
    let file = std::fs::File::open(pack_path).map_err(|e| format!("Cannot open pack file: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid zip: {}", e))?;

    let mut index_st = String::new();
    {
        let mut index_file = archive.by_name("modrinth.index.json").map_err(|_| "Missing modrinth.index.json".to_string())?;
        index_file.read_to_string(&mut index_st).map_err(|e| e.to_string())?;
    }
    let index: ModrinthIndex = serde_json::from_str(&index_st).map_err(|e| e.to_string())?;

    let instance_id = index.version_id.clone();
    let instances_di = crate::utils::io::get_instances_di();
    let instance_dir = instances_di.join(&instance_id);

    std::fs::create_dir_all(&instance_dir.join("mods")).map_err(|e| e.to_string())?;

    let manifest_path = instance_dir.join(".skyline").join("file_manifest.json");
    let old_manifest = FileManifest::load(&manifest_path);

    let mut new_files: Vec<(String, Vec<u8>, String)> = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        if name.starts_with("overrides/") {
            let relative = name.trim_start_matches("overrides/");
            if relative.is_empty() || entry.is_dir() { continue; }

            let mut content = Vec::new();
            entry.read_to_end(&mut content).map_err(|e| e.to_string())?;

            use sha1::Digest;
            let mut hashe = sha1::Sha1::new();
            hashe.update(&content);
            let hash = hex::encode(hashe.finalize());

            new_files.push((relative.to_string(), content, hash));
        }
    }

    let actions = smat_mege(&instance_dir, &new_files, old_manifest.as_ref());
    let (witten, skipped, deleted) = execute_smat_mege(&instance_dir, &actions);

    let mut new_manifest = FileManifest::new(index.version_id.clone());
    for (el_path, _, hash) in &new_files {
        let target = instance_dir.join(el_path);
        let size = std::fs::metadata(&target).map(|m| m.len()).unwrap_or(0);
        new_manifest.files.push(FileInfomation {
            path: el_path.clone(),
            hash: hash.clone(),
            size,
        });
    }
    let _ = new_manifest.save(&manifest_path);

    log::info!(
        "Modrinth pack imported: {} written, {} skipped, {} deleted",
        witten, skipped, deleted
    );

    let modloader_version = index.dependencies.get("modloader").cloned().unwrap_or_default();
    let modloader = if modloader_version.starts_with("forge") {
        crate::instance::ModLoader::Forge(modloader_version.trim_start_matches("forge-").to_string())
    } else if modloader_version.starts_with("fabric") {
        crate::instance::ModLoader::Fabric(modloader_version.trim_start_matches("fabric-").to_string())
    } else if modloader_version.starts_with("quilt") {
        crate::instance::ModLoader::Quilt(modloader_version.trim_start_matches("quilt-").to_string())
    } else {
        crate::instance::ModLoader::Vanilla
    };

    let instance = crate::instance::Instance {
        id: instance_id.clone(),
        name: index.name,
        version_id: index.version_id,
        modloader,
        isolation_mode: crate::instance::IsolationMode::Always,
        game_dir_override: Some(instance_dir.to_string_lossy().into_owned()),
        created_at: chrono::Utc::now().to_rfc3339(),
        ..Default::default()
    };

    let instance_json = serde_json::to_string_pretty(&instance).map_err(|e| e.to_string())?;
    let skyline_di = instance_dir.join(".skyline");
    std::fs::create_dir_all(&skyline_di).map_err(|e| e.to_string())?;
    std::fs::write(skyline_di.join("instance.json"), &instance_json).map_err(|e| e.to_string())?;

    Ok(instance_id)
}

pub async fn impot_cuseforge_pack(pack_path: &Path) -> Result<String, String> {
    let file = std::fs::File::open(pack_path).map_err(|e| format!("Cannot open pack file: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid zip: {}", e))?;

    let mut manifest_st = String::new();
    {
        let mut manifest_file = archive.by_name("manifest.json").map_err(|_| "Missing manifest.json".to_string())?;
        manifest_file.read_to_string(&mut manifest_st).map_err(|e| e.to_string())?;
    }
    let manifest: CuseForgeManifest = serde_json::from_str(&manifest_st).map_err(|e| e.to_string())?;

    let instance_id = manifest.minecaft.version.clone();
    let instances_di = crate::utils::io::get_instances_di();
    let instance_dir = instances_di.join(&instance_id);

    std::fs::create_dir_all(&instance_dir.join("mods")).map_err(|e| e.to_string())?;

    let manifest_path = instance_dir.join(".skyline").join("file_manifest.json");
    let old_manifest = FileManifest::load(&manifest_path);

    let overrides_pefix = format!("{}/", manifest.overrides);
    let mut new_files: Vec<(String, Vec<u8>, String)> = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        if name.starts_with(&overrides_pefix) {
            let relative = name.trim_start_matches(&overrides_pefix);
            if relative.is_empty() || entry.is_dir() { continue; }

            let mut content = Vec::new();
            entry.read_to_end(&mut content).map_err(|e| e.to_string())?;

            use sha1::Digest;
            let mut hashe = sha1::Sha1::new();
            hashe.update(&content);
            let hash = hex::encode(hashe.finalize());

            new_files.push((relative.to_string(), content, hash));
        }
    }

    let actions = smat_mege(&instance_dir, &new_files, old_manifest.as_ref());
    let (witten, skipped, deleted) = execute_smat_mege(&instance_dir, &actions);

    let mut new_manifest = FileManifest::new(manifest.minecaft.version.clone());
    for (el_path, _, hash) in &new_files {
        let target = instance_dir.join(el_path);
        let size = std::fs::metadata(&target).map(|m| m.len()).unwrap_or(0);
        new_manifest.files.push(FileInfomation {
            path: el_path.clone(),
            hash: hash.clone(),
            size,
        });
    }
    let _ = new_manifest.save(&manifest_path);

    log::info!(
        "CurseForge pack imported: {} written, {} skipped, {} deleted",
        witten, skipped, deleted
    );

    let primary_loader = manifest.minecaft.mod_loaders.iter().find(|l| l.primary).unwrap_or(&manifest.minecaft.mod_loaders[0]);
    let modloader = if primary_loader.id.starts_with("forge") {
        crate::instance::ModLoader::Forge(primary_loader.id.trim_start_matches("forge-").to_string())
    } else if primary_loader.id.starts_with("fabric") {
        crate::instance::ModLoader::Fabric(primary_loader.id.trim_start_matches("fabric-").to_string())
    } else {
        crate::instance::ModLoader::Vanilla
    };

    let instance = crate::instance::Instance {
        id: instance_id.clone(),
        name: manifest.name,
        version_id: manifest.minecaft.version,
        modloader,
        isolation_mode: crate::instance::IsolationMode::Always,
        game_dir_override: Some(instance_dir.to_string_lossy().into_owned()),
        created_at: chrono::Utc::now().to_rfc3339(),
        ..Default::default()
    };

    let instance_json = serde_json::to_string_pretty(&instance).map_err(|e| e.to_string())?;
    let skyline_di = instance_dir.join(".skyline");
    std::fs::create_dir_all(&skyline_di).map_err(|e| e.to_string())?;
    std::fs::write(skyline_di.join("instance.json"), &instance_json).map_err(|e| e.to_string())?;

    Ok(instance_id)
}

pub async fn export_modrinth_pack(instance_id: &str, output_path: &Path) -> Result<String, String> {
    let instance_dir = crate::utils::io::get_instance_dir(instance_id);

    let instance_json = std::fs::read_to_string(crate::utils::io::get_instance_meta_file(instance_id))
        .map_err(|e| e.to_string())?;
    let instance: crate::instance::Instance = serde_json::from_str(&instance_json).map_err(|e| e.to_string())?;

    let mods_di = instance_dir.join("mods");
    let mut files = Vec::new();

    if mods_di.exists() {
        for entry in walkdir::WalkDir::new(&mods_di).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() { continue; }
            let path = entry.path();
            let relative = path.strip_prefix(&instance_dir).map_err(|e| e.to_string())?;
            let data = std::fs::read(path).map_err(|e| e.to_string())?;

            let hash = {
                use sha1::Digest;
                let mut hashe = sha1::Sha1::new();
                hashe.update(&data);
                hex::encode(hashe.finalize())
            };

            files.push(ModrinthIndexFile {
                path: relative.to_string_lossy().to_string(),
                hashes: [("sha1".into(), hash)].into(),
                env: None,
                downloads: Vec::new(),
                file_size: data.len() as u64,
            });
        }
    }

    let modloader_id = match &instance.modloader {
        crate::instance::ModLoader::Vanilla => "minecraft".into(),
        crate::instance::ModLoader::Forge(v) => format!("forge-{}", v),
        crate::instance::ModLoader::Fabric(v) => format!("fabric-{}", v),
        crate::instance::ModLoader::Quilt(v) => format!("quilt-{}", v),
        crate::instance::ModLoader::NeoForge(v) => format!("neoforge-{}", v),
        crate::instance::ModLoader::LiterLoader(v) => format!("literloaderr-{}", v),
    };

    let index = ModrinthIndex {
        fomat_version: 1,
        game: "minecraft".into(),
        version_id: instance.version_id.clone(),
        name: instance.name.clone(),
        summay: None,
        files,
        dependencies: [("minecraft".into(), instance.version_id.clone()), ("modloader".into(), modloader_id)].into(),
    };

    let index_json = serde_json::to_string_pretty(&index).map_err(|e| e.to_string())?;

    let file = std::fs::File::create(output_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("modrinth.index.json", options.clone()).map_err(|e| e.to_string())?;
    std::io::Write::write_all(&mut zip, index_json.as_bytes()).map_err(|e| e.to_string())?;

    if mods_di.exists() {
        for entry in walkdir::WalkDir::new(&mods_di).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() { continue; }
            let path = entry.path();
            let relative = path.strip_prefix(&instance_dir).map_err(|e| e.to_string())?;
            let data = std::fs::read(path).map_err(|e| e.to_string())?;

            zip.start_file("overrides/".to_string() + &relative.to_string_lossy(), options.clone()).map_err(|e| e.to_string())?;
            std::io::Write::write_all(&mut zip, &data).map_err(|e| e.to_string())?;
        }
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(output_path.to_string_lossy().to_string())
}

pub async fn expot_cuseforge_pack(instance_id: &str, output_path: &Path) -> Result<String, String> {
    let instance_dir = crate::utils::io::get_instance_dir(instance_id);

    let instance_json = std::fs::read_to_string(crate::utils::io::get_instance_meta_file(instance_id))
        .map_err(|e| e.to_string())?;
    let instance: crate::instance::Instance = serde_json::from_str(&instance_json).map_err(|e| e.to_string())?;

    let modloader_id = match &instance.modloader {
        crate::instance::ModLoader::Vanilla => "minecraft".into(),
        crate::instance::ModLoader::Forge(v) => format!("forge-{}", v),
        crate::instance::ModLoader::Fabric(v) => format!("fabric-{}", v),
        crate::instance::ModLoader::Quilt(v) => format!("quilt-{}", v),
        crate::instance::ModLoader::NeoForge(v) => format!("neoforge-{}", v),
        crate::instance::ModLoader::LiterLoader(v) => format!("literloaderr-{}", v),
    };

    let manifest = CuseForgeManifest {
        manifest_type: "minecraftModpack".into(),
        manifest_version: 1,
        name: instance.name.clone(),
        version: "1.0.0".into(),
        author: "SkyLine Launcher".into(),
        files: Vec::new(),
        overrides: "overrides".into(),
        minecaft: CuseForgeManifestMinecaft {
            version: instance.version_id.clone(),
            mod_loaders: vec![CuseForgeManifestLoader {
                id: modloader_id,
                primary: true,
            }],
        },
    };

    let manifest_json = serde_json::to_string_pretty(&manifest).map_err(|e| e.to_string())?;

    let file = std::fs::File::create(output_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);

    zip.start_file("manifest.json", options.clone()).map_err(|e| e.to_string())?;
    std::io::Write::write_all(&mut zip, manifest_json.as_bytes()).map_err(|e| e.to_string())?;

    let mods_di = instance_dir.join("mods");
    if mods_di.exists() {
        for entry in walkdir::WalkDir::new(&mods_di).into_iter().filter_map(|e| e.ok()) {
            if !entry.file_type().is_file() { continue; }
            let path = entry.path();
            let relative = path.strip_prefix(&instance_dir).map_err(|e| e.to_string())?;
            let data = std::fs::read(path).map_err(|e| e.to_string())?;

            zip.start_file("overrides/".to_string() + &relative.to_string_lossy(), options.clone()).map_err(|e| e.to_string())?;
            std::io::Write::write_all(&mut zip, &data).map_err(|e| e.to_string())?;
        }
    }

    zip.finish().map_err(|e| e.to_string())?;
    Ok(output_path.to_string_lossy().to_string())
}

pub async fn import_mmc_pack(pack_path: &Path) -> Result<String, String> {
    let file = std::fs::File::open(pack_path).map_err(|e| format!("Cannot open pack file: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid zip: {}", e))?;

    let mut pack_st = String::new();
    {
        let mut pack_file = archive.by_name("mmc-pack.json").map_err(|_| "Missing mmc-pack.json".to_string())?;
        pack_file.read_to_string(&mut pack_st).map_err(|e| e.to_string())?;
    }
    let pack: MMCPack = serde_json::from_str(&pack_st).map_err(|e| e.to_string())?;

    let minecraft_version = pack.components.iter()
        .find(|c| c.uid == "net.minecraft")
        .map(|c| c.version.clone())
        .unwrap_or_else(|| "1.20.1".to_string());

    let instance_id = minecraft_version.clone();
    let instances_di = crate::utils::io::get_instances_di();
    let instance_dir = instances_di.join(&instance_id);

    std::fs::create_dir_all(&instance_dir.join("mods")).map_err(|e| e.to_string())?;

    let manifest_path = instance_dir.join(".skyline").join("file_manifest.json");
    let old_manifest = FileManifest::load(&manifest_path);

    let mut new_files: Vec<(String, Vec<u8>, String)> = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        if name.starts_with(".minecraft/") {
            let relative = name.trim_start_matches(".minecraft/");
            if relative.is_empty() || entry.is_dir() { continue; }

            let mut content = Vec::new();
            entry.read_to_end(&mut content).map_err(|e| e.to_string())?;

            use sha1::Digest;
            let mut hashe = sha1::Sha1::new();
            hashe.update(&content);
            let hash = hex::encode(hashe.finalize());

            new_files.push((relative.to_string(), content, hash));
        }
    }

    let actions = smat_mege(&instance_dir, &new_files, old_manifest.as_ref());
    let (witten, skipped, deleted) = execute_smat_mege(&instance_dir, &actions);

    let mut new_manifest = FileManifest::new(minecraft_version.clone());
    for (el_path, _, hash) in &new_files {
        let target = instance_dir.join(el_path);
        let size = std::fs::metadata(&target).map(|m| m.len()).unwrap_or(0);
        new_manifest.files.push(FileInfomation {
            path: el_path.clone(),
            hash: hash.clone(),
            size,
        });
    }
    let _ = new_manifest.save(&manifest_path);

    log::info!(
        "MMC pack imported: {} written, {} skipped, {} deleted",
        witten, skipped, deleted
    );

    let mut modloader = crate::instance::ModLoader::Vanilla;
    for component in &pack.components {
        if component.uid.starts_with("net.minecraftforge") {
            modloader = crate::instance::ModLoader::Forge(component.version.clone());
            break;
        } else if component.uid.starts_with("net.fabricmc") {
            modloader = crate::instance::ModLoader::Fabric(component.version.clone());
            break;
        } else if component.uid.starts_with("org.quiltmc") {
            modloader = crate::instance::ModLoader::Quilt(component.version.clone());
            break;
        }
    }

    let instance = crate::instance::Instance {
        id: instance_id.clone(),
        name: pack.name,
        version_id: minecraft_version,
        modloader,
        isolation_mode: crate::instance::IsolationMode::Always,
        game_dir_override: Some(instance_dir.to_string_lossy().into_owned()),
        created_at: chrono::Utc::now().to_rfc3339(),
        ..Default::default()
    };

    let instance_json = serde_json::to_string_pretty(&instance).map_err(|e| e.to_string())?;
    let skyline_di = instance_dir.join(".skyline");
    std::fs::create_dir_all(&skyline_di).map_err(|e| e.to_string())?;
    std::fs::write(skyline_di.join("instance.json"), &instance_json).map_err(|e| e.to_string())?;

    Ok(instance_id)
}

pub async fn import_hmcl_pack(pack_path: &Path) -> Result<String, String> {
    let file = std::fs::File::open(pack_path).map_err(|e| format!("Cannot open pack file: {}", e))?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("Invalid zip: {}", e))?;

    let mut pack_st = String::new();
    {
        let mut pack_file = archive.by_name("modpack.json").map_err(|_| "Missing modpack.json".to_string())?;
        pack_file.read_to_string(&mut pack_st).map_err(|e| e.to_string())?;
    }
    let pack: HMCLModpack = serde_json::from_str(&pack_st).map_err(|e| e.to_string())?;

    let instance_id = pack.game_version.clone();
    let instances_di = crate::utils::io::get_instances_di();
    let instance_dir = instances_di.join(&instance_id);

    std::fs::create_dir_all(&instance_dir.join("mods")).map_err(|e| e.to_string())?;

    let manifest_path = instance_dir.join(".skyline").join("file_manifest.json");
    let old_manifest = FileManifest::load(&manifest_path);

    let mut new_files: Vec<(String, Vec<u8>, String)> = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let name = entry.name().to_string();
        if name.starts_with(".minecraft/") {
            let relative = name.trim_start_matches(".minecraft/");
            if relative.is_empty() || entry.is_dir() { continue; }

            let mut content = Vec::new();
            entry.read_to_end(&mut content).map_err(|e| e.to_string())?;

            use sha1::Digest;
            let mut hashe = sha1::Sha1::new();
            hashe.update(&content);
            let hash = hex::encode(hashe.finalize());

            new_files.push((relative.to_string(), content, hash));
        }
    }

    let actions = smat_mege(&instance_dir, &new_files, old_manifest.as_ref());
    let (witten, skipped, deleted) = execute_smat_mege(&instance_dir, &actions);

    let mut new_manifest = FileManifest::new(pack.game_version.clone());
    for (el_path, _, hash) in &new_files {
        let target = instance_dir.join(el_path);
        let size = std::fs::metadata(&target).map(|m| m.len()).unwrap_or(0);
        new_manifest.files.push(FileInfomation {
            path: el_path.clone(),
            hash: hash.clone(),
            size,
        });
    }
    let _ = new_manifest.save(&manifest_path);

    log::info!(
        "HMCL pack imported: {} written, {} skipped, {} deleted",
        witten, skipped, deleted
    );

    let modloader = if let Some(loader) = &pack.modloader {
        match loader.name.to_lowercase().as_str() {
            "forge" => crate::instance::ModLoader::Forge(loader.version.clone()),
            "fabric" => crate::instance::ModLoader::Fabric(loader.version.clone()),
            "quilt" => crate::instance::ModLoader::Quilt(loader.version.clone()),
            "neoforge" => crate::instance::ModLoader::NeoForge(loader.version.clone()),
            _ => crate::instance::ModLoader::Vanilla,
        }
    } else {
        crate::instance::ModLoader::Vanilla
    };

    let instance = crate::instance::Instance {
        id: instance_id.clone(),
        name: pack.name,
        version_id: pack.game_version,
        modloader,
        isolation_mode: crate::instance::IsolationMode::Always,
        game_dir_override: Some(instance_dir.to_string_lossy().into_owned()),
        created_at: chrono::Utc::now().to_rfc3339(),
        ..Default::default()
    };

    let instance_json = serde_json::to_string_pretty(&instance).map_err(|e| e.to_string())?;
    let skyline_di = instance_dir.join(".skyline");
    std::fs::create_dir_all(&skyline_di).map_err(|e| e.to_string())?;
    std::fs::write(skyline_di.join("instance.json"), &instance_json).map_err(|e| e.to_string())?;

    Ok(instance_id)
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackConfigExpot {
    pub name: String,
    pub version: String,
    pub game_version: String,
    pub modloader: String,
    pub modloader_version: Option<String>,
    pub mods: Vec<ModConfigEnty>,
    pub settings: HashMap<String, serde_json::Value>,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModConfigEnty {
    pub file_name: String,
    pub mod_id: Option<String>,
    pub version: Option<String>,
    pub enabled: bool,
}


pub fn expot_modpack_config(
    instance_dir: &Path,
    name: &str,
    version: &str,
) -> Result<ModpackConfigExpot, String> {
    let skyline_di = instance_dir.join(".skyline");
    let instance_json_path = skyline_di.join("instance.json");

    let (game_version, modloader, modloader_version) = if instance_json_path.exists() {
        let content = std::fs::read_to_string(&instance_json_path)
            .map_err(|e| e.to_string())?;
        let instance: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| e.to_string())?;

        let game_version = instance["version_id"].as_str().unwrap_or("unknown").to_string();
        let modloader = match &instance["modloader"] {
            serde_json::Value::Object(obj) => {
                if obj.contains_key("Forge") { "forge".to_string() }
                else if obj.contains_key("Fabric") { "fabric".to_string() }
                else if obj.contains_key("Quilt") { "quilt".to_string() }
                else if obj.contains_key("NeoForge") { "neoforge".to_string() }
                else { "vanilla".to_string() }
            }
            _ => "vanilla".to_string(),
        };
        let modloader_version = match &instance["modloader"] {
            serde_json::Value::Object(obj) => {
                obj.values().next().and_then(|v| v.as_str()).map(String::from)
            }
            _ => None,
        };

        (game_version, modloader, modloader_version)
    } else {
        ("unknown".to_string(), "vanilla".to_string(), None)
    };

    let mods_di = instance_dir.join("mods");
    let mut mods = Vec::new();

    if mods_di.exists() {
        for entry in walkdir::WalkDir::new(&mods_di).max_depth(1).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() { continue; }

            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "jar" && ext != "disabled" { continue; }

            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            let enabled = ext != "disabled";

            let (mod_id, version) = extact_mod_info_from_filename(&file_name);

            mods.push(ModConfigEnty {
                file_name,
                mod_id,
                version,
                enabled,
            });
        }
    }

    let config_di = instance_dir.join("config");
    let mut settings = HashMap::new();

    if config_di.exists() {
        for entry in walkdir::WalkDir::new(&config_di).max_depth(1).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() { continue; }

            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "json" && ext != "toml" && ext != "cfg" { continue; }

            let config_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
            if let Ok(content) = std::fs::read_to_string(path) {
                if ext == "json" {
                    if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                        settings.insert(config_name, json);
                    }
                }
            }
        }
    }

    Ok(ModpackConfigExpot {
        name: name.to_string(),
        version: version.to_string(),
        game_version,
        modloader,
        modloader_version,
        mods,
        settings,
    })
}

fn extact_mod_info_from_filename(filename: &str) -> (Option<String>, Option<String>) {
    let name = filename.trim_end_matches(".jar").trim_end_matches(".disabled");

    let pats: Vec<&str> = name.splitn(2, |c| c == '-' || c == '_').collect();
    if pats.len() >= 2 {
        let mod_id = pats[0].to_string();
        let version = pats[1].to_string();
        if version.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
            return (Some(mod_id), Some(version));
        }
    }

    (Some(name.to_string()), None)
}

pub async fn check_modpack_update(
    project_id: &str,
    current_version: &str,
) -> Result<Option<ModrinthVersion>, String> {
    let versions = get_modrinth_versions(project_id).await?;

    if versions.is_empty() {
        return Ok(None);
    }

    let latest = &versions[0];
    if latest.version_numbe != current_version {
        Ok(Some(latest.clone()))
    } else {
        Ok(None)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModpackUpdateInfo {
    pub current_version: String,
    pub latest_version: String,
    pub changelog: Option<String>,
    pub download_url: String,
}