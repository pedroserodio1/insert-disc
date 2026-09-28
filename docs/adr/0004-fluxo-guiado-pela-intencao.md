# ADR-0004: Fluxo guiado pela intenção do usuário

**Status:** aceito; item 4 refinado pela [ADR-0019](0019-rejeicao-mantem-disco-e-atalhos.md) (a ejeção passa a ocorrer só em "Tentar outro disco")

## Contexto
No original, o disco comanda. Aqui, a biblioteca de capas é o centro da experiência.

## Decisão
1. O app mostra a biblioteca.
2. O usuário seleciona um jogo; o app entra em "aguardando disco" e abre a gaveta, se puder.
3. Ao detectar a mídia, lê o identificador e mostra o que identificou (rótulo, tipo de mídia, identificador).
4. Se o disco não for o esperado (vazio, de outro jogo, desconhecido, áudio, erro de leitura), o app explica o motivo e ejeta.
5. Se for o esperado, exibe o loading e lança.

## Consequências
- Máquina de estados em [UX-STATES](../UX-STATES.md).
- O disparo automático por inserção existe apenas como opção ([ADR-0013](0013-disco-sem-selecao-e-rearme.md)).
