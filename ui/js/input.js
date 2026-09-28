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

  // Gamepad API: polling por quadro; sem foco de janela, nada é processado (SECURITY R10).
  const held = new Map(); // nome -> { since, last }
  function kindOf(pad) {
    const id = pad.id || '';
    return /054c|dualshock|dualsense|playstation|wireless controller/i.test(id) ? 'playstation' : 'xbox';
  }
  function frame(now) {
    if (document.hasFocus()) {
      const active = new Set();
      for (const pad of navigator.getGamepads?.() || []) {
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
    }
    requestAnimationFrame(frame);
  }
  requestAnimationFrame(frame);

  return { device: () => device, setDevice };
}
