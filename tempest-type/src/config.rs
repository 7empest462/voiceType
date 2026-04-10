use serde::{Deserialize, Serialize};
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
            model: "qwen2.5:3b".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Self {
        confy::load("tempest-type", None).unwrap_or_else(|e| {
            eprintln!("⚠️  Failed to load config: {}. Using defaults.", e);
            Self::default()
        })
    }

    pub fn save(&self) -> anyhow::Result<()> {
        confy::store("tempest-type", None, self).map_err(|e| anyhow::anyhow!("Failed to save config: {}", e))
    }

    pub fn get_target_key(&self) -> Key {
        match self.hotkey.to_lowercase().as_str() {
            "altgr" => Key::AltGr,
            "alt" => Key::Alt,
            "controlleft" => Key::ControlLeft,
            "controlright" => Key::ControlRight,
            "shiftleft" => Key::ShiftLeft,
            "shiftright" => Key::ShiftRight,
            "meta" => Key::MetaLeft, 
            "metaleft" => Key::MetaLeft,
            "metaright" => Key::MetaRight,
            "capslock" => Key::CapsLock,
            "f1" => Key::F1,
            "f12" => Key::F12,
            "space" => Key::Space,
            "tab" => Key::Tab,
            _ => Key::AltGr,
        }
    }
}
