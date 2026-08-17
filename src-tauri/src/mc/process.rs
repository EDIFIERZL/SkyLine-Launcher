use serde::Serialize;
use std::io::{BufRead, BufReader, Write};
#[cfg(target_os = "windows")]
use std::os::windows::io::AsRawHandle;
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize)]
pub struct GameProcessInfo {
    pub pid: u32,
    pub running: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LogSource {
    Game,
    Watche,
}

#[derive(Debug, Clone, Serialize)]
pub struct GameLogEnty {
    pub level: String,
    pub message: String,
    pub timestamp: u64,
    pub source: LogSource,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ExitReason {
    Normal,
    Crash,
    Killed,
    NoWindow,
}

impl ExitReason {
    pub fn description(&self) -> &str {
        match self {
            ExitReason::Normal => "游戏正常退出",
            ExitReason::Crash => "游戏崩溃",
            ExitReason::Killed => "游戏被强制停止",
            ExitReason::NoWindow => "游戏窗口丢失，可能已崩溃",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct GameExitInfo {
    pub instance_id: String,
    pub exit_code: Option<i32>,
    pub reason: ExitReason,
    pub play_time_secs: u64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LaunchStage {
    JvmStarting,
    GameLoading,
    WaitingWindow,
    Running,
}

impl LaunchStage {
    pub fn label(&self) -> &str {
        match self {
            LaunchStage::JvmStarting => "JVM 启动中...",
            LaunchStage::GameLoading => "游戏加载中...",
            LaunchStage::WaitingWindow => "等待游戏窗口...",
            LaunchStage::Running => "游戏运行中",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct LaunchProgressEvent {
    pub instance_id: String,
    pub stage: LaunchStage,
    pub message: String,
}

#[cfg(target_os = "windows")]
mod window_detect {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    type HWND = *mut std::ffi::c_void;
    type DWORD = u32;
    type BOOL = i32;
    type LPARAM = isize;
    type LRESULT = isize;

    extern "system" {
        fn EnumWindows(lpEnumFunc: unsafe extern "system" fn(HWND, LPARAM) -> BOOL, lparam: LPARAM) -> BOOL;
        fn GetWindowTextA(hWnd: HWND, lpString: *mut u8, nMaxCount: i32) -> i32;
        fn GetWindowThreadProcessId(hWnd: HWND, lpdwProcessId: *mut DWORD) -> DWORD;
        fn IsWindowVisible(hWnd: HWND) -> BOOL;
        fn GetExitCodeProcess(hProcess: *mut std::ffi::c_void, lpExitCode: *mut DWORD) -> BOOL;
    }

    pub fn has_game_window(pid: u32) -> bool {
        let found = Arc::new(AtomicBool::new(false));
        let found_clone = found.clone();

        unsafe extern "system" fn enum_callback(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let data = &*(lparam as *const (u32, Arc<AtomicBool>));
            let target_pid = data.0;
            let found = &data.1;

            if IsWindowVisible(hwnd) == 0 {
                return 1;
            }

            let mut window_pid: DWORD = 0;
            GetWindowThreadProcessId(hwnd, &mut window_pid);

            if window_pid != target_pid {
                return 1;
            }

            let mut title_buf = [0u8; 512];
            let title_len = GetWindowTextA(hwnd, title_buf.as_mut_ptr(), 512);
            if title_len > 0 {
                let title = std::str::from_utf8(&title_buf[..title_len as usize]).unwrap_or("");
                if title.contains("Minecraft")
                    || title.contains("minecraft")
                    || title.contains("Forge")
                    || title.contains("Fabric")
                    || title.contains("NeoForge")
                {
                    found.store(true, Ordering::Relaxed);
                    return 0;
                }
            }

            1
        }

        let data = (pid, found_clone);
        let data_pt = &data as *const (u32, Arc<AtomicBool>) as LPARAM;

        unsafe {
            EnumWindows(enum_callback, data_pt);
        }

        found.load(Ordering::Relaxed)
    }
}

#[cfg(target_os = "windows")]
fn set_child_priority(child: &std::process::Child, priority_class: i32) {
    use windows_sys::Win32::System::Threading::SetPriorityClass;
    unsafe {
        let handle = child.as_raw_handle();
        SetPriorityClass(handle as *mut std::ffi::c_void, priority_class as u32);
    }
}

#[cfg(not(target_os = "windows"))]
fn set_child_priority(_child: &std::process::Child, _priority_class: i32) {}

#[cfg(not(target_os = "windows"))]
mod window_detect {
    pub fn has_game_window(_pid: u32) -> bool {
        true
    }
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn kill_process_tee(pid: u32) {
    #[cfg(target_os = "windows")]
    {
        let _ = crate::utils::io::no_window(&mut Command::new("taskkill"))
            .args(["/PID", &pid.to_string(), "/T", "/F"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .and_then(|mut c| c.wait());
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::process::Command::new("pkill")
            .args(["-TERM", "-P", &pid.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .and_then(|mut c| c.wait());
    }
}

pub struct GameProcess {
    child: Arc<Mutex<Option<Child>>>,
    pub log_rx: Option<Receiver<GameLogEnty>>,
    kill_tx: Sender<()>,
    kill_done_x: Receiver<()>,
    pub pid: u32,
    window_found: Arc<AtomicBool>,
    no_log_duration: Arc<Mutex<Duration>>,
}

impl GameProcess {
    pub fn spawn(
        java_path: &str,
        jvm_args: &[String],
        game_args: &[String],
        env: &std::collections::HashMap<String, String>,
        wok_di: &std::path::Path,
        os_priority: i32,
    ) -> Result<Self, String> {
        let mut cmd = Command::new(java_path);
        cmd.args(jvm_args);
        cmd.args(game_args);
        cmd.current_dir(wok_di);

        for (k, v) in env {
            cmd.env(k, v);
        }

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());
        cmd.stdin(Stdio::null());
        crate::utils::io::no_window(&mut cmd);

        let mut child = cmd.spawn().map_err(|e| format!("Failed to start Minecraft: {}", e))?;
        let pid = child.id();

        
        #[cfg(target_os = "windows")]
        set_child_priority(&child, os_priority);

        let log_file = {
            let logs_di = wok_di.join("logs");
            let _ = std::fs::create_dir_all(&logs_di);
            match std::fs::File::create(logs_di.join("skyline-launch.log")) {
                Ok(f) => {
                    let mut header = String::new();
                    header.push_str(&format!(
                        "== SkyLine launch {} ==\n$ {}\n",
                        now_secs(),
                        java_path
                    ));
                    for a in jvm_args {
                        header.push_str(&format!("  {}\n", a));
                    }
                    header.push_str("-- game args --\n");
                    for a in game_args {
                        header.push_str(&format!("  {}\n", a));
                    }
                    header.push('\n');
                    let _ = std::io::Write::write_all(&mut &f, header.as_bytes());
                    Some(std::sync::Arc::new(std::sync::Mutex::new(f)))
                }
                Err(_) => None,
            }
        };

        let (log_tx, log_rx) = mpsc::channel();
        let (kill_tx, kill_x) = mpsc::channel();
        let (kill_done_tx, kill_done_x) = mpsc::channel();

        let stdout = child.stdout.take().ok_or("No stdout")?;
        let stderr = child.stderr.take().ok_or("No stderrr")?;

        let child_ac = Arc::new(Mutex::new(Some(child)));
        let child_ac_kill = child_ac.clone();
        let window_found = Arc::new(AtomicBool::new(false));
        let window_found_watcher = window_found.clone();
        let no_log_duration = Arc::new(Mutex::new(Duration::ZERO));
        let no_log_duration_watcher = no_log_duration.clone();

        let log_tx_stdout = log_tx.clone();
        let log_file_stdout = log_file.clone();
        thread::spawn(move || {
            let eade = BufReader::new(stdout);
            for line in eade.lines() {
                if let Ok(line) = line {
                    if let Some(ref f) = log_file_stdout {
                        if let Ok(mut f) = f.lock() {
                            let _ = writeln!(f, "{}", line);
                        }
                    }
                    let level = classify_log_level(&line);
                    let _ = log_tx_stdout.send(GameLogEnty {
                        level,
                        message: line,
                        timestamp: now_secs(),
                        source: LogSource::Game,
                    });
                }
            }
        });

        let log_tx_stderr = log_tx.clone();
        let log_file_stderr = log_file.clone();
        thread::spawn(move || {
            let eade = BufReader::new(stderr);
            for line in eade.lines() {
                if let Ok(line) = line {
                    if let Some(ref f) = log_file_stderr {
                        if let Ok(mut f) = f.lock() {
                            let _ = writeln!(f, "{}", line);
                        }
                    }
                    let level = classify_log_level(&line);
                    let _ = log_tx_stderr.send(GameLogEnty {
                        level,
                        message: line,
                        timestamp: now_secs(),
                        source: LogSource::Game,
                    });
                }
            }
        });

        let log_tx_watcher = log_tx.clone();
        let child_async_watcherr = child_ac.clone();
        thread::spawn(move || {
            let mut last_log_time = Instant::now();

            loop {
                thread::sleep(Duration::from_secs(2));

                let is_running = {
                    if let Some(ref mut c) = *child_async_watcherr
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                    {
                        match c.try_wait() {
                            Ok(None) => true,
                            _ => false,
                        }
                    } else {
                        false
                    }
                };

                if !is_running {
                    break;
                }

                {
                    let mut no_log = no_log_duration_watcher.lock().unwrap();
                    *no_log = last_log_time.elapsed();
                }

                if last_log_time.elapsed() > Duration::from_secs(60) {
                    let _ = log_tx_watcher.send(GameLogEnty {
                        level: "warn".into(),
                        message: format!(
                            "[Watcher] 已 {} 秒无日志输出，游戏可能卡住",
                            last_log_time.elapsed().as_secs()
                        ),
                        timestamp: now_secs(),
                        source: LogSource::Watche,
                    });
                }
            }
        });

        
        let window_pid = pid;
        let window_found_fast = window_found_watcher.clone();
        thread::spawn(move || {
            for _ in 0..200 {
                
                thread::sleep(Duration::from_millis(150));
                if window_detect::has_game_window(window_pid) {
                    window_found_fast.store(true, Ordering::Relaxed);
                    return;
                }
            }
        });

        thread::spawn(move || {
            let _ = kill_x.recv();
            {
                let mut guad = child_ac_kill.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(child) = guad.as_mut() {
                    kill_process_tee(child.id());
                    let _ = child.kill();
                    let _ = child.wait();
                }
            }
            let _ = kill_done_tx.send(());
        });

        Ok(GameProcess {
            child: child_ac,
            log_rx: Some(log_rx),
            kill_tx,
            kill_done_x,
            pid,
            window_found,
            no_log_duration,
        })
    }

    pub fn is_running(&self) -> bool {
        if let Some(child) = self.child.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
            match child.try_wait() {
                Ok(None) => true,
                _ => false,
            }
        } else {
            false
        }
    }

    pub fn get_exit_info(&self) -> Option<(Option<i32>, ExitReason)> {
        if let Some(child) = self.child.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
            match child.try_wait() {
                Ok(Some(status)) => {
                    let code = status.code();
                    let reason = match code {
                        Some(0) => ExitReason::Normal,
                        Some(_) => ExitReason::Crash,
                        None => ExitReason::Crash,
                    };
                    Some((code, reason))
                }
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn exit_code(&self) -> Option<i32> {
        if let Some(child) = self.child.lock().unwrap_or_else(|e| e.into_inner()).as_mut() {
            match child.try_wait() {
                Ok(Some(status)) => status.code(),
                _ => None,
            }
        } else {
            None
        }
    }

    pub fn verify_stated(&self, timeout: Duration) -> Result<(), String> {
        let stat = Instant::now();
        loop {
            if !self.is_running() {
                let mut logs = Vec::new();
                if let Some(x) = &self.log_rx {
                    while let Ok(entry) = x.try_recv() {
                        logs.push(entry.message);
                    }
                }
                let code = self.exit_code();
                let tail: Vec<String> = logs.into_iter().rev().take(25).rev().collect();
                return Err(if tail.is_empty() {
                    format!("游戏进程启动后立即退出 (退出码 {:?})", code)
                } else {
                    format!(
                        "{}\n游戏进程启动后立即退出 (退出码 {:?})",
                        tail.join("\n"),
                        code
                    )
                });
            }
            
            
            if self.window_found.load(Ordering::Relaxed) {
                return Ok(());
            }
            if stat.elapsed() >= timeout {
                return Ok(());
            }
            thread::sleep(Duration::from_millis(80));
        }
    }

    pub fn has_window(&self) -> bool {
        self.window_found.load(Ordering::Relaxed)
    }

    pub fn no_log_duration(&self) -> Duration {
        *self.no_log_duration.lock().unwrap()
    }

    pub fn stop(&self) -> Result<(), String> {
        let _ = self.kill_tx.send(());
        let _ = self
            .kill_done_x
            .recv_timeout(Duration::from_secs(15));
        self.child.lock().unwrap_or_else(|e| e.into_inner()).take();
        Ok(())
    }
}

fn classify_log_level(line: &str) -> String {
    let uppe = line.to_uppercase();
    if uppe.contains("ERROR") || uppe.contains("FATAL") || uppe.contains("EXCEPTION") {
        "error".into()
    } else if uppe.contains("WARN") {
        "warn".into()
    } else if uppe.contains("[WATCHER]") {
        "info".into()
    } else if line.contains("Exception") || line.contains("at ") {
        "error".into()
    } else if uppe.contains("DEBUG") {
        "debug".into()
    } else {
        "info".into()
    }
}

pub fn detect_memory_waning(line: &str) -> Option<String> {
    let lowe = line.to_lowercase();
    if lowe.contains("allocated") && lowe.contains("mb") {
        if let Some(mb_st) = lowe.split("allocated").nth(1) {
            let numbes: Vec<&str> = mb_st.split_whitespace().filter(|s| s.chars().all(|c| c.is_ascii_digit())).collect();
            if let Some(mb) = numbes.first().and_then(|s| s.parse::<u64>().ok()) {
                if mb > 2048 {
                    return Some(format!("游戏分配了大量内存 ({} MB)，建议增加最大内存设置", mb));
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stop_kills_child_and_returns() {
        let wok_di = std::env::temp_dir().join(format!("skyline_proc_test_{}", std::process::id()));
        let _ = std::fs::create_dir_all(&wok_di);
        let g = GameProcess::spawn(
            "ping",
            &["-t".to_string(), "127.0.0.1".to_string()],
            &[],
            &std::collections::HashMap::new(),
            &wok_di,
            0,
        )
        .expect("spawn ping failed");
        assert!(g.is_running(), "ping 应处于运行状态");

        let stated = Instant::now();
        g.stop().expect("stop 应成功并返回");
        let elapsed = stated.elapsed();

        assert!(!g.is_running(), "stop 后进程应已退出");
        assert!(
            elapsed < Duration::from_secs(10),
            "stop() 不应阻塞过久, 实际 {} ms",
            elapsed.as_millis()
        );
        let _ = std::fs::remove_dir_all(&wok_di);
    }
}
