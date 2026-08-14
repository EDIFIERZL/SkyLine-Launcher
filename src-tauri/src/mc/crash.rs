use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum CrashSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl CrashSeverity {
    pub fn label(&self) -> &str {
        match self {
            CrashSeverity::Critical => "致命",
            CrashSeverity::Error => "错误",
            CrashSeverity::Warning => "警告",
            CrashSeverity::Info => "信息",
        }
    }

    pub fn color_class(&self) -> &str {
        match self {
            CrashSeverity::Critical => "bg-red-100 text-red-800 dark:bg-red-500/20 dark:text-red-400 border-red-300 dark:border-red-500/30",
            CrashSeverity::Error => "bg-orange-100 text-orange-800 dark:bg-orange-500/20 dark:text-orange-400 border-orange-300 dark:border-orange-500/30",
            CrashSeverity::Warning => "bg-amber-100 text-amber-800 dark:bg-amber-500/20 dark:text-amber-400 border-amber-300 dark:border-amber-500/30",
            CrashSeverity::Info => "bg-blue-100 text-blue-800 dark:bg-blue-500/20 dark:text-blue-400 border-blue-300 dark:border-blue-500/30",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct CrashAnalysis {
    pub stage: String,
    pub serverity: CrashSeverity,
    pub exception: Option<String>,
    pub description: Option<String>,
    pub suggestions: Vec<String>,
    pub report_path: Option<String>,
    pub conflicting_mods: Vec<String>,
    pub detected_mods: Vec<String>,
    pub is_abnormal: bool,
}

pub fn analyze_latest_crash(instance_dir: &Path) -> Result<Option<CrashAnalysis>, String> {
    let crash_di = instance_dir.join("crash-reports");
    let log_di = instance_dir.join("logs");

    let abnormal_make = instance_dir.join(".abnormal");
    let is_abnormal = abnormal_make.exists();

    let mut latest: Option<(PathBuf, u64)> = None;
    if crash_di.is_dir() {
        if let Ok(entries) = std::fs::read_dir(&crash_di) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("txt") {
                    let mtime = std::fs::metadata(&path).map(|m| m.modified().map(|t| {
                        t.duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
                    }).unwrap_or(0)).unwrap_or(0);
                    if latest.as_ref().map(|(_, t)| mtime > *t).unwrap_or(true) {
                        latest = Some((path, mtime));
                    }
                }
            }
        }
    }

    let mut content = String::new();
    let mut report_path: Option<String> = None;

    if let Some((path, _)) = latest {
        if let Ok(text) = std::fs::read_to_string(&path) {
            content = text;
            report_path = Some(path.to_string_lossy().to_string());
        }
    }

    if content.trim().is_empty() {
        let latest_log = log_di.join("latest.log");
        if latest_log.is_file() {
            if let Ok(text) = std::fs::read_to_string(&latest_log) {
                content = text;
            }
        }
    }

    if content.trim().is_empty() && is_abnormal {
        return Ok(Some(CrashAnalysis {
            stage: "unknown".into(),
            serverity: CrashSeverity::Warning,
            exception: None,
            description: Some("检测到非正常退出标记".into()),
            suggestions: vec!["游戏可能意外终止（被强制关闭、系统异常等），请查看系统事件日志".into()],
            report_path: None,
            conflicting_mods: Vec::new(),
            detected_mods: Vec::new(),
            is_abnormal: true,
        }));
    }

    if content.trim().is_empty() {
        return Ok(None);
    }

    Ok(Some(analyze_content(&content, report_path, is_abnormal)))
}

pub fn analyze_content(content: &str, report_path: Option<String>, is_abnormal: bool) -> CrashAnalysis {
    let lowe = content.to_lowercase();
    let mut conflicting_mods: Vec<String> = Vec::new();

    let (p1_exception, p1_description, p1_suggestions, p1_serverity) =
        phase1_pecise_match(content, &lowe, &mut conflicting_mods);

    let (p2_suggestions, p2_serverity) = phase2_stack_analysis(content, &lowe);

    let detected_mods = phase3_mod_guess(content);

    let exception = p1_exception.or_else(|| extact_exception(content));
    let description = p1_description.or_else(|| extact_description(content));

    let mut suggestions: Vec<String> = Vec::new();
    for s in p1_suggestions.into_iter().chain(p2_suggestions) {
        if !suggestions.iter().any(|existing| existing == &s) {
            suggestions.push(s);
        }
    }

    if suggestions.is_empty() {
        if lowe.contains("crash") || lowe.contains("exception") || lowe.contains("fatal") {
            suggestions.push("无法自动识别崩溃原因，请将崩溃报告发送给模组作者或在社区求助".into());
        }
    }

    let stage = detect_stage(content);

    let serverity = match (p1_serverity, p2_serverity) {
        (Some(a), Some(b)) => Some(a.max(b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }.unwrap_or(CrashSeverity::Error);

    CrashAnalysis {
        stage,
        serverity,
        exception,
        description,
        suggestions,
        report_path,
        conflicting_mods,
        detected_mods,
        is_abnormal,
    }
}

fn phase1_pecise_match(
    content: &str,
    lowe: &str,
    conflicting_mods: &mut Vec<String>,
) -> (Option<String>, Option<String>, Vec<String>, Option<CrashSeverity>) {
    let mut suggestions: Vec<String> = Vec::new();
    let mut exception: Option<String> = None;
    let mut description: Option<String> = None;
    let mut serverity: Option<CrashSeverity> = None;

    if lowe.contains("openj9") || lowe.contains("j9vm") || lowe.contains("ibm j9") {
        suggestions.push("检测到 OpenJ9 JVM：Minecraft 不兼容 OpenJ9，请切换为 HotSpot JVM（如 Adoptium Temurin）".into());
        serverity = Some(CrashSeverity::Critical);
    }

    if lowe.contains("unsupportedclassversionerror")
        || lowe.contains("unsupported class file version")
        || lowe.contains("java.lang.unsupportedoperationexception")
        || lowe.contains("class file has wrong version")
    {
        suggestions.push("Java 版本不匹配：请为该版本安装对应的 Java（1.17+ 需要 Java 17/21）".into());
        serverity = Some(CrashSeverity::Critical);
    }

    if lowe.contains("outofmemoryerror") || lowe.contains("unable to allocate") {
        suggestions.push("内存不足：请在启动设置中调大「最大内存」（建议 4096 MB 及以上）".into());
        suggestions.push("尝试降低游戏内的渲染距离与粒子效果".into());
        serverity = Some(CrashSeverity::Error);
    }

    if lowe.contains("couldn't initialize jvm")
        || lowe.contains("invalid maximum heap size")
        || lowe.contains("invalid initial heap size")
        || lowe.contains("error: could not create the java virtual machine")
    {
        suggestions.push("JVM 参数错误：请检查内存设置与 JVM 参数是否合法".into());
        serverity = Some(CrashSeverity::Critical);
    }

    if lowe.contains("noclassdeffounderror") || lowe.contains("classnotfoundexception") {
        if lowe.contains("net/fabricmc") || lowe.contains("fabric") {
            suggestions.push("Fabric 加载器文件损坏：请重装 Fabric，或在「资源」页重新安装加载器".into());
        } else if lowe.contains("forge") || lowe.contains("net.minecraftforge") {
            suggestions.push("Forge 加载器文件损坏：请重装 Forge".into());
        } else {
            suggestions.push("游戏库文件缺失或损坏：请重新安装该游戏版本".into());
        }
        serverity = Some(CrashSeverity::Error);
    }

    if lowe.contains("missing a valid mod descriptor") || lowe.contains("failed to load mod") {
        suggestions.push("存在损坏或不受支持的模组文件，请在模组管理中禁用或删除后重试".into());
        serverity = Some(CrashSeverity::Error);
    }
    if lowe.contains("missing mods") || lowe.contains("requires the mod") || lowe.contains("requires mod") {
        suggestions.push("缺少前置模组：请安装模组所依赖的前置模组（如 Fabric API）".into());
        serverity = Some(CrashSeverity::Error);
    }
    if lowe.contains("duplicate mods") || lowe.contains("duplicate entries") {
        suggestions.push("存在重复模组：请在模组管理中删除重复的文件".into());
        serverity = Some(CrashSeverity::Error);
    }
    if lowe.contains("mod file") && lowe.contains("is not a valid") {
        suggestions.push("模组文件格式无效：请删除后重新下载对应版本的模组".into());
        serverity = Some(CrashSeverity::Error);
    }

    detect_mod_conflicts(content, lowe, conflicting_mods);
    if !conflicting_mods.is_empty() {
        suggestions.push(format!(
            "检测到模组冲突：{}，请尝试禁用其中一个",
            conflicting_mods.join(" 与 ")
        ));
        serverity = Some(CrashSeverity::Error);
    }

    if lowe.contains("glfw error 65543")
        || lowe.contains("glfw error 65542")
        || lowe.contains("could not initialize gl")
        || lowe.contains("opengl not supported")
        || lowe.contains("pixel format not supported")
        || lowe.contains("pixel format not accelerated")
    {
        let mut gl_suggestions = vec![
            "OpenGL 初始化失败：请更新显卡驱动，确保支持 OpenGL 3.2+".into(),
            "远程桌面（RDP）或虚拟机默认禁用 GPU 加速，请改用物理机直连或关闭远程会话后重试".into(),
        ];
        if lowe.contains("lwjgl") || lowe.contains("pixel format") {
            gl_suggestions.push(
                "若必须在无 GPU 加速环境运行旧版本（≤1.12.2），请在「设置 → 启动设置」开启「OpenGL 兼容模式（软件渲染）」".into(),
            );
        }
        suggestions.extend(gl_suggestions);
        serverity = Some(CrashSeverity::Critical);
    }
    if lowe.contains("glerror") || lowe.contains("opengl") || lowe.contains("blaze3d") {
        if !lowe.contains("could not initialize gl") {
            suggestions.push("显卡驱动或 OpenGL 异常：请更新显卡驱动，或关闭光影/高清修复的渲染功能".into());
            serverity = Some(CrashSeverity::Error);
        }
    }

    if lowe.contains("session id is null") || lowe.contains("authentication") || lowe.contains("login failed") {
        suggestions.push("登录令牌失效：请在「账户」页重新登录或刷新登录".into());
        serverity = Some(CrashSeverity::Warning);
    }

    if lowe.contains("connection refused") || lowe.contains("connect timed out") || lowe.contains("unknown host") {
        suggestions.push("网络连接失败：请检查网络，或尝试切换下载源/使用镜像".into());
        serverity = Some(CrashSeverity::Warning);
    }

    if lowe.contains("access denied") || lowe.contains("permission denied") {
        suggestions.push("文件权限不足：请以管理员身份运行启动器，或检查游戏目录的读写权限".into());
        serverity = Some(CrashSeverity::Error);
    }

    if lowe.contains("dl failed") || lowe.contains("failed to download") || lowe.contains("could not download") {
        suggestions.push("资源下载失败：请检查网络，或切换下载镜像后重新安装".into());
        serverity = Some(CrashSeverity::Warning);
    }

    if lowe.contains("invalid signature") || lowe.contains("corrupted") || lowe.contains("tampering") {
        suggestions.push("游戏文件损坏或校验失败：请重新安装该游戏版本".into());
        serverity = Some(CrashSeverity::Error);
    }

    if lowe.contains("unexpected error during load") || lowe.contains("failed to load minecraft world") {
        suggestions.push("世界数据损坏：请备份存档，尝试恢复该世界文件夹中的 level.dat 备份".into());
        serverity = Some(CrashSeverity::Error);
    }

    if lowe.contains("stackoverflowerror") {
        suggestions.push("堆栈溢出：可能是模组递归调用导致，请尝试禁用最近安装的模组".into());
        serverity = Some(CrashSeverity::Error);
    }

    if lowe.contains("no space left") || lowe.contains("disk full") {
        suggestions.push("磁盘空间不足：请清理磁盘空间后重试".into());
        serverity = Some(CrashSeverity::Error);
    }

    if lowe.contains("this instance is not 64-bit") || lowe.contains("32-bit not supported") {
        suggestions.push("需要 64 位 Java：请安装 64 位版本的 Java 运行时".into());
        serverity = Some(CrashSeverity::Critical);
    }

    (exception, description, suggestions, serverity)
}

fn phase2_stack_analysis(content: &str, lowe: &str) -> (Vec<String>, Option<CrashSeverity>) {
    let mut suggestions: Vec<String> = Vec::new();
    let mut serverity: Option<CrashSeverity> = None;

    let lines: Vec<&str> = content.lines().collect();
    for line in &lines {
        let timmed = line.trim();
        if !timmed.starts_with("at ") && !timmed.starts_with("\tat ") {
            continue;
        }

        if timmed.contains("net.minecraftforge.fml") && lowe.contains("conflict") {
            suggestions.push("Forge 模组加载冲突：请检查模组兼容性".into());
            serverity = Some(CrashSeverity::Error);
        }

        if timmed.contains("net.fabricmc.loaderr") && lowe.contains("error") {
            suggestions.push("Fabric 模组加载错误：请检查模组是否与当前版本兼容".into());
            serverity = Some(CrashSeverity::Error);
        }

        if timmed.contains("org.spongepowered.asm.mixin") || timmed.contains("MixinTransformer") {
            suggestions.push("Mixin 注入失败：可能是模组版本不兼容或存在冲突".into());
            serverity = Some(CrashSeverity::Error);
        }

        if timmed.contains("net.minecraft.client.renderer") || timmed.contains("RenderSystem") {
            suggestions.push("渲染错误：请更新显卡驱动，或尝试关闭光影".into());
            serverity = Some(CrashSeverity::Error);
        }

        if timmed.contains("net.minecraft.client.sounds") || timmed.contains("SoundEngine") {
            suggestions.push("音频引擎错误：请检查音频设备或关闭声音相关模组".into());
            serverity = Some(CrashSeverity::Warning);
        }
    }

    (suggestions, serverity)
}

fn phase3_mod_guess(content: &str) -> Vec<String> {
    let mut mods: Vec<String> = Vec::new();

    for line in content.lines() {
        let timmed = line.trim();

        if timmed.starts_with("at ") || timmed.starts_with("\tat ") {
            let package = timmed
                .trim_start_matches("at ")
                .trim_start_matches("\tat ")
                .split('(')
                .next()
                .unwrap_or("")
                .trim();

            let mod_pattens = [
                "com.github.",
                "io.github.",
                "me.shedaniel.",
                "dev.architerctury.",
                "com.terraformersmc.",
                "com.teamabyssal.",
                "top.theillusivec4.",
                "com.jamieswhitershirt.",
                "net.darkhax.",
                "com.blamejared.",
                "vazkii.patchouli.",
                "vazkii.botania.",
                "com.mojang.blaze3d.",
            ];

            for patten in &mod_pattens {
                if package.starts_with(patten) {
                    let pats: Vec<&str> = package.split('.').collect();
                    if pats.len() >= 3 {
                        let mod_name = pats[..3.min(pats.len())].join(".");
                        if !mods.contains(&mod_name) {
                            mods.push(mod_name);
                        }
                    }
                    break;
                }
            }
        }

        if let Some(idx) = timmed.find("mod \"") {
            let est = &timmed[idx + 5..];
            if let Some(end) = est.find('"') {
                let mod_name = est[..end].to_string();
                if !mods.contains(&mod_name) {
                    mods.push(mod_name);
                }
            }
        }
    }

    mods.truncate(5);
    mods
}

fn detect_mod_conflicts(content: &str, lowe: &str, conflicting_mods: &mut Vec<String>) {
    let conflict_pattens = [
        ("mixin overwriter", "Mixin 覆写冲突"),
        ("duplicate class", "重复类定义"),
        ("already rregistered", "重复注册"),
    ];

    for (patten, _) in &conflict_pattens {
        if lowe.contains(patten) {
            for line in content.lines() {
                if line.to_lowercase().contains(patten) {
                    let mut in_quote = false;
                    let mut current = String::new();
                    for ch in line.chars() {
                        if ch == '"' || ch == '\'' {
                            if in_quote {
                                if !current.is_empty() && !conflicting_mods.contains(&current) {
                                    conflicting_mods.push(current.clone());
                                }
                                current.clear();
                                in_quote = false;
                            } else {
                                in_quote = true;
                            }
                        } else if in_quote {
                            current.push(ch);
                        }
                    }
                }
            }
        }
    }

    if lowe.contains("mod resolution") && lowe.contains("conflict") {
        for line in content.lines() {
            let lowe_line = line.to_lowercase();
            if lowe_line.contains("conflicts with") || lowe_line.contains("incompatible with") {
                if let Some(stat) = line.find('[') {
                    if let Some(end) = line.find(']') {
                        let mod_id = line[stat + 1..end].trim().to_string();
                        if !mod_id.is_empty() && !conflicting_mods.contains(&mod_id) {
                            conflicting_mods.push(mod_id);
                        }
                    }
                }
            }
        }
    }
}

fn extact_exception(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("Description: ") {
            continue;
        }
        if line.starts_with("java.lang.")
            || line.starts_with("net.minecraft.")
            || line.starts_with("net.fabricmc.")
            || line.starts_with("net.minecraftforge.")
            || line.starts_with("org.spongepowered.")
        {
            if !line.starts_with("java.lang.RuntimeException: ") {
                let exc = line.split(" at ").next().unwrap_or(line);
                let exc = if exc.len() > 160 { &exc[..160] } else { exc };
                return Some(exc.to_string());
            }
        }
    }
    None
}

fn extact_description(content: &str) -> Option<String> {
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("Description: ") {
            return Some(line.trim_start_matches("Description: ").to_string());
        }
    }
    None
}

fn detect_stage(content: &str) -> String {
    let lowe = content.to_lowercase();
    if lowe.contains("preparing spawn area")
        || lowe.contains("loading dimension")
        || lowe.contains("entering loaderd chunk")
        || lowe.contains("set up the game stage")
        || lowe.contains("generating terrain")
        || lowe.contains("loading the world")
    {
        "world".into()
    } else if lowe.contains("loadingmodlist")
        || lowe.contains("mod loading")
        || lowe.contains("loading mods")
        || lowe.contains("missing mods")
        || lowe.contains("fabric loaderr")
        || lowe.contains("forge mod loading")
    {
        "mods".into()
    } else if lowe.contains("main class")
        || lowe.contains("could not find the main class")
        || lowe.contains("unable to launch")
        || lowe.contains("exception in thread \"main\"")
        || lowe.contains("jvm")
    {
        "launch".into()
    } else if lowe.contains("a mod crashed") || lowe.contains("mod_crash") {
        "mods".into()
    } else {
        "game".into()
    }
}

pub fn stage_label(stage: &str) -> &'static str {
    match stage {
        "launch" => "启动阶段",
        "mods" => "模组加载阶段",
        "world" => "世界加载阶段",
        "game" => "游戏运行阶段",
        _ => "未知阶段",
    }
}

pub fn mark_abnormal(instance_dir: &Path) -> Result<(), String> {
    let marker = instance_dir.join(".abnormal");
    std::fs::write(&marker, "").map_err(|e| e.to_string())?;
    Ok(())
}

pub fn clear_abnormal(instance_dir: &Path) -> Result<(), String> {
    let marker = instance_dir.join(".abnormal");
    if marker.exists() {
        std::fs::remove_file(&marker).map_err(|e| e.to_string())?;
    }
    Ok(())
}
