# Spike W2: WM_DEVICECHANGE

Descartável. Janela oculta de nível superior que imprime cada `WM_DEVICECHANGE` de volume, com o tempo desde o início.

```bash
cargo build
target/debug/spike-w2-devicechange.exe 20   # escuta por 20 s; monte/desmonte uma ISO ou troque um disco
```

## Resultado com ISO montada (2026-09-28, Windows 11)

Montar (`Mount-DiskImage`) e desmontar duas vezes:

| Momento | Mensagens recebidas |
|---|---|
| Montar | `DEVNODES_CHANGED` ×2, depois `DEVICEARRIVAL` de volume `H:` com `dbcv_flags = 0x0` |
| Desmontar | `DEVNODES_CHANGED`, depois `REMOVECOMPLETE` de volume `H:` com `dbcv_flags = 0x0` |
| Latência | ~1,2 s do início do `Mount-DiskImage` até a chegada; ~3 s de "assentamento" entre as mensagens e o fim do cmdlet |

**Não houve `DBTF_MEDIA`.** Uma ISO montada é um dispositivo (unidade virtual) que aparece e some, não uma mídia inserida num leitor. Combina com o que o WMI mostrou (unidade `Microsoft Virtual DVD-ROM` nova a cada montagem).

## Conclusões

- **Não dá para depender da flag `DBTF_MEDIA`** para detectar inserção. O `WindowsDrive` deve tratar `DEVICEARRIVAL`/`REMOVECOMPLETE` de volume e `DEVNODES_CHANGED` como **gatilhos para reler o estado** da unidade escolhida (`MediaLoaded`, `VolumeName`), e emitir `MediaArrived`/`MediaRemoved` a partir da diferença.
- Continua valendo o polling como plano B (`Win32_CDROMDrive.MediaLoaded`, testado e funcional).

## Falta (exige hardware)

Drive USB físico: chegam `DBTF_MEDIA`? Com que latência? A gaveta aberta gera algo?
