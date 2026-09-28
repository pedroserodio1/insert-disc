# ADR-0008: Cadastro com gravação pelo app

**Status:** aceito

## Contexto
No original, o usuário edita o INI e copia arquivos à mão.

## Decisão
Fluxo: adicionar jogo → inserir disco virgem → o app confirma que está em branco → o usuário escolhe o jogo (Steam ou personalizado) → aviso explícito se for CD-R (gravação única) → o app grava o identificador → concluído.

Refinamentos definidos na Etapa 1:
- CD-R com conteúdo: sempre recusado.
- CD-RW com conteúdo: o app oferece apagar, com **confirmação dupla**, mostrando o conteúdo identificado.
- No drive falso, "gravar" gera um novo `.iso` reinserível.

## Consequências
- Detalhes em [BURNING](../BURNING.md); estados em [UX-STATES](../UX-STATES.md#cadastro-e-gravação).
- O catálogo só recebe o `disc_id` depois da verificação da gravação.
