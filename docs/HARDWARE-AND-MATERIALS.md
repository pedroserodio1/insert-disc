# Hardware e materiais

Lista de referência, sem preços. Decisão de mídia em [ADR-0001](adr/0001-midia-cd-r-drive-usb-slim.md).

## Drive

| Requisito | Obrigatório? | Por quê |
|---|---|---|
| Externo, USB, formato slim | sim | ADR-0001 |
| Carregamento por **gaveta** (não slot) | sim | ADR-0001; experiência de "abrir a gaveta" |
| Grava CD-R | sim | Cadastro ([BURNING](BURNING.md)) |
| Grava e apaga CD-RW | sim | Testes e reuso |
| Suporte a ejeção por software | desejável | [W4](RISKS-AND-SPIKES.md#w4-controle-da-gaveta); sem ele, o app mostra instrução |
| Fechamento motorizado da gaveta | desejável, raro em slim (hipótese) | [W4](RISKS-AND-SPIKES.md#w4-controle-da-gaveta) |
| Alimentação por uma porta USB (ou cabo Y incluso) | recomendado | Portas fracas causam falha de gravação (hipótese) |
| Funciona com driver genérico do Windows | sim | Sem instalar software do fabricante |

Antes de comprar: procurar no manual ou em reviews do modelo as menções a "eject" por software e a gravação de CD-RW. Depois de comprar: rodar os spikes W2–W5 antes de gravar CD-R.

## Mídia

| Item | Uso |
|---|---|
| CD-RW (poucas unidades) | Testes de gravação e apagamento; iteração |
| CD-R | Discos definitivos |

## Etiqueta e impressão

| Item | Observação |
|---|---|
| Etiqueta adesiva para CD de 115 mm (ex.: Pimaco CD10B) | Colada **depois** de gravar; a arte é só visual |
| Impressora jato de tinta comum | Conferir no fabricante da etiqueta o modelo e as configurações de impressão |
| Aplicador de etiqueta (opcional) | Centralizar; etiqueta torta pode desbalancear o disco em rotação (hipótese) |

Gravar e depois colar a etiqueta: nunca colar antes, e nunca reposicionar.

## Controle

| Item | Observação |
|---|---|
| Controle XInput (ex.: compatível com Xbox) | Principal alvo do [W1](RISKS-AND-SPIKES.md#w1-gamepad-no-webview2) |
| Controle DirectInput genérico (se houver) | Cobertura do W1 |

## Opcional

- Caixas para CD (jewel case ou slim) com encarte impresso.
- Um CD de áudio e um CD-ROM prensado qualquer para testar a rejeição ([TESTING-WITH-ISO](TESTING-WITH-ISO.md#o-que-só-dá-para-validar-com-hardware-real)).
