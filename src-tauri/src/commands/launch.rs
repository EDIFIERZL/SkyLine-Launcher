use crate::instance::manager;
use crate::mc::auth::AuthSession;
use crate::mc::java::{self, JavaInfo, JavaRecommendation};
use crate::mc::launch::{self, LaunchConfig, PreparedFiles};
use crate::mc::process::{GameProcessInfo, GameExitInfo, ExitReason, LaunchStage, LaunchProgressEvent};
use crate::mc::crash;
use crate::mc::hardware;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use tauri::{Emitter, State};
use std::collections::HashMap;

pub struct RunningGames(pub Arc<Mutex<HashMap<String, Arc<Mutex<Option<crate::mc::process::GameProcess>>>>>>);

pub struct CancelledLaunches(pub Arc<Mutex<HashMap<String, Arc<AtomicBool>>>>);




pub struct PreloadedGames(pub Arc<Mutex<HashMap<String, PreparedFiles>>>);


static PRELOAD_OPTIMIZING: AtomicBool = AtomicBool::new(false);

/// 解析世界引用为 Minecraft --quickPlaySingleplayer 需要的「世界文件夹名」。
/// 支持传入：完整路径、saves 相对路径、或纯文件夹名（大小写不敏感）。
fn resolve_quick_world_folder(world_ref: &str, game_dir: Option<&std::path::PathBuf>, version_id: &str) -> Option<String> {
    // --quickPlaySingleplayer 仅在 1.19.4+ 支持
    if crate::mc::version::is_version_less(version_id, "1.19.4") {
        log::warn!("[quickplay] version {} < 1.19.4, quickplay unsupported", version_id);
        return None;
    }

    // 提取文件夹名（兼容完整路径 / saves 相对路径 / 纯文件夹名）
    let input_path = std::path::PathBuf::from(world_ref);
    let is_path = input_path.is_absolute() || world_ref.contains('\\') || world_ref.contains('/');
    let folder_candidate = if is_path {
        input_path.file_name().map(|n| n.to_string_lossy().to_string())
    } else {
        Some(world_ref.to_string())
    };

    // 若传入完整路径且确实存在，直接用其文件夹名
    if input_path.is_absolute() && input_path.is_dir() {
        let folder = folder_candidate.clone();
        log::info!("[quickplay] resolved existing path {:?} -> folder {:?}", input_path, folder);
        return folder;
    }

    let candidates: Vec<std::path::PathBuf> = match game_dir {
        Some(d) => vec![d.clone()],
        None => vec![crate::utils::io::get_minecraft_di()],
    };
    for base in &candidates {
        let search_dirs = [
            base.join("saves"),
            base.join("versions").join(version_id).join("saves"),
        ];
        for saves in search_dirs.iter().filter(|d| d.exists()) {
            if let Ok(entries) = std::fs::read_dir(saves) {
                for entry in entries.flatten() {
                    if !entry.path().is_dir() { continue; }
                    let s = entry.file_name().to_string_lossy().to_string();
                    if s == world_ref || s.to_lowercase() == world_ref.to_lowercase() {
                        log::info!("[quickplay] matched saves folder {:?} for ref {:?}", s, world_ref);
                        return Some(s);
                    }
                    if let Some(fc) = &folder_candidate {
                        if s == *fc || s.to_lowercase() == fc.to_lowercase() {
                            log::info!("[quickplay] matched folder {:?} for ref {:?} (folder_candidate {:?})", s, world_ref, fc);
                            return Some(s);
                        }
                    }
                }
            }
        }
    }
    // 最终回退：通过读取每个存档的 level.dat LevelName 匹配（前端可能传 LevelName）
    for base in &candidates {
        if let Ok(worlds) = crate::mc::world::scan_worlds(base) {
            for w in &worlds {
                if w.name.eq_ignore_ascii_case(world_ref)
                    || w.name.to_lowercase() == world_ref.to_lowercase() {
                    let folder = std::path::PathBuf::from(&w.path)
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string());
                    log::info!("[quickplay] matched level name {:?} for ref {:?} -> folder {:?}", w.name, world_ref, folder);
                    return folder;
                }
            }
        }
    }
    log::warn!("[quickplay] could not resolve world {:?} in saves", world_ref);
    None
}

#[tauri::command]
pub async fn preload_game(
    instance_id: String,
    app_handle: tauri::AppHandle,
    preloaderd_games: State<'_, PreloadedGames>,
) -> Result<(), String> {
    log::info!("[preload] start for instance {}", instance_id);
    let instance = manager::get_instance(&instance_id)?
        .ok_or_else(|| "Instance not found".to_string())?;

    // Resolve Java using the same detection logic as launch, but never auto-download.
    let vs = load_version_settings(&instance.version_id);
    let java_override = vs.as_ref().and_then(|v| v.java_path.clone()).or(instance.java_path.clone());
    let java = if let Some(ref path) = java_override {
        java::probe_java(path)
    } else {
        best_detected_java(&instance)
    };
    let Some(java) = java else { return Ok(()) };

    let global_config = crate::commands::settings::load_config().ok();
    let global_jvm_args = global_config.as_ref().map(|c| c.jvm_args.clone()).unwrap_or_default();
    let global_max_memory = global_config.as_ref().map(|c| c.max_memory).unwrap_or(4096);
    let opengl_compat = global_config.as_ref().map(|c| c.opengl_compat).unwrap_or(false);

    let min_memory = vs.as_ref().and_then(|v| v.min_memory).unwrap_or(instance.min_memory);
    let max_memory = vs.as_ref().and_then(|v| v.max_memory).unwrap_or_else(|| {
        if instance.max_memory == 4096 { global_max_memory } else { instance.max_memory }
    });
    let isolation_mode = vs.as_ref().and_then(|v| v.isolation_mode.clone()).unwrap_or(instance.isolation_mode.clone());

    let launch_dir = manager::get_instance_launch_dir(&instance);
    let _ = std::fs::create_dir_all(&launch_dir);

    let mut jvm_args = instance.jvm_args.clone();
    for arg in &global_jvm_args {
        if !jvm_args.iter().any(|a| a == arg) {
            jvm_args.push(arg.clone());
        }
    }

    let server_target = instance.server_ip.clone();

    let config = LaunchConfig {
        version_id: instance.version_id.clone(),
        java,
        auth: AuthSession {
            access_token: String::new(),
            username: String::new(),
            uuid: String::new(),
            user_type: String::new(),
            refresh_token: None,
            expires_at: None,
        },
        min_memory,
        max_memory,
        jvm_args,
        game_args: instance.game_args.clone(),
        instance_dir: launch_dir,
        window_width: instance.window_width,
        window_height: instance.window_height,
        custom_resolution: instance.custom_resolution,
        fullscreen: false,
        server: server_target,
        quick_world: None,
        isolation_mode,
        modloader: instance.modloader.clone(),
        minecraft_root: instance.minecraft_root.as_ref().map(std::path::PathBuf::from),
        game_dir_override: vs
            .as_ref()
            .and_then(|v| v.game_dir_override.clone())
            .map(std::path::PathBuf::from)
            .or(instance.game_dir_override.as_ref().map(std::path::PathBuf::from)),
        process_priority: launch::ProcessPriority::Normal,
        pre_launch_command: None,
        post_exit_command: None,
        join_server_at_launch: false,
        opengl_compat,
        preloaded_files: None,
    };

    // Run expensive file preparation (classpath/natives/profile) in background
    let rresult = launch::prepare_game_files(&config).await?;

    // Cache it for instant launch
    {
        let mut map = preloaderd_games.0.lock().map_err(|e| e.to_string())?;
        map.insert(instance_id.clone(), rresult.clone());
    }

    let _ = app_handle.emit("preload-complete", &instance_id);

    // Preload finished -> free memory, then build CDS archive, then warm Java + mods.
    // Warming MUST happen AFTER optimization, which purges the standby list.
    let app = app_handle.clone();
    let sid = instance_id.clone();
    let files = rresult.clone();
    std::thread::spawn(move || {
        if !PRELOAD_OPTIMIZING.swap(true, Ordering::SeqCst) {
            let _ = crate::mc::hardware::optimize_system_memory_ex(true);
            PRELOAD_OPTIMIZING.store(false, Ordering::SeqCst);
        }
        // Warm Java binary, JVM dlls, classpath jars, natives and mod jars.
        let warm_paths = crate::mc::launch::collect_prewarm_paths(&files);
        let _ = crate::mc::launch::prewarm_files(&warm_paths, 3 * 1024 * 1024 * 1024);
        let _ = app.emit("preload-optimized", &sid);
    });

    Ok(())
}

/// Pick the closest installed Java for the instance without any download side effects.
fn best_detected_java(instance: &crate::instance::Instance) -> Option<JavaInfo> {
    let required = java::get_required_java_version_from_profile(&instance.version_id)
        .unwrap_or_else(|| java::get_required_java_version(&instance.version_id));
    let all = java::detect_java_versions_cached();
    if let Some(j) = all.iter().find(|j| j.major_version == required && j.is_64bit) {
        return Some(j.clone());
    }
    all.iter()
        .filter(|j| j.is_64bit && j.major_version >= required)
        .min_by_key(|j| j.major_version)
        .cloned()
}

#[tauri::command]
pub async fn cancel_preload(
    instance_id: String,
    preloaderd_games: State<'_, PreloadedGames>,
) -> Result<(), String> {
    let mut map = preloaderd_games.0.lock().map_err(|e| e.to_string())?;
    map.remove(&instance_id);
    Ok(())
}

#[tauri::command]
pub async fn detect_java() -> Result<Vec<JavaInfo>, String> {
    Ok(java::detect_java_versions_cached())
}

#[tauri::command]
pub async fn get_total_memory() -> Result<u64, String> {
    Ok(hardware::get_total_memory_mb())
}

#[tauri::command]
pub async fn get_memory_used_percent() -> Result<u64, String> {
    Ok(hardware::get_memory_used_percent())
}

#[tauri::command]
pub async fn optimize_memory() -> Result<(u64, u64), String> {
    let es = tokio::task::spawn_blocking(|| hardware::optimize_system_memory_ex(true))
        .await
        .map_err(|e| e.to_string())?;
    Ok((es.afte_percent, es.feed_mb))
}

#[tauri::command]
pub async fn get_java_recommendation(
    minecraft_version: String,
    modloader: Option<String>,
) -> Result<JavaRecommendation, String> {
    Ok(java::get_java_recommendation(&minecraft_version, modloader.as_deref()))
}

#[tauri::command]
pub async fn find_best_java(
    minecraft_version: String,
    modloader: Option<String>,
) -> Result<Option<JavaInfo>, String> {
    Ok(java::find_best_java(&minecraft_version, modloader.as_deref()))
}

#[tauri::command]
pub async fn remove_java(java_path: String) -> Result<(), String> {
    java::remove_java(&java_path)
}

#[tauri::command]
pub async fn download_java(major_version: u32, app_handle: tauri::AppHandle) -> Result<String, String> {
    let app = app_handle.clone();
    java::ensure_java(major_version, move |msg, pog| {
        let _ = app.emit(
            "install-progress",
            &crate::mc::install::InstallProgress {
                stage: "java".into(),
                progress: pog,
                message: msg,
            },
        );
    })
    .await
}

fn load_version_settings(version_id: &str) -> Option<crate::commands::settings::VersionSetting> {
    crate::commands::settings::load_config()
        .ok()
        .and_then(|c| c.version_settings.get(version_id).cloned())
}

async fn ensure_best_java(
    instance: &crate::instance::Instance,
    app_handle: &tauri::AppHandle,
) -> Result<JavaInfo, String> {
    let required = java::get_required_java_version_from_profile(&instance.version_id)
        .unwrap_or_else(|| java::get_required_java_version(&instance.version_id));
    let all = java::detect_java_versions_cached();

    if let Some(j) = all.iter().find(|j| j.major_version == required && j.is_64bit) {
        return Ok(j.clone());
    }
    if let Some(j) = all
        .iter()
        .filter(|j| j.is_64bit && j.major_version >= required)
        .min_by_key(|j| j.major_version)
    {
        return Ok(j.clone());
    }

    let app = app_handle.clone();
    let jpath = java::ensure_java(required, move |msg, pog| {
        let _ = app.emit(
            "install-progress",
            &crate::mc::install::InstallProgress {
                stage: "java".into(),
                progress: pog,
                message: msg,
            },
        );
    })
    .await
    .map_err(|e| e.to_string())?;

    Ok(java::probe_java(&jpath).unwrap_or(JavaInfo {
        path: jpath,
        version: "unknown".into(),
        major_version: required,
        is_64bit: true,
        architecture: "x64".into(),
        vendor: "Unknown".into(),
        is_jdk: false,
    }))
}

#[tauri::command]
pub async fn launch_game(
    instance_id: String,
    auth: AuthSession,
    app_handle: tauri::AppHandle,
    running_games: State<'_, RunningGames>,
    cancelled_launches: State<'_, CancelledLaunches>,
    preloaderd_games: State<'_, PreloadedGames>,
    quick_world: Option<String>,
    quick_server: Option<String>,
) -> Result<GameProcessInfo, String> {
    let launch_start = std::time::Instant::now();
    log::info!("[launch] start for instance {} quick_world={:?} quick_server={:?}", instance_id, quick_world, quick_server);
    let cancel_flag = {
        let mut map = cancelled_launches.0.lock().map_err(|e| e.to_string())?;
        map.entry(instance_id.clone())
            .or_insert_with(|| Arc::new(AtomicBool::new(false)))
            .clone()
    };
    let check_cancelled = || -> Result<(), String> {
        if cancel_flag.load(Ordering::SeqCst) {
            Err("启动已取消".to_string())
        } else {
            Ok(())
        }
    };

    // ═══════════════════════════════════════════════════════════════════════
    // FAST PATH: check preload cache FIRST — skip ALL Java/config/instance I/O
    // ═══════════════════════════════════════════════════════════════════════
    let preloaderd = {
        let mut map = preloaderd_games.0.lock().map_err(|e| e.to_string())?;
        map.remove(&instance_id)
    };

    let config = if let Some(cached) = preloaderd {
        // ── Cache HIT: build LaunchConfig from cached values + fresh auth ──
        // jvm_args already contains merged instance+global args from preload.
        // Authlib JVM args are pre-cached (independent of auth token).
        let mut jvm_args = cached.jvm_args.clone();
        jvm_args.extend(cached.authlib_jvm_args.iter().cloned());

        let quick_world_path = quick_world.and_then(|w| resolve_quick_world_folder(&w, cached.game_dir_override.as_ref(), &cached.version_id).map(std::path::PathBuf::from));
        let server_target = quick_server.clone();
        let join_server_at_launch = server_target.is_some();

        LaunchConfig {
            version_id: cached.version_id.clone(),
            java: JavaInfo {
                path: cached.java_path.clone(),
                version: String::new(),
                major_version: cached.java_major_version,
                is_64bit: true,
                architecture: "x64".into(),
                vendor: "Cached".into(),
                is_jdk: false,
            },
            auth,
            min_memory: cached.min_memory,
            max_memory: cached.max_memory,
            jvm_args,
            game_args: cached.game_args.clone(),
            instance_dir: cached.instance_dir.clone(),
            window_width: cached.window_width,
            window_height: cached.window_height,
            custom_resolution: cached.custom_resolution,
            fullscreen: false,
            server: server_target,
            quick_world: quick_world_path,
            isolation_mode: cached.isolation_mode.clone(),
            modloader: cached.modloader.clone(),
            minecraft_root: cached.minecraft_root.clone(),
            game_dir_override: cached.game_dir_override.clone(),
            process_priority: cached.process_priority.clone(),
            pre_launch_command: None,
            post_exit_command: None,
            join_server_at_launch,
            opengl_compat: cached.opengl_compat,
            preloaded_files: Some(cached),
        }
    } else {
        // ── Cache MISS: full preparation path (slow) ──────────────────────
        let instance = manager::get_instance(&instance_id)?
            .ok_or_else(|| "Instance not found".to_string())?;

        let vs = load_version_settings(&instance.version_id);
        let java_override = vs.as_ref().and_then(|v| v.java_path.clone()).or(instance.java_path.clone());

        let java = if let Some(ref path) = java_override {
            match java::probe_java(path) {
                Some(j) => j,
                None => ensure_best_java(&instance, &app_handle).await?,
            }
        } else {
            ensure_best_java(&instance, &app_handle).await?
        };
        check_cancelled()?;

        let launch_dir = manager::get_instance_launch_dir(&instance);
        std::fs::create_dir_all(&launch_dir).map_err(|e| e.to_string())?;

        let global_config = crate::commands::settings::load_config().ok();
        let global_jvm_args = global_config.as_ref().map(|c| c.jvm_args.clone()).unwrap_or_default();
        let global_max_memory = global_config.as_ref().map(|c| c.max_memory).unwrap_or(4096);
        let opengl_compat = global_config.as_ref().map(|c| c.opengl_compat).unwrap_or(false);

        let min_memory = vs.as_ref().and_then(|v| v.min_memory).unwrap_or(instance.min_memory);
        let max_memory = vs.as_ref().and_then(|v| v.max_memory).unwrap_or_else(|| {
            if instance.max_memory == 4096 { global_max_memory } else { instance.max_memory }
        });
        let isolation_mode = vs.as_ref().and_then(|v| v.isolation_mode.clone()).unwrap_or(instance.isolation_mode.clone());

        let mut jvm_args = instance.jvm_args.clone();
        for arg in &global_jvm_args {
            if !jvm_args.iter().any(|a| a == arg) {
                jvm_args.push(arg.clone());
            }
        }
        if auth.user_type == "authlib" {
            if !jvm_args.iter().any(|arg| arg.starts_with("-javaagent:") && arg.contains("authlib-injector")) {
                let authlib_config_path = launch_dir.join(".skyline").join("authlib.json");
                if let Ok(config_str) = std::fs::read_to_string(&authlib_config_path) {
                    if let Ok(config) = serde_json::from_str::<serde_json::Value>(&config_str) {
                        if let Some(server_url) = config["server_url"].as_str() {
                            let _ = crate::mc::authlib::ensure_authlib_jar();
                            let prefetched = config["prefetched_meta"].as_str();
                            let authlib_args = crate::mc::authlib::get_authlib_jvm_args(server_url, prefetched);
                            for arg in authlib_args {
                                if !jvm_args.contains(&arg) {
                                    jvm_args.push(arg);
                                }
                            }
                        }
                    }
                }
            }
        }

        let quick_world_path = quick_world.and_then(|w| resolve_quick_world_folder(&w, instance.game_dir_override.as_ref().map(|s| std::path::PathBuf::from(s)).as_ref(), &instance.version_id).map(std::path::PathBuf::from));
        let quick_server_target = quick_server.clone();
        let server_target = quick_server.or(instance.server_ip.clone());
        let join_server_at_launch = quick_server_target.is_some();

        LaunchConfig {
            version_id: instance.version_id.clone(),
            java,
            auth,
            min_memory,
            max_memory,
            jvm_args,
            game_args: instance.game_args.clone(),
            instance_dir: launch_dir,
            window_width: instance.window_width,
            window_height: instance.window_height,
            custom_resolution: instance.custom_resolution,
            fullscreen: false,
            server: server_target.clone(),
            quick_world: quick_world_path,
            isolation_mode,
            modloader: instance.modloader.clone(),
            minecraft_root: instance.minecraft_root.as_ref().map(std::path::PathBuf::from),
            game_dir_override: vs
                .as_ref()
                .and_then(|v| v.game_dir_override.clone())
                .map(std::path::PathBuf::from)
                .or(instance.game_dir_override.as_ref().map(std::path::PathBuf::from)),
            process_priority: launch::ProcessPriority::Normal,
            pre_launch_command: None,
            post_exit_command: None,
            join_server_at_launch,
            opengl_compat,
            preloaded_files: None,
        }
    };

    log::info!("[launch] {} cache_build took {} ms (cache_hit={})", instance_id, launch_start.elapsed().as_millis(), config.preloaded_files.is_some());
    check_cancelled()?;

    let jvm_start = std::time::Instant::now();
    let mut game = launch::launch_minecraft(config).await?;
    log::info!("[launch] {} jvm_spawn took {} ms", instance_id, jvm_start.elapsed().as_millis());
    check_cancelled().map_err(|e| {
        let _ = game.stop();
        e
    })?;
    let pid = game.pid;
    let log_rx = game.log_rx.take().expect("log_rx already taken");

    let game_arc = Arc::new(Mutex::new(Some(game)));
    let mut games = running_games.0.lock().map_err(|e| e.to_string())?;

    let _ = app_handle.emit("launch-progress", &LaunchProgressEvent {
        instance_id: instance_id.clone(),
        stage: LaunchStage::JvmStarting,
        message: "JVM 启动中...".into(),
    });

    let game_for_thread = game_arc.clone();
    let started_at = std::time::Instant::now();
    let cancellations_for_exit = cancelled_launches.0.clone();
    let app_for_exit = app_handle.clone();
    let instance_for_exit = instance_id.clone();
    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;

        let _ = app_for_exit.emit("launch-progress", &LaunchProgressEvent {
            instance_id: instance_for_exit.clone(),
            stage: LaunchStage::GameLoading,
            message: "游戏加载中...".into(),
        });

        let mut window_found_once = false;
        let mut check_count = 0u32;

        loop {
            check_count += 1;
            let (is_running, exit_info, has_window) = {
                let lock = game_for_thread.lock().unwrap_or_else(|e| e.into_inner());
                if let Some(ref g) = *lock {
                    (g.is_running(), g.get_exit_info(), g.has_window())
                } else {
                    (false, None, false)
                }
            };

            if has_window && !window_found_once {
                window_found_once = true;
                let _ = app_for_exit.emit("launch-progress", &LaunchProgressEvent {
                    instance_id: instance_for_exit.clone(),
                    stage: LaunchStage::Running,
                    message: "游戏窗口已出现，运行中".into(),
                });
            }

            if !window_found_once && check_count % 5 == 0 {
                let _ = app_for_exit.emit("launch-progress", &LaunchProgressEvent {
                    instance_id: instance_for_exit.clone(),
                    stage: LaunchStage::WaitingWindow,
                    message: "等待游戏窗口出现...".into(),
                });
            }

            if !is_running {
                let play_time = started_at.elapsed().as_secs();

                let reason = if let Some((_, ref exit_reason)) = exit_info {
                    exit_reason.clone()
                } else if !window_found_once {
                    ExitReason::Crash
                } else {
                    ExitReason::Normal
                };

                if let Ok(Some(mut inst)) = manager::get_instance(&instance_for_exit) {
                    manager::record_play_time(
                        &mut inst,
                        play_time,
                        chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                    );

                    if matches!(reason, ExitReason::Crash | ExitReason::NoWindow) {
                        let game_dir = crate::instance::manager::get_instance_mc_dir(&instance_for_exit)
                            .unwrap_or_else(|_| std::path::PathBuf::from("."));
                        let _ = crash::mark_abnormal(&game_dir);
                    }
                }

                let exit_info = GameExitInfo {
                    instance_id: instance_for_exit.clone(),
                    exit_code: exit_info.and_then(|(code, _)| code),
                    reason,
                    play_time_secs: play_time,
                };
                let _ = app_for_exit.emit("game-stopped", &exit_info);
                if let Ok(mut map) = cancellations_for_exit.lock() {
                    map.remove(&instance_for_exit);
                }
                break;
            }
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    });

    let instance_id_log = instance_id.clone();
    tokio::spawn(async move {
        while let Ok(entry) = log_rx.recv() {
            let _ = app_handle.emit("game-log", &(instance_id_log.clone(), entry));
        }
    });

    games.insert(instance_id.clone(), game_arc.clone());
    log::info!("[launch] {} total launch_game took {} ms", instance_id, launch_start.elapsed().as_millis());

    Ok(GameProcessInfo {
        pid,
        running: true,
    })
}

#[tauri::command]
pub async fn stop_game(
    instance_id: String,
    running_games: State<'_, RunningGames>,
    cancelled_launches: State<'_, CancelledLaunches>,
) -> Result<(), String> {
    if let Ok(mut map) = cancelled_launches.0.lock() {
        map.remove(&instance_id);
    }
    let mut games = running_games.0.lock().map_err(|e| e.to_string())?;
    if let Some(game) = games.remove(&instance_id) {
        drop(games);
        let g = game.lock().unwrap_or_else(|e| e.into_inner()).take();
        if let Some(g) = g {
            g.stop()?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn cancel_game_launch(
    instance_id: String,
    running_games: State<'_, RunningGames>,
    cancelled_launches: State<'_, CancelledLaunches>,
) -> Result<(), String> {
    if let Ok(mut map) = cancelled_launches.0.lock() {
        map.entry(instance_id.clone())
            .or_insert_with(|| Arc::new(AtomicBool::new(false)))
            .store(true, Ordering::SeqCst);
    }
    let mut games = running_games.0.lock().map_err(|e| e.to_string())?;
    if let Some(game) = games.remove(&instance_id) {
        drop(games);
        let g = game.lock().unwrap_or_else(|e| e.into_inner()).take();
        if let Some(g) = g {
            g.stop()?;
        }
    }
    Ok(())
}

fn cache_server_favicon(favicon: &str) -> Option<String> {
    use base64::Engine;
    let trimmed = favicon.trim();
    let re = regex::Regex::new(r"(?i)^data:image/(?:png|jpe?g|gif|webp);base64,([A-Za-z0-9+/=\s]+)$").ok()?;
    let caps = re.captures(trimmed)?;
    let clean: String = caps.get(1)?.as_str().chars().filter(|c| !c.is_whitespace()).collect();
    let bytes = base64::engine::general_purpose::STANDARD.decode(&clean).ok()?;
    if bytes.is_empty() {
        return None;
    }
    let ext = if bytes.starts_with(b"\x89PNG") {
        "png"
    } else if bytes.starts_with(b"\xFF\xD8") {
        "jpg"
    } else if bytes.starts_with(b"GIF8") {
        "gif"
    } else if bytes.starts_with(b"RIFF") {
        "webp"
    } else {
        "png"
    };
    let dir = std::env::temp_dir().join("skyline-launcher").join("server-icons");
    std::fs::create_dir_all(&dir).ok()?;
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    trimmed.hash(&mut hasher);
    let filename = format!("{:016x}.{}", hasher.finish(), ext);
    let path = dir.join(&filename);
    if !path.exists() {
        std::fs::write(&path, &bytes).ok()?;
    }
    Some(path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn query_server_status(address: String) -> Result<crate::mc::server_status::ServerStatus, String> {
    let (host, port) = if let Some((h, p)) = address.rsplit_once(':') {
        (h.to_string(), p.parse::<u16>().unwrap_or(25565))
    } else {
        (address, 25565)
    };

    // Run blocking network I/O in a blocking thread
    let mut status = tokio::task::spawn_blocking(move || {
        crate::mc::server_status::query_server(&host, port)
    })
    .await
    .map_err(|e| e.to_string())?;

    if let Some(fav) = status.favicon.clone() {
        if let Some(path) = cache_server_favicon(&fav) {
            status.favicon_path = Some(path);
        }
    }

    Ok(status)
}
