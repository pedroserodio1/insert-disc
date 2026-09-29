# O que falta e dá para fazer sem nada externo

Posição em 2026-09-29. "Sem nada externo" = só código e este PC: **sem drive físico, sem controle, sem certificado de assinatura, sem chave de serviço online e sem Linux**. Serve de fila de trabalho: cada item tem um critério de pronto verificável. O que **não** dá para fazer está na [última seção](#o-que-fica-bloqueado-e-por-quê).

## Onde estamos

| Peça | Estado |
|---|---|
| Núcleo (`insert-disc-core`): `GAME.INI`, catálogo, ISO, drive falso, máquina de estados, snapshot | Feito, 59 testes |
| `insert-disc-host` (API JSON, servidor de desenvolvimento) e UI (`ui/`) | Feito; fluxo de jogar e de cadastro verificados no navegador com o drive falso |
| App desktop (`apps/desktop`) | Abre no WebView2 e o fluxo de jogar funciona pelo IPC (E6 feito). **A1 feito:** catálogo em `%APPDATA%InsertDisc`, `--demo` e `--fake-drive` (Q7). No CI (job `desktop`). Achado: sem drive não há como cadastrar jogo (o cadastro passa pela gravação) e as capas de demonstração só existem no servidor de desenvolvimento |
| Spikes W2 (parcial), W6, W7, W12 | Rodados; W13 parcial |

O app de hoje é uma **demonstração**: estante de exemplo recriada a cada abertura, drive falso e um lançador que só registra o que executaria.

## Fila (na ordem sugerida)

Legenda: **S** pequeno (horas), **M** médio, **G** grande.

### A. Tornar o app real

> **Antes de A1:** fazer **E6** (abrir o app desktop) e decidir [Q7](OPEN-QUESTIONS.md): em release o drive falso só entra por flag explícita; travar o caminho de `Host::demo` (`remove_dir_all`). A2 e A3 compartilham um único módulo de localização da Steam (erro `SteamMissing`); os testes de A3 usam fixtures de VDF/ACF, não a Steam real. A2: trava = variável de ambiente ou flag de linha de comando; padrão só registra. A5 vem antes de C1 (define o modelo de threads).

| # | Item | Por quê | Pronto quando | Tam. |
|---|---|---|---|---|
| A1 ✅ | **Catálogo persistente** no app: pasta de dados do usuário (`%APPDATA%\InsertDisc\`), modo demonstração só por opção explícita | Hoje a estante some ao fechar | Fechar e abrir mantém jogos, discos e configurações; arquivo corrompido leva a `CATALOG_ERROR` sem sobrescrever; teste com pasta temporária | S |
| A2 ✅ | **Lançador real (Windows)**: `Command` sem shell, URI da Steam por `explorer.exe`, erros mapeados (`NotFound`, `SteamMissing`, `ElevationDenied`) | Sem isso o app não abre jogo | Um `.exe` de teste abre com args e pasta de trabalho corretos; o lançador de demonstração continua sendo o padrão em desenvolvimento (trava explícita para não abrir jogo por engano) | M |
| A3 ✅ (capa em B2) | **Importar da Steam** no "Novo jogo da Steam": lista dos instalados (o código do [W6](RISKS-AND-SPIKES.md#w6-integração-com-a-steam) vira módulo, com filtro de ferramentas), escolha por lista em vez de digitar o AppID | O AppID à mão é inviável | Os 26 jogos deste PC aparecem; escolher um cria o jogo com nome e AppID; parser do VDF com testes (formato novo e antigo) | M |
| A4 ✅ | **Edição completa do jogo**: hoje só dá para renomear; faltam executável, argumentos, pasta e elevação dos jogos `custom` | Sem isso um erro de digitação obriga apagar o jogo | Editar cada campo com a mesma validação da criação; `.bat`/`.cmd` com aviso ([Q13](OPEN-QUESTIONS.md#q13-bat-e-cmd-como-executável)) | S |
| A5 ✅ | **Operações do drive assíncronas**: `read_media`, `burn` e `erase` rodam numa thread e devolvem o resultado por `pump`; hoje são síncronas | Os estados `READING`, `BURNING` e `VERIFYING` duram uma chamada e **a UI nunca os mostra**; o progresso da gravação nunca aparece | O drive falso tem atraso configurável; a UI mostra "lendo" e o progresso subindo; a UI continua respondendo durante a gravação | M |
| A6 ✅ | **Instância única** ([Q22](OPEN-QUESTIONS.md#q22-instância-única)) e **log em arquivo** ([Q23](OPEN-QUESTIONS.md#q23-logs)) | Duas instâncias disputariam o mesmo drive; sem log não há como diagnosticar | Abrir de novo traz a janela existente; log rotativo sem chave de API nem caminhos fora do nível de depuração | S |

### B. Capas

| # | Item | Pronto quando | Tam. |
|---|---|---|---|
| B1 ✅ | **Pipeline de capa local** ([SECURITY R6](SECURITY.md#r6-capas)): validar por conteúdo (não por extensão), limitar tamanho e dimensões, recusar SVG, reencodar, salvar nos dados do app | Arquivos falsos, gigantes e SVG são recusados em testes; a capa salva abre na UI pelo protocolo de assets do Tauri | M |
| B2 ✅ | **Capa do cache da Steam** (54% dos jogos deste PC têm; o resto fica com placeholder) | Ao importar da Steam a capa é copiada pelo pipeline B1; sem capa, o placeholder aparece | S |
| B3 ✅ | **Cor da lombada calculada no backend** (cor dominante escurecida até contraste ≥ 4,5:1 com `etiqueta`) | Teste com imagens claras e escuras; o campo `spine_color` é preenchido | S |

### C. Windows sem drive físico

| # | Item | Pronto quando | Tam. |
|---|---|---|---|
| C1 ✅ | **`WindowsDrive`, parte de leitura**: listar unidades ópticas (`Win32_CDROMDrive`), ler `MediaLoaded`, `VolumeName` e o `GAME.INI` do volume, gatilho por `WM_DEVICECHANGE` **mais** polling ([W2](RISKS-AND-SPIKES.md#w2-detecção-de-inserção-e-remoção): a ISO montada chega com `flags=0`) | Com `Mount-DiskImage` de uma ISO gerada pelo projeto, o núcleo recebe `MediaArrived`/`MediaRemoved` e classifica o disco pelo mesmo caminho do drive físico | G |
| C2 ✅ | **Escolha do drive nas configurações** com a regra de que ISOs montadas criam unidades novas ([TESTING-WITH-ISO](TESTING-WITH-ISO.md#montagem-nativa-no-windows)) | A lista mostra as unidades atuais; escolher uma persiste; unidade sumindo leva a `DRIVE_PROBLEM` | S |
| C3 | **Teste de foco com um "jogo" fictício** ([W8](RISKS-AND-SPIKES.md#w8-tela-cheia-e-foco)): um programa de janela própria que o app lança; medir como o app sai da frente e como volta | Procedimento documentado (minimizar, retorno) e o comportamento escolhido implementado | M |

### D. Controle sem controle

| # | Item | Pronto quando | Tam. |
|---|---|---|---|
| D1 ✅ | **`GamepadSource` com `gilrs` no Rust**, emitindo os eventos de navegação do [UI-CONTRACT](UI-CONTRACT.md#entrada-de-controle), atrás de uma opção de compilação | Compila e roda sem controle; eventos sintéticos chegam à UI. Assim o resultado do W1 só escolhe o padrão, sem exigir código novo | M |
| D2 ✅ | **Simulador de controle na UI** (eventos sintéticos da Gamepad API) para testar navegação, repetição do direcional e segurar-para-confirmar | Testes automatizados de navegação por controle sem hardware | S |

### E. Qualidade e entrega

| # | Item | Pronto quando | Tam. |
|---|---|---|---|
| E1 ✅ | **Testes da UI** (`node --test`, sem dependências): `i18n` (chaves presentes nos dois idiomas, plural), montagem de cada tela a partir de instantâneos de exemplo, validação de formulário | Rodam no CI | M |
| E2 ✅ | **Script de contraste** dos tokens ([FRONTEND-DESIGN §3.1](FRONTEND-DESIGN.md#31-cor)) e checagem de que nenhum valor fora de token entrou no CSS | O CI falha se um par de cores ficar abaixo do mínimo | S |
| E3 | **Re-medir o [W13](RISKS-AND-SPIKES.md#w13-desempenho-de-css-3d-no-webview2)** depois da correção da estante, com a janela do app visível | Números de quadros por segundo e tarefas longas registrados; ou o app desktop medido direto | S |
| E4 | **Build local do instalador** (sem assinatura) e um workflow de release no CI que publica o artefato do Windows | O instalador instala e abre numa conta limpa; assinatura fica para o [W9](RISKS-AND-SPIKES.md#w9-empacotamento-assinatura-e-smartscreen) | M |
| E5 ✅ | **Arquivos de projeto**: `CONTRIBUTING`, modelos de issue e de PR, política de segurança do GitHub | Presentes e linkados no README | S |
| E6 ✅ | **Abrir e verificar o app desktop** (`apps/desktop`) no WebView2 | A janela sobe, a UI funciona, `Ctrl+Shift+D` mostra o painel; problemas achados viram itens aqui | S |

### E7–E11. Lacunas descobertas na revisão

| # | Item | Tam. |
|---|---|---|
| E7 | Checklist de conformidade com [FRONTEND-DESIGN](FRONTEND-DESIGN.md): clique direito abre opções (§7), botão voltar do mouse, roda do mouse na estante, estante esmaecida atrás da caixa aberta (§6.3), escolha do drive nas configurações | M |
| E8 | Trait `SystemIntegration` não existe (só `Launcher`): criar ou ajustar ARCHITECTURE, DRIVE-LAYER e ADR-0010 | S |
| E9 (CI feito) | `apps/desktop` no CI; protocolo de assets do Tauri (pré-requisito de B1); `bundle.active=false` só até E4 | S |
| E10 | ARCHITECTURE.md com a estrutura real; cenários C1–C30 de TESTING-WITH-ISO mapeados para `flows.rs`; testes de import/export do Host | S |
| E11 | Q11 (timeout de `read_media`), nota de disco editável, confirmação de import na UI, `focus_hint` com mais de um cliente | S |

Notas de critério: **C1**: ISO montada aparece como Microsoft Virtual DVD-ROM, então `MediaInfo.physical` fica `Unknown` (esperado). **D1**: feature cargo `gilrs`; exige canal de eventos Rust→UI (Tauri events). **E4**: build local sem assinatura; baixar WiX/NSIS depende de rede.

### F. Decisões que já têm padrão recomendado

Perguntas abertas que dá para fechar com a recomendação da própria doc e seguir (registrar como ADR quando decididas): [Q19](OPEN-QUESTIONS.md#q19-importar-estante) (importar substitui), [Q20](OPEN-QUESTIONS.md#q20-modo-de-janela-padrão) (janela na primeira execução), [Q21](OPEN-QUESTIONS.md#q21-ordenação-da-biblioteca), [Q22](OPEN-QUESTIONS.md#q22-instância-única), [Q23](OPEN-QUESTIONS.md#q23-logs), [Q24](OPEN-QUESTIONS.md#q24-idioma-e-plural). Já decididas no código e ainda abertas na doc (registrar em ADR): Q9, Q10, Q12, Q13, Q15, Q18. Nenhuma exige informação externa; falta apenas o seu "sim".

## Ordem sugerida

1. **E6 → A1** (o app abre, guarda dados): sem isso nada do resto é usável.
2. **A2 + A3 + A4** (abrir jogo, importar da Steam, editar): o app já serve para jogar clicando na estante, mesmo sem drive.
3. **A5** (drive assíncrono): destrava o feedback de leitura e gravação.
4. **B1–B3** (capas).
5. **C1 + C2** (`WindowsDrive` de leitura, testável com ISO montada).
6. **D1 + D2**, **E1–E5**, **A6**, **C3** em paralelo, conforme o tempo.

## O que fica bloqueado (e por quê)

| Bloqueio | O que não anda | Item da doc |
|---|---|---|
| **Drive USB físico** | Eventos e latência reais, tipo de mídia (CD-R/CD-RW/áudio), gaveta abrir/fechar por software, **gravação real** e apagamento, IMAPI2 a partir do Rust, o `WindowsDrive` de escrita | W2 (físico), W3, W4, W5; [BURNING](BURNING.md) |
| **Controle** | Confirmar se a Gamepad API entrega os botões no WebView2 (o código dos dois caminhos pode existir antes, ver D1) | [W1](RISKS-AND-SPIKES.md#w1-gamepad-no-webview2) |
| **Certificado de assinatura** | Instalador assinado, reputação no SmartScreen | [W9](RISKS-AND-SPIKES.md#w9-empacotamento-assinatura-e-smartscreen) |
| **Chave e rede** (SteamGridDB) | Capas online opcionais, termos e limites do serviço | [W10](RISKS-AND-SPIKES.md#w10-capas-online) |
| **Linux** | Fase 2 inteira: udisks2, `xorriso`, WebKitGTK, gamescope, AppImage/Flatpak | [PLATFORMS](PLATFORMS.md), L1–L8 |
| **Jogos e Steam reais em execução** | Escolher entre `run`, `rungameid` e `launch`; elevação (UAC) | [W6](RISKS-AND-SPIKES.md#w6-integração-com-a-steam), [W7](RISKS-AND-SPIKES.md#w7-lançamento-de-processos) |
| **Impressora e etiquetas** | Validar o fluxo físico de imprimir e colar a etiqueta | [HARDWARE-AND-MATERIALS](HARDWARE-AND-MATERIALS.md) |

## Limitações conhecidas do que já existe

- Ambiente de verificação: o painel do navegador embutido só desenha quando a janela do Claude está visível. Fora disso as capturas dão erro e o `requestAnimationFrame` é estrangulado, então medir quadros exige a janela na frente.
- Só o **modo inglês** foi visto no navegador durante os testes (a UI segue o idioma do navegador); os textos em pt-BR existem, mas não foram revisados visualmente.
- A UI não foi verificada em telas 4K nem em proporções diferentes de 16:9.
- Ainda não existe teste automatizado da UI (item E1).
