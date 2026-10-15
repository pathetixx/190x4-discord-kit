//! Поиск установок Discord и вставка/снятие BetterDiscord.
//!
//! Обновление Discord затирает только `discord_desktop_core/index.js`; BD
//! держится одной строкой в его начале (формат как у BD Guard):
//! `try { require("…/betterdiscord.asar"); } catch (e) {}`.
//! Модули: новый апдейтер — %LocalAppData%\Discord*\app-<ver>\modules\…,
//! старый — %AppData%\discord*\<ver>\modules\….

use crate::{paths, util};
use serde::Serialize;
use std::path::PathBuf;
use std::time::Duration;

pub struct Flavor {
    pub id: &'static str,
    pub title: &'static str,
    local: &'static str,
    roaming: &'static str,
    pub exe: &'static str,
    /// папка данных BD: data/<channel>
    pub channel: &'static str,
}

pub const FLAVORS: [Flavor; 3] = [
    Flavor {
        id: "stable",
        title: "Discord",
        local: "Discord",
        roaming: "discord",
        exe: "Discord.exe",
        channel: "stable",
    },
    Flavor {
        id: "ptb",
        title: "Discord PTB",
        local: "DiscordPTB",
        roaming: "discordptb",
        exe: "DiscordPTB.exe",
        channel: "ptb",
    },
    Flavor {
        id: "canary",
        title: "Discord Canary",
        local: "DiscordCanary",
        roaming: "discordcanary",
        exe: "DiscordCanary.exe",
        channel: "canary",
    },
];

#[derive(Serialize, Clone)]
pub struct Install {
    pub id: String,
    pub title: String,
    pub injected: bool,
    pub running: bool,
}

const MARKER: &str = "betterdiscord.asar";

fn is_version_dir(name: &str) -> bool {
    if let Some(rest) = name.strip_prefix("app-") {
        return !rest.is_empty();
    }
    let parts: Vec<&str> = name.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.chars().all(|c| c.is_ascii_digit()))
}

fn roots(f: &Flavor) -> Vec<PathBuf> {
    [paths::local_appdata().join(f.local), paths::appdata().join(f.roaming)]
        .into_iter()
        .filter(|p| p.is_dir())
        .collect()
}

fn dirs(path: &PathBuf) -> Vec<(String, PathBuf)> {
    std::fs::read_dir(path)
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| e.path().is_dir())
                .map(|e| (e.file_name().to_string_lossy().into_owned(), e.path()))
                .collect()
        })
        .unwrap_or_default()
}

/// Все discord_desktop_core/index.js данной сборки Discord.
pub fn core_indexes(f: &Flavor) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for root in roots(f) {
        for (name, app) in dirs(&root) {
            if !is_version_dir(&name) {
                continue;
            }
            for (module, dir) in dirs(&app.join("modules")) {
                if module.starts_with("discord_desktop_core-") {
                    let index = dir.join("discord_desktop_core").join("index.js");
                    if index.is_file() {
                        out.push(index);
                    }
                }
            }
        }
    }
    out
}

fn is_injected(index: &PathBuf) -> bool {
    std::fs::read_to_string(index)
        .map(|s| s.contains(MARKER))
        .unwrap_or(false)
}

pub fn installs() -> Vec<Install> {
    FLAVORS
        .iter()
        .filter_map(|f| {
            let indexes = core_indexes(f);
            if indexes.is_empty() {
                return None;
            }
            Some(Install {
                id: f.id.into(),
                title: f.title.into(),
                injected: indexes.iter().all(is_injected),
                running: util::is_running(f.exe),
            })
        })
        .collect()
}

pub fn present() -> Vec<&'static Flavor> {
    FLAVORS.iter().filter(|f| !core_indexes(f).is_empty()).collect()
}

fn inject_line() -> String {
    let asar = paths::bd_asar().to_string_lossy().replace('\\', "/");
    format!("try {{ require(\"{asar}\"); }} catch (e) {{}}")
}

/// Вписывает BD во все index.js, где его нет. Возвращает число исправленных.
pub fn inject_all() -> std::io::Result<usize> {
    let line = inject_line();
    let mut patched = 0;
    for f in &FLAVORS {
        for index in core_indexes(f) {
            let content = std::fs::read_to_string(&index)?;
            if content.contains(MARKER) {
                continue;
            }
            // апдейтер может ещё держать файл — несколько попыток
            let mut last = Ok(());
            for _ in 0..5 {
                last = std::fs::write(&index, format!("{line}\n{content}"));
                if last.is_ok() {
                    patched += 1;
                    break;
                }
                std::thread::sleep(Duration::from_millis(300));
            }
            last?;
        }
    }
    Ok(patched)
}

/// Убирает строку BD отовсюду (для удаления комплекта).
pub fn uninject_all() -> std::io::Result<usize> {
    let mut cleaned = 0;
    for f in &FLAVORS {
        for index in core_indexes(f) {
            let content = std::fs::read_to_string(&index)?;
            if !content.contains(MARKER) {
                continue;
            }
            let kept: Vec<&str> = content.lines().filter(|l| !l.contains(MARKER)).collect();
            std::fs::write(&index, kept.join("\n") + "\n")?;
            cleaned += 1;
        }
    }
    Ok(cleaned)
}

pub fn close(f: &Flavor) {
    util::kill(f.exe);
}

pub fn start(f: &Flavor) {
    let updater = paths::local_appdata().join(f.local).join("Update.exe");
    if updater.is_file() {
        let _ = util::spawn_detached(&updater, &["--processStart", f.exe]);
    }
}

pub fn any_running() -> bool {
    FLAVORS.iter().any(|f| util::is_running(f.exe))
}
