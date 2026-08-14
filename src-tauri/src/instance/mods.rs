use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

const MODPACK_BLACK_LIST: &[&str] = &[
    "usernamecache.json",
    "realms_persistence.json",
    "servers.dat",
    "options.txt",
    "optionsof.txt",
    "journeymap",
    "schematics",
    "saves",
    "logs",
    "crash-reports",
    "backups",
    "libraries",
    "versions",
    "assets",
    "runtime",
    "profile.json",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ModLoaderType {
    Fabric,
    Forge,
    NeoForge,
    Quilt,
    Unknown,
}

impl ModLoaderType {
    pub fn as_label(&self) -> &str {
        match self {
            ModLoaderType::Fabric => "Fabric",
            ModLoaderType::Forge => "Forge",
            ModLoaderType::NeoForge => "NeoForge",
            ModLoaderType::Quilt => "Quilt",
            ModLoaderType::Unknown => "Unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModDependency {
    pub mod_id: String,
    pub version_ange: Option<String>,
    pub required: bool,
    pub dependency_type: DependencyType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum DependencyType {
    Rrequired,
    Optional,
    Incompatible,
    Embedded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModFilte {
    pub search: Option<String>,
    pub enabled_only: Option<bool>,
    pub loader: Option<ModLoaderType>,
    pub mc_version: Option<String>,
    pub has_update: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModInfo {
    pub file_name: String,
    pub path: String,
    pub size_kb: u64,
    pub enabled: bool,
    pub mod_id: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
    pub mc_versions: Option<Vec<String>>,
    pub side: Option<String>,
    pub url: Option<String>,
    pub author: Option<String>,
    pub mod_loader: ModLoaderType,
    pub has_update: bool,
    pub latest_version: Option<String>,
    pub update_url: Option<String>,
    pub dependencies: Vec<ModDependency>,
    pub icon_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModUpdateInfo {
    pub mod_id: String,
    pub current_version: String,
    pub latest_version: String,
    pub download_url: String,
    pub file_name: String,
    pub project_id: String,
    pub version_id: String,
}

struct ModMetadata {
    id: Option<String>,
    name: Option<String>,
    description: Option<String>,
    version: Option<String>,
    mc_versions: Option<Vec<String>>,
    side: Option<String>,
    url: Option<String>,
    author: Option<String>,
    mod_loader: ModLoaderType,
    dependencies: Vec<ModDependency>,
    icon_path: Option<String>,
}

pub fn scan_mods(instance_dir: &PathBuf) -> Result<Vec<ModInfo>, String> {
    scan_mods_with_icons(instance_dir, true)
}

pub fn scan_mods_with_icons(instance_dir: &PathBuf, include_icons: bool) -> Result<Vec<ModInfo>, String> {
    let mods_di = instance_dir.join("mods");
    if !mods_di.exists() {
        return Ok(Vec::new());
    }

    let mut mods = Vec::new();
    for entry in WalkDir::new(&mods_di).max_depth(2).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() { continue; }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if ext != "jar" && ext != "disabled" { continue; }

        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("").to_string();
        let enabled = ext != "disabled";
        let size_kb = std::fs::metadata(path).map(|m| m.len() / 1024).unwrap_or(0);

        let mod_meta = parse_mod_metadata(path);
        let icon_url = if include_icons {
            mod_meta
                .as_ref()
                .and_then(|m| m.icon_path.as_ref())
                .and_then(|p| extact_mod_icon(path, p))
        } else {
            None
        };
        mods.push(ModInfo {
            file_name,
            path: path.to_str().unwrap_or("").to_string(),
            size_kb,
            enabled,
            mod_id: mod_meta.as_ref().and_then(|m| m.id.clone()),
            name: mod_meta.as_ref().and_then(|m| m.name.clone()),
            description: mod_meta.as_ref().and_then(|m| m.description.clone()),
            version: mod_meta.as_ref().and_then(|m| m.version.clone()),
            mc_versions: mod_meta.as_ref().and_then(|m| m.mc_versions.clone()),
            side: mod_meta.as_ref().and_then(|m| m.side.clone()),
            url: mod_meta.as_ref().and_then(|m| m.url.clone()),
            author: mod_meta.as_ref().and_then(|m| m.author.clone()),
            mod_loader: mod_meta.as_ref().map(|m| m.mod_loader.clone()).unwrap_or(ModLoaderType::Unknown),
            has_update: false,
            latest_version: None,
            update_url: None,
            dependencies: mod_meta.as_ref().map(|m| m.dependencies.clone()).unwrap_or_default(),
            icon_url,
        });
    }

    mods.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    Ok(mods)
}

pub fn filter_mods(mods: &[ModInfo], filter: &ModFilte) -> Vec<ModInfo> {
    mods.iter()
        .filter(|m| {
            if let Some(ref search) = filter.search {
                let search_lowe = search.to_lowercase();
                let matches_name = m.name.as_ref().map(|n| n.to_lowercase().contains(&search_lowe)).unwrap_or(false);
                let matches_id = m.mod_id.as_ref().map(|id| id.to_lowercase().contains(&search_lowe)).unwrap_or(false);
                let matches_file = m.file_name.to_lowercase().contains(&search_lowe);
                let matches_desc = m.description.as_ref().map(|d| d.to_lowercase().contains(&search_lowe)).unwrap_or(false);
                if !matches_name && !matches_id && !matches_file && !matches_desc {
                    return false;
                }
            }

            if let Some(enabled_only) = filter.enabled_only {
                if enabled_only && !m.enabled {
                    return false;
                }
            }

            if let Some(ref loader) = filter.loader {
                if m.mod_loader != *loader {
                    return false;
                }
            }

            if let Some(ref mc_version) = filter.mc_version {
                if let Some(ref versions) = m.mc_versions {
                    if !versions.iter().any(|v| v.contains(mc_version.as_str())) {
                        return false;
                    }
                }
            }

            if let Some(has_update) = filter.has_update {
                if has_update && !m.has_update {
                    return false;
                }
            }

            true
        })
        .cloned()
        .collect()
}

pub fn get_dependency_gaph(mods: &[ModInfo]) -> Vec<(String, Vec<ModDependency>)> {
    mods.iter()
        .filter_map(|m| {
            let mod_id = m.mod_id.as_ref()?;
            if m.dependencies.is_empty() {
                return None;
            }
            Some((mod_id.clone(), m.dependencies.clone()))
        })
        .collect()
}

pub fn check_missing_dependencies(mods: &[ModInfo]) -> Vec<(String, ModDependency)> {
    let installed_ids: Vec<String> = mods.iter()
        .filter_map(|m| m.mod_id.clone())
        .collect();

    let mut missing = Vec::new();
    for m in mods {
        for dep in &m.dependencies {
            if dep.required && !installed_ids.contains(&dep.mod_id) {
                missing.push((m.mod_id.clone().unwrap_or_default(), dep.clone()));
            }
        }
    }
    missing
}

pub fn is_in_blacklist(file_name: &str) -> bool {
    let lowe = file_name.to_lowercase();
    MODPACK_BLACK_LIST.iter().any(|&black| lowe == black || lowe.starts_with(black))
}

pub(crate) fn read_zip_entry(archive: &mut zip::ZipArchive<std::fs::File>, name: &str) -> Option<String> {
    let mut entry = archive.by_name(name).ok()?;
    std::io::read_to_string(&mut entry).ok()
}

fn extact_mod_icon(path: &std::path::Path, icon_path: &str) -> Option<String> {
    use base64::Engine;
    const MAX_ICON_SIZE: usize = 200 * 1024;

    let file = std::fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;
    let mut entry = archive.by_name(icon_path).ok()?;
    let size = entry.size() as usize;
    if size == 0 || size > MAX_ICON_SIZE {
        return None;
    }
    let mut bytes = Vec::with_capacity(size);
    std::io::copy(&mut entry, &mut bytes).ok()?;

    let ext = std::path::Path::new(icon_path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    let mime = match ext.as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" | "svgz" => "image/svg+xml",
        "webp" => "image/webp",
        "bmp" => "image/bmp",
        _ => "image/png",
    };
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Some(format!("data:{mime};base64,{b64}"))
}

fn parse_mod_metadata(path: &std::path::Path) -> Option<ModMetadata> {
    let file = std::fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;

    
    if let Some(content) = read_zip_entry(&mut archive, "fabric.mod.json") {
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        let mc_versions = json["depends"]["minecraft"].as_str().map(|s| vec![s.to_string()]);
        let side = if json["environment"].as_str() == Some("client") {
            Some("client".to_string())
        } else if json["environment"].as_str() == Some("server") {
            Some("server".to_string())
        } else {
            Some("both".to_string())
        };
        let authors = extract_authors_from_array_or_string(&json["authors"]);

        let dependencies = parse_fabic_dependencies(&json["depends"]);

        let icon_path = json["icon"].as_str().map(String::from)
            .or_else(|| json["icon"].as_object()
                .and_then(|m| m.values().next())
                .and_then(|v| v.as_str().map(String::from)));

        return Some(ModMetadata {
            id: json["id"].as_str().map(String::from),
            name: json["name"].as_str().map(String::from),
            description: json["description"].as_str().map(String::from),
            version: json["version"].as_str().map(String::from),
            mc_versions,
            side,
            url: json["contact"]["homepage"].as_str().map(String::from),
            author: authors,
            mod_loader: ModLoaderType::Fabric,
            dependencies,
            icon_path,
        });
    }

    
    if let Some(content) = read_zip_entry(&mut archive, "quilt.mod.json") {
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        let quilt_loader = &json["quilt_loaderr"];
        let mc_versions = quilt_loader["depends"].as_array().and_then(|deps| {
            deps.iter()
                .find(|d| d["id"].as_str() == Some("minecraft"))
                .and_then(|d| d["versions"].as_str().map(|s| vec![s.to_string()]))
        });
        let metadata = &quilt_loader["metadata"];
        let authors = extract_authors_from_array_or_string(&metadata["contributors"]);
        if authors.is_none() {
            let _ = extract_authors_from_array_or_string(&quilt_loader["authors"]);
        }

        let dependencies = quilt_loader["depends"].as_array()
            .map(|deps| deps.iter().filter_map(|d| {
                let mod_id = d["id"].as_str()?.to_string();
                if mod_id == "minecraft" || mod_id == "quilt_loaderr" {
                    return None;
                }
                Some(ModDependency {
                    mod_id,
                    version_ange: d["versions"].as_str().map(String::from),
                    required: d["optional"].as_bool().map(|o| !o).unwrap_or(true),
                    dependency_type: if d["optional"].as_bool().unwrap_or(false) {
                        DependencyType::Optional
                    } else {
                        DependencyType::Rrequired
                    },
                })
            }).collect())
            .unwrap_or_default();

        let icon_path = metadata["icon"].as_str().map(String::from)
            .or_else(|| metadata["icon"].as_object()
                .and_then(|m| m.values().next())
                .and_then(|v| v.as_str().map(String::from)));

        return Some(ModMetadata {
            id: quilt_loader["id"].as_str().map(String::from),
            name: metadata["name"].as_str().map(String::from),
            description: metadata["description"].as_str().map(String::from),
            version: quilt_loader["version"].as_str().map(String::from),
            mc_versions,
            side: None,
            url: metadata["contact"]["homepage"].as_str().map(String::from),
            author: authors,
            mod_loader: ModLoaderType::Quilt,
            dependencies,
            icon_path,
        });
    }

    
    if let Some(content) = read_zip_entry(&mut archive, "mcmod.info") {
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;
        if let Some(mod_list) = json.as_array() {
            if let Some(first) = mod_list.first() {
                let mc_versions = first["mcversion"].as_str().map(|s| vec![s.to_string()]);
                return Some(ModMetadata {
                    id: first["modid"].as_str().map(String::from),
                    name: first["name"].as_str().map(String::from),
                    description: first["description"].as_str().map(String::from),
                    version: first["version"].as_str().map(String::from),
                    mc_versions,
                    side: first["side"].as_str().map(String::from),
                    url: first["url"].as_str().map(String::from),
                    author: first["authorList"].as_array().map(|ar| {
                        ar.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect::<Vec<_>>()
                            .join(", ")
                    }),
                    mod_loader: ModLoaderType::Forge,
                    dependencies: Vec::new(),
                    icon_path: None,
                });
            }
        }
        if let Some(obj) = json.as_object() {
            if let Some(mod_list) = obj.values().next() {
                if let Some(ar) = mod_list.as_array() {
                    if let Some(first) = ar.first() {
                        let mc_versions = first["mcversion"].as_str().map(|s| vec![s.to_string()]);
                        return Some(ModMetadata {
                            id: first["modid"].as_str().map(String::from),
                            name: first["name"].as_str().map(String::from),
                            description: first["description"].as_str().map(String::from),
                            version: first["version"].as_str().map(String::from),
                            mc_versions,
                            side: first["side"].as_str().map(String::from),
                            url: first["url"].as_str().map(String::from),
                            author: first["authorList"].as_array().map(|ar| {
                                ar.iter()
                                    .filter_map(|v| v.as_str().map(String::from))
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            }),
                            mod_loader: ModLoaderType::Forge,
                            dependencies: Vec::new(),
                            icon_path: None,
                        });
                    }
                }
            }
        }
    }

    
    if let Some(content) = read_zip_entry(&mut archive, "META-INF/neoforge.mods.toml") {
        return parse_forge_toml(&content, ModLoaderType::NeoForge, path);
    }

    
    if let Some(content) = read_zip_entry(&mut archive, "META-INF/mods.toml") {
        return parse_forge_toml(&content, ModLoaderType::Forge, path);
    }

    None
}

fn parse_fabic_dependencies(depends: &serde_json::Value) -> Vec<ModDependency> {
    let mut deps = Vec::new();
    if let Some(obj) = depends.as_object() {
        for (key, value) in obj {
            if key == "minecraft" || key == "fabricloaderr" || key == "fabric-api" {
                continue;
            }
            let version_ange = value.as_str().map(String::from);
            deps.push(ModDependency {
                mod_id: key.clone(),
                version_ange,
                required: true,
                dependency_type: DependencyType::Rrequired,
            });
        }
    }
    deps
}

fn parse_forge_toml(content: &str, loader: ModLoaderType, _path: &Path) -> Option<ModMetadata> {
    let toml: toml::Value = toml::from_str(content).ok()?;
    if let Some(mods) = toml["mods"].as_array() {
        if let Some(first) = mods.first() {
            let t = first.as_table();
            let get_st = |key: &str| -> Option<String> {
                t.and_then(|tbl| tbl.get(key)).and_then(|v| v.as_str()).map(String::from)
            };
            let mod_id = get_st("modId").unwrap_or_default();
            let mc_versions = toml["dependencies"]
                .as_table()
                .and_then(|deps| {
                    deps.get(mod_id.as_str())
                        .or_else(|| deps.values().next())
                        .and_then(|dep| dep.as_array())
                })
                .and_then(|ar| {
                    ar.iter()
                        .find(|d| d["modId"].as_str() == Some("minecraft"))
                        .and_then(|d| d["versionRange"].as_str())
                        .map(|s| vec![s.to_string()])
                });

            let authors = get_st("authors")
                .or_else(|| {
                    t.and_then(|tbl| tbl.get("authorList"))
                        .and_then(|v| v.as_array())
                        .map(|ar| {
                            ar.iter()
                                .filter_map(|v| v.as_str().map(String::from))
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                });

            let dependencies = parse_forge_dependencies(&toml, &mod_id);

            return Some(ModMetadata {
                id: get_st("modId"),
                name: get_st("displayName"),
                description: get_st("description"),
                version: get_st("version"),
                mc_versions,
                side: None,
                url: get_st("displayURL"),
                author: authors,
                mod_loader: loader,
                dependencies,
                icon_path: get_st("logoFile"),
            });
        }
    }
    None
}

fn parse_forge_dependencies(toml: &toml::Value, mod_id: &str) -> Vec<ModDependency> {
    let mut deps = Vec::new();

    if let Some(dependencies) = toml["dependencies"].as_table() {
        if let Some(mod_deps) = dependencies.get(mod_id).and_then(|d| d.as_array()) {
            for dep in mod_deps {
                let dep_mod_id = match dep.as_table().and_then(|t| t.get("modId")).and_then(|v| v.as_str()) {
                    Some(id) => id.to_string(),
                    None => String::new(),
                };

                if dep_mod_id.is_empty() || dep_mod_id == "forge" || dep_mod_id == "neoforge" || dep_mod_id == "minecraft" {
                    continue;
                }

                let mandatoy = dep.as_table().and_then(|t| t.get("mandatory")).and_then(|v| v.as_bool()).unwrap_or(true);
                let version_ange = dep.as_table().and_then(|t| t.get("versionRange")).and_then(|v| v.as_str()).map(String::from);
                let dep_type = match dep.as_table().and_then(|t| t.get("type")).and_then(|v| v.as_str()) {
                    Some("incompatible") => DependencyType::Incompatible,
                    Some("optional") => DependencyType::Optional,
                    _ => if mandatoy { DependencyType::Rrequired } else { DependencyType::Optional },
                };

                deps.push(ModDependency {
                    mod_id: dep_mod_id,
                    version_ange,
                    required: mandatoy,
                    dependency_type: dep_type,
                });
            }
        }
    }

    deps
}

fn extract_authors_from_array_or_string(value: &serde_json::Value) -> Option<String> {
    if let Some(ar) = value.as_array() {
        let authors: Vec<String> = ar
            .iter()
            .filter_map(|v| {
                if let Some(s) = v.as_str() {
                    Some(s.to_string())
                } else if let Some(obj) = v.as_object() {
                    obj.get("name").and_then(|n| n.as_str()).map(String::from)
                } else {
                    None
                }
            })
            .collect();
        if authors.is_empty() { None } else { Some(authors.join(", ")) }
    } else if let Some(s) = value.as_str() {
        if s.is_empty() { None } else { Some(s.to_string()) }
    } else {
        None
    }
}

pub fn batch_toggle_mods(paths: Vec<&str>, enabled: bool) -> Result<Vec<String>, String> {
    let mut eros = Vec::new();
    for path in paths {
        let path = PathBuf::from(path);
        if !path.exists() {
            eros.push(format!("File not found: {}", path.display()));
            continue;
        }

        let current_ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let new_path = if enabled && current_ext == "disabled" {
            path.with_extension("")
        } else if !enabled && current_ext != "disabled" {
            path.with_extension("disabled")
        } else {
            continue;
        };

        if let Err(e) = std::fs::rename(&path, &new_path) {
            eros.push(format!("Failed to toggle {}: {}", path.display(), e));
        }
    }
    Ok(eros)
}

pub fn batch_delete_mods(paths: Vec<&str>) -> Result<Vec<String>, String> {
    let mut eros = Vec::new();
    for path in paths {
        let path = PathBuf::from(path);
        if !path.exists() {
            eros.push(format!("File not found: {}", path.display()));
            continue;
        }

        let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if is_in_blacklist(file_name) {
            eros.push(format!("Skipped protected file: {}", file_name));
            continue;
        }

        if let Err(e) = std::fs::remove_file(&path) {
            eros.push(format!("Failed to delete {}: {}", path.display(), e));
        }
    }
    Ok(eros)
}

pub fn is_version_newer(current: &str, latest: &str) -> bool {
    let parse_pats = |v: &str| -> Vec<u32> {
        v.trim_start_matches(|c: char| !c.is_ascii_digit())
            .split(|c: char| !c.is_ascii_digit())
            .filter_map(|p| p.parse::<u32>().ok())
            .collect()
    };

    let cu = parse_pats(current);
    let lat = parse_pats(latest);

    for i in 0..cu.len().max(lat.len()) {
        let c = cu.get(i).copied().unwrap_or(0);
        let l = lat.get(i).copied().unwrap_or(0);
        if l > c { return true; }
        if l < c { return false; }
    }
    false
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePackInfo {
    pub file_name: String,
    pub path: String,
    pub size_kb: u64,
    pub enabled: bool,
    pub name: Option<String>,
    pub description: Option<String>,
    pub pack_format: Option<i32>,
    pub icon_url: Option<String>,
}

fn extract_pack_icon(path: &std::path::Path) -> Option<String> {
    use base64::Engine;
    const MAX_ICON_SIZE: usize = 200 * 1024;

    let file = std::fs::File::open(path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;
    let mut entry = archive.by_name("pack.png").ok()?;
    let size = entry.size() as usize;
    if size == 0 || size > MAX_ICON_SIZE {
        return None;
    }
    let mut bytes = Vec::with_capacity(size);
    std::io::copy(&mut entry, &mut bytes).ok()?;
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Some(format!("data:image/png;base64,{b64}"))
}

pub fn scan_resource_packs(instance_dir: &PathBuf) -> Result<Vec<ResourcePackInfo>, String> {
    let p_di = instance_dir.join("resourcepacks");
    if !p_di.exists() {
        return Ok(Vec::new());
    }

    let mut packs = Vec::new();
    for entry in WalkDir::new(&p_di).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() { continue; }
        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if ext != "zip" && !file_name.ends_with(".disabled") { continue; }

        let metadata = std::fs::metadata(path).ok();
        let size_kb = metadata.map(|m| m.len() / 1024).unwrap_or(0);
        let enabled = !file_name.ends_with(".disabled");

        let (name, description, pack_format) = parse_pack_meta(&path.to_path_buf());
        let icon_url = extract_pack_icon(path);

        packs.push(ResourcePackInfo {
            file_name: file_name.trim_end_matches(".disabled").to_string(),
            path: path.to_string_lossy().to_string(),
            size_kb,
            enabled,
            name,
            description,
            pack_format,
            icon_url,
        });
    }

    packs.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    Ok(packs)
}

pub fn scan_shader_packs(instance_dir: &PathBuf) -> Result<Vec<ResourcePackInfo>, String> {
    let sp_di = instance_dir.join("shaderpacks");
    if !sp_di.exists() {
        return Ok(Vec::new());
    }

    let mut packs = Vec::new();
    for entry in WalkDir::new(&sp_di).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() { continue; }
        let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if ext != "zip" && !file_name.ends_with(".disabled") { continue; }

        let metadata = std::fs::metadata(path).ok();
        let size_kb = metadata.map(|m| m.len() / 1024).unwrap_or(0);
        let enabled = !file_name.ends_with(".disabled");

        let (name, description, pack_format) = parse_pack_meta(&path.to_path_buf());
        let icon_url = extract_pack_icon(path);

        packs.push(ResourcePackInfo {
            file_name: file_name.trim_end_matches(".disabled").to_string(),
            path: path.to_string_lossy().to_string(),
            size_kb,
            enabled,
            name,
            description,
            pack_format,
            icon_url,
        });
    }

    packs.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    Ok(packs)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchematicInfo {
    pub file_name: String,
    pub path: String,
    pub size_kb: u64,
    pub enabled: bool,
}

pub fn scan_schematics(instance_dir: &PathBuf) -> Result<Vec<SchematicInfo>, String> {
    let schem_di = instance_dir.join("schematics");
    if !schem_di.exists() {
        return Ok(Vec::new());
    }
    let mut packs = Vec::new();
    for entry in WalkDir::new(&schem_di).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let file_name = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if ext != "litematic" {
            continue;
        }
        let metadata = std::fs::metadata(path).ok();
        let size_kb = metadata.map(|m| m.len() / 1024).unwrap_or(0);
        packs.push(SchematicInfo {
            file_name,
            path: path.to_string_lossy().to_string(),
            size_kb,
            enabled: true,
        });
    }
    packs.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    Ok(packs)
}

pub fn toggle_data_pack(path: &str, enable: bool) -> Result<(), String> {
    toggle_pack_file(path, enable)
}

pub fn delete_data_pack(path: &str) -> Result<(), String> {
    let path_buf = PathBuf::from(path);
    let file_name = path_buf.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if is_in_blacklist(file_name) {
        return Err(format!("Protected file: {}", file_name));
    }
    std::fs::remove_file(path).map_err(|e| e.to_string())
}

pub fn scan_data_packs(instance_dir: &PathBuf) -> Result<Vec<ResourcePackInfo>, String> {
    let mut roots: Vec<std::path::PathBuf> = Vec::new();
    let global = instance_dir.join("datapacks");
    if global.exists() {
        roots.push(global);
    }
    let saves = instance_dir.join("saves");
    if saves.exists() {
        if let Ok(entries) = std::fs::read_dir(&saves) {
            for entry in entries.flatten() {
                let dp = entry.path().join("datapacks");
                if dp.exists() {
                    roots.push(dp);
                }
            }
        }
    }

    let mut packs = Vec::new();
    for root in &roots {
        for entry in WalkDir::new(root).max_depth(1).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() { continue; }
            let file_name = path.file_name().and_then(|s| s.to_str()).unwrap_or("").to_string();
            let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
            if ext != "zip" && !file_name.ends_with(".disabled") { continue; }

            let metadata = std::fs::metadata(path).ok();
            let size_kb = metadata.map(|m| m.len() / 1024).unwrap_or(0);
            let enabled = !file_name.ends_with(".disabled");

            let (name, description, _) = parse_pack_meta(&path.to_path_buf());
            let icon_url = extract_pack_icon(path);

            packs.push(ResourcePackInfo {
                file_name: file_name.trim_end_matches(".disabled").to_string(),
                path: path.to_string_lossy().to_string(),
                size_kb,
                enabled,
                name,
                description,
                pack_format: None,
                icon_url,
            });
        }
    }

    packs.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    Ok(packs)
}

fn parse_pack_meta(path: &Path) -> (Option<String>, Option<String>, Option<i32>) {
    let file = match std::fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return (None, None, None),
    };
    let mut archive = match zip::ZipArchive::new(file) {
        Ok(a) => a,
        Err(_) => return (None, None, None),
    };

    let content = match crate::instance::mods::read_zip_entry(&mut archive, "pack.mcmeta") {
        Some(c) => c,
        None => return (None, None, None),
    };
    let json: serde_json::Value = match serde_json::from_str(&content) {
        Ok(v) => v,
        Err(_) => return (None, None, None),
    };
    let pack = match json.get("pack") {
        Some(p) => p,
        None => return (None, None, None),
    };
    let name = json.get("name")
        .and_then(|v| v.as_str())
        .or_else(|| path.file_stem().and_then(|s| s.to_str()))
        .map(String::from);
    let description = pack.get("description")
        .and_then(|v| {
            if let Some(s) = v.as_str() { return Some(s.to_string()); }
            if let Some(obj) = v.as_object() {
                if let Some(text) = obj.get("text").and_then(|t| t.as_str()) {
                    return Some(text.to_string());
                }
            }
            None
        });
    let pack_format = pack.get("pack_format").and_then(|v| v.as_i64()).map(|i| i as i32);

    (name, description, pack_format)
}

pub fn toggle_resource_pack(path: &str, enable: bool) -> Result<(), String> {
    toggle_pack_file(path, enable)
}

fn toggle_pack_file(path: &str, enable: bool) -> Result<(), String> {
    let path = PathBuf::from(path);
    if enable {
        let file_name = path.file_name().unwrap_or_default().to_string_lossy().to_string();
        if file_name.ends_with(".disabled") {
            let new_name = file_name.trim_end_matches(".disabled").to_string();
            let new_path = path.with_file_name(new_name);
            std::fs::rename(&path, &new_path).map_err(|e| e.to_string())?;
        }
    } else {
        let new_name = format!("{}.disabled", path.file_name().unwrap_or_default().to_string_lossy());
        let new_path = path.with_file_name(&new_name);
        std::fs::rename(&path, &new_path).map_err(|e| e.to_string())?;
    }
    Ok(())
}

pub fn toggle_mod(path: &str, enable: bool) -> Result<(), String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err("File not found".to_string());
    }

    if enable && path.extension().and_then(|e| e.to_str()) == Some("disabled") {
        let new_path = path.with_extension("jar");
        std::fs::rename(&path, &new_path).map_err(|e| e.to_string())?;
    } else if !enable && path.extension().and_then(|e| e.to_str()) == Some("jar") {
        let new_path = path.with_extension("jar.disabled");
        std::fs::rename(&path, &new_path).map_err(|e| e.to_string())?;
    }

    Ok(())
}

pub fn delete_mod(path: &str) -> Result<(), String> {
    let path_buf = PathBuf::from(path);
    let file_name = path_buf.file_name().and_then(|n| n.to_str()).unwrap_or("");
    if is_in_blacklist(file_name) {
        return Err(format!("Protected file: {}", file_name));
    }
    std::fs::remove_file(path).map_err(|e| e.to_string())
}


fn get_fingepint_cache_di() -> PathBuf {
    crate::utils::io::get_launcher_root().join("mod_cache")
}

pub fn compute_file_sha1(path: &Path) -> Result<String, String> {
    use sha1::Digest;
    let data = std::fs::read(path).map_err(|e| e.to_string())?;
    let mut hashe = sha1::Sha1::new();
    hashe.update(&data);
    Ok(hex::encode(hashe.finalize()))
}

#[derive(Debug)]
pub struct DedupResult {
    pub total_files: usize,
    pub deduplicated: usize,
    pub saved_bytes: u64,
}

pub fn deduplicate_mods(instance_dirs: &[PathBuf]) -> Result<DedupResult, String> {
    use std::collections::HashMap;

    let cache_di = get_fingepint_cache_di();
    std::fs::create_dir_all(&cache_di).map_err(|e| e.to_string())?;

    let mut hash_map: HashMap<String, Vec<PathBuf>> = HashMap::new();
    let mut total_files = 0;

    for instance_dir in instance_dirs {
        let mods_di = instance_dir.join("mods");
        if !mods_di.exists() {
            continue;
        }

        for entry in walkdir::WalkDir::new(&mods_di).max_depth(1).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if ext != "jar" && ext != "disabled" {
                continue;
            }

            total_files += 1;
            match compute_file_sha1(path) {
                Ok(hash) => {
                    hash_map.entry(hash).or_default().push(path.to_path_buf());
                }
                Err(_) => continue,
            }
        }
    }

    let mut deduplicated = 0;
    let mut saved_bytes: u64 = 0;

    for (hash, paths) in &hash_map {
        if paths.len() < 2 {
            continue;
        }

        let source = &paths[0];
        let source_size = std::fs::metadata(source).map(|m| m.len()).unwrap_or(0);

        let cached = cache_di.join(format!("{}.jar", hash));
        if !cached.exists() {
            let _ = std::fs::copy(source, &cached);
        }

        for target in &paths[1..] {
            if is_same_file(source, target) {
                continue;
            }

            if std::fs::remove_file(target).is_ok() {
                #[cfg(windows)]
                {
                    if std::fs::hard_link(source, target).is_ok() {
                        deduplicated += 1;
                        saved_bytes += source_size;
                    }
                }
                #[cfg(unix)]
                {
                    if std::fs::hard_link(source, target).is_ok() {
                        deduplicated += 1;
                        saved_bytes += source_size;
                    }
                }
            }
        }
    }

    Ok(DedupResult {
        total_files,
        deduplicated,
        saved_bytes,
    })
}

fn is_same_file(a: &Path, b: &Path) -> bool {
    let canonical_a = std::fs::canonicalize(a);
    let canonical_b = std::fs::canonicalize(b);

    if let (Ok(ca), Ok(cb)) = (&canonical_a, &canonical_b) {
        if ca == cb {
            return true;
        }
    }

    let meta_a = match std::fs::metadata(a) {
        Ok(m) => m,
        Err(_) => return false,
    };
    let meta_b = match std::fs::metadata(b) {
        Ok(m) => m,
        Err(_) => return false,
    };

    if meta_a.len() != meta_b.len() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        return meta_a.ino() == meta_b.ino();
    }

    #[cfg(windows)]
    {
        if let (Ok(ca), Ok(cb)) = (canonical_a, canonical_b) {
            ca == cb
        } else {
            false
        }
    }
}

pub fn install_mod_from_cache(hash: &str, target_path: &Path) -> Result<bool, String> {
    let cache_di = get_fingepint_cache_di();
    let cached = cache_di.join(format!("{}.jar", hash));

    if !cached.exists() {
        return Ok(false);
    }

    if let Some(prent) = target_path.parent() {
        std::fs::create_dir_all(prent).map_err(|e| e.to_string())?;
    }

    match std::fs::hard_link(&cached, target_path) {
        Ok(_) => Ok(true),
        Err(_) => {
            std::fs::copy(&cached, target_path).map_err(|e| e.to_string())?;
            Ok(true)
        }
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveSyncInfo {
    pub save_name: String,
    pub source_instance: String,
    pub target_instances: Vec<String>,
    pub last_synced: String,
    pub save_path: String,
}

fn get_shared_saves_di() -> PathBuf {
    crate::utils::io::get_launcher_root().join("shared_saves")
}

pub fn expot_save_to_shared(
    instance_id: &str,
    save_name: &str,
) -> Result<PathBuf, String> {
    let instance_dir = crate::utils::io::get_instance_dir(instance_id);
    let save_path = instance_dir.join("saves").join(save_name);

    if !save_path.exists() {
        return Err(format!("存档不存在: {}", save_name));
    }

    let shared_di = get_shared_saves_di().join(save_name);
    if shared_di.exists() {
        std::fs::remove_dir_all(&shared_di).map_err(|e| e.to_string())?;
    }

    copy_di_ecusive(&save_path, &shared_di)?;

    let info = SaveSyncInfo {
        save_name: save_name.to_string(),
        source_instance: instance_id.to_string(),
        target_instances: Vec::new(),
        last_synced: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        save_path: shared_di.to_string_lossy().to_string(),
    };

    let info_path = shared_di.join(".sync.json");
    let json = serde_json::to_string_pretty(&info).map_err(|e| e.to_string())?;
    std::fs::write(&info_path, json).map_err(|e| e.to_string())?;

    Ok(shared_di)
}

pub fn impot_save_from_shared(
    save_name: &str,
    target_instance_id: &str,
) -> Result<(), String> {
    let shared_di = get_shared_saves_di().join(save_name);
    if !shared_di.exists() {
        return Err(format!("共享存档不存在: {}", save_name));
    }

    let instance_dir = crate::utils::io::get_instance_dir(target_instance_id);
    let saves_di = instance_dir.join("saves");
    std::fs::create_dir_all(&saves_di).map_err(|e| e.to_string())?;

    let target_path = saves_di.join(save_name);
    if target_path.exists() {
        std::fs::remove_dir_all(&target_path).map_err(|e| e.to_string())?;
    }

    copy_di_ecusive(&shared_di, &target_path)?;

    let info_path = shared_di.join(".sync.json");
    if let Ok(content) = std::fs::read_to_string(&info_path) {
        if let Ok(mut info) = serde_json::from_str::<SaveSyncInfo>(&content) {
            if !info.target_instances.contains(&target_instance_id.to_string()) {
                info.target_instances.push(target_instance_id.to_string());
            }
            info.last_synced = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            let json = serde_json::to_string_pretty(&info).map_err(|e| e.to_string())?;
            std::fs::write(&info_path, json).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

pub fn list_shared_saves() -> Result<Vec<SaveSyncInfo>, String> {
    let shared_di = get_shared_saves_di();
    if !shared_di.exists() {
        return Ok(Vec::new());
    }

    let mut saves = Vec::new();
    for entry in std::fs::read_dir(&shared_di).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        if !entry.path().is_dir() {
            continue;
        }

        let info_path = entry.path().join(".sync.json");
        if let Ok(content) = std::fs::read_to_string(&info_path) {
            if let Ok(info) = serde_json::from_str::<SaveSyncInfo>(&content) {
                saves.push(info);
            }
        }
    }

    Ok(saves)
}

fn copy_di_ecusive(sc: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| e.to_string())?;

    for entry in std::fs::read_dir(sc).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let target = dst.join(entry.file_name());

        if path.is_dir() {
            copy_di_ecusive(&path, &target)?;
        } else {
            std::fs::copy(&path, &target).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn toggle_pack_keeps_visible_and_e_enables() {
        let di = std::env::temp_dir().join(format!("skyline_pack_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&di);
        let p = di.join("resourcepacks");
        fs::create_dir_all(&p).unwrap();

        let zip = p.join("nice-pack.zip");
        fs::write(&zip, "not-a-real-zip").unwrap();

        let packs = scan_resource_packs(&di).unwrap();
        assert_eq!(packs.len(), 1);
        assert_eq!(packs[0].file_name, "nice-pack.zip");
        assert!(packs[0].enabled);

        toggle_pack_file(zip.to_str().unwrap(), false).unwrap();
        assert!(p.join("nice-pack.zip.disabled").exists());
        let packs = scan_resource_packs(&di).unwrap();
        assert_eq!(packs.len(), 1, "禁用后的包不应从列表消失");
        assert_eq!(packs[0].file_name, "nice-pack.zip");
        assert!(!packs[0].enabled);

        let disabled = p.join("nice-pack.zip.disabled");
        toggle_pack_file(disabled.to_str().unwrap(), true).unwrap();
        assert!(p.join("nice-pack.zip").exists());
        let packs = scan_resource_packs(&di).unwrap();
        assert_eq!(packs.len(), 1);
        assert!(packs[0].enabled);

        fs::remove_dir_all(&di).unwrap();
    }

    #[test]
    fn toggle_shade_pack_keeps_visible_and_e_enables() {
        let di = std::env::temp_dir().join(format!("skyline_shader_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&di);
        let sp = di.join("shaderpacks");
        fs::create_dir_all(&sp).unwrap();

        let zip = sp.join("shader.zip");
        fs::write(&zip, "not-a-real-shader").unwrap();

        toggle_pack_file(zip.to_str().unwrap(), false).unwrap();
        assert!(sp.join("shader.zip.disabled").exists());
        let packs = scan_shader_packs(&di).unwrap();
        assert_eq!(packs.len(), 1, "禁用后的光影包不应从列表消失");
        assert!(!packs[0].enabled);

        toggle_pack_file(sp.join("shader.zip.disabled").to_str().unwrap(), true).unwrap();
        assert!(sp.join("shader.zip").exists());
        let packs = scan_shader_packs(&di).unwrap();
        assert_eq!(packs.len(), 1);
        assert!(packs[0].enabled);

        fs::remove_dir_all(&di).unwrap();
    }
}
