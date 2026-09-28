// Estante de caixas de CD (FRONTEND-DESIGN §4, §5.1, §5.2, §5.2.1).
// Só a caixa em foco é um bloco 3D; as demais lombadas são planas. Só `transform` e `opacity`
// são animados. Navegação rápida (direcional segurado): nada gira; a caixa é puxada ao soltar.

import { h, bgImage } from './components.js';

const HOVER_MS = 150; // dur-hover
const FAST_SETTLE_MS = 140; // depois disto sem nova entrada, a caixa é puxada
const LEAVE_MS = 220; // dur-devolver + folga

const spineColor = (c) => (/^#[0-9a-f]{6}$/i.test(c || '') ? c : null);

function spineEl(g) {
  const s = h('div', { class: `spine${g.cover ? '' : ' no-cover'}` }, h('span', { class: 'name tipo-lombada' }, g.name), g.disc_count > 0 && h('span', { class: 'dot' }));
  const c = spineColor(g.spine_color);
  if (c && g.cover) s.style.setProperty('--spine', c);
  else if (c) s.style.setProperty('--spine', c), s.classList.remove('no-cover');
  return s;
}

function frontEl(g) {
  const front = h('div', { class: 'c-front' });
  const c = spineColor(g.spine_color);
  if (c) front.style.setProperty('--spine', c);
  const art = h('div', { class: 'front-art' });
  if (g.cover && bgImage(art, g.cover)) front.append(h('div', { class: 'front-band' }, h('span', { class: 'name' }, g.name)), art);
  else front.append(h('div', { class: 'front-name' }, g.name));
  return front;
}

export function createShelf({ games, focus = 0, onFocus, onAccept }) {
  const el = h('div', { class: 'shelf' });
  const track = h('div', { class: 'track instant' });
  el.append(track, h('div', { class: 'floor' }));
  let list = [];
  let idx = 0;
  let pullTimer = 0;
  let hoverTimer = 0;
  const slots = [];

  function build(gs) {
    list = gs;
    slots.length = 0;
    track.replaceChildren();
    gs.forEach((g, i) => {
      const s = h('div', { class: 'slot', style: `--i:${i}`, dataset: { id: g.game_id } }, spineEl(g));
      // só movimento real do mouse move o foco: as lombadas mudam de lugar sob um cursor parado
      s.addEventListener('mousemove', (e) => {
        if (!e.movementX && !e.movementY) return;
        clearTimeout(hoverTimer);
        hoverTimer = setTimeout(() => { if (i !== idx) onFocus?.(i); }, HOVER_MS);
      });
      s.addEventListener('mouseleave', () => clearTimeout(hoverTimer));
      s.addEventListener('mousedown', () => { if (i === idx) onAccept?.(i); else onFocus?.(i); });
      slots.push(s);
      track.append(s);
    });
  }

  function leave(slot, instant) {
    if (!slot) return;
    const c = slot.querySelector('.case');
    slot.classList.remove('focused', 'pending');
    if (!c) { slot.classList.remove('live'); return; }
    if (instant) { c.remove(); slot.classList.remove('live'); return; }
    c.classList.remove('pulled');
    setTimeout(() => { if (!slot.classList.contains('focused')) { c.remove(); slot.classList.remove('live'); } }, LEAVE_MS);
  }

  function pull(i) {
    const slot = slots[i];
    if (!slot || i !== idx || slot.querySelector('.case.pulled')) return;
    slot.querySelector('.case')?.remove();
    const g = list[i];
    const c = h('div', { class: 'case' }, h('div', { class: 'c-spine' }, spineEl(g)), frontEl(g));
    slot.classList.add('live', 'focused');
    slot.classList.remove('pending');
    slot.append(c);
    c.getBoundingClientRect(); // fixa o estado inicial antes de animar
    c.classList.add('pulled');
  }

  let lastMove = 0;
  function setFocus(i, { fast = false } = {}) {
    if (!list.length) return;
    // nova entrada antes do fim da animação: pula para o estado final (§5.2.1)
    const now = performance.now();
    if (now - lastMove < 300) fast = true;
    lastMove = now;
    i = Math.max(0, Math.min(list.length - 1, i));
    const prev = idx;
    if (i === prev && slots[i]?.querySelector('.case')) return;
    clearTimeout(pullTimer);
    if (i !== prev) leave(slots[prev], fast);
    idx = i;
    track.classList.toggle('instant', fast);
    track.style.setProperty('--f', String(i));
    if (fast) {
      slots[i].classList.add('pending');
      pullTimer = setTimeout(() => pull(i), FAST_SETTLE_MS);
    } else pull(i);
  }

  function update(gs, keepId) {
    const same = gs.length === list.length && gs.every((g, i) => g.game_id === list[i].game_id && g.name === list[i].name && g.disc_count === list[i].disc_count && g.cover === list[i].cover);
    if (same) return;
    const at = keepId ? gs.findIndex((g) => g.game_id === keepId) : -1;
    const prev = Math.max(0, idx);
    build(gs);
    idx = -1;
    track.classList.add('instant');
    setFocus(at >= 0 ? at : Math.min(prev, gs.length - 1), { fast: false });
    track.classList.remove('instant');
  }

  build(games);
  track.style.setProperty('--f', String(focus));
  idx = Math.max(0, Math.min(list.length - 1, focus));
  requestAnimationFrame(() => { track.classList.remove('instant'); if (list.length) { idx = -1; setFocus(focus); } });

  return {
    el,
    setFocus,
    update,
    focusedIndex: () => idx,
    focusedId: () => list[idx]?.game_id,
    count: () => list.length,
    destroy() { clearTimeout(pullTimer); clearTimeout(hoverTimer); },
  };
}
