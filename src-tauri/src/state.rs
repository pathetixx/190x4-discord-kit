//! Состояние Kit между запусками (%LocalAppData%\190x4DiscordKit\state.json).

use crate::{paths, util};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Default, Clone)]
pub struct State {
    /// id элемента манифеста → установленная версия
    #[serde(default)]
    pub installed: HashMap<String, String>,
    /// элементы, которые мы уже один раз включили в BD (дальше уважаем выбор юзера)
    #[serde(default)]
    pub enabled_once: Vec<String>,
    #[serde(default)]
    pub bd_version: Option<String>,
    #[serde(default)]
    pub last_content_check: u64,
    #[serde(default)]
    pub content_pending_since: u64,
    #[serde(default)]
    pub last_self_update: u64,
}

impl State {
    pub fn load() -> Self {
        std::fs::read(paths::state_file())
            .ok()
            .and_then(|b| serde_json::from_slice(&b).ok())
            .unwrap_or_default()
    }

    pub fn save(&self) -> std::io::Result<()> {
        let bytes = serde_json::to_vec_pretty(self).map_err(std::io::Error::other)?;
        util::write_atomic(&paths::state_file(), &bytes)
    }
}
