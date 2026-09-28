# ADR-0012: `GAME.INI` com `id` + `name`; rótulo informativo

**Status:** aceito (Etapa 1)

## Contexto
O autor quer manter o `GAME.INI` do original e usar o rótulo do volume com o nome do jogo. O `GAME.INI` do original, porém, contém AppID, processo e capa, o que conflita com as [ADR-0005](0005-disco-guarda-o-minimo.md) e [ADR-0006](0006-catalogo-local-define-execucao.md).

## Decisão
- `GAME.INI` na raiz, com apenas `id` (UUID aleatório, por disco) e `name`.
- O app decide **somente** pelo `id`. O `name` serve para exibição e recuperação.
- O rótulo do volume é derivado do nome do jogo e é **apenas informativo**. Divergência entre o rótulo e o catálogo não rejeita o disco.

## Consequências
- O limite de 15 caracteres com charset restrito do IMAPI2 obriga a sanitizar o rótulo ([DISC-FORMAT](../DISC-FORMAT.md#rótulo-do-volume), [Q15](../OPEN-QUESTIONS.md#q15-regra-de-abreviação-do-rótulo)).
- Um INI do projeto original num disco é classificado pelo `id` (se houver); as demais chaves são ignoradas.
