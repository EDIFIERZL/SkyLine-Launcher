use std::process::Command as StdCommand;
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Manager, State};

pub struct TerracottaPot(pub Mutex<Option<u16>>);

fn find_terracotta_exe(app: &AppHandle) -> Option<std::path::PathBuf> {
    let esource_di = app.path().resource_dir().ok()?;
    let candidates = [
        esource_di.join("terracotta").join("terracotta.exe"),
        esource_di.join("..").join("terracotta").join("terracotta.exe"),
    ];

    if let Some(p) = candidates.iter().find(|p| p.exists()) {
        return Some(p.clone());
    }

    if let Ok(exe) = std::env::current_exe() {
        let exe_di = exe.parent()?;
        let dev_fallbacks = [
            exe_di.join("..").join("..").join("resources").join("terracotta").join("terracotta.exe"),
            exe_di.join("resources").join("terracotta").join("terracotta.exe"),
        ];
        for p in dev_fallbacks.iter() {
            if p.exists() {
                return Some(p.clone());
            }
        }
    }

    None
}

fn read_lock_port() -> Option<u16> {
    let path = std::env::temp_dir().join("terracotta").join("terracotta.lock");
    if !path.exists() {
        return None;
    }

    #[cfg(target_os = "windows")]
    {
        use std::io::Read;
        use std::os::windows::fs::OpenOptionsExt;

        let mut f = std::fs::OpenOptions::new()
            .read(true)
            .share_mode(0x07 )
            .open(&path)
            .ok()?;
        let mut buf = [0u8; 2];
        f.read_exact(&mut buf).ok()?;
        Some(((buf[0] as u16) << 8) + buf[1] as u16)
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::io::Read;
        let mut f = std::fs::File::open(&path).ok()?;
        let mut buf = [0u8; 2];
        f.read_exact(&mut buf).ok()?;
        Some(((buf[0] as u16) << 8) + buf[1] as u16)
    }
}

async fn ping_meta(pot: u16) -> bool {
    let client = reqwest::Client::new();
    client
        .get(format!("http://127.0.0.1:{}/meta", pot))
        .timeout(Duration::from_millis(1000))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

fn base_url(pot_state: &TerracottaPot) -> Result<String, String> {
    let pot = pot_state
        .0
        .lock()
        .unwrap()
        .ok_or_else(|| "陶瓦联机服务未启动".to_string())?;
    Ok(format!("http://127.0.0.1:{}/", pot))
}

#[tauri::command]
pub fn launch_terracotta(app: AppHandle) -> Result<(), String> {
    let exe_path = find_terracotta_exe(&app)
        .ok_or_else(|| "找不到陶瓦联机程序".to_string())?;

    let pot_file = std::env::temp_dir().join("skyline-terracotta-port.json");
    let _ = std::fs::remove_file(&pot_file);

    StdCommand::new(&exe_path)
        .arg("--hmcl")
        .arg(&pot_file)
        .spawn()
        .map_err(|e| format!("启动陶瓦联机失败: {}", e))?;

    Ok(())
}

fn cached_port(pot_state: &TerracottaPot) -> Option<u16> {
    *pot_state.0.lock().unwrap()
}

fn store_port(pot_state: &TerracottaPot, pot: u16) {
    *pot_state.0.lock().unwrap() = Some(pot);
}

#[tauri::command]
pub async fn ensure_terracotta_running(
    app: AppHandle,
    pot_state: State<'_, TerracottaPot>,
) -> Result<u16, String> {
    if let Some(port) = cached_port(&pot_state) {
        if ping_meta(port).await {
            return Ok(port);
        }
    }

    if let Some(port) = read_lock_port() {
        if ping_meta(port).await {
            store_port(&pot_state, port);
            return Ok(port);
        }
    }

    let exe_path = find_terracotta_exe(&app).ok_or_else(|| "找不到陶瓦联机程序".to_string())?;
    let port_file = std::env::temp_dir().join("skyline-terracotta-port.json");
    let _ = std::fs::remove_file(&port_file);

    StdCommand::new(&exe_path)
        .arg("--hmcl")
        .arg(&port_file)
        .spawn()
        .map_err(|e| format!("启动陶瓦联机失败: {}", e))?;

    for _ in 0..120 {
        if let Ok(content) = std::fs::read_to_string(&port_file) {
            if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(p) = value.get("port").and_then(|p| p.as_u64()) {
                    let port = p as u16;
                    if ping_meta(port).await {
                        store_port(&pot_state, port);
                        return Ok(port);
                    }
                }
            }
        }
        tokio::time::sleep(Duration::from_millis(200)).await;
    }

    Err("启动陶瓦联机超时，请稍后重试".to_string())
}

#[tauri::command]
pub async fn terracotta_state(
    pot_state: State<'_, TerracottaPot>,
) -> Result<serde_json::Value, String> {
    let base = base_url(&pot_state)?;
    let client = reqwest::Client::new();
    client
        .get(format!("{}state", base))
        .timeout(Duration::from_secs(3))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn terracotta_meta(
    pot_state: State<'_, TerracottaPot>,
) -> Result<serde_json::Value, String> {
    let base = base_url(&pot_state)?;
    let client = reqwest::Client::new();
    client
        .get(format!("{}meta", base))
        .timeout(Duration::from_secs(3))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn terracotta_scanning(
    player: Option<String>,
    pot_state: State<'_, TerracottaPot>,
) -> Result<(), String> {
    let base = base_url(&pot_state)?;
    let client = reqwest::Client::new();
    let mut eq = client.get(format!("{}state/scanning", base));
    if let Some(p) = player.as_ref() {
        if !p.trim().is_empty() {
            eq = eq.query(&[("player", p)]);
        }
    }
    eq.timeout(Duration::from_secs(3))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn terracotta_guesting(
    room: String,
    player: Option<String>,
    pot_state: State<'_, TerracottaPot>,
) -> Result<bool, String> {
    let base = base_url(&pot_state)?;
    let client = reqwest::Client::new();
    let mut req = client
        .get(format!("{}state/guesting", base))
        .query(&[("room", room)]);
    if let Some(p) = player.as_deref() {
        if !p.trim().is_empty() {
            req = req.query(&[("player", p)]);
        }
    }
    let resp = req
        .timeout(Duration::from_secs(3))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(resp.status().is_success())
}

#[tauri::command]
pub async fn terracotta_ide(pot_state: State<'_, TerracottaPot>) -> Result<(), String> {
    let base = base_url(&pot_state)?;
    let client = reqwest::Client::new();
    client
        .get(format!("{}state/ide", base))
        .timeout(Duration::from_secs(3))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn terracotta_stop(pot_state: State<'_, TerracottaPot>) -> Result<(), String> {
    let base = base_url(&pot_state)?;
    let client = reqwest::Client::new();
    let _ = client
        .get(format!("{}panic", base))
        .query(&[("peaceful", "true")])
        .timeout(Duration::from_secs(3))
        .send()
        .await;
    *pot_state.0.lock().unwrap() = None;
    Ok(())
}