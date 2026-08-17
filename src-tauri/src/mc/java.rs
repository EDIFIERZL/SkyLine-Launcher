use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::process::Command;
use crate::utils::io;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaInfo {
    pub path: String,
    pub version: String,
    pub major_version: u32,
    pub is_64bit: bool,
    pub architecture: String,
    pub vendor: String,
    #[serde(default)]
    pub is_jdk: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaManifest {
    pub binaries: Vec<JavaVersionEntry>,
    #[serde(default)]
    pub elease_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaVersionEntry {
    #[serde(default)]
    pub package: Option<JavaDownload>,
    #[serde(default)]
    pub image_type: String,
    #[serde(default)]
    pub os: String,
    #[serde(default)]
    pub architecture: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JavaDownload {
    #[serde(default)]
    pub checksum: String,
    #[serde(default)]
    pub checksum_type: String,
    pub link: String,
    pub size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaRecommendation {
    pub minecraft_version: String,
    pub required_java: u32,
    pub recommended_java: u32,
    pub reason: String,
    #[serde(default)]
    pub compatible_java: Vec<JavaInfo>,
}

const ADOPTIUM_API: &str = "https://api.adoptium.net/v3";

const ADOPTIUM_MIRRORS: &[&str] = &[
    "https://mirrors.tuna.tsinghua.edu.cn/Adoptium",
    "https://mirror.lzu.edu.cn/adoptium",
];

#[cfg(target_os = "windows")]
const WINDOWS_REGISTRY_PATHS: &[&str] = &[
    r"SOFTWARE\JavaSoft\Java Runtime Environment",
    r"SOFTWARE\JavaSoft\Java Development Kit",
    r"SOFTWARE\JavaSoft\JRE",
    r"SOFTWARE\JavaSoft\JDK",
    r"SOFTWARE\Eclipse Adoptium\JRE",
    r"SOFTWARE\Eclipse Adoptium\JDK",
    r"SOFTWARE\Azul Systems\Zulu",
    r"SOFTWARE\Amazon\JDK",
    r"SOFTWARE\Microsoft\JDK",
];

#[cfg(target_os = "windows")]
const COMMON_JAVA_DIRS: &[&str] = &[
    r"C:\Program Files\Java",
    r"C:\Program Files (x86)\Java",
    r"C:\Program Files\Eclipse Adoptium",
    r"C:\Program Files\Eclipse Foundation",
    r"C:\Program Files\Azul",
    r"C:\Program Files\Amazon Corretto",
    r"C:\Program Files\Microsoft\jdk",
    r"C:\Program Files\BellSoft",
    r"C:\Program Files\GraalVM",
    r"C:\Program Files\AdoptOpenJDK",
    r"C:\Program Files\Zulu",
];

#[cfg(target_os = "macos")]
const COMMON_JAVA_DIRS: &[&str] = &[
    "/Library/Java/JavaVirtualMachines",
    "/usr/local/opt/openjdk",
    "/usr/local/opt/openjdk@11",
    "/usr/local/opt/openjdk@17",
    "/usr/local/opt/openjdk@21",
    "/opt/homebrew/opt/openjdk",
    "/opt/homebrew/opt/openjdk@11",
    "/opt/homebrew/opt/openjdk@17",
    "/opt/homebrew/opt/openjdk@21",
];

#[cfg(target_os = "linux")]
const COMMON_JAVA_DIRS: &[&str] = &[
    "/usr/lib/jvm",
    "/usr/java",
    "/usr/local/java",
    "/opt/java",
    "/opt/jdk",
    "/snap/openjdk/current/jdk",
];

pub async fn fetch_java_list(major_version: u32) -> Result<Vec<JavaManifest>, String> {
    let os = if cfg!(target_os = "windows") { "windows" }
        else if cfg!(target_os = "macos") { "mac" }
        else { "linux" };

    let ach = if cfg!(target_arch = "x86_64") { "x64" }
        else if cfg!(target_arch = "aarch64") { "aarch64" }
        else { "x64" };

    let url = format!(
        "{}/assets/feature_releases/{}/ga?architecture={}&image_type=jdk&os={}&page_size=1&vendor=oracle",
        ADOPTIUM_API, major_version, ach, os
    );

    let client = crate::mc::mirror::http_client();
    let esp = client.get(&url).send().await.map_err(|e| e.to_string())?;
    if !esp.status().is_success() {
        return Err(format!("Java API 返回 HTTP {}", esp.status()));
    }
    esp.json::<Vec<JavaManifest>>().await.map_err(|e| e.to_string())
}

async fn fetch_java_download_official(major_version: u32, os: &str, ach: &str) -> Option<JavaDownload> {
    let client = crate::mc::mirror::http_client();
    for vendor in ["oracle", "eclipse"] {
        for image_type in ["jre", "jdk"] {
            let meta_ul = format!(
                "{}/assets/feature_releases/{}/ga?architecture={}&image_type={}&os={}&page_size=1&vendor={}",
                ADOPTIUM_API, major_version, ach, image_type, os, vendor
            );
            let esp = match client.get(&meta_ul).send().await {
                Ok(esp) if esp.status().is_success() => esp,
                _ => continue,
            };
            if let Ok(eleases) = esp.json::<Vec<JavaManifest>>().await {
                if let Some(manifest) = eleases.first() {
                    if let Some(entry) = manifest.binaries.iter().find(|b| b.package.is_some()) {
                        return entry.package.clone();
                    }
                }
            }
        }
    }
    None
}

fn parse_mirror_zip_entries(html: &str) -> Vec<(String, u64)> {
    let mut out = Vec::new();
    let Ok(e) = regex::Regex::new(r#"<a href="([^"]+\.zip)"[^>]*>[^<]+</a>\s*</td>\s*<td class="size">([^<]+)</td>"#) else {
        return out;
    };
    for cap in e.captures_iter(html) {
        let name = cap[1].to_string();
        if !name.contains("hotspot") {
            continue;
        }
        out.push((name, parse_mirror_size(&cap[2])));
    }
    out
}

fn parse_mirror_size(s: &str) -> u64 {
    let s = s.trim();
    let split_at = s.find(|c: char| !c.is_ascii_digit() && c != '.').unwrap_or(s.len());
    let (num_st, unit) = s.split_at(split_at);
    let Ok(v) = num_st.trim().parse::<f64>() else {
        return 0;
    };
    let u = unit.trim().to_ascii_lowercase();
    match u.as_str() {
        "kib" | "kb" => (v * 1024.0) as u64,
        "mib" | "mb" => (v * 1024.0 * 1024.0) as u64,
        "gib" | "gb" => (v * 1024.0 * 1024.0 * 1024.0) as u64,
        _ => 0,
    }
}

fn java_version_key(file: &str) -> Vec<u64> {
    let base = file.trim_end_matches(".zip");
    let seg = match base.find("hotspot_") {
        Some(i) => &base[i + "hotspot_".len()..],
        None => base,
    };
    seg.split(['.', '_']).filter_map(|p| p.parse::<u64>().ok()).collect()
}

async fn fetch_java_download_from_mirror(major_version: u32, os: &str, ach: &str) -> Option<JavaDownload> {
    let client = crate::mc::mirror::http_client();
    for mirror in ADOPTIUM_MIRRORS {
        for image_type in ["jre", "jdk"] {
            let di_ul = format!("{}/{}/{}/{}/{}/", mirror, major_version, image_type, ach, os);
            let esp = match client.get(&di_ul).send().await {
                Ok(esp) if esp.status().is_success() => esp,
                _ => continue,
            };
            let html = match esp.text().await {
                Ok(h) => h,
                Err(_) => continue,
            };
            let entries = parse_mirror_zip_entries(&html);
            if let Some((name, size)) = entries
                .into_iter()
                .max_by(|a, b| java_version_key(&a.0).cmp(&java_version_key(&b.0)))
            {
                return Some(JavaDownload {
                    link: format!("{}{}", di_ul, name),
                    size: size as i64,
                    checksum: String::new(),
                    checksum_type: String::new(),
                });
            }
        }
    }
    None
}

async fn fetch_java_download(major_version: u32, os: &str, ach: &str) -> Option<JavaDownload> {
    let official = fetch_java_download_official(major_version, os, ach);
    let mirror = fetch_java_download_from_mirror(major_version, os, ach);
    tokio::pin!(official);
    tokio::pin!(mirror);

    let first = tokio::select! {
        o = &mut official => o,
        m = &mut mirror => m,
    };
    if let Some(dl) = first {
        return Some(dl);
    }
    if let Some(dl) = official.await {
        return Some(dl);
    }
    if let Some(dl) = mirror.await {
        return Some(dl);
    }
    None
}

fn java_download_candidates(download: &JavaDownload, major_version: u32, os: &str, ach: &str) -> Vec<String> {
    let mut urls = Vec::new();
    if ADOPTIUM_MIRRORS.iter().any(|m| download.link.starts_with(m)) {
        urls.push(download.link.clone());
        return urls;
    }
    if let Some(file) = download.link.split('/').next().filter(|f| f.ends_with(".zip")) {
        let image_type = if file.contains("-jre_") { "jre" } else { "jdk" };
        for mirror in ADOPTIUM_MIRRORS {
            urls.push(format!("{}/{}/{}/{}/{}/{}", mirror, major_version, image_type, ach, os, file));
        }
    }
    urls.push(download.link.clone());
    urls
}

pub fn detect_java_versions() -> Vec<JavaInfo> {
    let mut results = Vec::new();
    let mut seen_paths: Vec<String> = Vec::new();

    if let Ok(java_home) = std::env::var("JAVA_HOME") {
        let path = PathBuf::from(&java_home).join("bin").join("java");
        if path.exists() {
            if let Some(info) = probe_java(path.to_str().unwrap()) {
                if !seen_paths.contains(&info.path) {
                    seen_paths.push(info.path.clone());
                    results.push(info);
                }
            }
        } else {
            let path_exe = PathBuf::from(&java_home).join("bin").join("java.exe");
            if path_exe.exists() {
                if let Some(info) = probe_java(path_exe.to_str().unwrap()) {
                    if !seen_paths.contains(&info.path) {
                        seen_paths.push(info.path.clone());
                        results.push(info);
                    }
                }
            }
        }
    }

    if let Ok(path) = std::env::var("PATH") {
        for di in std::env::split_paths(&path) {
            let java = di.join("java");
            let java_exe = di.join("java.exe");
            if java_exe.exists() {
                if let Some(info) = probe_java(java_exe.to_str().unwrap()) {
                    if !seen_paths.contains(&info.path) {
                        seen_paths.push(info.path.clone());
                        results.push(info);
                    }
                }
            } else if java.exists() {
                if let Some(info) = probe_java(java.to_str().unwrap()) {
                    if !seen_paths.contains(&info.path) {
                        seen_paths.push(info.path.clone());
                        results.push(info);
                    }
                }
            }
        }
    }

    let java_dir = io::get_java_dir();
    if java_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&java_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let java_bin = if cfg!(target_os = "windows") {
                        path.join("bin").join("java.exe")
                    } else {
                        path.join("bin").join("java")
                    };
                    if java_bin.exists() {
                        if let Some(info) = probe_java(java_bin.to_str().unwrap()) {
                            if !seen_paths.contains(&info.path) {
                                seen_paths.push(info.path.clone());
                                results.push(info);
                            }
                        }
                    }
                }
            }
        }
    }

    for di_st in COMMON_JAVA_DIRS {
        let di = PathBuf::from(di_st);
        if !di.exists() {
            continue;
        }
        if let Ok(entries) = std::fs::read_dir(&di) {
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                let java_candidates = if cfg!(target_os = "macos") {
                    vec![
                        path.join("Contents/Home/bin/java"),
                        path.join("bin/java"),
                    ]
                } else {
                    vec![
                        path.join("bin/java.exe"),
                        path.join("bin/java"),
                    ]
                };
                for java_bin in java_candidates {
                    if java_bin.exists() {
                        if let Some(info) = probe_java(java_bin.to_str().unwrap()) {
                            if !seen_paths.contains(&info.path) {
                                seen_paths.push(info.path.clone());
                                results.push(info);
                            }
                        }
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        for base in [
            std::env::var("USERPROFILE").map(|p| format!("{}\\.jdks", p)).unwrap_or_default(),
            std::env::var("USERPROFILE").map(|p| format!("{}\\scoop\\apps", p)).unwrap_or_default(),
            std::env::var("LOCALAPPDATA").map(|p| format!("{}\\Programs", p)).unwrap_or_default(),
            std::env::var("LOCALAPPDATA").map(|p| format!("{}\\JetBrains\\Toolbox\\apps", p)).unwrap_or_default(),
        ] {
            if base.is_empty() {
                continue;
            }
            scan_dir_for_java(&PathBuf::from(base), &mut results, &mut seen_paths);
        }
    }

    #[cfg(target_os = "windows")]
    {
        for reg_path in WINDOWS_REGISTRY_PATHS {
            if let Ok(entries) = detect_java_from_registy(reg_path) {
                for java_path in entries {
                    if !seen_paths.contains(&java_path) {
                        if let Some(info) = probe_java(&java_path) {
                            seen_paths.push(info.path.clone());
                            results.push(info);
                        }
                    }
                }
            }
        }
    }

    results.sort_by(|a, b| b.major_version.cmp(&a.major_version));

    results
}



use std::sync::OnceLock;
static JAVA_CACHE: OnceLock<Vec<JavaInfo>> = OnceLock::new();

pub fn detect_java_versions_cached() -> Vec<JavaInfo> {
    JAVA_CACHE.get_or_init(|| detect_java_versions()).clone()
}

pub fn invalidate_java_cache() {
    
    
}

fn scan_dir_for_java(base: &std::path::Path, results: &mut Vec<JavaInfo>, seen_paths: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(base) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let name = path.file_name().map(|n| n.to_string_lossy().to_lowercase()).unwrap_or_default();
        let is_java_dir = [
            "jdk", "java", "jre", "temurin", "corretto", "zulu", "microsoft", "openjdk",
        ]
        .iter()
        .any(|k| name.contains(k));
        if !is_java_dir {
            continue;
        }
        for java_bin in [
            path.join("bin").join("java.exe"),
            path.join("bin").join("java"),
        ] {
            if java_bin.exists() {
                if let Some(info) = probe_java(java_bin.to_str().unwrap()) {
                    if !seen_paths.contains(&info.path) {
                        seen_paths.push(info.path.clone());
                        results.push(info);
                    }
                }
                break;
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn detect_java_from_registy(reg_path: &str) -> Result<Vec<String>, String> {
    use winreg::enums::*;
    use winreg::RegKey;

    let mut java_paths = Vec::new();

    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    if let Ok(key) = hklm.open_subkey_with_flags(reg_path, KEY_READ | KEY_ENUMERATE_SUB_KEYS) {
        for subkey_name in key.enum_keys().filter_map(|k| k.ok()) {
            if let Ok(subkey) = key.open_subkey_with_flags(&subkey_name, KEY_READ) {
                if let Ok(java_home) = subkey.get_value::<String, _>("JavaHome") {
                    let java_bin = PathBuf::from(&java_home).join("bin").join("java.exe");
                    if java_bin.exists() {
                        java_paths.push(java_bin.to_string_lossy().to_string());
                    }
                }
                if let Ok(install_path) = subkey.get_value::<String, _>("InstallationPath") {
                    let java_bin = PathBuf::from(&install_path).join("bin").join("java.exe");
                    if java_bin.exists() {
                        java_paths.push(java_bin.to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    Ok(java_paths)
}

pub fn probe_java(java_path: &str) -> Option<JavaInfo> {
    let output = crate::utils::io::no_window(&mut Command::new(java_path))
        .arg("-version")
        .output()
        .ok()?;

    let version_st = String::from_utf8_lossy(&output.stderr);
    let version_line = version_st.lines().next()?;

    let e = regex::Regex::new(r#""(\d+)(?:\.(\d+))?"#).ok()?;
    let majo = if let Some(caps) = e.captures(version_line) {
        let first = caps[1].parse::<u32>().unwrap_or(0);
        let second = caps.get(2).and_then(|m| m.as_str().parse::<u32>().ok());
        if first == 1 && second == Some(8) {
            8
        } else {
            first
        }
    } else {
        0
    };

    let is_64bit = version_st.contains("64-Bit");
    let architecture = if version_st.contains("aarch64") || version_st.contains("ARM64") {
        "aarch64".to_string()
    } else if is_64bit {
        "x64".to_string()
    } else {
        "x86".to_string()
    };

    let vendor = detect_vendor(&version_st);

    let is_jdk = {
        let java_dir = std::path::Path::new(java_path).parent().and_then(|p| p.parent());
        if let Some(di) = java_dir {
            let javac = di.join("bin").join(if cfg!(target_os = "windows") { "javac.exe" } else { "javac" });
            javac.exists()
        } else {
            false
        }
    };

    let clean_version: String = version_line
        .trim()
        .chars()
        .filter(|c| c.is_ascii_graphic() || *c == ' ')
        .collect();

    Some(JavaInfo {
        path: java_path.to_string(),
        version: clean_version,
        major_version: majo,
        is_64bit,
        architecture,
        vendor,
        is_jdk,
    })
}

fn detect_vendor(version_st: &str) -> String {
    if version_st.contains("Eclipse Temurin") || version_st.contains("Adoptium") {
        "Eclipse Temurin".to_string()
    } else if version_st.contains("Zulu") || version_st.contains("Azul") {
        "Azul Zulu".to_string()
    } else if version_st.contains("Amazon Corretto") || version_st.contains("Corretto") {
        "Amazon Corretto".to_string()
    } else if version_st.contains("Microsoft") || version_st.contains("OpenJDK") && version_st.contains("Microsoft") {
        "Microsoft OpenJDK".to_string()
    } else if version_st.contains("BellSoft") || version_st.contains("Liberica") {
        "BellSoft Liberica".to_string()
    } else if version_st.contains("GraalVM") || version_st.contains("graalvm") {
        "GraalVM".to_string()
    } else if version_st.contains("OpenJ9") || version_st.contains("IBM") {
        "IBM OpenJ9".to_string()
    } else if version_st.contains("SapMachine") || version_st.contains("SAP") {
        "SAP SapMachine".to_string()
    } else if version_st.contains("Red Hat") || version_st.contains("OpenJDK") && version_st.contains("Red Hat") {
        "Red Hat OpenJDK".to_string()
    } else if version_st.contains("Mandrel") {
        "Mandrel".to_string()
    } else if version_st.contains("Dragonwell") || version_st.contains("Alibaba") {
        "Alibaba Dragonwell".to_string()
    } else if version_st.contains("Tencent") || version_st.contains("Kona") {
        "Tencent Kona".to_string()
    } else if version_st.contains("Bisheng") || version_st.contains("Huawei") {
        "Huawei Bisheng".to_string()
    } else if version_st.contains("Java(TM) SE") || version_st.contains("Oracle") {
        "Oracle".to_string()
    } else if version_st.contains("OpenJDK") {
        "OpenJDK".to_string()
    } else {
        "Unknown".to_string()
    }
}

pub fn get_required_java_version(minecraft_version: &str) -> u32 {
    let mc = minecraft_version.split(['-', '_']).next().unwrap_or(minecraft_version);
    let mc = mc.trim_start_matches("MC").trim();
    let pats: Vec<&str> = mc.split('.').collect();
    if pats.len() < 2 {
        return 21;
    }
    let majo: u32 = pats[0].parse().unwrap_or(0);
    let mino: u32 = pats[1].parse().unwrap_or(0);
    if majo == 1 {
        if mino >= 21 {
            21
        } else if mino >= 17 {
            17
        } else {
            8
        }
    } else if majo >= 2 {
        if majo >= 25 {
            25
        } else {
            21
        }
    } else {
        8
    }
}

pub fn get_required_java_version_from_profile(version_id: &str) -> Option<u32> {
    let profile_path = crate::utils::io::get_versions_dir()
        .join(version_id)
        .join(format!("{}.json", version_id));
    let json = std::fs::read_to_string(profile_path).ok()?;
    let v: serde_json::Value = serde_json::from_str(&json).ok()?;
    v["javaVersion"]["majorVersion"].as_u64().map(|n| n as u32)
}

pub fn get_java_recommendation(minecraft_version: &str, modloader: Option<&str>) -> JavaRecommendation {
    let required = get_required_java_version(minecraft_version);
    let mut recommended = required;
    let mut reason = String::new();

    if let Some(loader) = modloader {
        match loader.to_lowercase().as_str() {
            "forge" => {
                let mc = minecraft_version.split(['-', '_']).next().unwrap_or(minecraft_version);
                let pats: Vec<&str> = mc.split('.').collect();
                if pats.len() >= 2 {
                    let mino: u32 = pats[1].parse().unwrap_or(0);
                    if mino <= 16 {
                        recommended = 8;
                        reason = "Forge 1.16 及以下版本需要 Java 8".to_string();
                    }
                }
            }
            "fabric" => {
                if required >= 17 {
                    recommended = 17;
                    reason = "Fabric 1.18+ 需要 Java 17+".to_string();
                }
            }
            "optifine" => {
                let mc = minecraft_version.split(['-', '_']).next().unwrap_or(minecraft_version);
                let pats: Vec<&str> = mc.split('.').collect();
                if pats.len() >= 2 {
                    let mino: u32 = pats[1].parse().unwrap_or(0);
                    if mino >= 8 && mino <= 11 {
                        recommended = 8;
                        reason = "OptiFine 1.8-1.11 需要 Java 8".to_string();
                    }
                }
            }
            _ => {}
        }
    }

    if reason.is_empty() {
        reason = format!("Minecraft {} 需要 Java {}+", minecraft_version, required);
    }

    if recommended < required {
        recommended = required;
    }

    let all_java = detect_java_versions();
    let compatible_java: Vec<JavaInfo> = all_java
        .into_iter()
        .filter(|j| j.major_version >= required && j.is_64bit)
        .collect();

    JavaRecommendation {
        minecraft_version: minecraft_version.to_string(),
        required_java: required,
        recommended_java: recommended,
        reason,
        compatible_java,
    }
}

pub fn find_best_java(minecraft_version: &str, modloader: Option<&str>) -> Option<JavaInfo> {
    let recommendation = get_java_recommendation(minecraft_version, modloader);
    let java_versions = detect_java_versions();

    if let Some(j) = java_versions
        .iter()
        .find(|j| j.major_version == recommendation.recommended_java && j.is_64bit)
    {
        return Some(j.clone());
    }
    java_versions
        .iter()
        .filter(|j| j.major_version >= recommendation.required_java && j.is_64bit)
        .min_by_key(|j| j.major_version)
        .cloned()
}

pub fn get_installed_java_versions() -> Vec<JavaInfo> {
    detect_java_versions()
}

pub fn remove_java(java_path: &str) -> Result<(), String> {
    let path = PathBuf::from(java_path);
    if !path.exists() {
        return Ok(());
    }

    let java_dir = io::get_java_dir();
    if path.starts_with(&java_dir) {
        let prent = path.parent()
            .and_then(|p| p.parent())
            .ok_or("Invalid Java path")?;
        if prent.starts_with(&java_dir) {
            std::fs::remove_dir_all(prent).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

pub async fn ensure_java(
    major_version: u32,
    on_progress: impl Fn(String, f64) + Send + Sync + 'static,
) -> Result<String, String> {
    use sha2::Digest;
    use std::sync::Arc;

    if let Some(j) = detect_java_versions()
        .iter()
        .find(|j| j.major_version == major_version && j.is_64bit)
    {
        return Ok(j.path.clone());
    }

    let java_dir = io::get_java_dir();
    std::fs::create_dir_all(&java_dir).map_err(|e| e.to_string())?;
    let target_dir = java_dir.join(format!("jre-{}", major_version));
    let java_exe = target_dir.join("bin").join(if cfg!(target_os = "windows") { "java.exe" } else { "java" });
    if java_exe.exists() {
        return Ok(java_exe.to_string_lossy().to_string());
    }

    let os = if cfg!(target_os = "windows") { "windows" }
        else if cfg!(target_os = "macos") { "mac" }
        else { "linux" };
    let arch = if cfg!(target_arch = "x86_64") { "x64" }
        else if cfg!(target_arch = "aarch64") { "aarch64" }
        else { "x64" };

    let on_progress = Arc::new(on_progress);
    on_progress(format!("正在获取 Java {} 下载信息...", major_version), 0.02);

    let download = fetch_java_download(major_version, os, arch).await.ok_or_else(|| {
        format!(
            "获取 Java {} 下载信息失败: 无法从 Adoptium 及其国内镜像获取下载地址,请检查网络后重试",
            major_version
        )
    })?;
    let candidates = java_download_candidates(&download, major_version, os, arch);
    let from_mirror = candidates.len() > 1;

    on_progress(
        format!(
            "开始下载 Java {} ({:.1} MB, {})...",
            major_version,
            download.size.max(1) as f64 / 1024.0 / 1024.0,
            if from_mirror { "国内镜像" } else { "官方源" }
        ),
        0.05,
    );

    let zip_path = java_dir.join(format!("jre-{}.zip", major_version));
    let mut file = tokio::fs::File::create(&zip_path)
        .await
        .map_err(|e| e.to_string())?;

    let mut downloaderd: u64 = 0;
    let mut last_pct: u32 = 0;
    {
        let client = crate::mc::mirror::http_client();
        let mut resp: Option<reqwest::Response> = None;
        let mut last_err = String::new();
        for (i, url) in candidates.iter().enumerate() {
            match client.get(url).send().await {
                Ok(r) if r.status().is_success() => {
                    resp = Some(r);
                    break;
                }
                Ok(r) => last_err = format!("HTTP {}", r.status()),
                Err(e) => last_err = e.to_string(),
            }
            if i + 1 < candidates.len() {
                on_progress(format!("镜像源不可用, 尝试备用下载源..."), 0.05);
            }
        }
        let resp = resp.ok_or_else(|| format!("下载 Java 失败: {}", last_err))?;
        let total_size = resp
            .content_length()
            .map(|n| n.max(1))
            .unwrap_or(download.size.max(1) as u64);

        let mut stream = resp.bytes_stream();
        use futures::StreamExt;
        use tokio::io::AsyncWriteExt;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|e| format!("下载 Java 中断: {}", e))?;
            file.write_all(&chunk).await.map_err(|e| e.to_string())?;
            downloaderd += chunk.len() as u64;
            let pct = (downloaderd as f64 / total_size as f64 * 0.80) as u32;
            if pct != last_pct {
                last_pct = pct;
                on_progress(
                    format!("正在下载 Java {}... ({:.1}%)", major_version, pct as f64),
                    0.05 + pct as f64 / 100.0 * 0.80,
                );
            }
        }
        file.flush().await.map_err(|e| e.to_string())?;
    }

    let data = std::fs::read(&zip_path).map_err(|e| e.to_string())?;
    let mut hasher = sha2::Sha256::new();
    hasher.update(&data);
    let actual = hex::encode(hasher.finalize());
    if !download.checksum.is_empty() && !actual.eq_ignore_ascii_case(&download.checksum) {
        let _ = std::fs::remove_file(&zip_path);
        return Err(format!("Java {} 下载校验失败，请重试", major_version));
    }

    on_progress(format!("正在解压 Java {}...", major_version), 0.88);

    let extract_dir = java_dir.join(format!("jre-{}-extract", major_version));
    if extract_dir.exists() {
        let _ = std::fs::remove_dir_all(&extract_dir);
    }
    std::fs::create_dir_all(&extract_dir).map_err(|e| e.to_string())?;

    {
        let file = std::fs::File::open(&zip_path).map_err(|e| e.to_string())?;
        let mut archive = zip::ZipArchive::new(file).map_err(|e| format!("解压 Java 失败: {}", e))?;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
            let name = entry.name().to_string();
            let stripped = match name.find('/') {
                Some(idx) => &name[idx + 1..],
                None => name.as_str(),
            };
            if stripped.is_empty() {
                continue;
            }
            let target = extract_dir.join(stripped);
            if entry.is_dir() {
                std::fs::create_dir_all(&target).map_err(|e| e.to_string())?;
            } else {
                if let Some(parent) = target.parent() {
                    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
                }
                let mut out = std::fs::File::create(&target).map_err(|e| e.to_string())?;
                std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
            }
        }
    }

    let _ = std::fs::remove_file(&zip_path);
    if target_dir.exists() {
        let _ = std::fs::remove_dir_all(&target_dir);
    }
    std::fs::rename(&extract_dir, &target_dir).map_err(|e| format!("移动 Java 目录失败: {}", e))?;

    if !java_exe.exists() {
        return Err(format!("Java {} 解压后未找到可执行文件", major_version));
    }

    on_progress(format!("Java {} 安装完成", major_version), 1.0);
    Ok(java_exe.to_string_lossy().to_string())
}
