# Como contribuir

Insert Disc é um lançador de jogos por CD gravável (veja o [README](README.md) e a [VISION](docs/VISION.md)). Contribuições são bem-vindas; este arquivo diz o mínimo para não perder tempo.

## Antes de começar

- **Decisões** estão em [docs/adr/](docs/adr/README.md) e só mudam com um novo ADR. **Perguntas em aberto** estão em [OPEN-QUESTIONS](docs/OPEN-QUESTIONS.md).
- O que dá para fazer hoje, sem drive físico, controle ou certificado, está em [BACKLOG-SEM-DEPENDENCIAS](docs/BACKLOG-SEM-DEPENDENCIAS.md). Abra uma issue antes de pegar algo grande.
- Segurança: leia [docs/SECURITY.md](docs/SECURITY.md). Vulnerabilidades **não** vão em issue pública; siga o [SECURITY.md](SECURITY.md).

## Rodar e testar

```bash
cargo test --workspace                          # núcleo e camada host
cargo clippy --workspace --all-targets -- -D warnings
node --test ui/test/ui.test.js                  # textos e utilitários da UI (sem npm)
node scripts/check-contrast.js                  # contraste dos tokens e cores fora de token
node scripts/check-docs.js                      # links e âncoras da documentação
cd apps/desktop && cargo run -- --demo          # app desktop com a estante de exemplo
```

O teste com uma ISO montada de verdade (só Windows) fica fora do `cargo test` normal:
`cargo test -p insert-disc-host --test windrive -- --ignored`.

## O que esperamos de um PR

- Uma mudança por PR, com o teste que falharia sem ela. Regra de negócio fica no núcleo (`crates/insert-disc-core`), com teste de fluxo; a UI só renderiza o instantâneo.
- Texto visível vai em `ui/js/i18n.js`, nos **dois** idiomas (pt-BR e en). Cores só por token (`ui/css/tokens.css`).
- Código do sistema (drive, lançamento, Steam) fica em `crates/insert-disc-host`. Nada de shell: argumentos vão como lista ([SECURITY R3](docs/SECURITY.md)).
- Se a mudança contradiz um ADR, o PR inclui o novo ADR.
- Commits em português, no imperativo curto (`feat:`, `fix:`, `docs:`, `test:`).

## Licença

MIT OU Apache-2.0, à sua escolha ([ADR-0017](docs/adr/0017-licenca-mit-apache.md)). Ao enviar um PR você concorda em licenciar a contribuição da mesma forma.
