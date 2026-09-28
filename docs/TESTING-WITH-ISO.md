# Testes com ISO

O autor ainda não tem o drive. A fase 1 começa com dois caminhos complementares:

| Caminho | O que exercita | Depende de |
|---|---|---|
| **A. ISO montada no Windows** | O `WindowsDrive` real lendo uma unidade de CD virtual | Montagem nativa do Windows |
| **B. `FakeIsoDrive`** | A máquina de estados inteira, inclusive cenários que ISO não representa | Nada do SO ([DRIVE-LAYER](DRIVE-LAYER.md#fakeisodrive)) |

## Montagem nativa no Windows

- O cmdlet `Mount-DiskImage` "mounts a previously created disk image (virtual hard disk or ISO), making it appear as a normal disk". ISOs são montadas **somente leitura**. Desde o Windows 8, montar ISO não exige administrador. O parâmetro `-NoDriveLetter` existe, então por padrão uma letra é atribuída ([Microsoft Learn: Mount-DiskImage](https://learn.microsoft.com/en-us/powershell/module/storage/mount-diskimage)).
- `Dismount-DiskImage` desmonta a imagem; no teste, faz o papel de "ejetar" (mesma página, links relacionados).
- Pelo Explorador de Arquivos: clique direito, **Montar**. O Windows cria uma unidade óptica virtual com letra própria. O duplo clique só monta se a associação `.iso` ainda for do Explorador ([IONOS](https://www.ionos.com/digitalguide/server/configuration/iso-mount/), [ElevenForum](https://www.elevenforum.com/t/mount-or-unmount-iso-and-img-file-in-windows-11.1201/)). Fontes secundárias; o comportamento é consistente com a documentação do cmdlet.
- **Verificado em 2026-09-28 (WMI, Windows 11):** montar uma ISO **cria uma unidade nova**, `Microsoft Virtual DVD-ROM` (`H:` no teste), que antes não existia; `Win32_CDROMDrive` mostra `MediaLoaded=True` e o rótulo em `VolumeName`. Ao desmontar, a unidade continua por instantes com `MediaLoaded=False` e desaparece em até 2 s; montar de novo reutilizou a mesma letra. Consequências: (1) montar equivale a `DriveAdded` + mídia, não só "mídia inserida"; (2) o identificador da unidade escolhida nas configurações não é estável para ISOs montadas, então o caminho A serve para exercitar `WindowsDrive`, e o drive falso continua sendo o caminho confiável para testes; (3) a detecção por polling de `MediaLoaded`/`VolumeName` funciona. **Ainda não verificado:** os eventos `WM_DEVICECHANGE` (`DBT_DEVICEARRIVAL` com `DBTF_MEDIA`) para ISO e para drive físico. É o spike [W2](RISKS-AND-SPIKES.md#w2-detecção-de-inserção-e-remoção).

## Como criar as ISOs de teste

Cada ISO é uma pasta com (ou sem) um `GAME.INI` na raiz, convertida em imagem ISO 9660/Joliet.

| Ferramenta | Status |
|---|---|
| IMAPI2FS (`IMAPI2FS.MsftFileSystemImage`, COM nativo do Windows): define `VolumeName`, adiciona a pasta e gera a imagem com `CreateResultImage` | Documentado ([CreateResultImage](https://learn.microsoft.com/en-us/windows/win32/api/imapi2fs/nf-imapi2fs-ifilesystemimage-createresultimage)); o uso via PowerShell aparece em exemplos da comunidade ([IDERA](https://www.idera.com/blogs/creating-iso-files/)). Mesmo limite de 15 caracteres no rótulo ([DISC-FORMAT](DISC-FORMAT.md#rótulo-do-volume)). |
| `oscdimg` (Windows ADK) | Não verificado nesta documentação |
| O próprio app, pelo `FakeIsoDrive` ("gravar" gera `.iso`) | Depois que o drive falso existir; é a forma preferida, pois gera exatamente o formato de produção |

Recomendação: manter um conjunto fixo de ISOs de teste versionado junto do projeto (são pequenas) e regenerá-las pelo drive falso quando o formato mudar.

## Cenários

| # | Cenário | Como produzir | Caminho A (montada) | Caminho B (falso) | Classe esperada |
|---|---|---|---|---|---|
| C1 | Disco válido do jogo selecionado | ISO com `GAME.INI` cujo `id` está no catálogo para X | sim | sim | `MATCH` |
| C2 | Disco de outro jogo | ISO com `id` associado a Y | sim | sim | `OTHER_GAME` |
| C3 | Disco desconhecido | ISO com UUID fora do catálogo | sim | sim | `UNKNOWN` (testar adoção) |
| C4 | CD de dados comum | ISO sem `GAME.INI` | sim | sim | `NO_GAME_INI` |
| C5 | INI malformado | `id` inválido, arquivo acima do limite, UTF-8 inválido, sem seção | sim | sim | `INVALID_GAME_INI` |
| C6 | INI com `[disc]`/`id` válidos e chaves extras (ex.: `STEAMID`, `PROCESS`) | ISO com essas chaves | sim | sim | Ignoradas; classe pelo `id` ([SECURITY R1](SECURITY.md#r1-o-disco-só-fornece-um-identificador)) |
| C7 | Rótulo longo, com acentos ou espaços | ISO com rótulo no limite | sim | sim | Exibição correta; sem efeito na decisão |
| C8 | Mídia virgem (CD-R / CD-RW) | **Não representável** por ISO | não | simulado | `BLANK` / cadastro |
| C9 | CD-RW com conteúdo | ISO + marcação "regravável" | não | simulado | Fluxo de apagamento |
| C10 | CD de áudio | **Não representável** por ISO | não | simulado | `AUDIO` |
| C11 | Falha de leitura | ISO truncada ou corrompida; ou simulação | parcial (o Windows pode recusar montar) | simulado | `READ_ERROR` |
| C12 | Leitura lenta / timeout | Simulação | não | simulado | `READ_ERROR` por timeout |
| C13 | Disco presente no boot | Montar antes de abrir o app | sim | sim | Desarmado ([UX-STATES](UX-STATES.md#inserção-sem-seleção-na-biblioteca)) |
| C14 | Remover e recolocar o mesmo disco | Desmontar e montar | sim (sujeito a W2) | sim | Rearma |
| C15 | Gaveta sem suporte a software | Capacidade `tray_open = no` | não | simulado | Instrução manual na tela |
| C16 | Falha no meio da gravação | Simulação | não | simulado | `BURN_FAILED`; CD-R "perdido" |
| C17 | Eventos duplicados ou fora de ordem | Simulação | não | simulado | Máquina de estados estável |
| C18 | `MediaArrived` duplicado logo após um lançamento por seleção, modo `launch` | Simulação | não | simulado | Não relança (deduplicação + desarme) |
| C19 | Disco do Reset Floppy original (`GAME.INI` com NAME/STEAMID, sem `[disc]`) | ISO | sim | sim | `LEGACY_GAME_INI` |
| C20 | Disco de Y inserido sem seleção | ISO, modos `focus` e `launch` | sim | sim | `KNOWN`: foca Y / lança Y |
| C21 | `OTHER_GAME` → "Jogar Y" | ISO de Y com X selecionado | sim | sim | `LAUNCHING` de Y |
| C22 | Rejeição mantém o disco; adoção a partir de `REJECTED` | ISO com UUID desconhecido | sim | sim | Disco não ejetado até `try_other`; adoção lança X |
| C23 | Mídia removida durante `READING` | Simulação | não | simulado | Volta a `WAITING_DISC` |
| C24 | `DriveRemoved` em `WAITING_DISC`, em `BURNING` e na biblioteca | Simulação | não | simulado | `DRIVE_PROBLEM` / `BURN_FAILED` / aviso ([UX-STATES](UX-STATES.md#drive-removido-em-qualquer-estado)) |
| C25 | Drive sem gravador ao adicionar jogo | Capacidade `write_cdr = no` | não | simulado | `DRIVE_PROBLEM(cannot_burn)` |
| C26 | CD-R com conteúdo no cadastro | Simulação | não | simulado | `REG_REJECTED` |
| C27 | Catálogo corrompido no boot | Arquivo de catálogo inválido | sim | sim | `CATALOG_ERROR`; arquivo preservado (W11) |
| C28 | `name` com quebra de linha ou caracteres de controle ao gravar | Nome de jogo malicioso | sim | sim | Gerador remove os caracteres; o INI gerado tem um único `id` ([DISC-FORMAT](DISC-FORMAT.md#regras-de-escrita)) |
| C29 | Apagamento de CD-RW falha | Simulação | não | simulado | `disc_id` antigo **continua** no catálogo |
| C30 | Jogo sem disco selecionado | Catálogo com jogo sem discos | sim | sim | `NO_DISC_YET` |

## O que só dá para validar com hardware real

| Item | Por quê |
|---|---|
| Abrir e fechar a gaveta por software | Depende do modelo; ISO não tem gaveta ([W4](RISKS-AND-SPIKES.md#w4-controle-da-gaveta)) |
| Evento físico de troca de mídia | Montar ISO pode gerar eventos diferentes ([W2](RISKS-AND-SPIKES.md#w2-detecção-de-inserção-e-remoção)) |
| Gravação real, verificação e falha real | IMAPI2 precisa de gravador ([W5](RISKS-AND-SPIKES.md#w5-gravação-com-imapi2)) |
| Detecção de mídia virgem real (CD-R e CD-RW) | Estado de mídia vem do gravador |
| CD de áudio real | Precisa de TOC real ([W3](RISKS-AND-SPIKES.md#w3-classificação-da-mídia)) |
| Tempo de spin-up e leitura do drive USB | Define timeouts reais ([Q11](OPEN-QUESTIONS.md#q11-timeouts)) |
| Desconectar o drive USB com o app aberto | `DriveRemoved` |
| Etiqueta colada: balanceamento e leitura | Físico ([HARDWARE-AND-MATERIALS](HARDWARE-AND-MATERIALS.md)) |

## Linux (fase posterior)

Material para quando a fase Linux for confirmada. O autor não tem Linux instalado hoje.

| Caminho | Valida | Não valida |
|---|---|---|
| Pendrive live (sessão temporária) | Hardware real: drive USB, udisks2, `eject`, gravação, gamepad, GPU real (NVIDIA/Wayland), gamescope | Instalação persistente, empacotamento instalado, Steam instalada com biblioteca real (salvo com persistência) |
| Máquina virtual | Build, UI, catálogo, drive falso, udisks2 com ISO anexada como CD virtual (hipótese) | GPU real, gamescope com aceleração real, gravação física, gaveta; o repasse USB do drive é possível, mas não verificado |
| Loop device (`losetup` de uma ISO) | Leitura de ISO pelo caminho de bloco do Linux | Eventos de mídia óptica (um loop device não é uma unidade de CD), gaveta, mídia virgem, áudio |
