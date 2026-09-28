// Componentes de UI. Todo texto entra por createTextNode: nada vindo do disco ou do catálogo
// vira HTML (SECURITY R7, FRONTEND-DESIGN §10).

import { t } from './i18n.js';

export function h(tag, props = {}, ...kids) {
  const el = document.createElement(tag);
  for (const [k, v] of Object.entries(props || {})) {
    if (v == null || v === false) continue;
    if (k === 'class') el.className = v;
    else if (k === 'style') el.style.cssText = v;
    else if (k === 'dataset') Object.assign(el.dataset, v);
    else if (k.startsWith('on')) el.addEventListener(k.slice(2), v);
    else el.setAttribute(k, v === true ? '' : v);
  }
  for (const kid of kids.flat(Infinity)) if (kid != null && kid !== false) el.append(kid.nodeType ? kid : document.createTextNode(String(kid)));
  return el;
}

const GLYPHS = {
  xbox: { accept: 'A', back: 'B', x: 'X', y: 'Y', lb: 'LB', rb: 'RB', menu: '≡' },
  playstation: { accept: '✕', back: '○', x: '□', y: '△', lb: 'L1', rb: 'R1', menu: '≡' },
  keys: { accept: 'Enter', back: 'Esc', x: 'N', y: 'O', lb: 'PgUp', rb: 'PgDn', menu: 'M' },
};

export function glyph(btn, device, primary = false) {
  const set = GLYPHS[device] || GLYPHS.xbox;
  return h('span', { class: `glyph${device === 'keys' ? ' key' : ''}${primary ? ' primary' : ''}`, 'aria-hidden': 'true' }, set[btn] || btn);
}

/** Dica de botão clicável: glifo + verbo. */
export function hint(btn, text, device, { primary = false, onClick } = {}) {
  return h('button', { class: `hint tipo-apoio${onClick ? ' clickable' : ''}`, onclick: onClick }, glyph(btn, device, primary), text);
}

export function bar(hints) {
  return h('div', { class: 'bar' }, hints);
}

/** Só aceita URL relativa do app ou asset local; nunca esquemas arbitrários. */
export function safeUrl(u) {
  return typeof u === 'string' && /^(\/[^/]|asset:|https?:\/\/asset\.localhost\/)/.test(u) ? u.replace(/["\\\n\r]/g, '') : null;
}

export function bgImage(el, url) {
  const u = safeUrl(url);
  if (u) el.style.backgroundImage = `url("${u}")`;
  return !!u;
}

/**
 * Disco (FRONTEND-DESIGN §5.3). variant: label | generic | virgin | unreadable.
 * flags: reading, readingStill, burning, erasing, rejected, spinning, enter, stopped.
 */
export function disc({ variant = 'label', cover, name, progress, size, ...flags } = {}) {
  const cls = ['disc', variant === 'label' ? '' : variant];
  for (const f of ['reading', 'burning', 'erasing', 'rejected', 'spinning', 'enter', 'stopped']) if (flags[f]) cls.push(f);
  if (flags.readingStill) cls.push('reading-still');
  const face = h('div', { class: 'face' });
  if (variant === 'label') {
    if (!bgImage(face, cover)) face.append(h('span', { class: 'tipo-apoio' }, name || ''));
  }
  const el = h('div', { class: cls.filter(Boolean).join(' ') }, h('div', { class: 'ring' }), face, h('div', { class: 'hole' }));
  if (size) el.style.setProperty('--d', size);
  if (progress != null) setDiscProgress(el, progress);
  return el;
}

export function setDiscProgress(el, pct) {
  el.style.setProperty('--r', `${Math.max(0, Math.min(100, pct))}%`);
}

/** Ficha de leitura (§5.4): lista de definição sem numeração. */
export function sheet(snap, device) {
  const m = snap.media;
  if (!m) return null;
  const na = (v) => (v ? h('dd', { class: 'tipo-dado' }, v) : h('dd', { class: 'tipo-dado na' }, t('common.not_informed')));
  const rows = [
    ['sheet.label', na(m.label)],
    ['sheet.media', na(t(`media.${m.physical}`))],
    ['sheet.id', na(m.disc_id)],
  ];
  if (m.ini_name) rows.push(['sheet.ini_name', na(m.ini_name)]);
  if (snap.other_game) rows.push(['sheet.belongs_to', na(snap.other_game.name)]);
  const dl = h('dl');
  for (const [k, dd] of rows) dl.append(h('dt', { class: 'tipo-apoio' }, t(k)), dd);
  return h('div', { class: 'sheet' }, dl);
}
