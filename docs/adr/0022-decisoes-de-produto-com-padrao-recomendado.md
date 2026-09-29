# ADR-0022: Decisões de produto fechadas com o padrão recomendado

**Status:** aceito; resolve as Q7, Q9, Q10, Q12, Q13, Q15, Q18, Q19, Q20, Q21, Q22, Q23 e Q24 de [OPEN-QUESTIONS](../OPEN-QUESTIONS.md)

## Contexto
Estas perguntas tinham uma recomendação na documentação e nenhuma dependia de informação externa (hardware, serviço, certificado). Algumas já estavam implementadas no código com esse padrão; as demais eram só falta de registro.

## Decisão
| Q | Decisão |
|---|---|
| Q7 | Drive falso em release **só por flag** de linha de comando (`--fake-drive`); em builds de depuração fica ligado. O modo `--demo` usa sempre pasta temporária e nunca toca os dados do usuário. |
| Q9 | `GAME.INI` sem campo de versão; chaves desconhecidas são ignoradas ([ADR-0012](0012-game-ini-minimo-e-label-informativo.md)). |
| Q10 | Disco inválido inserido sem seleção: aviso discreto, sem ejetar; `UNKNOWN` oferece "adotar". |
| Q12 | Falha na gravação mantém o jogo novo (sem discos) e descarta só o disco. |
| Q13 | `.bat` e `.cmd` permitidos como executável, com aviso e **sem argumentos**. |
| Q15 | Rótulo: maiúsculas, `A-Z 0-9 _`, sem acentos, espaços viram `_`, no máximo 15 caracteres, com prévia editável. |
| Q18 | Formulários de texto exigem teclado e mouse na fase 1; o fluxo "jogar" é 100% por controle. |
| Q19 | Importar estante **substitui** a atual, com backup automático antes. |
| Q20 | Primeira execução em janela; o modo de tela cheia é escolhido nas configurações e persiste. |
| Q21 | Ordem alfabética sem diferenciar maiúsculas nem acentos, sem tratar artigos; números antes das letras; LB/RB saltam para a próxima inicial existente. |
| Q22 | Instância única: abrir de novo traz a janela existente para a frente. |
| Q23 | Log em arquivo rotativo na pasta de dados; registra estados, classes e erros; nunca a chave de API; caminhos de executável só em nível de depuração. |
| Q24 | Idioma do sistema se for pt-BR ou en; qualquer outro cai em en. Plural por regra de plural da biblioteca ou tabela, nunca por concatenação. |

## Consequências
- Itens ainda por implementar (Q20, Q21 com acentos, Q22, Q23) estão no [backlog](../BACKLOG-SEM-DEPENDENCIAS.md) (A6, E7, E11).
- Q4, Q8, Q11 (timeouts medidos no drive) e Q14 continuam abertas: dependem de decisão de produto, hardware ou serviço externo.
