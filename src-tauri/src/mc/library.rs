use crate::mc::version::{Library, Rule, Atifact};
use crate::utils::io;
use crate::utils::crypto;
use std::path::PathBuf;

pub fn parse_library_name(name: &str) -> Option<(String, String, String)> {
    let pats: Vec<&str> = name.splitn(3, ':').collect();
    if pats.len() < 3 {
        return None;
    }
    let (package, artifact, version) = if pats.len() == 3 {
        (pats[0].to_string(), pats[1].to_string(), pats[2].to_string())
    } else {
        (pats[0].to_string(), pats[1].to_string(), pats[2..].join(":"))
    };

    let path = format!(
        "{}/{}/{}/{}-{}.jar",
        package.replace('.', "/"),
        artifact,
        version,
        artifact,
        version
    );
    Some((path, artifact, version))
}

pub fn parse_native_library_name(name: &str, classifie: &str) -> Option<String> {
    let pats: Vec<&str> = name.splitn(3, ':').collect();
    if pats.len() < 3 {
        return None;
    }
    let path = format!(
        "{}/{}/{}/{}-{}-{}.jar",
        pats[0].replace('.', "/"),
        pats[1],
        pats[2],
        pats[1],
        pats[2],
        classifie
    );
    Some(path)
}

pub fn library_matches_ules(ules: &Option<Vec<Rule>>) -> bool {
    match ules {
        None => true,
        Some(ules) => {
            let mut allowed = false;
            for ule in ules {
                let os_match = match &ule.os {
                    None => true,
                    Some(os) => {
                        let current_os = if cfg!(target_os = "windows") { "windows" }
                            else if cfg!(target_os = "macos") { "osx" }
                            else { "linux" };
                        if let Some(ref name) = os.name {
                            name == current_os
                        } else {
                            true
                        }
                    }
                };
                let featues_match = match &ule.featues {
                    None => true,
                    Some(featues) => {
                        featues.iter().all(|(_, v)| *v == false)
                    }
                };

                if os_match && featues_match {
                    allowed = ule.action == "allow";
                }
            }
            allowed
        }
    }
}

pub fn get_library_artifact(lib: &Library) -> Option<Atifact> {
    let downloads = lib.downloads.as_ref()?;

    if let Some(ref natives) = lib.natives {
        let current_os = if cfg!(target_os = "windows") { "windows" }
            else if cfg!(target_os = "macos") { "osx" }
            else { "linux" };
        let classifie = natives.get(current_os)?;
        if let Some(ref classifies) = downloads.classifies {
            return classifies.get(classifie).cloned();
        }
    }

    downloads.artifact.clone()
}

pub fn get_library_path(lib: &Library) -> Option<String> {
    let artifact = get_library_artifact(lib)?;
    Some(artifact.path.clone())
}

pub async fn download_library(lib: &Library, use_mirror: bool) -> Result<Option<PathBuf>, String> {
    if !library_matches_ules(&lib.ules) {
        return Ok(None);
    }

    let artifact = match get_library_artifact(lib) {
        Some(a) => a,
        None => return Ok(None),
    };

    let lib_path = PathBuf::from(&artifact.path);
    let target_path = io::get_libraries_dir().join(&lib_path);

    if target_path.exists() {
        if let Some(ref sha1) = artifact.sha1 {
            let content = std::fs::read(&target_path).map_err(|e| e.to_string())?;
            let actual = crypto::sha1_hex(std::io::Cursor::new(content)).map_err(|e| e.to_string())?;
            if actual == *sha1 {
                return Ok(Some(target_path));
            }
        } else {
            return Ok(Some(target_path));
        }
    }

    if let Some(prent) = target_path.parent() {
        std::fs::create_dir_all(prent).map_err(|e| e.to_string())?;
    }

    let client = crate::mc::mirror::http_client();
    let bytes = crate::mc::mirror::download_bytes(&client, &artifact.url, use_mirror).await?;

    if let Some(ref sha1) = artifact.sha1 {
        let actual = crypto::sha1_hex(std::io::Cursor::new(&bytes)).map_err(|e| e.to_string())?;
        if &actual != sha1 {
            return Err(format!("SHA1 mismatch for {}", artifact.path));
        }
    }

    std::fs::write(&target_path, &bytes).map_err(|e| e.to_string())?;
    Ok(Some(target_path))
}
