use crate::instance::manager;
use serde::{Deserialize, Serialize};
use std::process::Command;

const CONFIG_DIR: &str = ".skyline";
const API_KEY_FILE: &str = "ai_api_key.txt";
const PROVIDER_FILE: &str = "ai_provider.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProvider {
    pub id: String,
    pub name: String,
    pub endpoint: String,
    pub api_format: String,
    pub default_model: String,
}

impl Default for AiProvider {
    fn default() -> Self {
        Self {
            id: "agnes".into(),
            name: "Agnes AI".into(),
            endpoint: "https://apihub.agnes-ai.com/v1/chat/completions".into(),
            api_format: "openai".into(),
            default_model: "agnes-2.5-flash".into(),
        }
    }
}

pub fn get_providers() -> Vec<AiProvider> {
    vec![
        AiProvider {
            id: "agnes".into(),
            name: "Agnes AI".into(),
            endpoint: "https://apihub.agnes-ai.com/v1/chat/completions".into(),
            api_format: "openai".into(),
            default_model: "agnes-2.5-flash".into(),
        },
        AiProvider {
            id: "deepseek".into(),
            name: "DeepSeek".into(),
            endpoint: "https://api.deepseek.com/v1/chat/completions".into(),
            api_format: "openai".into(),
            default_model: "deepseek-chat".into(),
        },
        AiProvider {
            id: "openai".into(),
            name: "OpenAI".into(),
            endpoint: "https://api.openai.com/v1/chat/completions".into(),
            api_format: "openai".into(),
            default_model: "gpt-4o-mini".into(),
        },
        AiProvider {
            id: "anthropic".into(),
            name: "Anthropic".into(),
            endpoint: "https://api.anthropic.com/v1/messages".into(),
            api_format: "anthropic".into(),
            default_model: "claude-sonnet-4-20250514".into(),
        },
        AiProvider {
            id: "google".into(),
            name: "Google Gemini".into(),
            endpoint: "https://generativelanguage.googleapis.com/v1beta".into(),
            api_format: "google".into(),
            default_model: "gemini-2.0-flash".into(),
        },
        AiProvider {
            id: "moonshot".into(),
            name: "Moonshot (月之暗面)".into(),
            endpoint: "https://api.moonshot.cn/v1/chat/completions".into(),
            api_format: "openai".into(),
            default_model: "moonshot-v1-8k".into(),
        },
        AiProvider {
            id: "zhipu".into(),
            name: "智谱 (GLM)".into(),
            endpoint: "https://open.bigmodel.cn/api/paas/v4/chat/completions".into(),
            api_format: "openai".into(),
            default_model: "glm-4-flash".into(),
        },
        AiProvider {
            id: "siliconflow".into(),
            name: "SiliconFlow".into(),
            endpoint: "https://api.siliconflow.cn/v1/chat/completions".into(),
            api_format: "openai".into(),
            default_model: "Qwen/Qwen2.5-7B-Instruct".into(),
        },
        AiProvider {
            id: "openrouter".into(),
            name: "OpenRouter".into(),
            endpoint: "https://openrouter.ai/api/v1/chat/completions".into(),
            api_format: "openai".into(),
            default_model: "google/gemini-2.0-flash-001".into(),
        },
        AiProvider {
            id: "groq".into(),
            name: "Groq".into(),
            endpoint: "https://api.groq.com/openai/v1/chat/completions".into(),
            api_format: "openai".into(),
            default_model: "llama-3.1-70b-versatile".into(),
        },
        AiProvider {
            id: "mistral".into(),
            name: "Mistral".into(),
            endpoint: "https://api.mistral.ai/v1/chat/completions".into(),
            api_format: "openai".into(),
            default_model: "mistral-small-latest".into(),
        },
        AiProvider {
            id: "xai".into(),
            name: "xAI (Grok)".into(),
            endpoint: "https://api.x.ai/v1/chat/completions".into(),
            api_format: "openai".into(),
            default_model: "grok-2".into(),
        },
        AiProvider {
            id: "qwen".into(),
            name: "通义千问".into(),
            endpoint: "https://dashscope.aliyuncs.com/compatible-mode/v1/chat/completions".into(),
            api_format: "openai".into(),
            default_model: "qwen-turbo".into(),
        },
        AiProvider {
            id: "custom".into(),
            name: "自定义".into(),
            endpoint: String::new(),
            api_format: "openai".into(),
            default_model: String::new(),
        },
    ]
}

fn config_dir() -> Option<std::path::PathBuf> {
    std::env::current_dir().ok().map(|d| d.join(CONFIG_DIR))
}

fn get_api_key() -> Option<String> {
    let path = config_dir()?.join(API_KEY_FILE);
    std::fs::read_to_string(path)
        .ok()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.trim().to_string())
}

fn get_saved_provider() -> AiProvider {
    let path = match config_dir() {
        Some(d) => d.join(PROVIDER_FILE),
        None => return AiProvider::default(),
    };
    std::fs::read_to_string(path)
        .ok()
        .and_then(|c| serde_json::from_str(&c).ok())
        .unwrap_or_default()
}

fn save_provider_config(provider: &AiProvider) -> Result<(), String> {
    let dir = config_dir().ok_or("无法获取配置目录")?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let json = serde_json::to_string_pretty(provider).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(PROVIDER_FILE), json).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn save_agnes_api_key(api_key: String) -> Result<(), String> {
    let dir = config_dir().ok_or("无法获取配置目录")?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(API_KEY_FILE), api_key.trim()).map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn get_agnes_api_key_status() -> Result<bool, String> {
    Ok(get_api_key().is_some())
}

#[tauri::command]
pub async fn get_ai_providers() -> Result<Vec<AiProvider>, String> {
    Ok(get_providers())
}

#[tauri::command]
pub async fn get_ai_provider_config() -> Result<AiProvider, String> {
    Ok(get_saved_provider())
}

#[tauri::command]
pub async fn save_ai_provider_config(provider: AiProvider) -> Result<(), String> {
    save_provider_config(&provider)
}

#[tauri::command]
pub async fn get_ai_config_status() -> Result<serde_json::Value, String> {
    let key = get_api_key();
    let provider = get_saved_provider();
    Ok(serde_json::json!({
        "has_key": key.is_some(),
        "provider": provider,
    }))
}

fn build_openai_body(model: &str, messages: &[serde_json::Value], reasoning_effort: Option<&str>) -> serde_json::Value {
    let mut body = serde_json::json!({
        "model": model,
        "messages": messages,
        "temperature": 0.3,
    });
    if let Some(effort) = reasoning_effort {
        if !effort.is_empty() && effort != "none" {
            body["reasoning_effort"] = serde_json::Value::String(effort.to_string());
        }
    }
    body
}

fn build_anthropic_body(model: &str, messages: &[serde_json::Value]) -> serde_json::Value {
    let mut system = String::new();
    let mut user_msgs = Vec::new();
    for m in messages {
        if m["role"] == "system" {
            system.push_str(m["content"].as_str().unwrap_or(""));
            system.push('\n');
        } else {
            user_msgs.push(m.clone());
        }
    }
    serde_json::json!({
        "model": model,
        "max_tokens": 4096,
        "system": system.trim(),
        "messages": user_msgs,
    })
}

fn build_google_body(_model: &str, messages: &[serde_json::Value]) -> serde_json::Value {
    let mut contents = Vec::new();
    for m in messages {
        let role = if m["role"] == "assistant" { "model" } else { "user" };
        let text = m["content"].as_str().unwrap_or("");
        contents.push(serde_json::json!({
            "role": role,
            "parts": [{ "text": text }]
        }));
    }
    serde_json::json!({
        "contents": contents,
        "generationConfig": { "maxOutputTokens": 4096 }
    })
}

fn extract_reply(format: &str, data: &serde_json::Value) -> Option<String> {
    match format {
        "anthropic" => {
            data["content"][0]["text"].as_str().map(|s| s.to_string())
        }
        "google" => {
            data["candidates"][0]["content"]["parts"][0]["text"]
                .as_str()
                .map(|s| s.to_string())
        }
        _ => {
            data["choices"][0]["message"]["content"]
                .as_str()
                .map(|s| s.to_string())
        }
    }
}

fn friendly_error(status: u16, body: &str) -> String {
    let lower = body.to_lowercase();
    if status == 402 || lower.contains("insufficient_balance") || lower.contains("insufficient balance")
        || lower.contains("insufficient_quota") || lower.contains("insufficient quota") {
        return "AI 账户余额不足，请登录对应服务商官网充值后重试".into();
    }
    if status == 401 || lower.contains("invalid api key") || lower.contains("unauthorized") {
        return "API Key 无效或已过期，请检查并重新填写".into();
    }
    if status == 403 || lower.contains("permission_denied") || lower.contains("forbidden") {
        return "API Key 权限不足，请检查是否有对应模型的调用权限".into();
    }
    if status == 429 || lower.contains("rate_limit") || lower.contains("too many requests") {
        return "请求过于频繁，已触发限流，请稍等几秒后重试".into();
    }
    if status == 404 || lower.contains("model not found") || lower.contains("does not exist") {
        return "模型名称不正确，请检查所选模型是否匹配".into();
    }
    if status >= 500 {
        return "AI 服务商暂时不可用，请稍后重试".into();
    }
    format!("AI 请求失败 ({}): {}", status, &body[..body.len().min(200)])
}

#[tauri::command]
pub async fn ai_chat(body: String, api_key: String) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(90))
        .build()
        .map_err(|e| e.to_string())?;

    let esp = client
        .post("https://apihub.agnes-ai.com/v1/chat/completions")
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
        return Err(friendly_error(status.as_u16(), &text));
    }

    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("解析响应失败: {}", e))?;

    extract_reply("openai", &json)
        .ok_or_else(|| "AI 未能返回有效响应".to_string())
}

#[tauri::command]
pub async fn ai_chat_v2(
    messages: Vec<serde_json::Value>,
    api_key: String,
    reasoning_effort: Option<String>,
    model: Option<String>,
) -> Result<String, String> {
    let provider = get_saved_provider();
    let selected_model = model.filter(|value| !value.trim().is_empty()).unwrap_or_else(|| provider.default_model.clone());
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(90))
        .build()
        .map_err(|e| e.to_string())?;

    let body = match provider.api_format.as_str() {
        "anthropic" => build_anthropic_body(&selected_model, &messages),
        "google" => build_google_body(&selected_model, &messages),
        _ => build_openai_body(&selected_model, &messages, reasoning_effort.as_deref()),
    };

    let url = match provider.api_format.as_str() {
        "google" => {
            format!("{}/models/{}:generateContent?key={}", provider.endpoint, selected_model, api_key)
        }
        _ => provider.endpoint.clone(),
    };

    let mut req_builder = client.post(&url).header("Content-Type", "application/json");
    match provider.api_format.as_str() {
        "anthropic" => {
            req_builder = req_builder
                .header("x-api-key", &api_key)
                .header("anthropic-version", "2023-06-01");
        }
        "google" => {}
        _ => {
            req_builder = req_builder.header("Authorization", format!("Bearer {}", api_key));
        }
    }

    let esp = req_builder
        .body(serde_json::to_string(&body).unwrap_or_default())
        .send()
        .await
        .map_err(|e| format!("AI 请求失败: {}", e))?;

    let status = esp.status();
    let text = esp
        .text()
        .await
        .map_err(|e| format!("读取响应失败: {}", e))?;

    if !status.is_success() {
        return Err(friendly_error(status.as_u16(), &text));
    }

    let json: serde_json::Value = serde_json::from_str(&text)
        .map_err(|e| format!("解析响应失败: {}", e))?;

    extract_reply(&provider.api_format, &json)
        .ok_or_else(|| "AI 未能返回有效响应".to_string())
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
