# Política de segurança

## Reportar uma vulnerabilidade

**Não abra uma issue pública.** Use o [relato privado de vulnerabilidades do GitHub](https://github.com/pedroserodio1/insert-disc/security/advisories/new) deste repositório. Inclua o que você fez, o que esperava e o que aconteceu, e a versão ou o commit.

Respondemos em até 7 dias. O projeto é mantido por uma pessoa, em tempo livre: não há SLA de correção, mas problemas que permitam **executar código arbitrário a partir de um disco ou de um arquivo de estante** são tratados como prioridade.

## O que está no escopo

O modelo de ameaças está em [docs/SECURITY.md](docs/SECURITY.md). Em resumo: só o catálogo local define o que é executado; o disco guarda um `GAME.INI` mínimo e nunca comandos; capas são validadas e reencodadas; a interface não recebe caminhos para executar.

## Versões

Ainda não há release. Correções entram em `main`.
