use crate::instance::manager;
use std::process::Command;


const AGNES_API: &str = "https://apihub.agnes-ai.com/v1/chat/completions";
const AGNES_MODEL: &str = "agnes-2.0-flash";
const API_KEY_FILE: &str = ".skyline/agnes_api_key.txt";


fn get_agnes_api_key() -> Option<String> {
    let key_path = std::env::current_dir()
        .ok()?
        .join(API_KEY_FILE);
    std::fs::read_to_string(key_path)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
}


#[tauri::command]
pub async fn save_agnes_api_key(api_key: String) -> Result<(), String> {
    let di = std::env::current_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(di.join(".skyline")).map_err(|e| e.to_string())?;
    std::fs::write(di.join(API_KEY_FILE), api_key.trim()).map_err(|e| e.to_string())?;
    Ok(())
}


#[tauri::command]
pub async fn get_agnes_api_key_status() -> Result<bool, String> {
    Ok(get_agnes_api_key().is_some())
}



#[tauri::command]
pub async fn ai_chat(body: String, api_key: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(90))
        .build()
        .map_err(|e| e.to_string())?;

    let esp = client
        .post(AGNES_API)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .body(body)
        .send()
        .await
        .map_err(|e| format!("AI 请求失败: {}", e))?;

    let status = esp.status();
    let text = esp
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    if !status.is_success() {
        return Err(format!("AI 服务返回错误 ({})：{}", status, &text));
    }

    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("解析响应失败: {}", e))?;

    let reply = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| "AI 未能返回有效响应".to_string())?
        .to_string();

    Ok(reply)
}


#[tauri::command]
pub async fn analyze_crash_auto(instance_id: String, api_key: String) -> Result<String, String> {
    
    let mc_di = crate::instance::manager::get_instance_mc_dir(&instance_id)
        .map_err(|e| format!("获取实例目录失败: {}", e))?;
    let logs_di = mc_di.join("logs");
    let log_path = logs_di.join("latest.log");
    
    let mut log_content = std::fs::read_to_string(&log_path).ok();
    if log_content.as_ref().map(|c| c.trim().is_empty()).unwrap_or(true) {
        log_content = std::fs::read_to_string(logs_di.join("skyline-launch.log")).ok();
    }
    let log_content = log_content
        .filter(|c| !c.trim().is_empty())
        .map(|c| c.chars().rev().take(20000).collect::<String>().chars().rev().collect::<String>())
        .ok_or_else(|| "未找到崩溃日志（latest.log / skyline-launch.log 均不存在）".to_string())?;
    
    let content = format!("游戏启动时崩溃了，请帮我分析这个崩溃日志，找出崩溃原因（重点关注模组不兼容、模组加载失败等启动阶段问题）并给出详细可用、分步骤的解决方案：\n\n--- 崩溃日志内容 ---\n{}", log_content);
    
    
    ai_chat(
        serde_json::to_string(&serde_json::json!({
            "model": "agnes-2.5-flash",
            "messages": [
                {"role": "system", "content": "你是Minecraft游戏报错分析专家Agnes。你必须只使用中文回答。分析用户提供的崩溃日志，找出崩溃原因并给出详细可用、分步骤的解决方案。回答要简洁直接，不要输出思考过程。"},
                {"role": "user", "content": content}
            ],
            "temperature": 0.3
        })).map_err(|e| e.to_string())?,
        api_key
    ).await
}


#[tauri::command]
pub async fn get_crash_file_path(instance_id: String) -> Result<Option<String>, String> {
    let _instance = manager::get_instance(&instance_id)?
        .ok_or_else(|| "Instance not found".to_string())?;
    let mc_di = manager::get_instance_mc_dir(&instance_id)?;
    let crash_di = mc_di.join("crash-reports");

    if !crash_di.is_dir() {
        return Ok(None);
    }

    let mut latest: Option<(std::path::PathBuf, u64)> = None;
    if let Ok(entries) = std::fs::read_dir(&crash_di) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("txt") {
                let mtime = std::fs::metadata(&path)
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
                if latest.as_ref().map(|(_, t)| mtime > *t).unwrap_or(true) {
                    latest = Some((path, mtime));
                }
            }
        }
    }

    Ok(latest.map(|(p, _)| p.to_string_lossy().to_string()))
}


#[tauri::command]
pub async fn open_folder_select(path: String) -> Result<(), String> {
    let p = std::path::PathBuf::from(&path);
    if p.exists() {
        Command::new("explorer")
            .arg("/select,")
            .arg(p.to_string_lossy().as_ref())
            .spawn()
            .map_err(|e| format!("Failed to open explorer: {}", e))?;
    } else if let Some(prent) = p.parent() {
        if prent.exists() {
            Command::new("explorer")
                .arg(prent.to_string_lossy().as_ref())
                .spawn()
                .map_err(|e| format!("Failed to open explorer: {}", e))?;
        }
    }
    Ok(())
}


#[tauri::command]
pub async fn read_crash_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read crash file: {}", e))
}


#[tauri::command]
pub async fn read_latest_log(instance_id: String) -> Result<String, String> {
    let mc_di = manager::get_instance_mc_dir(&instance_id)?;
    let log_path = mc_di.join("logs").join("latest.log");
    if !log_path.is_file() {
        return Err("latest.log not found".into());
    }
    std::fs::read_to_string(&log_path)
        .map_err(|e| format!("Failed to read latest.log: {}", e))
}


#[tauri::command]
pub async fn read_file_as_base64(path: String) -> Result<String, String> {
    let bytes = std::fs::read(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;
    Ok(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes))
}
