# ADR-0006: Só o catálogo local define o que é executado

**Status:** aceito

## Contexto
O projeto é open source: qualquer pessoa pode fabricar um disco. Se o disco definisse o que roda, um disco malicioso executaria código arbitrário.

## Decisão
O mapeamento `identificador → como lançar` vive **somente** no catálogo local, editável apenas pelo usuário. O disco nunca contribui com caminho, comando, argumento ou URI.

## Consequências
- Regras detalhadas em [SECURITY](../SECURITY.md).
- Diferença essencial em relação ao original ([VISION](../VISION.md#diferenças-em-relação-ao-reset-floppy-game-system)).
