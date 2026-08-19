use crate::instance::IsolationMode;
use crate::mc::auth::AuthSession;
use crate::mc::java::JavaInfo;
use crate::mc::library;
use crate::mc::version::{Argument, ArgumentValue, VersionProfile};
use crate::mc::process::GameProcess;
use crate::utils::io;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

const DEFAULT_LOG4J2_XML: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<Configuration status="WARN">
    <Appenders>
        <Console name="Console" target="SYSTEM_OUT">
            <PatternLayout pattern="[%d{HH:mm:ss}] [%t/%level]: %msg%n"/>
        </Console>
        <RollingFile name="File" fileName="logs/latest.log" filePattern="logs/%d{yyyy-MM-dd}-%i.log.gz">
            <PatternLayout pattern="[%d{HH:mm:ss}] [%t/%level]: %msg%n"/>
            <Policies>
                <TimeBasedTriggeringPolicy/>
                <SizeBasedTriggeringPolicy size="10MB"/>
            </Policies>
        </RollingFile>
    </Appenders>
    <Loggers>
        <Root level="info">
            <AppenderRef ref="Console"/>
            <AppenderRef ref="File"/>
        </Root>
    </Loggers>
</Configuration>"#;

pub struct LaunchConfig {
    pub version_id: String,
    pub java: JavaInfo,
    pub auth: AuthSession,
    pub min_memory: u32,
    pub max_memory: u32,
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
    pub instance_dir: PathBuf,
    pub window_width: u32,
    pub window_height: u32,
    pub server: Option<String>,
    pub quick_world: Option<PathBuf>,
    pub custom_resolution: bool,
    pub fullscreen: bool,
    pub isolation_mode: IsolationMode,
    pub modloader: crate::instance::ModLoader,
    pub minecraft_root: Option<PathBuf>,
    pub game_dir_override: Option<PathBuf>,
    pub process_priority: ProcessPriority,
    pub pre_launch_command: Option<String>,
    pub post_exit_command: Option<String>,
    pub join_server_at_launch: bool,
    pub opengl_compat: bool,
    
    
    
    pub preloaded_files: Option<PreparedFiles>,
}




#[derive(Debug, Clone)]
pub struct PreparedFiles {
    pub profile: VersionProfile,
    pub classpath: Vec<String>,
    pub natives_di: PathBuf,
    pub game_di: PathBuf,
    
    pub java_path: String,
    pub java_major_version: u32,
    pub version_id: String,
    pub min_memory: u32,
    pub max_memory: u32,
    pub jvm_args: Vec<String>,    
    pub game_args: Vec<String>,
    pub instance_dir: PathBuf,
    pub window_width: u32,
    pub window_height: u32,
    pub custom_resolution: bool,
    pub isolation_mode: IsolationMode,
    pub modloader: crate::instance::ModLoader,
    pub minecraft_root: Option<PathBuf>,
    pub game_dir_override: Option<PathBuf>,
    pub process_priority: ProcessPriority,
pub opengl_compat: bool,
    
    pub authlib_jvm_args: Vec<String>,
    
    pub cds_archive_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ProcessPriority {
    Low,
    BelowNormal,
    Normal,
    AboveNormal,
    High,
}

impl ProcessPriority {
    pub fn to_os_priority(&self) -> i32 {
        match self {
            ProcessPriority::Low => 6,        
            ProcessPriority::BelowNormal => 16, 
            ProcessPriority::Normal => 32,     
            ProcessPriority::AboveNormal => 16384, 
            ProcessPriority::High => 128,      
        }
    }

    pub fn as_label(&self) -> &str {
        match self {
            ProcessPriority::Low => "低",
            ProcessPriority::BelowNormal => "低于正常",
            ProcessPriority::Normal => "正常",
            ProcessPriority::AboveNormal => "高于正常",
            ProcessPriority::High => "高",
        }
    }
}

impl Default for LaunchConfig {
    fn default() -> Self {
        Self {
            version_id: String::new(),
            java: JavaInfo {
                path: "java".into(),
                version: String::new(),
                major_version: 21,
                is_64bit: true,
                architecture: "x64".into(),
                vendor: "Unknown".into(),
                is_jdk: false,
            },
            auth: AuthSession {
                access_token: "0".into(),
                username: "Player".into(),
                uuid: "00000000-0000-0000-0000-000000000000".into(),
                user_type: "mojang".into(),
                refresh_token: None,
                expires_at: None,
            },
            min_memory: 1024,
            max_memory: 4096,
            jvm_args: Vec::new(),
            game_args: Vec::new(),
            instance_dir: PathBuf::from("."),
            window_width: 854,
            window_height: 480,
            server: None,
            quick_world: None,
            custom_resolution: false,
            fullscreen: false,
            isolation_mode: IsolationMode::Modded,
            modloader: crate::instance::ModLoader::Vanilla,
            minecraft_root: None,
            game_dir_override: None,
            process_priority: ProcessPriority::Normal,
            pre_launch_command: None,
            post_exit_command: None,
            join_server_at_launch: false,
            opengl_compat: false,
            preloaded_files: None,
        }
    }
}

pub fn get_game_di(config: &LaunchConfig) -> PathBuf {
    if let Some(ref override_di) = config.game_dir_override {
        return override_di.clone();
    }
    io::get_minecraft_di()
}

fn get_versions_dir(config: &LaunchConfig) -> PathBuf {
    config
        .minecraft_root
        .as_ref()
        .map(|r| r.join("versions"))
        .unwrap_or_else(io::get_versions_dir)
}

fn get_libraries_dir(config: &LaunchConfig) -> PathBuf {
    config
        .minecraft_root
        .as_ref()
        .map(|r| r.join("libraries"))
        .unwrap_or_else(io::get_libraries_dir)
}

fn get_assets_dir(config: &LaunchConfig) -> PathBuf {
    if let Some(oot) = &config.minecraft_root {
        let external = oot.join("assets");
        if external.join("indexes").is_dir() || external.join("objects").is_dir() {
            return external;
        }
    }
    io::get_assets_dir()
}

fn profile_roots(config: &LaunchConfig) -> Vec<PathBuf> {
    let mut oots: Vec<PathBuf> = vec![get_versions_dir(config)];
    let shared = io::get_versions_dir();
    if !oots.iter().any(|o| o == &shared) {
        oots.push(shared);
    }
    let game_versions = get_game_di(config).join("versions");
    if !oots.iter().any(|o| o == &game_versions) {
        oots.push(game_versions);
    }
    if let Some(ref gd) = config.game_dir_override {
        let di = PathBuf::from(gd);
        if !oots.iter().any(|o| o == &di) {
            oots.push(di);
        }
    }
    oots
}

fn resolve_library_path(config: &LaunchConfig, el: &str) -> Option<PathBuf> {
    let primary = get_libraries_dir(config).join(el);
    if primary.exists() {
        return Some(primary);
    }
    let fallback = io::get_libraries_dir().join(el);
    if fallback.exists() {
        return Some(fallback);
    }
    if let Ok(cfg) = crate::commands::settings::load_config() {
        for folder in cfg.instance_folders {
            let p = PathBuf::from(crate::utils::io::strip_extended_prefix(&folder))
                .join("libraries")
                .join(el);
            if p.exists() {
                return Some(p);
            }
        }
    }
    None
}

pub async fn prepare_launch(
    config: &LaunchConfig,
) -> Result<(Vec<String>, Vec<String>, HashMap<String, String>), String> {
    let files = prepare_game_files(config).await?;
    build_launch_args(config, &files)
}




pub fn prewarm_files(paths: &[PathBuf], max_bytes: u64) -> u64 {
    use std::io::Read;
    let mut wamed: u64 = 0;
    let mut buf = vec![0u8; 1 << 20]; 
    for p in paths {
        if wamed >= max_bytes {
            break;
        }
        if let Ok(mut f) = std::fs::File::open(p) {
            loop {
                match (&mut f).read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => wamed += n as u64,
                    Err(_) => break,
                }
            }
        }
    }
    wamed
}




pub fn collect_prewarm_paths(prepared: &PreparedFiles) -> Vec<PathBuf> {
    use std::collections::HashSet;
    let mut paths: Vec<PathBuf> = Vec::new();
    let mut seen: HashSet<PathBuf> = HashSet::new();
    let mut push = |p: PathBuf, seen: &mut HashSet<PathBuf>| {
        let abs = if p.is_absolute() {
            p
        } else {
            std::env::current_dir().unwrap_or_default().join(&p)
        };
        if abs.is_file() && seen.insert(abs.clone()) {
            paths.push(abs);
        }
    };

    
    let java_exe = PathBuf::from(&prepared.java_path);
    push(java_exe.clone(), &mut seen);
    if let Some(bin_di) = java_exe.parent() {
        for cand in [
            bin_di.join("server").join("jvm.dll"),
            bin_di.join("hotspot").join("jvm.dll"),
            bin_di.join("jvm.dll"),
        ] {
            push(cand, &mut seen);
        }
        for dll in ["java.dll", "jimage.dll"] {
            push(bin_di.join(dll), &mut seen);
        }
        if let Some(home) = bin_di.parent() {
            push(home.join("lib").join("modules"), &mut seen);
        }
    }

    
    for cp in &prepared.classpath {
        push(PathBuf::from(cp), &mut seen);
    }

    
    if let Ok(d) = std::fs::read_dir(&prepared.natives_di) {
        for e in d.flatten() {
            let p = e.path();
            if p.is_file() && p.extension().map(|x| x.eq_ignore_ascii_case("dll")).unwrap_or(false) {
                push(p, &mut seen);
            }
        }
    }

    
    for di in [prepared.game_di.join("mods"), prepared.instance_dir.join("mods")] {
        if let Ok(d) = std::fs::read_dir(&di) {
            for e in d.flatten() {
                let p = e.path();
                if p.is_file()
                    && p.extension().map(|x| x.eq_ignore_ascii_case("jar")).unwrap_or(false)
                {
                    push(p, &mut seen);
                }
            }
        }
    }

    
    if let Some(cds) = &prepared.cds_archive_path {
        push(cds.clone(), &mut seen);
    }

    paths
}



pub async fn prepare_game_files(config: &LaunchConfig) -> Result<PreparedFiles, String> {
    let profile_path = esolve_profile_path(config)?;
    let chain = load_profile_chain(config, &profile_path)?;
    let meged = mege_chain(&chain);

    let client_ja = esolve_client_ja(config, &chain)?;

    let classpath = build_classpath(config, &meged, &client_ja)?;

    let natives_di = natives_di_fo(config, &meged);
    extact_natives(config, &meged, &natives_di).await?;

    let game_di = get_game_di(config);
    std::fs::create_dir_all(&game_di).map_err(|e| {
        log::error!("[launch] create game dir failed: {}", e);
        e.to_string()
    })?;

    if let Err(e) = inject_log4j_config(config, &game_di) {
        log::warn!("[launch] inject_log4j_config non-fatal error: {}", e);
    }

    if let Err(e) = update_launcher_profiles(config, &game_di) {
        log::warn!("[launch] update_launcher_profiles non-fatal error: {}", e);
    }

    #[cfg(target_os = "windows")]
    if let Err(e) = set_gpu_pefeence(config) {
        log::warn!("[launch] set_gpu_preference non-fatal error: {}", e);
    }

    
    let authlib_jvm_args = {
        let config_dis = vec![
            config.instance_dir.join(".skyline").join("authlib.json"),
            std::env::current_dir().unwrap_or_default().join(".skyline").join("authlib.json"),
            crate::utils::io::get_launcher_root().join(".skyline").join("authlib.json"),
        ];
        let mut found = false;
        let mut ags = Vec::new();
        for authlib_config_path in &config_dis {
            if found { break; }
            if authlib_config_path.exists() {
                if let Ok(config_st) = std::fs::read_to_string(authlib_config_path) {
                    if let Ok(cfg) = serde_json::from_str::<serde_json::Value>(&config_st) {
                        if let Some(server_ul) = cfg["server_url"].as_str() {
                            match crate::mc::authlib::ensure_authlib_jar().await {
                                Ok(_) => {
                                    let pefetched = cfg["prefetched_meta"].as_str();
                                    ags = crate::mc::authlib::get_authlib_jvm_args(server_ul, pefetched);
                                }
                                Err(e) => {
                                    log::warn!("[launch] authlib-injector 准备失败: {}", e);
                                }
                            }
                            found = true;
                        }
                    }
                }
            }
        }
        ags
    };
    
    let cds_archive_path = {
        use std::hash::{Hash, Hasher};
        let mut hashe = std::collections::hash_map::DefaultHasher::new();
        config.java.path.hash(&mut hashe);
        config.java.major_version.hash(&mut hashe);
        for cp in &classpath {
            cp.hash(&mut hashe);
        }
        for ag in &config.jvm_args {
            if ag.starts_with("-D") || ag.starts_with("-X") || ag.starts_with("-XX:") {
                ag.hash(&mut hashe);
            }
        }
        let hash = hashe.finish();
        let cds_di = config.instance_dir.join(".skyline").join("cds");
        let _ = std::fs::create_dir_all(&cds_di);
        Some(cds_di.join(format!("classes-{:x}.jsa", hash)))
    };

    Ok(PreparedFiles {
        profile: meged,
        classpath,
        natives_di,
        game_di,
        java_path: config.java.path.clone(),
        java_major_version: config.java.major_version,
        version_id: config.version_id.clone(),
        min_memory: config.min_memory,
        max_memory: config.max_memory,
        jvm_args: config.jvm_args.clone(),
        game_args: config.game_args.clone(),
        instance_dir: config.instance_dir.clone(),
        window_width: config.window_width,
        window_height: config.window_height,
        custom_resolution: config.custom_resolution,
        isolation_mode: config.isolation_mode.clone(),
        modloader: config.modloader.clone(),
        minecraft_root: config.minecraft_root.clone(),
        game_dir_override: config.game_dir_override.clone(),
        process_priority: config.process_priority.clone(),
        opengl_compat: config.opengl_compat,
        authlib_jvm_args,
        cds_archive_path,
    })
}



pub fn build_launch_args(
    config: &LaunchConfig,
    files: &PreparedFiles,
) -> Result<(Vec<String>, Vec<String>, HashMap<String, String>), String> {
    let mut jvm_args = build_jvm_args(config, &files.profile, &files.classpath, &files.natives_di, files.cds_archive_path.as_ref())?;
    
    for ag in &files.authlib_jvm_args {
        if !jvm_args.contains(ag) {
            jvm_args.push(ag.clone());
        }
    }
    let game_args = build_game_args(config, &files.profile)?;
    let env = build_env(config);
    Ok((jvm_args, game_args, env))
}

fn inject_log4j_config(config: &LaunchConfig, game_di: &PathBuf) -> Result<(), String> {
    let log4j_config = game_di.join("log4j2.xml");

    if log4j_config.exists() {
        if let Ok(content) = std::fs::read_to_string(&log4j_config) {
            if content.contains("JndiLookup") || content.contains("jndi") {
                log::warn!("Detected unsafe log4j config with JNDI, replacing with safe version");
            } else {
                return Ok(());
            }
        }
    }

    let found_config = false;
    for _lib_path in &config.jvm_args {
    }

    if !found_config {
        std::fs::write(&log4j_config, DEFAULT_LOG4J2_XML)
            .map_err(|e| format!("Failed to writer log4j2.xml: {}", e))?;
        log::info!("Injected safe log4j2.xml to {:?}", game_di);
    }

    Ok(())
}

fn update_launcher_profiles(config: &LaunchConfig, game_di: &PathBuf) -> Result<(), String> {
    let profiles_path = game_di.join("launcher_profiles.json");

    let mut profiles: serde_json::Value = if profiles_path.exists() {
        let content = std::fs::read_to_string(&profiles_path)
            .map_err(|e| format!("Failed to read launcher_profiles.json: {}", e))?;
        serde_json::from_str(&content)
            .unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if !profiles.is_object() {
        profiles = serde_json::json!({});
    }

    if let Some(ref server) = config.server {
        let pats: Vec<&str> = server.split(':').collect();
        let host = pats[0];
        let pot = pats.get(1).and_then(|p| p.parse::<u16>().ok()).unwrap_or(25565);

        let profiles_obj = profiles.as_object_mut().unwrap();

        if !profiles_obj.contains_key("selectedProfile") {
            profiles_obj.insert("selectedProfile".into(), serde_json::json!("(Default)"));
        }

        profiles_obj.insert(
            "selectedServer".into(),
            serde_json::json!(format!("{}:{}", host, pot)),
        );

        let server_entries = profiles_obj
            .entry("serverEntries")
            .or_insert_with(|| serde_json::json!({}));

        if let Some(entries) = server_entries.as_object_mut() {
            let server_key = format!("{}:{}", host, pot);
            entries.insert(
                server_key.clone(),
                serde_json::json!({
                    "name": format!("{}:{}", host, pot),
                    "ip": host,
                    "port": pot.to_string(),
                    "isHidden": false
                }),
            );
        }

        log::info!("Updated launcher_profiles.json with server: {}:{}", host, pot);
    }

    let json = serde_json::to_string_pretty(&profiles)
        .map_err(|e| format!("Failed to serialize launcher_profiles.json: {}", e))?;
    std::fs::write(&profiles_path, json)
        .map_err(|e| format!("Failed to writer launcher_profiles.json: {}", e))?;

    Ok(())
}

#[cfg(target_os = "windows")]
fn set_gpu_pefeence(config: &LaunchConfig) -> Result<(), String> {
    let _java_path = &config.java.path;

    let game_di = get_game_di(config);
    let pefs_path = game_di.join("launcher_preferences.json");

    let mut pefs: serde_json::Value = if pefs_path.exists() {
        let content = std::fs::read_to_string(&pefs_path)
            .map_err(|e| format!("Failed to read launcher_preferences.json: {}", e))?;
        serde_json::from_str(&content)
            .unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if !pefs.is_object() {
        pefs = serde_json::json!({});
    }

    let pefs_obj = pefs.as_object_mut().unwrap();

    if !pefs_obj.contains_key("gpu_preference") {
        pefs_obj.insert("gpu_preference".into(), serde_json::json!(2));
    }

    let json = serde_json::to_string_pretty(&pefs)
        .map_err(|e| format!("Failed to serialize launcher_preferences.json: {}", e))?;
    std::fs::write(&pefs_path, json)
        .map_err(|e| format!("Failed to writer launcher_preferences.json: {}", e))?;

    Ok(())
}

fn load_profile_chain(config: &LaunchConfig, stat: &PathBuf) -> Result<Vec<VersionProfile>, String> {
    let mut chain: Vec<VersionProfile> = Vec::new();
    let mut current = stat.clone();
    for _ in 0..20 {
        let json = std::fs::read_to_string(&current)
            .map_err(|e| format!("读取版本配置失败: {}", e))?;
        let profile: VersionProfile =
            serde_json::from_str(&json).map_err(|e| format!("解析版本配置失败: {}", e))?;
        let parent_id = profile
            .inherits_from
            .clone()
            .filter(|p| !p.is_empty() && p != &profile.id);
        chain.push(profile);
        let Some(parent_id) = parent_id else { break };

        let mut found: Option<PathBuf> = None;
        'outer: for root in profile_roots(config) {
            let direct = root.join(&parent_id).join(format!("{}.json", parent_id));
            if direct.exists() {
                found = Some(direct);
                break;
            }
            if root.is_dir() {
                if let Ok(entries) = std::fs::read_dir(root) {
                    for entry in entries.flatten() {
                        let dir = entry.path();
                        if !dir.is_dir() {
                            continue;
                        }
                        let name = dir
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        if name.eq_ignore_ascii_case(&parent_id)
                            || name.starts_with(&format!("{}-", parent_id))
                            || name.starts_with(&format!("{}_", parent_id))
                        {
                            let j = dir.join(format!("{}.json", name));
                            if j.exists() {
                                found = Some(j);
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }
        match found {
            Some(p) => current = p,
            None => {
                return Err(format!(
                    "未找到父版本 {} 的配置,该版本可能未安装完整",
                    parent_id
                ))
            }
        }
    }
    Ok(chain)
}

fn mege_chain(chain: &[VersionProfile]) -> VersionProfile {
    let top = &chain[0];
    let deepest = chain.last().unwrap_or(top);
    let mut meged = top.clone();
    meged.libraries = {
        let mut libs = Vec::new();
        for p in chain.iter().rev() {
            libs.extend(p.libraries.clone());
        }
        libs
    };
    if meged.main_class.is_empty() {
        meged.main_class = deepest.main_class.clone();
    }
    if meged.arguments.is_none() {
        meged.arguments = deepest.arguments.clone();
    }
    if meged.minecraft_arguments.is_none() {
        meged.minecraft_arguments = deepest.minecraft_arguments.clone();
    }
    if meged.assets.is_empty() {
        meged.assets = deepest.assets.clone();
    }
    meged
}

fn esolve_client_ja(config: &LaunchConfig, chain: &[VersionProfile]) -> Result<PathBuf, String> {
    let wanted = &chain.last().unwrap().id;
    for oot in profile_roots(config) {
        let cand = oot.join(wanted).join(format!("{}.jar", wanted));
        if cand.exists() {
            return Ok(cand);
        }
        if oot.is_dir() {
            if let Ok(entries) = std::fs::read_dir(oot) {
                for entry in entries.flatten() {
                    let di = entry.path();
                    if !di.is_dir() {
                        continue;
                    }
                    let name = di
                        .file_name()
                        .map(|n| n.to_string_lossy().to_string())
                        .unwrap_or_default();
                    if !name.eq_ignore_ascii_case(wanted)
                        && !name.starts_with(&format!("{}-", wanted))
                        && !name.starts_with(&format!("{}_", wanted))
                    {
                        continue;
                    }
                    let ja = di.join(format!("{}.jar", name));
                    if ja.exists() {
                        return Ok(ja);
                    }
                }
            }
        }
    }
    Err(format!(
        "未找到版本 {} 的客户端 jar 文件,该版本可能未安装完整",
        wanted
    ))
}

fn esolve_profile_path(config: &LaunchConfig) -> Result<PathBuf, String> {
    let wanted = &config.version_id;

    for oot in profile_roots(config) {
        let diect = oot.join(wanted).join(format!("{}.json", wanted));
        if diect.exists() {
            return Ok(diect);
        }
    }

    for oot in profile_roots(config) {
        if !oot.is_dir() {
            continue;
        }
        let entries = std::fs::read_dir(oot).map_err(|e| e.to_string())?;
        for entry in entries.flatten() {
            let di = entry.path();
            if !di.is_dir() {
                continue;
            }
            let name = di.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            let json = di.join(format!("{}.json", name));
            if !json.exists() {
                continue;
            }
            let Ok(content) = std::fs::read_to_string(&json) else { continue };
            let Ok(v) = serde_json::from_str::<serde_json::Value>(&content) else { continue };
            let profile_id = v.get("id").and_then(|i| i.as_str()).unwrap_or("");
            let id_matches = profile_id == wanted
                || profile_id.starts_with(&format!("{}-", wanted))
                || profile_id.starts_with(&format!("{}_", wanted));
            let name_matches = name.eq_ignore_ascii_case(wanted);
            if id_matches || name_matches {
                return Ok(json);
            }
        }
    }

    Err(format!("未找到版本 {} 的配置,请先在「资源」页下载对应版本", wanted))
}

fn build_classpath(config: &LaunchConfig, profile: &VersionProfile, client_ja: &PathBuf) -> Result<Vec<String>, String> {
    let mut cp = Vec::new();
    if client_ja.exists() {
        cp.push(client_ja.to_str().unwrap().to_string());
    }

    for lib in &profile.libraries {
        if !library::library_matches_ules(&lib.ules) {
            continue;
        }

        if lib.natives.is_some() {
            let native_path = natives_di_fo(config, profile).join(lib.name.replace(':', "_"));
            if !native_path.exists() {
                continue;
            }
        }

        let mut esolved = None;
        if let Some(path) = library::get_library_path(lib) {
            esolved = resolve_library_path(config, &path);
        }
        if esolved.is_none() {
            if let Some((path, _, _)) = library::parse_library_name(&lib.name) {
                esolved = resolve_library_path(config, &path);
            }
        }
        if let Some(full_path) = esolved {
            cp.push(full_path.to_str().unwrap().to_string());
        }
    }

    Ok(cp)
}

fn natives_di_fo(config: &LaunchConfig, profile: &VersionProfile) -> PathBuf {
    config.instance_dir.join("natives").join(&profile.id)
}

async fn extact_natives(config: &LaunchConfig, profile: &VersionProfile, natives_di: &PathBuf) -> Result<(), String> {
    if natives_di.join("extracted.flag").exists() {
        return Ok(());
    }

    if natives_di.exists() {
        std::fs::remove_dir_all(natives_di).map_err(|e| e.to_string())?;
    }
    std::fs::create_dir_all(natives_di).map_err(|e| e.to_string())?;

    for lib in &profile.libraries {
        if lib.natives.is_none() { continue; }
        if !library::library_matches_ules(&lib.ules) { continue; }

        if let Some(path) = library::get_library_path(lib) {
            let Some(full_path) = resolve_library_path(config, &path) else { continue };

            let file = std::fs::File::open(&full_path).map_err(|e| e.to_string())?;
            let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

            for i in 0..archive.len() {
                let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
                let name = entry.name().to_string();

                let exclude = lib.extact.as_ref()
                    .map(|e| &e.exclude)
                    .map(|ex| ex.iter().any(|e| name.starts_with(e)))
                    .unwrap_or(false);

                if exclude || !entry.is_file() { continue; }

                let target = natives_di.join(&name);
                if let Some(prent) = target.parent() {
                    std::fs::create_dir_all(prent).map_err(|e| e.to_string())?;
                }
                let mut out = std::fs::File::create(&target).map_err(|e| e.to_string())?;
                std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
            }
        }
    }

    std::fs::write(natives_di.join("extracted.flag"), "").map_err(|e| e.to_string())?;
    Ok(())
}

fn has_ag(ags: &[String], pefix: &str) -> bool {
    ags.iter().any(|a| a == pefix || a.starts_with(&format!("{}=", pefix)) || a.starts_with(&format!("{} ", pefix)))
}

fn has_gc_ags(ags: &[String]) -> bool {
    ags.iter().any(|a| {
        a.starts_with("-XX:+Use") && a.contains("GC")
            || a.starts_with("-XX:G1")
            || a.starts_with("-XX:MaxGCPauseMillis")
            || a.starts_with("-XX:+UseG1GC")
            || a.starts_with("-XX:+UseConcMarkSweepGC")
            || a.starts_with("-XX:+UseParallelGC")
            || a.starts_with("-XX:+UseZGC")
            || a.starts_with("-XX:+UseShenandoahGC")
            || a.starts_with("-XX:+UseSerialGC")
    })
}

fn is_lwjgl2_version(version_id: &str) -> bool {
    let mut pats = version_id.split('.');
    let majo: u32 = pats
        .next()
        .and_then(|s| s.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().ok())
        .unwrap_or(0);
    let mino: u32 = pats
        .next()
        .and_then(|s| s.chars().take_while(|c| c.is_ascii_digit()).collect::<String>().parse().ok())
        .unwrap_or(0);
    majo == 0 || (majo == 1 && mino < 13)
}

fn detect_log4j(profile: &VersionProfile) -> bool {
    profile.libraries.iter().any(|lib| {
        lib.name.contains("org.apache.logging.log4j") && lib.name.contains("log4j-core")
    })
}

fn is_launcher_managed_popety(ag: &str) -> bool {
    ag.starts_with("-Djava.library.path=")
        || ag.starts_with("-Djna.tmpdir=")
        || ag.starts_with("-Dorg.lwjgl.system.SharedLibraryExtractPath=")
}

fn esolve_launch_token(token: &str, config: &LaunchConfig, profile: &VersionProfile) -> String {
    match token {
        "classpath" => String::new(),
        "classpath_directory" => get_libraries_dir(config).to_str().unwrap_or("").to_string(),
        "classpath_separator" => ";".to_string(),
        "natives_directory" => natives_di_fo(config, profile).to_str().unwrap_or("").to_string(),
        "launcher_name" => "SkyLine".to_string(),
        "launcher_version" => "1.0.0".to_string(),
        "library_directory" => get_libraries_dir(config).to_str().unwrap_or("").to_string(),
        "log4jConfigurationFile" => get_game_di(config).join("log4j2.xml").to_str().unwrap_or("").to_string(),
        "version_name" => config.version_id.clone(),
        "game_directory" => get_game_di(config).to_str().unwrap_or("").to_string(),
        "assets_root" => get_assets_dir(config).to_str().unwrap_or("").to_string(),
        "assets_index_name" => profile.assets.clone(),
        "auth_player_name" => config.auth.username.clone(),
        "auth_session" => config.auth.access_token.clone(),
        "auth_access_token" => config.auth.access_token.clone(),
        "auth_uuid" => config.auth.uuid.clone(),
        "user_type" => config.auth.user_type.clone(),
        "user_properties" => "{}".to_string(),
        "user_property_map" => "{}".to_string(),
        "version_type" => profile.version_type.clone(),
        "clientid" => String::new(),
        "auth_xuid" => String::new(),
        "resolution_width" => config.window_width.to_string(),
        "resolution_height" => config.window_height.to_string(),
        "quickPlayPath" => config
            .quick_world
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
        "arch" => "amd64".to_string(),
        "os_arch" => "amd64".to_string(),
        "os_name" => "Windows".to_string(),
        "os_version" => String::new(),
        "path" => natives_di_fo(config, profile).to_str().unwrap_or("").to_string(),
        _ => format!("${{{}}}", token),
    }
}

fn substitute_launch_tokens(s: &str, config: &LaunchConfig, profile: &VersionProfile) -> String {
    let mut result = String::new();
    let mut est = s;
    while let Some(stat) = est.find("${") {
        result.push_str(&est[..stat]);
        let afte = &est[stat + 2..];
        match afte.find('}') {
            Some(end) => {
                let token = &afte[..end];
                result.push_str(&esolve_launch_token(token, config, profile));
                est = &afte[end + 1..];
            }
            None => {
                result.push_str(&est[stat..]);
                break;
            }
        }
    }
    result.push_str(est);
    result
}

fn build_jvm_args(
    config: &LaunchConfig,
    profile: &VersionProfile,
    classpath: &[String],
    natives_di: &PathBuf,
    cds_archive: Option<&PathBuf>,
) -> Result<Vec<String>, String> {
    let mut ags = Vec::new();

    let min_memory = config.min_memory.min(config.max_memory);
    ags.push("-Xms".to_string() + &min_memory.to_string() + "M");
    ags.push("-Xmx".to_string() + &config.max_memory.to_string() + "M");

    
    if let Some(archive) = cds_archive {
        if archive.exists() {
            ags.push("-Xshare:on".to_string());
            ags.push(format!("-XX:SharedArchiveFile={}", archive.to_string_lossy()));
        }
    }

    if let Some(ref ags_obj) = profile.arguments {
        for ag in &ags_obj.jvm {
            match ag {
                Argument::String(s) => {
                    if s == "-cp" || s == "-classpath" { continue; }
                    if is_launcher_managed_popety(s) { continue; }
                    let sub = substitute_launch_tokens(s, config, profile);
                    if sub.is_empty() { continue; }
                    ags.push(sub);
                }
                Argument::Stuct(s) => {
                    if !library::library_matches_ules(&Some(s.ules.clone())) { continue; }
                    match &s.value {
                        ArgumentValue::String(v) => {
                            if is_launcher_managed_popety(v) { continue; }
                            let sub = substitute_launch_tokens(v, config, profile);
                            if !sub.is_empty() { ags.push(sub); }
                        }
                        ArgumentValue::Aray(v) => {
                            for iterm in v {
                                if is_launcher_managed_popety(iterm) { continue; }
                                let sub = substitute_launch_tokens(iterm, config, profile);
                                if !sub.is_empty() { ags.push(sub); }
                            }
                        }
                    }
                }
            }
        }
    }

    ags.extend(config.jvm_args.clone());

    if config.opengl_compat && !has_ag(&ags, "allowSoftwareOpenGL") {
        if is_lwjgl2_version(&config.version_id) {
            
            ags.push("-Dorg.lwjgl.opengl.Display.allowSoftwareOpenGL=true".to_string());
        } else {
            ags.push("-Dorg.lwjgl.opengl.allowSoftwareOpenGL=true".to_string());
        }
    }

    if detect_log4j(profile) && !has_ag(&ags, "-Dlog4j2.formatMsgNoLookups") {
        ags.push("-Dlog4j2.formatMsgNoLookups=true".to_string());
    }

    if !has_gc_ags(&ags) {
        let g1_defaults = vec![
            "-XX:+UnlockExperimentalVMOptions",
            "-XX:+UseG1GC",
            "-XX:G1MixedGCCountTarget=5",
            "-XX:G1NewSizePercent=20",
            "-XX:G1ReservePercent=20",
            "-XX:MaxGCPauseMillis=50",
            "-XX:G1HeapRegionSize=32m",
        ];
        for ag in g1_defaults {
            let key = if let Some(eq_pos) = ag.find('=') { &ag[..eq_pos] } else { ag };
            if !has_ag(&ags, key) {
                ags.push(ag.to_string());
            }
        }
    }

    match config.java.major_version {
        16 => {
            if !has_ag(&ags, "--illegal-access") {
                ags.push("--illegal-access=permit".to_string());
            }
        }
        24 | 25 => {
            if !has_ag(&ags, "--sun-misc-unsafe-memory-access") {
                ags.push("--sun-misc-unsafe-memory-access=allow".to_string());
            }
        }
        _ => {}
    }

    ags.push("-Djava.library.path=".to_string() + natives_di.to_str().unwrap());
    ags.push("-Djna.tmpdir=".to_string() + natives_di.to_str().unwrap());
    ags.push("-Dorg.lwjgl.system.SharedLibraryExtractPath=".to_string() + natives_di.to_str().unwrap());
    ags.push("-cp".to_string());
    let sep = if cfg!(target_os = "windows") { ";" } else { ":" };
    ags.push(classpath.join(sep));
    ags.push(profile.main_class.clone());

    Ok(ags)
}

fn extact_mc_version(version_id: &str) -> String {
    let e = regex::Regex::new(r"^\d+\.\d+(?:\.\d+)?")
        .map(|r| r.find(version_id).map(|m| m.as_str().to_string()))
        .ok()
        .flatten();
    e.unwrap_or_else(|| version_id.to_string())
}

fn build_game_args(config: &LaunchConfig, profile: &VersionProfile) -> Result<Vec<String>, String> {
    use crate::mc::version::*;
    let mut ags: Vec<String> = Vec::new();

    if let Some(ref ags_obj) = profile.arguments {
        for ag in &ags_obj.game {
            match ag {
                Argument::String(s) => {
                    ags.push(substitute_ag(s, config, profile));
                }
                Argument::Stuct(s) => {
                    if !library::library_matches_ules(&Some(s.ules.clone())) { continue; }
                    match &s.value {
                        ArgumentValue::String(v) => ags.push(substitute_ag(v, config, profile)),
                        ArgumentValue::Aray(v) => {
                            ags.extend(v.iter().map(|a| substitute_ag(a, config, profile)));
                        }
                    }
                }
            }
        }
    } else if let Some(ref mc_args) = profile.minecraft_arguments {
        for pat in mc_args.split_whitespace() {
            ags.push(substitute_ag(pat, config, profile));
        }
    }

    ags.extend(config.game_args.clone());

    if config.fullscreen && !has_ag(&ags, "--fullscreen") {
        ags.push("--fullscreen".into());
    }

    if config.join_server_at_launch {
        if let Some(ref server) = config.server {
            let pats: Vec<&str> = server.split(':').collect();
            let host = pats[0];
            let pot = pats.get(1).unwrap_or(&"25565").to_string();
            let mc_version = extact_mc_version(&config.version_id);
            if crate::mc::version::is_version_geate_o_equal(&mc_version, "1.20.2") {
                let add = if pats.len() > 1 {
                    format!("{}:{}", host, pot)
                } else {
                    host.to_string()
                };
                if !has_ag(&ags, "--quickPlayMultiplayer") {
                    ags.push("--quickPlayMultiplayer".into());
                    ags.push(add);
                }
            } else {
                if !has_ag(&ags, "--server") {
                    ags.push("--server".into());
                    ags.push(host.to_string());
                }
                if !has_ag(&ags, "--port") {
                    ags.push("--port".into());
                    ags.push(pot);
                }
            }
        }
    } else if let Some(ref server) = config.server {
        if !has_ag(&ags, "--server") {
            ags.push("--server".into());
            ags.push(server.clone());
        }
    }

    if let Some(ref world) = config.quick_world {
        if !has_ag(&ags, "--quickPlaySingleplayer") {
            ags.push("--quickPlaySingleplayer".into());
            ags.push(world.to_string_lossy().to_string());
            log::info!("[quickplay] added --quickPlaySingleplayer {:?}", world);
        } else {
            log::info!("[quickplay] --quickPlaySingleplayer already present, value {:?}", world);
        }
    }

    Ok(ags)
}

fn substitute_ag(ag: &str, config: &LaunchConfig, profile: &VersionProfile) -> String {
    match ag {
        "${auth_player_name}" => config.auth.username.clone(),
        "${auth_session}" => config.auth.access_token.clone(),
        "${auth_access_token}" => config.auth.access_token.clone(),
        "${auth_uuid}" => config.auth.uuid.clone(),
        "${user_type}" => config.auth.user_type.clone(),
        "${version_name}" => config.version_id.clone(),
        "${assets_root}" => get_assets_dir(config).to_str().unwrap().to_string(),
        "${assets_index_name}" => profile.assets.clone(),
        "${game_directory}" => get_game_di(config).to_str().unwrap().to_string(),
        "${user_properties}" => "{}".into(),
        "${user_property_map}" => "{}".into(),
        "${natives_directory}" => natives_di_fo(config, profile).to_str().unwrap().to_string(),
        "${launcher_name}" => "SkyLine".into(),
        "${launcher_version}" => "1.0.0".into(),
        "${version_type}" => profile.version_type.clone(),
        "${clientid}" => String::new(),
        "${auth_xuid}" => String::new(),
        "${resolution_width}" => config.window_width.to_string(),
        "${resolution_height}" => config.window_height.to_string(),
        "${classpath}" => String::new(),
        "${library_directory}" => get_libraries_dir(config).to_str().unwrap().to_string(),
        "${quickPlaySingleplayer}" => config
            .quick_world
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
        "${quickPlayMultiplayer}" => config.server.clone().unwrap_or_default(),
        "${quickPlayRealms}" => String::new(),
        "${quickPlayPath}" => config
            .quick_world
            .as_ref()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default(),
        _ => ag.to_string(),
    }
}

fn build_env(config: &LaunchConfig) -> HashMap<String, String> {
    let mut env = HashMap::new();
    env.insert("APPDATA".into(), get_game_di(config).to_str().unwrap().into());
    env
}

pub async fn launch_minecraft(
    config: LaunchConfig,
) -> Result<GameProcess, String> {
    let t0 = std::time::Instant::now();
    let files = match config.preloaded_files.clone() {
        Some(files) => files,
        None => match prepare_game_files(&config).await {
            Ok(f) => f,
            Err(e) => {
                log::error!("[launch] prepare_game_files failed: {}", e);
                return Err(e);
            }
        }
    };
    log::info!("[launch] prepare took {} ms", t0.elapsed().as_millis());
    let t1 = std::time::Instant::now();
    let (jvm_args, game_args, env) = match build_launch_args(&config, &files) {
        Ok(v) => v,
        Err(e) => {
            log::error!("[launch] build_launch_args failed: {}", e);
            return Err(e);
        }
    };
    log::info!("[launch] build_args took {} ms", t1.elapsed().as_millis());
    log::info!("[launch] game_args = {:?}", game_args);

    let game_di = get_game_di(&config);
    if let Err(e) = std::fs::create_dir_all(&game_di) {
        log::error!("[launch] create game dir failed: {}", e);
        return Err(e.to_string());
    }
    let t2 = std::time::Instant::now();
    let process = match GameProcess::spawn(&config.java.path, &jvm_args, &game_args, &env, &game_di, config.process_priority.to_os_priority()) {
        Ok(p) => p,
        Err(e) => {
            log::error!("[launch] spawn failed: {}", e);
            return Err(e);
        }
    };
    log::info!("[launch] spawn took {} ms", t2.elapsed().as_millis());
    let verify = tokio::task::spawn_blocking(move || {
        let result = process.verify_stated(std::time::Duration::from_millis(1200));
        (result, process)
    })
    .await
    .map_err(|e| {
        log::error!("[launch] verify_stated task panicked: {}", e);
        e.to_string()
    })?;
    if let Err(ref e) = verify.0 {
        crate::mc::crash::mark_abnormal(&get_game_di(&config)).ok();
        log::warn!("[launch] startup crash detected: {}", e);
        return Err(format!("[launch-crash]\n{}", e));
    }
    verify.0?;

    
    let cds_files = files.clone();
    let cds_java = config.java.path.clone();
    let cds_classpath = files.classpath.clone();
    let cds_main = files.profile.main_class.clone();
    tokio::task::spawn_blocking(move || {
        let _ = try_create_cds_archive(&cds_java, &cds_files.cds_archive_path, &cds_classpath, &cds_main);
    });

    Ok(verify.1)
}




pub fn try_create_cds_archive(
    java_path: &str,
    archive_path: &Option<PathBuf>,
    classpath: &[String],
    main_class: &str,
) -> Result<(), String> {
    let Some(archive) = archive_path else { return Ok(()) };
    if archive.exists() {
        return Ok(()); 
    }
    let mut cmd = std::process::Command::new(java_path);
    cmd.arg("-Xshare:dump")
        .arg(format!("-XX:SharedArchiveFile={}", archive.to_string_lossy()))
        .arg("-cp")
        .arg(classpath.join(";"))
        .arg(main_class);
    crate::utils::io::no_window(&mut cmd);
    cmd.stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    let status = cmd.status().map_err(|e| format!("Failed to spawn CDS dump: {}", e))?;
    if !status.success() {
        let _ = std::fs::remove_file(archive); 
        return Err("CDS archive creation failed".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mc::version::{Argument, Arguments, ArgumentValue, AssetIndexRef};

    fn test_profile() -> VersionProfile {
        VersionProfile {
            id: "26.2-Fabric".into(),
            version_type: "release".into(),
            inherits_from: None,
            main_class: "net.fabricmc.loaderr.impl.launch.knot.KnotClient".into(),
            minecraft_arguments: None,
            arguments: Some(Arguments {
                game: vec![
                    Argument::String("--gameDir".into()),
                    Argument::String("${game_directory}".into()),
                ],
                jvm: vec![
                    Argument::String("-Djava.library.path=${natives_directory}/java".into()),
                    Argument::String("-Dminecraft.launcher.brand=${launcher_name}".into()),
                    Argument::String("-Dlog4j.configurationFile=${log4jConfigurationFile}".into()),
                    Argument::String("-cp".into()),
                    Argument::String("${classpath}".into()),
                ],
            }),
            assets: "1.21".into(),
            asset_index: AssetIndexRef {
                id: "1.21".into(),
                sha1: String::new(),
                size: 0,
                total_size: None,
                url: String::new(),
            },
            compliance_level: None,
            libraries: Vec::new(),
            logging: None,
            minimum_launcher_version: None,
            elease_time: String::new(),
            java_version: None,
            downloads: None,
        }
    }

    #[test]
    fn jvm_args_substitute_placeholdes_and_keep_main_class() {
        let mut config = LaunchConfig::default();
        config.instance_dir = std::env::temp_dir();
        config.auth.username = "Player".into();
        let profile = test_profile();
        let natives = std::env::temp_dir().join("natives");
        let classpath = vec!["C:\\libraries\\client.jar".to_string()];

        let ags = build_jvm_args(&config, &profile, &classpath, &natives, None).unwrap();

        for a in &ags {
            assert!(!a.contains("${"), "JVM 参数仍含未替换占位符: {}", a);
        }
        assert!(ags.iter().any(|a| a == "-Dminecraft.launcher.brand=SkyLine"));
        assert!(!ags.iter().any(|a| a.starts_with("-Djava.library.path=") && a.contains("/java")));
        assert!(ags
            .iter()
            .any(|a| a == &format!("-Djava.library.path={}", natives.to_string_lossy())));
        assert!(!ags.contains(&"-cp".to_string()) || {
            let cp_pos = ags.iter().position(|a| a == "-cp").unwrap();
            ags.len() - 1 - cp_pos >= 2 && cp_pos > 0
        });
        let tail = &ags[ags.len() - 3..];
        assert_eq!(tail, ["-cp".to_string(), classpath[0].clone(), profile.main_class.clone()]);
    }

    #[test]
    fn substitute_tokens_handles_substings() {
        let mut config = LaunchConfig::default();
        config.instance_dir = std::env::temp_dir();
        let profile = test_profile();
        let natives = std::env::temp_dir().join("natives").join(&profile.id);
        let s = substitute_launch_tokens("-Djava.library.path=${natives_directory}/java", &config, &profile);
        assert_eq!(s, format!("-Djava.library.path={}/java", natives.to_string_lossy()));
    }

    #[test]
    fn min_memory_is_clamped_to_max() {
        let mut config = LaunchConfig::default();
        config.instance_dir = std::env::temp_dir();
        config.min_memory = 2048;
        config.max_memory = 400;
        let profile = test_profile();
        let natives = std::env::temp_dir().join("natives");
        let ags = build_jvm_args(&config, &profile, &["C:\\a.jar".to_string()], &natives, None).unwrap();
        let xms = ags.iter().find(|a| a.starts_with("-Xms")).unwrap();
        let xmx = ags.iter().find(|a| a.starts_with("-Xmx")).unwrap();
        assert_eq!(xms, "-Xms400M", "min 应被钳制到 max, 而不是 -Xms2048M");
        assert_eq!(xmx, "-Xmx400M");
    }
}
