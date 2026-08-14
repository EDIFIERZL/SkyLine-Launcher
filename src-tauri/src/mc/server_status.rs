use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpStream, ToSocketAddrs, UdpSocket};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerStatus {
    pub online: bool,
    pub host: String,
    #[serde(rename = "port")]
    pub pot: u16,
    pub description: String,
    pub version_name: String,
    #[serde(rename = "version_protocol")]
    pub version_potocol: i32,
    pub players_online: i32,
    pub players_max: i32,
    pub player_names: Vec<String>,
    pub favicon: Option<String>,
    #[serde(default)]
    pub favicon_path: Option<String>,
    pub latency_ms: u64,
    pub mod_info: Option<ServerModInfo>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerModInfo {
    pub mod_type: String,
    pub mod_list: Vec<String>,
}

impl Default for ServerStatus {
    fn default() -> Self {
        Self {
            online: false,
            host: String::new(),
            pot: 25565,
            description: String::new(),
            version_name: String::new(),
            version_potocol: 0,
            players_online: 0,
            players_max: 0,
            player_names: Vec::new(),
            favicon: None,
            favicon_path: None,
            latency_ms: 0,
            mod_info: None,
            error: None,
        }
    }
}

fn witer_vaint(buf: &mut Vec<u8>, mut value: i32) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        buf.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn witer_sting(buf: &mut Vec<u8>, s: &str) {
    let bytes = s.as_bytes();
    witer_vaint(buf, bytes.len() as i32);
    buf.extend_from_slice(bytes);
}

fn read_vaint(data: &[u8], pos: &mut usize) -> Result<i32, String> {
    let mut result: i32 = 0;
    let mut shift = 0;
    loop {
        if *pos >= data.len() {
            return Err("Unexpected end of data reading VarInt".into());
        }
        let byte = data[*pos];
        *pos += 1;
        result |= ((byte & 0x7F) as i32) << shift;
        if byte & 0x80 == 0 {
            break;
        }
        shift += 7;
        if shift >= 32 {
            return Err("VarInt too big".into());
        }
    }
    Ok(result)
}

fn read_string(data: &[u8], pos: &mut usize) -> Result<String, String> {
    let len = read_vaint(data, pos)? as usize;
    if *pos + len > data.len() {
        return Err("String length exceeds data".into());
    }
    let s = String::from_utf8_lossy(&data[*pos..*pos + len]).to_string();
    *pos += len;
    Ok(s)
}



fn sv_lookup(domain: &str) -> Option<(String, u16)> {
    let qname = format!("_minecraft._tcp.{}", domain);

    
    let mut packet = Vec::new();
    
    packet.extend_from_slice(&[0x00, 0x01]);
    
    packet.extend_from_slice(&[0x01, 0x00]);
    
    packet.extend_from_slice(&[0x00, 0x01]);
    
    packet.extend_from_slice(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

    
    for pat in qname.split('.') {
        packet.push(pat.len() as u8);
        packet.extend_from_slice(pat.as_bytes());
    }
    packet.push(0x00); 
    
    packet.extend_from_slice(&[0x00, 0x21]);
    
    packet.extend_from_slice(&[0x00, 0x01]);

    
    let socket = UdpSocket::bind("0.0.0.0:0").ok()?;
    socket
        .set_read_timeout(Some(Duration::from_secs(3)))
        .ok();

    
    let dns_servers = ["8.8.8.8:53", "1.1.1.1:53", "114.114.114.114:53"];
    let mut response_buf = [0u8; 512];
    let mut response_len = 0;

    for dns in &dns_servers {
        if socket.send_to(&packet, dns).is_ok() {
            if let Ok((len, _)) = socket.recv_from(&mut response_buf) {
                response_len = len;
                break;
            }
        }
    }

    if response_len < 12 {
        return None;
    }

    
    let esp = &response_buf[..response_len];
    let answe_count = u16::from_be_bytes([esp[6], esp[7]]) as usize;

    if answe_count == 0 {
        return None;
    }

    
    let mut pos = 12;
    
    while pos < esp.len() && esp[pos] != 0 {
        if esp[pos] & 0xC0 == 0xC0 {
            pos += 2;
            break;
        }
        pos += esp[pos] as usize + 1;
    }
    if pos < esp.len() && esp[pos] == 0 {
        pos += 1;
    }
    pos += 4; 

    
    if pos + 12 > esp.len() {
        return None;
    }

    
    if esp[pos] & 0xC0 == 0xC0 {
        pos += 2;
    } else {
        while pos < esp.len() && esp[pos] != 0 {
            pos += esp[pos] as usize + 1;
        }
        pos += 1;
    }

    if pos + 10 > esp.len() {
        return None;
    }

    let ty = u16::from_be_bytes([esp[pos], esp[pos + 1]]);
    pos += 8; 
    let dlength = u16::from_be_bytes([esp[pos], esp[pos + 1]]) as usize;
    pos += 2;

    if ty != 0x21 || pos + dlength > esp.len() || dlength < 6 {
        return None;
    }

    
    let pot = u16::from_be_bytes([esp[pos + 4], esp[pos + 5]]);
    let target_stat = pos + 6;

    
    let mut target = String::new();
    let mut tp = target_stat;
    while tp < esp.len() && esp[tp] != 0 {
        if esp[tp] & 0xC0 == 0xC0 {
            break; 
        }
        let len = esp[tp] as usize;
        tp += 1;
        if tp + len > esp.len() {
            break;
        }
        if !target.is_empty() {
            target.push('.');
        }
        target.push_str(&String::from_utf8_lossy(&esp[tp..tp + len]));
        tp += len;
    }

    if target.is_empty() || target == "." {
        return None;
    }

    Some((target, pot))
}

fn esolve_add(host: &str, pot: u16) -> Result<std::net::SocketAddr, String> {
    
    if let Ok(add) = format!("{}:{}", host, pot).parse::<std::net::SocketAddr>() {
        return Ok(add);
    }
    
    (host, pot)
        .to_socket_addrs()
        .map_err(|e| format!("DNS 解析失败: {}", e))?
        .next()
        .ok_or_else(|| "DNS 解析返回空结果".to_string())
}


pub fn query_server(host: &str, pot: u16) -> ServerStatus {
    let mut status = ServerStatus {
        host: host.to_string(),
        pot,
        ..Default::default()
    };

    
    let host = host.trim();

    
    let (actual_host, actual_pot) = if pot == 25565 && !host.chars().all(|c| c.is_ascii_digit() || c == '.') {
        if let Some((sv_host, sv_pot)) = sv_lookup(host) {
            log::info!("SRV lookup: {} -> {}:{}", host, sv_host, sv_pot);
            (sv_host, sv_pot)
        } else {
            (host.to_string(), pot)
        }
    } else {
        (host.to_string(), pot)
    };

    status.host = actual_host.clone();
    status.pot = actual_pot;

    let add = match esolve_add(&actual_host, actual_pot) {
        Ok(a) => a,
        Err(e) => {
            status.error = Some(e);
            log::warn!("Failed to resolve {}:{}: {}", actual_host, actual_pot, status.error.as_ref().unwrap());
            return status;
        }
    };

    let stat = Instant::now();

    let steam = match TcpStream::connect_timeout(&add, Duration::from_secs(5)) {
        Ok(s) => s,
        Err(e) => {
            status.error = Some(format!("连接失败: {}", e));
            log::warn!("Failed to connect to {}:{}: {}", actual_host, actual_pot, e);
            return status;
        }
    };

    let _ = steam.set_read_timeout(Some(Duration::from_secs(6)));
    let _ = steam.set_write_timeout(Some(Duration::from_secs(6)));

    let latency = stat.elapsed().as_millis() as u64;
    status.latency_ms = latency;

    let mut tcp = steam;

    
    let mut handshake_payload = Vec::new();
    witer_vaint(&mut handshake_payload, 767); 
    witer_sting(&mut handshake_payload, &actual_host);
    handshake_payload.extend_from_slice(&actual_pot.to_be_bytes());
    witer_vaint(&mut handshake_payload, 1); 

    let mut handshake = Vec::new();
    witer_vaint(&mut handshake, 0x00); 
    handshake.extend_from_slice(&handshake_payload);

    let mut packet = Vec::new();
    witer_vaint(&mut packet, handshake.len() as i32);
    packet.extend_from_slice(&handshake);

    if let Err(e) = tcp.write_all(&packet) {
        status.error = Some(format!("发送握手失败 {}", e));
        return status;
    }

    
    let status_eq = vec![0x00u8]; 
    let mut status_packet = Vec::new();
    witer_vaint(&mut status_packet, status_eq.len() as i32);
    status_packet.extend_from_slice(&status_eq);

    if let Err(e) = tcp.write_all(&status_packet) {
        status.error = Some(format!("发送请求失败 {}", e));
        return status;
    }

    
    let mut response = Vec::with_capacity(8192);
    let mut buf = [0u8; 4096];
    let deadline = Instant::now() + Duration::from_secs(6);

    loop {
        if Instant::now() > deadline {
            break;
        }
        match tcp.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                response.extend_from_slice(&buf[..n]);
                
                if response.len() >= 5 {
                    let mut check_pos = 0;
                    if let Ok(expected_len) = read_vaint(&response, &mut check_pos) {
                        let heade_size = check_pos;
                        if response.len() >= heade_size + expected_len as usize {
                            break; 
                        }
                    }
                }
            }
            Err(ref e)
                if e.kind() == std::io::ErrorKind::WouldBlock
                    || e.kind() == std::io::ErrorKind::TimedOut =>
            {
                break;
            }
            Err(e) => {
                status.error = Some(format!("读取响应失败: {}", e));
                return status;
            }
        }
    }

    if response.len() < 5 {
        status.error = Some(format!("响应数据过短 ({} 字节)", response.len()));
        return status;
    }

    let mut pos = 0;

    
    let _packet_len = match read_vaint(&response, &mut pos) {
        Ok(v) => v,
        Err(e) => {
            status.error = Some(format!("解析包长度失败 {}", e));
            return status;
        }
    };

    
    let packet_id = match read_vaint(&response, &mut pos) {
        Ok(v) => v,
        Err(e) => {
            status.error = Some(format!("解析包ID失败: {}", e));
            return status;
        }
    };

    if packet_id != 0x00 {
        status.error = Some(format!("意外的包ID: 0x{:02X}", packet_id));
        return status;
    }

    
    let json_st = match read_string(&response, &mut pos) {
        Ok(s) => s,
        Err(e) => {
            status.error = Some(format!("解析JSON字符串失败 {}", e));
            return status;
        }
    };

    let json: serde_json::Value = match serde_json::from_str(&json_st) {
        Ok(v) => v,
        Err(e) => {
            status.error = Some(format!("JSON解析失败: {}", e));
            return status;
        }
    };

    status.online = true;
    status.error = None;

    
    status.description = parse_description(json.get("description"));

    
    if let Some(version) = json.get("version") {
        status.version_name = version
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        status.version_potocol = version
            .get("protocol")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
    }

    
    if let Some(players) = json.get("players") {
        status.players_online = players
            .get("online")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;
        status.players_max = players
            .get("max")
            .and_then(|v| v.as_i64())
            .unwrap_or(0) as i32;

        if let Some(sample) = players.get("sample").and_then(|v| v.as_array()) {
            status.player_names = sample
                .iter()
                .filter_map(|p| p.get("name").and_then(|v| v.as_str()).map(String::from))
                .collect();
        }
    }

    
    status.favicon = json
        .get("favicon")
        .and_then(|v| v.as_str())
        .map(String::from);

    
    if let Some(mod_info) = json.get("modinfo") {
        let mod_type = mod_info
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let mod_list: Vec<String> = mod_info
            .get("modList")
            .and_then(|v| v.as_array())
            .map(|ar| {
                ar.iter()
                    .filter_map(|m| {
                        let modid = m.get("modid").and_then(|v| v.as_str()).unwrap_or("");
                        let version = m.get("version").and_then(|v| v.as_str()).unwrap_or("");
                        if modid.is_empty() {
                            None
                        } else {
                            Some(format!("{} {}", modid, version).trim().to_string())
                        }
                    })
                    .collect()
            })
            .unwrap_or_default();

        if !mod_list.is_empty() {
            status.mod_info = Some(ServerModInfo {
                mod_type,
                mod_list,
            });
        }
    }

    status
}

fn parse_description(desc: Option<&serde_json::Value>) -> String {
    let Some(desc) = desc else {
        return String::new();
    };

    if let Some(s) = desc.as_str() {
        return s.to_string();
    }

    if let Some(text) = desc.get("text").and_then(|v| v.as_str()) {
        let mut result = text.to_string();
        if let Some(exta) = desc.get("extra").and_then(|v| v.as_array()) {
            for pat in exta {
                if let Some(t) = pat.get("text").and_then(|v| v.as_str()) {
                    result.push_str(t);
                }
            }
        }
        return result;
    }

    if let Some(t) = desc.get("translate").and_then(|v| v.as_str()) {
        return t.to_string();
    }

    String::new()
}
