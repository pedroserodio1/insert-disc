# Insert Disc

Lançador de jogos por mídia física, open source. Você escolhe o jogo numa biblioteca com visual de console, insere o CD correspondente e o jogo abre. O CD é apenas uma chave física: o jogo continua instalado no PC.

Inspirado no [Reset Floppy Game System](https://resethub.com.br/2026/09/13/como-montar-seu-proprio-reset-floppy-game-system/), com diferenças deliberadas descritas em [VISION.md](docs/VISION.md).

## Status

| Item | Estado |
|---|---|
| Fase | **0: documentação.** Ainda não há código. |
| Plataforma alvo | Windows (fase 1). Linux numa fase posterior, ainda a confirmar. |
| Licença | MIT OR Apache-2.0, à sua escolha ([LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE), [ADR-0017](docs/adr/0017-licenca-mit-apache.md)) |
| Nome | Insert Disc ([ADR-0020](docs/adr/0020-nome-insert-disc.md)) |

## Como navegar

Convenção usada em todos os documentos:

- **Decisão:** registrada em [docs/adr/](docs/adr/README.md). Só muda com um novo ADR.
- **Hipótese:** afirmação técnica ainda não verificada. Vira spike em [RISKS-AND-SPIKES](docs/RISKS-AND-SPIKES.md).
- **Pergunta em aberto:** está em [OPEN-QUESTIONS](docs/OPEN-QUESTIONS.md), com opções e recomendação.

| Documento | Para quê |
|---|---|
| [VISION](docs/VISION.md) | Problema, princípios, escopo e diferenças em relação ao original |
| [ARCHITECTURE](docs/ARCHITECTURE.md) | Componentes, fronteira entre núcleo portátil e código de plataforma, fluxo de dados |
| [UX-STATES](docs/UX-STATES.md) | Máquina de estados da tela |
| [UI-CONTRACT](docs/UI-CONTRACT.md) | Protocolo entre a interface e o núcleo em Rust |
| [FRONTEND-DESIGN](docs/FRONTEND-DESIGN.md) | Design visual e de interação: tokens, componentes, telas, controle, textos |
| [DISC-FORMAT](docs/DISC-FORMAT.md) | O que vai no disco e o que não vai |
| [CATALOG-SPEC](docs/CATALOG-SPEC.md) | Catálogo local: campos, tipos de jogo, perfis de emulador, opções de formato |
| [DRIVE-LAYER](docs/DRIVE-LAYER.md) | Abstração de drive e suas implementações (real e falsa) |
| [BURNING](docs/BURNING.md) | Fluxo de cadastro e gravação |
| [SECURITY](docs/SECURITY.md) | Modelo de ameaça e regras |
| [TESTING-WITH-ISO](docs/TESTING-WITH-ISO.md) | Como testar sem drive, usando `.iso` |
| [PLATFORMS](docs/PLATFORMS.md) | O que é Windows, o que muda no Linux, o que é portátil |
| [RISKS-AND-SPIKES](docs/RISKS-AND-SPIKES.md) | Riscos como spikes com critério de sucesso e falha |
| [ROADMAP](docs/ROADMAP.md) | Ordem sugerida de trabalho |
| [OPEN-QUESTIONS](docs/OPEN-QUESTIONS.md) | Tudo o que está em aberto |
| [HARDWARE-AND-MATERIALS](docs/HARDWARE-AND-MATERIALS.md) | Lista de compras de referência |
| [adr/](docs/adr/README.md) | Registros de decisão |

## Contribuindo

Salvo indicação explícita em contrário, qualquer contribuição enviada intencionalmente para inclusão neste projeto, conforme definido na licença Apache-2.0, é licenciada como MIT OR Apache-2.0, sem termos ou condições adicionais.

Problemas de segurança: veja [SECURITY](docs/SECURITY.md#reportar-vulnerabilidades).
