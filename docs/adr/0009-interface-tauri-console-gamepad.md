# ADR-0009: Interface Tauri com visual de console e controle

**Status:** aceito; a forma de ler o controle depende do [W1](../RISKS-AND-SPIKES.md#w1-gamepad-no-webview2)

## Contexto
O objetivo é uma experiência de console, com o mesmo stack no Windows e (depois) no Linux.

## Decisão
- Aplicativo Tauri com visual de console.
- Navegação completa por controle (gamepad).
- Modos janela e tela cheia.
- Compatibilidade com gamescope é requisito da **fase Linux**, pois o gamescope é exclusivo de Linux.

## Consequências
- Há um bug conhecido da Gamepad API no WebView2 ([W1](../RISKS-AND-SPIKES.md#w1-gamepad-no-webview2)); a alternativa é `gilrs` no Rust.
- No Linux, a webview é WebKitGTK ([PLATFORMS](../PLATFORMS.md)).
