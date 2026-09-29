# Spike W8: tela cheia e foco

`spike-w8-focus` é um "jogo" fictício: abre uma janela própria, pede o primeiro plano como um jogo faria e registra o resultado em `W8_OUT`. Serve para medir como o app sai da frente ao lançar um jogo.

```bash
cd spikes/w8-focus && cargo build
```

Uso no teste ponta a ponta: um jogo `custom` no catálogo apontando para `target/debug/spike-w8-focus.exe` (argumento: segundos de vida), disco em ISO montada, app com `--real-drive --real-launch`.

## Resultado (2026-09-29)

- O lançador real abre o processo com a janela própria; a janela cria e pede o primeiro plano em ~370 ms, e do ponto de vista do jogo o pedido é atendido.
- **Comportamento adotado:** ao fim do loading (`LAUNCHING` → `LIBRARY`) o app sai de tela cheia e **minimiza**, e nunca chama `setFocus` sozinho. Verificado: a janela do app fica minimizada depois do loading. Ao voltar (Alt+Tab ou barra de tarefas), quem estava em tela cheia volta a ela.
- **Limitação da medição:** o PC de teste tem outros programas em primeiro plano (o autor joga e navega enquanto isso), então a ordem exata das janelas não é reproduzível; o teste não prova que o jogo ganha o foco em todo cenário. É a regra de falha do W8 ("minimizar sempre e nunca pedir foco sozinho") aplicada de saída.
- **Não medido:** Steam (`steam://run` abre a Steam e o jogo), fim do jogo por PID ou nome de processo ([Q6](../../docs/OPEN-QUESTIONS.md#q6-detectar-o-fim-do-jogo)), botão Guide do controle.
