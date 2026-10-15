//! Автозапуск фонового сторожа: HKCU\…\Run (без прав администратора).

use crate::util;
use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE};
use winreg::RegKey;

const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const NAME: &str = "190x4DiscordKit";

pub fn enable() -> std::io::Result<()> {
    let exe = std::env::current_exe()?;
    let (key, _) = RegKey::predef(HKEY_CURRENT_USER).create_subkey(RUN)?;
    key.set_value(NAME, &format!("\"{}\" --guard", exe.display()))
}

pub fn disable() -> std::io::Result<()> {
    let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(RUN, KEY_SET_VALUE)?;
    match key.delete_value(NAME) {
        Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e),
        _ => Ok(()),
    }
}

pub fn is_enabled() -> bool {
    RegKey::predef(HKEY_CURRENT_USER)
        .open_subkey(RUN)
        .and_then(|k| k.get_value::<String, _>(NAME))
        .is_ok()
}

/// Поднимает сторожа сейчас (второй экземпляр сам выйдет по мьютексу).
pub fn start_guard() -> std::io::Result<()> {
    util::spawn_detached(&std::env::current_exe()?, &["--guard"])
}
