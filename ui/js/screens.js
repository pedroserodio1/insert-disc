// Telas por estado (FRONTEND-DESIGN §6). Cada construtor devolve:
//   { el, items:[{el, run, left?, right?}], index, hints:[{btn,text,primary,fn}], back, onNav, hold, destroy }
// Estados, ações e destinos vêm do núcleo (UX-STATES); aqui só a aparência.

import { h, bgImage, disc, setDiscProgress, sheet } from './components.js';
import { createShelf } from './shelf.js';
import { getLocale, t } from './i18n.js';

const spineColor = (c) => (/^#[0-9a-f]{6}$/i.test(c || '') ? c : null);
const act = (ctx, id) => () => ctx.send({ type: 'action', id });
const back = (ctx) => () => ctx.send({ type: 'back' });

function headEl(title, body, { error = false } = {}) {
  return h('div', { class: 'head' }, h('h1', { class: 'tipo-titulo' }, title), body && h('p', { class: `tipo-corpo${error ? ' error' : ''}` }, body));
}

function makeList(defs, { scroll = false, className = '' } = {}) {
  const items = [];
  const el = h('div', { class: `list${scroll ? ' scroll' : ''} ${className}` });
  for (const d of defs) {
    if (d.section) { el.append(h('div', { class: 'item section tipo-apoio' }, d.section)); continue; }
    const b = h('button', { class: `item focusable tipo-corpo${d.destructive ? ' destructive' : ''}` }, h('span', { class: 'lbl' }, d.lead, d.label), d.value != null && h('span', { class: 'value tipo-apoio' }, d.value));
    items.push({ el: b, run: d.run, left: d.left, right: d.right, initial: d.initial });
    el.append(b);
  }
  return { el, items };
}

function initialIndex(items) {
  const i = items.findIndex((x) => x.initial);
  return i >= 0 ? i : 0;
}

const actionLabel = (snap, id) => {
  const [kind, arg] = id.split(':');
  switch (kind) {
    case 'play_other': return t('action.play_other', { other: snap.other_game?.name ?? '' });
    case 'burn_another': return t('options.burn_another');
    case 'remove_game': return t('options.remove');
    case 'unlink_disc': return `${t('options.unlink_disc')}: ${snap.game_discs.find((d) => d.disc_id === arg)?.label ?? ''}`;
    case 'start_empty': return t('catalog.start_empty');
    case 'restore_backup': return t('catalog.restore', { n: Number(arg) + 1 });
    default: return t(`action.${kind}`);
  }
};

function actionDefs(snap, ctx) {
  return snap.actions.map((a) => ({ label: actionLabel(snap, a.id), run: act(ctx, a.id), initial: a.default, destructive: a.id === 'remove_game' || a.id.startsWith('unlink_disc') }));
}

const hintsBackSelect = (ctx) => [
  { btn: 'accept', text: t('action.select'), primary: true, fn: () => ctx.accept() },
  { btn: 'back', text: t('action.back'), fn: back(ctx) },
];

// ---------- BOOT ----------
export function boot() {
  return { el: h('div', { class: 'screen' }, h('div', { class: 'splash' }, h('div', { class: 'tipo-marca' }, 'Insert Disc'))), items: [], index: 0, hints: [], back: null };
}

// ---------- LIBRARY ----------
export function library(snap, ctx) {
  const games = snap.library;
  const head = h('div', { class: 'head' });
  const el = h('div', { class: 'screen' }, head);
  const items = [];
  let shelf = null;

  const focused = () => games[shelf?.focusedIndex() ?? 0];
  function fillHead() {
    head.replaceChildren();
    if (!games.length) {
      head.append(h('h1', { class: 'tipo-titulo' }, t('library.empty.title')), h('p', { class: 'tipo-corpo' }, t('library.empty.body')));
      return;
    }
    const g = focused();
    head.append(h('h1', { class: 'tipo-titulo' }, g.name), h('p', { class: 'tipo-apoio' }, t(`library.kind.${g.kind}`)), h('p', { class: 'tipo-apoio' }, g.disc_count ? t('library.disc_count', { n: g.disc_count }) : t('library.no_discs')));
  }

  if (games.length) {
    const startId = ctx.ui.libFocusId;
    const start = Math.max(0, games.findIndex((g) => g.game_id === startId));
    shelf = createShelf({
      games,
      focus: start,
      onFocus: (i) => { shelf.setFocus(i); ctx.ui.libFocusId = focused().game_id; fillHead(); },
      onAccept: () => ctx.send({ type: 'select', game_id: focused().game_id }),
    });
    el.append(shelf.el);
    ctx.ui.libFocusId = games[start].game_id;
  } else {
    const add = h('button', { class: 'item focusable tipo-corpo', style: 'margin-top:calc(var(--u)*3);display:inline-flex' }, t('action.add_game'));
    el.append(add);
    items.push({ el: add, run: () => ctx.send({ type: 'add_game' }) });
    el.append(h('div', { class: 'shelf' }, h('div', { class: 'floor' })));
  }
  fillHead();

  const jump = (dir) => {
    if (!shelf) return;
    const first = (g) => (g.name[0] || '').toLocaleUpperCase(getLocale());
    const i = shelf.focusedIndex();
    const cur = first(games[i]);
    let j = i;
    if (dir > 0) { while (j < games.length - 1 && first(games[j]) === cur) j++; }
    else { while (j > 0 && first(games[j]) === cur) j--; const c = first(games[j]); while (j > 0 && first(games[j - 1]) === c) j--; }
    shelf.setFocus(j); ctx.ui.libFocusId = focused().game_id; fillHead();
  };

  const move = (d, repeat) => {
    if (!shelf) return true;
    shelf.setFocus(shelf.focusedIndex() + d, { fast: repeat });
    ctx.ui.libFocusId = focused().game_id;
    fillHead();
    return true;
  };

  return {
    el, items, index: 0,
    hints: games.length
      ? [
        { btn: 'accept', text: t('action.play'), primary: true, fn: () => ctx.send({ type: 'select', game_id: focused().game_id }) },
        { btn: 'x', text: t('action.add_game'), fn: () => ctx.send({ type: 'add_game' }) },
        { btn: 'y', text: t('action.options'), fn: () => ctx.send({ type: 'options', game_id: focused().game_id }) },
        { btn: 'menu', text: t('action.settings'), fn: () => ctx.send({ type: 'settings' }) },
      ]
      : [{ btn: 'accept', text: t('action.add_game'), primary: true, fn: () => ctx.send({ type: 'add_game' }) }, { btn: 'menu', text: t('action.settings'), fn: () => ctx.send({ type: 'settings' }) }],
    back: null,
    onNav(name, repeat) {
      if (name === 'left') return move(-1, repeat);
      if (name === 'right') return move(1, repeat);
      if (name === 'up' || name === 'down') return true;
      if (name === 'x') { ctx.send({ type: 'add_game' }); return true; }
      if (name === 'y' && shelf) { ctx.send({ type: 'options', game_id: focused().game_id }); return true; }
      if (name === 'menu') { ctx.send({ type: 'settings' }); return true; }
      if (name === 'lb') { jump(-1); return true; }
      if (name === 'rb') { jump(1); return true; }
      return false;
    },
    acceptDefault: () => shelf && ctx.send({ type: 'select', game_id: focused().game_id }),
    update(next) {
      shelf?.update(next.library, ctx.ui.libFocusId);
      if (next.focus_hint) {
        const i = next.library.findIndex((g) => g.game_id === next.focus_hint);
        if (i >= 0) { shelf.setFocus(i); ctx.ui.libFocusId = next.focus_hint; }
      }
      games.splice(0, games.length, ...next.library);
      fillHead();
    },
    destroy() { shelf?.destroy(); },
  };
}

// ---------- cena da caixa aberta ----------
function caseBox(game, verso, disc_) {
  const c = spineColor(game?.spine_color);
  const outer = h('div', { class: 'cover-outer' });
  if (c) outer.style.setProperty('--spine', c);
  if (game?.cover) bgImage(outer, game.cover);
  return h('div', { class: 'casebox' },
    h('div', { class: 'cover' }, h('div', { class: 'cover-inner' }, h('span', { class: 'tipo-secao' }, verso)), outer),
    h('div', { class: 'tray' }, disc_));
}

function discFor(snap, mode) {
  const g = snap.game;
  switch (mode) {
    case 'empty': return h('div', {});
    case 'dashed': return h('div', { style: 'display:contents' }, h('div', { class: 'slot-outline' }), h('div', { class: 'pin' }));
    case 'reading': return disc({ cover: g?.cover, name: g?.name, reading: true });
    case 'reading-still': return disc({ cover: g?.cover, name: g?.name, readingStill: true });
    case 'label': return disc({ cover: g?.cover, name: g?.name });
    default: return disc(mode);
  }
}

function rejectedDisc(snap) {
  const cls = snap.reason || '';
  if (cls === 'OTHER_GAME') return disc({ cover: snap.other_game?.cover, name: snap.other_game?.name, rejected: true });
  if (cls === 'BLANK') return disc({ variant: 'virgin', rejected: true });
  if (cls === 'READ_ERROR') return disc({ variant: 'unreadable', rejected: true });
  return disc({ variant: 'generic', rejected: true });
}

function caseScene(snap, ctx, { title, body, tray, verso, withSheet = false, actions = true }) {
  const g = snap.game;
  const el = h('div', { class: 'screen' }, headEl(title, body), caseBox(g, verso ?? g?.name ?? t('reg.case_label'), tray));
  if (withSheet) {
    const s = sheet(snap, ctx.device());
    if (s) el.append(h('div', { class: 'sheet-wrap' }, s));
  }
  let items = [];
  if (actions && snap.actions.length) {
    const list = makeList(actionDefs(snap, ctx));
    el.append(h('div', { class: 'side-actions' }, list.el));
    items = list.items;
  }
  return { el, items, index: initialIndex(items), hints: hintsBackSelect(ctx), back: back(ctx) };
}

export function noDiscYet(snap, ctx) {
  return caseScene(snap, ctx, { title: t('nodisc.title', { game: snap.game.name }), body: t('nodisc.body'), tray: discFor(snap, 'empty') });
}

export function waitingDisc(snap, ctx) {
  const tray = { tray_opening: 'wait.tray_opening', tray_manual: 'wait.tray_manual', tray_failed: 'wait.tray_failed' }[snap.reason];
  return caseScene(snap, ctx, { title: t('wait.title', { game: snap.game.name }), body: t(tray), tray: discFor(snap, 'dashed') });
}

export function reading(snap, ctx) {
  return caseScene(snap, ctx, { title: t('reading.title'), tray: discFor(snap, 'reading') });
}

export function identified(snap, ctx) {
  return caseScene(snap, ctx, { title: t('reading.title'), tray: discFor(snap, 'reading-still'), withSheet: true });
}

export function rejected(snap, ctx) {
  const key = `reject.${snap.reason}`;
  return caseScene(snap, ctx, {
    title: t(key, { other: snap.other_game?.name ?? '', game: snap.game?.name ?? '' }),
    tray: rejectedDisc(snap), withSheet: true,
  });
}

export function regInsert(snap, ctx) {
  const body = { tray_opening: 'wait.tray_opening', tray_manual: 'wait.tray_manual', tray_failed: 'wait.tray_failed' }[snap.reason];
  return caseScene(snap, ctx, { title: t('reg.insert'), body: t(body), tray: discFor(snap, 'dashed'), verso: snap.game?.name ?? t('reg.case_label') });
}

export function regRejected(snap, ctx) {
  const disc_ = snap.reason === 'AUDIO' ? disc({ variant: 'generic', rejected: true }) : snap.reason === 'READ_ERROR' ? disc({ variant: 'unreadable', rejected: true }) : disc({ variant: 'generic', rejected: true });
  return caseScene(snap, ctx, { title: t(`reg.reject.${snap.reason}`), tray: disc_, withSheet: true, verso: snap.game?.name ?? t('reg.case_label') });
}

export function burnFailed(snap, ctx) {
  const virgin = snap.reason === 'write_error' || snap.reason === 'erase_error' ? disc({ variant: 'virgin', rejected: true }) : disc({ variant: 'unreadable', rejected: true });
  return caseScene(snap, ctx, { title: t(`burn.failed.${snap.reason}`), tray: virgin, verso: snap.game?.name ?? t('reg.case_label') });
}

// ---------- diálogos ----------
function dialogScreen(snap, ctx, { title, body, withSheet, buttons, backFn }) {
  const items = [];
  const btns = h('div', { class: 'btns' });
  for (const b of buttons) {
    const el = h('button', { class: `btn focusable tipo-corpo ${b.kind || ''}` }, b.ring && h('span', { class: 'hold-ring' }), b.label);
    items.push({ el, run: b.run, initial: b.initial });
    btns.append(el);
  }
  const dlg = h('div', { class: 'dialog' }, h('h2', { class: 'tipo-secao' }, title), body && h('p', { class: 'tipo-corpo' }, body), withSheet ? sheet(snap, ctx.device()) : null, btns);
  const el = h('div', { class: 'screen overlay' }, h('div', { class: 'veil' }, dlg));
  return { el, items, index: initialIndex(items), hints: hintsBackSelect(ctx), back: backFn ?? back(ctx), dialog: true, holdRing: btns.querySelector('.hold-ring') };
}

export function adoptConfirm(snap, ctx) {
  return dialogScreen(snap, ctx, {
    title: t('adopt.title', { game: snap.game.name }),
    body: t('adopt.body', { name: snap.media?.ini_name ?? '', game: snap.game.name }),
    buttons: [
      { label: t('action.back'), kind: 'secondary', run: back(ctx) },
      { label: t('adopt.confirm'), run: act(ctx, 'confirm'), initial: true },
    ],
  });
}

export function removeConfirm(snap, ctx) {
  return dialogScreen(snap, ctx, {
    title: t('remove.title', { game: snap.game.name }), body: t('remove.body'),
    buttons: [
      { label: t('action.back'), kind: 'secondary', run: back(ctx), initial: true },
      { label: t('remove.confirm'), kind: 'danger', run: act(ctx, 'confirm') },
    ],
  });
}

export function regCdrWarning(snap, ctx) {
  return dialogScreen(snap, ctx, {
    title: t('reg.cdr_warning.title'), body: t('reg.cdr_warning.body'),
    buttons: [
      { label: t('action.back'), kind: 'secondary', run: back(ctx), initial: true },
      { label: t('action.burn'), run: act(ctx, 'burn') },
    ],
  });
}

export function regEraseConfirm(snap, ctx) {
  const step2 = snap.actions.length === 0;
  if (!step2) {
    return dialogScreen(snap, ctx, {
      title: t('erase.title'), body: t('erase.body'), withSheet: true,
      buttons: [
        { label: t('action.back'), kind: 'secondary', run: back(ctx), initial: true },
        { label: t('action.erase'), kind: 'danger', run: act(ctx, 'confirm') },
      ],
    });
  }
  const s = dialogScreen(snap, ctx, {
    title: t('erase.title'), body: t('erase.body'), withSheet: true,
    buttons: [
      { label: t('action.back'), kind: 'secondary', run: back(ctx), initial: true },
      { label: t('erase.hold'), kind: 'danger', ring: true, run: () => {}, hold: true },
    ],
  });
  s.holdIndex = 1; // segurar o "aceitar" sobre este botão (5.7)
  return s;
}

// ---------- cenas do disco ----------
function discScene(snap, ctx, { discEl, caption, progress, extra, items = [], index = 0, hints = [], backFn = null }) {
  const el = h('div', { class: 'screen' }, h('div', { class: 'disc-scene' }, discEl), caption && h('div', { class: 'caption' }, h('h1', { class: 'tipo-secao' }, caption)));
  if (progress != null) el.querySelector('.disc-scene').append(h('div', { class: 'progress' }, h('div', { class: 'num' }, `${progress}%`)));
  if (extra) el.append(extra);
  return { el, items, index, hints, back: backFn, discEl };
}

export function launching(snap, ctx) {
  const min = snap.timing?.min_ms ?? 0;
  const animated = min > 0;
  const d = disc({ cover: snap.game.cover, name: snap.game.name, enter: animated, spinning: animated });
  if (animated) d.style.setProperty('--enter-ms', `${Math.max(120, Math.min(800, min * 0.9))}ms`);
  const fade = h('div', { class: 'launch-fade' });
  const s = discScene(snap, ctx, { discEl: d, caption: t('launch.title', { game: snap.game.name }), extra: fade });
  const timer = animated ? setTimeout(() => fade.classList.add('on'), Math.max(0, min - 500)) : 0;
  s.destroy = () => clearTimeout(timer);
  return s;
}

export function launchError(snap, ctx) {
  const list = makeList(actionDefs(snap, ctx));
  const s = discScene(snap, ctx, {
    discEl: disc({ cover: snap.game.cover, name: snap.game.name, rejected: true, stopped: true }),
    caption: t(`launch.error.${snap.reason}`, { game: snap.game.name }),
    items: list.items, hints: [{ btn: 'accept', text: t('action.back_to_shelf'), primary: true, fn: () => ctx.accept() }],
    backFn: back(ctx),
  });
  s.el.append(h('div', { class: 'side-actions', style: 'left:5vw;top:auto;bottom:calc(5vh + var(--s)*70)' }, list.el));
  s.el.querySelector('.caption').style.bottom = 'calc(5vh + var(--s)*150)';
  return s;
}

export function progressScene(snap, ctx) {
  const st = snap.state;
  const erasing = st === 'ERASING';
  const d = disc({ variant: st === 'VERIFYING' ? 'generic' : 'virgin', burning: !erasing, erasing, reading: st === 'VERIFYING' || st === 'REG_READING', progress: erasing ? 100 - (snap.progress ?? 0) : snap.progress ?? 0 });
  const caption = { ERASING: t('erase.progress'), BURNING: t('burn.progress'), VERIFYING: t('burn.verifying'), REG_READING: t('reg.reading') }[st];
  return discScene(snap, ctx, { discEl: d, caption, progress: st === 'BURNING' || st === 'ERASING' ? snap.progress ?? 0 : null });
}

export function burnDone(snap, ctx) {
  const list = makeList(actionDefs(snap, ctx));
  const s = discScene(snap, ctx, {
    discEl: disc({ cover: snap.game.cover, name: snap.game.name }),
    caption: t('burn.done.title'), items: list.items, hints: [{ btn: 'accept', text: t('action.done'), primary: true, fn: () => ctx.accept() }], backFn: back(ctx),
  });
  s.el.querySelector('.caption').append(h('p', { class: 'tipo-corpo', style: 'color:var(--policarbonato)' }, t('burn.done.body')));
  s.el.querySelector('.caption').style.bottom = 'calc(5vh + var(--s)*150)';
  s.el.append(h('div', { class: 'side-actions', style: 'left:5vw;top:auto;bottom:calc(5vh + var(--s)*70)' }, list.el));
  return s;
}

export function regLabelPreview(snap, ctx) {
  const d = disc({ variant: 'virgin' });
  const input = h('input', { type: 'text', maxlength: '15', value: snap.label ?? '', 'aria-label': t('reg.label.title'), autocomplete: 'off', spellcheck: 'false' });
  const field = h('div', { class: 'field' }, h('label', { class: 'tipo-apoio' }, t('reg.label.title')), input, h('span', { class: 'tipo-apoio', style: 'color:var(--grafite)' }, t('reg.label.body')));
  const form = h('div', { class: 'form', style: 'margin-top:0;padding:calc(var(--u)*3)' }, field);
  const wrap = h('div', { class: 'label-edit' }, form);
  const s = discScene(snap, ctx, { discEl: d, caption: t('reg.label.title'), extra: wrap });
  s.el.querySelector('.caption').style.bottom = 'calc(5vh + var(--s)*300)';
  wrap.style.bottom = 'calc(5vh + var(--s)*70)';
  const go = async () => {
    if (input.value !== (snap.label ?? '')) await ctx.send({ type: 'label_edit', text: input.value });
    ctx.send({ type: 'action', id: 'continue' });
  };
  input.addEventListener('keydown', (e) => { if (e.key === 'Enter') { e.preventDefault(); go(); } });
  const item = { el: h('button', { class: 'hint tipo-apoio', style: 'display:none' }), run: go };
  return { ...s, items: [item], index: 0, hints: [{ btn: 'accept', text: t('action.continue'), primary: true, fn: go }, { btn: 'back', text: t('action.back'), fn: back(ctx) }], back: back(ctx), focusInput: input };
}

// ---------- listas ----------
export function regChooseGame(snap, ctx) {
  const defs = [
    { label: t('reg.new_steam'), run: () => ctx.openForm('steam'), initial: true },
    { label: t('reg.new_custom'), run: () => ctx.openForm('custom') },
    ...snap.library.map((g) => ({
      label: g.name,
      lead: h('span', { class: 'mini-spine', style: spineColor(g.spine_color) ? `background:${g.spine_color}` : 'background:var(--policarbonato)' }),
      value: g.disc_count ? t('library.disc_count', { n: g.disc_count }) : '',
      run: () => ctx.send({ type: 'choose_game', game_id: g.game_id }),
    })),
  ];
  const list = makeList(defs, { scroll: true });
  return { el: h('div', { class: 'screen' }, headEl(t('reg.choose')), list.el), items: list.items, index: 0, hints: hintsBackSelect(ctx), back: back(ctx) };
}

export function gameOptions(snap, ctx) {
  const g = snap.game;
  const defs = [
    { label: t('options.edit'), run: () => ctx.openForm('edit'), initial: true },
    ...actionDefs(snap, ctx).map((d) => ({ ...d, initial: false })),
  ];
  const list = makeList(defs, { scroll: true });
  const discs = h('div', { class: 'tipo-apoio', style: 'margin-top:var(--u);color:var(--policarbonato)' },
    snap.game_discs.length
      ? snap.game_discs.map((d) => h('div', {}, `${d.label} · ${t(`disc.origin.${d.origin}`)}`))
      : t('options.no_discs'));
  const head = headEl(g.name, null);
  head.append(h('p', { class: 'tipo-apoio' }, t(`library.kind.${g.kind}`)), discs);
  return { el: h('div', { class: 'screen' }, head, list.el), items: list.items, index: 0, hints: hintsBackSelect(ctx), back: back(ctx) };
}

const LOAD_STEPS = [0, 1, 2, 3, 5, 8, 10];

export function settings(snap, ctx) {
  const st = snap.settings;
  const set = (key, value) => ctx.send({ type: 'set_setting', key, value });
  const cycle = (vals, cur, d) => vals[(vals.indexOf(cur) + d + vals.length) % vals.length];
  const langs = [null, 'pt-BR', 'en'];
  const langName = (l) => (l === 'pt-BR' ? 'Português (Brasil)' : l === 'en' ? 'English' : t('settings.language.system'));
  const insert = ['focus', 'launch'];
  const secs = st.loading_min_ms / 1000;
  const defs = [
    { label: t('settings.drive'), value: st.drive ?? (snap.drive === 'ok' ? 'auto' : t('common.none')), run: () => {} },
    { label: t('settings.on_insert'), value: t(`settings.on_insert.${st.on_disc_insert}`), run: () => set('on_disc_insert', cycle(insert, st.on_disc_insert, 1)), left: () => set('on_disc_insert', cycle(insert, st.on_disc_insert, -1)), right: () => set('on_disc_insert', cycle(insert, st.on_disc_insert, 1)) },
    { label: t('settings.loading_min'), value: t('settings.seconds', { n: secs }), run: () => set('loading_min_ms', cycle(LOAD_STEPS, secs, 1) * 1000), left: () => set('loading_min_ms', cycle(LOAD_STEPS, secs, -1) * 1000), right: () => set('loading_min_ms', cycle(LOAD_STEPS, secs, 1) * 1000) },
    { label: t('settings.language'), value: langName(st.locale), run: () => set('locale', cycle(langs, st.locale, 1)), left: () => set('locale', cycle(langs, st.locale, -1)), right: () => set('locale', cycle(langs, st.locale, 1)) },
    { label: t('settings.fullscreen'), value: st.window_mode === 'fullscreen' ? t('common.on') : t('common.off'), run: () => { const on = st.window_mode !== 'fullscreen'; set('window_mode', on ? 'fullscreen' : 'windowed'); ctx.applyFullscreen(on); }, left: null, right: null },
    { label: t('settings.online_covers'), value: st.covers_online_enabled ? t('common.on') : t('common.off'), run: () => set('covers_online', !st.covers_online_enabled) },
    { label: t('settings.export'), run: () => ctx.exportCatalog() },
    { label: t('settings.import'), run: () => ctx.importCatalog() },
  ];
  const list = makeList(defs);
  return { el: h('div', { class: 'screen' }, headEl(t('settings.title')), list.el), items: list.items, index: ctx.ui.settingsIndex ?? 0, hints: [{ btn: 'accept', text: t('action.select'), primary: true, fn: () => ctx.accept() }, { btn: 'back', text: t('action.back'), fn: back(ctx) }], back: back(ctx), keepIndex: 'settingsIndex' };
}

export function driveProblem(snap, ctx) {
  const key = { no_drive: 'none', removed: 'removed', cannot_burn: 'cannot_burn' }[snap.reason];
  const list = makeList(actionDefs(snap, ctx));
  return { el: h('div', { class: 'screen' }, headEl(t(`drive.${key}.title`), t(`drive.${key}.body`)), list.el), items: list.items, index: 0, hints: hintsBackSelect(ctx), back: back(ctx) };
}

export function catalogError(snap, ctx) {
  const list = makeList(actionDefs(snap, ctx));
  return { el: h('div', { class: 'screen' }, headEl(t('catalog.error.title'), t('catalog.error.body')), list.el), items: list.items, index: initialIndex(list.items), hints: [{ btn: 'accept', text: t('action.select'), primary: true, fn: () => ctx.accept() }], back: null };
}

// ---------- formulários (locais à UI; enviam create_game / rename_game) ----------
export function form(kind, snap, ctx) {
  const field = (id, label, value = '', multiline = false) => {
    const input = multiline ? h('textarea', { id, rows: '3' }) : h('input', { id, type: 'text', autocomplete: 'off', spellcheck: 'false' });
    input.value = value;
    return { input, el: h('div', { class: 'field' }, h('label', { class: 'tipo-apoio', for: id }, label), input) };
  };
  const editing = kind === 'edit';
  const name = field('f-name', t('form.name'), editing ? snap.game.name : '');
  const fields = [name];
  let appId, exe, args, wd;
  if (kind === 'steam') { appId = field('f-app', t('form.app_id')); fields.push(appId); }
  if (kind === 'custom') {
    exe = field('f-exe', t('form.executable')); args = field('f-args', t('form.args'), '', true); wd = field('f-wd', t('form.workdir'));
    fields.push(exe, args, wd);
  }
  const err = h('div', { class: 'err tipo-apoio', role: 'alert' });
  const title = { steam: 'form.steam.title', custom: 'form.custom.title', edit: 'form.edit.title' }[kind];

  const submit = async () => {
    err.textContent = '';
    const nm = name.input.value.trim();
    if (!nm) { err.textContent = t('form.error.name'); return; }
    let res;
    if (editing) res = await ctx.send({ type: 'rename_game', game_id: snap.game.game_id, name: nm });
    else if (kind === 'steam') {
      const id = Number(appId.input.value.trim());
      if (!Number.isInteger(id) || id <= 0) { err.textContent = t('form.error.app_id'); return; }
      res = await ctx.send({ type: 'create_game', name: nm, kind: 'steam', app_id: id });
    } else {
      res = await ctx.send({ type: 'create_game', name: nm, kind: 'custom', executable: exe.input.value.trim(), args: args.input.value.split('\n').map((l) => l.trim()).filter(Boolean), working_dir: wd.input.value.trim() || undefined });
    }
    if (res?.invalid) err.textContent = t('form.error.invalid', { reason: res.invalid });
    else ctx.closeForm();
  };
  const save = h('button', { class: 'btn focusable tipo-corpo' }, t('action.save'));
  const cancel = h('button', { class: 'btn secondary focusable tipo-corpo' }, t('action.cancel'));
  const items = [
    ...fields.map((f) => ({ el: f.input, run: () => f.input.focus(), isField: true })),
    { el: save, run: submit }, { el: cancel, run: () => ctx.closeForm() },
  ];
  const el = h('div', { class: 'screen' }, headEl(t(title)), h('div', { class: 'form' }, fields.map((f) => f.el), err, h('div', { class: 'btns', style: 'display:flex;gap:calc(var(--u)*2)' }, save, cancel)));
  for (const f of fields) f.input.addEventListener('keydown', (e) => { if (e.key === 'Enter' && f.input.tagName === 'INPUT') { e.preventDefault(); submit(); } });
  return { el, items, index: 0, hints: [{ btn: 'accept', text: t('action.select'), primary: true, fn: () => ctx.accept() }, { btn: 'back', text: t('action.cancel'), fn: () => ctx.closeForm() }], back: () => ctx.closeForm(), isForm: true, focusInput: name.input };
}

export { setDiscProgress };
