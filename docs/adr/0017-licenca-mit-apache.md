# ADR-0017: Licença MIT OR Apache-2.0

**Status:** aceito; resolve a [Q2](../OPEN-QUESTIONS.md#q2-licença)

## Contexto
A licença precisava ser definida antes do primeiro commit público ([ADR-0011](0011-decisoes-em-tentativa-e-teste.md)). Opções avaliadas: MIT/Apache-2.0, GPL-3.0, MPL-2.0 e licenças não comerciais.

Foi considerado que a GPL-3.0 **não proíbe venda**; apenas obriga a manter o código aberto. Proibir uso comercial exigiria uma licença não comercial, que deixa de ser open source (OSI).

## Decisão
Licença dupla **MIT OR Apache-2.0**, à escolha de quem usa (`LICENSE-MIT` e `LICENSE-APACHE` na raiz).

## Consequências
- Compatível com o ecossistema Rust e Tauri, e com `gilrs` (MIT/Apache-2.0).
- A Apache-2.0 acrescenta uma concessão explícita de patentes.
- Terceiros podem criar derivados fechados e vendê-los. Isso foi aceito.
- Na fase Linux, gravadores GPL (`xorriso`, `wodim`) devem ser **chamados como processo externo instalado pelo sistema**, não empacotados junto ([BURNING](../BURNING.md#ferramentas-candidatas-windows-primeiro)).
- Contribuições são aceitas sob os mesmos termos (cláusula padrão de contribuição a definir no `CONTRIBUTING`, quando existir).
