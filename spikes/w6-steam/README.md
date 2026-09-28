# Spike W6: integração com a Steam

Descartável. Localiza a Steam (registro), lê `libraryfolders.vdf`, lista os `appmanifest_*.acf` de cada biblioteca e procura capas em `appcache/librarycache`. Só leitura.

```bash
cargo run
```

## Resultado (2026-09-28, PC do autor, Windows 11)

| Item | Resultado |
|---|---|
| Localizar a Steam | `HKCU\Software\Valve\Steam` → `SteamPath` funcionou (`c:/program files (x86)/steam`) |
| Handler `steam://` | Registrado em `HKCR\steam\shell\open\command`: `steam.exe -- "%1"` |
| Bibliotecas (formato novo do `libraryfolders.vdf`) | 2 lidas (`"path"` dentro de cada entrada numerada), uma em outro disco (`E:\JOGUITOS`) |
| `appmanifest_*.acf` | 28 manifests → 26 jogos após descartar ferramentas (nome começando em `Steamworks Common`/`Proton`…) |
| Capas em `librarycache/<appid>/` | **14 de 26** têm `library_600x900.jpg`; **12 de 26** só têm ícone 32×32 e `logo.png` 640×360, sem capa retrato |

## Conclusões

- **Sucesso** nos critérios de instalação e bibliotecas: dá para listar todos os jogos instalados, em vários discos.
- **Capas locais cobrem só 54%** neste PC. O placeholder com o nome e as capas online opcionais ([ADR-0015](../../docs/adr/0015-i18n-e-capas-online-opcionais.md)) não são luxo: são necessários.
- O nome do arquivo com hash **não** é uma capa alternativa neste PC (eram ícones e logos). Não vale procurar heurística de "imagem retrato" em nomes hash.
- Ferramentas (Proton, Steamworks Redistributables) aparecem como `appmanifest` e precisam de filtro.

## Não verificado

- Qual dos esquemas (`steam://run`, `rungameid`, `launch`) é o melhor: não abri nenhum jogo para não ter efeito colateral.
- Comportamento com Steam Flatpak/Linux (fase posterior).
- Formato de `libraryfolders.vdf` mais antigo: o código trata (`"1" "D:\\Lib"`), mas só foi exercitado com o formato novo.
