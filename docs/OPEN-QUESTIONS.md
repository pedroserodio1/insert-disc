# Perguntas em aberto

Cada item traz as opções, uma recomendação e o que destrava a decisão. Ao decidir, registrar a decisão como ADR e marcar o item como resolvido aqui.

## Q1. Nome

**Resolvida:** "Insert Disc" ([ADR-0020](adr/0020-nome-insert-disc.md)).

## Q2. Licença

**Resolvida:** MIT OR Apache-2.0 ([ADR-0017](adr/0017-licenca-mit-apache.md)). Análise original mantida abaixo como histórico.

| Opção | Efeito |
|---|---|
| MIT ou Apache-2.0 (ou dupla) | Máxima adoção; compatível com `gilrs` (MIT/Apache-2.0, [docs.rs](https://docs.rs/gilrs/latest/gilrs/)) e com o ecossistema Rust e Tauri |
| GPL-3.0 | Obriga derivados a permanecerem abertos; facilita empacotar ferramentas GPL como `xorriso` (GPL, [Libburnia](https://en.wikipedia.org/wiki/Libburnia)), se isso for necessário |
| MPL-2.0 | Meio-termo (copyleft por arquivo) |

- **Critérios:** (1) compatibilidade com dependências reais depois dos spikes W1 e W5; (2) se o Linux vai **empacotar** ou apenas **chamar** um gravador GPL; (3) preferência do autor quanto a derivados fechados.
- **Recomendação:** MIT/Apache-2.0 dupla, se o W5 confirmar IMAPI2 no Windows e o Linux chamar `xorriso` como processo externo instalado pelo sistema. Isso não é aconselhamento jurídico.

## Q3. Formato do catálogo

Opções e critérios em [CATALOG-SPEC](CATALOG-SPEC.md#formato-em-aberto).

- **Recomendação provisória:** JSON único, com versão de esquema, escrita atômica e backups rotativos. É o mais simples de exportar e inspecionar. Reavaliar se a biblioteca passar de algumas centenas de jogos ou se surgir necessidade de consultas.

## Q4. Perfis de emulador

Opções em [CATALOG-SPEC](CATALOG-SPEC.md#perfis-de-emulador-exploração-sem-decisão).

- **Recomendação:** começar sem perfis (opção A) na fase 1c e medir a repetição na biblioteca real. Adotar B ou C se houver mais de 3 jogos por emulador.

## Q5. Conta em nuvem

Levantada durante a Etapa 1 e deixada fora da fase 1.

- **Opções:** (a) nada; (b) exportar e importar para uma pasta que o usuário sincroniza por conta própria (OneDrive, Dropbox); (c) conta e servidor do projeto.
- **Riscos de (c):** o catálogo define o que é executado ([ADR-0006](adr/0006-catalogo-local-define-execucao.md)). Um servidor que o altere vira fronteira de confiança: exige autenticação, integridade, hospedagem, custo, privacidade e um modelo de ameaça próprio. Sincronizar entre PCs também quebra caminhos absolutos de jogos `custom`.
- **Recomendação:** (a) na fase 1, e (b) como primeiro passo quando houver demanda. (c) só com um ADR e um documento de ameaça dedicados.

## Q6. Detectar o fim do jogo

- **Opções:** (a) não detectar (fase 1); (b) só para `custom` (PID do filho, falha com launchers); (c) nome do processo configurado no catálogo (funciona com Steam; manual).
- **Recomendação:** (a) na fase 1; decidir depois do [W8](RISKS-AND-SPIKES.md#w8-tela-cheia-e-foco).

## Q7. Drive falso em builds de release

- **Opções:** (a) só em builds de dev; (b) em release, atrás de uma flag de linha de comando; (c) sempre disponível nas configurações.
- **Recomendação:** (b). Permite que quem não tem drive experimente o app, sem expor o recurso a usuários comuns ([SECURITY R8](SECURITY.md#r8-drive-falso)).

## Q8. Onde guardar a chave de API

- **Opções:** (a) no catálogo (texto); (b) arquivo separado não exportado; (c) armazenamento de credenciais do SO.
- **Recomendação:** (c), com fallback para (b). Mecanismo Windows a verificar na implementação.

## Q9. Campo de versão no `GAME.INI`

O formato decidido tem apenas `id` e `name` ([ADR-0012](adr/0012-game-ini-minimo-e-label-informativo.md)).

- **Opções:** (a) sem versão; a seção `[disc]` e as regras de "ignorar desconhecido" bastam; (b) `version = 1`.
- **Recomendação:** (a). Como chaves desconhecidas são ignoradas, qualquer evolução compatível dispensa versão. Adicionar só se surgir mudança incompatível.

## Q10. Disco inválido sem seleção

Inserção armada na biblioteca de um disco `UNKNOWN`, `AUDIO`, `BLANK` etc.

- **Opções:** (a) ignorar; (b) aviso discreto sem ejetar; (c) aviso e ejetar.
- **Recomendação:** (b). O usuário não pediu nada, então ejetar seria intrusivo. Para `UNKNOWN`, o aviso oferece "adotar".

## Q11. Timeouts

Timeout de `read_media`, intervalo de polling (se W2 levar a polling) e padrão de `loading_min_ms`.

- **Recomendação:** medir no drive real (W2 e W3); `loading_min_ms` padrão de "alguns segundos" (ex.: 3 s) até ajuste pelo uso.

## Q12. Rascunho após falha

Um jogo **novo**, criado durante um cadastro cuja gravação falhou, deve ser mantido?

- **Recomendação:** manter o jogo (sem discos), porque o usuário já preencheu os dados, e descartar apenas o disco.

## Q13. `.bat` e `.cmd` como executável

- **Opções:** (a) recusar; (b) permitir com aviso e sem argumentos; (c) permitir.
- **Recomendação:** (b). Alguns jogos antigos só abrem por `.bat`, mas argumentos para `cmd.exe` são o vetor de injeção documentado ([docs Rust](https://doc.rust-lang.org/std/process/struct.Command.html)).

## Q14. Serviço de capas online

- **Opções:** SteamGridDB (chave do usuário, [W10](RISKS-AND-SPIKES.md#w10-capas-online)) ou outro.
- **Recomendação:** SteamGridDB, se os termos permitirem ([W10](RISKS-AND-SPIKES.md#w10-capas-online)).

## Q15. Regra de abreviação do rótulo

O limite do IMAPI2 é de 15 caracteres com charset restrito ([DISC-FORMAT](DISC-FORMAT.md#rótulo-do-volume)).

- **Recomendação:** remover acentos, trocar espaços por `_`, truncar. A prévia editável já existe no fluxo (`REG_LABEL_PREVIEW` em [UX-STATES](UX-STATES.md#cadastro-e-gravação)); falta fixar a regra exata no W5.

## Q16. Várias unidades ao mesmo tempo

A fase 1 observa uma única unidade (escolhida nas configurações).

- **Recomendação:** manter assim. Revisitar se aparecer caso de uso real.

## Q17. Framework do frontend

**Resolvida:** JavaScript e CSS puros, sem framework nem build ([ADR-0021](adr/0021-frontend-js-css-sem-framework.md)). Análise original mantida como histórico.

O design ([FRONTEND-DESIGN](FRONTEND-DESIGN.md)) exige componentes próprios, sem kit pronto.

- **Opções:** TypeScript puro; Svelte; Solid; React.
- **Critérios:** tamanho do bundle, facilidade de gerenciar foco por controle, estado derivado da máquina de estados do backend (a UI só renderiza).
- **Recomendação da época:** Svelte ou TypeScript puro, por serem leves e porque a UI é uma projeção do estado vindo do Rust.

## Q18. Entrada de texto com controle

Cadastro e configurações pedem texto (nome, caminho, argumentos, chave de API).

- **Opções:** (a) exigir teclado e mouse nesses formulários na fase 1; (b) teclado virtual próprio; (c) teclado de toque do Windows (comportamento com controle não verificado).
- **Recomendação:** (a) na fase 1, com o fluxo "jogar" 100% por controle. Seletor de arquivo e importação da Steam reduzem a digitação.

## Q19. Importar estante

- **Opções:** (a) substituir a estante atual (com backup automático antes); (b) mesclar, com regra para conflito de `disc_id`.
- **Recomendação:** (a) na fase 1. Mesclar exige resolver conflitos (mesmo `disc_id` em jogos diferentes) e caminhos de jogos `custom` de outra máquina.

## Q20. Modo de janela padrão

- **Opções:** janela ou tela cheia na primeira execução.
- **Recomendação:** janela na primeira execução (menos intrusivo enquanto o usuário configura); o usuário liga tela cheia nas configurações e a escolha persiste.

## Q21. Ordenação da biblioteca

- **Pontos:** ordem alfabética com colação do idioma (acentos), artigos iniciais ("The", "O"), números; como LB/RB agrupam por letra.
- **Recomendação:** ordem alfabética com colação do idioma da UI, sem tratamento de artigos na fase 1; números antes das letras; LB/RB saltam para a próxima inicial existente.

## Q22. Instância única

- **Recomendação:** permitir só uma instância; abrir o app de novo traz a janela existente para a frente. Mecanismo no Tauri a verificar na implementação.

## Q23. Logs

- **Pontos:** local, rotação, conteúdo.
- **Recomendação:** arquivo rotativo no diretório de dados do app; registrar estados, classes e erros; **nunca** a chave de API; caminhos de executável só em nível de depuração.

## Q24. Idioma e plural

- **Recomendação:** idioma do sistema se for pt-BR ou en; qualquer outro cai em en. Plurais pelo mecanismo da biblioteca de i18n escolhida (ex.: regras de plural do padrão ICU/CLDR), nunca por concatenação.
