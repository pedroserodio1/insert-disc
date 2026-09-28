# ADR-0001: Mídia CD-R/CD-RW em drive USB slim de gaveta

**Status:** aceito

## Contexto
O original usa disquete. Hoje, disquetes e drives são escassos. CDs são baratos, graváveis em casa e dão a sensação de "mídia de console".

## Decisão
- Mídia: CD-R para discos definitivos; CD-RW para testes.
- Drive: gravador/leitor externo USB slim, de gaveta.
- Etiqueta adesiva de 115 mm (ex.: Pimaco CD10B), impressa em jato de tinta comum e colada **depois** de gravar. A arte é só visual.

## Consequências
- O controle da gaveta por software varia por modelo. O app detecta o suporte e degrada com instrução na tela ([W4](../RISKS-AND-SPIKES.md#w4-controle-da-gaveta)).
- A gravação exige drive gravador ([BURNING](../BURNING.md)).
- Requisitos de compra: [HARDWARE-AND-MATERIALS](../HARDWARE-AND-MATERIALS.md).
