// Ponto de entrada da UI: consulta o núcleo, escolhe a tela do estado, gerencia foco, entrada,
// avisos e a confirmação por segurar. Nenhuma regra de negócio aqui (UI-CONTRACT).

import { bridge } from './bridge.js';
import { bar, h, hint, setDiscProgress } from './components.js';
import { createInput } from './input.js';
import { setLocale, t } from './i18n.js';
import * as S from './screens.js';

const app = document.getElementById('app');
const toastEl = document.getElementById('toast');
const POLL_MS = 100;
const HOLD_MS = 1500; // dur-segurar
const TOAST_MS = 5000; // dur-aviso

const ui = { libFocusId: null, form: null, settingsIndex: 0 };
let snap = null;
let current = null;
let barEl = null;
let sig = '';
let focusIdx = 0;
let device = 'keys';
let localeKey = null;
let busy = false;
let hold = null;
let toastId = 0;
let toastTimer = 0;

// ---------- contexto entregue às telas ----------
const ctx = {
  ui,
  device: () => device,
  async send(intent) {
    const res = await bridge.intent(intent);
    await refresh();
    return res;
  },
  accept: () => doAccept(),
  async openForm(kind) {
    // Steam: escolher da lista de instalados (A3); sem Steam ou sem jogos, digitar o AppID
    if (kind === 'steam') { ui.steamGames = (await bridge.steamGames().catch(() => null))?.games ?? []; if (ui.steamGames.length) kind = 'steam-pick'; }
    ui.form = kind; onSnapshot(snap, true);
  },
  closeForm() { ui.form = null; onSnapshot(snap, true); },
  applyFullscreen(on) {
    const w = window.__TAURI__?.window?.getCurrentWindow?.();
    if (w) w.setFullscreen(on);
    else if (on) document.documentElement.requestFullscreen?.().catch(() => {});
    else if (document.fullscreenElement) document.exitFullscreen?.();
  },
  async exportCatalog() {
    const url = URL.createObjectURL(new Blob([await bridge.exportCatalog()], { type: 'application/json' }));
    const a = h('a', { href: url, download: 'insert-disc-estante.json' });
    document.body.append(a); a.click(); a.remove(); URL.revokeObjectURL(url);
  },
  /** Escolher um arquivo de imagem como capa do jogo (o backend valida e reencoda, SECURITY R6). */
  pickCover(gameId) {
    const input = h('input', { type: 'file', accept: 'image/png,image/jpeg,image/webp', style: 'display:none' });
    input.addEventListener('change', async () => {
      const file = input.files?.[0];
      input.remove();
      if (!file) return;
      const bytes = new Uint8Array(await file.arrayBuffer());
      let bin = '';
      for (let i = 0; i < bytes.length; i += 0x8000) bin += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
      const res = await bridge.setCover({ game_id: gameId, data: btoa(bin) }).catch((e) => ({ error: String(e) }));
      if (res?.error || typeof res === 'string') flash('cover_refused');
      await refresh();
    });
    document.body.append(input); input.click();
  },
  importCatalog() {
    const input = h('input', { type: 'file', accept: 'application/json,.json', style: 'display:none' });
    input.addEventListener('change', async () => {
      const text = await input.files?.[0]?.text();
      input.remove();
      if (text) { await bridge.importCatalog(text); await refresh(); }
    });
    document.body.append(input); input.click();
  },
};

// ---------- foco ----------
function applyFocus() {
  if (!current) return;
  current.items.forEach((it, i) => it.el.classList.toggle('focused', i === focusIdx));
  const it = current.items[focusIdx];
  if (it?.el && it.isField) it.el.focus({ preventScroll: true });
  it?.el?.scrollIntoView?.({ block: 'nearest' });
}

function setFocus(i) {
  if (!current || !current.items.length) return;
  focusIdx = Math.max(0, Math.min(current.items.length - 1, i));
  applyFocus();
}

function doAccept() {
  if (!current) return;
  const it = current.items[focusIdx];
  if (it) it.run?.();
  else current.acceptDefault?.();
}

function wire(screen) {
  screen.items.forEach((it, i) => {
    it.el.addEventListener('mousemove', (e) => { if ((e.movementX || e.movementY) && focusIdx !== i) setFocus(i); });
    if (!it.isField) it.el.addEventListener('click', () => { setFocus(i); if (screen.holdIndex !== i) it.run?.(); });
  });
}

// ---------- barra de ações ----------
function renderBar() {
  barEl?.remove();
  barEl = null;
  if (!current || !current.hints?.length) return;
  barEl = bar(current.hints.map((x) => hint(x.btn, x.text, device, { primary: x.primary, onClick: x.fn })));
  app.append(barEl);
}

// ---------- confirmação por segurar (§5.7) ----------
function holdStart() {
  if (!current || current.holdIndex == null || focusIdx !== current.holdIndex || hold) return false;
  const ring = current.el.querySelector('.hold-ring');
  const t0 = performance.now();
  hold = { raf: 0, ring };
  const tick = (now) => {
    if (!hold) return;
    const p = Math.min(1, (now - t0) / HOLD_MS);
    ring?.style.setProperty('--p', String(p));
    if (p >= 1) { holdCancel(); ctx.send({ type: 'hold_complete' }); return; }
    hold.raf = requestAnimationFrame(tick);
  };
  hold.raf = requestAnimationFrame(tick);
  return true;
}

function holdCancel() {
  if (!hold) return;
  cancelAnimationFrame(hold.raf);
  hold.ring?.style.setProperty('--p', '0');
  hold = null;
}

// ---------- entrada ----------
const input = createInput({
  press(name, { repeat = false } = {}) {
    if (!current) return;
    if (name === 'accept' && holdStart()) return;
    if (current.onNav?.(name, repeat)) return;
    if (current.isForm && (name === 'left' || name === 'right')) return;
    switch (name) {
      case 'up': setFocus(focusIdx - 1); break;
      case 'down': setFocus(focusIdx + 1); break;
      case 'left': if (current.dialog) setFocus(focusIdx - 1); else current.items[focusIdx]?.left?.(); break;
      case 'right': if (current.dialog) setFocus(focusIdx + 1); else current.items[focusIdx]?.right?.(); break;
      case 'accept': doAccept(); break;
      case 'back': current.back?.(); break;
      case 'x': case 'y': case 'menu': case 'lb': case 'rb': break;
    }
  },
  release(name) { if (name === 'accept') holdCancel(); },
  deviceChanged(d) { device = d; renderBar(); },
});
window.addEventListener('blur', holdCancel);
// controle lido pelo Rust (gilrs, D1): o app só emite com a janela em foco; sem a feature, nada chega
window.__TAURI__?.event?.listen('pad', (e) => input.feedPad(e.payload.name, e.payload.pressed, e.payload.device));

// ---------- avisos (§5.6) ----------
function showToast(tv) {
  if (!tv || tv.id === toastId) return;
  toastId = tv.id;
  flash(tv.code, tv.params?.other ?? '');
}

/** Aviso local da UI (não vem do núcleo, não interfere na deduplicação dos avisos dele). */
function flash(code, other = '') {
  const text = t(`toast.${code}`, { other });
  const bad = ['drive_removed', 'read_error', 'not_a_game', 'cover_refused'].includes(code);
  toastEl.textContent = text;
  toastEl.style.setProperty('--tc', bad ? 'var(--laser)' : 'var(--ftalocianina)');
  toastEl.hidden = false;
  clearTimeout(toastTimer);
  toastTimer = setTimeout(() => { toastEl.hidden = true; }, TOAST_MS);
}

// ---------- estado -> tela ----------
function build(s) {
  if (ui.form && ['REG_CHOOSE_GAME', 'GAME_OPTIONS'].includes(s.state)) return S.form(ui.form, s, ctx);
  ui.form = null;
  switch (s.state) {
    case 'BOOT': return S.boot(s, ctx);
    case 'LIBRARY': return S.library(s, ctx);
    case 'NO_DISC_YET': return S.noDiscYet(s, ctx);
    case 'WAITING_DISC': return S.waitingDisc(s, ctx);
    case 'READING': return S.reading(s, ctx);
    case 'IDENTIFIED': return S.identified(s, ctx);
    case 'REJECTED': return S.rejected(s, ctx);
    case 'ADOPT_CONFIRM': return S.adoptConfirm(s, ctx);
    case 'LAUNCHING': return S.launching(s, ctx);
    case 'LAUNCH_ERROR': return S.launchError(s, ctx);
    case 'GAME_OPTIONS': return S.gameOptions(s, ctx);
    case 'REMOVE_CONFIRM': return S.removeConfirm(s, ctx);
    case 'SETTINGS': return S.settings(s, ctx);
    case 'DRIVE_PROBLEM': return S.driveProblem(s, ctx);
    case 'CATALOG_ERROR': return S.catalogError(s, ctx);
    case 'REG_INSERT': return S.regInsert(s, ctx);
    case 'REG_REJECTED': return S.regRejected(s, ctx);
    case 'REG_ERASE_CONFIRM': return S.regEraseConfirm(s, ctx);
    case 'REG_CHOOSE_GAME': return S.regChooseGame(s, ctx);
    case 'REG_LABEL_PREVIEW': return S.regLabelPreview(s, ctx);
    case 'REG_CDR_WARNING': return S.regCdrWarning(s, ctx);
    case 'BURN_DONE': return S.burnDone(s, ctx);
    case 'BURN_FAILED': return S.burnFailed(s, ctx);
    case 'BURNING': case 'VERIFYING': case 'ERASING': case 'REG_READING': return S.progressScene(s, ctx);
    default: return S.boot(s, ctx);
  }
}

function signature(s) {
  const base = [s.state, s.reason, s.game?.game_id, s.game?.name, s.other_game?.game_id, s.actions.map((a) => a.id + a.default).join(), s.media?.disc_id, s.media?.class, s.media?.label, s.step, s.label, localeKey, ui.form, s.drive];
  if (s.state === 'SETTINGS') base.push(JSON.stringify(s.settings));
  if (s.state === 'GAME_OPTIONS') base.push(s.game_discs.map((d) => d.disc_id).join());
  if (s.state === 'REG_CHOOSE_GAME') base.push(s.library.map((g) => g.game_id + g.disc_count).join());
  if (s.state === 'LAUNCHING') base.push(s.timing?.started_at);
  return JSON.stringify(base);
}

function onSnapshot(s, force = false) {
  if (!s) return;
  snap = s;
  const wanted = s.settings.locale ?? 'system';
  if (wanted !== localeKey) { localeKey = wanted; setLocale(wanted); force = true; }
  if (s.focus_hint) ui.libFocusId = s.focus_hint;
  showToast(s.toast);

  const next = signature(s);
  if (force || next !== sig) {
    holdCancel();
    const keepIndex = current?.keepIndex && s.state === 'SETTINGS' ? focusIdx : null;
    current?.destroy?.();
    if (current?.keepIndex && s.state === 'SETTINGS') ui.settingsIndex = focusIdx;
    sig = next;
    const before = current;
    current = build(s);
    // diálogo aparece sobre a cena anterior, com o véu atrás (§5.7)
    const under = current.dialog && before && !before.dialog && !before.error ? before.el : null;
    app.replaceChildren(...(under ? [under, current.el] : [current.el]));
    wire(current);
    renderBar();
    focusIdx = keepIndex ?? current.index ?? 0;
    if (current.keepIndex) focusIdx = Math.min(ui.settingsIndex ?? 0, Math.max(0, current.items.length - 1));
    applyFocus();
    if (current.focusInput) current.focusInput.focus({ preventScroll: true });
  } else {
    current?.update?.(s);
    const d = current?.discEl;
    if (d && s.progress != null) setDiscProgress(d, s.state === 'ERASING' ? 100 - s.progress : s.progress);
  }
}

async function refresh() {
  if (busy) return;
  busy = true;
  try {
    onSnapshot(await bridge.snapshot());
  } catch (e) {
    if (!current || current.error !== true) {
      current = { el: h('div', { class: 'screen' }, h('h1', { class: 'tipo-titulo' }, t('error.connection'))), items: [], hints: [], error: true };
      app.replaceChildren(current.el);
      sig = '';
    }
  } finally {
    busy = false;
  }
}

setLocale('system');
current = S.boot();
app.replaceChildren(current.el);
refresh();
setInterval(refresh, POLL_MS);
// Painel do drive falso: ?dev=1 ou Ctrl+Shift+D (no app desktop não há URL para editar).
let devPanel = null;
const toggleDev = () => {
  if (devPanel) { devPanel.remove(); devPanel = null; return; }
  import('./dev.js').then((m) => { devPanel = m.mountDevPanel(input); });
};
if (new URLSearchParams(location.search).has('dev')) toggleDev();
window.addEventListener('keydown', (e) => { if (e.ctrlKey && e.shiftKey && (e.key === 'D' || e.key === 'd')) { e.preventDefault(); toggleDev(); } });
window.__insertDisc = { ui, get snap() { return snap; }, ctx, input };
