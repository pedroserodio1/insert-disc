# Plataformas

Regra ([ADR-0010](adr/0010-windows-primeiro-nucleo-portatil.md)): o núcleo (catálogo, máquina de estados, lançador-política, formato do disco) não usa API de SO. O que é específico fica em `DriveBackend`, `SystemIntegration` e `GamepadSource` ([ARCHITECTURE](ARCHITECTURE.md)).

## Matriz

| Área | Portátil | Windows (fase 1) | Linux (fase posterior) |
|---|---|---|---|
| Catálogo, máquina de estados, `GAME.INI`, política de lançamento | **sim** | — | — |
| Webview | — | WebView2 (Chromium) ([Tauri](https://v2.tauri.app/reference/webview-versions/)) | WebKitGTK (`webkit2gtk`) ([Tauri](https://v2.tauri.app/reference/webview-versions/)) |
| Gamepad | Interface `GamepadSource` | Gamepad API no WebView2 **ou** `gilrs` (backend Windows Gaming Input por padrão; XInput opcional) ([gilrs](https://docs.rs/gilrs/latest/gilrs/)) | Gamepad API do WebKitGTK depende de build com libmanette ([BLFS](https://www.linuxfromscratch.org/blfs/view/stable-systemd/x/webkitgtk.html)) **ou** `gilrs` (evdev; pode exigir regra udev) ([gilrs](https://docs.rs/gilrs/latest/gilrs/)) |
| Eventos de mídia | Interface `DriveBackend` | `WM_DEVICECHANGE` / WMI | udisks2 (D-Bus) ou ioctl |
| Gaveta | idem | `IOCTL_STORAGE_EJECT_MEDIA` / `LOAD_MEDIA`, MCI | `eject`, `eject -t` |
| Gravação | orquestração portátil | IMAPI2 (COM) | `xorriso` ou `wodim` (processo externo) |
| Steam: localização | Interface em `SystemIntegration` | Pasta da instalação + `steamapps/libraryfolders.vdf` | Steam nativa vs Flatpak mudam caminhos e comando (a verificar) |
| Steam: lançamento | URI `steam://run/<id>` | Abrir URI pelo SO | Idem; com Steam Flatpak, a forma de abrir pode mudar (a verificar) |
| Execução de `custom` | lista de args | `Command` sem shell; UAC | `Command` sem shell; permissões |
| Segredos | Interface | Armazenamento de credenciais do Windows (a verificar) | Secret Service (a verificar) |
| Janela / tela cheia / foco | API Tauri (`setFullscreen`, `setFocus`, `minimize`) ([Tauri](https://v2.tauri.app/reference/javascript/api/namespacewindow/)) | Regras de foreground do Windows (a verificar) | Compositor (X11/Wayland) e gamescope |
| Empacotamento | — | Instalador + assinatura (SmartScreen) | AppImage ou pacote de distro; Flatpak depois |

## Windows (fase 1)

Específico de Windows e isolado atrás das interfaces:

- `WindowsDrive`: `WM_DEVICECHANGE`, WMI, IOCTLs, IMAPI2 ([DRIVE-LAYER](DRIVE-LAYER.md#windowsdrive)).
- `WindowsSystem`: abrir URI `steam://`, localizar a Steam, UAC, foco de janela, credenciais.
- Instalador e assinatura ([W9](RISKS-AND-SPIKES.md#w9-empacotamento-assinatura-e-smartscreen)).

## Linux (fase posterior)

Pendente de confirmação. O que muda:

| Tema | Nota |
|---|---|
| Webview | WebKitGTK tem histórico de problemas de renderização com NVIDIA/Wayland (hipótese; spike [L2](RISKS-AND-SPIKES.md#linux-fase-posterior)) |
| Gamepad | Gamepad API só se o WebKitGTK tiver sido compilado com suporte ([BLFS](https://www.linuxfromscratch.org/blfs/view/stable-systemd/x/webkitgtk.html), [lista webkit-gtk](https://www.mail-archive.com/webkit-gtk@lists.webkit.org/msg03600.html)); `gilrs` é a alternativa |
| Drive | udisks2 expõe `MediaAvailable`, `OpticalBlank`, `OpticalNumAudioTracks`, `Eject` ([UDisks2](https://storaged.org/doc/udisks2-api/latest/gdbus-org.freedesktop.UDisks2.Drive.html)) |
| Gravação | `xorriso` (GPL, [Libburnia](https://en.wikipedia.org/wiki/Libburnia)) tem modo de emulação do cdrecord ([xorrecord(1)](https://manpages.ubuntu.com/manpages/jammy/man1/xorrecord.1.html)); `wodim` vem do cdrkit ([Cdrkit](https://en.wikipedia.org/wiki/Cdrkit)) |
| gamescope | Microcompositor da Valve; roda aninhado sobre um desktop comum ([README](https://github.com/ValveSoftware/gamescope/blob/master/README.md)). Requisito da fase Linux: app e jogos sob gamescope ([L6](RISKS-AND-SPIKES.md#linux-fase-posterior)). **Exclusivo de Linux.** |
| Empacotamento | Começar por AppImage ou pacote de distro. Flatpak fica para depois, porque o sandbox limita udisks2, o lançamento de programas do host e o acesso à Steam (hipótese; [L7](RISKS-AND-SPIKES.md#linux-fase-posterior)) |
| Steam | Nativa vs Flatpak: caminhos e comando diferentes ([L8](RISKS-AND-SPIKES.md#linux-fase-posterior)) |
