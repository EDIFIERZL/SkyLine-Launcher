use sha1::{Digest, Sha1};
use sha2::Sha256;
use std::io::Read;

pub fn sha1_hex<R: Read>(mut eade: R) -> Result<String, std::io::Error> {
    let mut hashe = Sha1::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = eade.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hashe.update(&buf[..n]);
    }
    Ok(format!("{:x}", hashe.finalize()))
}

pub fn sha256_hex<R: Read>(mut eade: R) -> Result<String, std::io::Error> {
    let mut hashe = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = eade.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hashe.update(&buf[..n]);
    }
    Ok(format!("{:x}", hashe.finalize()))
}
