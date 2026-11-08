//! Комплект: тема и плагины из манифеста (content/manifest.json в репо Kit).
//! Каждый файл проверяется по sha256 из манифеста; ставится атомарно.

use crate::{discord, paths, state::State, util};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Отдельный файл: content/manifest.json читают уже установленные Kit < 0.2,
/// они поставили бы вторую тему и не умеют выбирать между темами.
const MANIFEST_URLS: [&str; 2] = [
    "https://raw.githubusercontent.com/pathetixx/190x4-discord-kit/main/content/manifest2.json",
    "https://cdn.jsdelivr.net/gh/pathetixx/190x4-discord-kit@main/content/manifest2.json",
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
    /// подпись темы в окне Kit
    #[serde(default)]
    pub title: Option<String>,
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
    pub title: String,
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
            if m.schema == 2 {
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
    set_enabled(file, &[(item.name.clone(), true)]);
}

/// Включает выбранную тему комплекта и выключает остальные его темы.
fn activate_theme(manifest: &Manifest, id: &str) {
    let changes: Vec<(String, bool)> = manifest
        .items
        .iter()
        .filter(|i| i.kind == "theme")
        .map(|i| (i.name.clone(), i.id == id))
        .collect();
    set_enabled("themes.json", &changes);
}

fn set_enabled(file: &str, changes: &[(String, bool)]) {
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
        for (name, on) in changes {
            map.insert(name.clone(), serde_json::Value::Bool(*on));
        }
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
            if item.kind != "theme" {
                enable(item);
            }
            state.enabled_once.push(item.id.clone());
        }
    }
    if enable_new {
        let themes = || manifest.items.iter().filter(|i| i.kind == "theme");
        let chosen = state
            .selected_theme
            .clone()
            .filter(|id| themes().any(|i| &i.id == id))
            .or_else(|| themes().next().map(|i| i.id.clone()));
        if let Some(id) = chosen {
            activate_theme(&manifest, &id);
            state.selected_theme = Some(id);
        }
    }
    let _ = state.save();
    Ok(updated)
}

/// Делает тему `id` активной: докачивает её файл, если нужно, и переключает BD.
pub fn select_theme(client: &Client, state: &mut State, id: &str) -> Result<(), String> {
    let manifest = fetch_manifest(client)?;
    let item = manifest
        .items
        .iter()
        .find(|i| i.kind == "theme" && i.id == id)
        .ok_or_else(|| "такой темы нет в комплекте".to_string())?;
    if needs_update(item) {
        install_item(client, item)?;
    }
    state.installed.insert(item.id.clone(), item.version.clone());
    activate_theme(&manifest, &item.id);
    state.selected_theme = Some(item.id.clone());
    state
        .save()
        .map_err(|e| format!("не удалось сохранить выбор темы: {e}"))
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
            title: item.title.clone().unwrap_or_else(|| item.name.clone()),
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
