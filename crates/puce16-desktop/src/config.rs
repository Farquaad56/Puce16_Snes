use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, PartialEq, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct Config {
    pub scale: u8,
    pub fit_window: bool,
    pub ratio: String,
    pub filter: String,
    pub volume: u8,
    pub muted: bool,
    pub fullscreen: bool,
    pub block_opposite_dirs: bool,
    pub pause_on_focus_loss: bool,
    pub recent_roms: Vec<String>,
    pub last_dir: String,
    pub keymap: BTreeMap<String, Vec<String>>,
    pub pad_map: BTreeMap<String, String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            scale: 3,
            fit_window: false,
            ratio: "square".to_string(),
            filter: "nearest".to_string(),
            volume: 100,
            muted: false,
            fullscreen: false,
            block_opposite_dirs: true,
            pause_on_focus_loss: false,
            recent_roms: Vec::new(),
            last_dir: String::new(),
            keymap: BTreeMap::new(),
            pad_map: BTreeMap::new(),
        }
    }
}

impl Config {
    pub fn sanitize(&mut self) {
        if self.scale < 1 || self.scale > 8 {
            self.scale = 3;
        }
        if self.volume > 100 {
            self.volume = 100;
        }
        if self.ratio != "square" && self.ratio != "par87" {
            self.ratio = "square".to_string();
        }
        if self.filter != "nearest" && self.filter != "linear" && self.filter != "scanlines" {
            self.filter = "nearest".to_string();
        }
        // Truncate recent_roms to 8, remove duplicates (keep first occurrence)
        let mut seen = std::collections::HashSet::new();
        let mut cleaned = Vec::new();
        for rom in &self.recent_roms {
            if !seen.contains(rom.as_str()) && cleaned.len() < 8 {
                seen.insert(rom.as_str());
                cleaned.push(rom.clone());
            }
        }
        self.recent_roms = cleaned;
    }

    pub fn add_recent(&mut self, path: &str) {
        // Remove existing entry if present
        self.recent_roms.retain(|r| r != path);
        // Add to front
        let mut new_list = Vec::new();
        new_list.push(path.to_string());
        for rom in &self.recent_roms {
            if new_list.len() >= 8 {
                break;
            }
            new_list.push(rom.clone());
        }
        self.recent_roms = new_list;
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default() + "\n"
    }

    pub fn from_json(text: &str) -> Config {
        match serde_json::from_str::<Config>(text) {
            Ok(mut cfg) => {
                cfg.sanitize();
                cfg
            }
            Err(_) => Config::default(),
        }
    }
}

pub fn config_dir_from(appdata: Option<&str>, home: Option<&str>) -> Option<PathBuf> {
    if let Some(a) = appdata {
        if !a.is_empty() {
            return Some(PathBuf::from(a).join("Puce16"));
        }
    }
    if let Some(h) = home {
        if !h.is_empty() {
            return Some(PathBuf::from(h).join(".config").join("puce16"));
        }
    }
    None
}

pub fn config_path() -> Option<PathBuf> {
    let appdata = std::env::var("APPDATA").ok();
    let home = std::env::var("HOME").ok();
    config_dir_from(appdata.as_deref(), home.as_deref()).map(|d| d.join("config.json"))
}

pub fn load() -> Config {
    match config_path() {
        Some(path) => match fs::read_to_string(&path) {
            Ok(text) => Config::from_json(&text),
            Err(_) => Config::default(),
        },
        None => Config::default(),
    }
}

pub fn save(cfg: &Config) -> Result<(), String> {
    let path = config_path().ok_or_else(|| "config path not found".to_string())?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("failed to create config dir: {}", e))?;
    }
    super::files::write_atomic(&path, cfg.to_json().as_bytes()).map_err(|e| format!("failed to write config: {}", e))
}

