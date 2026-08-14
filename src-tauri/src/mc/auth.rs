use serde::{Deserialize, Serialize};

use oauth2::basic::{BasicClient, BasicErrorResponse};
use oauth2::{
    AuthUrl, ClientId, DeviceAuthorizationUrl, DeviceCodeErrorResponse, DeviceCodeErrorResponseType,
    EndpointNotSet, EndpointSet, HttpClientError, RefreshToken, RequestTokenError, Scope,
    StandardDeviceAuthorizationResponse, TokenResponse, TokenUrl,
};
use minecraft_msa_auth::{MinecraftAuthorizationError, MinecraftAuthorizationFlow};

struct MsaHttpClient(reqwest::Client);

#[maybe_async::async_impl]
impl minecraft_msa_auth::HttpClient for MsaHttpClient {
    type Error = reqwest::Error;

    async fn call(
        &self,
        request: minecraft_msa_auth::HttpRequest,
    ) -> Result<minecraft_msa_auth::HttpResponse, Self::Error> {
        let response = self.0.execute(reqwest::Request::try_from(request)?).await?;
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.bytes().await?.to_vec();
        let mut esp = http::Response::new(body);
        *esp.status_mut() = status;
        *esp.headers_mut() = headers;
        Ok(esp)
    }
}

type MsOauthClient = BasicClient<
    EndpointSet,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Account {
    pub id: String,
    pub username: String,
    pub auth_type: AuthType,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub uuid: Option<String>,
    pub authlib_server_ul: Option<String>,
    pub nide_server_id: Option<String>,
    pub client_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuthType {
    Microsoft,
    Mojang,
    Offline,
    AuthlibInjecto,
    Nide,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSession {
    pub access_token: String,
    pub username: String,
    pub uuid: String,
    pub user_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<i64>,
}


pub fn offline_uuid(username: &str) -> String {
    use md5::Digest;
    let mut hashe = md5::Md5::new();
    hashe.update(format!("OfflinePlayer:{}", username).as_bytes());
    let mut bytes = hashe.finalize();
    bytes[6] = (bytes[6] & 0x0f) | 0x30; 
    bytes[8] = (bytes[8] & 0x3f) | 0x80; 
    let uuid_bytes: [u8; 16] = bytes.into();
    uuid::Uuid::from_bytes(uuid_bytes).to_string()
}

pub fn offline_auth(username: &str) -> AuthSession {
    let uuid = offline_uuid(username);
    AuthSession {
        access_token: "0".to_string(),
        username: username.to_string(),
        uuid,
        user_type: "offline".to_string(),
        refresh_token: None,
        expires_at: None,
    }
}






pub async fn fetch_skin_textures(uuid: &str) -> Result<(Option<String>, Option<String>), String> {
    let client = crate::mc::mirror::http_client();

    let uuid_stripped = uuid.replace("-", "");
    let profile_url = format!(
        "https://sessionserver.mojang.com/session/minecraft/profile/{}",
        uuid_stripped
    );
    let mirror_url = crate::mc::mirror::mirror_url(&profile_url);

    #[derive(Deserialize)]
    struct ProfileResponse {
        properties: Vec<ProfileProperty>,
    }
    #[derive(Deserialize)]
    struct ProfileProperty {
        name: String,
        value: String,
    }

    async fn fetch_profile(
        client: &reqwest::Client,
        url: &str,
    ) -> Option<String> {
        let resp = client.get(url).send().await.ok()?;
        if !resp.status().is_success() {
            return None;
        }
        let profile: ProfileResponse = resp.json().await.ok()?;
        profile
            .properties
            .iter()
            .find(|p| p.name == "textures")
            .map(|p| p.value.clone())
    }




    let prop_value = {
        let mirror_attempt = tokio::time::timeout(
            std::time::Duration::from_secs(4),
            fetch_profile(&client, &mirror_url),
        )
        .await;
        match mirror_attempt {
            Ok(Some(v)) => Some(v),
            _ => fetch_profile(&client, &profile_url).await,
        }
    };
    let Some(prop_value) = prop_value else {
        return Ok((None, None));
    };

    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(&prop_value)
        .map_err(|e| e.to_string())?;
    let textures: serde_json::Value = serde_json::from_slice(&decoded).map_err(|e| e.to_string())?;

    async fn fetch_texture(client: &reqwest::Client, url: &str) -> Option<String> {
        let resp = client.get(url).send().await.ok()?;
        let bytes = resp.bytes().await.ok()?;
        Some(base64::engine::general_purpose::STANDARD.encode(bytes.as_ref()))
    }

    let skin_url = textures["textures"]["SKIN"]["url"].as_str();
    let cape_url = textures["textures"]["CAPE"]["url"].as_str();

    let (skin, cape) = match (skin_url, cape_url) {
        (Some(s), Some(c)) => tokio::join!(fetch_texture(&client, s), fetch_texture(&client, c)),
        (Some(s), None) => (fetch_texture(&client, s).await, None),
        (None, Some(c)) => (None, fetch_texture(&client, c).await),
        (None, None) => (None, None),
    };

    Ok((skin, cape))
}

pub async fn fetch_skin_head_base64(uuid: &str) -> Result<Option<String>, String> {
    let (skin, _) = fetch_skin_textures(uuid).await?;
    Ok(skin)
}

pub async fn mojang_auth(email: &str, password: &str) -> Result<AuthSession, String> {
    let client = crate::mc::mirror::http_client();
    let body = serde_json::json!({
        "agent": { "name": "Minecraft", "version": 1 },
        "username": email,
        "password": password,
    });

    let esp = client
        .post("https://authserver.mojang.com/authenticate")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !esp.status().is_success() {
        return Err(format!("Auth failed: {}", esp.status()));
    }

    #[derive(Deserialize)]
    struct MojangResponse {
        accessToken: String,
        selectedProfile: Option<MojangProfile>,
    }
    #[derive(Deserialize)]
    struct MojangProfile {
        name: String,
        id: String,
    }

    let data: MojangResponse = esp.json().await.map_err(|e| format!("Parse error: {}", e))?;
    let profile = data.selectedProfile.ok_or("No Minecraft profile")?;

    Ok(AuthSession {
        access_token: data.accessToken,
        username: profile.name,
        uuid: profile.id,
        user_type: "mojang".to_string(),
        refresh_token: None,
        expires_at: None,
    })
}

fn microsoft_client_id() -> String {
    std::env::var("SKYLINE_MS_CLIENT_ID").unwrap_or_else(|_| "0ea17b32-74f8-473a-82ee-30952aa99698".to_string())
}

fn ms_oauth_client() -> Result<MsOauthClient, String> {
    Ok(BasicClient::new(ClientId::new(microsoft_client_id()))
        .set_auth_uri(
            AuthUrl::new(
                "https://login.microsoftonline.com/consumers/oauth2/v2.0/authorize".to_string(),
            )
            .map_err(|e| e.to_string())?,
        )
        .set_token_uri(
            TokenUrl::new("https://login.microsoftonline.com/consumers/oauth2/v2.0/token".to_string())
                .map_err(|e| e.to_string())?,
        )
        .set_device_authorization_url(
            DeviceAuthorizationUrl::new(
                "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode".to_string(),
            )
            .map_err(|e| e.to_string())?,
        ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicrosoftDeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    
    #[serde(default)]
    pub verification_uri_complete: Option<String>,
    pub interval: u64,
    pub expires_in: u64,
}

pub async fn microsoft_auth_start() -> Result<MicrosoftDeviceCode, String> {
    let http_client = crate::mc::mirror::http_client();
    let client = ms_oauth_client()?;

    let details: StandardDeviceAuthorizationResponse = client
        .exchange_device_code()
        .add_scope(Scope::new("XboxLive.signin offline_access".to_string()))
        .request_async(&http_client)
        .await
        .map_err(map_basic_error)?;

    Ok(MicrosoftDeviceCode {
        device_code: details.device_code().secret().to_string(),
        user_code: details.user_code().secret().to_string(),
        verification_uri: details.verification_uri().as_str().to_string(),
        verification_uri_complete: details
            .verification_uri_complete()
            .map(|v| v.secret().to_string()),
        interval: details.interval().as_secs().max(5),
        expires_in: details.expires_in().as_secs().max(60),
    })
}

pub async fn microsoft_auth_poll(info: MicrosoftDeviceCode) -> Result<AuthSession, String> {
    let http_client = crate::mc::mirror::http_client();
    let client = ms_oauth_client()?;

    
    let details: StandardDeviceAuthorizationResponse = serde_json::from_value(serde_json::json!({
        "device_code": info.device_code,
        "user_code": info.user_code,
        "verification_uri": info.verification_uri,
        "expires_in": info.expires_in,
        "interval": info.interval,
    }))
    .map_err(|e| format!("设备码信息无效: {}", e))?;

    let token = client
        .exchange_device_access_token(&details)
        .request_async(&http_client, tokio::time::sleep, None)
        .await
        .map_err(map_device_ero)?;

    let access_token = token.access_token().secret().to_string();
    let refresh_token = token.refresh_token().map(|r| r.secret().to_string());

    let (mc_token, uuid, name) = exchange_microsoft_tokens(&http_client, &access_token).await?;

    Ok(AuthSession {
        access_token: mc_token,
        username: name,
        uuid,
        user_type: "msa".to_string(),
        refresh_token,
        expires_at: Some((std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64) + 3600_000),
    })
}

async fn exchange_microsoft_tokens(
    client: &reqwest::Client,
    microsoft_access_token: &str,
) -> Result<(String, String, String), String> {
    
    let flow = MinecraftAuthorizationFlow::new(MsaHttpClient(client.clone()));
    let mc_esp = flow
        .exchange_microsoft_token(microsoft_access_token)
        .await
        .map_err(map_msa_ero)?;

    let mc_token = mc_esp.access_token().as_ref().to_string();

    let profile_esp = client
        .get("https://api.minecraftservices.com/minecraft/profile")
        .header("Authorization", format!("Bearer {}", mc_token))
        .send()
        .await
        .map_err(|e| format!("档案获取失败: {}", e))?;

    if !profile_esp.status().is_success() {
        return Err(match profile_esp.status().as_u16() {
            404 => "该微软账户没有购买 Minecraft,无法登录".to_string(),
            _ => format!("档案获取失败: HTTP {}", profile_esp.status()),
        });
    }

    #[derive(Deserialize)]
    struct McProfile {
        id: String,
        name: String,
    }

    let profile: McProfile = profile_esp.json().await.map_err(|e| format!("档案解析失败: {}", e))?;

    Ok((mc_token, profile.id, profile.name))
}

fn map_msa_ero(er: MinecraftAuthorizationError<reqwest::Error>) -> String {
    use minecraft_msa_auth::MinecraftAuthorizationError;
    match er {
        MinecraftAuthorizationError::Http(e) => format!("网络请求失败: {}", e),
        MinecraftAuthorizationError::HttpStatus(s) => match s.as_u16() {
            429 => "登录尝试过于频繁,请稍后再试".to_string(),
            403 => "访问被拒绝,请检查网络环境(可能被限制访问 Minecraft 服务)".to_string(),
            503 => "Minecraft 服务器暂时不可用,请稍后再试".to_string(),
            401 => "微软令牌无效或已过期,请重新登录".to_string(),
            n => format!("Minecraft 登录失败: HTTP {}", n),
        },
        MinecraftAuthorizationError::HttpStatusAtStep { step, status } => match status.as_u16() {
            403 => match step {
                "xsts" => "Xbox 安全令牌授权被拒绝(HTTP 403)。该账号可能未注册 Xbox Live,或地区网络限制访问 Xbox 服务,请开启代理/VPN 后重试。".to_string(),
                "minecraft" => "Minecraft 服务拒绝访问(HTTP 403)。该微软账号可能未购买/拥有 Java 版 Minecraft,或网络被限制访问 Minecraft 服务,请开启代理/VPN 后重试。".to_string(),
                _ => "访问被拒绝,请检查网络环境(可能被限制访问 Minecraft 服务)".to_string(),
            },
            401 => "微软令牌无效或已过期,请重新登录".to_string(),
            429 => "登录尝试过于频繁,请稍后再试".to_string(),
            503 => "Minecraft 服务器暂时不可用,请稍后再试".to_string(),
            n => format!("{} 步骤失败: HTTP {}", step_name(step), n),
        },
        MinecraftAuthorizationError::Json(e) => format!("响应解析失败: {}", e),
        MinecraftAuthorizationError::AddToFamily => "该账户为未成年人账户,需要家长将其加入家庭组后才能登录".to_string(),
        MinecraftAuthorizationError::NoXbox => "该微软账户尚未注册 Xbox Live,请先在 Xbox 官网注册后再登录".to_string(),
        MinecraftAuthorizationError::MissingClaims => "Xbox Live 响应缺少必要声明".to_string(),
    }
}

fn step_name(step: &str) -> &str {
    match step {
        "xbl" => "Xbox Live 认证",
        "xsts" => "Xbox 安全令牌授权",
        "minecraft" => "Minecraft 服务",
        othe => othe,
    }
}

fn map_basic_error(
    er: RequestTokenError<HttpClientError<reqwest::Error>, BasicErrorResponse>,
) -> String {
    match er {
        RequestTokenError::ServerResponse(esp) => format!("微软服务器返回错误: {}", esp),
        RequestTokenError::Request(e) => format!("网络请求失败: {}", e),
        RequestTokenError::Parse(e, _) => format!("响应解析失败: {}", e),
        RequestTokenError::Other(msg) => msg,
    }
}

fn map_device_ero(
    er: RequestTokenError<HttpClientError<reqwest::Error>, DeviceCodeErrorResponse>,
) -> String {
    match er {
        RequestTokenError::ServerResponse(esp) => match esp.error() {
            DeviceCodeErrorResponseType::AccessDenied => "用户拒绝了授权".to_string(),
            DeviceCodeErrorResponseType::ExpiredToken => "授权已过期,请重新开始登录".to_string(),
            DeviceCodeErrorResponseType::SlowDown => "请求过于频繁,已放慢轮询".to_string(),
            DeviceCodeErrorResponseType::AuthorizationPending => "等待用户授权...".to_string(),
            othe => format!("登录失败: {}", othe.as_ref()),
        },
        RequestTokenError::Request(e) => format!("网络请求失败: {}", e),
        RequestTokenError::Parse(e, _) => format!("响应解析失败: {}", e),
        RequestTokenError::Other(msg) => msg,
    }
}

pub async fn microsoft_efesh(refresh_token: &str) -> Result<AuthSession, String> {
    let http_client = crate::mc::mirror::http_client();
    let client = ms_oauth_client()?;

    let token = client
        .exchange_refresh_token(&RefreshToken::new(refresh_token.to_string()))
        .add_scope(Scope::new("XboxLive.signin offline_access".to_string()))
        .request_async(&http_client)
        .await
        .map_err(map_basic_error)?;

    let access_token = token.access_token().secret().to_string();
    let new_refresh_token = token.refresh_token().map(|r| r.secret().to_string());

    let (mc_token, uuid, name) = exchange_microsoft_tokens(&http_client, &access_token).await?;

    Ok(AuthSession {
        access_token: mc_token,
        username: name,
        uuid,
        user_type: "msa".to_string(),
        refresh_token: new_refresh_token,
        expires_at: Some((std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64) + 3600_000),
    })
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LittleSkinDeviceCode {
    pub device_code: String,
    pub user_code: String,
    pub verification_uri: String,
    #[serde(default)]
    pub verification_uri_complete: Option<String>,
    pub interval: u64,
    pub expires_in: u64,
}

fn littleskin_client_id() -> String {
    std::env::var("SKYLINE_LITTLE_SKIN_CLIENT_ID")
        .unwrap_or_else(|_| "1497".to_string())
}

const LITTLE_SKIN_SCOPE: &str = "openid offline_access Yggdrasil.PlayerProfiles.Select Yggdrasil.MinecraftToken.Create Yggdrasil.Server.Join";
const LITTLE_SKIN_DEVICE_API: &str = "https://open.littleskin.cn/oauth/device_code";
const LITTLE_SKIN_TOKEN_API: &str = "https://open.littleskin.cn/oauth/token";
const LITTLE_SKIN_MC_TOKEN_API: &str = "https://littleskin.cn/api/yggdrasil/authserver/oauth";
const LITTLE_SKIN_YGGDRASIL: &str = "https://littleskin.cn/api/yggdrasil";

pub async fn littleskin_auth_status() -> Result<LittleSkinDeviceCode, String> {
    let client = crate::mc::mirror::http_client();
    let params = [
        ("client_id", littleskin_client_id()),
        ("scope", LITTLE_SKIN_SCOPE.to_string()),
    ];
    let resp = client
        .post(LITTLE_SKIN_DEVICE_API)
        .header("Accept", "application/json")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("请求设备码失败: {}", e))?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("请求设备码失败 (HTTP {}): {}", status, text));
    }
    serde_json::from_str(&text).map_err(|e| format!("解析设备码失败: {}", e))
}

fn decode_jwt_payload(token: &str) -> Result<serde_json::Value, String> {
    use base64::Engine;
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() < 2 {
        return Err("ID 令牌格式无效".into());
    }
    let mut b64 = parts[1].to_string();
    let remainder = b64.len() % 4;
    if remainder == 2 {
        b64.push_str("==");
    } else if remainder == 3 {
        b64.push('=');
    }
    let bytes = base64::engine::general_purpose::URL_SAFE
        .decode(&b64)
        .map_err(|e| format!("ID 令牌解码失败: {}", e))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("ID 令牌解析失败: {}", e))
}

fn littleskin_save_authlib_config() -> Result<(), String> {
    let config_di = crate::utils::io::get_launcher_root().join(".skyline");
    std::fs::create_dir_all(&config_di).map_err(|e| e.to_string())?;
    let config = serde_json::json!({
        "server_url": LITTLE_SKIN_YGGDRASIL,
    });
    std::fs::write(
        config_di.join("authlib.json"),
        serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

fn littleskin_build_session(
    mc_token: &str,
    uuid: &str,
    name: &str,
    oauth_refresh_token: Option<String>,
    expires_in: u64,
) -> AuthSession {
    AuthSession {
        access_token: mc_token.to_string(),
        username: name.to_string(),
        uuid: uuid.to_string(),
        user_type: "authlib".to_string(),
        refresh_token: oauth_refresh_token,
        expires_at: Some(
            (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as i64)
                + (expires_in.max(60) * 1000) as i64,
        ),
    }
}

async fn littleskin_exchange_mc_token(
    client: &reqwest::Client,
    oauth_access_token: &str,
    uuid: &str,
) -> Result<(String, String, String), String> {
    #[derive(Deserialize)]
    struct McTokenResp {
        accessToken: String,
        #[serde(default)]
        selectedProfile: Option<McProfile>,
        #[serde(default)]
        availableProfiles: Vec<McProfile>,
    }
    #[derive(Deserialize)]
    struct McProfile {
        id: String,
        name: String,
    }

    let resp = client
        .post(LITTLE_SKIN_MC_TOKEN_API)
        .bearer_auth(oauth_access_token)
        .json(&serde_json::json!({ "uuid": uuid }))
        .send()
        .await
        .map_err(|e| format!("获取 Minecraft 令牌失败: {}", e))?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("获取 Minecraft 令牌失败 (HTTP {}): {}", status, text));
    }
    let mc: McTokenResp = serde_json::from_str(&text)
        .map_err(|e| format!("解析 Minecraft 令牌失败: {}", e))?;
    let profile = mc
        .selectedProfile
        .or_else(|| mc.availableProfiles.into_iter().next())
        .ok_or("Minecraft 令牌响应缺少角色信息")?;
    Ok((mc.accessToken, profile.id, profile.name))
}

pub async fn littleskin_auth_poll(info: LittleSkinDeviceCode) -> Result<AuthSession, String> {
    let client = crate::mc::mirror::http_client();
    let params = [
        (
            "grant_type",
            "urn:ietf:params:oauth:grant-type:device_code".to_string(),
        ),
        ("client_id", littleskin_client_id()),
        ("device_code", info.device_code),
    ];
    let resp = client
        .post(LITTLE_SKIN_TOKEN_API)
        .header("Accept", "application/json")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("轮询授权结果失败: {}", e))?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        let err: serde_json::Value =
            serde_json::from_str(&text).unwrap_or_else(|_| serde_json::json!({}));
        let err_type = err["error"].as_str().unwrap_or("unknown").to_string();
        let desc = err["error_description"].as_str().unwrap_or("").to_string();
        return Err(match err_type.as_str() {
            "authorization_pending" => "authorization_pending".to_string(),
            "slow_down" => "slow_down".to_string(),
            "expired_token" => "授权已过期，请重新开始登录".to_string(),
            "invalid_client" => "应用未通过 LittleSkin 设备代码流白名单校验，请确认已申请白名单且 client_id 正确".to_string(),
            "invalid_scope" => "应用申请了未在白名单中的权限，请在 LittleSkin 白名单中补充 Yggdrasil 相关权限".to_string(),
            "access_denied" => "用户拒绝了授权".to_string(),
            _ => format!("授权失败 ({}): {}", err_type, desc),
        });
    }

    #[derive(Deserialize)]
    struct TokenResp {
        access_token: String,
        #[serde(default)]
        refresh_token: Option<String>,
        #[serde(default)]
        id_token: Option<String>,
        #[serde(default)]
        expires_in: Option<u64>,
    }
    let token: TokenResp =
        serde_json::from_str(&text).map_err(|e| format!("解析令牌失败: {}", e))?;

    let id_payload = decode_jwt_payload(token.id_token.as_deref().unwrap_or(""))
        .map_err(|e| format!("ID 令牌解析失败: {}", e))?;
    let selected = &id_payload["selectedProfile"];
    let uuid = selected["id"]
        .as_str()
        .ok_or("ID 令牌缺少 selectedProfile，请确认白名单包含 Yggdrasil.PlayerProfiles.Select 权限")?
        .to_string();
    let name = selected["name"]
        .as_str()
        .unwrap_or("Player")
        .to_string();

    let (mc_token, mc_uuid, mc_name) =
        littleskin_exchange_mc_token(&client, &token.access_token, &uuid).await?;

    let _ = littleskin_save_authlib_config();

    Ok(littleskin_build_session(
        &mc_token,
        &mc_uuid,
        &mc_name,
        token.refresh_token,
        token.expires_in.unwrap_or(259200),
    ))
}

pub async fn littleskin_auth_refresh(refresh_token: &str) -> Result<AuthSession, String> {
    let client = crate::mc::mirror::http_client();
    let params = [
        ("grant_type", "refresh_token".to_string()),
        ("refresh_token", refresh_token.to_string()),
        ("client_id", littleskin_client_id()),
    ];
    let resp = client
        .post(LITTLE_SKIN_TOKEN_API)
        .header("Accept", "application/json")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&params)
        .send()
        .await
        .map_err(|e| format!("刷新令牌失败: {}", e))?;
    let status = resp.status();
    let text = resp.text().await.unwrap_or_default();
    if !status.is_success() {
        return Err(format!("刷新令牌失败 (HTTP {}): {}", status, text));
    }

    #[derive(Deserialize)]
    struct TokenResp {
        access_token: String,
        #[serde(default)]
        refresh_token: Option<String>,
        #[serde(default)]
        id_token: Option<String>,
        #[serde(default)]
        expires_in: Option<u64>,
    }
    let token: TokenResp =
        serde_json::from_str(&text).map_err(|e| format!("解析刷新响应失败: {}", e))?;

    let id_payload = decode_jwt_payload(token.id_token.as_deref().unwrap_or(""))
        .map_err(|e| format!("ID 令牌解析失败: {}", e))?;
    let selected = &id_payload["selectedProfile"];
    let uuid = selected["id"]
        .as_str()
        .ok_or("ID 令牌缺少 selectedProfile，请确认白名单包含 Yggdrasil.PlayerProfiles.Select 权限")?
        .to_string();

    let (mc_token, mc_uuid, mc_name) =
        littleskin_exchange_mc_token(&client, &token.access_token, &uuid).await?;

    let _ = littleskin_save_authlib_config();

    Ok(littleskin_build_session(
        &mc_token,
        &mc_uuid,
        &mc_name,
        token.refresh_token,
        token.expires_in.unwrap_or(259200),
    ))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NideServerInfo {
    pub server_id: String,
    pub name: String,
    pub server_ip: Option<String>,
    pub server_pot: Option<u16>,
}

#[derive(Debug, Clone, Deserialize)]
struct NideJoinResponse {
    #[serde(default)]
    status: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct NideHasJoinedResponse {
    id: String,
    name: String,
    #[serde(default)]
    popeties: Vec<NidePopety>,
}

#[derive(Debug, Clone, Deserialize)]
struct NidePopety {
    name: String,
    value: String,
}

pub async fn get_nide_server_info(server_id: &str) -> Result<NideServerInfo, String> {
    let client = crate::mc::mirror::http_client();
    let url = format!("https://auth2.nide8.com/api/server/{}", server_id);
    
    let esp = client.get(&url).send().await
        .map_err(|e| format!("获取 Nide 服务器信息失败: {}", e))?;
    
    if !esp.status().is_success() {
        return Err(format!("获取 Nide 服务器信息失败: HTTP {}", esp.status()));
    }
    
    #[derive(Deserialize)]
    struct NideApiServerResponse {
        name: String,
        #[serde(default)]
        ip: Option<String>,
        #[serde(default)]
        pot: Option<u16>,
    }
    
    let data: NideApiServerResponse = esp.json().await
        .map_err(|e| format!("解析 Nide 服务器信息失败: {}", e))?;
    
    Ok(NideServerInfo {
        server_id: server_id.to_string(),
        name: data.name,
        server_ip: data.ip,
        server_pot: data.pot,
    })
}

pub async fn nide_auth(
    server_id: &str,
    username: &str,
    password: &str,
) -> Result<AuthSession, String> {
    let server_ul = format!("https://auth2.nide8.com/api/yggdrasil/server/{}", server_id);
    
    let client = crate::mc::mirror::http_client();
    let url = format!("{}/authserver/authenticate", server_ul);
    
    let body = serde_json::json!({
        "agent": {
            "name": "Minecraft",
            "version": 1
        },
        "username": username,
        "password": password,
        "clientToken": "skyline-launcher",
        "requestUser": true
    });
    
    let esp = client.post(&url).json(&body).send().await
        .map_err(|e| format!("Nide 认证请求失败: {}", e))?;
    
    if !esp.status().is_success() {
        let status = esp.status();
        let text = esp.text().await.unwrap_or_default();
        return Err(format!("Nide 认证失败 ({}): {}", status, text));
    }
    
    #[derive(Deserialize)]
    struct NideAuthResponse {
        accessToken: String,
        clientToken: String,
        #[serde(default)]
        availableProfiles: Vec<NideProfile>,
        #[serde(default)]
        selectedProfile: Option<NideProfile>,
    }
    
    #[derive(Deserialize)]
    struct NideProfile {
        id: String,
        name: String,
    }
    
    let data: NideAuthResponse = esp.json().await
        .map_err(|e| format!("Nide 认证响应解析失败: {}", e))?;
    
    let profile = data.selectedProfile
        .or_else(|| data.availableProfiles.into_iter().next())
        .ok_or("Nide 认证响应中没有可用的档案")?;
    
    Ok(AuthSession {
        access_token: data.accessToken,
        username: profile.name,
        uuid: profile.id,
        user_type: "nide".to_string(),
        refresh_token: None,
        expires_at: None,
    })
}

pub async fn nide_efesh(
    server_id: &str,
    access_token: &str,
    client_token: &str,
) -> Result<AuthSession, String> {
    let server_ul = format!("https://auth2.nide8.com/api/yggdrasil/server/{}", server_id);
    let client = crate::mc::mirror::http_client();
    let url = format!("{}/authserver/refresh", server_ul);
    
    let body = serde_json::json!({
        "accessToken": access_token,
        "clientToken": client_token,
        "requestUser": true
    });
    
    let esp = client.post(&url).json(&body).send().await
        .map_err(|e| format!("Nide 刷新失败: {}", e))?;
    
    if !esp.status().is_success() {
        return Err("Nide 刷新失败".to_string());
    }
    
    #[derive(Deserialize)]
    struct NideRefeshResponse {
        accessToken: String,
        selectedProfile: NideProfile,
    }
    
    #[derive(Deserialize)]
    struct NideProfile {
        id: String,
        name: String,
    }
    
    let data: NideRefeshResponse = esp.json().await
        .map_err(|e| format!("Nide 刷新响应解析失败: {}", e))?;
    
    Ok(AuthSession {
        access_token: data.accessToken,
        username: data.selectedProfile.name,
        uuid: data.selectedProfile.id,
        user_type: "nide".to_string(),
        refresh_token: None,
        expires_at: None,
    })
}

pub fn get_nide_jvm_args(server_id: &str) -> Vec<String> {
    vec![
        format!("-javaagent:nide8auth.jar={}", server_id),
        "-Dnide8auth.client=skyline-launcher".to_string(),
    ]
}


const ENCRYPT_KEY: &[u8] = b"skyline-launcher-encrypt-key-2024";

pub fn encypt_data(data: &str) -> String {
    use base64::Engine;
    let bytes = data.as_bytes();
    let encrypted: Vec<u8> = bytes.iter()
        .enumerate()
        .map(|(i, &b)| b ^ ENCRYPT_KEY[i % ENCRYPT_KEY.len()])
        .collect();
    base64::engine::general_purpose::STANDARD.encode(&encrypted)
}

pub fn decypt_data(encrypted: &str) -> Result<String, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD.decode(encrypted)
        .map_err(|e| format!("Base64 解码失败: {}", e))?;
    let decypted: Vec<u8> = bytes.iter()
        .enumerate()
        .map(|(i, &b)| b ^ ENCRYPT_KEY[i % ENCRYPT_KEY.len()])
        .collect();
    String::from_utf8(decypted)
        .map_err(|e| format!("UTF-8 解码失败: {}", e))
}

pub fn encypt_account(account: &Account) -> Account {
    let mut encrypted = account.clone();
    if let Some(ref token) = account.access_token {
        encrypted.access_token = Some(encypt_data(token));
    }
    if let Some(ref token) = account.refresh_token {
        encrypted.refresh_token = Some(encypt_data(token));
    }
    encrypted
}

pub fn decypt_account(account: &Account) -> Result<Account, String> {
    let mut decypted = account.clone();
    if let Some(ref token) = account.access_token {
        decypted.access_token = Some(decypt_data(token)?);
    }
    if let Some(ref token) = account.refresh_token {
        decypted.refresh_token = Some(decypt_data(token)?);
    }
    Ok(decypted)
}
