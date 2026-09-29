// Ponte com o núcleo (docs/UI-CONTRACT.md). No Tauri usa `invoke`; no navegador (servidor de
// desenvolvimento) usa fetch para /api. A UI só conhece estas funções.

const tauri = window.__TAURI__?.core;

async function http(method, path, body) {
  const res = await fetch(path, { method, headers: { 'Content-Type': 'application/json' }, body: body === undefined ? undefined : JSON.stringify(body) });
  if (!res.ok && res.status !== 400) throw new Error(`${path}: ${res.status}`);
  return res.json();
}

export const bridge = {
  kind: tauri ? 'tauri' : 'http',
  snapshot: () => (tauri ? tauri.invoke('snapshot').then(JSON.parse) : http('GET', '/api/snapshot')),
  intent: (intent) => (tauri ? tauri.invoke('intent', { intent }) : http('POST', '/api/intent', intent)),
  steamGames: () => (tauri ? tauri.invoke('steam_games') : http('GET', '/api/steam')),
  setCover: (request) => (tauri ? tauri.invoke('set_cover', { request }) : http('POST', '/api/cover', request)),
  devState:() => (tauri ? tauri.invoke('dev_state') : http('GET', '/api/dev')),
  dev: (cmd) => (tauri ? tauri.invoke('dev', { cmd }) : http('POST', '/api/dev', cmd)),
  exportCatalog: () => (tauri ? tauri.invoke('export_catalog') : fetch('/api/export').then((r) => r.text())),
  importCatalog: (json) => (tauri ? tauri.invoke('import_catalog', { json }) : fetch('/api/import', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: json })),
};
