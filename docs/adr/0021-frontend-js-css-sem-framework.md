# ADR-0021: Front-end em JavaScript e CSS puros, sem framework nem etapa de build

**Status:** aceito; resolve a [Q17](../OPEN-QUESTIONS.md#q17-framework-do-frontend)

## Contexto
A interface é uma projeção do estado que vem do núcleo em Rust ([UI-CONTRACT](../UI-CONTRACT.md)): ela renderiza um instantâneo e envia intenções, sem regra de negócio. O design exige componentes próprios (estante 3D, disco, fichas), sem kit pronto ([FRONTEND-DESIGN](../FRONTEND-DESIGN.md)). Opções avaliadas: TypeScript puro, Svelte, Solid, React.

## Decisão
JavaScript (módulos ES) e CSS puros, sem framework e sem etapa de build. Toda a UI fica em `ui/` e é servida como arquivos estáticos (pelo Tauri no app; pelo servidor de desenvolvimento no navegador).

## Consequências
- Nenhuma dependência de npm: nada a auditar, nada a atualizar, e o build do app é só Rust.
- O ciclo de desenvolvimento é recarregar a página; o servidor de desenvolvimento roda a UI inteira com o drive falso ([README](../../README.md#como-rodar)).
- Sem checagem de tipos. Mitigações: telas pequenas e separadas por estado, o contrato JSON documentado e os testes de ponta a ponta da API no host.
- Todo texto vindo do disco ou do catálogo entra na página só por `createTextNode` (nunca como HTML), em cumprimento de [SECURITY R7](../SECURITY.md#r7-superfície-do-tauri).
- Se a UI crescer a ponto de o gerenciamento de estado local ficar difícil, reavaliar (um novo ADR pode adotar TypeScript ou Svelte; as telas já são funções do instantâneo).
