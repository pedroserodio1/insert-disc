# Spike W1: gamepad no WebView2

Protótipo **descartável** do spike [W1](../../docs/RISKS-AND-SPIKES.md#w1-gamepad-no-webview2). Compara, na mesma janela e na mesma linha do tempo, a **Gamepad API** do WebView2 (coluna esquerda) com o **gilrs** no Rust (coluna direita).

## Rodar

```bash
cargo run
```

Requer Rust (MSVC) e o runtime do WebView2 (já vem no Windows 11). Na primeira compilação, leva alguns minutos.

## Protocolo de teste

Para cada controle disponível (ideal: um XInput tipo Xbox e um DirectInput genérico), faça os passos abaixo e anote na tabela de resultados.

| # | Passo | O que observar |
|---|---|---|
| 1 | Abra o app com o controle **já conectado** | As duas colunas mostram o controle? (A Gamepad API costuma só aparecer depois do primeiro botão.) |
| 2 | Aperte A/B/X/Y, direcional e analógicos | Os eventos aparecem nas duas colunas? A linha "chegou N ms antes" indica qual fonte é mais rápida. |
| 3 | Desconecte e reconecte o controle | `disconnected` / `connected` nas duas colunas? |
| 4 | Aperte F11 (tela cheia) e repita o passo 2 | Algo muda em tela cheia? |
| 5 | Clique em outra janela (app sem foco) e aperte botões | As entradas continuam chegando? Linhas em vermelho = "janela sem foco". A doc exige **ignorar** entrada sem foco ([SECURITY R10](../../docs/SECURITY.md#r10-entrada-só-com-foco)); aqui só medimos o comportamento. |
| 6 | Volte ao app (Alt+Tab ou clique) e aperte botões | As duas fontes voltam a funcionar sem reiniciar? |
| 7 | Abra um jogo por cima, jogue alguns segundos, volte | Idem ao 6. |

## Critério (da doc)

- **Sucesso:** uma das fontes entrega todos os eventos de forma confiável, sem DevTools, nas condições acima. Ela vira ADR.
- **Falha:** nenhuma funciona com o app em primeiro plano. Nesse caso, reavaliar a stack de UI.

## Resultados

Preencher e commitar junto com a conclusão.

| Controle | Passo | Gamepad API | gilrs | Observação |
|---|---|---|---|---|
| | 1 | | | |
| | 2 | | | |
| | 3 | | | |
| | 4 | | | |
| | 5 | | | |
| | 6 | | | |
| | 7 | | | |

**Conclusão:** _pendente_
