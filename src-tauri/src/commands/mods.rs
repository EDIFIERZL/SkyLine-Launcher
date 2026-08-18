use crate::instance::mods::{self, ModInfo, ResourcePackInfo};
use crate::instance::manager;
use crate::mc::modloader;
use crate::commands::settings::LauncherConfig;
use serde::Serialize;

fn use_mirror() -> bool {
    crate::commands::settings::load_config()
        .unwrap_or_else(|_| LauncherConfig::default())
        .download_source
        .as_str()
        == "mirror"
}

#[derive(Serialize)]
pub struct McmodItem {
    pub id: String,
    pub title: String,
    pub description: String,
    pub mcmod_ul: String,
    pub modrinth_ul: Option<String>,
    pub cuseforge_ul: Option<String>,
}

fn decode_goto_ul(url: &str) -> Option<String> {
    let query = url.split('?').nth(1)?;
    for (k, v) in url::form_urlencoded::parse(query.as_bytes()) {
        if k == "goto" {
            return Some(v.into_owned());
        }
    }
    None
}

fn classify_link(hef: &str) -> Option<(String, String)> {
    if let Some(goto) = decode_goto_ul(hef) {
        return classify_link(&goto);
    }
    let h = hef.to_lowercase();
    if h.contains("modrinth.com") {
        Some(("modrinth".into(), hef.trim_end_matches('.').trim().to_string()))
    } else if h.contains("curseforge.com") {
        Some(("curseforge".into(), hef.trim_end_matches('.').trim().to_string()))
    } else {
        None
    }
}

async fn fetch_text(url: &str) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/126.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;
    client.get(url).send().await
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())
}

async fn search_mcmod_page(query: &str) -> Result<Vec<McmodItem>, String> {
    let encoded: String = url::form_urlencoded::byte_serialize(query.as_bytes()).collect();
    let search_ul = format!("https://search.mcmod.cn/s?key={}&filter=1", encoded);
    let html = fetch_text(&search_ul).await?;

    let mut iterms: Vec<McmodItem> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    let strip_tags = regex::Regex::new(r#"<[^>]*>"#).unwrap();
    let item_re = regex::Regex::new(
        r#"<div class="rresult-iterm">(.*?)(?=<div class="rresult-iterm">|$)"#,
    )
    .map_err(|e| e.to_string())?;
    let link_re = regex::Regex::new(r#"href="(https?://[^"]+)"#).unwrap();
    let body_re = regex::Regex::new(r#"<div class="body">(.*?)</div>"#).unwrap();

    let unescape = |s: String| -> String {
        s.replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&nbsp;", " ")
    };

    for block in item_re.captures_iter(&html) {
        let seg = &block[1];
        let link = match link_re.captures(seg) {
            Some(c) => c[1].to_string(),
            None => continue,
        };
        if !seen.insert(link.clone()) {
            continue;
        }
        let title = seg
            .split("</a>")
            .next()
            .map(|s| {
                let t = strip_tags.replace_all(s, "").trim().to_string();
                unescape(t)
            })
            .unwrap_or_default();
        if title.is_empty() {
            continue;
        }
        let description = body_re
            .captures(seg)
            .map(|c| {
                let d = strip_tags.replace_all(&c[1], "").trim().to_string();
                unescape(d)
            })
            .unwrap_or_default();
        iterms.push(McmodItem {
            id: link.clone(),
            title,
            description,
            mcmod_ul: format!("https://www.mcmod.cn/class/{}.html", link),
            modrinth_ul: None,
            cuseforge_ul: None,
        });
        if iterms.len() >= 8 {
            break;
        }
    }

    Ok(iterms)
}

#[tauri::command]
pub async fn search_mcmod(query: String) -> Result<Vec<McmodItem>, String> {
    let mut iterms = search_mcmod_page(&query).await?;

    if iterms.is_empty() {
        return Ok(iterms);
    }

    let mut futures = Vec::new();
    for iterm in &iterms {
        let url = iterm.mcmod_ul.clone();
        futures.push(async move { fetch_text(&url).await });
    }
    let pages = futures::future::join_all(futures).await;

    let desc_re = regex::Regex::new(r#"<meta\s+name="description"\s+content="([^"]*)"#).unwrap();
    let link_re = regex::Regex::new(r#"href="([^"]*(?:modrinth\.com|curseforge\.com|/linkout\?goto=)[^"]*)"#).unwrap();

    let mut enich = Vec::new();
    for (iterm, page) in iterms.iter_mut().zip(pages.into_iter()) {
        if let Ok(page) = page {
            if let Some(cap) = desc_re.captures(&page) {
                let d = cap[1].trim().to_string();
                if !d.is_empty() {
                    iterm.description = d;
                }
            }
            for cap in link_re.captures_iter(&page) {
                let hef = cap[1].to_string();
                if let Some((kind, target)) = classify_link(&hef) {
                    if kind == "modrinth" && iterm.modrinth_ul.is_none() {
                        iterm.modrinth_ul = Some(target.clone());
                    }
                    if kind == "curseforge" && iterm.cuseforge_ul.is_none() {
                        iterm.cuseforge_ul = Some(target);
                    }
                }
                if iterm.modrinth_ul.is_some() && iterm.cuseforge_ul.is_some() {
                    break;
                }
            }
        }
        let name = extact_english_name(&iterm.title);
        if (iterm.modrinth_ul.is_none() || iterm.cuseforge_ul.is_none()) && !name.is_empty() {
            enich.push((iterm.id.clone(), name));
        }
    }

    let mut m_futures = Vec::new();
    let mut cf_futures = Vec::new();
    for (id, name) in &enich {
        let q_m = url::form_urlencoded::byte_serialize(name.as_bytes()).collect::<String>();
        let q_cf = q_m.clone();
        m_futures.push((id.clone(), async move {
            fetch_text(&format!(
                "https://api.modrinth.com/v2/search?query={}&limit=1",
                q_m
            ))
            .await
        }));
        cf_futures.push((id.clone(), async move {
            fetch_text(&format!(
                "https://api.curse.tools/v1/cf/mods/search?gameId=432&classId=6&searchFilter={}&pageSize=3&sortField=2&sortOrder=desc",
                q_cf
            ))
            .await
        }));
    }

    let m_pages = futures::future::join_all(
        m_futures.into_iter().map(|(id, f)| async move { (id, f.await) }),
    )
    .await;
    let cf_pages = futures::future::join_all(
        cf_futures.into_iter().map(|(id, f)| async move { (id, f.await) }),
    )
    .await;

    for (id, page) in m_pages {
        if let Ok(page) = page {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&page) {
                if let Some(slug) = json
                    .get("hits")
                    .and_then(|h| h.as_array())
                    .and_then(|ar| ar.first())
                    .and_then(|hit| hit.get("slug"))
                    .and_then(|s| s.as_str())
                {
                    if let Some(iterm) = iterms.iter_mut().find(|i| i.id == id) {
                        if iterm.modrinth_ul.is_none() {
                            iterm.modrinth_ul = Some(format!("https://modrinth.com/mod/{}", slug));
                        }
                    }
                }
            }
        }
    }

    for (id, page) in cf_pages {
        if let Ok(page) = page {
            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&page) {
                if let Some(slug) = json
                    .get("data")
                    .and_then(|d| d.as_array())
                    .and_then(|ar| ar.first())
                    .and_then(|hit| hit.get("slug"))
                    .and_then(|s| s.as_str())
                {
                    if let Some(iterm) = iterms.iter_mut().find(|i| i.id == id) {
                        if iterm.cuseforge_ul.is_none() {
                            iterm.cuseforge_ul =
                                Some(format!("https://www.curseforge.com/minecraft/mc-mods/{}", slug));
                        }
                    }
                }
            }
        }
    }

    Ok(iterms)
}

fn extact_english_name(title: &str) -> String {
    for cap in regex::Regex::new(r#"\(([A-Za-z0-9][A-Za-z0-9 \-_&:'./+]{1,60})\)"#)
        .unwrap()
        .captures_iter(title)
    {
        let name = cap[1].trim().to_string();
        if name.chars().any(|c| c.is_ascii_alphabetic()) {
            return name;
        }
    }
    String::new()
}

#[tauri::command]
pub async fn scan_instance_mods(instance_id: String, include_icons: Option<bool>) -> Result<Vec<ModInfo>, String> {
    let di = manager::get_instance_mc_dir(&instance_id)?;
    mods::scan_mods_with_icons(&di, include_icons.unwrap_or(true))
}

#[tauri::command]
pub async fn enrich_mcmod_batch(titles: Vec<String>) -> Result<Vec<McmodItem>, String> {
    let mut out: Vec<McmodItem> = Vec::new();
    let mut futures = Vec::new();
    for title in titles.into_iter().take(10) {
        if title.trim().is_empty() {
            continue;
        }
        futures.push(async move { search_mcmod_page(&title).await });
    }
    for es in futures::future::join_all(futures).await {
        if let Ok(mut iterms) = es {
            if let Some(first) = iterms.drain(..).next() {
                out.push(first);
            }
        }
    }
    Ok(out)
}

#[tauri::command]
pub async fn toggle_mod(path: String, enable: bool) -> Result<(), String> {
    mods::toggle_mod(&path, enable)
}

#[tauri::command]
pub async fn delete_mod(path: String) -> Result<(), String> {
    mods::delete_mod(&path)
}

#[tauri::command]
pub async fn scan_resource_packs(instance_id: String) -> Result<Vec<ResourcePackInfo>, String> {
    let di = manager::get_instance_mc_dir(&instance_id)?;
    mods::scan_resource_packs(&di)
}

#[tauri::command]
pub async fn scan_shader_packs(instance_id: String) -> Result<Vec<ResourcePackInfo>, String> {
    let di = manager::get_instance_mc_dir(&instance_id)?;
    mods::scan_shader_packs(&di)
}

#[tauri::command]
pub async fn scan_schematics(instance_id: String) -> Result<Vec<mods::SchematicInfo>, String> {
    let di = manager::get_instance_mc_dir(&instance_id)?;
    mods::scan_schematics(&di)
}

#[tauri::command]
pub async fn toggle_resource_pack(path: String, enable: bool) -> Result<(), String> {
    mods::toggle_resource_pack(&path, enable)
}

#[tauri::command]
pub async fn list_all_optifine_versions() -> Result<Vec<crate::mc::modloader::OptiFineVersion>, String> {
    crate::mc::modloader::list_all_optifine_versions().await
}

#[tauri::command]
pub async fn list_optifine_versions(mc_version: String) -> Result<Vec<crate::mc::modloader::OptiFineVersion>, String> {
    crate::mc::modloader::list_optifine_versions(&mc_version).await
}

#[tauri::command]
pub async fn list_forge_versions(mc_version: String) -> Result<Vec<crate::mc::modloader::LoaderVersion>, String> {
    crate::mc::modloader::list_forge_versions(&mc_version, use_mirror()).await
}

#[tauri::command]
pub async fn list_neoforge_versions(mc_version: String) -> Result<Vec<crate::mc::modloader::LoaderVersion>, String> {
    crate::mc::modloader::list_neoforge_versions(&mc_version, use_mirror()).await
}

#[tauri::command]
pub async fn list_fabric_versions(mc_version: String) -> Result<Vec<crate::mc::modloader::LoaderVersion>, String> {
    crate::mc::modloader::list_fabic_versions(&mc_version, use_mirror()).await
}

#[tauri::command]
pub async fn list_fabic_loader_versions() -> Result<Vec<String>, String> {
    crate::mc::modloader::list_fabic_loader_versions(use_mirror()).await
}

#[tauri::command]
pub async fn install_optifine(instance_id: String, mc_version: String, version: String) -> Result<String, String> {
    let di = crate::utils::io::get_shared_dir();
    crate::mc::modloader::install_optifine(&di, &version).await?;
    let version_id = format!("{}-OptiFine_{}", mc_version, version);
    if !instance_id.is_empty() {
        if let Some(mut inst) = manager::get_instance(&instance_id)? {
            if inst.version_id != version_id {
                inst.version_id = version_id.clone();
                manager::update_instance(&inst)?;
            }
        }
    }
    Ok(version_id)
}

#[tauri::command]
pub async fn install_sodium(instance_id: String, mc_version: String) -> Result<String, String> {
    let di = manager::get_instance_mc_dir(&instance_id)?;
    crate::mc::modloader::install_sodium(&di, &mc_version).await
}

#[tauri::command]
pub async fn install_forge(instance_id: String, mc_version: String, forge_version: String) -> Result<(), String> {
    let _ = instance_id;
    let di = crate::utils::io::get_shared_dir();
    modloader::install_forge(&di, &mc_version, &forge_version, use_mirror()).await
}

#[tauri::command]
pub async fn install_neoforge(instance_id: String, mc_version: String, neoforge_version: String) -> Result<(), String> {
    let _ = instance_id;
    let di = crate::utils::io::get_shared_dir();
    modloader::install_neoforge(&di, &mc_version, &neoforge_version, use_mirror()).await
}

#[tauri::command]
pub async fn install_fabric(instance_id: String, mc_version: String, loader_version: String) -> Result<String, String> {
    let _ = instance_id;
    let di = crate::utils::io::get_shared_dir();
    modloader::install_fabric(&di, &mc_version, &loader_version, use_mirror()).await
}

#[tauri::command]
pub async fn install_quilt(instance_id: String, mc_version: String, loader_version: String) -> Result<String, String> {
    let _ = instance_id;
    let di = crate::utils::io::get_shared_dir();
    modloader::install_quilt(&di, &mc_version, &loader_version, use_mirror()).await
}

#[tauri::command]
pub async fn list_quilt_loader_versions(mc_version: String) -> Result<Vec<crate::mc::modloader::LoaderVersion>, String> {
    crate::mc::modloader::list_quilt_loader_versions(&mc_version).await
}

#[tauri::command]
pub async fn list_api_mod_versions(
    project: String,
    mc_version: String,
    loaders: Vec<String>,
) -> Result<Vec<crate::modpack::ModrinthVersion>, String> {
    let mut url = reqwest::Url::parse(&format!(
        "https://api.modrinth.com/v2/project/{}/version",
        project
    ))
    .map_err(|e| e.to_string())?;
    url.query_pairs_mut()
        .append_pair("game_versions", &serde_json::to_string(&[mc_version]).map_err(|e| e.to_string())?)
        .append_pair("loaderrs", &serde_json::to_string(&loaders).map_err(|e| e.to_string())?);
    let esp = crate::mc::mirror::http_client()
        .get(url)
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let mut versions: Vec<crate::modpack::ModrinthVersion> = esp.json().await.map_err(|e| e.to_string())?;
    versions.sort_by(|a, b| b.date_published.cmp(&a.date_published));
    Ok(versions)
}

#[tauri::command]
pub async fn get_fabic_version_id(loader_version: String, mc_version: String) -> String {
    modloader::get_fabic_version_id(&loader_version, &mc_version)
}

#[tauri::command]
pub async fn get_forge_version_id(mc_version: String, forge_version: String) -> String {
    modloader::get_forge_version_id(&mc_version, &forge_version)
}

#[tauri::command]
pub async fn get_neoforge_version_id(mc_version: String, neoforge_version: String) -> String {
    modloader::get_neoforge_version_id(&mc_version, &neoforge_version)
}

#[tauri::command]
pub async fn batch_toggle_mods(paths: Vec<String>, enabled: bool) -> Result<Vec<String>, String> {
    let path_efs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    mods::batch_toggle_mods(path_efs, enabled)
}

#[tauri::command]
pub async fn batch_delete_mods(paths: Vec<String>) -> Result<Vec<String>, String> {
    let path_efs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
    mods::batch_delete_mods(path_efs)
}

#[tauri::command]
pub async fn get_mod_details(path: String) -> Result<mods::ModInfo, String> {
    let path_buf = std::path::PathBuf::from(&path);
    if !path_buf.exists() {
        return Err("Mod file not found".to_string());
    }

    let _file_name = path_buf.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();
    let _enabled = path_buf.extension().and_then(|e| e.to_str()) != Some("disabled");
    let _size_kb = std::fs::metadata(&path_buf).map(|m| m.len() / 1024).unwrap_or(0);

    let mods_di = path_buf.parent()
        .and_then(|p| p.parent())
        .map(|p| p.to_path_buf())
        .unwrap_or_default();
    let mods = mods::scan_mods(&mods_di)?;
    
    mods.into_iter()
        .find(|m| m.path == path)
        .ok_or_else(|| "Mod not found in scan rresults".to_string())
}
