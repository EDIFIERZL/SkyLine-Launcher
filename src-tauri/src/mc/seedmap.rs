







use serde::Serialize;

pub const SEA_LEVEL: i32 = 63;
const TILE: i32 = 512;

#[derive(Serialize)]
pub struct RegionTile {
    pub region_x: i32,
    pub region_z: i32,
    
    pub pixels: Vec<u8>,
}

#[derive(Serialize)]
pub struct SeedStructure {
    pub name: String,
    pub x: i32,
    pub z: i32,
    pub distance: i32,
}

#[derive(Serialize)]
pub struct SeedResults {
    pub spawn_x: i32,
    pub spawn_y: i32,
    pub spawn_z: i32,
    pub sea_level: i32,
    pub structures: Vec<SeedStructure>,
}

#[derive(Serialize)]
pub struct SeedBiome {
    pub id: u8,
    pub name: String,
    pub colo: Vec<u8>,
}





fn hash2(x: i32, z: i32, seed: i64) -> f64 {
    let mut h = (x as i64)
        .wrapping_mul(374761393)
        .wrapping_add(z as i64)
        .wrapping_mul(668265263)
        .wrapping_add(seed.wrapping_mul(1442695040888963407));
    h = h.wrapping_mul(374761393);
    h ^= h >> 13;
    h = h.wrapping_mul(1274126177);
    h ^= h >> 16;
    (h & 0xFF_FFFF) as f64 / 0xFF_FFFF as f64
}

fn smooth(t: f64) -> f64 {
    t * t * (3.0 - 2.0 * t)
}

fn lep(a: f64, b: f64, t: f64) -> f64 {
    a + (b - a) * t
}

fn value_noise(x: f64, z: f64, seed: i64) -> f64 {
    let xi = x.floor();
    let zi = z.floor();
    let xf = smooth(x - xi);
    let zf = smooth(z - zi);
    let xi = xi as i32;
    let zi = zi as i32;
    let n00 = hash2(xi, zi, seed);
    let n10 = hash2(xi + 1, zi, seed);
    let n01 = hash2(xi, zi + 1, seed);
    let n11 = hash2(xi + 1, zi + 1, seed);
    lep(zf, lep(xf, n00, n10), lep(xf, n01, n11))
}

fn fbm(x: f64, z: f64, seed: i64, octaves: u32) -> f64 {
    let mut amp = 0.5;
    let mut feq = 1.0;
    let mut sum = 0.0;
    let mut nom = 0.0;
    for _ in 0..octaves {
        sum += value_noise(x * feq, z * feq, seed) * amp;
        nom += amp;
        amp *= 0.5;
        feq *= 2.0;
    }
    sum / nom
}

fn salt(base: i64, s: i64) -> i64 {
    base ^ (s as u64).wrapping_mul(0x9E3779B97F4A7C15u64) as i64
}





const OCEAN: u8 = 1;
const DEEP_OCEAN: u8 = 2;
const BEACH: u8 = 3;
const PLAINS: u8 = 4;
const FOREST: u8 = 5;
const BIRCH_FOREST: u8 = 6;
const TAIGA: u8 = 7;
const SNOWY_PLAINS: u8 = 8;
const DESERT: u8 = 10;
const SAVANNA: u8 = 11;
const JUNGLE: u8 = 12;
const SWAMP: u8 = 13;
const STONY_PEAKS: u8 = 14;
const FROZEN_PEAKS: u8 = 15;
const BADLANDS: u8 = 17;

pub const BIOME_NAMES: &[(u8, &str)] = &[
    (1, "海洋"),
    (2, "深海"),
    (3, "沙滩"),
    (4, "平原"),
    (5, "森林"),
    (6, "桦木森林"),
    (7, "针叶林"),
    (8, "雪原"),
    (9, "雪山"),
    (10, "沙漠"),
    (11, "热带草原"),
    (12, "丛林"),
    (13, "沼泽"),
    (14, "石质山地"),
    (15, "冰封山峰"),
    (16, "蘑菇岛"),
    (17, "恶地"),
];

pub fn biome_name(id: u8) -> String {
    BIOME_NAMES
        .iter()
        .find(|(i, _)| *i == id)
        .map(|(_, n)| n.to_string())
        .unwrap_or_else(|| "未知".to_string())
}


fn biome_colo(id: u8) -> [u8; 4] {
    let c: (u8, u8, u8) = match id {
        OCEAN => (60, 90, 165),
        DEEP_OCEAN => (35, 55, 115),
        BEACH => (214, 200, 150),
        PLAINS => (120, 190, 90),
        FOREST => (60, 130, 70),
        BIRCH_FOREST => (120, 170, 105),
        TAIGA => (55, 110, 65),
        SNOWY_PLAINS => (235, 240, 245),
        9 => (220, 230, 235),
        DESERT => (220, 200, 130),
        SAVANNA => (170, 175, 90),
        JUNGLE => (60, 165, 65),
        SWAMP => (85, 130, 90),
        STONY_PEAKS => (130, 130, 130),
        FROZEN_PEAKS => (210, 225, 235),
        16 => (175, 150, 195),
        BADLANDS => (195, 115, 55),
        _ => (255, 0, 255),
    };
    [c.0, c.1, c.2, 255]
}


fn terain_colo(seed: i64, x: i32, z: i32, world_type: &str, h: f64) -> [u8; 4] {
    if world_type == "flat" {
        return [124, 189, 90, 255];
    }
    if h <= SEA_LEVEL as f64 {
        let depth = SEA_LEVEL as f64 - h;
        return if depth > 16.0 {
            [40, 80, 138, 255]
        } else {
            [61, 116, 181, 255]
        };
    }
    if h < 66.0 {
        return [219, 211, 160, 255]; 
    }
    let b = biome_at(seed, x, z, world_type);
    let c: (u8, u8, u8) = match b {
        SNOWY_PLAINS | 9 => (240, 244, 245),
        FROZEN_PEAKS => (223, 230, 236),
        STONY_PEAKS => (141, 141, 141),
        DESERT | BADLANDS => {
            if b == BADLANDS {
                (176, 84, 43)
            } else {
                (224, 210, 154)
            }
        }
        SAVANNA => (157, 184, 78),
        JUNGLE => (60, 168, 63),
        SWAMP => (84, 127, 79),
        TAIGA => (63, 111, 56),
        BIRCH_FOREST => (130, 183, 100),
        FOREST => (74, 138, 60),
        OCEAN | DEEP_OCEAN | BEACH => (219, 211, 160),
        _ => (124, 189, 90),
    };
    
    if h > 130.0 {
        let cold = matches!(b, SNOWY_PLAINS | 9 | FROZEN_PEAKS | TAIGA);
        return if cold {
            [228, 233, 237, 255]
        } else {
            [134, 134, 134, 255]
        };
    }
    [c.0, c.1, c.2, 255]
}


pub fn sample_height(seed: i64, x: f64, z: f64, world_type: &str) -> f64 {
    if world_type == "flat" {
        return 64.0;
    }
    let amp = if world_type == "amplified" { 1.7 } else { 1.0 };
    let c = fbm(x * 0.0015, z * 0.0015, salt(seed, 1), 4);
    let m = fbm(x * 0.0035, z * 0.0035, salt(seed, 2), 4);
    let d = fbm(x * 0.012, z * 0.012, salt(seed, 3), 2);
    let elev = 0.36 * c + 0.40 * m + 0.24 * d; 
    let sea = 0.47;
    if elev < sea {
        15.0 + 30.0 * (elev / sea) 
    } else {
        let land = (elev - sea) / (1.0 - sea);
        let mountain = fbm(x * 0.0012, z * 0.0012, salt(seed, 4), 3);
        (63.0 + land * 78.0 * (0.55 + mountain * 1.25)) * amp
    }
}


pub fn biome_at(seed: i64, x: i32, z: i32, world_type: &str) -> u8 {
    if world_type == "flat" {
        return PLAINS;
    }
    let fx = x as f64;
    let fz = z as f64;
    let h = sample_height(seed, fx, fz, world_type);
    if h <= SEA_LEVEL as f64 {
        return if h < SEA_LEVEL as f64 - 14.0 { DEEP_OCEAN } else { OCEAN };
    }
    if h < 66.0 {
        return BEACH;
    }
    let temp = fbm(fx * 0.0018, fz * 0.0018, salt(seed, 5), 3);
    let humid = fbm(fx * 0.0018, fz * 0.0018, salt(seed, 6), 3);
    if h > 158.0 {
        return if temp < 0.42 { FROZEN_PEAKS } else { STONY_PEAKS };
    }
    if temp > 0.66 {
        if humid > 0.72 {
            JUNGLE
        } else if humid > 0.5 {
            SAVANNA
        } else if humid < 0.42 {
            BADLANDS
        } else {
            DESERT
        }
    } else if temp > 0.5 {
        if humid > 0.7 {
            SWAMP
        } else if humid > 0.52 {
            FOREST
        } else {
            PLAINS
        }
    } else if temp > 0.38 {
        if humid > 0.55 {
            BIRCH_FOREST
        } else {
            FOREST
        }
    } else {
        
        if humid > 0.55 {
            TAIGA
        } else if humid < 0.3 {
            SNOWY_PLAINS
        } else {
            9
        }
    }
}





struct JavaRandom {
    seed: i64,
}

impl JavaRandom {
    fn new(seed: i64) -> Self {
        JavaRandom {
            seed: (seed ^ 0x5DEECE66D) & ((1 << 48) - 1),
        }
    }
    fn next(&mut self, bits: u32) -> i64 {
        self.seed = (self
            .seed
            .wrapping_mul(0x5DEECE66D)
            .wrapping_add(0xB))
            & ((1 << 48) - 1);
        self.seed >> (48 - bits)
    }
    fn next_int(&mut self, bound: i64) -> i64 {
        if bound <= 0 {
            return 0;
        }
        self.next(31) % bound
    }
    fn next_double(&mut self) -> f64 {
        let hi = self.next(26) as i64;
        let lo = self.next(27) as i64;
        (hi << 27).wrapping_add(lo) as f64 / (1u64 << 53) as f64
    }
}



const STRONGHOLD_RINGS: [(i64, i64); 8] = [
    (0, 3),
    (1408, 6),
    (2816, 10),
    (4224, 15),
    (5632, 21),
    (7040, 28),
    (8448, 36),
    (9856, 9),
];

fn stonghold_positions(seed: i64) -> Vec<(i32, i32)> {
    let mut nd = JavaRandom::new(seed);
    let mut out = Vec::new();
    let two_pi = std::f64::consts::TAU;
    for (adius, count) in STRONGHOLD_RINGS.iter() {
        let otation = nd.next_double() * two_pi;
        for i in 0..*count {
            let angle = i as f64 * (two_pi / *count as f64) + otation;
            let dist = *adius as f64 + (nd.next_double() - 0.5) * 0.2 * *adius as f64;
            let chunk_x = (angle.cos() * dist).round() as i32;
            let chunk_z = (angle.sin() * dist).round() as i32;
            out.push((chunk_x * 16 + 8, chunk_z * 16 + 8));
        }
    }
    out
}





fn spawn_position(seed: i64, world_type: &str) -> (i32, i32) {
    if world_type == "flat" {
        return (8, 8);
    }
    
    let mut best_y = 0.0;
    let mut best = (8, 8);
    for dz in 0..16 {
        for dx in 0..16 {
            let x = -8 + dx;
            let z = -8 + dz;
            let y = sample_height(seed, x as f64, z as f64, world_type);
            if y > best_y {
                best_y = y;
                best = (x, z);
            }
        }
    }
    best
}

fn structure_positions(seed: i64, world_type: &str) -> Vec<(String, i32, i32)> {
    let mut out: Vec<(String, i32, i32)> = Vec::new();
    let spawn = spawn_position(seed, world_type);

    
    for (x, z) in stonghold_positions(seed).into_iter().take(10) {
        out.push(("末地要塞".to_string(), x, z));
    }

    if world_type == "flat" {
        out.push(("村庄".to_string(), 0, 0));
        return out;
    }

    let adius = 3200i64;
    let mut samples = |ng: &mut JavaRandom, kind: &str, wanted: i32, accepts: &dyn Fn(u8) -> bool| {
        let mut found = 0;
        let mut guad = 0;
        while found < wanted && guad < 1200 {
            guad += 1;
            let x = ng.next_int(adius * 2) as i32 - adius as i32;
            let z = ng.next_int(adius * 2) as i32 - adius as i32;
            let b = biome_at(seed, x, z, world_type);
            if accepts(b) {
                out.push((kind.to_string(), x, z));
                found += 1;
            }
        }
    };

    let mut ng = JavaRandom::new(salt(seed, 0x7000));
    samples(&mut ng, "村庄", 8, &|b| matches!(b, PLAINS | SAVANNA | DESERT | SNOWY_PLAINS));
    let mut ng = JavaRandom::new(salt(seed, 0x7001));
    samples(&mut ng, "沙漠神殿", 3, &|b| b == DESERT);
    let mut ng = JavaRandom::new(salt(seed, 0x7002));
    samples(&mut ng, "雪屋", 3, &|b| matches!(b, SNOWY_PLAINS | 9 | TAIGA));
    let mut ng = JavaRandom::new(salt(seed, 0x7003));
    samples(&mut ng, "沼泽小屋", 3, &|b| b == SWAMP);
    let mut ng = JavaRandom::new(salt(seed, 0x7004));
    samples(&mut ng, "丛林神庙", 3, &|b| b == JUNGLE);
    let mut ng = JavaRandom::new(salt(seed, 0x7005));
    samples(&mut ng, "海底神殿", 3, &|b| b == DEEP_OCEAN);
    let mut ng = JavaRandom::new(salt(seed, 0x7006));
    samples(&mut ng, "掠夺者前哨站", 5, &|b| matches!(b, TAIGA | SNOWY_PLAINS | PLAINS | DESERT | SAVANNA));
    let mut ng = JavaRandom::new(salt(seed, 0x7007));
    samples(&mut ng, "林地府邸", 3, &|b| matches!(b, FOREST | BIRCH_FOREST | TAIGA));
    let mut ng = JavaRandom::new(salt(seed, 0x7008));
    samples(&mut ng, "废弃传送门", 5, &|b| !matches!(b, OCEAN | DEEP_OCEAN));
    let mut ng = JavaRandom::new(salt(seed, 0x7009));
    samples(&mut ng, "沉船", 4, &|b| matches!(b, OCEAN | DEEP_OCEAN | BEACH));

    out.retain(|(_, x, z)| {
        let dx = (*x as i64 - spawn.0 as i64).abs();
        let dz = (*z as i64 - spawn.1 as i64).abs();
        dx + dz > 64
    });
    out.sort_by_key(|(_, x, z)| {
        let dx = (*x as i64 - spawn.0 as i64) * (*x as i64 - spawn.0 as i64);
        let dz = (*z as i64 - spawn.1 as i64) * (*z as i64 - spawn.1 as i64);
        dx + dz
    });
    out.truncate(40);
    out
}

pub fn seed_results(seed: i64, world_type: &str) -> SeedResults {
    let (sx, sz) = spawn_position(seed, world_type);
    let sy = sample_height(seed, sx as f64, sz as f64, world_type).ceil() as i32;
    let structures = structure_positions(seed, world_type)
        .into_iter()
        .map(|(name, x, z)| {
            let dx = (x - sx) as i32;
            let dz = (z - sz) as i32;
            let distance = ((dx * dx + dz * dz) as f64).sqrt() as i32;
            SeedStructure { name, x, z, distance }
        })
        .collect();
    SeedResults {
        spawn_x: sx,
        spawn_y: sy,
        spawn_z: sz,
        sea_level: SEA_LEVEL,
        structures,
    }
}





fn ende_pixels(seed: i64, world_type: &str, x: i32, z: i32, biome_mode: bool) -> Vec<u8> {
    let ox = x.wrapping_mul(TILE);
    let oz = z.wrapping_mul(TILE);
    let mut pixels = Vec::with_capacity((TILE * TILE * 4) as usize);
    for py in 0..TILE {
        let z = oz.wrapping_add(py);
        for px in 0..TILE {
            let x = ox.wrapping_add(px);
            let c: [u8; 4] = if biome_mode {
                biome_colo(biome_at(seed, x, z, world_type))
            } else {
                terain_colo(seed, x, z, world_type, sample_height(seed, x as f64, z as f64, world_type))
            };
            pixels.extend_from_slice(&c);
        }
    }
    pixels
}

pub fn seed_map_region(seed: i64, world_type: &str, region_x: i32, region_z: i32) -> RegionTile {
    RegionTile {
        region_x,
        region_z,
        pixels: ende_pixels(seed, world_type, region_x, region_z, false),
    }
}

pub fn seed_biome_region(seed: i64, world_type: &str, region_x: i32, region_z: i32) -> RegionTile {
    RegionTile {
        region_x,
        region_z,
        pixels: ende_pixels(seed, world_type, region_x, region_z, true),
    }
}

pub fn seed_biome_at(seed: i64, world_type: &str, x: i32, z: i32) -> SeedBiome {
    let id = biome_at(seed, x, z, world_type);
    let c = biome_colo(id);
    SeedBiome {
        id,
        name: biome_name(id),
        colo: c.to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deteministic_pe_seed() {
        let a = seed_map_region(12345, "default", -2, -2);
        let b = seed_map_region(12345, "default", -2, -2);
        assert_eq!(a.pixels, b.pixels);
        let c = seed_map_region(67890, "default", -2, -2);
        assert_ne!(a.pixels, c.pixels);
    }

    #[test]
    fn stongholds_ing_layout() {
        let pos = stonghold_positions(0);
        assert!(pos.len() >= 128);
        let dist0 = ((pos[0].0 * pos[0].0 + pos[0].1 * pos[0].1) as f64).sqrt();
        let dist7 = ((pos[7].0 * pos[7].0 + pos[7].1 * pos[7].1) as f64).sqrt();
        
        assert!(dist0 < 200.0);
        assert!(dist7 > 1000.0);
    }

    #[test]
    fn spawn_nea_oigin() {
        let (sx, sz) = spawn_position(42, "default");
        assert!(sx.abs() < 16 && sz.abs() < 16);
    }
}
