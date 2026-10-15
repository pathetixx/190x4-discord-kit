//! Комплект: тема и плагины из манифеста (content/manifest.json в репо Kit).
//! Каждый файл проверяется по sha256 из манифеста; ставится атомарно.

use crate::{discord, paths, state::State, util};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const MANIFEST_URLS: [&str; 2] = [
    "https://raw.githubusercontent.com/pathetixx/190x4-discord-kit/main/content/manifest.json",
    "https://cdn.jsdelivr.net/gh/pathetixx/190x4-discord-kit@main/content/manifest.json",
];

#[derive(Deserialize, Serialize, Clone)]
pub struct Item {
    pub id: String,
    pub kind: String,
    /// @name аддона — ключ в themes.json / plugins.json BD
    pub name: String,
    pub file: String,
    pub version: String,
    pub url: String,
    pub sha256: String,
}

#[derive(Deserialize)]
pub struct Manifest {
    pub schema: u32,
    pub items: Vec<Item>,
}

#[derive(Serialize, Clone)]
pub struct ItemStatus {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub version: Option<String>,
    pub present: bool,
}

pub fn fetch_manifest(client: &Client) -> Result<Manifest, String> {
    for url in MANIFEST_URLS {
        let got = client
            .get(url)
            .send()
            .and_then(|r| r.error_for_status())
            .and_then(|r| r.json::<Manifest>());
        if let Ok(m) = got {
            if m.schema == 1 {
                return Ok(m);
            }
        }
    }
    Err("список комплекта недоступен — проверь интернет".into())
}

fn safe_name(file: &str) -> bool {
    !file.is_empty() && !file.contains(['/', '\\']) && !file.contains("..")
}

fn target(item: &Item) -> Option<PathBuf> {
    if !safe_name(&item.file) {
        return None;
    }
    let dir = if item.kind == "theme" {
        paths::bd_themes()
    } else {
        paths::bd_plugins()
    };
    Some(dir.join(&item.file))
}

fn needs_update(item: &Item) -> bool {
    match target(item) {
        Some(path) => util::sha256_file(&path).as_deref() != Some(item.sha256.to_lowercase().as_str()),
        None => false,
    }
}

fn install_item(client: &Client, item: &Item) -> Result<(), String> {
    let path = target(item).ok_or_else(|| format!("{}: недопустимое имя файла", item.name))?;
    let bytes = client
        .get(&item.url)
        .send()
        .and_then(|r| r.error_for_status())
        .and_then(|r| r.bytes())
        .map_err(|e| format!("{}: не скачался ({e})", item.name))?;
    if util::sha256_hex(&bytes) != item.sha256.to_lowercase() {
        return Err(format!("{}: контрольная сумма не совпала — файл не тронут", item.name));
    }
    util::write_atomic(&path, &bytes).map_err(|e| format!("{}: не записался ({e})", item.name))
}

/// Включает аддон в BD для всех найденных сборок Discord.
fn enable(item: &Item) {
    let file = if item.kind == "theme" {
        "themes.json"
    } else {
        "plugins.json"
    };
    let mut channels: Vec<&str> = discord::present().iter().map(|f| f.channel).collect();
    if channels.is_empty() {
        channels.push("stable");
    }
    for channel in channels {
        let path = paths::bd_channel(channel).join(file);
        let mut map: serde_json::Map<String, serde_json::Value> = std::fs::read(&path)
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default();
        map.insert(item.name.clone(), serde_json::Value::Bool(true));
        if let Ok(bytes) = serde_json::to_vec_pretty(&map) {
            let _ = util::write_atomic(&path, &bytes);
        }
    }
}

/// Ставит/обновляет всё, что расходится с манифестом. `enable_new` — при
/// первой установке включить аддоны (дальше выбор юзера в BD не трогаем).
/// Возвращает имена обновлённых элементов.
pub fn sync(client: &Client, state: &mut State, enable_new: bool) -> Result<Vec<String>, String> {
    let manifest = fetch_manifest(client)?;
    let mut updated = Vec::new();
    for item in &manifest.items {
        if needs_update(item) {
            install_item(client, item)?;
            updated.push(item.name.clone());
        }
        state.installed.insert(item.id.clone(), item.version.clone());
        if enable_new && !state.enabled_once.contains(&item.id) {
            enable(item);
            state.enabled_once.push(item.id.clone());
        }
    }
    let _ = state.save();
    Ok(updated)
}

/// Что из комплекта стоит сейчас (по манифесту, если он доступен).
pub fn status(client: &Client, state: &State) -> Vec<ItemStatus> {
    let Ok(manifest) = fetch_manifest(client) else {
        return Vec::new();
    };
    manifest
        .items
        .iter()
        .map(|item| ItemStatus {
            id: item.id.clone(),
            name: item.name.clone(),
            kind: item.kind.clone(),
            version: state.installed.get(&item.id).cloned(),
            present: target(item).map(|p| p.is_file()).unwrap_or(false),
        })
        .collect()
}

/// Удаляет файлы комплекта (BD и чужие аддоны не трогаем).
pub fn remove_all(client: &Client) {
    if let Ok(manifest) = fetch_manifest(client) {
        for item in &manifest.items {
            if let Some(path) = target(item) {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}
