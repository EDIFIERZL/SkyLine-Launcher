mod mc;
mod instance;
mod download;
mod modpack;
mod utils;
mod commands;
mod task;

use commands::auth::AuthState;
use commands::launch::{RunningGames, CancelledLaunches, PreloadedGames};
use commands::download::DownloadState;
use commands::terracotta::TerracottaPot;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use tauri::Manager;



static STARTUP_OPTIMIZED: AtomicBool = AtomicBool::new(false);

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_process::init())
        .manage(AuthState(Mutex::new(None)))
        .manage(RunningGames(Arc::new(Mutex::new(HashMap::new()))))
        .manage(CancelledLaunches(Arc::new(Mutex::new(HashMap::new()))))
        .manage(PreloadedGames(Arc::new(Mutex::new(HashMap::new()))))
        .manage(TerracottaPot(Mutex::new(None)))
        .manage(DownloadState(Arc::new(tokio::sync::Mutex::new(
            download::manager::DownloadManager::new(3, download::manager::DownloadSource::Auto)
        ))))
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            let launcher_root = utils::io::get_launcher_root();
            std::fs::create_dir_all(&launcher_root).ok();
            task::run_first_time_tasks();
            utils::io::ensure_minecraft_structure();

            
            
            {
                let handle = app.handle().clone();
                tauri::async_runtime::spawn(async move {
                    
                    tokio::time::sleep(std::time::Duration::from_millis(400)).await;
                    if let Ok(cfg) = crate::commands::settings::load_config() {
                        if let Some(last) = cfg.last_selected_instance {
                            let _ = crate::preload_last_instance(handle, last).await;
                        }
                    }
                });
            }

            std::thread::spawn(|| {
                
                
                
                crate::mc::nt_memory::optimize(true);
                std::thread::sleep(std::time::Duration::from_millis(200));
                crate::mc::nt_memory::optimize(true);
                std::thread::sleep(std::time::Duration::from_millis(200));
                crate::mc::nt_memory::optimize(true);
                std::thread::sleep(std::time::Duration::from_millis(100));
                crate::mc::nt_memory::optimize(true);
                STARTUP_OPTIMIZED.store(true, Ordering::SeqCst);
            });

            
            #[cfg(target_os = "windows")]
            {
                use webview2_com::AcceleratorKeyPressedEventHandler;
                use windows_sys::Win32::UI::Input::KeyboardAndMouse;

                if let Some(win) = app.get_webview_window("main") {
                    let webview = win.as_ref();
                    let _ = webview.with_webview(|platfrom_webview| {
                        let contolle = platfrom_webview.controller();
                        
                        if let Ok(webview) = unsafe { contolle.CoreWebView2() } {
                            if let Ok(settings) = unsafe { webview.Settings() } {
                                unsafe { settings.SetAreDevToolsEnabled(false) }.ok();
                            }

                            
                            let handle = AcceleratorKeyPressedEventHandler::create(Box::new(
                                move |_contolle, ags: Option<webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2AcceleratorKeyPressedEventArgs>| {
                                    if let Some(ags) = ags {
                                        let mut vk: u32 = 0;
                                        if unsafe { ags.VirtualKey(&mut vk) }.is_ok() {
                                            let ctl = (unsafe { KeyboardAndMouse::GetKeyState(0x11) } & 0x8000u16 as i16) != 0;
                                            let shift = (unsafe { KeyboardAndMouse::GetKeyState(0x10) } & 0x8000u16 as i16) != 0;
                                            
                                            let blocked = vk == 0x7B
                                                || (ctl && shift && (vk == 0x49 || vk == 0x4A || vk == 0x43))
                                                || (ctl && vk == 0x55); 
                                            if blocked {
                                                unsafe { ags.SetHandled(true) }.ok();
                                            }
                                        }
                                    }
                                    Ok(())
                                },
                            ));
                            
                            let token: &mut i64 = Box::leak(Box::new(0i64));
                            unsafe { contolle.add_AcceleratorKeyPressed(&handle, token) }.ok();
                        }
                    });
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::auth::login_offline,
            commands::auth::login_mojang,
commands::auth::microsoft_auth_start,
commands::auth::microsoft_auth_poll,
commands::auth::microsoft_auth_refresh,
commands::auth::littleskin_auth_status,
commands::auth::littleskin_auth_poll,
commands::auth::littleskin_auth_refresh,
            commands::auth::login_authlib,
            commands::auth::save_authlib_config,
            commands::auth::get_authlib_server_meta,
            commands::auth::get_skin_head,
            commands::auth::get_skin_textures,
            commands::auth::get_default_skin,
            commands::auth::get_authlib_skin,
            commands::auth::get_authlib_textures,
            commands::auth::save_custom_skin,
            commands::instance::list_instances,
            commands::instance::get_instance,
            commands::instance::open_instance_folder,
            commands::instance::create_instance,
            commands::instance::delete_instance,
            commands::instance::update_instance,
            commands::instance::install_game,
              commands::instance::install_game_multi,
            commands::instance::list_installed_versions,
            commands::instance::analyze_crash,
            commands::instance::fetch_versions,
            commands::instance::install_version,
            commands::instance::list_instance_folders,
            commands::instance::auto_scan_instance_folders,
            commands::instance::add_instance_folder,
            commands::instance::remove_instance_folder,
            commands::instance::list_home_instances,
            commands::instance::list_instance_worlds,
            commands::instance::list_multiplayer_servers,
            commands::instance::get_world_info,
            commands::instance::generate_map_preview,
            commands::instance::world_map_region,
            commands::instance::world_map_region_with_structures,
        commands::instance::seed_map_region,
        commands::instance::seed_biome_region,
        commands::instance::seed_biome_at,
        commands::instance::seed_results,
        commands::instance::list_screenshots,
        commands::instance::read_screenshot_base64,
            commands::instance::open_screenshot,
            commands::instance::open_file,
        commands::instance::delete_screenshot,
            commands::instance::get_active_instance_folder,
            commands::instance::set_active_instance_folder,
            commands::launch::detect_java,
            commands::launch::get_java_recommendation,
            commands::launch::get_total_memory,
            commands::launch::get_memory_used_percent,
            commands::launch::optimize_memory,
            commands::launch::find_best_java,
            commands::launch::remove_java,
            commands::launch::download_java,
            commands::launch::launch_game,
            commands::launch::preload_game,
            commands::launch::cancel_preload,
            commands::launch::stop_game,
            commands::launch::cancel_game_launch,
            commands::launch::query_server_status,
            commands::crash_ai::get_crash_file_path,
            commands::crash_ai::open_folder_select,
            commands::crash_ai::read_crash_file,
            commands::crash_ai::read_latest_log,
            commands::crash_ai::read_file_as_base64,
            commands::crash_ai::ai_chat,
            commands::crash_ai::save_agnes_api_key,
            commands::crash_ai::get_agnes_api_key_status,
            commands::crash_ai::analyze_crash_auto,
            commands::music::read_audio_file,
            commands::music::check_files_exist,
            commands::settings::load_config,
            commands::settings::save_config,
            commands::settings::set_last_selected_instance,
            commands::settings::read_background_media,
            commands::settings::set_game_folder,
            commands::settings::migrate_game_folder,
            commands::mods::scan_instance_mods,
            commands::mods::toggle_mod,
            commands::mods::delete_mod,
            commands::mods::scan_resource_packs,
            commands::mods::scan_shader_packs,
            commands::mods::scan_schematics,
            commands::mods::toggle_resource_pack,
            commands::mods::install_forge,
            commands::mods::install_neoforge,
            commands::mods::install_fabric,
            commands::mods::install_quilt,
            commands::mods::list_quilt_loader_versions,
            commands::mods::list_api_mod_versions,
            commands::mods::list_optifine_versions,
            commands::mods::list_all_optifine_versions,
            commands::mods::list_forge_versions,
            commands::mods::list_neoforge_versions,
            commands::mods::list_fabric_versions,
            commands::mods::list_fabic_loader_versions,
            commands::mods::install_optifine,
            commands::mods::install_sodium,
            commands::mods::search_mcmod,
            commands::mods::enrich_mcmod_batch,
            commands::mods::get_neoforge_version_id,
            commands::mods::batch_toggle_mods,
            commands::mods::batch_delete_mods,
            commands::mods::get_mod_details,
            commands::instance::scan_data_packs,
            commands::instance::toggle_data_pack,
            commands::instance::delete_data_pack,
            commands::instance::import_world_zip,
            commands::instance::import_world_from_url,
            commands::instance::delete_world,
            commands::modpack::search_modrinth_mods,
              commands::modpack::recommended_mods,
            commands::modpack::recommended_resource_packs,
            commands::modpack::recommended_shader_packs,
            commands::modpack::recommended_modpacks,
            commands::modpack::recommended_datapacks,
            commands::modpack::recommended_worlds,
            commands::modpack::get_modrinth_versions,
            commands::modpack::get_modrinth_project_detail,
            commands::modpack::download_modrinth_mod,
            commands::modpack::resolve_modrinth_dependencies,
            commands::modpack::download_file,
            commands::modpack::install_modrinth_modpack,
            commands::modpack::install_curseforge_modpack,
            commands::modpack::search_curseforge_mods,
            commands::modpack::search_curseforge_category,
            commands::modpack::get_curseforge_project,
            commands::modpack::get_curseforge_files,
            commands::modpack::export_modrinth_pack,
            commands::modpack::export_curseforge_pack,
            commands::modpack::import_modrinth_pack,
            commands::modpack::import_curseforge_pack,
            commands::modpack::import_mmc_pack,
            commands::modpack::import_hmcl_pack,
            commands::modpack::detect_modpack_type,
            commands::modpack::check_mod_updates,
            commands::modpack::search_resource_packs,
            commands::modpack::search_shader_packs,
            commands::modpack::search_datapacks,
            commands::modpack::search_worlds,
            commands::modpack::search_modpacks,
            commands::modpack::install_modrinth_content,
            commands::modpack::install_modrinth_map,
            commands::download::get_download_source,
            commands::download::set_download_source,
            commands::download::add_download_task,
            commands::download::start_download,
            commands::download::get_download_status,
            commands::download::get_all_downloads,
            commands::download::remove_download,
            commands::download::clear_completed_downloads,
            commands::download::retry_failed_downloads,
            commands::download::verify_file,
            commands::terracotta::launch_terracotta,
            commands::terracotta::ensure_terracotta_running,
            commands::terracotta::terracotta_state,
            commands::terracotta::terracotta_meta,
            commands::terracotta::terracotta_scanning,
            commands::terracotta::terracotta_guesting,
            commands::terracotta::terracotta_ide,
            commands::terracotta::terracotta_stop,
            commands::memory::optimize_memory_aggressive,
            commands::memory::optimize_memory_silent,
            commands::memory::optimize_memory_best,
            commands::memory::get_memory_usage,
            commands::memory::start_periodic_optimization,
            commands::memory::stop_periodic_optimization,
        ])
        .run(tauri::generate_context!())
        .expect("error while running SkyLine Launcher");
}


async fn preload_last_instance(handle: tauri::AppHandle, instance_id: String) -> Result<(), String> {
    let cfg = crate::commands::settings::load_config().map_err(|e| e.to_string())?;
    let instance = crate::instance::manager::get_instance(&instance_id)
        .map_err(|e| e.to_string())?
        .ok_or("instance not found")?;

    
    let vs = cfg.version_settings.get(&instance.version_id).cloned();
    let java_override = vs.as_ref().and_then(|v| v.java_path.clone()).or(instance.java_path.clone());
    let java = if let Some(ref path) = java_override {
        crate::mc::java::probe_java(path)
    } else {
        
        let required = crate::mc::java::get_required_java_version_from_profile(&instance.version_id)
            .unwrap_or_else(|| crate::mc::java::get_required_java_version(&instance.version_id));
        let all = crate::mc::java::detect_java_versions_cached();
        all.iter()
            .filter(|j| j.is_64bit && j.major_version >= required)
            .min_by_key(|j| j.major_version)
            .cloned()
    };
    let Some(java) = java else { return Ok(()) };

    let global_jvm_args = cfg.jvm_args.clone();
    let global_max_memory = cfg.max_memory;
    let opengl_compat = cfg.opengl_compat;

    let min_memory = vs.as_ref().and_then(|v| v.min_memory).unwrap_or(instance.min_memory);
    let max_memory = vs.as_ref().and_then(|v| v.max_memory).unwrap_or_else(|| {
        if instance.max_memory == 4096 { global_max_memory } else { instance.max_memory }
    });
    let isolation_mode = vs.as_ref().and_then(|v| v.isolation_mode.clone()).unwrap_or(instance.isolation_mode.clone());
    let launch_di = crate::instance::manager::get_instance_launch_dir(&instance);
    let _ = std::fs::create_dir_all(&launch_di);

    let mut jvm_args = instance.jvm_args.clone();
    for ag in &global_jvm_args {
        if !jvm_args.iter().any(|a| a == ag) {
            jvm_args.push(ag.clone());
        }
    }

    let config = crate::mc::launch::LaunchConfig {
        version_id: instance.version_id.clone(),
        java,
        auth: crate::mc::auth::AuthSession {
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
        instance_dir: launch_di,
        window_width: instance.window_width,
        window_height: instance.window_height,
        custom_resolution: instance.custom_resolution,
        fullscreen: false,
        server: instance.server_ip.clone(),
        quick_world: None,
        isolation_mode,
        modloader: instance.modloader.clone(),
        minecraft_root: instance.minecraft_root.as_ref().map(std::path::PathBuf::from),
        game_dir_override: vs
            .as_ref()
            .and_then(|v| v.game_dir_override.clone())
            .map(std::path::PathBuf::from)
            .or(instance.game_dir_override.as_ref().map(std::path::PathBuf::from)),
        process_priority: crate::mc::launch::ProcessPriority::Normal,
        pre_launch_command: None,
        post_exit_command: None,
        join_server_at_launch: false,
        opengl_compat,
        preloaded_files: None,
    };

    let result = crate::mc::launch::prepare_game_files(&config).await?;

    
    let preloaderd = handle.state::<commands::launch::PreloadedGames>();
    let mut map = preloaderd.0.lock().map_err(|e| e.to_string())?;
    map.insert(instance_id, result.clone());

    
    
    
    let warm_files = result.clone();
    tauri::async_runtime::spawn(async move {
        
        for _ in 0..80 {
            if STARTUP_OPTIMIZED.load(Ordering::SeqCst) {
                break;
            }
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        }
        let _ = std::thread::Builder::new()
            .name("jvm-prewarm".into())
            .spawn(move || {
                let wam_paths = crate::mc::launch::collect_prewarm_paths(&warm_files);
                let _ = crate::mc::launch::prewarm_files(&wam_paths, 3 * 1024 * 1024 * 1024);
            });
    });

    Ok(())
}






