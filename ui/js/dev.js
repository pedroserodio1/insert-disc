// Painel de desenvolvimento do drive falso (?dev=1 ou Ctrl+Shift+D).

import { bridge } from './bridge.js';
import { h } from './components.js';

const SCENARIOS = ['unknown', 'legacy', 'invalid', 'noini', 'blank_cdr', 'blank_cdrw', 'audio', 'unreadable', 'cdr_used', 'cdrw_used'];

export function mountDevPanel() {
  const panel = h('aside', { class: 'devpanel' });
  document.body.append(panel);
  const flags = {};
  let selected = '';

  const cmd = async (c) => { await bridge.dev(c); render(); };
  const btn = (label, c, on) => h('button', { class: on ? 'on' : '', onclick: () => cmd(c) }, label);

  async function render() {
    if (!panel.isConnected) return;
    const s = await bridge.devState();
    const withDisc = s.games.filter((g) => g.has_disc);
    if (!selected || !withDisc.some((g) => g.game_id === selected)) selected = withDisc[0]?.game_id ?? '';
    const sel = h('select', { onchange: (e) => { selected = e.target.value; } }, withDisc.map((g) => h('option', { value: g.game_id, selected: g.game_id === selected }, g.name)));
    const flag = (name, label) => h('button', {
      class: flags[name] ? 'on' : '',
      onclick: async () => { flags[name] = !flags[name]; await bridge.dev({ cmd: name, on: flags[name] }); render(); },
    }, label);
    const cap = (name) => h('button', { class: s.caps[name] ? '' : 'on', onclick: () => cmd({ cmd: 'cap', name, on: !s.caps[name] }) }, `${name}: ${s.caps[name] ? 'sim' : 'não'}`);

    panel.replaceChildren(
      h('h3', {}, `Drive falso · mídia: ${s.has_media ? 'sim' : 'não'} · armada: ${s.armed ? 'sim' : 'não'}`),
      h('h3', {}, 'Inserir disco de um jogo'), sel,
      h('button', { onclick: () => cmd({ cmd: 'insert', what: `game:${selected}` }) }, 'CD-ROM'),
      h('button', { onclick: () => cmd({ cmd: 'insert', what: `cdrw_used:${selected}` }) }, 'CD-RW regravável'),
      h('h3', {}, 'Cenários'), ...SCENARIOS.map((w) => btn(w, { cmd: 'insert', what: w })),
      h('h3', {}, 'Ações do drive'),
      btn('Remover disco', { cmd: 'remove' }), btn('Evento duplicado', { cmd: 'duplicate' }), btn('Desconectar', { cmd: 'disconnect' }), btn('Reconectar', { cmd: 'reconnect' }),
      h('h3', {}, 'Falhas'), flag('fail_burn', 'falha ao gravar'), flag('fail_erase', 'falha ao apagar'), flag('fail_open_tray', 'gaveta travada'),
      h('h3', {}, 'Duração das operações'), btn('instantâneo', { cmd: 'op_delay', ms: 0 }), btn('3 s', { cmd: 'op_delay', ms: 3000 }), btn('8 s', { cmd: 'op_delay', ms: 8000 }),
      h('h3', {}, 'Capacidades'), cap('tray_open'), cap('eject'), cap('write_cdr'), cap('write_cdrw'),
      h('h3', {}, 'Lançador'),
      btn('ok', { cmd: 'launcher_fail', reason: null }), btn('sem Steam', { cmd: 'launcher_fail', reason: 'steam_missing' }), btn('não achou', { cmd: 'launcher_fail', reason: 'not_found' }),
      h('h3', {}, 'Lançado'), h('div', { class: 'log' }, s.launched.join('\n') || '(nada)'),
    );
  }

  render();
  const timer = setInterval(() => { if (panel.isConnected) render(); else clearInterval(timer); }, 1500);
  return panel;
}
