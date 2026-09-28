# ADR-0005: O disco guarda o mínimo

**Status:** aceito

## Contexto
Tudo que fica no disco é imutável (CD-R) e não confiável (mídia removível).

## Decisão
O disco guarda apenas o estritamente necessário para identificá-lo. Nome, capa, comando de lançamento e demais metadados ficam no catálogo do PC. Conteúdo concreto em [ADR-0012](0012-game-ini-minimo-e-label-informativo.md).

## Consequências
- Mudar como um jogo é lançado não exige regravar o disco.
- Sem o catálogo, o disco é anônimo. Mitigações em [CATALOG-SPEC](../CATALOG-SPEC.md#perda-e-recuperação).
