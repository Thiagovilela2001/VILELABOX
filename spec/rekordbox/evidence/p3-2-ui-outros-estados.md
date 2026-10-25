# G0 — Medição da UI em execução: modo EDIT e quatro decks

Complemento de [`p3-1-ui-medido.md`](p3-1-ui-medido.md), que mediu o estado
PERFORMANCE + 2Deck Horizontal. Aqui ficam os outros dois estados que faltavam para
implementar o seletor de modo e o seletor de disposição.

## Método

Mesmo medidor: `tools/ui/medir_ui.ps1` (Win32 + UI Automation + captura) e
`tools/ui/analisar_layout.py`. A troca de estado foi feita por UI Automation, sem
clique manual: o seletor de modo e o de disposição são `ComboBox` que expõem
`ExpandCollapsePattern`, e as opções aparecem como `MenuItem` na árvore do desktop.

Estado de partida e de chegada: PERFORMANCE + 2Deck Horizontal. Ao devolver o modo, o
próprio programa carregou a faixa seleccionada no deck 1; nada mais mudou.

## Opções que os seletores realmente oferecem (7.2.8)

| seletor | opções medidas |
| --- | --- |
| modo | `EXPORT`, `PERFORMANCE`, `EDIT` |
| disposição | `2Deck Horizontal`, `2Deck Vertical`, `4Deck Horizontal`, `4Deck Vertical`, `Browser` |

O menu de disposição é mais alto que a lista de modos: as quatro disposições ficam em
`y 89, 120, 151, 182` (31 px cada) e a entrada `Browser` vem depois.

## Estado A — modo EDIT (mesma tela 1920×1080, mesma janela)

O EDIT não é o performance com menos painéis: é outro espaço de trabalho.

| região | posição medida |
| --- | --- |
| barra de menus | só quatro itens: Ficheiro, Editor, Ver, Ajuda |
| seletor de modo | `ComboBox EDIT` em 12,35 138×20 |
| barra do projecto | `ComboBox New Project` 25,71 500×28; `Loop` 1045,76 29×18; `AUTO` 1162,73 125×22; tempo em 1288,115 |
| área de edição | faixa preta de 64..~640 de altura, largura 0..~1540 |
| PALETTE (painel direito) | botão `PALETTE` em 1548,111 182×38; painel de ~1548 a 1920 |
| abas de ferramentas | `BEAT SELECT` 4,721 376×38 · `SELECT` 380,721 376×38 · `EDIT` 759,721 378×38 |
| BROWSER | recolhido a uma barra: botão em 2,985 756×38 |
| barra de status | texto em 1421,1026 462×20: "Seleccionadas: 102 faixas, 4 horas…" |

Conteúdo das abas, visível na captura: em BEAT SELECT há botões de 1, 2, 4, 8, 16, 32,
64 e 128 beats; em EDIT há CLONE, COPY, PASTE, INSERT, DELETE, CLEAR, UNDO e REDO.

A árvore de acessibilidade do EDIT é muito menor que a do performance: 74 nós contra
484, e só 20 com nome. O modo usa superfícies desenhadas, não componentes nomeados.

Tema medido no EDIT: fundos em torno de `#1b1a19` e painéis em `#2e2d2c`, mais claros e
neutros que o preto do performance (`#000000`, `#0f0f0f`, `#1e1e1e`).

## Estado B — PERFORMANCE + 4Deck Horizontal

Quatro decks em duas colunas por duas linhas, com a coluna de mixer no meio.

| bloco | posição medida (px físicos, origem no cliente) |
| --- | --- |
| deck 1 (esquerda, topo) | número em 10,320 |
| deck 2 (direita, topo) | número em 1092,320 |
| deck 3 (esquerda, base) | número em 10,559 |
| deck 4 (direita, base) | número em 1092,559 |
| coluna de mixer | quatro canais em x 858, 912, 968 e 1022 (45 px de largura, passo de 54 px) |
| TRIM / HIGH / MID / LOW | y 372, 422, 472 e 522 nos quatro canais |
| CUE | y 545, 38×25 por canal |
| KEY SYNC / BEAT SYNC por deck | BEAT SYNC em 740,312 e 1822,312 (linha de cima) e 740,551 / 1822,551 (linha de baixo) |
| barra de mixagem inferior | rótulos MIX em 646,807 e LEVEL em 726,807 |

Daí sai a proporção: coluna de mixer ≈ 850..1080 (230 px, 12,0% da largura), decks de
~845 px cada lado. A faixa de formas de onda cresce em altura em relação ao 2 decks,
porque cada deck precisa da sua: a banda de decks começa por volta de y 300.

## Classificação das conclusões

- **OBSERVADO:**
  - As opções dos dois seletores (`EXPORT`/`PERFORMANCE`/`EDIT` e as quatro disposições).
  - As posições da tabela acima, com DPI 120 e cliente 1920×1051.
  - A árvore de acessibilidade do EDIT com 74 nós e 20 nomes.
  - Fundos do EDIT mais claros (`#1b1a19`, `#2e2d2c`) que no performance.
  - Ao voltar de EDIT para PERFORMANCE, o programa carrega no deck 1 a faixa que estava
    seleccionada no navegador.
- **INFERIDO:**
  - A coluna de mixer de 4 canais corresponde a um canal por deck, pela ordem dos
    quatro controles da esquerda para a direita.
  - Os 230 px da coluna de mixer são largura de painel, e não margem: os controles das
    pontas ficam a 12 px das bordas.
- **DESCONHECIDO:**
  - Geometria das disposições verticais (2Deck Vertical, 4Deck Vertical).
  - Geometria do modo EXPORT: não foi medido nesta passagem.
  - O que a disposição "Browser" faz com a área de reprodução.
  - Comportamento com painéis auxiliares abertos em qualquer um dos estados.

## Rastreabilidade

- Medições cruas: `third_party/rekordbox/ui-workbench/medicoes-edit/` e
  `medicoes-4deck/` (privadas, com `captura.png`, `uia.tsv`, `janela.tsv`, `meta.txt`)
- Especificação: [`spec/rekordbox/40-telas.md`](../40-telas.md)
- Medição do estado principal: [`p3-1-ui-medido.md`](p3-1-ui-medido.md)
