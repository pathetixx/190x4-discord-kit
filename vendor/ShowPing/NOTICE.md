# ShowPing — модифицированная копия

Оригинал: [nicola02nb/BetterDiscord-Stuff](https://github.com/nicola02nb/BetterDiscord-Stuff/tree/main/Plugins/ShowPing), версия 2.6.13, лицензия GPL-3.0.

Изменение 190x4 (2026-10): Discord минифицирует ключи CSS-модулей (`{ei:"labelWrapper__06d62"}`), из-за чего `Filters.byKeys("labelWrapper", …)` перестал находить классы и плагин падал при загрузке. Модули ищутся по значениям классов (`byClassNames` / `getClasses`), карта имён восстанавливается.

Копия распространяется под той же лицензией GPL-3.0 (`LICENSE`). Если автор выпустит исправленную версию, комплект перейдёт на неё.
