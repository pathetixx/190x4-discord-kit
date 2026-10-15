//! Мелкие помощники: атомарная запись, sha256, процессы без окна.

use sha2::{Digest, Sha256};
use std::io;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Command;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const DETACHED_PROCESS: u32 = 0x0000_0008;

/// Пишем во временный файл рядом и переименовываем: BD и Discord никогда не
/// видят наполовину записанный файл.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)?;
    }
    let tmp = path.with_extension("kit-tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}

pub fn sha256_file(path: &Path) -> Option<String> {
    std::fs::read(path).ok().map(|b| sha256_hex(&b))
}

pub fn hidden(program: &str) -> Command {
    let mut cmd = Command::new(program);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

pub fn spawn_detached(exe: &Path, args: &[&str]) -> io::Result<()> {
    Command::new(exe)
        .args(args)
        .creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS)
        .spawn()
        .map(|_| ())
}

pub fn is_running(image: &str) -> bool {
    hidden("tasklist")
        .args(["/FI", &format!("IMAGENAME eq {image}"), "/NH", "/FO", "CSV"])
        .output()
        .map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .to_lowercase()
                .contains(&format!("\"{}\"", image.to_lowercase()))
        })
        .unwrap_or(false)
}

pub fn kill(image: &str) {
    let _ = hidden("taskkill").args(["/F", "/IM", image]).output();
}

pub fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
