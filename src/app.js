// 190x4 Discord Kit — окно установщика.
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const appWindow = window.__TAURI__.window.getCurrentWindow();

const $ = (id) => document.getElementById(id);
const term = $('term');
let busy = false;

function log(text, kind = '') {
  const line = document.createElement('div');
  line.className = `term-line ${kind}`;
  line.textContent = text;
  term.appendChild(line);
  term.scrollTop = term.scrollHeight;
}

function setPart(name, state, ok) {
  const row = document.querySelector(`[data-part="${name}"]`);
  row.classList.toggle('ok', ok === true);
  row.classList.toggle('warn', ok === false);
  row.querySelector('.pstate').textContent = state;
}

function setBusy(on) {
  busy = on;
  for (const id of ['btn-install', 'btn-restart', 'btn-uninstall']) $(id).disabled = on;
}

async function refresh() {
  let s;
  try {
    s = await invoke('get_status');
  } catch (e) {
    log(`не удалось проверить состояние: ${e}`, 'err');
    return;
  }
  $('ver').textContent = `v${s.app_version}`;

  if (s.discord.length === 0) {
    setPart('discord', 'не найден', false);
  } else {
    setPart('discord', s.discord.map((d) => d.title.replace('Discord', '').trim() || 'Stable').join(', '), true);
  }
  const injected = s.discord.length > 0 && s.discord.every((d) => d.injected);
  if (!s.bd_installed) setPart('bd', 'не установлен', null);
  else setPart('bd', injected ? (s.bd_version || 'установлен') : 'отключён обновлением', injected);

  const theme = s.content.find((c) => c.kind === 'theme');
  const plugins = s.content.filter((c) => c.kind === 'plugin');
  if (s.content.length === 0) {
    setPart('theme', 'нет связи', null);
    setPart('plugins', 'нет связи', null);
  } else {
    setPart('theme', theme && theme.present ? `v${theme.version || '?'}` : 'не установлена', theme && theme.present ? true : null);
    const have = plugins.filter((p) => p.present);
    setPart('plugins', `${have.length}/${plugins.length}`, have.length === plugins.length ? true : null);
    document.querySelector('[data-part="plugins"]').title = plugins.map((p) => p.name).join(', ');
  }
  setPart('guard', s.guard ? 'включена' : 'выключена', s.guard ? true : null);

  const installed = s.bd_installed && injected && theme && theme.present && s.guard;
  $('btn-install').textContent = installed ? 'ПЕРЕУСТАНОВИТЬ' : 'УСТАНОВИТЬ ВСЁ';
  $('btn-install').disabled = busy || s.discord.length === 0;
  $('install-hint').textContent = s.discord.length === 0
    ? 'Сначала установи Discord с discord.com и запусти его.'
    : 'Discord закроется и запустится заново.';
  $('btn-uninstall').hidden = !s.bd_installed && !s.guard;
}

listen('kit-progress', (e) => {
  const { state, text } = e.payload;
  // строка «run» потом заменяется результатом того же шага
  const last = term.lastElementChild;
  if (state !== 'run' && last && last.classList.contains('run')) last.remove();
  log(text, state);
});

$('btn-install').addEventListener('click', async () => {
  if (busy) return;
  setBusy(true);
  term.innerHTML = '';
  log('> установка комплекта 190x4', 'muted');
  try {
    await invoke('install_all');
    log('> готово. Discord запущен с темой 190x4.', 'muted');
  } catch (e) {
    log('> установка прервана — смотри строку выше.', 'muted');
  }
  setBusy(false);
  refresh();
});

$('btn-restart').addEventListener('click', async () => {
  if (busy) return;
  setBusy(true);
  log('> перезапускаю Discord', 'muted');
  try { await invoke('restart_discord'); } catch (e) { log(String(e), 'err'); }
  setBusy(false);
});

let uninstallArmed = null;
$('btn-uninstall').addEventListener('click', async () => {
  if (busy) return;
  const btn = $('btn-uninstall');
  if (!uninstallArmed) {
    btn.textContent = 'Точно удалить? Нажми ещё раз';
    btn.classList.add('armed');
    uninstallArmed = setTimeout(() => {
      btn.textContent = 'Удалить комплект';
      btn.classList.remove('armed');
      uninstallArmed = null;
    }, 4000);
    return;
  }
  clearTimeout(uninstallArmed);
  uninstallArmed = null;
  btn.textContent = 'Удалить комплект';
  btn.classList.remove('armed');
  setBusy(true);
  term.innerHTML = '';
  log('> удаление комплекта 190x4', 'muted');
  try {
    await invoke('uninstall_all');
    log('> комплект удалён, Discord снова обычный.', 'muted');
  } catch (e) {
    log('> удаление прервано — смотри строку выше.', 'muted');
  }
  setBusy(false);
  refresh();
});

$('btn-min').addEventListener('click', () => appWindow.minimize());
$('btn-close').addEventListener('click', () => appWindow.close());

async function checkUpdate() {
  const box = $('update');
  try {
    const upd = await invoke('check_app_update');
    if (!upd) {
      box.textContent = 'последняя версия';
      return;
    }
    box.textContent = `доступна v${upd.version}`;
    const btn = document.createElement('button');
    btn.textContent = 'Обновить';
    btn.addEventListener('click', async () => {
      btn.disabled = true;
      btn.textContent = 'Скачиваю…';
      try {
        await invoke('install_app_update');
      } catch (e) {
        btn.textContent = 'Ошибка';
        log(`обновление не удалось: ${e}`, 'err');
      }
    });
    box.appendChild(btn);
  } catch (e) {
    box.textContent = 'нет связи с сервером обновлений';
  }
}

refresh();
checkUpdate();
