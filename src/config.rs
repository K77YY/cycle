//! 設定とシーケンスの保存 (~/.config/cycle/)。

use crate::model::Sequence;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    pub sound: bool,
    /// false = タイマー(カウントダウン)以外を隠す
    pub show_menu: bool,
    pub last_sequence: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            sound: true,
            show_menu: true,
            last_sequence: None,
        }
    }
}

pub fn config_dir() -> PathBuf {
    if let Some(x) = std::env::var_os("XDG_CONFIG_HOME").filter(|v| !v.is_empty()) {
        PathBuf::from(x).join("cycle")
    } else if let Some(h) = std::env::var_os("HOME") {
        PathBuf::from(h).join(".config").join("cycle")
    } else {
        PathBuf::from(".cycle")
    }
}

fn sequences_dir() -> PathBuf {
    config_dir().join("sequences")
}

fn settings_path() -> PathBuf {
    config_dir().join("settings.json")
}

/// シーケンス名 → 安全なファイル名
pub fn file_stem(name: &str) -> String {
    let s: String = name
        .trim()
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || matches!(c, '-' | '_' | ' ') {
                c
            } else {
                '_'
            }
        })
        .collect();
    let s = s.trim().to_string();
    if s.is_empty() {
        "sequence".to_string()
    } else {
        s
    }
}

fn write_json<T: Serialize>(path: &PathBuf, value: &T) -> Result<(), String> {
    let dir = path.parent().ok_or("保存先が不正です")?;
    fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    let text = serde_json::to_string_pretty(value).map_err(|e| e.to_string())?;
    let tmp = path.with_extension("json.tmp");
    fs::write(&tmp, text).map_err(|e| e.to_string())?;
    fs::rename(&tmp, path).map_err(|e| e.to_string())
}

pub fn load_settings() -> Settings {
    fs::read_to_string(settings_path())
        .ok()
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

pub fn save_settings(settings: &Settings) -> Result<(), String> {
    write_json(&settings_path(), settings)
}

pub fn save_sequence(seq: &Sequence) -> Result<(), String> {
    let path = sequences_dir().join(format!("{}.json", file_stem(&seq.name)));
    write_json(&path, seq)
}

fn read_sequence(path: &PathBuf) -> Result<Sequence, String> {
    let text = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let mut seq: Sequence = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    seq.sanitize()?;
    Ok(seq)
}

pub fn load_sequence(name: &str) -> Result<Sequence, String> {
    read_sequence(&sequences_dir().join(format!("{}.json", file_stem(name))))
}

pub fn list_sequences() -> Result<Vec<Sequence>, String> {
    let entries = match fs::read_dir(sequences_dir()) {
        Ok(e) => e,
        Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(e.to_string()),
    };
    let mut out: Vec<Sequence> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .filter_map(|p| read_sequence(&p).ok())
        .collect();
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}
