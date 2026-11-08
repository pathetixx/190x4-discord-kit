// 190x4 Discord Kit — окно установщика.
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
const appWindow = window.__TAURI__.window.getCurrentWindow();

const $ = (id) => document.getElementById(id);
const term = $('term');
let busy = false;
let selectedTheme = null;
let chosenTheme = null;
let updateAvailable = null;

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
  for (const id of ['btn-install', 'btn-restart', 'btn-uninstall', 'btn-update']) $(id).disabled = on;
  updateThemeButton();
}

function showPage(name) {
  for (const b of document.querySelectorAll('.nav-item')) b.classList.toggle('active', b.dataset.page === name);
  for (const p of document.querySelectorAll('.page')) p.classList.toggle('active', p.id === `page-${name}`);
}

function updateThemeButton() {
  $('btn-theme').disabled = busy || !chosenTheme || chosenTheme === selectedTheme;
}

function renderThemes(themes) {
  const box = $('themes');
  box.textContent = '';
  for (const t of themes) {
    const card = document.createElement('button');
    card.className = 'theme';
    card.classList.toggle('selected', t.id === chosenTheme);
    card.addEventListener('click', () => {
      chosenTheme = t.id;
      renderThemes(themes);
      updateThemeButton();
    });

    const head = document.createElement('div');
    head.className = 'theme-head';
    const name = document.createElement('span');
    name.className = 'theme-name';
    name.textContent = t.title;
    const mark = document.createElement('span');
    mark.className = 'theme-mark';
    const enso = document.createElement('img');
    enso.src = 'img/enso.webp';
    enso.alt = '';
    mark.append(enso, t.id === selectedTheme ? 'Выбрана' : 'Будет выбрана');
    head.append(name, mark);

    const preview = document.createElement('img');
    preview.className = 'theme-preview';
    preview.src = `img/preview-${t.id}.webp`;
    preview.alt = `Как выглядит тема ${t.title}`;
    card.append(head, preview);
    box.appendChild(card);
  }
}

function renderPlugins(plugins) {
  const box = $('plugins');
  box.textContent = '';
  for (const p of plugins) {
    const row = document.createElement('li');
    const name = document.createElement('span');
    name.textContent = p.name;
    const state = document.createElement('span');
    state.textContent = p.present ? `v${p.version || '?'}` : 'не установлен';
    row.append(name, state);
    box.appendChild(row);
  }
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

  const themes = s.content.filter((c) => c.kind === 'theme');
  const plugins = s.content.filter((c) => c.kind === 'plugin');
  selectedTheme = s.selected_theme || (themes[0] && themes[0].id) || null;
  chosenTheme = selectedTheme;
  const theme = themes.find((t) => t.id === selectedTheme);
  if (s.content.length === 0) {
    setPart('theme', 'нет связи', null);
    setPart('plugins', 'нет связи', null);
  } else {
    setPart('theme', theme && theme.present ? `${theme.title} v${theme.version || '?'}` : 'не установлена', theme && theme.present ? true : null);
    const have = plugins.filter((p) => p.present);
    setPart('plugins', `${have.length}/${plugins.length}`, have.length === plugins.length ? true : null);
  }
  renderThemes(themes);
  renderPlugins(plugins);
  updateThemeButton();
  setPart('guard', s.guard ? 'включена' : 'выключена', s.guard ? true : null);
  $('guard-state').textContent = s.guard ? 'Защита включена' : 'Защита выключена';

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
  if (state === 'err') $('theme-hint').textContent = text;
});

const mb = (n) => (n / 1048576).toFixed(1).replace('.', ',');

listen('kit-update', (e) => {
  const { received, total } = e.payload;
  $('bar').hidden = false;
  $('update-size').hidden = false;
  if (total) {
    const pct = Math.min(100, Math.round((received / total) * 100));
    $('bar-fill').style.width = `${pct}%`;
    $('update-size').textContent = `${pct}%  ·  ${mb(received)} из ${mb(total)} МБ`;
  } else {
    $('update-size').textContent = `${mb(received)} МБ`;
  }
});

for (const b of document.querySelectorAll('.nav-item')) b.addEventListener('click', () => showPage(b.dataset.page));

$('btn-install').addEventListener('click', async () => {
  if (busy) return;
  setBusy(true);
  term.innerHTML = '';
  log('установка комплекта 190x4', 'muted');
  try {
    await invoke('install_all');
    log('готово, Discord запущен с темой', 'muted');
  } catch (e) {
    log('установка прервана, причина в строке выше', 'muted');
  }
  setBusy(false);
  refresh();
});

$('btn-theme').addEventListener('click', async () => {
  if (busy || !chosenTheme) return;
  setBusy(true);
  $('theme-hint').textContent = 'Переключаю тему…';
  try {
    await invoke('apply_theme', { id: chosenTheme });
    $('theme-hint').textContent = 'Готово, тема применена.';
  } catch (e) {
    $('theme-hint').textContent = `Не получилось: ${e}`;
  }
  setBusy(false);
  refresh();
});

$('btn-restart').addEventListener('click', async () => {
  if (busy) return;
  setBusy(true);
  log('перезапускаю Discord', 'muted');
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
  showPage('home');
  term.innerHTML = '';
  log('удаление комплекта 190x4', 'muted');
  try {
    await invoke('uninstall_all');
    log('комплект удалён, Discord снова обычный', 'muted');
  } catch (e) {
    log('удаление прервано, причина в строке выше', 'muted');
  }
  setBusy(false);
  refresh();
});

$('btn-min').addEventListener('click', () => appWindow.minimize());
$('btn-close').addEventListener('click', () => appWindow.close());

function setUpdateButton(text) {
  $('btn-update').textContent = text;
}

async function checkUpdate(manual) {
  const text = $('update-text');
  $('bar').hidden = true;
  $('update-size').hidden = true;
  updateAvailable = null;
  setUpdateButton('Проверить обновления');
  text.textContent = 'Проверяю обновления…';
  try {
    const upd = await invoke('check_app_update');
    if (upd) {
      updateAvailable = upd;
      text.textContent = `Доступно обновление ${upd.version}`;
      setUpdateButton('Установить');
      return;
    }
    if (!manual) {
      text.textContent = 'Установлена последняя версия';
      return;
    }
    const names = await invoke('sync_content');
    text.textContent = names.length
      ? `Обновлено: ${names.join(', ')}. Перезапусти Discord`
      : 'Всё уже обновлено';
    refresh();
  } catch (e) {
    text.textContent = 'Нет связи с сервером обновлений';
  }
}

async function installUpdate() {
  $('update-text').textContent = `Обновление ${updateAvailable.version} загружается`;
  $('bar').hidden = false;
  $('bar-fill').style.width = '0%';
  try {
    await invoke('install_app_update');
  } catch (e) {
    $('update-text').textContent = 'Обновление не удалось';
    $('bar').hidden = true;
    $('update-size').hidden = true;
    log(`обновление не удалось: ${e}`, 'err');
  }
}

$('btn-update').addEventListener('click', async () => {
  if (busy) return;
  setBusy(true);
  if (updateAvailable) await installUpdate();
  else await checkUpdate(true);
  setBusy(false);
});

refresh();
checkUpdate(false);
