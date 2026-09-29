# Insert Disc

Lançador de jogos por mídia física, open source. Você escolhe o jogo numa biblioteca com visual de console, insere o CD correspondente e o jogo abre. O CD é apenas uma chave física: o jogo continua instalado no PC.

Inspirado no [Reset Floppy Game System](https://resethub.com.br/2026/09/13/como-montar-seu-proprio-reset-floppy-game-system/), com diferenças deliberadas descritas em [VISION.md](docs/VISION.md).

## Status

| Item | Estado |
|---|---|
| Fase | **1 (em andamento):** núcleo em Rust, drive falso com ISO e interface funcionando de ponta a ponta no navegador. **Ainda não há** drive real, gravação real, app Tauri empacotado nem leitura de controle validada. Veja o [ROADMAP](docs/ROADMAP.md). |
| Plataforma alvo | Windows (fase 1). Linux numa fase posterior, ainda a confirmar. |
| Licença | MIT OR Apache-2.0, à sua escolha ([LICENSE-MIT](LICENSE-MIT), [LICENSE-APACHE](LICENSE-APACHE), [ADR-0017](docs/adr/0017-licenca-mit-apache.md)) |
| Nome | Insert Disc ([ADR-0020](docs/adr/0020-nome-insert-disc.md)) |

## Como rodar

Precisa de Rust (toolchain estável). A interface roda no navegador contra o núcleo real, com o **drive falso** (sem hardware):

```bash
cargo run -p insert-disc-host
```

Abra <http://127.0.0.1:5173/> (com `?dev=1` aparece o painel que simula discos, falhas e o drive). Teclado: setas, Enter, Esc, `N` (adicionar jogo), `O` (opções), `M` (configurações). Um controle também funciona (Gamepad API).

```bash
cargo test                     # núcleo e API (57 + testes da camada host)
cargo clippy --all-targets -- -D warnings
node scripts/check-docs.js     # links e âncoras da documentação
```

O lançador do modo de desenvolvimento **só registra** o que executaria; nenhum jogo é aberto.

### App desktop (Tauri) e teste do controle

```bash
cd apps/desktop && cargo run
```

Abre a mesma UI numa janela do WebView2 (com o drive falso; `Ctrl+Shift+D` mostra o painel de simulação). É o jeito de **testar o controle de verdade** (spike [W1](docs/RISKS-AND-SPIKES.md#w1-gamepad-no-webview2)): conecte um controle, clique na janela e navegue. Se a Gamepad API não entregar os botões nessa janela, o plano B é ler o controle no Rust com o `gilrs` ([spikes/w1-gamepad](spikes/w1-gamepad/README.md) compara as duas fontes lado a lado).

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
| [BACKLOG-SEM-DEPENDENCIAS](docs/BACKLOG-SEM-DEPENDENCIAS.md) | O que falta e dá para fazer sem drive, controle nem nada externo, e o que fica bloqueado |
| [OPEN-QUESTIONS](docs/OPEN-QUESTIONS.md) | Tudo o que está em aberto |
| [HARDWARE-AND-MATERIALS](docs/HARDWARE-AND-MATERIALS.md) | Lista de compras de referência |
| [adr/](docs/adr/README.md) | Registros de decisão |

## Contribuindo

Salvo indicação explícita em contrário, qualquer contribuição enviada intencionalmente para inclusão neste projeto, conforme definido na licença Apache-2.0, é licenciada como MIT OR Apache-2.0, sem termos ou condições adicionais.

Problemas de segurança: veja [SECURITY](docs/SECURITY.md#reportar-vulnerabilidades).
