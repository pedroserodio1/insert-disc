// Entrada normalizada (FRONTEND-DESIGN §7): teclado, mouse e Gamepad API viram os mesmos eventos
// (left, right, up, down, accept, back, x, y, lb, rb, menu). Se o spike W1 escolher o gilrs,
// só esta fonte muda: o resto da UI continua consumindo estes eventos (UI-CONTRACT).

const KEYS = {
  ArrowLeft: 'left', ArrowRight: 'right', ArrowUp: 'up', ArrowDown: 'down',
  Enter: 'accept', ' ': 'accept', Escape: 'back', Backspace: 'back',
  n: 'x', N: 'x', o: 'y', O: 'y', PageUp: 'lb', PageDown: 'rb', m: 'menu', M: 'menu',
};

// "standard mapping" da Gamepad API
const PAD_BUTTONS = { 0: 'accept', 1: 'back', 2: 'x', 3: 'y', 4: 'lb', 5: 'rb', 9: 'menu', 12: 'up', 13: 'down', 14: 'left', 15: 'right' };
const REPEATABLE = new Set(['up', 'down', 'left', 'right']);
const REPEAT_DELAY = 400; // dur-repeticao-atraso
const REPEAT_EVERY = 80; // dur-repeticao-intervalo

/** Tipo de controle pelo id da Gamepad API (glifos). */
export function kindOf(pad) {
  return /054c|dualshock|dualsense|playstation|wireless controller/i.test(pad.id || '') ? 'playstation' : 'xbox';
}

/**
 * Converte o estado dos controles em eventos, uma chamada por quadro (`step`). Puro (sem DOM):
 * é isto que os testes e o simulador da UI alimentam com controles sintéticos.
 */
export function createPadTracker(handler, setDevice = () => {}) {
  const held = new Map(); // nome -> { since, last }
  return {
    step(now, pads) {
      const active = new Set();
      for (const pad of pads) {
        if (!pad) continue;
        const down = new Set();
        pad.buttons.forEach((b, i) => { if (b.pressed && PAD_BUTTONS[i]) down.add(PAD_BUTTONS[i]); });
        const [ax, ay] = pad.axes;
        if (ax < -0.5) down.add('left'); else if (ax > 0.5) down.add('right');
        if (ay < -0.5) down.add('up'); else if (ay > 0.5) down.add('down');
        for (const name of down) {
          active.add(name);
          const h = held.get(name);
          if (!h) {
            held.set(name, { since: now, last: now });
            setDevice(kindOf(pad));
            handler.press(name, { repeat: false });
          } else if (REPEATABLE.has(name) && now - h.since >= REPEAT_DELAY && now - h.last >= REPEAT_EVERY) {
            h.last = now;
            handler.press(name, { repeat: true });
          }
        }
      }
      for (const name of [...held.keys()]) {
        if (!active.has(name)) { held.delete(name); if (name === 'accept') handler.release('accept'); }
      }
    },
  };
}

/** Controle sintético no "standard mapping" com os botões de `names` apertados. */
export function syntheticPad(names, id = 'synthetic xbox controller') {
  const idx = Object.fromEntries(Object.entries(PAD_BUTTONS).map(([i, n]) => [n, Number(i)]));
  const buttons = Array.from({ length: 17 }, () => ({ pressed: false }));
  for (const n of names) if (n in idx) buttons[idx[n]].pressed = true;
  return { id, buttons, axes: [0, 0] };
}

export function createInput(handler) {
  let device = 'keys'; // 'keys' | 'xbox' | 'playstation'
  const setDevice = (d) => { if (d !== device) { device = d; handler.deviceChanged?.(d); } };

  const isText = (el) => el && (el.tagName === 'INPUT' || el.tagName === 'TEXTAREA');

  window.addEventListener('keydown', (e) => {
    if (e.ctrlKey || e.altKey || e.metaKey) return;
    if (isText(e.target)) {
      if (e.key === 'Escape') { e.target.blur(); handler.press('back'); e.preventDefault(); }
      return; // nos campos de texto, o resto é digitação
    }
    const name = KEYS[e.key];
    if (!name) return;
    e.preventDefault();
    setDevice('keys');
    if (name === 'accept' && e.repeat) return; // segurar não repete o "aceitar"
    handler.press(name, { repeat: e.repeat });
  });
  window.addEventListener('keyup', (e) => {
    if (isText(e.target)) return;
    const name = KEYS[e.key];
    if (name === 'accept') handler.release('accept');
  });
  window.addEventListener('mousedown', () => setDevice('keys'), { passive: true });
  // Mouse (FRONTEND-DESIGN §7): botão direito = opções (Y), botão "voltar" = B, roda = mover o foco.
  window.addEventListener('contextmenu', (e) => {
    e.preventDefault();
    if (isText(e.target)) return;
    setDevice('keys');
    handler.press('y', { repeat: false });
  });
  const backButton = (e) => { if (e.button === 3) { e.preventDefault(); setDevice('keys'); if (e.type === 'mouseup') handler.press('back', { repeat: false }); } };
  window.addEventListener('mousedown', backButton);
  window.addEventListener('mouseup', backButton);
  window.addEventListener('auxclick', (e) => { if (e.button === 3 || e.button === 4) e.preventDefault(); });
  let lastWheel = 0;
  window.addEventListener('wheel', (e) => {
    if (e.target.closest?.('.scroll, textarea') || Math.abs(e.deltaY) < 1) return; // listas rolam sozinhas
    const now = performance.now();
    if (now - lastWheel < 70) return; // um passo por "clique" da roda, sem disparar em rajada
    lastWheel = now;
    setDevice('keys');
    handler.wheel?.(e.deltaY > 0 ? 1 : -1);
  }, { passive: true });

  // Gamepad API: polling por quadro; sem foco de janela, nada é processado (SECURITY R10).
  // Controles sintéticos (painel de desenvolvimento) entram pelo mesmo caminho.
  const tracker = createPadTracker(handler, setDevice);
  const synthetic = new Set();
  let syntheticId = 'synthetic xbox controller';
  function frame(now) {
    if (document.hasFocus()) {
      const pads = [...(navigator.getGamepads?.() || [])];
      if (synthetic.size) pads.push(syntheticPad(synthetic, syntheticId));
      tracker.step(now, pads);
    }
    requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);

  return {
    device: () => device,
    setDevice,
    /** Evento do controle lido pelo Rust (gilrs, D1): mantém apertado até o `pressed: false`. */
    feedPad(name, pressed, device) {
      if (pressed) { syntheticId = device === 'playstation' ? 'playstation controller' : 'synthetic xbox controller'; synthetic.add(name); }
      else synthetic.delete(name);
    },
    /** Aperta `name` num controle sintético por `ms` (simulador, D2). */
    simulate(name, ms = 120) {
      synthetic.add(name);
      setTimeout(() => synthetic.delete(name), ms);
    },
  };
}
