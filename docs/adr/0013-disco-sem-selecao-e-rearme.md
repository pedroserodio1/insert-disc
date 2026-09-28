# ADR-0013: Disco sem seleção (`focus` / `launch`) e regras de rearme

**Status:** aceito (Etapa 1, revisado pelo autor)

## Contexto
As ADR-0002 e ADR-0003 foram pensadas para o modelo "o disco dispara". Com o fluxo por intenção ([ADR-0004](0004-fluxo-guiado-pela-intencao.md)), era preciso definir o que acontece quando um disco entra sem seleção, e quando o mesmo disco continua no drive.

## Decisão
1. **Seleção explícita sempre lança** se o disco certo estiver presente. Isso vale inclusive para disco presente no boot e para disco que ficou no drive depois de uma partida.
2. **Inserção sem seleção** (na biblioteca) é configurável em `on_disc_insert`:
   - `focus` (**padrão**): foca a capa do jogo e não lança;
   - `launch`: lança direto, passando pelo loading.
3. **Rearme** (só para disparos automáticos): um disco presente no boot fica desarmado; um lançamento automático desarma o drive; remover a mídia rearma. Um disco desarmado não dispara nada: o mesmo disco parado no drive não relança o jogo depois que ele fecha.

## Consequências
- Estados e tabela em [UX-STATES](../UX-STATES.md#inserção-sem-seleção-na-biblioteca).
- A segurança não muda: nos dois modos, o que roda vem do catálogo ([ADR-0006](0006-catalogo-local-define-execucao.md)).
