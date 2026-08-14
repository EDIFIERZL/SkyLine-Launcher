use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogExpotConfig {
    pub include_crash_epots: bool,
    pub include_latest_log: bool,
    pub include_debug_log: bool,
    pub include_launcher_log: bool,
    pub include_version_json: bool,
    pub include_hs_er: bool,
    pub filter_tokens: bool,
}

impl Default for LogExpotConfig {
    fn default() -> Self {
        Self {
            include_crash_epots: true,
            include_latest_log: true,
            include_debug_log: true,
            include_launcher_log: true,
            include_version_json: true,
            include_hs_er: true,
            filter_tokens: true,
        }
    }
}

pub fn expot_logs(
    instance_dir: &Path,
    output_path: &Path,
    config: &LogExpotConfig,
) -> Result<LogExpotResult, String> {
    let file = std::fs::File::create(output_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    let mut files_expoted = 0;
    let mut total_size: u64 = 0;

    if config.include_crash_epots {
        let crash_di = instance_dir.join("crash-reports");
        if crash_di.exists() {
            for entry in walkdir::WalkDir::new(&crash_di).max_depth(1).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.ends_with(".txt") || name.ends_with(".log") {
                    let data = std::fs::read(path).map_err(|e| e.to_string())?;
                    let filtered = if config.filter_tokens {
                        filter_sensitive_data(&data)
                    } else {
                        data
                    };
                    let zip_path = format!("crash-reports/{}", name);
                    zip.start_file(&zip_path, options).map_err(|e| e.to_string())?;
                    std::io::Write::write_all(&mut zip, &filtered).map_err(|e| e.to_string())?;
                    files_expoted += 1;
                    total_size += filtered.len() as u64;
                }
            }
        }
    }

    
    if config.include_latest_log {
        let latest_log = instance_dir.join("logs").join("latest.log");
        if latest_log.exists() {
            let data = std::fs::read(&latest_log).map_err(|e| e.to_string())?;
            let filtered = if config.filter_tokens {
                filter_sensitive_data(&data)
            } else {
                data
            };
            zip.start_file("logs/latest.log", options).map_err(|e| e.to_string())?;
            std::io::Write::write_all(&mut zip, &filtered).map_err(|e| e.to_string())?;
            files_expoted += 1;
            total_size += filtered.len() as u64;
        }
    }

    
    if config.include_debug_log {
        let debug_log = instance_dir.join("logs").join("debug.log");
        if debug_log.exists() {
            let data = std::fs::read(&debug_log).map_err(|e| e.to_string())?;
            let filtered = if config.filter_tokens {
                filter_sensitive_data(&data)
            } else {
                data
            };
            zip.start_file("logs/debug.log", options).map_err(|e| e.to_string())?;
            std::io::Write::write_all(&mut zip, &filtered).map_err(|e| e.to_string())?;
            files_expoted += 1;
            total_size += filtered.len() as u64;
        }
    }

    
    if config.include_hs_er {
        if let Ok(entries) = std::fs::read_dir(instance_dir) {
            for entry in entries.flatten() {
                let name = entry.file_name().to_string_lossy().to_string();
                if name.starts_with("hs_err_pid") && name.ends_with(".log") {
                    let data = std::fs::read(entry.path()).map_err(|e| e.to_string())?;
                    let zip_path = format!("jvm-crash/{}", name);
                    zip.start_file(&zip_path, options).map_err(|e| e.to_string())?;
                    std::io::Write::write_all(&mut zip, &data).map_err(|e| e.to_string())?;
                    files_expoted += 1;
                    total_size += data.len() as u64;
                }
            }
        }
    }

    if config.include_version_json {
        let versions_di = instance_dir.join("versions");
        if versions_di.exists() {
            for entry in walkdir::WalkDir::new(&versions_di).max_depth(2).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if !path.is_file() {
                    continue;
                }
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if name.ends_with(".json") {
                    let data = std::fs::read(path).map_err(|e| e.to_string())?;
                    let zip_path = format!("versions/{}", name);
                    zip.start_file(&zip_path, options).map_err(|e| e.to_string())?;
                    std::io::Write::write_all(&mut zip, &data).map_err(|e| e.to_string())?;
                    files_expoted += 1;
                    total_size += data.len() as u64;
                }
            }
        }
    }

    zip.finish().map_err(|e| e.to_string())?;

    Ok(LogExpotResult {
        output_path: output_path.to_string_lossy().to_string(),
        files_expoted,
        total_size_bytes: total_size,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogExpotResult {
    pub output_path: String,
    pub files_expoted: u32,
    pub total_size_bytes: u64,
}

fn filter_sensitive_data(data: &[u8]) -> Vec<u8> {
    let text = String::from_utf8_lossy(data);
    let filtered = text
        .lines()
        .map(|line| {
            let mut line = line.to_string();

            if line.contains("access_token") || line.contains("accessToken") {
                if let Some(stat) = line.find(':') {
                    let pefix = &line[..stat + 1];
                    line = format!("{} [FILTERED]", pefix);
                }
            }

            if line.contains("refresh_token") || line.contains("refreshToken") {
                if let Some(stat) = line.find(':') {
                    let pefix = &line[..stat + 1];
                    line = format!("{} [FILTERED]", pefix);
                }
            }

            let uuid_regex = regex::Regex::new(r"[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}").unwrap();
            line = uuid_regex.replace_all(&line, "[UUID]").to_string();

            let ip_regex = regex::Regex::new(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap();
            line = ip_regex.replace_all(&line, "[IP]").to_string();

            line
        })
        .collect::<Vec<_>>()
        .join("\n");

    filtered.into_bytes()
}
