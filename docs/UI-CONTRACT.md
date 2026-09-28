# Contrato UI ↔ núcleo

Como a UI (webview) e o núcleo (Rust) conversam. Objetivo: o implementador do front-end não inventar protocolo. Nomes são sugestões de serialização; a forma exata (JSON via eventos e comandos do Tauri) é detalhe de implementação, mas **os papéis abaixo são obrigatórios**.

## Princípios

1. **O núcleo decide, a UI renderiza.** A UI não conhece regras (armar, classificar, rejeitar). Ela mostra o estado e envia intenções.
2. **O núcleo não envia texto visível.** Envia códigos (estado, classe, motivo, tipo de erro); a UI traduz via i18n ([FRONTEND-DESIGN §9](FRONTEND-DESIGN.md#9-textos-pt-br--en)).
3. **A UI é dona do foco.** O núcleo só envia uma **dica de foco** pontual; nunca o foco contínuo.
4. **A UI nunca envia caminhos, comandos ou URIs para executar.** Envia `game_id`; o núcleo resolve pelo catálogo ([SECURITY R7](SECURITY.md#r7-superfície-do-tauri)).

## Núcleo → UI: instantâneo de estado

Enviado inteiro a cada mudança (simples e sem ambiguidade; o volume é pequeno).

| Campo | Conteúdo |
|---|---|
| `version` | Versão do contrato |
| `state` | Nome do estado ([UX-STATES](UX-STATES.md)) |
| `game` | Jogo em contexto (X): `game_id`, `name`, `kind`, `cover` (URL local servida pelo Tauri), `spine_color`, `disc_count` |
| `other_game` | Y, em `OTHER_GAME` / `KNOWN` |
| `media` | Última leitura: `label`, `physical`, `disc_id`, `ini_name`, `class` |
| `reason` | Código do motivo (`drive_removed`, `not_found`, `steam_missing`, `elevation_denied`, `write_error`, `verify_mismatch`, …) |
| `actions` | Ações válidas no estado, na ordem de exibição, com a padrão marcada (ex.: `[try_other*, adopt]`) |
| `progress` | 0–100 em `BURNING`/`ERASING`; ausente nos demais |
| `timing` | Em `LAUNCHING`: `min_ms` e `started_at` ([Tempo](#tempo)) |
| `capabilities` | Do drive (para textos como `wait.tray_manual`) |
| `drive` | `ok` \| `none` \| `removed` |
| `library` | Lista de jogos (ordem de exibição, [Q21](OPEN-QUESTIONS.md#q21-ordenação-da-biblioteca)) com os campos de `game` |
| `settings` | Valores atuais das configurações |
| `focus_hint` | `game_id` a focar, **uma vez** (modo `focus`, retorno de `LAUNCHED`, fim de cadastro); a UI aplica e ignora repetições do mesmo valor |
| `toast` | `{ code, params, id }`; a UI mostra uma vez por `id` |

## UI → núcleo: intenções

| Intenção | Parâmetros | Válida em |
|---|---|---|
| `select` | `game_id` | `LIBRARY` |
| `back` | — | onde a coluna B de [UX-STATES](UX-STATES.md) não é "—" |
| `action` | `id` (um dos `actions` do instantâneo) | onde houver ações |
| `hold_complete` | — | segundo passo de `REG_ERASE_CONFIRM` |
| `add_game` / `options` / `settings` | `game_id` quando aplicável | `LIBRARY` |
| `setting_change` | `key`, `value` | `SETTINGS` |
| `game_edit` | campos do jogo | formulários de jogo |
| `label_edit` | texto | `REG_LABEL_PREVIEW` |

O núcleo **ignora** intenções inválidas para o estado atual (sem erro para a UI). Isso cobre entradas atrasadas ou duplicadas.

## Tempo

- `loading_min_ms` é cronometrado pelo **núcleo**: ele fica em `LAUNCHING` até `max(tempo mínimo, disparo concluído)`.
- A UI recebe `min_ms` e ajusta a coreografia ([FRONTEND-DESIGN §6.5](FRONTEND-DESIGN.md#65-launching-o-momento-marcante)): se `min_ms` for menor que a coreografia completa, comprime; com `0`, não anima.
- `ficha-min` (tempo da ficha em `MATCH`) também é do núcleo.

## Entrada de controle

- Se o W1 escolher `gilrs`, o núcleo converte eventos do controle em **eventos de navegação** (`nav_up`, `nav_down`, `nav_left`, `nav_right`, `press_a`, `press_b`, `press_x`, `press_y`, `press_lb`, `press_rb`, `press_menu`, `hold_a_start`/`hold_a_end`) e os envia à UI, com o **tipo de controle** detectado (para os glifos).
- Se o W1 escolher a Gamepad API, a UI gera os mesmos eventos internamente. Em ambos os casos, o restante da UI consome só eventos de navegação.
- Nenhum evento de navegação é emitido sem foco de janela ([SECURITY R10](SECURITY.md#r10-entrada-só-com-foco)).
