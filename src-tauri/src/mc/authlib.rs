use serde::{Deserialize, Serialize};
use reqwest::Client;
use crate::mc::auth::AuthSession;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthlibInjectoServer {
    pub name: String,
    pub url: String,
    pub registe_ul: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthlibProfile {
    pub id: String,
    pub name: String,
    pub properties: Option<Vec<AuthlibProperty>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthlibProperty {
    pub name: String,
    pub value: String,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthlibAuthenticateResponse {
    pub access_token: String,
    pub client_token: String,
    pub available_profiles: Vec<AuthlibProfile>,
    pub selected_profile: Option<AuthlibProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthlibRefeshResponse {
    pub access_token: String,
    pub client_token: String,
    pub selected_profile: AuthlibProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TexturesInfo {
    pub textures: TexturesData,
}

#[allow(non_snake_case)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TexturesData {
    #[serde(default)]
    pub SKIN: Option<TextureEntry>,
    #[serde(default)]
    pub CAPE: Option<TextureEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureEntry {
    pub url: String,
    #[serde(default)]
    pub metadata: Option<TextureMetadata>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureMetadata {
    #[serde(default)]
    pub model: Option<String>,
}

pub fn get_authlib_jar_path() -> PathBuf {
    let launcher_root = crate::utils::io::get_launcher_root();
    launcher_root.join("authlib-injector.jar")
}

pub fn is_authlib_jar_downloaded() -> bool {
    get_authlib_jar_path().exists()
}



pub async fn ensure_authlib_jar() -> Result<PathBuf, String> {
    let jar_path = get_authlib_jar_path();
    if jar_path.exists() {
        return Ok(jar_path);
    }

    
    let launcher_root = crate::utils::io::get_launcher_root();
    let bundled = launcher_root.join("resources").join("authlib-injector.jar");
    if bundled.exists() {
        if let Some(prent) = jar_path.parent() {
            std::fs::create_dir_all(prent).map_err(|e| e.to_string())?;
        }
        std::fs::copy(&bundled, &jar_path).map_err(|e| format!("复制 authlib-injector 失败: {}", e))?;
        return Ok(jar_path);
    }

    download_authlib_jar().await
}

pub async fn download_authlib_jar() -> Result<PathBuf, String> {
    let jar_path = get_authlib_jar_path();
    if jar_path.exists() {
        return Ok(jar_path);
    }

    let client = reqwest::Client::new();
    let url = "https://github.com/yushijinhun/authlib-injector/releases/download/v1.2.5/authlib-injector-1.2.5.jar";
    let esp = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("下载 authlib-injector 失败: {}", e))?;

    if !esp.status().is_success() {
        return Err(format!("下载 authlib-injector 失败: HTTP {}", esp.status()));
    }

    let bytes = esp.bytes().await
        .map_err(|e| format!("读取 authlib-injector 失败: {}", e))?;

    if let Some(prent) = jar_path.parent() {
        std::fs::create_dir_all(prent).map_err(|e| e.to_string())?;
    }

    std::fs::write(&jar_path, &bytes).map_err(|e| format!("保存 authlib-injector 失败: {}", e))?;
    Ok(jar_path)
}

pub fn get_authlib_jvm_args(server_ul: &str, pefetched_meta: Option<&str>) -> Vec<String> {
    let jar_path = get_authlib_jar_path();
    let mut ags = vec![
        format!("-javaagent:{}={}", jar_path.to_string_lossy(), server_ul),
        "-Dauthlibinjector.side=client".to_string(),
    ];
    if let Some(meta) = pefetched_meta {
        ags.push(format!("-Dauthlibinjector.yggdrasil.prefetched={}", meta));
    }
    ags
}

pub async fn authlib_authenticate(
    server_ul: &str,
    username: &str,
    password: &str,
    client_token: Option<&str>,
) -> Result<AuthSession, String> {
    let client = Client::new();
    let url = format!("{}/authserver/authenticate", server_ul.trim_end_matches('/'));
    
    let body = serde_json::json!({
        "agent": {
            "name": "Minecraft",
            "version": 1
        },
        "username": username,
        "password": password,
        "clientToken": client_token.unwrap_or("skyline-launcher"),
        "requestUser": true
    });

    let esp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Authlib request failed: {}", e))?;

    if !esp.status().is_success() {
        let status = esp.status();
        let text = esp.text().await.unwrap_or_default();
        return Err(format!("Authlib auth failed ({}): {}", status, text));
    }

    let data: AuthlibAuthenticateResponse = esp.json().await
        .map_err(|e| format!("Authlib parse error: {}", e))?;

    let profile = data.selected_profile
        .or_else(|| data.available_profiles.into_iter().next())
        .ok_or("No profile available")?;

    Ok(AuthSession {
        access_token: data.access_token,
        username: profile.name,
        uuid: profile.id,
        user_type: "authlib".to_string(),
        refresh_token: None,
        expires_at: None,
    })
}

pub async fn authlib_efesh(
    server_ul: &str,
    access_token: &str,
    client_token: &str,
) -> Result<AuthSession, String> {
    let client = Client::new();
    let url = format!("{}/authserver/refresh", server_ul.trim_end_matches('/'));
    
    let body = serde_json::json!({
        "accessToken": access_token,
        "clientToken": client_token,
        "requestUser": true
    });

    let esp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Authlib refresh failed: {}", e))?;

    if !esp.status().is_success() {
        return Err("Authlib refresh failed".to_string());
    }

    let data: AuthlibRefeshResponse = esp.json().await
        .map_err(|e| format!("Authlib refresh parse error: {}", e))?;

    Ok(AuthSession {
        access_token: data.access_token,
        username: data.selected_profile.name,
        uuid: data.selected_profile.id,
        user_type: "authlib".to_string(),
        refresh_token: None,
        expires_at: None,
    })
}

pub async fn authlib_validate(server_ul: &str, access_token: &str, client_token: &str) -> Result<bool, String> {
    let client = Client::new();
    let url = format!("{}/authserver/validate", server_ul.trim_end_matches('/'));
    
    let body = serde_json::json!({
        "accessToken": access_token,
        "clientToken": client_token
    });

    let esp = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Authlib validate failed: {}", e))?;

    Ok(esp.status().is_success())
}

pub async fn authlib_invalidate(server_ul: &str, access_token: &str, client_token: &str) -> Result<(), String> {
    let client = Client::new();
    let url = format!("{}/authserver/invalidate", server_ul.trim_end_matches('/'));
    
    let body = serde_json::json!({
        "accessToken": access_token,
        "clientToken": client_token
    });

    client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Authlib invalidate failed: {}", e))?;

    Ok(())
}

pub async fn fetch_authlib_textures(server_ul: &str, uuid: &str) -> Result<(Option<String>, Option<String>), String> {
    let client = Client::new();
    
    let uuid_clean = uuid.replace("-", "");
    let url = format!("{}/sessionserver/session/minecraft/profile/{}",
        server_ul.trim_end_matches('/'), uuid_clean);

    let esp = client.get(&url).send().await
        .map_err(|e| format!("Authlib skin fetch failed: {}", e))?;

    if !esp.status().is_success() {
        return Ok((None, None));
    }

    #[derive(Deserialize)]
    struct ProfileResponse {
        properties: Vec<AuthlibProperty>,
    }

    let profile: ProfileResponse = esp.json().await
        .map_err(|e| format!("Authlib skin parse error: {}", e))?;

    let Some(pop) = profile.properties.iter().find(|p| p.name == "textures") else {
        return Ok((None, None));
    };

    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&pop.value)
        .map_err(|e| format!("Base64 decode error: {}", e))?;
    
    let textures: TexturesInfo = serde_json::from_slice(&decoded)
        .map_err(|e| format!("Textures parse error: {}", e))?;

    let skin = if let Some(skin_entry) = &textures.textures.SKIN {
        let skin_esp = client.get(&skin_entry.url).send().await
            .map_err(|e| format!("Skin download failed: {}", e))?;
        let bytes = skin_esp.bytes().await
            .map_err(|e| format!("Skin bytes read failed: {}", e))?;
        Some(base64::engine::general_purpose::STANDARD.encode(bytes.as_ref()))
    } else {
        None
    };

    let cape = if let Some(cape_entry) = &textures.textures.CAPE {
        let cape_esp = client.get(&cape_entry.url).send().await
            .map_err(|e| format!("Cape download failed: {}", e))?;
        let bytes = cape_esp.bytes().await
            .map_err(|e| format!("Cape bytes read failed: {}", e))?;
        Some(base64::engine::general_purpose::STANDARD.encode(bytes.as_ref()))
    } else {
        None
    };

    Ok((skin, cape))
}

pub async fn fetch_authlib_skin(server_ul: &str, uuid: &str) -> Result<Option<String>, String> {
    let (skin, _) = fetch_authlib_textures(server_ul, uuid).await?;
    Ok(skin)
}

pub async fn get_authlib_server_meta(server_ul: &str) -> Result<AuthlibInjectoServer, String> {
    let client = Client::new();
    let url = format!("{}/api/yggdrasil", server_ul.trim_end_matches('/'));

    let esp = client.get(&url).send().await
        .map_err(|e| format!("Authlib meta fetch failed: {}", e))?;

    if !esp.status().is_success() {
        return Err("Failed to fetch server metadata".to_string());
    }

    #[derive(Deserialize)]
    struct MetaResponse {
        meta: MetaInfo,
    }

    #[derive(Deserialize)]
    struct MetaInfo {
        #[serde(rename = "serverName")]
        server_name: Option<String>,
        #[serde(rename = "registerUrl")]
        registe_ul: Option<String>,
    }

    let data: MetaResponse = esp.json().await
        .map_err(|e| format!("Authlib meta parse error: {}", e))?;

    Ok(AuthlibInjectoServer {
        name: data.meta.server_name.unwrap_or_else(|| "Authlib Injector Server".to_string()),
        url: server_ul.to_string(),
        registe_ul: data.meta.registe_ul,
    })
}
