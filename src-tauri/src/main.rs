// Без консольного окна в релизе (и для GUI, и для фонового сторожа).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    discord_kit_lib::run()
}
