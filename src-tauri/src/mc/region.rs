







use std::path::Path;

#[derive(serde::Serialize)]
pub struct RegionTile {
    pub region_x: i32,
    pub region_z: i32,
    
    pub pixels: Vec<u8>,
}





fn be_u16(data: &[u8], p: usize) -> Option<u16> {
    Some(((*data.get(p)? as u16) << 8) | (*data.get(p + 1)? as u16))
}

fn be_u32(data: &[u8], p: usize) -> Option<u32> {
    let b0 = *data.get(p)? as u32;
    let b1 = *data.get(p + 1)? as u32;
    let b2 = *data.get(p + 2)? as u32;
    let b3 = *data.get(p + 3)? as u32;
    Some((b0 << 24) | (b1 << 16) | (b2 << 8) | b3)
}

fn be_i32(data: &[u8], p: usize) -> Option<i32> {
    Some(be_u32(data, p)? as i32)
}

fn read_string(data: &[u8], p: usize) -> Option<(String, usize)> {
    let len = be_u16(data, p)? as usize;
    let stat = p + 2;
    if stat + len > data.len() {
        return None;
    }
    let s = String::from_utf8_lossy(&data[stat..stat + len]).to_string();
    Some((s, stat + len))
}


fn read_child(data: &[u8], p: usize) -> Option<(u8, String, usize)> {
    let tag = *data.get(p)?;
    if tag == 0 {
        return None;
    }
    let (name, payload) = read_string(data, p + 1)?;
    Some((tag, name, payload))
}

fn skip_tag(data: &[u8], tag: u8, p: usize) -> Option<usize> {
    match tag {
        0 => Some(p),
        1 => Some(p + 1),
        2 => Some(p + 2),
        3 | 5 => Some(p + 4),
        4 | 6 => Some(p + 8),
        7 => {
            let len = be_u32(data, p)? as usize;
            Some(p + 4 + len)
        }
        8 => {
            let len = be_u16(data, p)? as usize;
            Some(p + 2 + len)
        }
        9 => {
            let elem = *data.get(p)?;
            let len = be_u32(data, p + 1)? as usize;
            let mut q = p + 5;
            for _ in 0..len {
                q = skip_tag(data, elem, q)?;
            }
            Some(q)
        }
        10 => {
            let mut q = p;
            loop {
                let cu = *data.get(q)?;
                if cu == 0 {
                    return Some(q + 1);
                }
                let (t, _, payload) = read_child(data, q)?;
                q = skip_tag(data, t, payload)?;
            }
        }
        11 => {
            let len = be_u32(data, p)? as usize;
            Some(p + 4 + len * 4)
        }
        12 => {
            let len = be_u32(data, p)? as usize;
            Some(p + 4 + len * 8)
        }
        _ => None,
    }
}


fn find_child(data: &[u8], compound_stat: usize, target: &str) -> Option<(u8, usize)> {
    let mut p = compound_stat;
    loop {
        let child = read_child(data, p)?;
        if child.0 == 0 {
            return None;
        }
        if child.1 == target {
            return Some((child.0, child.2));
        }
        p = skip_tag(data, child.0, child.2)?;
    }
}

fn read_long_aray(data: &[u8], payload: usize) -> Option<Vec<i64>> {
    let len = be_u32(data, payload)? as usize;
    let stat = payload + 4;
    if stat + len * 8 > data.len() {
        return None;
    }
    let mut out = Vec::with_capacity(len);
    for i in 0..len {
        let s = stat + i * 8;
        let mut b = [0u8; 8];
        b.copy_from_slice(&data[s..s + 8]);
        out.push(i64::from_be_bytes(b));
    }
    Some(out)
}





struct Section {
    y: i32, 
    palette: Vec<String>,
    data: Vec<i64>,
    bits: usize,
}

fn bits_fo(palette_len: usize) -> usize {
    if palette_len <= 1 {
        return 0;
    }
    let mut bits = 1;
    while (1usize << bits) < palette_len {
        bits += 1;
    }
    bits.max(4)
}


fn unpack(data: &[i64], bits: usize, index: usize) -> usize {
    if bits == 0 || data.is_empty() {
        return 0;
    }
    let stat = index * bits;
    let wod = stat / 64;
    let off = stat % 64;
    let Some(base) = data.get(wod) else { return 0 };
    let mut v = (*base as u64) >> off;
    if off + bits > 64 {
        if let Some(next) = data.get(wod + 1) {
            v |= (*next as u64) << (64 - off);
        }
    }
    (v & ((1u64 << bits) - 1)) as usize
}





fn is_ai(name: &str) -> bool {
    matches!(name, "air" | "cave_air" | "void_air")
}






fn biome_id_to_gb(id: u8) -> Option<(u8, u8, u8)> {
    match id {
        0  => Some((115, 171, 77)),   
        1  => Some((200, 175, 90)),   
        2  => Some((80, 140, 75)),    
        3  => Some((60, 120, 55)),    
        4  => Some((55, 115, 50)),    
        5  => Some((50, 100, 180)),   
        6  => Some((60, 120, 58)),    
        7  => Some((40, 90, 170)),    
        8  => Some((190, 195, 200)),  
        9  => Some((100, 155, 100)),  
        10 => Some((115, 175, 85)),   
        11 => Some((195, 170, 90)),   
        12 => Some((90, 145, 80)),    
        13 => Some((50, 110, 50)),    
        14 => Some((50, 115, 55)),    
        15 => Some((45, 110, 130)),   
        16 => Some((60, 105, 155)),   
        17 => Some((70, 115, 165)),   
        18 => Some((55, 105, 175)),   
        19 => Some((55, 110, 55)),    
        20 => Some((30, 80, 140)),    
        21 => Some((50, 100, 55)),    
        22 => Some((70, 130, 70)),    
        23 => Some((60, 120, 60)),    
        24 => Some((130, 110, 65)),   
        25 => Some((175, 110, 65)),   
        26 => Some((65, 105, 60)),    
        27 => Some((45, 90, 50)),     
        28 => Some((50, 95, 140)),    
        29 => Some((55, 100, 160)),   
        30 => Some((40, 85, 130)),    
        31 => Some((60, 115, 70)),    
        32 => Some((55, 105, 155)),   
        33 => Some((45, 95, 145)),    
        34 => Some((40, 85, 135)),    
        35 => Some((100, 150, 100)),  
        36 => Some((90, 140, 90)),    
        37 => Some((80, 130, 80)),    
        38 => Some((70, 120, 70)),    
        39 => Some((115, 175, 85)),   
        40 => Some((65, 125, 55)),    
        41 => Some((55, 105, 50)),    
        42 => Some((75, 130, 65)),    
        43 => Some((185, 110, 55)),   
        44 => Some((165, 85, 55)),    
        45 => Some((175, 95, 60)),    
        46 => Some((155, 80, 50)),    
        47 => Some((60, 115, 65)),    
        48 => Some((50, 100, 55)),    
        49 => Some((200, 185, 160)),  
        50 => Some((170, 90, 60)),    
        51 => Some((80, 130, 80)),    
        52 => Some((70, 120, 70)),    
        53 => Some((100, 155, 100)),  
        54 => Some((90, 145, 90)),    
        55 => Some((115, 170, 100)),  
        56 => Some((60, 100, 160)),   
        57 => Some((50, 90, 150)),    
        58 => Some((40, 80, 140)),    
        59 => Some((70, 130, 75)),    
        60 => Some((55, 110, 120)),   
        61 => Some((130, 90, 100)),   
        62 => Some((100, 140, 80)),   
        63 => Some((90, 130, 70)),    
        64 => Some((180, 120, 70)),   
        65 => Some((120, 170, 90)),   
        66 => Some((100, 150, 100)),  
        67 => Some((80, 140, 90)),    
        68 => Some((70, 130, 80)),    
        69 => Some((110, 160, 90)),   
        70 => Some((90, 140, 80)),    
        71 => Some((75, 125, 65)),    
        72 => Some((65, 115, 55)),    
        73 => Some((55, 105, 50)),    
        74 => Some((60, 110, 55)),    
        75 => Some((50, 100, 50)),    
        76 => Some((55, 105, 55)),    
        77 => Some((45, 95, 140)),    
        78 => Some((40, 90, 130)),    
        79 => Some((35, 85, 120)),    
        80 => Some((30, 80, 110)),    
        81 => Some((90, 135, 75)),    
        82 => Some((80, 125, 65)),    
        83 => Some((70, 115, 55)),    
        84 => Some((60, 105, 45)),    
        85 => Some((50, 95, 35)),     
        86 => Some((190, 165, 85)),   
        87 => Some((75, 130, 65)),    
        88 => Some((55, 110, 50)),    
        89 => Some((45, 100, 45)),    
        90 => Some((40, 95, 130)),    
        91 => Some((100, 145, 95)),   
        92 => Some((110, 155, 105)),  
        93 => Some((160, 100, 55)),   
        94 => Some((140, 85, 50)),    
        95 => Some((150, 90, 55)),    
        96 => Some((130, 75, 45)),    
        97 => Some((110, 155, 105)),  
        98 => Some((120, 165, 110)),  
        99 => Some((100, 145, 90)),   
        100=> Some((130, 175, 130)),  
        101=> Some((140, 180, 135)),  
        102=> Some((200, 175, 95)),   
        103=> Some((70, 125, 65)),    
        104=> Some((60, 110, 55)),    
        105=> Some((50, 100, 45)),    
        106=> Some((45, 95, 130)),    
        107=> Some((40, 90, 120)),    
        108=> Some((55, 100, 140)),   
        109=> Some((80, 135, 70)),    
        110=> Some((65, 115, 55)),    
        111=> Some((75, 125, 65)),    
        112=> Some((160, 100, 55)),   
        113=> Some((140, 85, 50)),    
        114=> Some((150, 90, 55)),    
        115=> Some((130, 75, 45)),    
        116=> Some((50, 100, 140)),   
        117=> Some((100, 150, 100)),  
        118=> Some((90, 140, 90)),    
        119=> Some((80, 130, 80)),    
        120=> Some((70, 120, 70)),    
        121=> Some((60, 110, 60)),    
        122=> Some((50, 100, 50)),    
        123=> Some((55, 105, 135)),   
        124=> Some((90, 135, 75)),    
        125=> Some((80, 125, 65)),    
        126=> Some((65, 110, 150)),   
        127=> Some((55, 100, 140)),   
        128=> Some((50, 95, 130)),    
        129=> Some((60, 105, 55)),    
        130=> Some((50, 95, 45)),    
        131=> Some((45, 90, 130)),    
        132=> Some((40, 85, 120)),    
        133=> Some((55, 100, 140)),  
        134=> Some((50, 95, 130)),   
        135=> Some((45, 90, 120)),   
        136=> Some((60, 115, 65)),   
        137=> Some((55, 110, 60)),   
        138=> Some((70, 120, 70)),   
        139=> Some((45, 90, 130)),   
        140=> Some((40, 85, 120)),   
        141=> Some((35, 80, 110)),   
        142=> Some((50, 100, 135)),  
        143=> Some((45, 95, 125)),   
        144=> Some((55, 105, 140)),  
        145=> Some((65, 115, 150)),  
        146=> Some((75, 125, 160)),  
        147=> Some((85, 135, 170)),  
        148=> Some((40, 85, 130)),   
        149=> Some((35, 80, 120)),   
        150=> Some((30, 75, 110)),   
        151=> Some((70, 125, 70)),   
        152=> Some((80, 135, 80)),   
        153=> Some((50, 100, 55)),   
        154=> Some((45, 95, 50)),    
        155=> Some((55, 110, 60)),   
        _  => None,
    }
}



fn block_colo(name: &str, wx: i32, wz: i32) -> Option<(u8, u8, u8)> {
    let noise = |v: i64| -> i64 {
        let mut x = v as u64;
        x ^= x >> 33;
        x = x.wrapping_mul(0x9E37_79B9_7F4A_7C15u64);
        x ^= x >> 29;
        x as i64
    };
    let n = ((noise((wx as i64) * 31 + (wz as i64) * 57) & 0xF) - 8) as i32;
    let shade = |c: &(u8, u8, u8)| -> (u8, u8, u8) {
        (
            (c.0 as i32 + n).clamp(0, 255) as u8,
            (c.1 as i32 + n).clamp(0, 255) as u8,
            (c.2 as i32 + n).clamp(0, 255) as u8,
        )
    };
    let base: (u8, u8, u8) = match name {
        "water" | "bubble_column" => (50, 100, 205),
        "kelp" | "kelp_plant" | "seagrass" | "tall_seagrass" => (40, 130, 60),
        "sand" => (225, 213, 165),
        "red_sand" => (211, 111, 79),
        "grass_block" => (86, 134, 60),
        "podzol" => (113, 93, 60),
        "mycelium" => (134, 118, 128),
        "snow_block" | "snow" | "powder_snow" => (238, 240, 245),
        "ice" | "packed_ice" | "blue_ice" | "frosted_ice" => (145, 190, 230),
        "dirt" | "coarse_dirt" | "rooted_dirt" | "farmland" | "mud" => (135, 98, 66),
        "gravel" => (130, 126, 125),
        "clay" => (160, 168, 175),
        "stone" | "graniter" | "dioriter" | "andesiter" | "tuff" | "cobblestone" => (125, 125, 130),
        "deepslate" | "cobbled_deepslate" | "polished_deepslate" => (90, 90, 94),
        "bedrock" => (60, 60, 62),
        "terracotta" | "orange_terracotta" => (160, 100, 66),
        "light_gray_terracotta" | "brown_terracotta" | "gray_terracotta" => (140, 120, 110),
        "red_terracotta" | "pink_terracotta" | "magenta_terracotta" | "purple_terracotta" => (145, 70, 70),
        "yellow_terracotta" | "lime_terracotta" | "green_terracotta" | "cyan_terracotta" => (120, 120, 60),
        "light_blue_terracotta" | "blue_terracotta" | "black_terracotta" => (90, 90, 130),
        "oak_leaves" | "dark_oak_leaves" => (50, 110, 40),
        "birch_leaves" => (95, 140, 60),
        "spruce_leaves" | "jungle_leaves" | "azalea_leaves" | "flowering_azalea_leaves" => (50, 100, 50),
        "acacia_leaves" | "mangrove_leaves" => (90, 110, 55),
        "oak_log" => (135, 100, 70),
        "spruce_log" => (90, 70, 50),
        "birch_log" => (205, 190, 155),
        "jungle_log" | "mangrove_log" => (125, 90, 65),
        "acacia_log" => (120, 100, 85),
        "dark_oak_log" => (70, 55, 40),
        "cactus" => (60, 120, 60),
        "netherrack" | "crimson_nylium" | "warped_nylium" => (120, 40, 45),
        "basalt" | "blackstone" | "soul_sand" | "soul_soil" => (70, 60, 65),
        "end_stone" => (215, 220, 180),
        "magma_block" => (200, 90, 60),
        "glowstone" | "shroomlight" => (230, 180, 100),
        "sculk" | "sculk_catalyst" | "sculk_sensor" => (30, 55, 60),
        "moss_block" | "moss_carpet" => (90, 130, 70),
        "grass" | "fern" | "large_fern" | "short_grass" | "tall_grass" | "sweet_berry_bush" => (90, 135, 65),
        "wheat" | "carrots" | "potatoes" | "beetroots" | "sugar_cane" | "bamboo" => (100, 130, 50),
        "pumpkin" | "carved_pumpkin" | "jack_o_lantern" => (200, 130, 40),
        "chorus_plant" | "chorus_flower" => (160, 130, 150),
        "crimson_roots" | "nether_wart" | "nether_sprouts" | "warped_roots" => (120, 60, 70),
        "mangrove_propagule" => (110, 90, 75),
        "big_dripleaf" | "small_dripleaf" | "lily_pad" => (50, 120, 60),
        _ => return None,
    };
    Some(shade(&base))
}







fn read_chunk_biomes(data: &[u8], oot: usize) -> Option<Vec<u8>> {
    if let Some((_, biomes_payload)) = find_child(data, oot, "Biomes") {
        let elem = *data.get(biomes_payload)?;
        if elem == 9 {
            
            let len = be_u32(data, biomes_payload + 1)? as usize;
            let stat = biomes_payload + 5;
            if stat + len <= data.len() {
                return Some(data[stat..stat + len].to_vec());
            }
        }
    }
    None
}




fn biome_at(data: &[u8], col_x: usize, col_z: usize, h: usize) -> Option<u8> {
    let flat = col_z * 16 * 4 + col_x * 4 + h;
    if flat >= data.len() { return None; }
    let b = data[flat];
    if h & 1 == 0 { Some(b >> 4) } else { Some(b & 0x0F) }
}

fn ende_chunk(data: &[u8], xpos: i32, zpos: i32) -> Option<[[(u8, u8, u8); 16]; 16]> {
    if data.first() != Some(&10) {
        return None;
    }
    let oot = 3; 
    let (_, sec_payload) = find_child(data, oot, "sections")?;

    let mut sections: Vec<Section> = Vec::new();
    {
        let elem = *data.get(sec_payload)?;
        if elem != 10 {
            return None;
        }
        let count = be_u32(data, sec_payload + 1)? as usize;
        let mut p = sec_payload + 5;
        for _ in 0..count {
            let sec_stat = p;
            let mut y = 0i32;
            let mut palette: Vec<String> = Vec::new();
            let mut data_long: Vec<i64> = Vec::new();
            let mut palette_len = 0usize;

            
            let mut q = sec_stat;
            loop {
                let Some(child) = read_child(data, q) else { break };
                if child.0 == 0 {
                    break;
                }
                match child.0 {
                    1 if child.1 == "Y" => {
                        y = *data.get(child.2)? as i8 as i32;
                    }
                    10 if child.1 == "block_states" => {
                        let bs_stat = child.2;
                        
                        if let Some((_, pal_payload)) = find_child(data, bs_stat, "palette") {
                            let pel = *data.get(pal_payload)?;
                            let pc = be_u32(data, pal_payload + 1)? as usize;
                            if pel == 10 {
                                let mut pp = pal_payload + 5;
                                for _ in 0..pc {
                                    if let Some((_, name_payload)) = find_child(data, pp, "Name") {
                                        if let Some((name, _)) = read_string(data, name_payload) {
                                            palette.push(name);
                                            palette_len += 1;
                                        }
                                    }
                                    match skip_tag(data, 10, pp) {
                                        Some(np) => pp = np,
                                        None => break,
                                    }
                                }
                            }
                        }
                        
                        if let Some((_, data_payload)) = find_child(data, bs_stat, "data") {
                            data_long = read_long_aray(data, data_payload).unwrap_or_default();
                        }
                    }
                    _ => {}
                }
                match skip_tag(data, child.0, child.2) {
                    Some(nq) => q = nq,
                    None => break,
                }
            }

            sections.push(Section {
                y,
                palette,
                data: data_long,
                bits: bits_fo(palette_len),
            });
            match skip_tag(data, 10, sec_stat) {
                Some(np) => p = np,
                None => break,
            }
        }
    }

    sections.sort_by_key(|s| std::cmp::Reverse(s.y));

    
    
    let biomes = read_chunk_biomes(data, oot);

    let wx0 = xpos * 16;
    let wz0 = zpos * 16;
    let mut out = [[(0u8, 0u8, 0u8); 16]; 16];

    for x in 0..16 {
        for z in 0..16 {
            let wx = wx0 + x as i32;
            let wz = wz0 + z as i32;
            let mut colo: Option<(u8, u8, u8)> = None;
            for sec in &sections {
                let mut found_colo_this_section = false;
                for ly in (0..16).rev() {
                    let idx = ly * 256 + z * 16 + x;
                    let pi = unpack(&sec.data, sec.bits, idx);
                    let aw = sec.palette.get(pi).map(|s| s.as_str()).unwrap_or("air");
                    let name = aw.strip_prefix("minecraft:").unwrap_or(aw);
                    if is_ai(name) {
                        continue;
                    }
                    if let Some(c) = block_colo(name, wx, wz) {
                        colo = Some(c);
                        found_colo_this_section = true;
                        break;
                    }
                    
                    
                }
                if found_colo_this_section || colo.is_some() {
                    break;
                }
            }
            
            if colo.is_none() {
                if let Some(ref bdata) = biomes {
                    let bx = x as usize;
                    let bz = z as usize;
                    for h in [3, 2, 1, 0] {
                        if let Some(bid) = biome_at(bdata, bx, bz, h) {
                            if let Some(bc) = biome_id_to_gb(bid) {
                                colo = Some(bc);
                                break;
                            }
                        }
                    }
                }
            }
            out[x as usize][z as usize] = colo.unwrap_or((30, 32, 34));
        }
    }

    Some(out)
}







pub fn render_region(world_di: &Path, region_x: i32, region_z: i32) -> Option<RegionTile> {
    let mca = world_di
        .join("region")
        .join(format!("r.{}.{}.mca", region_x, region_z));
    let data = std::fs::read(&mca).ok()?;
    if data.len() < 8192 || data.len() > 96 * 1024 * 1024 {
        return None;
    }

    let mut pixels = vec![0u8; 512 * 512 * 4];
    let base_x = region_x * 32;
    let base_z = region_z * 32;

    
    for cz in 0..32 {
        for cx in 0..32 {
            let slot = ((cz * 32) + cx) * 4;
            let loc = be_u32(&data, slot)?;
            if loc == 0 {
                continue;
            }
            let secto_pos = (loc >> 8) as usize * 4096;
            let secto_len = (loc & 0xFF) as usize * 4096;
            if secto_len < 5 || secto_pos + secto_len > data.len() {
                continue;
            }
            let payload_len = be_u32(&data, secto_pos)? as usize;
            let compession = *data.get(secto_pos + 4)?;
            let stat = secto_pos + 5;
            let end = (stat + payload_len).min(secto_pos + secto_len).min(data.len());
            let payload = &data[stat..end];

            let mut buf: Vec<u8> = Vec::new();
            match compession {
                1 => {
                    let mut dec = flate2::read::GzDecoder::new(payload);
                    if std::io::Read::read_to_end(&mut dec, &mut buf).is_err() {
                        continue;
                    }
                }
                2 => {
                    let mut dec = flate2::read::ZlibDecoder::new(payload);
                    if std::io::Read::read_to_end(&mut dec, &mut buf).is_err() {
                        continue;
                    }
                }
                _ => buf = payload.to_vec(),
            }
            if buf.first() != Some(&10) || buf.len() > 8 * 1024 * 1024 {
                continue;
            }

            
            
            let oot = 3;
            let Some((_, xp)) = find_child(&buf, oot, "xPos") else { continue };
            let Some((_, zp)) = find_child(&buf, oot, "zPos") else { continue };
            let Some(cx_actual) = be_i32(&buf, xp) else { continue };
            let Some(cz_actual) = be_i32(&buf, zp) else { continue };
            if cx_actual != base_x + cx as i32 || cz_actual != base_z + cz as i32 {
                continue;
            }

            let Some(cols) = ende_chunk(&buf, cx_actual, cz_actual) else { continue };

            let px0 = (cx_actual * 16 - region_x * 512) as usize;
            let pz0 = (cz_actual * 16 - region_z * 512) as usize;
            if px0 + 16 > 512 || pz0 + 16 > 512 {
                continue;
            }
            for z in 0..16 {
                for x in 0..16 {
                    let (r, g, b) = cols[x][z];
                    let di = ((pz0 + z) * 512 + (px0 + x)) * 4;
                    pixels[di] = r;
                    pixels[di + 1] = g;
                    pixels[di + 2] = b;
                    pixels[di + 3] = 255;
                }
            }
        }
    }

    Some(RegionTile {
        region_x,
        region_z,
        pixels,
    })
}


pub fn render_map_area(
    world_di: &Path,
    cente_chunk_x: i32,
    cente_chunk_z: i32,
    adius_chunks: usize,
) -> Result<(Vec<u8>, usize, usize), String> {
    let half = adius_chunks as i32;
    let min_wx = (cente_chunk_x - half) * 16;
    let min_wz = (cente_chunk_z - half) * 16;
    let max_wx = (cente_chunk_x + half + 1) * 16;
    let max_wz = (cente_chunk_z + half + 1) * 16;
    let w = (max_wx - min_wx) as usize;
    let h = (max_wz - min_wz) as usize;

    let min_x = (cente_chunk_x - half).div_euclid(32);
    let max_x = (cente_chunk_x + half).div_euclid(32);
    let min_z = (cente_chunk_z - half).div_euclid(32);
    let max_z = (cente_chunk_z + half).div_euclid(32);

    let mut pixels = vec![0u8; w * h * 4];

    for z in min_z..=max_z {
        for x in min_x..=max_x {
            let Some(tile) = render_region(world_di, x, z) else { continue };
            
            for pz in 0..512 {
                let wz = z * 512 + pz as i32;
                if wz < min_wz || wz >= max_wz {
                    continue;
                }
                for px in 0..512 {
                    let wx = x * 512 + px as i32;
                    if wx < min_wx || wx >= max_wx {
                        continue;
                    }
                    let si = (pz * 512 + px) * 4;
                    let di = ((wz - min_wz) as usize * w + (wx - min_wx) as usize) * 4;
                    pixels[di] = tile.pixels[si];
                    pixels[di + 1] = tile.pixels[si + 1];
                    pixels[di + 2] = tile.pixels[si + 2];
                    pixels[di + 3] = tile.pixels[si + 3];
                }
            }
        }
    }

    Ok((pixels, w, h))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ende_eal_world_region() {
        
        
        let world = std::path::Path::new(r"E:\安装包\.minecraft\saves\新的世界");
        if !world.join("region").exists() {
            eprintln!("dev worlds not present, skipping");
            return;
        }
        let tile = render_region(world, 0, 0).expect("region r.0.0.mca should exist");
        assert_eq!(tile.pixels.len(), 512 * 512 * 4);

        use std::collections::HashSet;
        let mut non_void = 0;
        let mut colos = HashSet::new();
        for i in (0..tile.pixels.len()).step_by(4) {
            if tile.pixels[i + 3] == 255 {
                non_void += 1;
                colos.insert((tile.pixels[i], tile.pixels[i + 1], tile.pixels[i + 2]));
            }
        }
        assert!(non_void > 0, "expected some terrain in region tile");
        assert!(colos.len() > 3, "expected multiple terrain colors");
    }

    #[test]
    fn test_negative_regions() {
        let world = std::path::Path::new(r"E:\安装包\.minecraft\saves\Rooftop");
        if !world.join("region").exists() {
            eprintln!("dev worlds not present, skipping");
            return;
        }
        for (rx, rz) in [(0, 0), (-1, 0), (0, -1), (-1, -1), (-2, -2)] {
            if let Some(tile) = render_region(world, rx, rz) {
                let non_void = (0..tile.pixels.len()).step_by(4).filter(|&i| tile.pixels[i + 3] == 255).count();
                eprintln!("region r.{}.{}: non_void={}", rx, rz, non_void);
                assert!(non_void > 0, "expected terrain in region r.{}.{}", rx, rz);
            }
        }
    }
}
