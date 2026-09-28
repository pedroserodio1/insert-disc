# ADR-0016: Tela de loading com duração mínima configurável

**Status:** aceito (Etapa 1)

## Contexto
A leitura de um disco de poucos bytes é quase instantânea, e o lançamento via Steam não dá retorno de progresso. Sem uma tela intermediária, a experiência perde o "ritual" de console.

## Decisão
Todo lançamento (por seleção ou automático) passa por uma tela de loading com animação e a capa do jogo. A tela dura no mínimo `loading_min_ms`, configurável (0 permitido), mesmo que o trabalho real termine antes. O loading é, em parte, cenográfico.

## Consequências
- Estado `LAUNCHING` em [UX-STATES](../UX-STATES.md).
- Valor padrão em [Q11](../OPEN-QUESTIONS.md#q11-timeouts).
