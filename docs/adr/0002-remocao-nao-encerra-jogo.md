# ADR-0002: Remover o disco não encerra o jogo

**Status:** aceito

## Contexto
O original encerra o processo do jogo com `Stop-Process -Force` quando o disquete é removido. Isso causa perda de progresso se alguém esbarrar no drive ou se a leitura falhar.

## Decisão
A remoção do disco **nunca** encerra o jogo. O app usa a remoção só para **rearmar** o drive ([ADR-0013](0013-disco-sem-selecao-e-rearme.md)).

## Consequências
- O app não precisa rastrear o processo do jogo para encerrá-lo.
- O disco deixa de "segurar" a sessão; é um ritual de entrada, não uma trava.
