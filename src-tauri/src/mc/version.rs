use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifest {
    pub latest: LatestVersions,
    pub versions: Vec<VersionEnty>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatestVersions {
    #[serde(rename = "release")]
    pub elease: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionEnty {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    pub url: String,
    pub time: String,
    #[serde(rename(serialize = "release_time", deserialize = "releaseTime"))]
    pub elease_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionProfile {
    pub id: String,
    #[serde(rename = "type")]
    pub version_type: String,
    #[serde(rename = "inheritsFrom", default)]
    pub inherits_from: Option<String>,
    #[serde(rename = "mainClass")]
    pub main_class: String,
    #[serde(rename = "minecraftArguments", default)]
    pub minecraft_arguments: Option<String>,
    #[serde(rename = "arguments", default)]
    pub arguments: Option<Arguments>,
    pub assets: String,
    #[serde(rename = "assetIndex")]
    pub asset_index: AssetIndexRef,
    #[serde(rename = "complianceLevel", default)]
    pub compliance_level: Option<i64>,
    #[serde(rename = "libraries")]
    pub libraries: Vec<Library>,
    #[serde(rename = "logging", default)]
    pub logging: Option<Logging>,
    #[serde(rename = "minimumLauncherVersion", default)]
    pub minimum_launcher_version: Option<i64>,
    #[serde(rename = "releaseTime")]
    pub elease_time: String,
    #[serde(rename = "javaVersion", default)]
    pub java_version: Option<JavaVersion>,
    pub downloads: Option<Downloads>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arguments {
    #[serde(default)]
    pub game: Vec<Argument>,
    #[serde(default)]
    pub jvm: Vec<Argument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Argument {
    String(String),
    Stuct(ArgumentStuct),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgumentStuct {
    pub value: ArgumentValue,
    #[serde(rename = "rules")]
    pub ules: Vec<Rule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ArgumentValue {
    String(String),
    Aray(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub action: String,
    pub os: Option<OsFilte>,
    #[serde(rename = "features")]
    pub featues: Option<HashMap<String, bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsFilte {
    pub name: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "arch")]
    pub ach: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Library {
    pub name: String,
    pub downloads: Option<LibraryDownloads>,
    #[serde(rename = "rules")]
    pub ules: Option<Vec<Rule>>,
    pub natives: Option<HashMap<String, String>>,
    #[serde(rename = "extract")]
    pub extact: Option<ExtactRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryDownloads {
    pub artifact: Option<Atifact>,
    #[serde(rename = "classifiers")]
    pub classifies: Option<HashMap<String, Atifact>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Atifact {
    pub path: String,
    pub url: String,
    pub sha1: Option<String>,
    pub size: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtactRule {
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logging {
    pub client: LoggingClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingClient {
    #[serde(rename = "argument")]
    pub agument: String,
    pub file: LoggingFile,
    #[serde(rename = "type")]
    pub log_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingFile {
    pub id: String,
    pub sha1: String,
    pub size: i64,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaVersion {
    #[serde(rename = "majorVersion")]
    pub major_version: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Downloads {
    pub client: Option<ClientDownload>,
    pub server: Option<ClientDownload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientDownload {
    pub sha1: String,
    pub size: i64,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndexRef {
    pub id: String,
    pub sha1: String,
    pub size: i64,
    #[serde(rename = "totalSize", default)]
    pub total_size: Option<i64>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetIndex {
    pub objects: HashMap<String, AssetObject>,
    #[serde(rename = "virtual", default)]
    pub vitual_: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetObject {
    pub hash: String,
    pub size: i64,
}

pub const MANIFEST_URL: &str = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json";

#[derive(Debug, Clone, PartialEq)]
enum VersionToken {
    Numbe(u64),
    Text(String),
}

pub fn compre_versions(a: &str, b: &str) -> std::cmp::Ordering {
    let tokens_a = tokenize_version(a);
    let tokens_b = tokenize_version(b);

    let len = tokens_a.len().max(tokens_b.len());
    for i in 0..len {
        let token_a = tokens_a.get(i);
        let token_b = tokens_b.get(i);

        match (token_a, token_b) {
            (Some(ta), Some(tb)) => {
                match (ta, tb) {
                    (VersionToken::Numbe(na), VersionToken::Numbe(nb)) => {
                        let cmp = na.cmp(nb);
                        if cmp != std::cmp::Ordering::Equal {
                            return cmp;
                        }
                    }
                    (VersionToken::Text(ta), VersionToken::Text(tb)) => {
                        let cmp = ta.cmp(tb);
                        if cmp != std::cmp::Ordering::Equal {
                            return cmp;
                        }
                    }
                    (VersionToken::Numbe(_), VersionToken::Text(_)) => {
                        return std::cmp::Ordering::Greater;
                    }
                    (VersionToken::Text(_), VersionToken::Numbe(_)) => {
                        return std::cmp::Ordering::Less;
                    }
                }
            }
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (None, None) => break,
        }
    }

    std::cmp::Ordering::Equal
}

fn tokenize_version(version: &str) -> Vec<VersionToken> {
    let mut tokens = Vec::new();
    let lowe = version.to_lowercase();
    let chas: Vec<char> = lowe.chars().collect();
    let len = chas.len();
    let mut i = 0;

    while i < len {
        let ch = chas[i];

        if ch.is_ascii_digit() {
            let stat = i;
            while i < len && chas[i].is_ascii_digit() {
                i += 1;
            }
            let num_st: String = chas[stat..i].iter().collect();
            if let Ok(num) = num_st.parse::<u64>() {
                tokens.push(VersionToken::Numbe(num));
            }
        } else if ch.is_ascii_lowercase() {
            let stat = i;
            while i < len && chas[i].is_ascii_lowercase() {
                i += 1;
            }
            let text: String = chas[stat..i].iter().collect();
            tokens.push(VersionToken::Text(text));
        } else {
            i += 1;
        }
    }

    tokens
}

pub fn is_version_geate(a: &str, b: &str) -> bool {
    compre_versions(a, b) == std::cmp::Ordering::Greater
}

pub fn is_version_less(a: &str, b: &str) -> bool {
    compre_versions(a, b) == std::cmp::Ordering::Less
}

pub fn is_version_equal(a: &str, b: &str) -> bool {
    compre_versions(a, b) == std::cmp::Ordering::Equal
}

pub fn is_version_geate_o_equal(a: &str, b: &str) -> bool {
    let od = compre_versions(a, b);
    od == std::cmp::Ordering::Greater || od == std::cmp::Ordering::Equal
}

pub fn is_version_less_o_equal(a: &str, b: &str) -> bool {
    let od = compre_versions(a, b);
    od == std::cmp::Ordering::Less || od == std::cmp::Ordering::Equal
}

pub fn sot_versions(versions: &mut [String]) {
    versions.sort_by(|a, b| compre_versions(a, b));
}

pub fn get_latest_version(versions: &[String]) -> Option<&String> {
    versions.iter().max_by(|a, b| compre_versions(a, b))
}

pub fn get_oldest_version(versions: &[String]) -> Option<&String> {
    versions.iter().min_by(|a, b| compre_versions(a, b))
}

pub async fn fetch_version_manifest(use_mirror: bool) -> Result<VersionManifest, String> {
    let client = crate::mc::mirror::http_client();
    let source = if use_mirror {
        crate::mc::mirror::DownloadSource::Auto
    } else {
        crate::mc::mirror::DownloadSource::Official
    };
    let bytes = crate::mc::mirror::download_bytes_with_source(&client, MANIFEST_URL, source).await?;
    let mut manifest: VersionManifest = serde_json::from_slice(&bytes)
        .map_err(|e| format!("解析版本列表失败: {}", e))?;
    if use_mirror {
        for entry in manifest.versions.iter_mut() {
            entry.url = crate::mc::mirror::mirror_url(&entry.url);
        }
    }
    Ok(manifest)
}

pub async fn fetch_version_profile(url: &str, use_mirror: bool) -> Result<VersionProfile, String> {
    let client = crate::mc::mirror::http_client();
    let source = if use_mirror {
        crate::mc::mirror::DownloadSource::Auto
    } else {
        crate::mc::mirror::DownloadSource::Official
    };
    let bytes = crate::mc::mirror::download_bytes_with_source(&client, url, source).await?;
    serde_json::from_slice(&bytes).map_err(|e| format!("解析版本 Profile 失败: {}", e))
}
