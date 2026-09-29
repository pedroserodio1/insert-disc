// E2: contraste dos tokens de cor (docs/FRONTEND-DESIGN.md §3.1) e "nenhuma cor fora de token no CSS".
// Sem dependências: node scripts/check-contrast.js
const fs = require('fs');
const path = require('path');

const root = path.join(__dirname, '..');
const tokensCss = fs.readFileSync(path.join(root, 'ui/css/tokens.css'), 'utf8');
const appCss = fs.readFileSync(path.join(root, 'ui/css/app.css'), 'utf8');

const tokens = {};
for (const m of tokensCss.matchAll(/--([a-z-]+):\s*#([0-9a-f]{6})\s*;/gi)) tokens[m[1]] = [0, 2, 4].map((i) => parseInt(m[2].slice(i, i + 2), 16));

const lin = (c) => ((c /= 255) <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
const lum = ([r, g, b]) => 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
const ratio = (a, b) => {
  const [x, y] = [lum(tokens[a]), lum(tokens[b])];
  return (Math.max(x, y) + 0.05) / (Math.min(x, y) + 0.05);
};

let bad = 0;
const fail = (msg) => { console.error('ERRO:', msg); bad++; };

// pares permitidos para texto e bordas (tabela de §3.1): mínimo 4,5:1
const allowed = [
  ['etiqueta', 'azo'], ['policarbonato', 'azo'], ['ftalocianina', 'azo'], ['cianina', 'azo'], ['laser', 'azo'],
  ['azo', 'etiqueta'], ['grafite', 'etiqueta'], ['laser-escuro', 'etiqueta'],
];
for (const [fg, bg] of allowed) {
  if (!tokens[fg] || !tokens[bg]) { fail(`token ausente: ${fg} ou ${bg}`); continue; }
  const r = ratio(fg, bg);
  if (r < 4.5) fail(`${fg} sobre ${bg} = ${r.toFixed(2)} (< 4,5)`);
}
// pares proibidos: se algum passar a ter contraste alto, a tabela do design ficou desatualizada
for (const fg of ['ftalocianina', 'policarbonato', 'cianina', 'laser']) {
  if (ratio(fg, 'etiqueta') >= 3) fail(`${fg} sobre etiqueta deixou de ser < 3: atualizar §3.1`);
}

// cores literais no CSS: só se forem um token (com transparência) ou neutras do sistema
const known = new Set(Object.values(tokens).map((c) => c.join(' ')));
const hex = (h) => (h.length === 3 ? [...h].map((c) => parseInt(c + c, 16)) : [0, 2, 4].map((i) => parseInt(h.slice(i, i + 2), 16))).join(' ');
const neutral = new Set(['0 0 0', '255 255 255']);
const lines = appCss.split('\n');
lines.forEach((line, i) => {
  if (/^\.devpanel\b/.test(line)) return; // painel de desenvolvimento, fora do produto
  for (const m of line.matchAll(/#([0-9a-f]{3}|[0-9a-f]{6})\b/gi)) {
    const v = hex(m[1]);
    if (!known.has(v) && !neutral.has(v)) fail(`app.css:${i + 1}: cor literal #${m[1]} fora dos tokens`);
  }
  for (const m of line.matchAll(/rgba?\(\s*(\d+)[\s,]+(\d+)[\s,]+(\d+)/g)) {
    const v = `${m[1]} ${m[2]} ${m[3]}`;
    if (!known.has(v) && !neutral.has(v)) fail(`app.css:${i + 1}: rgb(${v}) fora dos tokens`);
  }
});

if (bad) process.exit(1);
console.log('ok: contraste dos tokens e cores do CSS');
