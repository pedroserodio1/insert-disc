# Segurança

Projeto open source: qualquer pessoa conhece o formato do disco. Por isso a segurança não pode depender de segredo no disco. Ela depende de **o disco não ter poder** ([ADR-0006](adr/0006-catalogo-local-define-execucao.md)).

## Modelo de ameaça

| Ativo | Ameaça | Origem |
|---|---|---|
| Execução de programas no PC | Um disco faz o app executar algo que o usuário não configurou | Mídia removível não confiável (disco emprestado, forjado, malicioso) |
| Processo do app | Parser do `GAME.INI` ou do sistema de arquivos explorado por entrada malformada | Mídia removível |
| Catálogo | Adulteração que troca o que um jogo lança | Outro software ou usuário local com acesso ao perfil |
| Chave de API de capas | Vazamento (logs, exportação do catálogo, repositório) | Descuido na implementação |
| Imagens baixadas | Arquivo malicioso ou inesperado exibido pela UI | Serviço online de capas |
| Distribuição | Binário adulterado | Cadeia de build e release |

Fora do modelo: um atacante com controle total da conta do usuário. Nesse caso ele já pode executar qualquer coisa sem precisar do app.

## Regras

### R1. O disco só fornece um identificador

- Do disco, o app usa apenas o `id` do `GAME.INI`, como chave de busca ([DISC-FORMAT](DISC-FORMAT.md)).
- `name` e rótulo são exibidos como texto (escapado na UI) e nunca viram caminho, comando, URI ou chave.
- Chaves desconhecidas do INI são ignoradas. Nenhum outro arquivo do disco é aberto.

### R2. Parsing defensivo

- Leitura do `GAME.INI` com limite rígido de bytes, feita no backend antes de qualquer parsing.
- UTF-8 estrito; UUID validado. Em qualquer violação, classifica como `INVALID_GAME_INI` e não trata como erro fatal.
- Timeout de leitura; o app não fica bloqueado com mídia defeituosa.
- No drive falso, o parser de ISO lê arquivos de teste. Ele não é caminho de produção, mas segue os mesmos limites.

### R3. Execução sem shell

- `custom.args` é uma **lista**; a execução passa a lista diretamente ao processo. Em Rust, `std::process::Command` não usa shell: "the argument is not passed through a shell, but given literally to the program" ([docs Rust](https://doc.rust-lang.org/std/process/struct.Command.html)).
- **Alerta documentado:** `cmd.exe` e arquivos `.bat`/`.cmd` decodificam argumentos de forma não padrão e são "vulnerable to malicious input" ([docs Rust](https://doc.rust-lang.org/std/process/struct.Command.html)). Regra: o catálogo **recusa ou avisa** ao configurar `.bat`/`.cmd` como executável (escolha em [Q13](OPEN-QUESTIONS.md#q13-bat-e-cmd-como-executável)).
- URI da Steam montada a partir de `app_id` **validado como inteiro**, nunca por concatenação de texto livre.
- Elevação (UAC) só com `requires_elevation` definido pelo usuário. O app nunca roda elevado por padrão ([W7](RISKS-AND-SPIKES.md#w7-lançamento-de-processos)).

### R4. O catálogo pertence ao usuário

- Fica no diretório de dados do usuário, com as permissões padrão do perfil.
- Só é alterado por ação explícita na UI (ou edição manual do usuário). Nada vindo do disco nem da rede altera mapeamentos de execução.
- A adoção de disco ([ADR-0014](adr/0014-um-jogo-n-discos-e-adocao.md)) só associa um `id` a um jogo **escolhido pelo usuário**. O `name` do INI é apenas sugestão visual.

### R5. Segredos

- A chave de API de capas não fica no arquivo do catálogo, não aparece em logs e não entra na exportação ([Q8](OPEN-QUESTIONS.md#q8-onde-guardar-a-chave-de-api)).

### R6. Capas

- Capas online desligadas por padrão ([ADR-0015](adr/0015-i18n-e-capas-online-opcionais.md)); o app não envia dados além do necessário para a busca (nome ou AppID).
- **Toda capa**, de qualquer origem (cache da Steam, arquivo do usuário, online), é validada (tipo por conteúdo, não por extensão; formatos raster esperados; tamanho máximo de arquivo e de dimensões) e **reencodada** para um formato único antes de ser salva nos dados do app. SVG não é aceito.

### R7. Superfície do Tauri

- Expor ao frontend apenas os comandos necessários. O Tauri v2 exige permissões por API de janela, por exemplo ([Tauri: Window API](https://v2.tauri.app/reference/javascript/api/namespacewindow/)). O modelo de capabilities deve ser mínimo; revisar no início da implementação.
- O frontend não recebe caminhos arbitrários para executar: envia `game_id`, e o backend resolve pelo catálogo ([UI-CONTRACT](UI-CONTRACT.md#princípios)).
- CSP restritiva na webview (sem scripts remotos, sem `eval`); textos vindos do disco ou do catálogo são inseridos sempre como texto, nunca como HTML.

### R8. Drive falso

- Não deve estar acessível em builds de release, ou deve exigir um opt-in explícito ([Q7](OPEN-QUESTIONS.md#q7-drive-falso-em-builds-de-release)).

### R9. Distribuição

- Builds reproduzíveis a partir de CI pública e checksums publicados (recomendação).
- Assinatura de código: [W9](RISKS-AND-SPIKES.md#w9-empacotamento-assinatura-e-smartscreen).

### R10. Entrada só com foco

- Com a janela do app sem foco (o jogo está por cima), nenhum evento de controle é processado. Isso impede que apertar A dentro do jogo selecione ou lance algo no app ([UI-CONTRACT](UI-CONTRACT.md#entrada-de-controle)).

### R11. Instância única

- Duas instâncias reagiriam ao mesmo drive. Mecanismo: [Q22](OPEN-QUESTIONS.md#q22-instância-única).

## Reportar vulnerabilidades

Não abra issue pública. Use o reporte privado de vulnerabilidades do GitHub no repositório (a ser habilitado na publicação) ou o e-mail do mantenedor informado no perfil do repositório.
