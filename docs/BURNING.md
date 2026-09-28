# Cadastro e gravação

Decisões: [ADR-0008](adr/0008-cadastro-com-gravacao-pelo-app.md) e [ADR-0014](adr/0014-um-jogo-n-discos-e-adocao.md). A máquina de estados completa está em [UX-STATES](UX-STATES.md#cadastro-e-gravação).

## Fluxo

1. **Adicionar jogo**, na biblioteca.
2. **Inserir disco.** O app abre a gaveta, se puder.
3. **Verificar a mídia** e decidir:

   | Mídia | Ação |
   |---|---|
   | CD-R virgem | Segue |
   | CD-RW virgem | Segue |
   | CD-RW com conteúdo | Oferece apagar, com **confirmação dupla**, mostrando o que há no disco (`GAME.INI` e jogo do catálogo, se houver) |
   | CD-R com conteúdo (mesmo se "appendable") | **Recusa**, com o motivo; ejeta ao escolher "Tentar outro disco" |
   | CD-ROM prensado, CD de áudio, ilegível, não gravável | **Recusa**, com o motivo; idem |
   | Drive sem gravação | Nem chega aqui: `DRIVE_PROBLEM(cannot_burn)` ao adicionar jogo |

4. **Escolher o jogo:** um existente (nova cópia) ou um novo (Steam importado ou personalizado). A entrada fica em **rascunho** até a gravação terminar bem. Vindo de "jogo sem disco" ([ADR-0019](adr/0019-rejeicao-mantem-disco-e-atalhos.md)), o jogo já vem escolhido.
5. **Prévia do rótulo:** mostra o rótulo sanitizado (editável) e o `name` do INI ([DISC-FORMAT](DISC-FORMAT.md#regras-de-escrita)). Vale para CD-R e CD-RW.
6. **Aviso de CD-R:** gravação única, e uma falha perde o disco. Exige confirmação. Não aparece para CD-RW.
7. **Gravar:** o app gera um `disc_id` novo (UUID aleatório), monta a imagem (`GAME.INI` + rótulo sanitizado, [DISC-FORMAT](DISC-FORMAT.md)) e grava em sessão única, fechando o disco (hipótese a confirmar em [W5](RISKS-AND-SPIKES.md#w5-gravação-com-imapi2)).
8. **Verificar:** relê o `GAME.INI` e compara com o gerado.
9. **Concluir:** efetiva o rascunho (associa `disc_id` ao jogo) e sugere imprimir a etiqueta.

Invariante: **o catálogo só recebe o `disc_id` depois da verificação.** Um disco que falhou nunca vira chave válida.

## CD-RW: apagamento

- IMAPI2 oferece `IDiscFormat2Erase` com `EraseMedia` e a propriedade `FullErase` (rápido ou completo) ([Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/imapi2/nn-imapi2-idiscformat2erase)). Tempo e comportamento no drive real: não verificados.
- Se o disco apagado estava associado a um jogo, o `disc_id` antigo é **removido do catálogo só depois que o apagamento termina com sucesso**. Se o apagamento falhar, a associação continua (cenário C29 em [TESTING-WITH-ISO](TESTING-WITH-ISO.md#cenários)).

## Falhas

| Falha | Efeito | Tratamento |
|---|---|---|
| Erro durante a gravação (CD-R) | Disco provavelmente inutilizado | Mensagem clara; o rascunho do disco é descartado; oferece tentar com outro disco |
| Erro durante a gravação (CD-RW) | Disco recuperável com apagamento | Oferece "Apagar e tentar de novo" (apaga sem nova confirmação dupla, pois o conteúdo é a gravação que acabou de falhar) e volta à prévia do rótulo |
| Remoção do disco ou do drive durante a gravação | Idem acima | A UI avisa "não remova" durante toda a gravação; o cancelamento fica desabilitado |
| Queda do app ou do PC | O catálogo não tem o `disc_id` (invariante) | Na próxima inserção, o disco é `UNKNOWN` ou ilegível e pode ser adotado, se for legível |
| Verificação divergente | Disco não confiável | Trata como falha; não associa |
| Drive não grava CD-R/CD-RW | — | `capabilities` e recusa antes de qualquer escrita |

## Ferramentas candidatas (Windows primeiro)

| Opção | Licença / redistribuição | Prós | Contras | Status |
|---|---|---|---|---|
| **IMAPI2** (nativo, COM) | Faz parte do Windows; nada a redistribuir | Sem dependência externa; suporta CD-R/CD-RW, ISO 9660/Joliet/UDF, apagamento e verificação ([About IMAPI](https://learn.microsoft.com/en-us/windows/win32/imapi/about-imapi)) | Interop COM a partir do Rust; nome de volume limitado a 15 caracteres ([put_VolumeName](https://learn.microsoft.com/en-us/windows/win32/api/imapi2fs/nf-imapi2fs-ifilesystemimage-put_volumename)) | **Candidata principal**, spike [W5](RISKS-AND-SPIKES.md#w5-gravação-com-imapi2) |
| xorriso (libburnia), processo externo | GPL ([Wikipedia: Libburnia](https://en.wikipedia.org/wiki/Libburnia)); versão exata e build para Windows **não verificados** | Mesmo motor da fase Linux | Distribuição no Windows incerta; implicações de licença se for empacotado | Reservada para Linux |
| cdrkit / `wodim`, processo externo | Fork GPL do cdrtools ([Wikipedia: Cdrkit](https://en.wikipedia.org/wiki/Cdrkit)) | Conhecido no Linux | Mesmas incertezas no Windows | Reservada para Linux |
| Ferramenta comercial ou freeware de terceiros | Em geral incompatível com redistribuição num projeto open source | — | Licença e automação | Descartada, salvo evidência em contrário |

Chamar um programa GPL como processo separado **versus** empacotá-lo junto tem implicações diferentes. Isso entra na decisão de licença ([Q2](OPEN-QUESTIONS.md#q2-licença)), sem conclusão jurídica aqui.

## Drive falso

"Gravar" gera um `.iso` novo com a mesma imagem que iria para o disco ([DRIVE-LAYER](DRIVE-LAYER.md#fakeisodrive)). Isso valida o formato gerado de ponta a ponta. Não valida a gravação física.
