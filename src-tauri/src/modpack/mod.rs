use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tokio::io::AsyncWriteExt;
pub mod modpack;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthProject {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub versions: Vec<String>,
    pub client_side: String,
    pub server_side: String,
    #[serde(rename = "categories")]
    pub categoies: Vec<String>,
    pub license: Option<String>,
    pub icon_url: Option<String>,
    pub project_id: Option<String>,
    pub author: Option<String>,
    pub downloads: Option<i64>,
    pub date_modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthVersion {
    pub id: String,
    pub project_id: String,
    pub name: String,
    #[serde(rename = "version_number")]
    pub version_numbe: String,
    #[serde(default)]
    pub version_type: Option<String>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub files: Vec<ModrinthFile>,
    #[serde(default)]
    pub dependencies: Vec<ModrinthDependency>,
    pub date_published: String,
    pub changelog: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthDependency {
    #[serde(rename = "project_id")]
    pub project_id: Option<String>,
    #[serde(rename = "version_id")]
    pub version_id: Option<String>,
    #[serde(rename = "dependency_type")]
    pub dependency_type: String,
    #[serde(default)]
    pub file_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthFile {
    pub url: String,
    pub filename: String,
    pub primary: bool,
    pub size: i64,
    pub hashes: ModrinthHashers,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthHashers {
    pub sha512: String,
    pub sha1: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuseForgeMod {
    pub id: u64,
    pub name: String,
    pub slug: String,
    #[serde(rename = "summary")]
    pub summay: String,
    pub downloads: u64,
    pub category: Option<String>,
    #[serde(rename = "logo_url")]
    pub logo_ul: Option<String>,
    pub authors: Vec<String>,
    pub game_versions: Vec<String>,
    pub date_modified: String,
    pub categories: Vec<String>,
    pub license: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CuseForgeFile {
    pub id: u64,
    pub display_name: String,
    pub file_name: String,
    pub file_date: String,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,

    pub elease_type: Option<u64>,
    pub file_length: u64,
    pub download_url: String,
}

/// ── New CurseForge API (CF for Studios / api.curseforge.com) ──
///
/// NOTE: Register a free API key at https://console.curseforge.com/ and
/// replace the placeholder below. The key is embedded in the binary so users
/// do not need to configure anything.
const CF_API_KEY: &str = "PLACEHOLDER_REPLACE_WITH_YOUR_KEY";

#[derive(Debug, Clone, Deserialize)]
struct CfNewSearchEnvelope {
    data: CfNewSearchData,
}

#[derive(Debug, Clone, Deserialize)]
struct CfNewSearchData {
    results: Vec<CfNewModResult>,
    pagination: CfNewPagination,
}

#[derive(Debug, Clone, Deserialize)]
struct CfNewPagination {
    current_page: u32,
    total_count: u64,
}

#[derive(Debug, Clone, Deserialize)]
struct CfNewModResult {
    id: u64,
    name: String,
    slug: String,
    summary: Option<String>,
    download_count: Option<u64>,
    logo_url: Option<String>,
    authors: Vec<CfNewAuthor>,
    #[serde(rename = "latestFile")]
    latest_file: Option<CfNewLatestFile>,
    #[serde(rename = "dateModified")]
    date_modified: String,
    categories: Option<Vec<String>>,
    license: Option<String>,
    #[serde(rename = "classes")]
    class_ids: Option<Vec<CfNewClassId>>,
}

#[derive(Debug, Clone, Deserialize)]
struct CfNewAuthor {
    name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct CfNewLatestFile {
    #[serde(rename = "gameVersions")]
    game_versions: Vec<CfNewGameVersionRef>,
    #[serde(rename = "fileTypes")]
    file_types: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
struct CfNewGameVersionRef {
    #[serde(rename = "gameVersionTypeId")]
    gv_type_id: u64,
    #[serde(rename = "gameVersion")]
    gv_name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct CfNewClassId {
    id: u64,
    name: String,
}

#[derive(Debug, Clone, Deserialize)]
struct CfNewFileEnvelope {
    data: Vec<CfNewFile>,
}

#[derive(Debug, Clone, Deserialize)]
struct CfNewFile {
    id: u64,
    #[serde(rename = "displayName")]
    display_name: String,
    #[serde(rename = "fileName")]
    file_name: String,
    #[serde(rename = "fileDate")]
    file_date: String,
    #[serde(rename = "gameVersions")]
    game_versions: Vec<CfNewFileGameVersion>,
    #[serde(rename = "releaseType")]
    release_type: u64,
    #[serde(rename = "fileLength")]
    file_length: u64,
    #[serde(rename = "downloadUrl")]
    download_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct CfNewFileGameVersion {
    #[serde(rename = "gameVersionTypeId")]
    gv_type_id: u64,
    #[serde(rename = "gameVersion")]
    gv_name: String,
}

fn cuseforge_download_url(file_id: u64, file_name: &str) -> String {
    let id_st = file_id.to_string();
    if id_st.len() <= 8 {
        let padded = format!("{:0>8}", id_st);
        let (first, last) = padded.split_at(4);
        return format!("https://edge.forgecdn.net/files/{}/{}/{}", first, last, file_name);
    }
    let (first, last) = id_st.split_at(id_st.len() - 4);
    format!(
        "https://edge.forgecdn.net/files/{}/{}/{}",
        &first[..4],
        last,
        file_name
    )
}

/// Get the CurseForge API key from config, returning empty string if not set.
fn get_cf_api_key() -> String {
    CF_API_KEY.to_string()
}

/// Build the base URL for the new CurseForge API.
fn cf_api_base() -> String {
    "https://api.curseforge.com".to_string()
}

/// Search mods on CurseForge using the new API.
/// `class_id` is the category filter (0 = all, 6 = mods, 12 = resourcepacks, 6552 = shaderpacks, etc.)
pub async fn search_cuseforge_with_class(
    query: &str,
    game_version: Option<&str>,
    class_id: u32,
) -> Result<Vec<CuseForgeMod>, String> {
    let api_key = get_cf_api_key();
    let base = cf_api_base();

    let mut url = format!(
        "{}/v1/mods/search?gameId=432&pageSize=20&sortBy=6&classId={}&searchFilter={}",
        base, class_id, url_encode(query)
    );
    if let Some(gv) = game_version {
        if !gv.trim().is_empty() {
            url.push_str(&format!("&gameVersion={}", url_encode(gv)));
        }
    }

    let client = crate::mc::mirror::http_client();
    let mut req = client.get(&url);
    if !api_key.is_empty() {
        req = req.header("X-API-Key", api_key);
    }

    let esp = req.send().await.map_err(|e| format!("CurseForge 连接失败: {}", e))?;
    if !esp.status().is_success() {
        let status = esp.status();
        let body = esp.text().await.unwrap_or_default();
        if body.contains("401") || body.contains("Unauthorized") {
            return Err("请先到设置页面填写 CurseForge API Key（免费获取：https://console.curseforge.com/）".into());
        }
        return Err(format!("CurseForge 接口返回 {}: {}", status, body.chars().take(200).collect::<String>()));
    }
    let body = esp.text().await.map_err(|e| format!("读取 CurseForge 响应失败: {}", e))?;
    let envelope: CfNewSearchEnvelope = serde_json::from_str(&body)
        .map_err(|e| format!("CurseForge 响应解析失败: {}（接口可能返回了错误页面，请重试或检查网络）", e))?;

    Ok(envelope.data.results.into_iter().map(cf_new_mod_to_cuseforge_mod).collect())
}

pub async fn search_cuseforge(query: &str, game_version: Option<&str>, _category: Option<&str>) -> Result<Vec<CuseForgeMod>, String> {
    search_cuseforge_with_class(query, game_version, 6).await
}

/// Recommended CurseForge mods — empty query, sorted by downloads.
pub async fn recommended_cuseforge_mods(limit: u32, game_version: Option<&str>) -> Result<Vec<CuseForgeMod>, String> {
    let base = cf_api_base();
    let url = format!(
        "{}/v1/mods/search?gameId=432&pageSize={}&sortBy=6&classId=6",
        base, limit
    );
    if let Some(gv) = game_version {
        if !gv.trim().is_empty() {
            // gameVersion param expects a comma-separated list or single value
            // For recommendations, we don't filter by game version by default
        }
    }
    let client = crate::mc::mirror::http_client();
    let mut req = client.get(&url);
    if !CF_API_KEY.is_empty() {
        req = req.header("X-API-Key", CF_API_KEY);
    }
    let esp = req.send().await.map_err(|e| format!("CurseForge 连接失败: {}", e))?;
    if !esp.status().is_success() {
        return Err(format!("CurseForge 接口返回 {}", esp.status()));
    }
    let body = esp.text().await.map_err(|e| format!("读取 CurseForge 响应失败: {}", e))?;
    let envelope: CfNewSearchEnvelope = serde_json::from_str(&body)
        .map_err(|e| format!("CurseForge 响应解析失败: {}", e))?;
    Ok(envelope.data.results.into_iter().map(cf_new_mod_to_cuseforge_mod).collect())
}

/// Convert a new-format mod result to the internal CuseForgeMod type.
fn cf_new_mod_to_cuseforge_mod(m: CfNewModResult) -> CuseForgeMod {
    // Extract game versions and loaders from latestFile.gameVersions
    let mut game_versions: Vec<String> = Vec::new();
    let mut loaders: Vec<String> = Vec::new();
    if let Some(ref lf) = m.latest_file {
        for gv in &lf.game_versions {
            if gv.gv_type_id == 1 {
                game_versions.push(gv.gv_name.clone());
            } else if gv.gv_type_id == 2 {
                loaders.push(gv.gv_name.clone());
            }
        }
    }

    // Map category IDs to names
    let category = m.class_ids
        .as_ref()
        .and_then(|classes| classes.first())
        .map(|c| c.name.clone());

    CuseForgeMod {
        id: m.id,
        name: m.name,
        slug: m.slug,
        summay: m.summary.unwrap_or_default(),
        downloads: m.download_count.unwrap_or(0),
        category,
        logo_ul: m.logo_url,
        authors: m.authors.into_iter().map(|a| a.name).collect(),
        game_versions,
        date_modified: m.date_modified,
        categories: m.categories.unwrap_or_default(),
        license: m.license,
    }
}

/// Get detailed mod info from CurseForge using the new API.
pub async fn get_cuseforge_project(mod_id: u64) -> Result<CuseForgeMod, String> {
    let api_key = get_cf_api_key();
    let base = cf_api_base();
    let url = format!("{}/v1/mods/{}", base, mod_id);

    let client = crate::mc::mirror::http_client();
    let mut req = client.get(&url);
    if !api_key.is_empty() {
        req = req.header("X-API-Key", api_key);
    }

    let esp = req.send().await.map_err(|e| format!("CurseForge 连接失败: {}", e))?;
    if !esp.status().is_success() {
        return Err(format!("CurseForge 接口返回 {}", esp.status()));
    }
    let body = esp.text().await.map_err(|e| format!("读取 CurseForge 响应失败: {}", e))?;
    let envelope: CfNewSearchEnvelope = serde_json::from_str(&body)
        .map_err(|e| format!("CurseForge 响应解析失败: {}", e))?;

    // The API returns a single mod in a list; take the first result
    let result = envelope
        .data
        .results
        .into_iter()
        .next()
        .ok_or_else(|| "CurseForge 未找到该模组".to_string())?;

    Ok(cf_new_mod_to_cuseforge_mod(result))
}

/// Get file list for a CurseForge mod using the new API.
pub async fn get_cuseforge_files(mod_id: u64) -> Result<Vec<CuseForgeFile>, String> {
    let api_key = get_cf_api_key();
    let base = cf_api_base();
    let url = format!("{}/v1/mods/{}/files", base, mod_id);

    let client = crate::mc::mirror::http_client();
    let mut req = client.get(&url);
    if !api_key.is_empty() {
        req = req.header("X-API-Key", api_key);
    }

    let esp = req.send().await.map_err(|e| format!("CurseForge 连接失败: {}", e))?;
    if !esp.status().is_success() {
        return Err(format!("CurseForge 接口返回 {}", esp.status()));
    }
    let body = esp.text().await.map_err(|e| format!("读取 CurseForge 响应失败: {}", e))?;
    let envelope: CfNewFileEnvelope = serde_json::from_str(&body)
        .map_err(|e| format!("CurseForge 响应解析失败: {}", e))?;

    Ok(envelope.data.into_iter().map(|f| {
        // Split gameVersions by type: 1=game version, 2=loader
        let mut game_versions: Vec<String> = Vec::new();
        let mut loaders: Vec<String> = Vec::new();
        for gv in &f.game_versions {
            if gv.gv_type_id == 1 {
                game_versions.push(gv.gv_name.clone());
            } else if gv.gv_type_id == 2 {
                loaders.push(gv.gv_name.clone());
            }
        }
        let download_url = f.download_url.unwrap_or_else(|| {
            // Fallback: construct download URL from file ID
            cuseforge_download_url(f.id, &f.file_name)
        });
        CuseForgeFile {
            id: f.id,
            display_name: f.display_name,
            file_name: f.file_name,
            file_date: f.file_date,
            game_versions,
            loaders,
            elease_type: Some(f.release_type),
            file_length: f.file_length,
            download_url,
        }
    }).collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModrinthProjectDetail {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub body: String,
    pub downloads: i64,
    pub follows: i64,
    pub published: String,
    pub updated: String,
    pub license: Option<String>,
    pub categoies: Vec<String>,
    pub client_side: String,
    pub server_side: String,
    pub project_type: String,
    pub icon_url: Option<String>,
    pub game_versions: Vec<String>,
    pub loaders: Vec<String>,
    pub source_ul: Option<String>,
    pub wiki_ul: Option<String>,
    pub issues_ul: Option<String>,
    pub team: Vec<String>,
}

fn url_encode(s: &str) -> String {
    let mut encoded = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(byte as char);
            }
            b' ' => encoded.push_str("%20"),
            _ => encoded.push_str(&format!("%{:02X}", byte)),
        }
    }
    encoded
}


fn chinese_mod_keywods(query: &str) -> String {
    let map = [
        ("钠", "sodium"),
        ("优化", "performance sodium lithium"),
        ("高清修复", "optifine"),
        ("光影", "shaders"),
        ("小地图", "minimap journey map voxelmap"),
        ("背包", "inventory JEI REI EMI"),
        ("整理", "inventory sorting"),
        ("苹果", "appleskin"),
        ("伤害", "damage indicator"),
        ("放大", "zoom"),
        ("区块", "chunk"),
        ("信息", "info HUD"),
        ("血量", "health"),
        ("经验", "experience"),
        ("掉落", "drop"),
        ("传送", "waylink travel"),
        ("地图", "map journey"),
        ("按键", "keybind"),
        ("皮肤", "skin custom"),
        ("披风", "cape"),
        ("中文", "chinese language translation"),
        ("RPG", "rpg magic"),
        ("武器", "weapon"),
        ("工具", "tool"),
        ("食物", "food"),
        ("能源", "energy"),
        ("存储", "storage"),
        ("管道", "pipe transport"),
        ("林业", "forestry"),
        ("农业", "farm"),
        ("矿业", "mining ore"),
        ("建筑", "build decoration"),
    ];
    for (cn, en) in map {
        if query.contains(cn) {
            return format!("{} {}", query, en);
        }
    }
    query.to_string()
}

pub async fn search_modrinth_ex(
    query: &str,
    limit: u32,
    offset: u32,
    project_type: &str,
    index: Option<&str>,
    game_version: Option<&str>,
    loaders: Option<&[String]>,
) -> Result<Vec<ModrinthProject>, String> {
    let enhanced_query = chinese_mod_keywods(query);
    let mut facet_goups: Vec<String> = Vec::new();
    facet_goups.push(format!("[\"project_type:{}\"]", project_type));
    if let Some(gv) = game_version {
        if !gv.trim().is_empty() {
            facet_goups.push(format!("[\"versions:{}\"]", gv.trim()));
        }
    }
    if let Some(lds) = loaders {
        for l in lds.iter().filter(|l| !l.trim().is_empty()) {
            facet_goups.push(format!("[\"categories:{}\"]", l.trim()));
        }
    }
    let facets = format!("[{}]", facet_goups.join(","));
    let mut url = format!(
        "https://api.modrinth.com/v2/search?query={}&limit={}&offset={}&facets={}&hl=zh",
        url_encode(&enhanced_query),
        limit,
        offset,
        facets
    );
    if let Some(index) = index {
        url.push_str(&format!("&index={}", index));
    }

    #[derive(Deserialize)]
    struct Hit {
        slug: String,
        title: String,
        description: String,
        versions: Vec<String>,
        client_side: String,
        server_side: String,
        #[serde(rename = "categories")]
        categoies: Vec<String>,
        license: Option<String>,
        icon_url: Option<String>,
        project_id: Option<String>,
        author: Option<String>,
        downloads: Option<i64>,
        date_modified: Option<String>,
    }

    #[derive(Deserialize)]
    struct SeachResponse {
        hits: Vec<Hit>,
        total_hits: u64,
    }

    let client = crate::mc::mirror::http_client();
    let esp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let data: SeachResponse = esp.json().await.map_err(|e| e.to_string())?;
    Ok(data.hits.into_iter().map(|h| ModrinthProject {
        slug: h.slug,
        title: h.title,
        description: h.description,
        versions: h.versions,
        client_side: h.client_side,
        server_side: h.server_side,
        categoies: h.categoies,
        license: h.license,
        icon_url: h.icon_url,
        project_id: h.project_id,
        author: h.author,
        downloads: h.downloads,
        date_modified: h.date_modified,
    }).collect())
}

pub async fn search_modrinth(query: &str, limit: u32, offset: u32, game_version: Option<&str>, loaders: Option<&[String]>) -> Result<Vec<ModrinthProject>, String> {
    search_modrinth_ex(query, limit, offset, "mod", None, game_version, loaders).await
}

pub async fn search_resource_packs(query: &str, limit: u32, offset: u32, game_version: Option<&str>, loaders: Option<&[String]>) -> Result<Vec<ModrinthProject>, String> {
    search_modrinth_ex(query, limit, offset, "resourcepack", None, game_version, loaders).await
}

pub async fn search_shader_packs(query: &str, limit: u32, offset: u32, game_version: Option<&str>, loaders: Option<&[String]>) -> Result<Vec<ModrinthProject>, String> {
    search_modrinth_ex(query, limit, offset, "shader", None, game_version, loaders).await
}

pub async fn search_datapacks(query: &str, limit: u32, offset: u32, game_version: Option<&str>) -> Result<Vec<ModrinthProject>, String> {
    search_modrinth_ex(query, limit, offset, "datapack", None, game_version, None).await
}

pub async fn search_worlds(query: &str, limit: u32, offset: u32, game_version: Option<&str>) -> Result<Vec<ModrinthProject>, String> {
    search_modrinth_ex(query, limit, offset, "world", None, game_version, None).await
}

pub async fn recommended_mods(limit: u32, game_version: Option<&str>, loaders: Option<&[String]>) -> Result<Vec<ModrinthProject>, String> {
    search_modrinth_ex("", limit, 0, "mod", Some("downloads"), game_version, loaders).await
}

pub async fn recommended_resource_packs(limit: u32, game_version: Option<&str>) -> Result<Vec<ModrinthProject>, String> {
    search_modrinth_ex("", limit, 0, "resourcepack", Some("downloads"), game_version, None).await
}

pub async fn recommended_shader_packs(limit: u32, game_version: Option<&str>) -> Result<Vec<ModrinthProject>, String> {
    search_modrinth_ex("", limit, 0, "shader", Some("downloads"), game_version, None).await
}

pub async fn recommended_modpacks(limit: u32, game_version: Option<&str>) -> Result<Vec<ModrinthProject>, String> {
    search_modrinth_ex("", limit, 0, "modpack", Some("downloads"), game_version, None).await
}

pub async fn recommended_datapacks(limit: u32, game_version: Option<&str>) -> Result<Vec<ModrinthProject>, String> {
    search_modrinth_ex("", limit, 0, "datapack", Some("downloads"), game_version, None).await
}

pub async fn recommended_worlds(limit: u32, game_version: Option<&str>) -> Result<Vec<ModrinthProject>, String> {
    search_modrinth_ex("", limit, 0, "world", Some("downloads"), game_version, None).await
}

pub async fn search_modpacks(query: &str, limit: u32, offset: u32, game_version: Option<&str>) -> Result<Vec<ModrinthProject>, String> {
    search_modrinth_ex(query, limit, offset, "modpack", None, game_version, None).await
}

pub async fn get_modrinth_project(slug: &str) -> Result<ModrinthProject, String> {
    let url = format!("https://api.modrinth.com/v2/project/{}", url_encode(slug));
    let client = crate::mc::mirror::http_client();
    let esp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    esp.json().await.map_err(|e| e.to_string())
}

pub async fn get_modrinth_versions(project_id: &str) -> Result<Vec<ModrinthVersion>, String> {
    let url = format!("https://api.modrinth.com/v2/project/{}/version", url_encode(project_id));
    let client = crate::mc::mirror::http_client();
    let esp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    esp.json().await.map_err(|e| e.to_string())
}

pub async fn get_modrinth_project_detail(slug: &str) -> Result<ModrinthProjectDetail, String> {
    let client = reqwest::Client::builder()
        .user_agent("SkyLineLauncher/1.0 (contact: launcher)")
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| e.to_string())?;

    let url = format!("https://api.modrinth.com/v2/project/{}", url_encode(slug));
    let esp = client.get(&url).send().await.map_err(|e| format!("Modrinth 连接失败: {}", e))?;
    if !esp.status().is_success() {
        return Err(format!("Modrinth 接口返回 {}", esp.status()));
    }
    let json: serde_json::Value = esp.json().await.map_err(|e| e.to_string())?;

    let get_st = |k: &str| json.get(k).and_then(|v| v.as_str()).unwrap_or("").to_string();
    let get_opt_st = |k: &str| json.get(k).and_then(|v| v.as_str()).map(String::from);
    let get_ar = |k: &str| -> Vec<String> {
        json.get(k)
            .and_then(|v| v.as_array())
            .map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect())
            .unwrap_or_default()
    };

    let license = json
        .get("license")
        .and_then(|v| v.get("id"))
        .and_then(|v| v.as_str())
        .map(String::from)
        .or_else(|| get_opt_st("license"));

    let team: Vec<String> = {
        let membes_ul = format!("https://api.modrinth.com/v2/project/{}/members", url_encode(slug));
        if let Ok(esp) = client.get(&membes_ul).send().await {
            if let Ok(ar) = esp.json::<serde_json::Value>().await {
                ar.as_array()
                    .map(|a| {
                        a.iter()
                            .filter_map(|m| {
                                m.get("user")
                                    .and_then(|u| u.get("username"))
                                    .and_then(|u| u.as_str())
                                    .map(String::from)
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        }
    };

    Ok(ModrinthProjectDetail {
        slug: get_st("slug"),
        title: get_st("title"),
        description: get_st("description"),
        body: get_st("body"),
        downloads: json.get("downloads").and_then(|v| v.as_i64()).unwrap_or(0),
        follows: json.get("follows").and_then(|v| v.as_i64()).unwrap_or(0),
        published: get_st("published"),
        updated: get_st("updated"),
        license,
        categoies: get_ar("categories"),
        client_side: get_st("client_side"),
        server_side: get_st("server_side"),
        project_type: get_st("project_type"),
        icon_url: get_opt_st("icon_url"),
        game_versions: get_ar("game_versions"),
        loaders: get_ar("loaders"),
        source_ul: get_opt_st("source_url"),
        wiki_ul: get_opt_st("wiki_url"),
        issues_ul: get_opt_st("issues_url"),
        team,
    })
}


pub async fn check_mod_updates(mods_di: &std::path::Path) -> Result<Vec<ModUpdateInfo>, String> {
    use std::collections::HashMap;
    let mut updates = Vec::new();

    if !mods_di.exists() {
        return Ok(updates);
    }

    let mut sha1_map: HashMap<String, String> = HashMap::new();
    for entry in walkdir::WalkDir::new(mods_di).max_depth(1).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if !path.is_file() { continue; }
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        if ext != "jar" { continue; }

        let data = std::fs::read(path).map_err(|e| e.to_string())?;
        let hash = {
            use sha1::Digest;
            let mut hashe = sha1::Sha1::new();
            hashe.update(&data);
            hex::encode(hashe.finalize())
        };
        sha1_map.insert(hash, path.to_string_lossy().to_string());
    }

    if sha1_map.is_empty() {
        return Ok(updates);
    }

    let hashes: Vec<&str> = sha1_map.keys().map(|s| s.as_str()).collect();
    let url = format!("https://api.modrinth.com/v2/version_files?hashes={}&algorithm=sha1",
        serde_json::to_string(&hashes).map_err(|e| e.to_string())?);

    let client = crate::mc::mirror::http_client();
    let esp = client.get(&url)
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    #[derive(Deserialize)]
    struct VersionFileResponse {
        id: String,
        project_id: String,
        name: String,
        version_numbe: String,
        date_published: String,
        files: Vec<ModrinthFile>,
    }

    let version_data: HashMap<String, VersionFileResponse> = esp.json().await.map_err(|e| e.to_string())?;

    for (hash, path_st) in &sha1_map {
        if let Some(version) = version_data.get(hash) {
            
            let versions_ul = format!("https://api.modrinth.com/v2/project/{}/version", version.project_id);
            if let Ok(ve_esp) = client.get(&versions_ul).send().await {
                if let Ok(all_versions) = ve_esp.json::<Vec<ModrinthVersion>>().await {
                    let latest = all_versions.first();
                    if let Some(latest_version) = latest {
                        if latest_version.date_published > version.date_published {
                            let primary = latest_version.files.iter().find(|f| f.primary).unwrap_or(&latest_version.files[0]);
                            updates.push(ModUpdateInfo {
                                mod_path: path_st.clone(),
                                current_version: version.version_numbe.clone(),
                                latest_version: latest_version.version_numbe.clone(),
                                download_url: primary.url.clone(),
                                filename: primary.filename.clone(),
                                project_id: version.project_id.clone(),
                                version_id: latest_version.id.clone(),
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(updates)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModUpdateInfo {
    pub mod_path: String,
    pub current_version: String,
    pub latest_version: String,
    pub download_url: String,
    pub filename: String,
    pub project_id: String,
    pub version_id: String,
}

pub async fn download_file_to(
    url: &str,
    filename: &str,
    dest: &std::path::Path,
    on_progress: impl Fn(u64, u64) + Send + Sync + 'static,
    concurrency: usize,
) -> Result<String, String> {
    
    

    let client = crate::mc::mirror::http_client();

    // First, do a HEAD to get content-length and check range support
    let hread_resp = client.head(url).send().await.map_err(|e| format!("HEAD请求失败: {}", e))?;
    let total_size = hread_resp.content_length().unwrap_or(0);
    let supports_range = hread_resp
        .headers()
        .get("accept-ranges")
        .map(|v| v.to_str().unwrap_or("").contains("bytes"))
        .unwrap_or(false);

    // Use segmented download for large files with range support
    if total_size >= 2 * 1024 * 1024 && supports_range && concurrency > 1 {
        return download_segmented(&client, url, filename, dest, on_progress, concurrency).await;
    }

    // Simple sequential download
    download_file_sequential(&client, url, filename, dest, on_progress).await
}

async fn download_file_sequential(
    client: &reqwest::Client,
    url: &str,
    filename: &str,
    dest: &std::path::Path,
    on_progress: impl Fn(u64, u64) + Send + Sync + 'static,
) -> Result<String, String> {
    let response = client.get(url).send().await.map_err(|e| format!("下载失败: {}", e))?;
    let status = response.status();
    if !status.is_success() {
        return Err(format!("下载失败: HTTP {}", status));
    }
    let total_size = response.content_length().unwrap_or(0);
    let file_path = dest.join(sanitize_filename(filename));
    let mut file = tokio::fs::File::create(&file_path).await.map_err(|e| format!("创建文件失败: {}", e))?;

    let mut steam = response.bytes_stream();
    let mut downloaderd: u64 = 0;
    use futures::StreamExt;
    use tokio::io::AsyncWriteExt;
    while let Some(chunk) = steam.next().await {
        let chunk = chunk.map_err(|e| format!("下载中断: {}", e))?;
        file.write_all(&chunk).await.map_err(|e| format!("写入文件失败: {}", e))?;
        downloaderd += chunk.len() as u64;
        on_progress(downloaderd, total_size);
    }
    file.flush().await.map_err(|e| e.to_string())?;

    Ok(file_path.to_string_lossy().to_string())
}

async fn download_segmented(
    client: &reqwest::Client,
    url: &str,
    filename: &str,
    dest: &std::path::Path,
    on_progress: impl Fn(u64, u64) + Send + Sync + 'static,
    concurrency: usize,
) -> Result<String, String> {
    use std::sync::atomic::{AtomicU64, Ordering};
    use tokio::io::AsyncSeekExt;

    let total_size = client.head(url).send().await.map_err(|e| format!("HEAD失败: {}", e))?.content_length().unwrap_or(0);
    if total_size == 0 {
        return download_file_sequential(client, url, filename, dest, on_progress).await;
    }

    let segment_size = 4 * 1024 * 1024; // 4MB per segment
    let segment_count = ((total_size - 1) / segment_size) + 1;
    let file_path = dest.join(sanitize_filename(filename));

    // Pre-allocate file
    {
        let f = std::fs::OpenOptions::new().create(true).write(true).truncate(true).open(&file_path)
            .map_err(|e| format!("创建文件失败: {}", e))?;
        f.set_len(total_size).map_err(|e| e.to_string())?;
    }

    let downloaderd = Arc::new(AtomicU64::new(0));
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let client = Arc::new(client.clone());

    let mut handles = Vec::new();

    for i in 0..segment_count {
        let start = i * segment_size;
        let end = std::cmp::min(start + segment_size - 1, total_size - 1);
        let sem = semaphore.clone();
        let cl = client.clone();
        let dl = downloaderd.clone();
        let url_str = url.to_string();
        let path = file_path.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.map_err(|e| format!("信号量获取失败: {}", e))?;
            let resp = cl.get(&url_str)
                .header("Range", format!("bytes={}-{}", start, end))
                .send().await
                .map_err(|e| format!("Segment {} 请求失败: {}", i, e))?;

            if !resp.status().is_success() {
                return Err(format!("Segment {} HTTP {}", i, resp.status()));
            }

            let bytes = resp.bytes().await.map_err(|e| format!("Segment {} 读取失败: {}", i, e))?;
            let expected_len = (end - start + 1) as usize;
            if bytes.len() != expected_len {
                return Err(format!("Segment {} 长度不匹配: 期望{}实际{}", i, expected_len, bytes.len()));
            }

            let mut file = tokio::fs::OpenOptions::new().write(true).open(&path).await
                .map_err(|e| format!("打开文件失败: {}", e))?;
            file.seek(std::io::SeekFrom::Start(start)).await
                .map_err(|e| format!("寻址失败: {}", e))?;
            file.write_all(&bytes).await
                .map_err(|e| format!("写入失败: {}", e))?;
            file.flush().await.map_err(|e| e.to_string())?;

            dl.fetch_add(bytes.len() as u64, Ordering::Relaxed);
            Ok::<(), String>(())
        });
        handles.push(handle);
    }

    // Progress updater — capture on_progress by cloning into Arc
    use std::sync::Mutex;
    let prog_dl = downloaderd.clone();
    let prog_total = total_size;
    let prog_cb = Arc::new(Mutex::new(on_progress));
    let prog_cb_clone = prog_cb.clone();
    let progress_handle = tokio::spawn(async move {
        let mut last = 0u64;
        loop {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            let cur = prog_dl.load(Ordering::Relaxed);
            if cur != last {
                last = cur;
                prog_cb_clone.lock().unwrap()(cur, prog_total);
            }
            if cur >= prog_total { break; }
        }
    });

    for handle in handles {
        handle.await.map_err(|e| e.to_string())??;
    }
    progress_handle.abort();

    // Final progress report
    prog_cb.lock().unwrap()(total_size, total_size);

    Ok(file_path.to_string_lossy().to_string())
}

fn sanitize_filename(name: &str) -> String {
    let mut s: String = name
        .chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' | '\0' | '\n' | '\r' => '_',
            c => c,
        })
        .collect();
    if s.trim().is_empty() {
        s = "download".to_string();
    }
    s
}

pub async fn download_modrinth_mod(
    version_id: &str,
    dest: &std::path::Path,
    on_progress: impl Fn(u64, u64) + Send + Sync + 'static,
    concurrency: usize,
) -> Result<String, String> {
    let url = format!("https://api.modrinth.com/v2/version/{}", version_id);
    let client = crate::mc::mirror::http_client();
    let esp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let version: ModrinthVersion = esp.json().await.map_err(|e| e.to_string())?;

    let primary = version.files.iter().find(|f| f.primary).unwrap_or(&version.files[0]);
    download_file_to(&primary.url, &primary.filename, dest, on_progress, concurrency).await
}

pub async fn fetch_modrinth_version(version_id: &str) -> Result<ModrinthVersion, String> {
    let url = format!("https://api.modrinth.com/v2/version/{}", url_encode(version_id));
    let client = crate::mc::mirror::http_client();
    let esp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    esp.json().await.map_err(|e| e.to_string())
}

async fn pick_dependency_version(project_id: &str, mc_version: &str, loader: &str, visiterd: &mut std::collections::HashSet<String>) -> Result<Option<String>, String> {
    if visiterd.contains(project_id) {
        return Ok(None);
    }
    visiterd.insert(project_id.to_string());
    let url = format!(
        "https://api.modrinth.com/v2/project/{}/version",
        url_encode(project_id)
    );
    let client = crate::mc::mirror::http_client();
    let esp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    let versions: Vec<ModrinthVersion> = esp.json().await.map_err(|e| e.to_string())?;

    let loaders_match: Vec<String> = if loader.is_empty() || loader == "vanilla" {
        Vec::new()
    } else {
        vec![loader.to_string()]
    };

    let mut best: Option<ModrinthVersion> = None;
    for v in versions {
        if !v.game_versions.iter().any(|g| g == mc_version) {
            continue;
        }
        if !loaders_match.is_empty() {
            let vl: Vec<String> = v.loaders.iter().map(|l| l.to_lowercase()).collect();
            if !vl.is_empty() && !vl.iter().any(|l| loaders_match.contains(l)) {
                continue;
            }
        }
        if v.files.is_empty() {
            continue;
        }
        let pre_release = v.version_type.as_deref().unwrap_or("release") == "release";
        match &best {
            Some(b) => {
                let b_pre = b.version_type.as_deref().unwrap_or("release") == "release";
                let b_date = &b.date_published;
                if pre_release && !b_pre {
                    best = Some(v);
                } else if pre_release == b_pre && &v.date_published > b_date {
                    best = Some(v);
                }
            }
            None => best = Some(v),
        }
    }
    Ok(best.map(|v| v.id))
}

pub async fn resolve_modrinth_dependencies(
    version_id: &str,
    mc_version: &str,
    loader: &str,
) -> Result<Vec<ModrinthDependency>, String> {
    let mut collected: Vec<ModrinthDependency> = Vec::new();
    let mut visiterd: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut queue: Vec<String> = vec![version_id.to_string()];

    while let Some(vid) = queue.pop() {
        let version = fetch_modrinth_version(&vid).await?;
        for dep in version.dependencies {
            if dep.dependency_type != "required" {
                continue;
            }
            let esolved_id = match &dep.version_id {
                Some(x) if !x.is_empty() => Some(x.clone()),
                _ => match &dep.project_id {
                    Some(p) if !p.is_empty() => {
                        pick_dependency_version(p, mc_version, loader, &mut visiterd).await?
                    }
                    _ => None,
                },
            };
            if let Some(id) = esolved_id {
                if !visiterd.insert(id.clone()) {
                    continue;
                }
                let dep_name = dep
                    .file_name
                    .clone()
                    .unwrap_or_else(|| id.clone());
                collected.push(ModrinthDependency {
                    project_id: dep.project_id.clone(),
                    version_id: Some(id.clone()),
                    dependency_type: "required".to_string(),
                    file_name: Some(dep_name),
                });
                queue.push(id);
            }
        }
    }

    Ok(collected)
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_recommended_mods() {
        let res = super::recommended_mods(12, None, None).await;
        match res {
            Ok(list) => {
                println!("recommended_mods OK len={}", list.len());
                if let Some(f) = list.first() {
                    println!("first: {} / {}", f.title, f.slug);
                }
            }
            Err(e) => println!("recommended_mods ERR: {}", e),
        }
        let res2 = super::recommended_resource_packs(12, None).await;
        match res2 {
            Ok(list) => println!("recommended_resource_packs OK len={}", list.len()),
            Err(e) => println!("recommended_resource_packs ERR: {}", e),
        }
    }
}
