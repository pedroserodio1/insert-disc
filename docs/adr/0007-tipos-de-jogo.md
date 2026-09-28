# ADR-0007: Tipos de jogo — Steam e personalizado

**Status:** aceito

## Contexto
O original só suporta Steam. O autor também usa emuladores e launchers de terceiros.

## Decisão
- **Steam:** lançamento via `steam://run/<AppID>`, com importação da biblioteca instalada.
- **Personalizado:** executável + argumentos (inclui emuladores com ROM e launchers).
- **Perfis de emulador** serão explorados em documento, **sem decisão** ([CATALOG-SPEC](../CATALOG-SPEC.md#perfis-de-emulador-exploração-sem-decisão), [Q4](../OPEN-QUESTIONS.md#q4-perfis-de-emulador)).

## Consequências
- Riscos em [W6](../RISKS-AND-SPIKES.md#w6-integração-com-a-steam) e [W7](../RISKS-AND-SPIKES.md#w7-lançamento-de-processos).
- A escolha exata do esquema de URI (`run`, `rungameid` ou `launch`) fica para o W6.
