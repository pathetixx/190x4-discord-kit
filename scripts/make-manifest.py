#!/usr/bin/env python3
"""Пересобирает манифесты: скачивает каждый файл по его URL и пишет фактические
sha256/размер. Запускать после выхода новой версии темы или плагина (поменяв
version/url ниже).

content/manifest.json — для установленных Kit < 0.2: только первая тема.
content/manifest2.json — для Kit с выбором темы: все темы и подписи."""
import hashlib, json, pathlib, urllib.request

ROOT = pathlib.Path(__file__).resolve().parent.parent
ITEMS = [
    dict(id="theme-190x4", kind="theme", name="190x4", title="190x4", file="190x4.theme.css", version="4.3.0",
         url="https://github.com/pathetixx/190x4-bd-theme/releases/download/theme-v4.3.0/190x4.theme.css"),
    dict(id="theme-eclipse", kind="theme", name="190x4 Eclipse", title="Затмение", file="190x4-Eclipse.theme.css", version="1.0.0",
         url="https://github.com/pathetixx/190x4-bd-theme/releases/download/theme-v4.4.0/190x4-Eclipse.theme.css"),
    dict(id="plugin-companion", kind="plugin", name="190x4Companion", file="190x4Companion.plugin.js", version="2.6.0",
         url="https://github.com/pathetixx/190x4-bd-theme/releases/download/theme-v4.3.0/190x4Companion.plugin.js"),
    dict(id="plugin-regionswitcher", kind="plugin", name="RegionSwitcher", file="RegionSwitcher.plugin.js", version="4.2.1",
         url="https://raw.githubusercontent.com/pathetixx/190x4-regionswitcher/687ceb16e7c1de209134a3d3a77c4d7cd977209d/RegionSwitcher.plugin.js"),
    dict(id="plugin-showping", kind="plugin", name="ShowPing", file="ShowPing.plugin.js", version="2.6.13-190x4.1",
         url="https://raw.githubusercontent.com/pathetixx/190x4-discord-kit/main/vendor/ShowPing/ShowPing.plugin.js"),
]

def fetch(url: str) -> bytes:
    req = urllib.request.Request(url, headers={"User-Agent": "190x4-discord-kit-manifest"})
    with urllib.request.urlopen(req, timeout=60) as r:
        return r.read()

def main() -> None:
    out = []
    for item in ITEMS:
        if item["url"].startswith("https://raw.githubusercontent.com/pathetixx/190x4-discord-kit/main/"):
            data = (ROOT / item["url"].split("/main/", 1)[1]).read_bytes()  # ещё не запушено — берём локальный
        else:
            data = fetch(item["url"])
        out.append({**item, "sha256": hashlib.sha256(data).hexdigest(), "size": len(data)})
        print(f'{item["id"]:24s} {len(data):7d} {out[-1]["sha256"][:16]}')
    legacy = [{k: v for k, v in it.items() if k != "title"} for it in out if it["id"] != "theme-eclipse"]
    for name, schema, items in (("manifest.json", 1, legacy), ("manifest2.json", 2, out)):
        text = json.dumps({"schema": schema, "items": items}, ensure_ascii=False, indent=2) + "\n"
        (ROOT / "content" / name).write_text(text, encoding="utf-8")

if __name__ == "__main__":
    main()
