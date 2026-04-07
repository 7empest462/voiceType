use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use rdev::Key;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub hotkey: String,
    pub model: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            hotkey: "AltGr".to_string(),
            model: "qwen2.5-coder:3b".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let config_path = Self::get_config_path();
        if let Ok(content) = fs::read_to_string(&config_path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
        
        let default = Self::default();
        let _ = default.save();
        default
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let config_path = Self::get_config_path();
        let content = serde_json::to_string_pretty(self)?;
        fs::write(config_path, content)?;
        Ok(())
    }

    fn get_config_path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        std::path::Path::new(&home).join(".voice-type").join("config.json")
    }

    pub fn get_target_key(&self) -> Key {
        match self.hotkey.to_lowercase().as_str() {
            "altgr" => Key::AltGr,
            "alt" => Key::Alt,
            "controlleft" => Key::ControlLeft,
            "controlright" => Key::ControlRight,
            "shiftleft" => Key::ShiftLeft,
            "shiftright" => Key::ShiftRight,
            "meta" => Key::MetaLeft, // Command on Mac / Windows Key
            "metaleft" => Key::MetaLeft,
            "metaright" => Key::MetaRight,
            "capslock" => Key::CapsLock,
            "f1" => Key::F1,
            "f2" => Key::F2,
            "f3" => Key::F3,
            "f4" => Key::F4,
            "f5" => Key::F5,
            "f6" => Key::F6,
            "f7" => Key::F7,
            "f8" => Key::F8,
            "f9" => Key::F9,
            "f10" => Key::F10,
            "f11" => Key::F11,
            "f12" => Key::F12,
            "space" => Key::Space,
            "tab" => Key::Tab,
            "keya" => Key::KeyA,
            "keyb" => Key::KeyB,
            // Fallback to AltGr
            _ => Key::AltGr,
        }
    }
}
