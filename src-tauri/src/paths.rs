//! Пути BetterDiscord, Discord и самого Kit.

use std::path::PathBuf;

fn env_dir(key: &str) -> PathBuf {
    PathBuf::from(std::env::var_os(key).unwrap_or_default())
}

pub fn appdata() -> PathBuf {
    env_dir("APPDATA")
}

pub fn local_appdata() -> PathBuf {
    env_dir("LOCALAPPDATA")
}

pub fn bd_root() -> PathBuf {
    appdata().join("BetterDiscord")
}

pub fn bd_asar() -> PathBuf {
    bd_root().join("data").join("betterdiscord.asar")
}

pub fn bd_themes() -> PathBuf {
    bd_root().join("themes")
}

pub fn bd_plugins() -> PathBuf {
    bd_root().join("plugins")
}

/// data/<канал> — BD хранит включённые аддоны отдельно для stable/ptb/canary.
pub fn bd_channel(channel: &str) -> PathBuf {
    bd_root().join("data").join(channel)
}

pub fn kit_dir() -> PathBuf {
    local_appdata().join("190x4DiscordKit")
}

pub fn state_file() -> PathBuf {
    kit_dir().join("state.json")
}
