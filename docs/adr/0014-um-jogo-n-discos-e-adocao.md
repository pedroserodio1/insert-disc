# ADR-0014: 1 jogo : N discos; adoção de disco desconhecido

**Status:** aceito (Etapa 1)

## Decisão
- Cada disco gravado tem um `disc_id` próprio. Um jogo pode ter vários discos (reserva, CD-RW de teste); um disco pertence a um único jogo.
- Um disco com `GAME.INI` válido e `id` desconhecido pode ser **adotado**: o usuário associa o `id` a um jogo que **ele escolhe**. O `name` do INI só sugere.

## Consequências
- O catálogo separa `game_id` de `disc_id` ([CATALOG-SPEC](../CATALOG-SPEC.md)).
- A adoção recupera discos depois de perder o catálogo ou trocar de PC, sem gastar CD-R.
- Regras de segurança da adoção em [SECURITY R4](../SECURITY.md#r4-o-catálogo-pertence-ao-usuário).
