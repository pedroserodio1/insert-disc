# Visão

## Problema

A biblioteca digital de jogos é uma lista infinita, sem presença física. O projeto devolve o ritual do console: pegar a caixa, abrir a gaveta, colocar o disco e ver o jogo abrir. E faz isso sem mudar onde os jogos estão instalados nem como são comprados.

## O que o projeto é

- Um aplicativo desktop com visual de console, navegável por controle (gamepad).
- Uma **chave física** por jogo: um CD-R/CD-RW com um identificador e uma etiqueta impressa.
- Um **catálogo local**, que é a única fonte de verdade sobre o que cada disco lança.

## O que o projeto não é

| Fora do escopo | Por quê |
|---|---|
| DRM, antipirataria ou proteção contra cópia | O disco é só uma chave de conveniência. Copiar um disco apenas cria outra chave válida. |
| Distribuir jogos em mídia | O jogo continua instalado no PC; o disco não carrega o jogo. |
| Loja, conta online ou sincronização em nuvem (fase 1) | Ver [OPEN-QUESTIONS Q5](OPEN-QUESTIONS.md#q5-conta-em-nuvem). |
| Encerrar o jogo quando o disco é removido | [ADR-0002](adr/0002-remocao-nao-encerra-jogo.md). |

## Princípios

1. **Intenção do usuário antes do disco.** O disco confirma uma escolha; ele não comanda o app sozinho, a menos que o usuário ligue esse modo ([ADR-0004](adr/0004-fluxo-guiado-pela-intencao.md), [ADR-0013](adr/0013-disco-sem-selecao-e-rearme.md)).
2. **O disco nunca define o que é executado** ([ADR-0006](adr/0006-catalogo-local-define-execucao.md)).
3. **O mínimo no disco** ([ADR-0005](adr/0005-disco-guarda-o-minimo.md)).
4. **Núcleo portátil e plataforma atrás de interfaces** ([ADR-0010](adr/0010-windows-primeiro-nucleo-portatil.md)).
5. **Degradar com aviso, sem falhar em silêncio.** Por exemplo, a gaveta que não abre por software gera uma instrução na tela.
6. **Testável sem hardware.** Há um drive falso alimentado por `.iso` ([DRIVE-LAYER](DRIVE-LAYER.md)).

## Diferenças em relação ao Reset Floppy Game System

Comportamento do original conforme o [artigo](https://resethub.com.br/2026/09/13/como-montar-seu-proprio-reset-floppy-game-system/).

| Aspecto | Original | Este projeto |
|---|---|---|
| Mídia | Disquete no drive `A:` | CD-R/CD-RW em drive USB slim de gaveta ([ADR-0001](adr/0001-midia-cd-r-drive-usb-slim.md)) |
| Conteúdo do disco | `GAME.INI` com NAME, STEAMID, PROCESS, COVER e DISKID (opcional) | `GAME.INI` com apenas `id` e `name` ([ADR-0012](adr/0012-game-ini-minimo-e-label-informativo.md)) |
| Quem define o que roda | O disco (o AppID fica no INI) | O catálogo local ([ADR-0006](adr/0006-catalogo-local-define-execucao.md)) |
| Disparo | Inserir o disco lança o jogo | O usuário seleciona o jogo e depois insere o disco. "Lançar ao inserir" é opcional ([ADR-0013](adr/0013-disco-sem-selecao-e-rearme.md)) |
| Remoção | `Stop-Process -Force` no processo do jogo | Não encerra nada; só rearma ([ADR-0002](adr/0002-remocao-nao-encerra-jogo.md)) |
| Detecção | Verificação a cada 100 ms | A definir por spike ([W2](RISKS-AND-SPIKES.md#w2-detecção-de-inserção-e-remoção)) |
| Interface | Janela WPF 1280×720 com animação | App Tauri com visual de console, controle, janela ou tela cheia ([ADR-0009](adr/0009-interface-tauri-console-gamepad.md)) |
| Tipos de jogo | Steam | Steam e personalizado (executável, emulador, launcher) ([ADR-0007](adr/0007-tipos-de-jogo.md)) |
| Cadastro | Manual (editar o INI e copiar a capa) | Pelo app, com gravação do disco ([ADR-0008](adr/0008-cadastro-com-gravacao-pelo-app.md)) |
| Plataforma | Windows (PowerShell) | Windows na fase 1; Linux depois |

## Projetos relacionados

Encontrados em 2026-09-28. Servem de referência; nenhum código deles é usado.

| Projeto | Proposta |
|---|---|
| [media-deck](https://github.com/antonio-abrantes/media-deck) | Chaves de mídia física (disquete ou disco) para jogos de PC no Windows, com launcher retrô |
| [DiscLauncher](https://github.com/CoachSludge/DiscLauncher) | Lançar jogos a partir de um CD |
| [SWM-Launcher](https://github.com/SharkWaveMedia/SWM-Launcher) | Lançar jogos de PC a partir de disquetes |
