// Painel de desenvolvimento do drive falso (?dev=1). Só existe no servidor de desenvolvimento.

import { bridge } from './bridge.js';
import { h } from './components.js';

export function mountDevPanel() {
  const panel = h('aside', { class: 'devpanel' });
  document.body.append(panel);
  const cmd = async (c) => { await bridge.dev(c); render(); };
  let st = null;

  const btn = (label, c, on) => h('button', { class: on ? 'on' : '', onclick: () => cmd(c) }, label);

  function render() {
    bridge.devState().then((s) => {
      st = s;
      const withDisc = s.games.filter((g) => g.has_disc);
      const gameSel = h('select', {}, withDisc.map((g) => h('option', { value: g.game_id }, g.name)));
      const flag = (name, label) => h('button', { onclick: async () => { const on = !flag.state?.[name]; (flag.state ??= {})[name] = on; await bridge.dev({ cmd: name, on }); render(); }, class: flag.state?.[name] ? 'on' : '' }, label);
      const cap = (name) => h('button', { class: s.caps[name] ? '' : 'on', onclick: () => cmd({ cmd: 'cap', name, on: !s.caps[name] }) }, `${name}: ${s.caps[name] ? 'sim' : 'não'}`);
      panel.replaceChildren(
        h('h3', {}, `Drive falso · mídia: ${s.has_media ? 'sim' : 'não'} · armada: ${s.armed ? 'sim' : 'não'}`),
        h('h3', {}, 'Inserir disco de um jogo'), gameSel,
        btn('Inserir (CD-ROM)', null), // substituído abaixo
        h('h3', {}, 'Cenários'),
        ...['unknown', 'legacy', 'invalid', 'noini', 'blank_cdr', 'blank_cdrw', 'audio', 'unreadable', 'cdr_used', 'cdrw_used'].map((w) => btn(w, { cmd: 'insert', what: w })),
        h('h3', {}, 'Ações do drive'),
        btn('Remover disco', { cmd: 'remove' }), btn('Evento duplicado', { cmd: 'duplicate' }), btn('Desconectar', { cmd: 'disconnect' }), btn('Reconectar', { cmd: 'reconnect' }),
        h('h3', {}, 'Falhas'),
        flag('fail_burn', 'falha ao gravar'), flag('fail_erase', 'falha ao apagar'), flag('fail_open_tray', 'gaveta travada'),
        h('h3', {}, 'Capacidades'), cap('tray_open'), cap('eject'), cap('write_cdr'), cap('write_cdrw'),
        h('h3', {}, 'Lançador'),
        btn('ok', { cmd: 'launcher_fail', reason: null }), btn('sem Steam', { cmd: 'launcher_fail', reason: 'steam_missing' }), btn('não achou', { cmd: 'launcher_fail', reason: 'not_found' }),
        h('h3', {}, 'Lançado'), h('div', { class: 'log' }, s.launched.join('\n') || '(nada)'),
      );
      // botões que dependem do select
      const anchor = [...panel.querySelectorAll('button')].find((b) => b.textContent === 'Inserir (CD-ROM)');
      const mk = (label, prefix) => h('button', { onclick: () => cmd({ cmd: 'insert', what: `${prefix}${gameSel.value}` }) }, label);
      anchor?.replaceWith(mk('CD-ROM', 'game:'), mk('CD-RW regravável', 'cdrw_used:'));
    });
  }
  render();
  setInterval(render, 1500);
  window.__dev = { render, get state() { return st; } };
}
