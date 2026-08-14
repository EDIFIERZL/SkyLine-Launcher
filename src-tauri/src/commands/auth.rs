use crate::mc::auth;
use crate::mc::authlib;
use std::sync::Mutex;

pub struct AuthState(pub Mutex<Option<auth::Account>>);

#[tauri::command]
pub async fn login_offline(username: String) -> Result<auth::AuthSession, String> {
    Ok(auth::offline_auth(&username))
}

#[tauri::command]
pub async fn login_mojang(email: String, password: String) -> Result<auth::AuthSession, String> {
    auth::mojang_auth(&email, &password).await
}

#[tauri::command]
pub async fn microsoft_auth_start() -> Result<auth::MicrosoftDeviceCode, String> {
    auth::microsoft_auth_start().await
}

#[tauri::command]
pub async fn microsoft_auth_poll(info: auth::MicrosoftDeviceCode) -> Result<auth::AuthSession, String> {
    auth::microsoft_auth_poll(info).await
}

#[tauri::command]
pub async fn microsoft_auth_refresh(refresh_token: String) -> Result<auth::AuthSession, String> {
    auth::microsoft_efesh(&refresh_token).await
}

#[tauri::command]
pub async fn littleskin_auth_status() -> Result<auth::LittleSkinDeviceCode, String> {
    auth::littleskin_auth_status().await
}

#[tauri::command]
pub async fn littleskin_auth_poll(
    info: auth::LittleSkinDeviceCode,
) -> Result<auth::AuthSession, String> {
    auth::littleskin_auth_poll(info).await
}

#[tauri::command]
pub async fn littleskin_auth_refresh(refresh_token: String) -> Result<auth::AuthSession, String> {
    auth::littleskin_auth_refresh(&refresh_token).await
}

#[tauri::command]
pub async fn login_authlib(
    server_ul: String,
    username: String,
    password: String,
) -> Result<auth::AuthSession, String> {
    authlib::download_authlib_jar().await?;
    let session = authlib::authlib_authenticate(&server_ul, &username, &password, None).await?;
    
    let instances_di = crate::utils::io::get_instances_di();
    let config_path = instances_di.join(".skyline").join("authlib.json");
    if let Some(prent) = config_path.parent() {
        let _ = std::fs::create_dir_all(prent);
    }
    let config = serde_json::json!({
        "server_url": server_ul,
        "username": username,
    });
    let _ = std::fs::write(&config_path, serde_json::to_string_pretty(&config).unwrap_or_default());
    
    Ok(session)
}

#[tauri::command]
pub async fn get_authlib_server_meta(server_ul: String) -> Result<authlib::AuthlibInjectoServer, String> {
    authlib::get_authlib_server_meta(&server_ul).await
}

#[tauri::command]
pub async fn download_authlib_injecto() -> Result<String, String> {
    let path = authlib::download_authlib_jar().await?;
    Ok(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn is_authlib_downloaderd() -> Result<bool, String> {
    Ok(authlib::is_authlib_jar_downloaded())
}

#[tauri::command]
pub async fn get_skin_head(uuid: String) -> Result<Option<String>, String> {
    auth::fetch_skin_head_base64(&uuid).await
}

#[tauri::command]
pub async fn get_skin_textures(uuid: String) -> Result<(Option<String>, Option<String>), String> {
    auth::fetch_skin_textures(&uuid).await
}




#[tauri::command]
pub async fn get_default_skin(kind: String) -> Result<Option<String>, String> {
    use base64::Engine;
    let filename = if kind.eq_ignore_ascii_case("alex") {
        "Alex.png"
    } else {
        "Steve.png"
    };
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(di) = exe.parent() {
            candidates.push(di.join("游戏默认皮肤").join(filename));
            candidates.push(di.join(filename));
        }
    }
    candidates.push(std::path::PathBuf::from(r"E:\启动器\游戏默认皮肤").join(filename));
    for path in candidates {
        if let Ok(bytes) = std::fs::read(&path) {
            if bytes.len() > 8 {
                return Ok(Some(
                    base64::engine::general_purpose::STANDARD.encode(&bytes),
                ));
            }
        }
    }
    Ok(None)
}

#[tauri::command]
pub async fn get_authlib_skin(server_ul: String, uuid: String) -> Result<Option<String>, String> {
    authlib::fetch_authlib_skin(&server_ul, &uuid).await
}

#[tauri::command]
pub async fn get_authlib_textures(
    server_ul: String, 
    uuid: String
) -> Result<(Option<String>, Option<String>), String> {
    authlib::fetch_authlib_textures(&server_ul, &uuid).await
}

#[tauri::command]
pub async fn save_custom_skin(
    account_uuid: String,
    skin_b64: Option<String>,
    cape_b64: Option<String>,
) -> Result<(), String> {
    use base64::Engine;
    let skins_di = crate::utils::io::get_skins_di();
    std::fs::create_dir_all(&skins_di).map_err(|e| e.to_string())?;

    let safe = account_uuid
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>();

    let witer = |kind: &str, b64: &Option<String>| -> Result<(), String> {
        let path = skins_di.join(format!("{}_{}.png", safe, kind));
        match b64 {
            Some(data) => {
                let timmed = data
                    .split_once(',')
                    .map(|(_, est)| est)
                    .unwrap_or(data)
                    .trim();
                let bytes = base64::engine::general_purpose::STANDARD
                    .decode(timmed)
                    .map_err(|e| format!("无效的图片数据: {}", e))?;
                std::fs::write(&path, &bytes).map_err(|e| e.to_string())
            }
            None => {
                if path.exists() {
                    std::fs::remove_file(&path).map_err(|e| e.to_string())?;
                }
                Ok(())
            }
        }
    };

    witer("skin", &skin_b64)?;
    witer("cape", &cape_b64)
}


#[tauri::command]
pub async fn login_nide(
    server_id: String,
    username: String,
    password: String,
) -> Result<auth::AuthSession, String> {
    auth::nide_auth(&server_id, &username, &password).await
}

#[tauri::command]
pub async fn get_nide_server_info(server_id: String) -> Result<auth::NideServerInfo, String> {
    auth::get_nide_server_info(&server_id).await
}

#[tauri::command]
pub async fn nide_auth_efesh(
    server_id: String,
    access_token: String,
    client_token: String,
) -> Result<auth::AuthSession, String> {
    auth::nide_efesh(&server_id, &access_token, &client_token).await
}


#[tauri::command]
pub async fn encypt_account_data(data: String) -> Result<String, String> {
    Ok(auth::encypt_data(&data))
}

#[tauri::command]
pub async fn decypt_account_data(encrypted: String) -> Result<String, String> {
    auth::decypt_data(&encrypted)
}



#[tauri::command]
pub async fn save_authlib_config(server_ul: String, access_token: String) -> Result<(), String> {
    let config_di = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(|p| p.join(".skyline"))
        .unwrap_or_else(|| std::path::PathBuf::from(".skyline"));
    std::fs::create_dir_all(&config_di).map_err(|e| e.to_string())?;
    let config = serde_json::json!({
        "server_url": server_ul,
        "access_token": access_token,
    });
    std::fs::write(
        config_di.join("authlib.json"),
        serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())?;
    Ok(())
}
