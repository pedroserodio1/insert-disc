# Formato do disco

Decisões: [ADR-0005](adr/0005-disco-guarda-o-minimo.md), [ADR-0012](adr/0012-game-ini-minimo-e-label-informativo.md), [ADR-0014](adr/0014-um-jogo-n-discos-e-adocao.md).

## O que vai no disco

| Item | Conteúdo | Usado para decidir? |
|---|---|---|
| `GAME.INI` (raiz) | `id` (UUID aleatório, um por disco) e `name` | **Só o `id`**, e apenas como chave de busca no catálogo |
| Rótulo do volume | Derivado do nome do jogo (ver limites abaixo) | Não. Serve para exibir e para humanos identificarem o disco. |

## O que **não** vai no disco, e por quê

| Fora do disco | Motivo |
|---|---|
| AppID, executável, argumentos, ROM, processo | O disco não pode definir o que roda ([ADR-0006](adr/0006-catalogo-local-define-execucao.md)) |
| Capa e metadados | Ficam no catálogo; não há motivo para gastar a mídia com eles |
| Versão do formato | Em aberto ([Q9](OPEN-QUESTIONS.md#q9-campo-de-versão-no-gameini)) |
| Autorun e executáveis | Nunca |

## Exemplo

```ini
[disc]
id = 6f1c2a4e-9b7d-4c3e-8a51-2d0f7e9b1c44
name = Hollow Knight
```

## Regras de leitura

| Regra | Valor / comportamento |
|---|---|
| Localização | `GAME.INI` na raiz do volume. A comparação do nome do arquivo ignora maiúsculas e minúsculas. |
| Tamanho máximo | Limite pequeno e fixo (proposta: 4 KiB). Acima disso: `INVALID_GAME_INI`. |
| Codificação | UTF-8 (com ou sem BOM). Bytes inválidos: `INVALID_GAME_INI`. |
| Fim de linha | LF ou CRLF |
| Seção e chaves | Seção `[disc]`; chaves `id` e `name`. Nomes de seção e de chave comparados sem diferenciar maiúsculas. Chaves desconhecidas e outras seções são **ignoradas** (nunca interpretadas). |
| Formato da linha | `chave = valor`; espaços ao redor de `=` e nas pontas do valor são removidos. Aspas não têm significado especial (fazem parte do valor). |
| Comentários | Linhas iniciadas por `;` ou `#` são ignoradas |
| Chave duplicada | `id` ou `name` repetidos na seção `[disc]`: `INVALID_GAME_INI` |
| `id` | UUID na forma canônica de 36 caracteres com hífens (`8-4-4-4-12`), hexadecimal em qualquer caixa; normalizado para minúsculas antes da busca. Com chaves `{}`, sem hífens ou UUID nulo (`00000000-…`): `INVALID_GAME_INI`. |
| `name` | Opcional na leitura. Truncado para exibição. Nunca vira caminho, comando nem chave de busca. |
| Sem seção `[disc]`, mas com chaves do projeto original (`NAME`, `STEAMID`, `PROCESS`, `COVER`, `DISKID`) | `LEGACY_GAME_INI` ([UX-STATES](UX-STATES.md#classificação-de-mídia)) |
| Outros arquivos | Ignorados. |

## Regras de escrita

| Regra | Valor |
|---|---|
| Conteúdo | Exatamente a seção `[disc]` com `id` e `name`, nessa ordem; UTF-8 sem BOM; fim de linha CRLF |
| `id` | UUID v4 aleatório, minúsculas |
| `name` | Nome do jogo sem caracteres de controle (incluindo quebras de linha), sem `[`/`]` no início, com no máximo 64 caracteres. Garante que o arquivo gerado sempre relê com um único `id` (cenário C28 em [TESTING-WITH-ISO](TESTING-WITH-ISO.md#cenários)). |

## Rótulo do volume

"Rótulo = nome do jogo" é a intenção. **Os limites técnicos fazem o rótulo ser uma versão abreviada e sanitizada do nome.**

| Fonte | Limite |
|---|---|
| IMAPI2 `IFileSystemImage::put_VolumeName` | "The string is limited to 15 characters". ISO 9660 aceita `A–Z`, `0–9`, `_`; Joliet/UDF aceitam também minúsculas e `.` ([Microsoft Learn](https://learn.microsoft.com/en-us/windows/win32/api/imapi2fs/nf-imapi2fs-ifilesystemimage-put_volumename)) |
| ISO 9660 (volume identifier, PVD) | Campo de 32 caracteres, d-characters ([OSDev Wiki](https://wiki.osdev.org/ISO_9660)) |
| Joliet | Nome do volume limitado a 16 caracteres UCS-2 ([Joliet Specification](https://pismotec.com/cfs/jolspec.html)) |

Consequências:

- Não há espaço no charset do IMAPI2. "Hollow Knight" viraria algo como `HOLLOW_KNIGHT` (ISO 9660) ou `Hollow_Knight` (Joliet).
- A regra exata de abreviação e sanitização fica para o spike de gravação ([W5](RISKS-AND-SPIKES.md#w5-gravação-com-imapi2)).
- Como o rótulo é informativo, colisões e truncamentos não causam erro.

## Sistema de arquivos

Hipótese: ISO 9660 + Joliet, sessão única, disco fechado. Um nome `GAME.INI` cabe no formato 8.3 do ISO 9660 nível 1. Confirmar no spike [W5](RISKS-AND-SPIKES.md#w5-gravação-com-imapi2), incluindo se o Windows e (depois) o Linux leem o arquivo sem surpresas.

## Identidade

- O `id` é do **disco**, não do jogo. Um jogo pode ter N discos ([ADR-0014](adr/0014-um-jogo-n-discos-e-adocao.md)).
- Copiar um disco bit a bit cria outro disco com o mesmo `id`. Isso é aceito: o projeto não é DRM ([VISION](VISION.md#o-que-o-projeto-não-é)).
