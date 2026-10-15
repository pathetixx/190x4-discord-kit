//! Фоновый сторож (`--guard`): стартует с Windows из HKCU\Run, без окна.
//! • каждые 15 с проверяет index.js всех сборок Discord и возвращает BD,
//!   если обновление Discord его стёрло;
//! • раз в 6 ч сверяет комплект с манифестом и тихо обновляет тему/плагины
//!   (пока Discord открыт — ждёт, чтобы BD не терял тему на лету; после
//!   3 суток ожидания обновляет всё равно и просит перезапустить Discord);
//! • раз в сутки запускает `--self-update` — OTA самого Kit.
//! Выходит сам, если автозапуск выключили (удаление комплекта).

use crate::{autostart, bd, content, discord, net, state::State, util};
use std::time::Duration;
use tauri_winrt_notification::Toast;

const TICK: Duration = Duration::from_secs(15);
const CONTENT_EVERY: u64 = 6 * 3600;
const CONTENT_RETRY: u64 = 3600;
const PENDING_MAX: u64 = 3 * 24 * 3600;
const SELF_UPDATE_EVERY: u64 = 24 * 3600;

fn toast(text: &str) {
    let _ = Toast::new(Toast::POWERSHELL_APP_ID)
        .title("190x4 Discord Kit")
        .text1(text)
        .show();
}

/// Один сторож на пользователя: именованный мьютекс живёт до выхода процесса.
fn single_instance() -> bool {
    use windows_sys::Win32::Foundation::{GetLastError, ERROR_ALREADY_EXISTS};
    use windows_sys::Win32::System::Threading::CreateMutexW;
    let name: Vec<u16> = "Local\\190x4DiscordKitGuard".encode_utf16().chain(Some(0)).collect();
    // SAFETY: имя — валидная нуль-терминированная UTF-16 строка; хэндл
    // намеренно не закрываем — он держит мьютекс до конца процесса.
    unsafe {
        let handle = CreateMutexW(std::ptr::null(), 0, name.as_ptr());
        !handle.is_null() && GetLastError() != ERROR_ALREADY_EXISTS
    }
}

pub fn run() {
    if !single_instance() {
        return;
    }
    let client = net::client();
    let mut state = State::load();
    loop {
        if !autostart::is_enabled() {
            return;
        }
        if bd::installed() {
            if let Ok(n) = discord::inject_all() {
                if n > 0 {
                    toast("Discord обновился — BetterDiscord и тема 190x4 возвращены. Если тема не появилась, перезапусти Discord через трей (Выход).");
                }
            }
        }

        let now = util::now();
        if bd::installed() && now.saturating_sub(state.last_content_check) >= CONTENT_EVERY {
            let waited =
                state.content_pending_since != 0 && now.saturating_sub(state.content_pending_since) >= PENDING_MAX;
            if discord::any_running() && !waited {
                if state.content_pending_since == 0 {
                    state.content_pending_since = now;
                    let _ = state.save();
                }
            } else {
                let running = discord::any_running();
                match content::sync(&client, &mut state, false) {
                    Ok(updated) => {
                        state.last_content_check = now;
                        state.content_pending_since = 0;
                        let _ = state.save();
                        if !updated.is_empty() {
                            let tail = if running {
                                " Перезапусти Discord, чтобы увидеть."
                            } else {
                                ""
                            };
                            toast(&format!("Обновлено: {}.{tail}", updated.join(", ")));
                        }
                    }
                    Err(_) => {
                        state.last_content_check = now.saturating_sub(CONTENT_EVERY - CONTENT_RETRY);
                        let _ = state.save();
                    }
                }
            }
        }

        if now.saturating_sub(state.last_self_update) >= SELF_UPDATE_EVERY {
            state.last_self_update = now;
            let _ = state.save();
            if let Ok(exe) = std::env::current_exe() {
                let _ = util::spawn_detached(&exe, &["--self-update"]);
            }
        }

        std::thread::sleep(TICK);
    }
}
