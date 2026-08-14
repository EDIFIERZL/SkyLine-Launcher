use base64::Engine;

fn mime_from_path(path: &str) -> &'static str {
    let ext = path
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "mp3" => "audio/mpeg",
        "flac" => "audio/flac",
        "wav" => "audio/wav",
        "ogg" | "oga" | "opus" => "audio/ogg",
        "m4a" | "aac" => "audio/mp4",
        "wma" => "audio/x-ms-wma",
        "aiff" | "aif" => "audio/aiff",
        "ape" => "audio/x-ape",
        "webm" => "audio/webm",
        _ => "application/octet-stream",
    }
}

#[tauri::command]
pub fn check_files_exist(paths: Vec<String>) -> Result<Vec<bool>, String> {
    Ok(paths
        .iter()
        .map(|p| std::fs::metadata(p).map(|m| m.is_file()).unwrap_or(false))
        .collect())
}

#[tauri::command]
pub fn read_audio_file(path: String) -> Result<String, String> {
    const MAX_SIZE: u64 = 400 * 1024 * 1024;
    let meta = std::fs::metadata(&path).map_err(|e| format!("无法读取文件: {e}"))?;
    if !meta.is_file() {
        return Err("不是有效的音频文件".into());
    }
    if meta.len() > MAX_SIZE {
        return Err("文件过大(>400MB)，暂不支持播放".into());
    }
    let bytes = std::fs::read(&path).map_err(|e| format!("读取文件失败: {e}"))?;
    if bytes.is_empty() {
        return Err("文件为空".into());
    }
    let mime = mime_from_path(&path);
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{mime};base64,{b64}"))
}
