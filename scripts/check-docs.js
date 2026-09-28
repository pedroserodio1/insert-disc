// Verifica links relativos e âncoras (slug do GitHub) em README.md e docs/**.
// Uso: node scripts/check-docs.js
const fs = require('fs'), path = require('path');
const root = path.resolve(__dirname, '..');
const walk = (d) => fs.readdirSync(d, { withFileTypes: true }).flatMap((e) => (e.isDirectory() ? walk(path.join(d, e.name)) : e.name.endsWith('.md') ? [path.join(d, e.name)] : []));
const files = [path.join(root, 'README.md'), ...walk(path.join(root, 'docs'))];
const slug = (h) => h.trim().toLowerCase().replace(/<[^>]+>/g, '').replace(/[^\p{L}\p{N}\s_-]/gu, '').replace(/\s/g, '-');
const heads = {};
for (const f of files) {
  const s = fs.readFileSync(f, 'utf8').replace(/```[\s\S]*?```/g, '');
  heads[f] = new Set(s.split('\n').filter((l) => /^#{1,6} /.test(l)).map((l) => slug(l.replace(/^#+ /, ''))));
}
let bad = 0;
for (const f of files) {
  const s = fs.readFileSync(f, 'utf8').replace(/```[\s\S]*?```/g, '');
  for (const m of s.matchAll(/\]\(([^)\s#]*)(?:#([^)\s]+))?\)/g)) {
    const [, target, anchor] = m;
    if (/^[a-z]+:/.test(target)) continue;
    const file = target ? path.resolve(path.dirname(f), target) : f;
    if (target && !fs.existsSync(file)) { bad++; console.log('ARQUIVO', path.relative(root, f), '->', target); continue; }
    if (anchor && heads[file] && !heads[file].has(decodeURIComponent(anchor))) { bad++; console.log('ÂNCORA', path.relative(root, f), '->', target + '#' + anchor); }
  }
}
console.log(bad ? `${bad} problema(s)` : 'ok');
process.exit(bad ? 1 : 0);
