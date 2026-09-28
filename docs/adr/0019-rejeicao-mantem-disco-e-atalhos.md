# ADR-0019: Rejeição mantém o disco; "Jogar Y"; jogo sem disco oferece gravar

**Status:** aceito; refina o item 4 da [ADR-0004](0004-fluxo-guiado-pela-intencao.md)

## Contexto
A revisão independente da documentação mostrou que ejetar o disco ao rejeitar tornava a adoção inalcançável (a tela saía antes de o usuário escolher). Também faltavam os casos "disco de outro jogo da estante" e "jogo sem nenhum disco".

## Decisão
1. **Rejeitar não ejeta.** A tela de rejeição mantém o disco no drive. O app ejeta quando o usuário escolhe "Tentar outro disco"; voltar à biblioteca deixa o disco no drive, desarmado.
2. **Disco de outro jogo (`OTHER_GAME`):** a rejeição oferece "Jogar Y" além de "Tentar outro disco". É uma intenção explícita do usuário e o que roda continua vindo do catálogo ([ADR-0006](0006-catalogo-local-define-execucao.md)).
3. **Jogo sem disco:** selecionar um jogo sem discos mostra "ainda não tem disco" com a ação "Gravar disco" (cadastro com o jogo pré-escolhido), sem abrir a gaveta.

## Consequências
- Estados `NO_DISC_YET` e ação `play_other` em [UX-STATES](../UX-STATES.md).
- Cenários C21, C22 e C30 em [TESTING-WITH-ISO](../TESTING-WITH-ISO.md#cenários).
