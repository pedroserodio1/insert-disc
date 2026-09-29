// E1: testes da UI sem dependências: `node --test ui/test`.
// Os módulos da UI esperam o navegador; estes stubs bastam para as partes sem DOM.
import test from 'node:test';
import assert from 'node:assert/strict';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

globalThis.document = { documentElement: {} };
globalThis.window = {};

const js = path.join(path.dirname(fileURLToPath(import.meta.url)), '..', 'js');
const load = (name) => import(pathToFileURL(path.join(js, name)).href);
const source = (name) => fs.readFileSync(path.join(js, name), 'utf8');

// as chaves ficam no objeto L de i18n.js: extrai pelo texto para não expor L
const table = () => {
  const src = source('i18n.js');
  const body = src.slice(src.indexOf('const L = {'), src.indexOf('\n};') + 3);
  return new Function(`${body}; return L;`)();
};

test('toda chave tem pt-BR e en, com os mesmos marcadores e o mesmo plural', () => {
  const L = table();
  const keys = Object.keys(L);
  assert.ok(keys.length > 100);
  for (const k of keys) {
    const [pt, en] = L[k];
    assert.ok(pt && en, `${k}: texto vazio`);
    const marks = (s) => [...s.matchAll(/\{(\w+)\}/g)].map((m) => m[1]).sort().join();
    assert.equal(marks(pt), marks(en), `${k}: marcadores diferentes`);
    assert.equal(pt.includes('|'), en.includes('|'), `${k}: plural só em um idioma`);
  }
});

test('toda chave estática usada em t(...) existe', async () => {
  const { hasKey } = await load('i18n.js');
  const missing = [];
  for (const f of fs.readdirSync(js).filter((n) => n.endsWith('.js'))) {
    for (const m of source(f).matchAll(/\bt\(\s*'([a-z0-9_.]+)'/g)) if (!hasKey(m[1])) missing.push(`${f}: ${m[1]}`);
  }
  assert.deepEqual(missing, []);
});

test('chaves dinâmicas do núcleo têm texto em pt-BR e en', async () => {
  const { hasKey } = await load('i18n.js');
  const dyn = {
    'toast.': ['focus', 'unknown', 'not_a_game', 'read_error', 'drive_removed', 'cover_refused', 'import_confirm'],
    'launch.error.': ['not_found', 'steam_missing', 'elevation_denied', 'generic'],
    'library.kind.': ['steam', 'custom'],
    'disc.origin.': ['burned', 'adopted'],
  };
  for (const [prefix, codes] of Object.entries(dyn)) for (const c of codes) assert.ok(hasKey(prefix + c), prefix + c);
});

test('plural e interpolação, nunca por concatenação', async () => {
  const { t, setLocale } = await load('i18n.js');
  setLocale('en');
  assert.equal(t('library.disc_count', { n: 1 }), '1 disc');
  assert.equal(t('library.disc_count', { n: 3 }), '3 discs');
  assert.equal(t('action.play_other', { other: 'Celeste' }), 'Play Celeste');
  setLocale('pt-BR');
  assert.equal(t('library.disc_count', { n: 2 }), '2 discos');
  assert.equal(t('action.play_other', { other: 'Hades' }), 'Jogar Hades');
  assert.equal(t('chave.inexistente'), 'chave.inexistente');
});

test('safeUrl só aceita capa do app, caminho relativo e asset local', async () => {
  const { safeUrl } = await load('components.js');
  assert.equal(safeUrl('cover:3f2b7c1e-0a4d-4f5b-9c1d-2e8a7b6c5d4e.jpg'), '/cover/3f2b7c1e-0a4d-4f5b-9c1d-2e8a7b6c5d4e.jpg');
  assert.equal(safeUrl('/demo-cover/1.svg'), '/demo-cover/1.svg');
  for (const bad of ['javascript:alert(1)', 'data:image/svg+xml,<svg/>', '//evil.example/x.png', 'https://evil.example/x.png', 'cover:../x.jpg', 'cover:x.png', '', null, 5]) {
    assert.equal(safeUrl(bad), null, String(bad));
  }
  assert.equal(safeUrl('/a"b\\c.png'), '/ab' + 'c.png'); // aspas e barra invertida não escapam do url("...")
});

test('controle sintético: um evento por aperto, repetição só do direcional, soltar o aceitar', async () => {
  const { createPadTracker, syntheticPad, kindOf } = await load('input.js');
  const log = [];
  const devices = [];
  const tracker = createPadTracker({ press: (n, o) => log.push(`${n}${o?.repeat ? '*' : ''}`), release: (n) => log.push(`~${n}`) }, (d) => devices.push(d));
  const held = (...names) => [syntheticPad(names)];

  tracker.step(0, held('down'));
  tracker.step(100, held('down')); // ainda dentro do atraso da repetição (400 ms)
  assert.deepEqual(log, ['down']);
  tracker.step(400, held('down'));
  tracker.step(440, held('down')); // dentro do intervalo (80 ms)
  tracker.step(480, held('down'));
  assert.deepEqual(log, ['down', 'down*', 'down*']);
  tracker.step(500, held());
  tracker.step(520, held('down')); // soltou e apertou de novo: evento novo, sem repetição
  assert.equal(log.at(-1), 'down');

  log.length = 0;
  tracker.step(1000, held('accept', 'x'));
  tracker.step(2000, held('accept', 'x')); // aceitar e botões de ação nunca repetem
  tracker.step(2100, held());
  assert.deepEqual(log, ['accept', 'x', '~accept']);

  // direcional analógico e nomes do "standard mapping"
  log.length = 0;
  tracker.step(3000, [{ id: 'x', buttons: [], axes: [-0.9, 0] }]);
  tracker.step(3100, [{ id: 'x', buttons: [], axes: [0, 0] }]);
  tracker.step(3200, [{ id: 'x', buttons: [], axes: [0.1, 0.8] }]);
  assert.deepEqual(log, ['left', 'down']);

  assert.equal(devices.at(-1), 'xbox');
  assert.equal(kindOf({ id: 'Wireless Controller (STANDARD GAMEPAD Vendor: 054c)' }), 'playstation');
});
