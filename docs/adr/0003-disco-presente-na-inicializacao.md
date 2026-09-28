# ADR-0003: Disco presente na inicialização não dispara nada

**Status:** aceito; refinado pela [ADR-0013](0013-disco-sem-selecao-e-rearme.md)

## Contexto
Se o app abre com um disco no drive, lançar o jogo sozinho seria surpresa (ex.: o PC reiniciou com o disco dentro).

## Decisão
Um disco presente no boot fica **desarmado**: não lança o jogo nem foca a biblioteca até ser retirado e recolocado.

**Exceção (ADR-0013):** se o usuário **selecionar** o jogo desse disco, ele lança normalmente, porque a intenção explícita prevalece.

## Consequências
- A regra vale só para disparos automáticos (modos `focus` e `launch`).
- Cenário C13 em [TESTING-WITH-ISO](../TESTING-WITH-ISO.md#cenários).
