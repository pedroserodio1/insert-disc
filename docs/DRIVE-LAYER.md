# Camada de drive

Interface `DriveBackend`: tudo o que o núcleo precisa de uma unidade óptica. O núcleo não sabe se está falando com hardware, com uma ISO ou com Linux ([ADR-0010](adr/0010-windows-primeiro-nucleo-portatil.md)).

A assinatura abaixo é conceitual (operações e dados), não código.

## Operações

| Operação | Entrada | Saída | Observação |
|---|---|---|---|
| `list_drives` | — | lista de `DriveInfo` (id estável, nome, letra/caminho) | O usuário escolhe um nas configurações |
| `capabilities` | drive | `Capabilities` | Consultado no início e usado para degradar a UI |
| `open_tray` | drive | ok \| `Unsupported` \| erro | Pode não ser suportado ([W4](RISKS-AND-SPIKES.md#w4-controle-da-gaveta)) |
| `close_tray` | drive | ok \| `Unsupported` \| erro | Muitos slim só fecham à mão (hipótese, [W4](RISKS-AND-SPIKES.md#w4-controle-da-gaveta)) |
| `eject` | drive | ok \| `Unsupported` \| erro | |
| `read_media` | drive | `MediaInfo` | Com timeout. Inclui a leitura do `GAME.INI` (só na raiz, com limite rígido de bytes); não há outra operação de leitura de arquivo exposta ao núcleo. |
| `burn` | drive, `DiscImageSpec` | stream de `BurnProgress` → ok \| erro | [BURNING](BURNING.md) |
| `erase` | drive, `quick` \| `full` | stream de progresso → ok \| erro | Só para CD-RW |
| `start_op` / `poll_ops` | drive, `DriveOp` (`Read`, `Burn`, `Erase`); `now` | `OpUpdate` (`Progress`, `Done`) | **Como o núcleo chama `read_media`, `burn` e `erase`** (A5): inicia sem bloquear e recebe progresso e resultado por `pump`. Backend real: uma thread por operação (uma por vez) e um canal; o drive falso simula com atraso configurável (`set_op_delay_ms`). Tirar o disco no meio faz a operação falhar; sair de `READING` descarta o resultado. |

### Tipos

| Tipo | Campos |
|---|---|
| `Capabilities` | `tray_open`, `tray_close`, `eject`, `write_cdr`, `write_cdrw`, `erase`, `media_events` (`yes`: emite eventos; `no`/`unknown`: o núcleo faz polling), cada um `yes` \| `no` \| `unknown` |
| `MediaInfo` | `present`, `kind` (`blank` \| `data` \| `audio` \| `mixed` \| `unreadable`), `physical` (`cd-rom` \| `cd-r` \| `cd-rw` \| `unknown`), `appendable/closed` quando souber, `label`, `game_ini` (bytes crus limitados, se houver) |
| `BackendKind` | `real` \| `fake`. O núcleo grava `media = iso` no catálogo ([CATALOG-SPEC](CATALOG-SPEC.md#disc)) quando o disco foi gravado ou adotado com o backend `fake`. |
| `DiscImageSpec` | `label` (já sanitizado), conteúdo do `GAME.INI`. Nada além disso. |

## Eventos

| Evento | Quando |
|---|---|
| `MediaArrived(drive)` | Mídia inserida e pronta para leitura |
| `MediaRemoved(drive)` | Mídia removida ou ejetada |
| `DriveAdded(drive)` / `DriveRemoved(drive)` | Unidade conectada ou desconectada (USB) |
| `TrayOpened` / `TrayClosed` | **Opcional.** Só se o backend souber; não há garantia de que o hardware informe (hipótese, [W4](RISKS-AND-SPIKES.md#w4-controle-da-gaveta)). |

Regras do núcleo sobre eventos:

- O núcleo **deduplica**: `MediaArrived` com mídia já presente é ignorado; só `MediaRemoved` zera a presença. Isso impede que um evento repetido relance um jogo ([UX-STATES](UX-STATES.md#conceitos-transversais)).
- O núcleo tolera eventos duplicados e fora de ordem. A Microsoft avisa que o mesmo evento de volume pode gerar mais de uma mensagem ([DEV_BROADCAST_VOLUME](https://learn.microsoft.com/en-us/windows/win32/api/dbt/ns-dbt-dev_broadcast_volume)).
- Sem eventos (`media_events = no`), o núcleo faz polling de `read_media` num intervalo configurável.
- A lógica de armar e desarmar ([UX-STATES](UX-STATES.md#conceitos-transversais)) vive no núcleo, não no backend.

## Implementações previstas

| Implementação | Fase | Propósito |
|---|---|---|
| `WindowsDrive` | 1 | Hardware real no Windows |
| `FakeIsoDrive` | 1 | Testes e desenvolvimento sem drive |
| `LinuxDrive` | posterior | Hardware real no Linux |

### WindowsDrive

Candidatos por operação. **Nenhum foi validado com o hardware do projeto**; todos dependem de spike.

| Operação | Candidatos | Fonte / status |
|---|---|---|
| Eventos de mídia | `WM_DEVICECHANGE` com `DBT_DEVICEARRIVAL` / `DBT_DEVICEREMOVECOMPLETE` e `DEV_BROADCAST_VOLUME.dbcv_flags & DBTF_MEDIA` | Documentado ([WM_DEVICECHANGE](https://learn.microsoft.com/en-us/windows/win32/devio/wm-devicechange), [DEV_BROADCAST_VOLUME](https://learn.microsoft.com/en-us/windows/win32/api/dbt/ns-dbt-dev_broadcast_volume)). A Microsoft diz que as mensagens de mídia só são enviadas "for media in devices that support a soft-eject mechanism". Medido com ISO montada: chegam `DEVICEARRIVAL`/`REMOVECOMPLETE` de volume com `flags = 0` (sem `DBTF_MEDIA`), então o `WindowsDrive` usa os eventos só como **gatilho para reler o estado** ([W2](RISKS-AND-SPIKES.md#w2-detecção-de-inserção-e-remoção)); drive USB físico: **não verificado**. |
| Presença de mídia (polling) | WMI `Win32_CDROMDrive.MediaLoaded`, `VolumeName`, `Drive` | Documentado ([Win32_CDROMDrive](https://learn.microsoft.com/en-us/windows/win32/cimwin32prov/win32-cdromdrive)). Latência e custo: não verificados. |
| Mídia virgem / tipo físico | IMAPI2 (`IMAPI_FORMAT2_DATA_MEDIA_STATE`: `BLANK`, `APPENDABLE`, `FINALIZED`, `ERASE_REQUIRED`…) | Documentado ([enum](https://learn.microsoft.com/en-us/windows/win32/api/imapi2/ne-imapi2-imapi_format2_data_media_state)). Só vale para gravadores. |
| CD de áudio | Leitura de TOC (ex.: ioctl de TOC do CD-ROM) | **Não verificado** ([W3](RISKS-AND-SPIKES.md#w3-classificação-da-mídia)) |
| Ejetar | `IOCTL_STORAGE_EJECT_MEDIA` | Documentado: "may or may not be supported" ([Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ni-winioctl-ioctl_storage_eject_media)) |
| Fechar gaveta | `IOCTL_STORAGE_LOAD_MEDIA`; MCI `set cdaudio door closed` (legado) | Documentado: "valid only for devices that support loadable media" ([Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/winioctl/ni-winioctl-ioctl_storage_load_media)); MCI ([blog Microsoft](https://learn.microsoft.com/en-us/archive/blogs/larryosterman/playing-audio-cds-part-3-mci)) |
| Gravar / apagar | IMAPI2 (`IDiscFormat2Data`, `IDiscFormat2Erase`, `IFileSystemImage`) | Documentado ([About IMAPI](https://learn.microsoft.com/en-us/windows/win32/imapi/about-imapi), [IDiscFormat2Erase](https://learn.microsoft.com/en-us/windows/win32/api/imapi2/nn-imapi2-idiscformat2erase)). Interop a partir do Rust: spike [W5](RISKS-AND-SPIKES.md#w5-gravação-com-imapi2). |

### FakeIsoDrive

Objetivo: exercitar **toda** a máquina de estados sem hardware e sem depender da montagem de ISO do Windows.

| Aspecto | Comportamento |
|---|---|
| "Drive" | Uma pasta de trabalho configurada (só em modo dev; ver [Q7](OPEN-QUESTIONS.md#q7-drive-falso-em-builds-de-release)) |
| Inserir | O desenvolvedor escolhe um cenário num painel de dev ou equivalente; o backend emite `MediaArrived` |
| Remover / ejetar | Emite `MediaRemoved`; `eject()` também remove |
| Leitura de ISO | Lê o arquivo `.iso` **diretamente** (parser ISO 9660/Joliet no processo), sem montar no Windows. A escolha da biblioteca é spike [W12](RISKS-AND-SPIKES.md#w12-leitura-de-iso-no-drive-falso). |
| Cenários sem ISO real | `BLANK_CDR`, `BLANK_CDRW`, `AUDIO`, `READ_ERROR`, `SLOW_READ` são **simulados** (ISO não representa mídia virgem nem trilhas de áudio) |
| Capacidades | Configuráveis por cenário (ex.: simular "gaveta não fecha por software") |
| Gravar | Num slot "virgem", gera um **novo `.iso`** com o `GAME.INI` e o rótulo, reinserível depois. Pode simular falha no meio (o slot vira "CD-R perdido"). |
| Apagar | Num slot "CD-RW com conteúdo", volta ao estado virgem |

Tabela de cenários: [TESTING-WITH-ISO](TESTING-WITH-ISO.md#cenários).

### LinuxDrive (fase posterior)

| Operação | Candidatos | Fonte / status |
|---|---|---|
| Eventos e propriedades | udisks2 via D-Bus: `org.freedesktop.UDisks2.Drive` com `MediaAvailable`, `OpticalBlank`, `OpticalNumAudioTracks` e o método `Eject` | Documentado ([UDisks2 Drive](https://storaged.org/doc/udisks2-api/latest/gdbus-org.freedesktop.UDisks2.Drive.html)). Não testado. |
| Alternativa | ioctl direto no dispositivo | Não verificado |
| Gaveta | `eject` / `eject -t`. O manual avisa: "Not all devices support this command" ([eject(1)](https://man7.org/linux/man-pages/man1/eject.1.html)) | Documentado |
| Gravar | `xorriso` (ou `wodim`) como processo externo | [PLATFORMS](PLATFORMS.md#linux-fase-posterior) |
