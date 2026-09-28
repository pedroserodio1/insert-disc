// Spike W1: registra eventos das duas fontes na mesma linha do tempo (performance.now).
const { listen } = window.__TAURI__.event;
const win = window.__TAURI__.window.getCurrentWindow();

// Índices do "standard mapping" da Gamepad API -> nomes de botão do gilrs.
const STANDARD = ['South', 'East', 'West', 'North', 'LeftTrigger', 'RightTrigger', 'LeftTrigger2',
  'RightTrigger2', 'Select', 'Start', 'LeftThumb', 'RightThumb', 'DPadUp', 'DPadDown', 'DPadLeft',
  'DPadRight', 'Mode'];

const lastDown = { web: {}, rust: {} };

function log(list, kind, detail, pad, focused) {
  const li = document.createElement('li');
  li.className = focused ? kind : `${kind} unfocused`;
  const t = document.createElement('span');
  t.className = 't';
  t.textContent = performance.now().toFixed(1);
  li.append(t, `${kind} ${detail} — ${pad}${focused ? '' : ' (janela sem foco)'}`);
  const ol = document.getElementById(list);
  ol.prepend(li);
  while (ol.children.length > 300) ol.lastChild.remove();

  if (kind === 'down') {
    const now = performance.now();
    lastDown[list][detail] = now;
    const other = lastDown[list === 'web' ? 'rust' : 'web'][detail];
    if (other !== undefined && now - other < 200) {
      const [first, ms] = list === 'web' ? ['gilrs', now - other] : ['Gamepad API', now - other];
      document.getElementById('delta').textContent = `${detail}: ${first} chegou ${ms.toFixed(1)} ms antes`;
    }
  }
}

// gilrs: eventos vindos do Rust.
listen('gilrs', ({ payload: p }) => log('rust', p.kind, p.detail, p.gamepad, p.window_focused));

// Gamepad API: polling a cada quadro.
const prev = {};
window.addEventListener('gamepadconnected', (e) => log('web', 'connected', `mapping=${e.gamepad.mapping || 'none'}`, e.gamepad.id, document.hasFocus()));
window.addEventListener('gamepaddisconnected', (e) => log('web', 'disconnected', '', e.gamepad.id, document.hasFocus()));

function zone(v) { return v > 0.5 ? 1 : v < -0.5 ? -1 : 0; }

function poll() {
  for (const gp of navigator.getGamepads()) {
    if (!gp) continue;
    const p = prev[gp.index] ??= { buttons: [], axes: [] };
    gp.buttons.forEach((b, i) => {
      if (b.pressed !== !!p.buttons[i]) {
        const name = gp.mapping === 'standard' ? STANDARD[i] ?? `b${i}` : `b${i}`;
        log('web', b.pressed ? 'down' : 'up', name, gp.id, document.hasFocus());
        p.buttons[i] = b.pressed;
      }
    });
    gp.axes.forEach((v, i) => {
      const z = zone(v);
      if (z !== (p.axes[i] ?? 0)) {
        log('web', 'axis', `a${i} ${z > 0 ? '+' : ''}${z}`, gp.id, document.hasFocus());
        p.axes[i] = z;
      }
    });
  }
  requestAnimationFrame(poll);
}
requestAnimationFrame(poll);

async function status() {
  const pads = navigator.getGamepads().filter(Boolean).length;
  const fs = await win.isFullscreen();
  document.getElementById('status').innerHTML = '';
  const s = document.getElementById('status');
  s.append('Foco: ', b(document.hasFocus() ? 'sim' : 'não'), ' · Tela cheia: ', b(fs ? 'sim' : 'não'),
    ' · Controles vistos pela Gamepad API: ', b(String(pads)));
}
function b(text) { const el = document.createElement('b'); el.textContent = text; return el; }
setInterval(status, 250);

window.addEventListener('keydown', async (e) => {
  if (e.key === 'F11') { e.preventDefault(); await win.setFullscreen(!(await win.isFullscreen())); }
  if (e.key === 'c' || e.key === 'C') { document.getElementById('web').replaceChildren(); document.getElementById('rust').replaceChildren(); }
});
