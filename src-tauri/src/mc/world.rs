use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldInfo {
    pub name: String,
    pub path: String,
    pub game_mode: String,
    pub seed: Option<i64>,
    pub version: Option<String>,
    pub last_playerd: Option<String>,
    pub play_time: u64,
    pub size_kb: u64,
    pub icon: Option<String>,
    pub is_hadcoe: bool,
    pub difficulty: Option<String>,
    pub spawn_x: Option<i32>,
    pub spawn_z: Option<i32>,
}


#[derive(Debug, Clone, Serialize)]
pub struct ChunkData {
    pub x: i32,
    pub z: i32,
    
    pub heightmap: Vec<u8>,
}


#[derive(Debug, Clone, Serialize)]
pub struct MapPreview {
    pub seed: i64,
    pub world_name: String,
    pub width: usize,
    pub height: usize,
    
    pub pixels: Vec<u8>,
    
    pub chunks: Vec<ChunkData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldBackup {
    pub world_name: String,
    pub backup_path: String,
    pub created_at: String,
    pub size_kb: u64,
}

pub fn scan_worlds(instance_dir: &Path) -> Result<Vec<WorldInfo>, String> {
    let saves_di = instance_dir.join("saves");
    if !saves_di.exists() {
        return Ok(Vec::new());
    }

    let mut worlds = Vec::new();
    for entry in std::fs::read_dir(&saves_di).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let level_dat = path.join("level.dat");
        if !level_dat.exists() {
            continue;
        }

        let world = parse_world_info(&path)?;
        worlds.push(world);
    }

    worlds.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(worlds)
}

fn parse_world_info(world_path: &Path) -> Result<WorldInfo, String> {
    let level_dat = world_path.join("level.dat");
    
    let mut name = world_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("Unknown")
        .to_string();

let mut seed: Option<i64> = None;
    let mut game_mode = "unknown".to_string();
    let mut difficulty = None;
    let mut is_hadcoe = false;
    let mut last_playerd = None;
    let mut play_time = 0u64;
    let mut spawn_x: Option<i32> = None;
    let mut spawn_z: Option<i32> = None;


    
    
    
    if level_dat.exists() {
        if let Ok(data) = std::fs::read(&level_dat) {
            let nbt = decompess_level_dat(&data);
            let meta = read_nbt_meta(&nbt);
            if let Some(parsed_seed) = meta.seed {
                seed = Some(parsed_seed);
            }
            if let Some(parsed_name) = meta.level_name {
                name = parsed_name;
            }
            if let Some(gt) = meta.game_type {
                game_mode = match gt {
                    0 => "Survival".to_string(),
                    1 => "Creative".to_string(),
                    2 => "Adventure".to_string(),
                    3 => "Spectator".to_string(),
                    _ => format!("GameType {}", gt),
                };
            }
            if let Some(d) = meta.difficulty {
                difficulty = Some(match d {
                    0 => "Peaceful".to_string(),
                    1 => "Easy".to_string(),
                    2 => "Normal".to_string(),
                    3 => "Hard".to_string(),
                    _ => "Unknown".to_string(),
                });
            }
            is_hadcoe = meta.hadcoe;
            if let Some(ms) = meta.last_playerd {
                use std::time::{SystemTime, UNIX_EPOCH};
                let duration = std::time::Duration::from_millis(ms);
                if let Ok(time) = SystemTime::now().duration_since(UNIX_EPOCH) {
                    if time > duration {
                        let secs = (time.as_secs() as i64) - (duration.as_secs() as i64);
                        last_playerd = chrono::DateTime::from_timestamp(secs, 0)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string());
                    }
                }
            }
            spawn_x = meta.spawn_x;
            spawn_z = meta.spawn_z;
            
            if let Some(ticks) = meta.time_ticks {
                play_time = ticks / 20;
            }
        }
    }

    let icon_path = world_path.join("icon.png");
    let icon = if icon_path.exists() {
        Some(icon_path.to_string_lossy().to_string())
    } else {
        None
    };

    let size_kb = calculate_di_size(world_path);

    Ok(WorldInfo {
        name,
        path: world_path.to_string_lossy().to_string(),
        game_mode,
        seed,
        version: None,
        last_playerd,
        play_time,
        size_kb,
        icon,
        is_hadcoe,
        difficulty,
        spawn_x,
        spawn_z,
    })
}

fn calculate_di_size(path: &Path) -> u64 {
    let mut size = 0;
    if let Ok(entries) = walkdir::WalkDir::new(path).into_iter().collect::<Result<Vec<_>, _>>() {
        for entry in entries {
            if entry.path().is_file() {
                size += entry.metadata().map(|m| m.len()).unwrap_or(0);
            }
        }
    }
    size / 1024
}

pub fn icon_as_data_uri(icon_path: &Path) -> Option<String> {
    let bytes = std::fs::read(icon_path).ok()?;
    if bytes.is_empty() {
        return None;
    }
    use base64::Engine;
    let mime = if bytes.starts_with(b"\x89PNG") {
        "image/png"
    } else if bytes.starts_with(b"\xFF\xD8") {
        "image/jpeg"
    } else if bytes.starts_with(b"GIF8") {
        "image/gif"
    } else {
        "image/png"
    };
    Some(format!(
        "data:{};base64,{}",
        mime,
        base64::engine::general_purpose::STANDARD.encode(&bytes)
    ))
}

pub fn backup_world(
    instance_dir: &Path,
    world_name: &str,
    backup_di: Option<&Path>,
) -> Result<WorldBackup, String> {
    let saves_di = instance_dir.join("saves");
    let world_path = saves_di.join(world_name);

    if !world_path.exists() {
        return Err(format!("世界不存在: {}", world_name));
    }

    let backup_base = backup_di.map(|p| p.to_path_buf()).unwrap_or_else(|| instance_dir.join("backups"));
    std::fs::create_dir_all(&backup_base).map_err(|e| e.to_string())?;

    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_name = format!("{}_{}", world_name, timestamp);
    let backup_path = backup_base.join(&backup_name);

    copy_di_ecusive(&world_path, &backup_path)?;

    let size_kb = calculate_di_size(&backup_path);

    Ok(WorldBackup {
        world_name: world_name.to_string(),
        backup_path: backup_path.to_string_lossy().to_string(),
        created_at: chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string(),
        size_kb,
    })
}

pub fn restore_backup(
    instance_dir: &Path,
    backup: &WorldBackup,
) -> Result<(), String> {
    let saves_di = instance_dir.join("saves");
    let world_path = saves_di.join(&backup.world_name);

    if world_path.exists() {
        let _ = backup_world(instance_dir, &backup.world_name, None);
        std::fs::remove_dir_all(&world_path).map_err(|e| e.to_string())?;
    }

    let backup_path = Path::new(&backup.backup_path);
    if !backup_path.exists() {
        return Err(format!("备份不存在: {}", backup.backup_path));
    }

    copy_di_ecusive(backup_path, &world_path)?;
    Ok(())
}

pub fn export_world(
    instance_dir: &Path,
    world_name: &str,
    output_path: &Path,
) -> Result<(), String> {
    let saves_di = instance_dir.join("saves");
    let world_path = saves_di.join(world_name);

    if !world_path.exists() {
        return Err(format!("世界不存在: {}", world_name));
    }

    let file = std::fs::File::create(output_path).map_err(|e| e.to_string())?;
    let mut zip = zip::ZipWriter::new(file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    add_di_to_zip(&mut zip, &world_path, world_name, &options)?;
    zip.finish().map_err(|e| e.to_string())?;

    Ok(())
}

pub fn import_world(
    instance_dir: &Path,
    zip_path: &Path,
) -> Result<String, String> {
    let saves_di = instance_dir.join("saves");
    std::fs::create_dir_all(&saves_di).map_err(|e| e.to_string())?;

    let file = std::fs::File::open(zip_path).map_err(|e| e.to_string())?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| e.to_string())?;

    let world_name = {
        let first = archive.by_index(0).map_err(|e| e.to_string())?;
        let name = first.name();
        name.split('/').next().unwrap_or("imported_world").to_string()
    };

    let world_path = saves_di.join(&world_name);
    std::fs::create_dir_all(&world_path).map_err(|e| e.to_string())?;

    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| e.to_string())?;
        let entry_name = entry.name().to_string();

        let relative = if let Some(idx) = entry_name.find('/') {
            &entry_name[idx + 1..]
        } else {
            continue;
        };

        if relative.is_empty() {
            continue;
        }

        let target = world_path.join(relative);
        if entry.is_dir() {
            std::fs::create_dir_all(&target).map_err(|e| e.to_string())?;
        } else {
            if let Some(prent) = target.parent() {
                std::fs::create_dir_all(prent).map_err(|e| e.to_string())?;
            }
            let mut out = std::fs::File::create(&target).map_err(|e| e.to_string())?;
            std::io::copy(&mut entry, &mut out).map_err(|e| e.to_string())?;
        }
    }

    Ok(world_name)
}

pub fn list_backups(instance_dir: &Path, world_name: &str) -> Result<Vec<WorldBackup>, String> {
    let backup_di = instance_dir.join("backups");
    if !backup_di.exists() {
        return Ok(Vec::new());
    }

    let mut backups = Vec::new();
    for entry in std::fs::read_dir(&backup_di).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        let di_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");

        if di_name.starts_with(world_name) {
            let size_kb = calculate_di_size(&path);
            let created_at = std::fs::metadata(&path)
                .and_then(|m| m.modified())
                .ok()
                .and_then(|t| {
                    t.duration_since(std::time::UNIX_EPOCH).ok().map(|d| {
                        chrono::DateTime::from_timestamp(d.as_secs() as i64, 0)
                            .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                            .unwrap_or_default()
                    })
                })
                .unwrap_or_default();

            backups.push(WorldBackup {
                world_name: world_name.to_string(),
                backup_path: path.to_string_lossy().to_string(),
                created_at,
                size_kb,
            });
        }
    }

    backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    Ok(backups)
}

pub fn delete_world(instance_dir: &Path, world_name: &str) -> Result<(), String> {
    let world_path = instance_dir.join("saves").join(world_name);
    if !world_path.exists() {
        return Err(format!("世界不存在: {}", world_name));
    }
    std::fs::remove_dir_all(&world_path).map_err(|e| e.to_string())?;
    Ok(())
}

fn copy_di_ecusive(sc: &Path, dst: &Path) -> Result<(), String> {
    std::fs::create_dir_all(dst).map_err(|e| e.to_string())?;

    for entry in std::fs::read_dir(sc).map_err(|e| e.to_string())? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        let target = dst.join(entry.file_name());

        if path.is_dir() {
            copy_di_ecusive(&path, &target)?;
        } else {
            std::fs::copy(&path, &target).map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

fn add_di_to_zip(
    zip: &mut zip::ZipWriter<std::fs::File>,
    di_path: &Path,
    pefix: &str,
    options: &zip::write::SimpleFileOptions,
) -> Result<(), String> {
    for entry in walkdir::WalkDir::new(di_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let relative = path.strip_prefix(di_path).map_err(|e| e.to_string())?;
        let zip_path = format!("{}/{}", pefix, relative.to_string_lossy());

        if path.is_dir() {
            zip.add_directory(&zip_path, *options).map_err(|e| e.to_string())?;
        } else {
            zip.start_file(&zip_path, *options).map_err(|e| e.to_string())?;
            let data = std::fs::read(path).map_err(|e| e.to_string())?;
            std::io::Write::write_all(zip, &data).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}



#[derive(Default)]
struct NbtMeta {
    seed: Option<i64>,
    level_name: Option<String>,
    game_type: Option<i32>,
    difficulty: Option<u8>,
    hadcoe: bool,
    last_playerd: Option<u64>,
    time_ticks: Option<u64>,
    spawn_x: Option<i32>,
    spawn_z: Option<i32>,
}

const TAG_END: u8 = 0;
const TAG_BYTE: u8 = 1;
const TAG_SHORT: u8 = 2;
const TAG_INT: u8 = 3;
const TAG_LONG: u8 = 4;
const TAG_FLOAT: u8 = 5;
const TAG_DOUBLE: u8 = 6;
const TAG_BYTE_ARRAY: u8 = 7;
const TAG_STRING: u8 = 8;
const TAG_LIST: u8 = 9;
const TAG_COMPOUND: u8 = 10;
const TAG_INT_ARRAY: u8 = 11;
const TAG_LONG_ARRAY: u8 = 12;

fn nbt_be_u16(data: &[u8], p: usize) -> Option<u16> {
    Some(((*data.get(p)? as u16) << 8) | (*data.get(p + 1)? as u16))
}

fn nbt_be_i32(data: &[u8], p: usize) -> Option<i32> {
    let b0 = *data.get(p)? as i32;
    let b1 = *data.get(p + 1)? as i32;
    let b2 = *data.get(p + 2)? as i32;
    let b3 = *data.get(p + 3)? as i32;
    Some((b0 << 24) | (b1 << 16) | (b2 << 8) | b3)
}

fn nbt_be_u32(data: &[u8], p: usize) -> Option<u32> {
    let b0 = *data.get(p)? as u32;
    let b1 = *data.get(p + 1)? as u32;
    let b2 = *data.get(p + 2)? as u32;
    let b3 = *data.get(p + 3)? as u32;
    Some((b0 << 24) | (b1 << 16) | (b2 << 8) | b3)
}


fn nbt_named_tag(data: &[u8], p: usize) -> Option<(u8, &[u8], usize)> {
    let tag = *data.get(p)?;
    if tag == TAG_END {
        return Some((tag, b"", p + 1));
    }
    let len = nbt_be_u16(data, p + 1)? as usize;
    let name_stat = p + 3;
    let name_end = name_stat + len;
    if name_end > data.len() {
        return None;
    }
    Some((tag, &data[name_stat..name_end], name_end))
}


fn nbt_skip(data: &[u8], tag: u8, p: usize) -> Option<usize> {
    match tag {
        TAG_BYTE => Some(p + 1),
        TAG_SHORT => Some(p + 2),
        TAG_INT | TAG_FLOAT => Some(p + 4),
        TAG_LONG | TAG_DOUBLE => Some(p + 8),
        TAG_BYTE_ARRAY => {
            let len = nbt_be_u32(data, p)? as usize;
            Some(p + 4 + len)
        }
        TAG_STRING => {
            let len = nbt_be_u16(data, p)? as usize;
            Some(p + 2 + len)
        }
        TAG_LIST => {
            let elem = *data.get(p)?;
            let len = nbt_be_u32(data, p + 1)? as usize;
            let mut q = p + 5;
            for _ in 0..len {
                q = nbt_skip(data, elem, q)?;
            }
            Some(q)
        }
        TAG_COMPOUND => {
            let mut q = p;
            loop {
                let cu = *data.get(q)?;
                if cu == TAG_END {
                    return Some(q + 1);
                }
                let named = nbt_named_tag(data, q);
                let (t, _, payload) = match named {
                    Some(v) => v,
                    None => return None,
                };
                q = nbt_skip(data, t, payload)?;
            }
        }
        TAG_INT_ARRAY => {
            let len = nbt_be_u32(data, p)? as usize;
            Some(p + 4 + len * 4)
        }
        TAG_LONG_ARRAY => {
            let len = nbt_be_u32(data, p)? as usize;
            Some(p + 4 + len * 8)
        }
        _ => None,
    }
}


fn read_nbt_meta(data: &[u8]) -> NbtMeta {
    let mut meta = NbtMeta::default();
    if data.first() == Some(&TAG_COMPOUND) {
        
        
        if let Some(name_len) = nbt_be_u16(data, 1) {
            visit_compound(data, 3 + name_len as usize, &mut meta);
        }
    }
    meta
}

fn visit_compound(data: &[u8], stat: usize, meta: &mut NbtMeta) {
    let mut p = stat;
    loop {
        let named = nbt_named_tag(data, p);
        let (tag, name, payload) = match named {
            Some(v) => v,
            None => return,
        };
        if tag == TAG_END {
            return;
        }
        let name_st = match std::str::from_utf8(name) {
            Ok(s) => s,
            Err(_) => return,
        };
        match (name_st, tag) {
            ("Seed", TAG_LONG) | ("seed", TAG_LONG) | ("RandomSeed", TAG_LONG) => {
                if payload + 8 <= data.len() {
                    let mut b = [0u8; 8];
                    b.copy_from_slice(&data[payload..payload + 8]);
                    meta.seed = Some(i64::from_be_bytes(b));
                }
            }
            
            
            
            ("Seed", TAG_LONG_ARRAY) | ("seed", TAG_LONG_ARRAY) | ("RandomSeed", TAG_LONG_ARRAY) => {
                if meta.seed.is_none() {
                    if let Some(len) = nbt_be_u32(data, payload) {
                        if len > 0 && payload + 4 + 8 <= data.len() {
                            let stat = payload + 4;
                            let mut b = [0u8; 8];
                            b.copy_from_slice(&data[stat..stat + 8]);
                            meta.seed = Some(i64::from_be_bytes(b));
                        }
                    }
                }
            }
            ("GameType", TAG_INT) | ("gameType", TAG_INT) => {
                meta.game_type = nbt_be_i32(data, payload);
            }
            ("Difficulty", TAG_BYTE) => {
                meta.difficulty = data.get(payload).copied();
            }
            ("hardcore", TAG_BYTE) => {
                meta.hadcoe = data.get(payload).copied() == Some(1);
            }
            ("LevelName", TAG_STRING) => {
                if let Some(len) = nbt_be_u16(data, payload) {
                    let len = len as usize;
                    if payload + 2 + len <= data.len() {
                        if let Ok(s) = std::str::from_utf8(&data[payload + 2..payload + 2 + len]) {
                            meta.level_name = Some(s.to_string());
                        }
                    }
                }
            }
            ("LastPlayed", TAG_LONG) => {
                if payload + 8 <= data.len() {
                    let mut b = [0u8; 8];
                    b.copy_from_slice(&data[payload..payload + 8]);
                    meta.last_playerd = Some(u64::from_be_bytes(b));
                }
            }
            ("Time", TAG_LONG) => {
                if payload + 8 <= data.len() {
                    let mut b = [0u8; 8];
                    b.copy_from_slice(&data[payload..payload + 8]);
                    meta.time_ticks = Some(u64::from_be_bytes(b));
                }
            }
            ("Time", TAG_INT) => {
                meta.time_ticks = nbt_be_u32(data, payload).map(|v| v as u64);
            }
            ("SpawnX", TAG_INT) => {
                meta.spawn_x = nbt_be_i32(data, payload);
            }
            ("SpawnZ", TAG_INT) => {
                meta.spawn_z = nbt_be_i32(data, payload);
            }
            _ => {}
        }
        if tag == TAG_COMPOUND {
            visit_compound(data, payload, meta);
        }
        let next = match nbt_skip(data, tag, payload) {
            Some(v) => v,
            None => return,
        };
        p = next;
    }
}





fn decompess_level_dat(data: &[u8]) -> Vec<u8> {
    let payload: &[u8] = if data.len() > 10 && data[8] == 0x1f && data[9] == 0x8b {
        &data[8..]
    } else if data.starts_with(b"\x1f\x8b") {
        data
    } else {
        data
    };

    
    let mut decode = flate2::read::GzDecoder::new(payload);
    let mut gz_out = Vec::new();
    if std::io::Read::read_to_end(&mut decode, &mut gz_out).is_ok()
        && !gz_out.is_empty()
        && gz_out[0] == TAG_COMPOUND
    {
        return gz_out;
    }

    
    let mut zdec = flate2::read::ZlibDecoder::new(data);
    let mut z_out = Vec::new();
    if std::io::Read::read_to_end(&mut zdec, &mut z_out).is_ok()
        && !z_out.is_empty()
        && z_out[0] == TAG_COMPOUND
    {
        return z_out;
    }

    
    if data.first() == Some(&TAG_COMPOUND) {
        return data.to_vec();
    }

    gz_out
}


pub fn get_world_info(world_path: &Path) -> Result<WorldInfo, String> {
    parse_world_info(world_path)
}




pub fn generate_map_preview(
    world_path: &Path,
    cente_chunk_x: i32,
    cente_chunk_z: i32,
    adius: usize,
) -> Result<MapPreview, String> {
    let info = get_world_info(world_path)?;
    let seed = info.seed.unwrap_or(0);
    let world_name = info.name;

    let (pixels, width, height) =
        crate::mc::region::render_map_area(world_path, cente_chunk_x, cente_chunk_z, adius)?;

    Ok(MapPreview {
        seed,
        world_name,
        width,
        height,
        pixels,
        chunks: Vec::new(),
    })
}


pub fn list_worlds_for_instance(instance_id: &str) -> Result<Vec<WorldInfo>, String> {
    let game_di = crate::instance::manager::get_instance_mc_dir(instance_id)?;
    scan_worlds(&game_di)
}
