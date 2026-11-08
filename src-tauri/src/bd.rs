//! Официальный betterdiscord.asar с GitHub BetterDiscord.

use crate::{paths, util};
use reqwest::blocking::Client;
use serde::Deserialize;

const LATEST: &str = "https://api.github.com/repos/BetterDiscord/BetterDiscord/releases/latest";

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
    #[serde(default)]
    digest: Option<String>,
}

impl Asset {
    /// Хэш, посчитанный GitHub (`sha256:<hex>`).
    fn sha256(&self) -> Option<String> {
        self.digest.as_deref()?.strip_prefix("sha256:").map(str::to_lowercase)
    }
}

fn latest_asar(client: &Client) -> Result<(String, Asset), String> {
    let release: Release = client
        .get(LATEST)
        .send()
        .and_then(|r| r.error_for_status())
        .and_then(|r| r.json())
        .map_err(|e| format!("не удалось узнать версию BetterDiscord: {e}"))?;
    let asset = release
        .assets
        .into_iter()
        .find(|a| a.name == "betterdiscord.asar")
        .ok_or("в релизе BetterDiscord нет betterdiscord.asar")?;
    Ok((release.tag_name, asset))
}

fn write_asset(client: &Client, asset: &Asset) -> Result<(), String> {
    let bytes = client
        .get(&asset.browser_download_url)
        .send()
        .and_then(|r| r.error_for_status())
        .and_then(|r| r.bytes())
        .map_err(|e| format!("не удалось скачать BetterDiscord: {e}"))?;
    if bytes.len() < 100_000 {
        return Err("скачанный BetterDiscord подозрительно мал — прервал установку".into());
    }
    if asset.sha256().is_some_and(|sha| sha != util::sha256_hex(&bytes)) {
        return Err("хэш скачанного BetterDiscord не совпал с релизом — прервал установку".into());
    }
    util::write_atomic(&paths::bd_asar(), &bytes).map_err(|e| format!("не удалось записать BetterDiscord: {e}"))
}

/// Скачивает свежий asar в %AppData%\BetterDiscord\data. Discord должен быть
/// закрыт: запущенный Electron держит asar открытым.
pub fn install_asar(client: &Client) -> Result<String, String> {
    let (tag, asset) = latest_asar(client)?;
    write_asset(client, &asset)?;
    Ok(tag)
}

/// Заменяет asar, если он отличается от ассета последнего релиза, и возвращает
/// тег. Сверяем хэш, а не версию: BD перезаливает исправленный файл под тем же
/// тегом. Discord должен быть закрыт.
pub fn update_asar(client: &Client) -> Result<Option<String>, String> {
    let (tag, asset) = latest_asar(client)?;
    let Some(sha) = asset.sha256() else {
        return Ok(None);
    };
    if util::sha256_file(&paths::bd_asar()).as_deref() == Some(sha.as_str()) {
        return Ok(None);
    }
    write_asset(client, &asset)?;
    Ok(Some(tag))
}

pub fn installed() -> bool {
    paths::bd_asar().is_file()
}
