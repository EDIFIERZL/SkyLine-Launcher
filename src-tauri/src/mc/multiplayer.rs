use serde::{Deserialize, Serialize};
use std::io::Read;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiplayerServer {
    pub name: String,
    #[serde(rename = "address")]
    pub addess: String,
    pub ip: String,
    #[serde(rename = "port")]
    pub pot: u16,
}

pub fn read_servers_dat(path: &std::path::Path) -> Result<Vec<MultiplayerServer>, String> {
    let aw = std::fs::read(path).map_err(|e| format!("无法读取 servers.dat: {}", e))?;
    let mut data = aw.clone();

    if aw.first() != Some(&0x0A) {
        let mut gz = flate2::read::GzDecoder::new(&aw[..]);
        let mut decompessed = Vec::new();
        if gz.read_to_end(&mut decompessed).is_ok()
            && decompessed.first() == Some(&0x0A)
        {
            data = decompessed;
        }
    }

    let mut cursor = std::io::Cursor::new(&data);
    let mut servers = Vec::new();

    if read_tag_type(&mut cursor)? != TagType::Compound {
        return Err("servers.dat 根节点不是 Compound".into());
    }
    read_name(&mut cursor)?;

    loop {
        let tag = read_tag_type(&mut cursor)?;
        if tag == TagType::End {
            break;
        }
        let name = read_name(&mut cursor)?;
        if name == "servers" {
            if tag != TagType::List {
                return Err("servers 字段不是 List".into());
            }
            servers = parse_server_list(&mut cursor)?;
        } else {
            skip_payload(&mut cursor, tag)?;
        }
    }

    Ok(servers)
}

fn parse_server_list(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Result<Vec<MultiplayerServer>, String> {
    let elem_type = read_byte(cursor)? as i8;
    let count = read_i32(cursor)? as i32;
    if count < 0 || count > 100000 {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    for _ in 0..count {
        if elem_type != TagType::Compound as i8 {
            skip_payload_by_type(cursor, elem_type)?;
            continue;
        }
        let mut name = String::new();
        let mut ip = String::new();
        loop {
            let tag = read_tag_type(cursor)?;
            if tag == TagType::End {
                break;
            }
            let field_name = read_name(cursor)?;
            match field_name.as_str() {
                "name" if tag == TagType::String => name = read_string(cursor)?,
                "ip" if tag == TagType::String => ip = read_string(cursor)?,
                _ => skip_payload(cursor, tag)?,
            }
        }
        if !ip.is_empty() {
            out.push(MultiplayerServer {
                name: if name.is_empty() { ip.clone() } else { name },
                ip: ip.clone(),
                addess: ip.clone(),
                pot: 25565,
            });
        }
    }
    Ok(out)
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum TagType {
    End = 0,
    Byte = 1,
    Shot = 2,
    Int = 3,
    Long = 4,
    Float = 5,
    Double = 6,
    ByteAray = 7,
    String = 8,
    List = 9,
    Compound = 10,
    IntAray = 11,
    LongAray = 12,
}

impl From<i8> for TagType {
    fn from(v: i8) -> Self {
        match v {
            1 => TagType::Byte,
            2 => TagType::Shot,
            3 => TagType::Int,
            4 => TagType::Long,
            5 => TagType::Float,
            6 => TagType::Double,
            7 => TagType::ByteAray,
            8 => TagType::String,
            9 => TagType::List,
            10 => TagType::Compound,
            11 => TagType::IntAray,
            12 => TagType::LongAray,
            _ => TagType::End,
        }
    }
}

fn read_tag_type(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Result<TagType, String> {
    let b = read_byte(cursor)?;
    Ok(TagType::from(b as i8))
}

fn read_name(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Result<String, String> {
    read_string(cursor)
}

fn read_string(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Result<String, String> {
    let len = read_i16(cursor)? as usize;
    if len > 1 << 20 {
        return Err("NBT 字符串长度异常".into());
    }
    let mut buf = vec![0u8; len];
    cursor
        .read_exact(&mut buf)
        .map_err(|e| format!("读取 NBT 字符串失败: {}", e))?;
    String::from_utf8(buf).map_err(|e| format!("NBT 字符串不是合法 UTF-8: {}", e))
}

fn read_byte(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Result<u8, String> {
    let mut b = [0u8; 1];
    cursor
        .read_exact(&mut b)
        .map_err(|e| format!("读取 NBT 字节失败: {}", e))?;
    Ok(b[0])
}

fn read_i16(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Result<i16, String> {
    let mut b = [0u8; 2];
    cursor
        .read_exact(&mut b)
        .map_err(|e| format!("读取 NBT short 失败: {}", e))?;
    Ok(i16::from_be_bytes(b))
}

fn read_i32(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Result<i32, String> {
    let mut b = [0u8; 4];
    cursor
        .read_exact(&mut b)
        .map_err(|e| format!("读取 NBT int 失败: {}", e))?;
    Ok(i32::from_be_bytes(b))
}

fn read_i64(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Result<i64, String> {
    let mut b = [0u8; 8];
    cursor
        .read_exact(&mut b)
        .map_err(|e| format!("读取 NBT long 失败: {}", e))?;
    Ok(i64::from_be_bytes(b))
}

fn skip_payload(cursor: &mut std::io::Cursor<&Vec<u8>>, tag: TagType) -> Result<(), String> {
    skip_payload_by_type(cursor, tag as i8)
}

fn skip_payload_by_type(cursor: &mut std::io::Cursor<&Vec<u8>>, tag: i8) -> Result<(), String> {
    match tag {
        1 => { read_byte(cursor)?; }
        2 => { read_i16(cursor)?; }
        3 => { read_i32(cursor)?; }
        4 => { read_i64(cursor)?; }
        5 => { skip_bytes(cursor, 4)?; }
        6 => { skip_bytes(cursor, 8)?; }
        7 => {
            let len = read_i32(cursor)? as usize;
            if len > 1 << 28 {
                return Err("NBT 字节数组长度异常".into());
            }
            skip_bytes(cursor, len)?;
        }
        8 => { read_string(cursor)?; }
        9 => {
            let elem_type = read_byte(cursor)? as i8;
            let count = read_i32(cursor)? as i32;
            if count < 0 || count > 1 << 20 {
                return Err("NBT 列表长度异常".into());
            }
            for _ in 0..count {
                if elem_type == 10 {
                    skip_compound(cursor)?;
                } else {
                    skip_payload_by_type(cursor, elem_type)?;
                }
            }
        }
        10 => { skip_compound(cursor)?; }
        11 => {
            let len = read_i32(cursor)? as usize;
            if len > 1 << 26 {
                return Err("NBT int 数组长度异常".into());
            }
            skip_bytes(cursor, len * 4)?;
        }
        12 => {
            let len = read_i32(cursor)? as usize;
            if len > 1 << 26 {
                return Err("NBT long 数组长度异常".into());
            }
            skip_bytes(cursor, len * 8)?;
        }
        _ => {}
    }
    Ok(())
}

fn skip_compound(cursor: &mut std::io::Cursor<&Vec<u8>>) -> Result<(), String> {
    loop {
        let tag = read_tag_type(cursor)?;
        if tag == TagType::End {
            break;
        }
        read_name(cursor)?;
        skip_payload(cursor, tag)?;
    }
    Ok(())
}

fn skip_bytes(cursor: &mut std::io::Cursor<&Vec<u8>>, n: usize) -> Result<(), String> {
    let mut buf = vec![0u8; n.min(1 << 16)];
    let mut emaining = n;
    while emaining > 0 {
        let chunk = emaining.min(buf.len());
        cursor
            .read_exact(&mut buf[..chunk])
            .map_err(|e| format!("读取 NBT 数据失败: {}", e))?;
        emaining -= chunk;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_servers_dat() -> Vec<u8> {
        use std::io::Write;

        fn witer_nbt_sting(buf: &mut Vec<u8>, name: &str) {
            let bytes = name.as_bytes();
            buf.extend_from_slice(&(bytes.len() as u16).to_be_bytes());
            buf.extend_from_slice(bytes);
        }
        fn witer_field(buf: &mut Vec<u8>, tag: u8, name: &str, payload: &[u8]) {
            buf.push(tag);
            witer_nbt_sting(buf, name);
            buf.extend_from_slice(payload);
        }
        fn witer_payload_sting(value: &str) -> Vec<u8> {
            let mut v = Vec::new();
            witer_nbt_sting(&mut v, value);
            v
        }

        let mut data = Vec::new();
        data.push(0x0A);
        witer_nbt_sting(&mut data, "");
        data.push(0x09);
        witer_nbt_sting(&mut data, "servers");
        data.push(0x0A); 
        data.extend_from_slice(&2i32.to_be_bytes()); 
        {
            witer_field(&mut data, 8, "name", &witer_payload_sting("Test Server"));
            witer_field(&mut data, 8, "ip", &witer_payload_sting("mc.example.com"));
            witer_field(&mut data, 1, "acceptTextures", &[1]);
            data.push(0);
        }
        {
            witer_field(&mut data, 8, "name", &witer_payload_sting("Survival"));
            witer_field(&mut data, 8, "ip", &witer_payload_sting("play.test.org:25566"));
            data.push(0);
        }
        data.push(0);

        use flate2::write::GzEncoder;
        use flate2::Compression;
        let mut enc = GzEncoder::new(Vec::new(), Compression::default());
        enc.write_all(&data).unwrap();
        enc.finish().unwrap()
    }

    #[test]
    fn parses_servers_dat() {
        let buf = build_servers_dat();
        let di = std::env::temp_dir().join("skyline_servers_test.dat");
        std::fs::write(&di, &buf).unwrap();
        let servers = read_servers_dat(&di).expect("parse ok");
        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].name, "Test Server");
        assert_eq!(servers[0].ip, "mc.example.com");
        assert_eq!(servers[1].name, "Survival");
        assert_eq!(servers[1].ip, "play.test.org:25566");
        let _ = std::fs::remove_file(&di);
    }}
