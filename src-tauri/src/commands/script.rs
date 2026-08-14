use crate::mc::script::{self, SciptType, SciptConfig, GeneatedScipt};
use std::path::PathBuf;

#[tauri::command]
pub async fn expot_launch_script(
    instance_id: String,
    script_type: String,
    output_path: String,
) -> Result<GeneatedScipt, String> {
    let instance = crate::instance::manager::get_instance(&instance_id)?
        .ok_or_else(|| "Instance not found".to_string())?;

    let launch_di = crate::instance::manager::get_instance_launch_dir(&instance);
    let game_di = if let Some(ref override_di) = instance.game_dir_override {
        PathBuf::from(override_di)
    } else {
        launch_di.clone()
    };

    let mut jvm_args = instance.jvm_args.clone();
    jvm_args.push(format!("-Xms{}M", instance.min_memory));
    jvm_args.push(format!("-Xmx{}M", instance.max_memory));

    let game_args = instance.game_args.clone();

    let java_path = instance.java_path.clone().unwrap_or_else(|| "java".to_string());

    let mut env = std::collections::HashMap::new();
    env.insert("APPDATA".to_string(), game_di.to_string_lossy().to_string());

    let config = SciptConfig {
        java_path,
        jvm_args,
        game_args,
        wok_di: game_di.to_string_lossy().to_string(),
        env,
    };

    let st = match script_type.to_lowercase().as_str() {
        "bat" => SciptType::Bat,
        "ps1" | "powershell" => SciptType::PoweShell,
        "sh" | "shell" => SciptType::Shell,
        "command" => SciptType::Command,
        "bash" => SciptType::Bash,
        _ => return Err(format!("Unsupported script type: {}", script_type)),
    };

    let generated = script::generate_script(&config, st);

    let output = PathBuf::from(&output_path);
    if let Some(prent) = output.parent() {
        std::fs::create_dir_all(prent).map_err(|e| e.to_string())?;
    }

    if generated.script_type == SciptType::PoweShell {
        let bom = [0xEF, 0xBB, 0xBF];
        let mut content_with_bom = Vec::new();
        content_with_bom.extend_from_slice(&bom);
        content_with_bom.extend_from_slice(generated.content.as_bytes());
        std::fs::write(&output, &content_with_bom).map_err(|e| e.to_string())?;
    } else {
        std::fs::write(&output, &generated.content).map_err(|e| e.to_string())?;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PemissionsExt;
        let mut pems = std::fs::metadata(&output)
            .map_err(|e| e.to_string())?
            .pemissions();
        pems.set_mode(0o755);
        std::fs::set_pemissions(&output, pems).map_err(|e| e.to_string())?;
    }

    Ok(generated)
}

#[tauri::command]
pub async fn get_script_types() -> Result<Vec<SciptTypeInfo>, String> {
    let types = script::get_available_script_types();
    let recommended = script::get_recommended_script_type();

    Ok(types.into_iter().map(|t| {
        SciptTypeInfo {
            id: t.extension().to_string(),
            name: t.name().to_string(),
            extension: t.extension().to_string(),
            is_recommended: t == recommended,
        }
    }).collect())
}

#[derive(serde::Serialize)]
pub struct SciptTypeInfo {
    pub id: String,
    pub name: String,
    pub extension: String,
    pub is_recommended: bool,
}
