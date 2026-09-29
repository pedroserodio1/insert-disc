## O que muda e por quê

<!-- Uma mudança por PR. Link para a issue ou o item do backlog (ex.: A6). -->

## Como foi verificado

- [ ] `cargo test --workspace` e `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `node --test ui/test/ui.test.js`, `node scripts/check-contrast.js` e `node scripts/check-docs.js` (se mexeu na UI ou na doc)
- [ ] Teste novo que falharia sem a mudança
- [ ] Textos novos nos dois idiomas; cores só por token
- [ ] Contradiz algum ADR? Se sim, o novo ADR está neste PR
