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
}

/// Скачивает свежий asar в %AppData%\BetterDiscord\data. Discord должен быть
/// закрыт: запущенный Electron держит asar открытым.
pub fn install_asar(client: &Client) -> Result<String, String> {
    let release: Release = client
        .get(LATEST)
        .send()
        .and_then(|r| r.error_for_status())
        .and_then(|r| r.json())
        .map_err(|e| format!("не удалось узнать версию BetterDiscord: {e}"))?;
    let asset = release
        .assets
        .iter()
        .find(|a| a.name == "betterdiscord.asar")
        .ok_or("в релизе BetterDiscord нет betterdiscord.asar")?;
    let bytes = client
        .get(&asset.browser_download_url)
        .send()
        .and_then(|r| r.error_for_status())
        .and_then(|r| r.bytes())
        .map_err(|e| format!("не удалось скачать BetterDiscord: {e}"))?;
    if bytes.len() < 100_000 {
        return Err("скачанный BetterDiscord подозрительно мал — прервал установку".into());
    }
    util::write_atomic(&paths::bd_asar(), &bytes).map_err(|e| format!("не удалось записать BetterDiscord: {e}"))?;
    Ok(release.tag_name)
}

pub fn installed() -> bool {
    paths::bd_asar().is_file()
}
