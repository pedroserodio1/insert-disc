# Registros de decisão (ADR)

Formato: Status · Contexto · Decisão · Consequências.

Regras:

- O conteúdo da decisão de um ADR aceito só muda com um novo ADR que o refine ou substitua.
- **Exceção permitida:** acrescentar ao ADR antigo uma linha de status ou nota apontando para o ADR que o refinou ou resolveu, sem alterar a decisão original.

Termos: **"Etapa 1"** = a rodada de entendimento e perguntas feita antes da escrita da documentação (não confundir com a "Fase 1" do [ROADMAP](../ROADMAP.md)).

| # | Decisão | Status |
|---|---|---|
| [0001](0001-midia-cd-r-drive-usb-slim.md) | CD-R/CD-RW em drive USB slim de gaveta; etiqueta de 115 mm | Aceito |
| [0002](0002-remocao-nao-encerra-jogo.md) | Remover o disco não encerra o jogo | Aceito |
| [0003](0003-disco-presente-na-inicializacao.md) | Disco presente na inicialização não dispara nada | Aceito (refinado pela 0013) |
| [0004](0004-fluxo-guiado-pela-intencao.md) | Fluxo guiado pela intenção do usuário | Aceito (item 4 refinado pela 0019) |
| [0005](0005-disco-guarda-o-minimo.md) | O disco guarda o mínimo | Aceito |
| [0006](0006-catalogo-local-define-execucao.md) | Só o catálogo local define o que é executado | Aceito |
| [0007](0007-tipos-de-jogo.md) | Tipos de jogo: Steam e personalizado | Aceito |
| [0008](0008-cadastro-com-gravacao-pelo-app.md) | Cadastro com gravação pelo app | Aceito |
| [0009](0009-interface-tauri-console-gamepad.md) | Interface Tauri com visual de console e controle | Aceito (sujeito ao W1) |
| [0010](0010-windows-primeiro-nucleo-portatil.md) | Windows primeiro; núcleo portátil | Aceito (nota: `GamepadSource`) |
| [0011](0011-decisoes-em-tentativa-e-teste.md) | Nome, licença e formato do catálogo em "tentativa e teste" | Aceito (licença resolvida pela 0017; nome pela 0020) |
| [0012](0012-game-ini-minimo-e-label-informativo.md) | `GAME.INI` com `id` + `name`; rótulo informativo | Aceito |
| [0013](0013-disco-sem-selecao-e-rearme.md) | Disco sem seleção configurável (`focus`/`launch`) e regras de rearme | Aceito |
| [0014](0014-um-jogo-n-discos-e-adocao.md) | 1 jogo : N discos; adoção de disco desconhecido | Aceito |
| [0015](0015-i18n-e-capas-online-opcionais.md) | i18n pt-BR/en; capas online opcionais com chave do usuário | Aceito |
| [0016](0016-tela-de-loading-duracao-minima.md) | Tela de loading sempre, com duração mínima configurável | Aceito |
| [0017](0017-licenca-mit-apache.md) | Licença MIT OR Apache-2.0 | Aceito |
| [0018](0018-direcao-visual-estante-de-cds.md) | Direção visual "Estante de CDs" | Aceito |
| [0019](0019-rejeicao-mantem-disco-e-atalhos.md) | Rejeição mantém o disco; "Jogar Y"; jogo sem disco oferece gravar | Aceito |
| [0020](0020-nome-insert-disc.md) | Nome do projeto: Insert Disc | Aceito |
