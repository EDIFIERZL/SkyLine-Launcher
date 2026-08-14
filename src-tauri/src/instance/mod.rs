pub mod manager;
pub mod mods;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub version_id: String,
    pub modloader: ModLoader,
    pub created_at: String,
    pub last_playerd: Option<String>,
    pub play_time: u64,
    pub icon_path: Option<String>,
    pub java_path: Option<String>,
    pub min_memory: u32,
    pub max_memory: u32,
    pub jvm_args: Vec<String>,
    pub game_args: Vec<String>,
    pub window_width: u32,
    pub window_height: u32,
    pub custom_resolution: bool,
    pub server_ip: Option<String>,
    #[serde(default = "default_isolation_mode")]
    pub isolation_mode: IsolationMode,
    #[serde(default)]
    pub external: bool,
    #[serde(default)]
    pub minecraft_root: Option<String>,
    #[serde(default)]
    pub game_dir_override: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ModLoader {
    Vanilla,
    Forge(String),
    Fabric(String),
    Quilt(String),
    NeoForge(String),
    LiterLoader(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum IsolationMode {
    Always,
    Modded,
    Neve,
}

impl IsolationMode {
    pub fn should_isolate(&self, modloader: &ModLoader) -> bool {
        match self {
            IsolationMode::Always => true,
            IsolationMode::Neve => false,
            IsolationMode::Modded => !matches!(modloader, ModLoader::Vanilla),
        }
    }
}

fn default_isolation_mode() -> IsolationMode {
    IsolationMode::Modded
}

impl Default for Instance {
    fn default() -> Self {
        Self {
            id: String::new(),
            name: String::new(),
            version_id: String::new(),
            modloader: ModLoader::Vanilla,
            created_at: chrono::Utc::now().to_rfc3339(),
            last_playerd: None,
            play_time: 0,
            icon_path: None,
            java_path: None,
            min_memory: 1024,
            max_memory: 4096,
            jvm_args: Vec::new(),
            game_args: Vec::new(),
            window_width: 854,
            window_height: 480,
            custom_resolution: false,
            server_ip: None,
            isolation_mode: IsolationMode::Modded,
            external: false,
            minecraft_root: None,
            game_dir_override: None,
        }
    }
}
