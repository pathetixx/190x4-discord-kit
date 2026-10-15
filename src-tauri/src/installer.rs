//! Установка и удаление комплекта по шагам — с прогрессом в окно.

use crate::{autostart, bd, content, discord, net, paths, state::State};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

#[derive(Serialize, Clone)]
struct Progress {
    step: &'static str,
    state: &'static str,
    text: String,
}

fn emit(app: &AppHandle, step: &'static str, state: &'static str, text: impl Into<String>) {
    let _ = app.emit(
        "kit-progress",
        Progress {
            step,
            state,
            text: text.into(),
        },
    );
}

/// Шаг: run → ok | err (ошибка прерывает установку).
fn step<T>(app: &AppHandle, id: &'static str, title: &str, f: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    emit(app, id, "run", title);
    match f() {
        Ok(v) => {
            emit(app, id, "ok", title);
            Ok(v)
        }
        Err(e) => {
            emit(app, id, "err", e.clone());
            Err(e)
        }
    }
}

pub fn install_all(app: &AppHandle) -> Result<(), String> {
    let client = net::client();
    let mut state = State::load();

    let flavors = step(app, "discord", "Ищу Discord", || {
        let found = discord::present();
        if found.is_empty() {
            Err("Discord не найден. Установи его с discord.com и запусти хотя бы раз.".into())
        } else {
            Ok(found)
        }
    })?;

    let was_running: Vec<_> = flavors
        .iter()
        .filter(|f| crate::util::is_running(f.exe))
        .copied()
        .collect();
    step(app, "close", "Закрываю Discord", || {
        for f in &flavors {
            discord::close(f);
        }
        std::thread::sleep(std::time::Duration::from_millis(1500));
        Ok(())
    })?;

    let tag = step(app, "bd", "Ставлю BetterDiscord", || bd::install_asar(&client))?;
    state.bd_version = Some(tag);
    let _ = state.save();

    step(app, "inject", "Подключаю BetterDiscord к Discord", || {
        discord::inject_all().map_err(|e| format!("не удалось изменить файлы Discord: {e}"))
    })?;

    step(app, "content", "Ставлю тему и плагины 190x4", || {
        content::sync(&client, &mut state, true)
    })?;

    step(
        app,
        "guard",
        "Включаю защиту от обновлений Discord",
        || {
            autostart::enable().map_err(|e| format!("не удалось включить автозапуск: {e}"))?;
            autostart::start_guard().map_err(|e| format!("не удалось запустить защиту: {e}"))
        },
    )?;

    step(app, "launch", "Запускаю Discord", || {
        let targets = if was_running.is_empty() {
            vec![flavors[0]]
        } else {
            was_running.clone()
        };
        for f in targets {
            discord::start(f);
        }
        Ok(())
    })?;
    Ok(())
}

pub fn uninstall_all(app: &AppHandle) -> Result<(), String> {
    let client = net::client();
    let flavors = discord::present();
    let was_running: Vec<_> = flavors
        .iter()
        .filter(|f| crate::util::is_running(f.exe))
        .copied()
        .collect();

    step(
        app,
        "guard",
        "Выключаю защиту от обновлений",
        || autostart::disable().map_err(|e| format!("не удалось убрать автозапуск: {e}")),
    )?;
    step(app, "close", "Закрываю Discord", || {
        for f in &flavors {
            discord::close(f);
        }
        std::thread::sleep(std::time::Duration::from_millis(1500));
        Ok(())
    })?;
    step(app, "inject", "Отключаю BetterDiscord от Discord", || {
        discord::uninject_all().map_err(|e| format!("не удалось изменить файлы Discord: {e}"))
    })?;
    step(app, "content", "Убираю тему и плагины 190x4", || {
        content::remove_all(&client);
        let _ = std::fs::remove_file(paths::state_file());
        Ok(())
    })?;
    step(app, "launch", "Запускаю Discord", || {
        for f in was_running {
            discord::start(f);
        }
        Ok(())
    })?;
    Ok(())
}

pub fn restart_discord() {
    let flavors = discord::present();
    let running: Vec<_> = flavors
        .iter()
        .filter(|f| crate::util::is_running(f.exe))
        .copied()
        .collect();
    let targets = if running.is_empty() {
        flavors.into_iter().take(1).collect()
    } else {
        running
    };
    for f in &targets {
        discord::close(f);
    }
    std::thread::sleep(std::time::Duration::from_millis(1500));
    for f in targets {
        discord::start(f);
    }
}
