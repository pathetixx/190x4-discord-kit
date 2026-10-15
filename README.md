# 190x4 Discord Kit

Discord в стиле 190x4 в один клик: BetterDiscord, тема 190x4 и плагины ставятся вместе и не слетают после обновлений Discord.

## Что ставит

- **BetterDiscord** — официальный `betterdiscord.asar` с [GitHub BetterDiscord](https://github.com/BetterDiscord/BetterDiscord/releases), подключается к Discord так же, как их установщик.
- **Тема [190x4](https://github.com/pathetixx/190x4-bd-theme)** и плагины: **190x4Companion**, **[RegionSwitcher](https://github.com/pathetixx/190x4-regionswitcher)**, **ShowPing** (исправленная копия, см. `vendor/ShowPing`). Всё сразу включается.
- **Защиту от обновлений Discord** — фоновый сторож без окна стартует вместе с Windows: обновление Discord стирает подключение BetterDiscord, сторож возвращает его за секунды.

Работает со Stable, PTB и Canary. Права администратора не нужны.

## Как пользоваться

1. Скачай `190x4-Discord-Kit_<версия>_x64-setup.exe` из [последнего релиза](https://github.com/pathetixx/190x4-discord-kit/releases/latest) и установи.
   Windows может показать «Система Windows защитила ваш компьютер» — установщик не подписан сертификатом: «Подробнее» → «Выполнить в любом случае».
2. Открой **190x4 Discord Kit** → **УСТАНОВИТЬ ВСЁ**. Discord закроется и запустится уже с темой.

Удалить: кнопка **Удалить комплект** в приложении (Discord станет обычным), затем само приложение — через «Установленные приложения».

## Обновления

- **Приложение** обновляется само (подписанный OTA): раз в сутки тихо, а в окне — кнопкой «Обновить».
- **Тема и плагины** — по `content/manifest.json` в этом репозитории: сторож раз в 6 часов сверяет версии и sha256 и подтягивает новое. Пока Discord открыт, ждёт, чтобы тема не пропала на лету.

Новая версия темы или плагина: поменять `version`/`url` в `scripts/make-manifest.py`, запустить его и закоммитить `content/manifest.json` — друзья получат обновление без переустановки.

## Сборка

Только в GitHub Actions: тег `vX.Y.Z` (совпадает с версией в `src-tauri/tauri.conf.json`) собирает NSIS-установщик, подпись апдейтера и `latest.json`.

## Благодарности

[BetterDiscord](https://github.com/BetterDiscord/BetterDiscord) · иконки темы — [MingCute](https://github.com/Richard9394/MingCute) (Apache 2.0) · [ShowPing](https://github.com/nicola02nb/BetterDiscord-Stuff) — nicola02nb (GPL-3.0) · шрифты Orbitron, Inter, JetBrains Mono (SIL OFL 1.1).

## Лицензия

MIT для кода Kit. `vendor/ShowPing` — GPL-3.0.
