# Riscos e spikes

Cada spike é um experimento descartável, com critério de sucesso ou falha definido **antes** de começar. O resultado vira ADR ou atualiza [OPEN-QUESTIONS](OPEN-QUESTIONS.md).

Legenda de hardware: **—** nenhum · **drive** drive USB · **drive+mídia** drive + CD-R/CD-RW · **controle** gamepad.

## Resumo

| ID | Tema | Risco | Hardware | Dá para fazer já? |
|---|---|---|---|---|
| W1 | Gamepad no WebView2 | **Alto** (bug conhecido) | controle | **sim** |
| W2 | Detecção de inserção e remoção | **Alto** | drive (ISO para triagem) | parcial |
| W3 | Classificação da mídia | Alto | drive+mídia | não |
| W4 | Controle da gaveta | Médio (há degradação) | drive | não |
| W5 | Gravação com IMAPI2 | **Alto** | drive+mídia | não |
| W6 | Integração com a Steam | Médio | — | sim |
| W7 | Lançamento de processos | Médio | — | sim |
| W8 | Tela cheia e foco | Médio | controle | sim |
| W9 | Empacotamento, assinatura e SmartScreen | Médio | — | sim |
| W10 | Capas online | Baixo | — | sim |
| W11 | Perda do catálogo | Médio (impacto alto, mitigado) | — | sim |
| W12 | Leitura de ISO no drive falso | Baixo | — | sim |
| W13 | Desempenho de CSS 3D no WebView2 | Médio | controle | sim |

## Windows (fase 1)

### W1. Gamepad no WebView2

- **Contexto:** o Tauri usa WebView2 no Windows ([Tauri](https://v2.tauri.app/reference/webview-versions/)). Há uma issue reportando que a Gamepad API no WebView2 só funcionava com o DevTools aberto e focado. Foi fechada como "tracked internally", sem correção documentada ([WebView2Feedback #3025](https://github.com/MicrosoftEdge/WebView2Feedback/issues/3025)). Navegadores também só expõem controles depois de uma interação com a página focada ([MDN](https://developer.mozilla.org/en-US/docs/Web/API/Gamepad_API/Using_the_Gamepad_API)).
- **Alternativa:** `gilrs` no Rust (MIT/Apache-2.0), com backend Windows Gaming Input por padrão ou XInput opcional. Limitação documentada: WGI "requires an in focus window to be associated with the process to receive events" ([docs.rs/gilrs](https://docs.rs/gilrs/latest/gilrs/)).
- **Como validar:** app Tauri mínimo em janela e em tela cheia, com um controle XInput (ex.: Xbox) e, se houver, um DirectInput genérico. Testar a Gamepad API e o `gilrs` lado a lado: botões, analógicos, conectar e desconectar, foco perdido e recuperado (Alt+Tab, jogo aberto por cima).
- **Sucesso:** um dos dois caminhos entrega todos os eventos de forma confiável, sem DevTools, nas condições acima. Ele vira ADR.
- **Falha:** nenhum dos dois funciona com o app em primeiro plano. Nesse caso, reavaliar a stack de UI (nova ADR sobre a [ADR-0009](adr/0009-interface-tauri-console-gamepad.md)).

### W2. Detecção de inserção e remoção

- **Contexto:** candidatos em [DRIVE-LAYER](DRIVE-LAYER.md#windowsdrive). A Microsoft diz que mensagens de mídia só são enviadas "for media in devices that support a soft-eject mechanism", e que um evento pode gerar mais de uma mensagem ([DEV_BROADCAST_VOLUME](https://learn.microsoft.com/en-us/windows/win32/api/dbt/ns-dbt-dev_broadcast_volume)).
- **Como validar:** registrar e comparar, por latência e confiabilidade, (a) `WM_DEVICECHANGE`, (b) WMI `Win32_CDROMDrive.MediaLoaded` por polling e (c) consulta direta de volume por polling. Fazer isso com (1) ISO montada e desmontada e (2) drive físico com troca de disco.
- **Sucesso:** um mecanismo detecta 20 de 20 inserções e remoções no drive físico, com latência aceitável (limite a definir) e identificação estável da unidade escolhida pelo usuário. Também fica documentado como a ISO montada se comporta (mesma unidade ou unidade nova).
- **Falha:** nenhum evento confiável. Nesse caso, polling com o intervalo medido vira o padrão, e `media_events = no`.
- **Resultado parcial (2026-09-28):** triagem por ISO feita ([spikes/w2-devicechange](../spikes/w2-devicechange/README.md)). Montar uma ISO cria uma unidade virtual nova (`Microsoft Virtual DVD-ROM`). O `WM_DEVICECHANGE` traz `DEVICEARRIVAL`/`REMOVECOMPLETE` de volume com **`flags = 0` (sem `DBTF_MEDIA`)**, precedidos de `DEVNODES_CHANGED`, com ~1,2 s de latência; `Win32_CDROMDrive.MediaLoaded`/`VolumeName` refletem a mídia e o polling funciona. **Decisão de desenho:** tratar os eventos como gatilho para reler o estado, sem depender da flag. **Falta (exige hardware):** eventos e latência do drive USB físico.

### W3. Classificação da mídia

- **Contexto:** distinguir virgem, dados, áudio, misto e ilegível, além do tipo físico (CD-R ou CD-RW). O IMAPI2 informa o estado para gravação (`BLANK`, `APPENDABLE`, `FINALIZED`…) ([enum](https://learn.microsoft.com/en-us/windows/win32/api/imapi2/ne-imapi2-imapi_format2_data_media_state)). Detectar áudio via TOC **não foi verificado**.
- **Como validar:** CD-R virgem, CD-RW virgem, CD-RW gravado, CD-R gravado, CD de áudio, CD misto (se houver) e CD riscado.
- **Sucesso:** classificação correta de todos os casos disponíveis, com uma única combinação de APIs.
- **Falha:** algum caso indistinguível. Ele passa a ser tratado como a classe mais segura (`READ_ERROR` ou "não gravável"), com mensagem genérica.

### W4. Controle da gaveta

- **Contexto:** `IOCTL_STORAGE_EJECT_MEDIA` "may or may not be supported"; `IOCTL_STORAGE_LOAD_MEDIA` só vale para "devices that support loadable media" ([eject](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ni-winioctl-ioctl_storage_eject_media), [load](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ni-winioctl-ioctl_storage_load_media)). MCI é legado. É hipótese que slim USB só fecham à mão.
- **Como validar:** abrir, fechar e ejetar por cada método; verificar se dá para **detectar** o suporte antes de tentar.
- **Sucesso:** abrir e ejetar funcionam, e a capacidade é detectável ou a falha é identificável. Fechar é bônus.
- **Falha:** a UI sempre mostra instrução manual (degradação prevista).

### W5. Gravação com IMAPI2

- **Contexto:** IMAPI2 grava CD-R/CD-RW, ISO 9660/Joliet/UDF, apaga e verifica ([About IMAPI](https://learn.microsoft.com/en-us/windows/win32/imapi/about-imapi)). O rótulo tem limite de 15 caracteres ([put_VolumeName](https://learn.microsoft.com/en-us/windows/win32/api/imapi2fs/nf-imapi2fs-ifilesystemimage-put_volumename)). A interop COM a partir do Rust não foi verificada.
- **Como validar:** a partir do Rust, gerar a imagem (`GAME.INI` + rótulo), gravar num CD-RW, reler, apagar (rápido e completo), repetir. Depois, **um** CD-R. Medir o tempo. Definir a regra de sanitização do rótulo. Verificar se o disco é lido no Windows (e, mais tarde, no Linux).
- **Sucesso:** 5 ciclos de gravar, verificar e apagar sem erro no CD-RW, 1 CD-R gravado e lido, e progresso reportável na UI.
- **Falha:** interop inviável. Nesse caso, avaliar um processo auxiliar e as implicações de licença ([Q2](OPEN-QUESTIONS.md#q2-licença)).

### W6. Integração com a Steam

- **Contexto:** `steamapps/libraryfolders.vdf` lista as bibliotecas, e cada uma tem `appmanifest_*.acf` ([discussão Steam](https://steamcommunity.com/discussions/forum/1/1754653602249814307/)). Essas são fontes da comunidade: o formato não tem documentação oficial. O cache de capas mudou para `appcache/librarycache/<appid>/…` com nomes de arquivo com hash ([issue](https://github.com/4o66/sunshine-apps-ui/issues/21)), ou seja, **é instável**. `run`, `rungameid` e `launch` diferem em opções de lançamento ([Valve Developer Community](https://developer.valvesoftware.com/wiki/Steam_browser_protocol)).
- **Como validar:** com o PC do autor (múltiplas bibliotecas em discos diferentes, se houver): localizar a Steam, listar jogos instalados, achar capas e lançar pelos três esquemas de URI.
- **Sucesso:** lista completa e correta de instalados; capa encontrada para a maioria; esquema de URI escolhido com justificativa.
- **Falha:** parsing frágil. Nesse caso, a importação vira "melhor esforço" com edição manual; capas faltantes caem em placeholder ou no serviço online.
- **Resultado (2026-09-28): sucesso** ([spikes/w6-steam](../spikes/w6-steam/README.md)). Steam localizada pelo registro, 2 bibliotecas em discos diferentes, 26 jogos listados e o handler `steam://` confirmado. **Só 14 de 26 (54%) têm capa retrato no cache local**; os outros 12 só têm ícone e logo. As capas online opcionais passam a ser necessárias. **Não verificado:** qual esquema de URI usar (`run`, `rungameid`, `launch`).

### W7. Lançamento de processos

- **Contexto:** `Command` não usa shell; `.bat`/`.cmd` são perigosos ([docs Rust](https://doc.rust-lang.org/std/process/struct.Command.html)). Elevação (UAC) para jogos que exigem administrador: mecanismo **não verificado**.
- **Como validar:** executável com espaços no caminho e nos args; diretório de trabalho; emulador com ROM; launcher que abre outro processo e sai; executável que exige elevação (sem e com `requires_elevation`).
- **Sucesso:** todos lançam corretamente, sem shell; um executável que exige elevação produz o prompt do UAC (quando configurado) ou um erro claro (quando não).
- **Falha:** algum caso exige shell. Nesse caso, documentar e restringir, nunca abrir exceção silenciosa.
- **Resultado (2026-09-28): sucesso parcial** ([spikes/w7-launch](../spikes/w7-launch/README.md)). Sem shell, argumentos hostis chegam intactos, com espaços no caminho; `NotFound` é distinguível; o `spawn` não bloqueia. `.bat` com argumentos foi seguro no Rust 1.93 nos casos testados, mas a política continua conservadora ([Q13](OPEN-QUESTIONS.md#q13-bat-e-cmd-como-executável)). **Não verificado:** abrir `steam://` por `explorer.exe` e elevação por UAC.

### W8. Tela cheia e foco

- **Contexto:** o Tauri expõe `setFullscreen`, `setFocus`, `minimize`, `setAlwaysOnTop` ([Tauri](https://v2.tauri.app/reference/javascript/api/namespacewindow/)). Restrições do Windows para roubar foco: não verificadas. Detectar o fim do jogo está em aberto ([Q6](OPEN-QUESTIONS.md#q6-detectar-o-fim-do-jogo)).
- **Como validar:** app em tela cheia lança um jogo Steam e um `custom`. Observar se o jogo aparece por cima, se o app deve minimizar, e como o usuário volta (Alt+Tab, botão Guide, fim do jogo). Testar também a opção de detectar o fim por PID (`custom`) e por nome de processo.
- **Sucesso:** o jogo sempre fica na frente e o retorno ao app é previsível, com um procedimento documentado.
- **Falha:** o app briga pelo foco. Nesse caso, a regra passa a ser minimizar sempre e nunca pedir foco sozinho.
- **Resultado parcial (2026-09-29):** com um "jogo" fictício de janela própria ([spikes/w8-focus](../spikes/w8-focus/README.md)), o app lança, termina o loading e **minimiza**, sem pedir foco; ao voltar, retoma a tela cheia. A medição de quem ganha o primeiro plano não é limpa (o PC de teste tem outros programas em uso), então a regra da falha vale de saída. **Falta:** Steam, fim do jogo (Q6) e botão Guide.

### W9. Empacotamento, assinatura e SmartScreen

- **Contexto:** sem assinatura, o SmartScreen avisa. Desde 2024, certificado EV não dá reputação imediata; OV e EV constroem reputação do mesmo jeito ([Tauri: Windows Code Signing](https://v2.tauri.app/distribute/sign/windows/)).
- **Como validar:** gerar o instalador pelo Tauri, instalar numa máquina limpa (ou VM) e registrar os avisos com e sem assinatura. Levantar opções de assinatura acessíveis a um projeto open source individual.
- **Sucesso:** processo de release documentado, com os avisos esperados e instruções ao usuário.
- **Falha:** não há como assinar. Nesse caso, publicar checksums e instruções de verificação ([SECURITY R9](SECURITY.md#r9-distribuição)).

### W10. Capas online

- **Contexto:** a API v2 do SteamGridDB usa uma chave gerada pelo usuário na conta dele, via Bearer ([node-steamgriddb](https://github.com/SteamGridDB/node-steamgriddb)). Termos de uso e limites: **não verificados**.
- **Como validar:** ler os termos; buscar capa por AppID Steam e por nome; medir o tamanho das imagens; testar uma chave inválida.
- **Sucesso:** termos compatíveis com um app open source em que cada usuário usa a própria chave; fluxo "baixa uma vez, guarda local".
- **Falha:** termos incompatíveis. Nesse caso, apenas capas locais.

### W11. Perda do catálogo

- **Contexto:** sem catálogo, discos anônimos ([CATALOG-SPEC](CATALOG-SPEC.md#perda-e-recuperação)).
- **Como validar:** apagar o catálogo e recuperar por (a) importação de backup e (b) adoção de discos. Corromper o arquivo e verificar que o app não o sobrescreve.
- **Sucesso:** recuperação total por (a) e parcial, mas guiada, por (b), sem perda silenciosa.
- **Falha:** revisar as mitigações.

### W12. Leitura de ISO no drive falso

- **Contexto:** o `FakeIsoDrive` lê `.iso` diretamente; a biblioteca Rust de ISO 9660/Joliet não foi escolhida.
- **Como validar:** ler `GAME.INI` e rótulo de ISOs geradas pelo IMAPI2FS e pelo próprio drive falso; ISO truncada.
- **Sucesso:** leitura correta, e ISO corrompida vira erro sem pânico.
- **Falha:** implementar a leitura mínima (PVD + diretório raiz) ou montar via Windows.
- **Resultado (2026-09-28): sucesso.** O leitor e o gerador de ISO 9660 estão em `insert-disc-core` (`iso.rs`), com testes de imagem truncada e corrompida (sem pânico). O **Windows montou a ISO gerada** como unidade `CDFS` com rótulo `HOLLOW_KNIGHT` e leu o `GAME.INI` (ver [TESTING-WITH-ISO](TESTING-WITH-ISO.md#montagem-nativa-no-windows)). Sem dependência externa; a "leitura mínima" foi a escolhida.

### W13. Desempenho de CSS 3D no WebView2

- **Contexto:** a estante usa CSS 3D (perspectiva, rotação, `translateZ`) e o disco usa gradiente cônico animado ([FRONTEND-DESIGN §5.2.1](FRONTEND-DESIGN.md#521-coreografia-3d-da-caixa)). Não foi medido no WebView2 com centenas de lombadas.
- **Como validar:** protótipo da estante com 300 jogos e capas reais; navegar com direcional segurado; puxar, abrir e lançar; medir quadros por segundo em 1080p e 4K, na GPU do autor e numa GPU integrada, se houver.
- **Sucesso:** 60 fps estáveis em 1080p durante navegação e coreografias; sem atraso de entrada perceptível com repetição a 80 ms.
- **Falha:** reduzir 3D às caixas vizinhas, simplificar a iridescência (imagem estática rotacionada) ou virtualizar a estante (só renderizar as lombadas visíveis).
- **Resultado parcial (2026-09-28):** medido no Chromium do painel do app do Claude (**não** no WebView2), com 300 lombadas em 1080p. Navegação segurada: ~119 fps, sem quadros lentos. Giro 3D da caixa: engasgos (11 quadros acima de 33 ms em 8 toques, pior de 67 ms). A causa foi achada: o custo de `setFocus` crescia com o número de lombadas (~17 ms e tarefas longas de 65–90 ms com 300; ~2 ms com 16), porque mudar uma variável CSS herdada recalculava o estilo de todas. **Correção aplicada:** só as lombadas que trocam de lado do foco mudam de classe e a trilha usa um `transform` inline; `setFocus` caiu para ~2,8 ms com 300, e a geometria foi conferida (posição do foco, afastamento de 540·s, espaçamento de 40·s). **Falta:** re-medir os quadros depois da correção (o painel estava oculto no momento), medir no WebView2 real e numa GPU integrada.

## Linux (fase posterior)

| ID | Tema | Como validar | Sucesso | Falha | Hardware |
|---|---|---|---|---|---|
| L1 | Gamepad no WebKitGTK | Gamepad API na build da distro-alvo vs `gilrs` (evdev, udev) | Um caminho funciona sem configuração manual, ou com uma regra udev documentada | Usar `gilrs` como padrão | controle |
| L2 | Renderização WebKitGTK | Animações da UI em NVIDIA + Wayland, NVIDIA + X11, AMD/Intel | Animações fluidas nos três | Reduzir efeitos ou documentar variáveis de ambiente de contorno (a pesquisar) | GPU real |
| L3 | Detecção de mídia | udisks2 (D-Bus) vs ioctl | Mesmos critérios do W2 | ioctl ou polling | drive |
| L4 | Gaveta | `eject`, `eject -t` ("Not all devices support this command", [eject(1)](https://man7.org/linux/man-pages/man1/eject.1.html)) | Igual ao W4 | Instrução manual | drive |
| L5 | Gravação | `xorriso` vs `wodim`; licença se empacotado | Igual ao W5 | — | drive+mídia |
| L6 | gamescope | App e jogos sob gamescope ([README](https://github.com/ValveSoftware/gamescope/blob/master/README.md)): foco, controle, retorno | Fluxo completo dentro de uma sessão gamescope | Documentar as limitações | GPU real |
| L7 | Empacotamento | AppImage ou pacote primeiro; depois Flatpak com udisks2, lançamento no host e Steam | AppImage funcional | Flatpak fica fora do escopo | — |
| L8 | Steam nativa vs Flatpak | Caminhos e forma de abrir `steam://` nos dois | Detecção de ambos | Suportar só a nativa | — |
