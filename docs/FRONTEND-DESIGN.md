# Design do front-end

Especificação visual e de interação da UI. Implementador previsto: **Sonnet 5.5**. Direção escolhida pelo autor: **Estante de CDs** ([ADR-0018](adr/0018-direcao-visual-estante-de-cds.md)).

Divisão de responsabilidade entre documentos:

| Assunto | Fonte de verdade |
|---|---|
| Estados, transições, o que cada botão faz em cada estado | [UX-STATES](UX-STATES.md) |
| Protocolo entre UI e núcleo | [UI-CONTRACT](UI-CONTRACT.md) |
| Aparência, movimento, layout e **todos os textos visíveis** | Este documento |

## 0. Como ler (para o implementador)

| Marca | Significado |
|---|---|
| **DEVE** | Obrigatório. Desviar exige conversar com o autor. |
| **DEVERIA** | Padrão esperado; desvio aceitável com justificativa no PR. |
| **PODE** | Liberdade do implementador. |

Regras gerais:

1. **DEVE** usar só os tokens da seção 3 (cor, tipo, dimensão, opacidade, duração). Todo número visual deste documento é um token; se faltar um, crie-o na seção 3 e avise no PR.
2. **DEVE** passar todo texto visível pelo i18n; as chaves estão na §9 ([ADR-0015](adr/0015-i18n-e-capas-online-opcionais.md)). O núcleo envia códigos, nunca texto ([UI-CONTRACT](UI-CONTRACT.md#princípios)).
3. **DEVE** tratar controle como entrada principal; teclado e mouse também funcionam em todas as telas (§7).
4. **NÃO DEVE** usar kit de componentes pronto (Material, shadcn, Bootstrap etc.). Framework: [Q17](OPEN-QUESTIONS.md#q17-framework-do-frontend).
5. **DEVE** empacotar a fonte localmente e incluir a licença OFL da Archivo no aviso de terceiros do app.
6. **DEVERIA** revisar cada tela por screenshot usando o drive falso ([DRIVE-LAYER](DRIVE-LAYER.md#fakeisodrive)) contra os wireframes daqui. O painel de controle do drive falso é ferramenta de dev: visual simples, fora deste design.

## 1. Assunto, público, trabalho

| | |
|---|---|
| **Assunto** | Uma coleção pessoal de jogos em CD-R gravados em casa, guardados em caixas de CD numa estante |
| **Público** | Quem joga no PC (muitas vezes na TV, do sofá, com controle) e gosta de mídia física |
| **Trabalho principal** | Escolher um jogo e transformar "colocar o disco" num ritual curto e satisfatório |
| **Distância de uso** | "10-foot UI": a 2–3 m de uma TV, em 1080p |

## 2. Conceito: a estante

| Objeto real | Vira na UI |
|---|---|
| Lombada da caixa | Item da biblioteca: nome do jogo em tipo condensado, na vertical |
| Caixa puxada da estante | Item em foco: sai da fila e gira para mostrar a capa |
| Caixa aberta, **bandeja vazia** | "Aguardando disco": o disco está na gaveta do drive, não na caixa |
| Contracapa da caixa | Ficha de leitura (o que o app identificou no disco) |
| Corante do CD-R (cianina, ftalocianina, azo) | A paleta |
| Etiqueta de 115 mm colada no disco | A capa recortada em círculo no disco |
| CD-R gravado de dentro para fora | O progresso da gravação |

**Ousadia num lugar só:** o **disco**. É o único elemento com iridescência e o único protagonista de uma sequência orquestrada (§6.5).

## 3. Tokens

### 3.1 Cor

| Token | Hex | Origem | Uso |
|---|---|---|---|
| `azo` | `#14264A` | Corante azo | Fundo de todas as telas; texto sobre `etiqueta` |
| `azo-fundo` | `#0E1D3A` | Azo na sombra | Interior da estante, fundo de avisos, verso da capa |
| `policarbonato` | `#C4CAD3` | Face prateada do disco | Texto secundário sobre `azo`; aresta das lombadas; face do disco genérico |
| `etiqueta` | `#F3F5F4` | Etiqueta adesiva branca fosca | Texto principal sobre `azo`; superfície de fichas e diálogos |
| `ftalocianina` | `#D8B64C` | Corante ftalocianina | **Foco** e ação primária sobre `azo` |
| `cianina` | `#3FB8AF` | Corante cianina | Indicadores em andamento sobre `azo` (ler, gravar, verificar) |
| `cianina-gravada` | `#1F6F69` | Corante após gravação | Área gravada do disco |
| `laser` | `#F2645F` | Laser de leitura | Erros e rejeições sobre `azo` |
| `laser-escuro` | `#B3261E` | — | Erros sobre `etiqueta` (anel de "segurar", textos de erro em diálogos) |
| `grafite` | `#5B6472` | — | Texto secundário sobre `etiqueta` (ex.: "não informado"); face do disco ilegível |

Contraste (WCAG, calculado):

| Par | Razão | Uso permitido |
|---|---|---|
| `etiqueta` / `policarbonato` / `ftalocianina` / `cianina` / `laser` sobre `azo` | 13,6 · 9,0 · 7,6 · 6,2 · 4,8 | Texto e bordas |
| `azo` / `grafite` / `laser-escuro` sobre `etiqueta` | 13,6 · 5,5 · 6,0 | Texto e bordas |
| `ftalocianina`, `policarbonato`, `cianina`, `laser` sobre `etiqueta` | < 3 | **Proibido** para texto, borda ou foco |

**Iridescência** (só no componente Disco): gradiente cônico em torno do centro, com fatias estreitas e irregulares de `cianina`, `ftalocianina`, `policarbonato` e `azo` em baixa opacidade sobre `policarbonato`.

**Cor da lombada:** calculada pelo **backend** ao importar ou trocar a capa (cor dominante, escurecida até contraste ≥ 4,5:1 com `etiqueta`) e guardada no catálogo (`spine_color`, [CATALOG-SPEC](CATALOG-SPEC.md#game)). Sem capa: `policarbonato` com texto `azo`. O contraste da lombada com o fundo é garantido pela aresta (§5.1), não pela cor.

### 3.2 Tipografia

**Uma família: Archivo** (variável, OFL; `wdth` 62–125, `wght` 100–900; [google/fonts METADATA](https://github.com/google/fonts/blob/main/ofl/archivo/METADATA.pb)). O contraste entre papéis vem da **largura**, como nas lombadas de CD dos anos 90–2000. Algarismos tabulares (`tnum`) em todo número que muda.

| Token | `wdth` | `wght` | Tamanho | Entrelinha | Uso |
|---|---|---|---|---|---|
| `tipo-lombada` | 62 | 600 | 24 | 1,0 | Nome na lombada |
| `tipo-titulo` | 112 | 700 | 60 | 1,05 | Título de tela |
| `tipo-secao` | 100 | 600 | 36 | 1,15 | Diálogos, listas de gestão |
| `tipo-corpo` | 100 | 400 | 28 | 1,4 | Mensagens, instruções |
| `tipo-apoio` | 100 | 400 | 21 | 1,35 | Ficha, dicas de botão, texto secundário |
| `tipo-dado` | 88 | 500 | 24 | 1,3 | Rótulo do volume e UUID |
| `tipo-marca` | 125 | 700 | 72 | 1,0 | Nome do app no `BOOT` |

Escala: 21 · 24 · 28 · 36 · 60 · 72, baseada na escala clássica de Bringhurst, com o 28 acrescentado para leitura de corpo a distância de TV.

**Escala de tela (DEVE):** tamanhos em px lógicos @1080p. Fator = `altura interna da janela em px CSS / 1080` (o WebView já aplica o DPI do Windows). Janela mínima: 1280×720.

Regras de texto: sentence case; **sem** rótulos em caixa alta (exceto o rótulo do volume, que é dado real); corpo com no máximo 60 caracteres por linha; sem destacar uma palavra isolada; UUID em `tipo-dado` (não monoespaçado).

### 3.3 Dimensões, opacidade, forma

Valores em px lógicos @1080p (escalados pelo fator da §3.2).

| Token | Valor | Uso |
|---|---|---|
| `u` | 8 | Unidade de espaçamento; todo espaçamento é múltiplo de `u` |
| `margem-segura` | 5% de cada borda | Overscan de TV |
| `lombada-largura` | 40 | Lombada (proporção real 10 × 125 mm) |
| `caixa-altura` | 440 | Lombada e frente |
| `frente-largura` | 500 | Frente da caixa (proporção real 142 × 125 mm) |
| `disco-caixa` | 422 | Diâmetro do disco dentro da caixa (120 mm na mesma escala) |
| `disco-lancamento` | 60% da altura da janela | Diâmetro do disco em `LAUNCHING` e cadastro |
| `z-puxar` | 160 | Deslocamento em Z da caixa puxada |
| `borda-foco` | 3 | Foco |
| `borda-aviso` | 4 | Barra lateral do aviso |
| `borda-caixa` | 1 | Aresta do acrílico (`policarbonato` a `opac-aresta`) |
| `raio-caixa` | 3 | Cantos da caixa |
| `raio-folha` | 0 | Fichas e diálogos |
| `opac-aresta` | 40% | Aresta da caixa |
| `veu-estante` | `azo` a 60% | Sobre a estante quando uma caixa está aberta |
| `veu-dialogo` | `azo` a 80% | Atrás de diálogos |

Sombra: só na caixa puxada e no disco (projeção para baixo em `azo-fundo`). Alinhamento: **à esquerda** em todas as telas de texto; só o disco e a caixa em foco são centralizados nos seus eixos.

### 3.4 Movimento

| Token | Valor | Uso |
|---|---|---|
| `dur-mover-foco` | 160 ms, ease-out | Foco entre lombadas |
| `dur-puxar` | 280 ms, ease-out | Puxar caixa (§5.2.1) |
| `dur-devolver` | 200 ms, ease-in-out | Devolver caixa |
| `dur-abrir` | 360 ms, ease-in-out | Abrir caixa |
| `dur-transicao` | 200 ms, ease-out | Troca de tela e entrada de diálogo (opacidade + 1 `u`) |
| `dur-reduzido` | 120 ms | Troca por opacidade com movimento reduzido |
| `dur-entrada-disco` | 800 ms | Coreografia de entrada do disco (§6.5) |
| `dur-giro-aceleracao` | 600 ms | Disco acelera até ≈ 1 volta/s |
| `dur-ficha-min` | 600 ms | Ficha visível em `MATCH` (cronometrado pelo núcleo) |
| `dur-aviso` | 5000 ms | Tempo de um aviso na tela |
| `dur-segurar` | 1500 ms | Confirmar segurando (§5.7) |
| `dur-repeticao-atraso` / `dur-repeticao-intervalo` | 400 / 80 ms | Direcional segurado |
| `dur-hover` | 150 ms | Permanência do mouse antes de mover o foco |
| `overshoot-giro` | ≤ 4° de rotação | Assentamento da caixa puxada |

**Animações autônomas** (sem ação do usuário) permitidas, e só estas:

1. A coreografia de lançamento (§6.5), que é o momento marcante;
2. O arco de leitura (Disco › Lendo) em `READING`, `REG_READING` e `VERIFYING`;
3. O crescimento/encolhimento da área gravada em `BURNING` e `ERASING`.

As duas últimas são indicadores de espera, não decoração. O disco **não gira** durante a gravação.

`prefers-reduced-motion`: sem rotação 3D, sem deslocamento em Z, sem giro, sem overshoot; trocas por opacidade em `dur-reduzido`. Os indicadores de espera viram texto + percentual estático.

## 4. Layout da biblioteca

```
┌──────────────────────────────────────────────────────────────┐
│ (margem segura)                                               │
│   Hollow Knight                          ← tipo-titulo        │
│   Steam                                  ← tipo-apoio         │
│   2 discos                                                    │
│                                                               │
│  ┃┃┃┃┃┃┃┃┃  ╔═════════════╗  ┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃           │
│  ┃┃┃┃┃┃┃┃┃  ║    frente   ║  ┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃  estante  │
│  ┃┃┃┃┃┃┃┃┃  ║   da caixa  ║  ┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃           │
│  ┃┃┃┃┃┃┃┃┃  ╚═════════════╝  ┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃┃           │
│ ▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔ chão      │
│   (A) Jogar  (X) Adicionar jogo  (Y) Opções  (≡) Configurações│
└──────────────────────────────────────────────────────────────┘
```

- Caixa em foco a ≈ 1/3 da largura, a partir da esquerda; a estante rola para manter isso.
- Barra de ações abaixo do chão, à esquerda; mostra só as ações válidas (vêm de `actions` no [UI-CONTRACT](UI-CONTRACT.md)).
- Ordem das lombadas: [Q21](OPEN-QUESTIONS.md#q21-ordenação-da-biblioteca).

## 5. Componentes

### 5.1 Lombada

| Aspecto | Especificação |
|---|---|
| Tamanho | `lombada-largura` × `caixa-altura` |
| Fundo | `spine_color` do jogo |
| Aresta | 1 px `policarbonato` na borda esquerda (brilho do acrílico): garante a separação visual contra o fundo, qualquer que seja a cor |
| Texto | Nome, `tipo-lombada`, `etiqueta`, de cima para baixo (rotação 90° horária), alinhado ao topo com 2 `u`; trunca com reticências |
| Marca | Pequeno círculo `policarbonato` no pé, se o jogo tem ≥ 1 disco |

### 5.2 Caixa (jewel case)

| Estado | Aparência |
|---|---|
| **Na estante** | = Lombada |
| **Puxada** (foco) | Fora da estante, frente visível (§5.2.1). **Indicador de foco:** borda `ftalocianina` de `borda-foco` ao redor. Sombra. |
| **Aberta** | Abre como livro: à esquerda, o verso da capa (`azo-fundo`, com o nome do jogo em `tipo-secao`); à direita, a bandeja: contorno tracejado do disco (`disco-caixa`) em `policarbonato` e o pino central |

**Frente** (`frente-largura` × `caixa-altura`, quase quadrada como a caixa real):

```
┌──────────────────┐
│ faixa   │ arte   │   arte na altura total, colada à direita;
│ (cor da │ (2:3)  │   faixa ocupa a largura restante, na cor
│ lombada)│        │   da lombada, com o nome no rodapé
│ nome    │        │   (tipo-apoio, wdth 62)
└──────────────────┘
```

| Proporção da arte | Regra |
|---|---|
| Mais estreita que a frente (ex.: 2:3) | Altura total, colada à direita; faixa na largura restante. Se a faixa ficar com menos de 10 `u`, usar recorte centralizado sem faixa. |
| Igual ou mais larga que a frente | Recorte centralizado cobrindo a frente, sem faixa |
| Sem capa | Frente inteira na cor da lombada, com o nome em `tipo-secao` (`wdth` 62) no rodapé à esquerda |

#### 5.2.1 Coreografia 3D da caixa

A caixa é um bloco 3D (CSS 3D: `perspective` no contêiner da estante; `transform-style: preserve-3d`; `backface-visibility: hidden`). Faces renderizadas: lombada e frente. Na estante, a lombada encara a tela e a frente está girada 90°.

**Puxar (ganhar foco), `dur-puxar`:**

1. 0–120 ms: a caixa sai da estante em direção ao espectador (`z-puxar`), e as vizinhas se afastam para abrir `frente-largura`.
2. 60–280 ms (sobreposto): gira em Y de 90° para 0° e chega à posição de foco, assentando com `overshoot-giro`.
3. Últimos 80 ms: borda de foco e sombra aparecem.

**Devolver:** o inverso, em `dur-devolver`, sem overshoot.

**Navegação rápida (DEVE):** nova entrada antes do fim da animação faz a animação em curso **pular para o estado final**. Com o direcional segurado, as caixas intermediárias não giram: só a última, ao soltar. A estante nunca atrasa em relação ao controle.

**Abrir, `dur-abrir`:** a frente gira na dobradiça esquerda (`rotateY` 0° → −160°), revelando a bandeja; a caixa desliza para o centro-esquerda ao mesmo tempo.

**Desempenho (DEVE):** animar só `transform` e `opacity`. Apenas a caixa em foco e duas vizinhas de cada lado ficam em 3D. Meta: 60 fps em 1080p ([W13](RISKS-AND-SPIKES.md#w13-desempenho-de-css-3d-no-webview2)).

### 5.3 Disco

Proporções reais: 120 mm de diâmetro, furo de 15 mm, área de etiqueta até 115 mm.

| Variante | Aparência |
|---|---|
| **Etiquetado** | Capa recortada em círculo (recorte centralizado, cobrindo a área da etiqueta), furo, anel externo iridescente. Sem capa: etiqueta em `etiqueta` com o nome centralizado (`tipo-apoio`, cor `azo`). |
| **Genérico** | Sem etiqueta: face `policarbonato` iridescente |
| **Virgem** | Face em `cianina` com iridescência |
| **Ilegível** | Face `grafite`, sem iridescência |
| **Lendo** | Contorno tracejado com arco `cianina` percorrendo a borda |
| **Gravando** | Virgem + área em `cianina-gravada` crescendo do centro para fora (raio = progresso); percentual ao lado em `tipo-corpo` com `tnum` |
| **Apagando** | Inverso de Gravando: a área gravada encolhe até o centro |
| **Girando** | Etiquetado com giro; a iridescência fica fixa enquanto a etiqueta gira |
| **Rejeitado** | Variante da classe (tabela abaixo) + contorno `laser` de `borda-foco` |

Disco em `REJECTED` por classe:

| Classe | Variante |
|---|---|
| `OTHER_GAME` | Etiquetado com a capa de Y |
| `UNKNOWN`, `LEGACY_GAME_INI`, `NO_GAME_INI`, `INVALID_GAME_INI`, `AUDIO` | Genérico |
| `BLANK` | Virgem |
| `READ_ERROR` | Ilegível |

### 5.4 Ficha de leitura (contracapa)

Fundo `etiqueta`, texto `azo`, `raio-folha`. Lista de definição **sem numeração**. Termos em `tipo-apoio`; valores em `tipo-dado`.

| Linha | Quando aparece |
|---|---|
| Rótulo | Sempre |
| Mídia | Sempre |
| Identificador | Sempre |
| Nome no disco | Quando o `GAME.INI` tem `name` (destaque em `UNKNOWN`) |
| Pertence a | `OTHER_GAME` e `KNOWN` (nome de Y) |

Valor ausente: `common.not_informed` em `grafite`.

### 5.5 Barra de ações, listas de ações e glifos

- Cada ação = **glifo do botão** + verbo (§9). Ação primária: glifo preenchido em `ftalocianina`; demais: contorno `policarbonato`.
- Glifos conforme o controle detectado (Xbox: A/B/X/Y; PlayStation: ✕/○/□/△); sem controle ou com teclado/mouse como última entrada: teclas (§7). Fallback: Xbox.
- Quando um estado tem **várias ações equivalentes** (ex.: `REJECTED`), elas aparecem como **lista vertical focável** acima da barra; A ativa a ação em foco; a barra mostra só "(A) Selecionar" e "(B) Voltar". O foco inicial é a ação padrão (`actions` marcada no [UI-CONTRACT](UI-CONTRACT.md)).
- As dicas da barra são **clicáveis** com o mouse.

### 5.6 Aviso (toast)

- **Só informativo: sem ações.** Não consome entrada: qualquer botão continua agindo na tela atual.
- Faixa baixa, acima do chão, à esquerda; fundo `azo-fundo`; barra lateral de `borda-aviso` na cor semântica (`ftalocianina` informativo, `laser` problema).
- Some após `dur-aviso`. Um por vez; um novo substitui o anterior.

### 5.7 Diálogo de confirmação

- Folha `etiqueta` (`raio-folha`), coluna de texto à esquerda, com `veu-dialogo` atrás.
- **Foco dentro de folhas `etiqueta`:** borda dupla, `azo` externa (`borda-foco`) + `ftalocianina` interna de 2 px. Contraste garantido pela `azo`.
- Em diálogos destrutivos (apagar CD-RW, gravar CD-R), o foco inicial fica em **Voltar**.

**Confirmar segurando** (segundo passo de apagar CD-RW):

| Entrada | Comportamento |
|---|---|
| Controle | Segurar A por `dur-segurar` |
| Teclado | Segurar Enter: conta de `keydown` a `keyup`, **ignorando autorepeat** |
| Mouse | Pressionar e segurar o botão na tela |

Um anel `laser-escuro` preenche o glifo durante o tempo. Soltar antes zera o anel. Ao completar, a UI envia `hold_complete`.

## 6. Telas por estado

Estados, ações e destinos: [UX-STATES](UX-STATES.md). Aqui só a aparência.

### 6.1 `BOOT`

Fundo `azo`, nome do app em `tipo-marca` à esquerda na margem segura. Sem disco.

### 6.2 `LIBRARY`

Layout da §4. Biblioteca vazia:

```
│   Sua estante está vazia                         ← tipo-titulo│
│   Adicione um jogo e grave o disco dele.         ← tipo-corpo │
│                                                               │
│  (estante sem lombadas, só o chão)                            │
│ ▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔           │
│   [(A) Adicionar jogo]   (≡) Configurações                    │
```

O foco começa em "Adicionar jogo" (sempre existe um elemento focado).

### 6.3 Fluxo do disco: `NO_DISC_YET`, `WAITING_DISC`, `READING`, `IDENTIFIED`, `REJECTED`

A caixa em foco **abre** e desliza ao centro-esquerda; `veu-estante` sobre a estante.

```
┌──────────────────────────────────────────────────────────────┐
│   Coloque o disco de Hollow Knight na gaveta                  │
│   A gaveta está abrindo.                                      │
│                                                               │
│        ╔══════════╦══════════╗        ┌ ficha de leitura ┐    │
│        ║ verso    ║  ┌ ─ ─ ┐ ║        │ (IDENTIFIED e     │    │
│        ║ (nome do ║    ( · ) ║        │  REJECTED)        │    │
│        ║  jogo)   ║  └ ─ ─ ┘ ║        └───────────────────┘    │
│        ╚══════════╩══════════╝                                │
│   (lista de ações em REJECTED)                                │
│ ▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔           │
│   (B) Voltar                                                  │
└──────────────────────────────────────────────────────────────┘
```

| Estado | Na bandeja | Título / apoio |
|---|---|---|
| `NO_DISC_YET` | Vazia, sem contorno tracejado | `nodisc.title` / `nodisc.body` |
| `WAITING_DISC` | Contorno tracejado + pino | `wait.title` / `wait.tray_opening`, `wait.tray_manual` ou `wait.tray_failed` |
| `READING` | Disco › Lendo | `reading.title` |
| `IDENTIFIED` | Disco › Lendo parado | Ficha à direita |
| `REJECTED` | Disco › Rejeitado (§5.3) | `reject.<classe>` + ficha + lista de ações |

### 6.4 `ADOPT_CONFIRM`

Diálogo (5.7): `adopt.title`, `adopt.body`; ações `adopt.confirm` / `action.back`.

### 6.5 `LAUNCHING`: o momento marcante

Coreografia completa (quando `min_ms ≥ dur-entrada-disco`):

1. 0–300 ms: a caixa se fecha e sai de cena; a estante some por opacidade.
2. 300–800 ms: o disco etiquetado **vira e vem para a frente**. Começa deitado, como na gaveta (`rotateX` ≈ 75°, pequeno, no terço inferior direito); levanta até ficar de frente (`rotateX` 0°) enquanto se aproxima e chega ao centro com `disco-lancamento`. A iridescência do anel muda de ângulo durante a virada.
3. 800 ms até o fim: o disco gira (`dur-giro-aceleracao`). Abaixo, à esquerda: `launch.title` em `tipo-secao`. Sem barra de progresso.
4. Fim: fade para `azo` puro; a janela sai da frente.

Tempo ([UI-CONTRACT](UI-CONTRACT.md#tempo)): com `min_ms` < `dur-entrada-disco`, as fases 1–2 são comprimidas proporcionalmente até caber; com `min_ms = 0`, nenhuma animação (corte direto). O jogo é disparado no início de `LAUNCHING` e pode aparecer por cima da animação; isso é aceitável.

```
┌──────────────────────────────────────────────────────────────┐
│                         ╭──────────╮                          │
│                      ╭──┤  capa    ├──╮   disco girando,      │
│                      │  │   (·)    │  │   anel iridescente    │
│                      ╰──┤          ├──╯                       │
│                         ╰──────────╯                          │
│   Abrindo Hollow Knight                                       │
└──────────────────────────────────────────────────────────────┘
```

`LAUNCH_ERROR`: o disco para, ganha o contorno `laser`; `launch.error.<motivo>`; ação `action.back_to_shelf`.

### 6.6 Cadastro e gravação

| Estado | Tela |
|---|---|
| `REG_INSERT` | Caixa aberta vazia. Verso: `reg.case_label` (ou o nome do jogo, se veio de `NO_DISC_YET`). Título `reg.insert`. |
| `REG_READING` | Disco › Lendo, `reg.reading` |
| `REG_ERASE_CONFIRM` | Diálogo com a ficha do conteúdo; passo 1 `erase.title`/`erase.body` e `action.erase`; passo 2 `erase.hold` (segurar) |
| `ERASING` | Disco › Apagando no centro, `erase.progress` |
| `REG_CHOOSE_GAME` | Lista vertical em `tipo-secao`: `reg.new_steam`, `reg.new_custom`, depois os jogos existentes (cada um com a lombada deitada à esquerda do nome) |
| `REG_LABEL_PREVIEW` | Disco › Virgem com o rótulo gravado ao redor do furo; campo editável do rótulo; `reg.label.*` |
| `REG_CDR_WARNING` | Diálogo `reg.cdr_warning.*`; ações `action.burn` / `action.back` (foco em Voltar) |
| `BURNING` | Disco › Gravando no centro (`disco-lancamento`), `burn.progress` |
| `VERIFYING` | Disco › Lendo sobre o disco gravado, `burn.verifying` |
| `BURN_DONE` | Disco › Etiquetado + `burn.done.title` / `burn.done.body`; ação `action.done` |
| `BURN_FAILED` | Disco › Rejeitado (Virgem ou Ilegível) + `burn.failed.<motivo>`; lista: `action.erase_retry` (CD-RW) ou `action.try_other` |
| `REG_REJECTED` | Disco › Rejeitado + ficha + `reg.reject.<classe>`; `action.try_other` |

### 6.7 Gestão e problemas

| Estado | Tela |
|---|---|
| `GAME_OPTIONS` | Caixa puxada à esquerda; lista vertical à direita (`tipo-secao`): `options.edit`, `options.discs`, `options.burn_another`, `options.remove` |
| Discos do jogo | Lista: rótulo, data, origem (`disc.origin.burned`/`adopted`); ação por item `options.unlink_disc` |
| `REMOVE_CONFIRM` | Diálogo destrutivo `remove.*` |
| `DRIVE_PROBLEM` | Estante sob véu; `drive.<motivo>.title`/`.body`; ações `action.choose_drive`, `action.back` |
| `CATALOG_ERROR` | Tela sem estante; `catalog.error.title`/`.body`; lista de backups (data e hora) + `catalog.start_empty` |

Formulários (editar jogo, novo jogo personalizado, chave de API): campos em `tipo-corpo` sobre folha `etiqueta`, um por linha, rótulo acima em `tipo-apoio` `grafite`. Entrada por teclado e mouse na fase 1 ([Q18](OPEN-QUESTIONS.md#q18-entrada-de-texto-com-controle)); o controle navega entre campos e botões.

### 6.8 Configurações

Lista vertical à esquerda (`tipo-corpo`), valor atual à direita na mesma linha. Itens: `settings.drive`, `settings.on_insert` (`settings.on_insert.focus` / `.launch`), `settings.loading_min`, `settings.language`, `settings.fullscreen`, `settings.online_covers` (e chave), `settings.export`, `settings.import`. Mudanças se aplicam na hora, sem botão Salvar.

## 7. Entrada e foco

| Controle | Teclado | Mouse | Função |
|---|---|---|---|
| Direcional / analógico | Setas | Roda (estante: ←→) | Mover foco; repete com `dur-repeticao-*` |
| A / ✕ | Enter ou Espaço | Clique | Ação primária / ação em foco |
| B / ○ | Esc ou Backspace | Botão "voltar" do mouse; dica clicável | Voltar (coluna B em [UX-STATES](UX-STATES.md)) |
| X / □ | N | Dica clicável | Adicionar jogo (`LIBRARY`) |
| Y / △ | O | Clique direito na caixa | Opções do jogo |
| LB / RB | Page Up / Page Down | — | Letra inicial anterior/próxima |
| Menu / Start | M | Dica clicável | Configurações |

Regras:

- **Exatamente um elemento em foco**, sempre visível.
- Mouse: pousar sobre um item por `dur-hover` move o foco para ele (e a caixa é puxada como no controle). Passar rapidamente por cima não anima nada.
- Voltar de uma tela restaura o foco anterior; `focus_hint` do núcleo é aplicado uma vez ([UI-CONTRACT](UI-CONTRACT.md#núcleo--ui-instantâneo-de-estado)).
- Sem foco de janela, nenhuma entrada de controle é processada ([SECURITY R10](SECURITY.md#r10-entrada-só-com-foco)).

## 8. Janela

- Tela cheia e janela ([ADR-0009](adr/0009-interface-tauri-console-gamepad.md)); mesma composição; mínimo 1280×720. Padrão: [Q20](OPEN-QUESTIONS.md#q20-modo-de-janela-padrão).
- Proporções diferentes de 16:9: a estante estica na horizontal; a altura manda na escala.
- Sem barra de título customizada na fase 1.

## 9. Textos (pt-BR / en)

**Fonte única de todos os textos visíveis.** Voz: direta, sentence case, sem desculpas, dizendo o que aconteceu e o que fazer. Uma ação tem o mesmo nome no fluxo inteiro. `{game}` = X, `{other}` = Y, `{name}` = `name` do INI. Plurais pelo mecanismo do i18n ([Q24](OPEN-QUESTIONS.md#q24-idioma-e-plural)).

### Ações

| Chave | pt-BR | en |
|---|---|---|
| `action.play` | Jogar | Play |
| `action.play_other` | Jogar {other} | Play {other} |
| `action.back` | Voltar | Back |
| `action.back_to_shelf` | Voltar à estante | Back to shelf |
| `action.select` | Selecionar | Select |
| `action.add_game` | Adicionar jogo | Add game |
| `action.options` | Opções | Options |
| `action.settings` | Configurações | Settings |
| `action.try_other` | Tentar outro disco | Try another disc |
| `action.adopt` | Adotar este disco | Adopt this disc |
| `action.burn` | Gravar disco | Burn disc |
| `action.erase` | Apagar | Erase |
| `action.erase_retry` | Apagar e tentar de novo | Erase and try again |
| `action.retry_tray` | Abrir a gaveta | Open the tray |
| `action.continue` | Continuar | Continue |
| `action.done` | Concluir | Done |
| `action.choose_drive` | Escolher drive | Choose drive |

### Biblioteca e avisos

| Chave | pt-BR | en |
|---|---|---|
| `library.empty.title` | Sua estante está vazia | Your shelf is empty |
| `library.empty.body` | Adicione um jogo e grave o disco dele. | Add a game and burn its disc. |
| `library.kind.steam` | Steam | Steam |
| `library.kind.custom` | Personalizado | Custom |
| `library.disc_count` | {n} disco / {n} discos | {n} disc / {n} discs |
| `toast.focus` | {other} está na gaveta. | {other} is in the tray. |
| `toast.unknown` | Disco desconhecido na gaveta. Selecione um jogo para associá-lo. | Unknown disc in the tray. Select a game to link it. |
| `toast.not_a_game` | O disco na gaveta não é um disco de jogo. | The disc in the tray isn't a game disc. |
| `toast.read_error` | Não deu para ler o disco na gaveta. | Couldn't read the disc in the tray. |
| `toast.drive_removed` | O drive foi desconectado. | The drive was disconnected. |

### Fluxo do disco

| Chave | pt-BR | en |
|---|---|---|
| `nodisc.title` | {game} ainda não tem disco | {game} doesn't have a disc yet |
| `nodisc.body` | Grave um disco para jogar por aqui. | Burn a disc to play it from here. |
| `wait.title` | Coloque o disco de {game} na gaveta | Put the {game} disc in the tray |
| `wait.tray_opening` | A gaveta está abrindo. | The tray is opening. |
| `wait.tray_manual` | Abra a gaveta do drive e feche depois de colocar o disco. | Open the drive tray, then close it after inserting the disc. |
| `wait.tray_failed` | A gaveta não abriu. Abra pelo botão do drive. | The tray didn't open. Use the button on the drive. |
| `reading.title` | Lendo o disco | Reading the disc |
| `reject.OTHER_GAME` | Este disco é de {other}. | This disc is for {other}. |
| `reject.UNKNOWN` | Este disco não está na sua estante. | This disc isn't on your shelf. |
| `reject.LEGACY_GAME_INI` | Este disco é do Reset Floppy Game System. Grave um disco novo para {game}. | This disc is from Reset Floppy Game System. Burn a new disc for {game}. |
| `reject.NO_GAME_INI` | Este CD não é um disco de jogo. | This CD isn't a game disc. |
| `reject.INVALID_GAME_INI` | O identificador deste disco está danificado. | This disc's ID is damaged. |
| `reject.BLANK` | Este disco está vazio. | This disc is blank. |
| `reject.AUDIO` | Este é um CD de música. | This is a music CD. |
| `reject.READ_ERROR` | Não deu para ler o disco. Limpe a face e tente de novo. | Couldn't read the disc. Wipe it and try again. |
| `adopt.title` | Associar este disco a {game}? | Link this disc to {game}? |
| `adopt.body` | O disco diz ser {name}. Depois disso, ele abre {game}. | The disc says it's {name}. After this, it opens {game}. |
| `adopt.confirm` | Associar disco | Link disc |
| `launch.title` | Abrindo {game} | Opening {game} |
| `launch.error.not_found` | O programa de {game} não foi encontrado. Confira o caminho nas opções do jogo. | {game}'s program wasn't found. Check the path in the game's options. |
| `launch.error.steam_missing` | A Steam não foi encontrada neste PC. | Steam wasn't found on this PC. |
| `launch.error.elevation_denied` | {game} precisa de permissão de administrador, e ela foi negada. | {game} needs administrator permission, and it was denied. |
| `launch.error.generic` | {game} não abriu. | {game} didn't open. |

### Ficha

| Chave | pt-BR | en |
|---|---|---|
| `sheet.label` | Rótulo | Label |
| `sheet.media` | Mídia | Media |
| `sheet.id` | Identificador | ID |
| `sheet.ini_name` | Nome no disco | Name on disc |
| `sheet.belongs_to` | Pertence a | Belongs to |
| `common.not_informed` | não informado | not available |

### Cadastro e gravação

| Chave | pt-BR | en |
|---|---|---|
| `reg.insert` | Coloque um CD-R ou CD-RW virgem | Insert a blank CD-R or CD-RW |
| `reg.case_label` | Novo disco | New disc |
| `reg.reading` | Verificando o disco | Checking the disc |
| `reg.new_steam` | Novo jogo da Steam | New Steam game |
| `reg.new_custom` | Novo jogo personalizado | New custom game |
| `reg.label.title` | Rótulo do disco | Disc label |
| `reg.label.body` | É o nome que aparece no Windows. Não muda qual jogo o disco abre. | It's the name Windows shows. It doesn't change which game the disc opens. |
| `reg.cdr_warning.title` | CD-R só pode ser gravado uma vez | A CD-R can only be burned once |
| `reg.cdr_warning.body` | Se a gravação falhar, este disco não poderá ser usado. | If burning fails, this disc can't be used. |
| `reg.reject.CDR_USED` | Este CD-R já foi gravado. Use um disco virgem. | This CD-R has already been burned. Use a blank disc. |
| `reg.reject.NOT_WRITABLE` | Este disco não pode ser gravado. | This disc can't be burned. |
| `reg.reject.AUDIO` | Este é um CD de música. Use um disco virgem. | This is a music CD. Use a blank disc. |
| `reg.reject.READ_ERROR` | Não deu para ler o disco. | Couldn't read the disc. |
| `erase.title` | Apagar este CD-RW? | Erase this CD-RW? |
| `erase.body` | Tudo o que está nele será apagado. | Everything on it will be erased. |
| `erase.hold` | Segure para apagar | Hold to erase |
| `erase.progress` | Apagando. Não remova o disco. | Erasing. Don't remove the disc. |
| `burn.progress` | Gravando. Não remova o disco. | Burning. Don't remove the disc. |
| `burn.verifying` | Verificando a gravação | Verifying the burn |
| `burn.done.title` | Disco gravado | Disc burned |
| `burn.done.body` | Agora imprima e cole a etiqueta. | Now print and stick on the label. |
| `burn.failed.write_error` | A gravação falhou. | Burning failed. |
| `burn.failed.verify_mismatch` | O disco gravado não confere com o original. | The burned disc doesn't match. |
| `burn.failed.drive_removed` | O drive foi desconectado durante a gravação. | The drive was disconnected while burning. |
| `burn.failed.cdr_lost` | Este CD-R não poderá ser usado. | This CD-R can't be used. |

### Gestão, drive e catálogo

| Chave | pt-BR | en |
|---|---|---|
| `options.edit` | Editar jogo | Edit game |
| `options.discs` | Discos | Discs |
| `options.burn_another` | Gravar outro disco | Burn another disc |
| `options.remove` | Remover jogo | Remove game |
| `options.unlink_disc` | Desassociar disco | Unlink disc |
| `disc.origin.burned` | Gravado aqui | Burned here |
| `disc.origin.adopted` | Adotado | Adopted |
| `remove.title` | Remover {game} da estante? | Remove {game} from the shelf? |
| `remove.body` | Os discos dele passam a ser desconhecidos. Dá para adotá-los de novo depois. | Its discs become unknown. You can adopt them again later. |
| `remove.confirm` | Remover jogo | Remove game |
| `drive.none.title` | Nenhum drive encontrado | No drive found |
| `drive.none.body` | Conecte o drive de CD e escolha-o nas configurações. | Connect the CD drive and choose it in settings. |
| `drive.removed.title` | O drive foi desconectado | The drive was disconnected |
| `drive.removed.body` | Conecte o drive de novo para continuar. | Reconnect the drive to continue. |
| `drive.cannot_burn.title` | Este drive não grava CDs | This drive can't burn CDs |
| `drive.cannot_burn.body` | Use um drive gravador para cadastrar jogos. | Use a burner drive to add games. |
| `catalog.error.title` | Não deu para abrir sua estante | Couldn't open your shelf |
| `catalog.error.body` | O arquivo foi preservado. Restaure um backup ou comece uma estante vazia. | The file was kept. Restore a backup or start an empty shelf. |
| `catalog.start_empty` | Começar vazia | Start empty |

### Configurações

| Chave | pt-BR | en |
|---|---|---|
| `settings.drive` | Drive | Drive |
| `settings.on_insert` | Ao colocar um disco | When a disc is inserted |
| `settings.on_insert.focus` | Mostrar o jogo | Show the game |
| `settings.on_insert.launch` | Abrir o jogo | Open the game |
| `settings.loading_min` | Duração mínima da abertura | Minimum opening time |
| `settings.language` | Idioma | Language |
| `settings.fullscreen` | Tela cheia | Fullscreen |
| `settings.online_covers` | Capas da internet | Covers from the internet |
| `settings.export` | Exportar estante | Export shelf |
| `settings.import` | Importar estante | Import shelf |

## 10. Acessibilidade e qualidade mínima

- Contraste conforme §3.1, incluindo foco e bordas (WCAG 1.4.11, 3:1).
- Nenhuma informação só por cor: rejeição = `laser` **e** texto **e** variante do disco.
- `prefers-reduced-motion` respeitado (§3.4).
- Tudo alcançável por controle e por teclado; mouse opcional.
- Textos traduzidos podem crescer até 40%: nenhum componente de texto com largura fixa.
- Nome do jogo, rótulo e `name` do INI renderizados sempre como **texto** (nunca HTML) ([SECURITY R1](SECURITY.md#r1-o-disco-só-fornece-um-identificador)).

## 11. Não fazer

| Evitar | Por quê |
|---|---|
| Grade de cards arredondados com sombra cinza | "Kit SaaS"; a biblioteca é uma estante |
| Rótulos em caixa alta acima de títulos | Padrão genérico |
| Fonte monoespaçada para dados | Padrão genérico; use `tipo-dado` |
| Separadores "A · B · C" | Use linhas separadas |
| Gradientes decorativos | A única "luz" é a iridescência do disco |
| Animação de entrada em cada item; hover animado | Só as animações autônomas da §3.4 |
| Barra de progresso no loading | Não há progresso real |
| Foco em outra cor que não `ftalocianina` (ou a borda dupla em folhas) | Um só sinal de foco |
| Ícones genéricos (Material Icons etc.) | Glifos próprios: disco, caixa, botões |
| Ações dentro de avisos | Avisos somem e não consomem entrada (§5.6) |

## 12. Revisão do plano

| Primeira ideia | Problema | Revisado para |
|---|---|---|
| Fundo quase preto com um acento vivo | Padrão genérico | `azo`, azul saturado do corante, com três corantes em papéis distintos |
| Grade de capas 2:3 | Padrão de launcher | Estante de lombadas; capa só na caixa puxada |
| Barra de progresso no loading | Não há progresso | Disco girando |
| Monoespaçada para o UUID | Padrão genérico | `tipo-dado` com `tnum` |
| Ficha numerada como faixas de CD | Não é sequência | Lista de definição |
| Duas famílias | Contraste por família é o padrão | Uma família; contraste por largura |
| "Steam · 2 discos" | Separador genérico | Duas linhas |
| Foco dourado em todo lugar | Falha de contraste sobre `etiqueta` (revisão independente) | Borda dupla `azo` + `ftalocianina` em folhas claras |
