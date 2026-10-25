# G0 — Medição da UI em execução (interface, medição direta)

Primeira medição feita sobre o rekordbox em execução nesta máquina, e não sobre
documentação. Fecha a lacuna declarada em
[`40-telas.md`](../40-telas.md), que até aqui dizia "nada foi medido em tela própria".

Este dossiê registra evidência dinâmica e não substitui a decompilação no Ghidra.
Hierarquia interna, transições de estado e regras que não podem ser demonstradas
pelas medições abaixo permanecem bloqueadas no Gate G0 até receberem evidência
estática correspondente.

## Artefato e ambiente

| Campo | Valor |
| --- | --- |
| Programa | `rekordbox.exe` 7.2.8.0, PID 13724 |
| Janela | `hwnd 0x15041A`, classe `JUCE_1a16e9b2ba5`, maximizada (`-9,-9` a `1929,1089`) |
| Tela | 1920×1080, DPI 120 (`escala = 1,25`), barra de tarefas auto-oculta |
| Área de cliente | 1920×1051 px físicos = **1536×841 px lógicos** |
| Estado medido | modo **PERFORMANCE**, disposição **2Deck Horizontal**, navegador embaixo, nenhum painel auxiliar aberto, nenhuma faixa carregada, quatro slots "blank list" |
| Captura | `PrintWindow` falha aqui: a área de reprodução é composta na GPU e sai preta com a janela ocluída. A medição usa foco breve (`-TrazerParaFrente`), devolvido à janela anterior |

Todas as medidas em **pixels físicos** da área de cliente; a coluna "px lógicos"
divide por 1,25, que é a unidade que o Slint desenha.

## Como reproduzir

```bash
powershell -NoProfile -ExecutionPolicy Bypass -File tools/ui/medir_ui.ps1 -TrazerParaFrente
python tools/ui/analisar_layout.py third_party/rekordbox/ui-workbench/medicoes \
    --saida spec/rekordbox/evidence/layout-rekordbox-7.2.8.json
```

`medir_ui.ps1` grava em `third_party/rekordbox/ui-workbench/medicoes/` (área ignorada
pelo Git, por conter captura de tela): `meta.txt`, `janela.tsv` (janelas Win32),
`uia.tsv` (árvore completa de UI Automation, 484 nós), `captura.png`, `cliente.raw`.
`analisar_layout.py` só usa biblioteca padrão e deriva o JSON versionável com faixas,
painéis, colunas e marcos semânticos.

## Regiões da janela (cliente 1920×1051)

| região | y (px físicos) | altura | % da altura | px lógicos |
| --- | --- | --- | --- | --- |
| barra de menus (Ficheiro/Ver/Faixa/Lista de reprodução/Ajuda) | 0..27 | 28 | 2,66% | 22,4 |
| barra global (modo, disposição, painéis, LINK, MASTER, FREE, CPU, relógio) | 28..63 | 36 | 3,43% | 28,8 |
| reprodução | 64..580 | 517 | 49,19% | 413,6 |
| ↳ faixa de formas de onda | 64..239 | 176 | 16,75% | 140,8 |
| ↳ banda de decks e mixer | 240..580 | 341 | 32,44% | 272,8 |
| navegador | 581..1019 | 439 | 41,77% | 351,2 |
| barra de status | 1016..1050 | 35 | 3,33% | 28,0 |

As fronteiras de região coincidem com os contêineres que o próprio programa expõe:
`Pane` em `0,28 1920×554` (reprodução), `Custom` em `0,581 1920×438` (navegador) e
`Custom` em `0,1022 1920×29` (status).

## Banda de decks e mixer (y 240..580)

Três painéis, separados por faixas de 3 px sem fundo (`x 890..892` e `x 1027..1029`):

| painel | x | largura | % da largura | px lógicos |
| --- | --- | --- | --- | --- |
| deck 1 | 0..889 | 890 | 46,35% | 712,0 |
| faixa de mixer (dois canais) | 893..1026 | 134 | 6,98% | 107,2 |
| deck 2 | 1030..1919 | 890 | 46,35% | 712,0 |

O funil de mixer mede 134 px — praticamente igual aos 133 px já registrados no
workbench (`third_party/rekordbox/ui-workbench/ui-tools/aplicar_escala.py`). Já os
decks medem **890 px cada**, e não os 853/887 px usados lá: aqueles valores excluíam
um trilho de 36 px que **não existe** nesta disposição. Não foi detectada nenhuma
fronteira vertical dentro da reprodução além das duas acima.

Estrutura interna do deck 1 (deck 2 é espelho, deslocado de +1030 px):

| bloco | posição medida |
| --- | --- |
| número do deck | `Text '1'` em 10,250 15×15 |
| identificação da faixa | `Text 'Not Loaded.'` em 80,252 578×22 |
| KEY SYNC / BEAT SYNC / MASTER | 688,255 88×16 / 792,250 55×28 / 792,282 88×16 |
| contador de tempo | textos em x 511..653, y 276..296 |
| formas de onda | faixa 176 px de altura, duas linhas de 86 px (63..149 e 150..235) |
| pads (A..P, quatro colunas) | letras em x 58, 292, 528, 762; linhas em y 379, 405, 431, 457 |
| HOT CUE | 48,486 85×25 |
| transporte (Cue / Reproduzir-Pause) | 630,409 45×45 / 630,466 45×45 |
| tempo e BPM | `00.00` em 725,419 68×29 |

Faixa de mixer, dois canais de 45 px de largura útil: TRIM (905,275 e 970,275),
HIGH (y 316), MID (y 357), LOW (y 399), CUE (y 423, 45×22).

## Navegador (y 581..1019)

| bloco | x | largura | px lógicos |
| --- | --- | --- | --- |
| trilho de ícones | 0..46 | 46 | 36,8 |
| árvore de fontes ("Explorador") | 48..452 | 405 | 324,0 |
| tabela de faixas | 459..1919 | 1461 | 1168,8 |

Faixas internas: linha de slots `581..613`, separador `614..620`, cabeçalho de
colunas `621..653`, linhas de dados de **33–34 px** alternando fundo `#1e1e1e` e
`#0f0f0f` (`656..991` = dez linhas visíveis), e barra de rolagem horizontal em
`1006..1015`.

Colunas medidas (cabeçalho real, não estimativa):

| coluna | x | largura |
| --- | --- | --- |
| Atributo | 459 | 38 |
| Pré-visualização | 534 | 250 |
| Ilustração | 784 | 100 |
| Título da Faixa | 884 | 160 |
| Intérprete | 1044 | 160 |
| Álbum | 1204 | 160 |
| Género | 1364 | 160 |
| BPM | 1524 | 100 |
| Classificação | 1624 | 112 |
| Tempo | 1736 | 100 |
| Tonalidade | 1836 | 160 (cortada em 1920) |

## Barra de status (y 1016..1050)

Conteúdo exposto: três `Text` em y 1026 (x 162 largura 225, x 162 largura 1670,
x 1421 largura 412) e o botão `btnMultiWindow` em 1885,1026 20×20.

## Nomes de controle expostos pelo programa

A árvore de UI Automation entrega 484 nós, 171 com nome, e nomes de componente úteis
para o port — por exemplo `PERFORMANCE` e `2Deck Horizontal` (ComboBox),
`AudioCpuGraphButton`, `ShowHideTreeShortcutButton`, `ShowHidePaletteButton`,
`ShowHideMyTagButton`, `ShowHideTrackInfoButton`, `WiFiSyncDetectButton`,
`SelectFullArtworkButton`, `prevCueSnapReg`. A lista completa, com retângulo de cada
nó, está no JSON ao lado e no `uia.tsv` da medição.

## Classificação das conclusões

- **OBSERVADO:**
  - Regiões, alturas e larguras da tabela acima, com DPI 120 e cliente 1920×1051.
  - Decks de 890 px e faixa de mixer de 134 px, sem trilho de 36 px na reprodução.
  - Navegador 405 px de árvore e 1461 px de tabela; colunas com as larguras da tabela.
  - Linhas de tabela de 33–34 px e cabeçalho de 33 px.
  - `PrintWindow` devolve a área de reprodução em preto quando a janela está ocluída.
- **INFERIDO:**
  - A divisão da faixa de formas de onda em duas linhas de 86 px vem dos contêineres
    de acessibilidade: sem faixa carregada, a área é preta uniforme e os pixels não
    confirmam a divisão.
  - Os 3 px entre painéis são separadores sem fundo, e não margem de conteúdo.
- **DESCONHECIDO:**
  - Geometria nos modos de exportação, quatro decks, vertical, e com painéis
    auxiliares abertos (FX, sampler, gravação) — medidos apenas no estado acima.
  - Geometria com faixa carregada (waveform, marcadores, medidores em movimento).
  - Outras escalas de interface (o programa aceita fator de 0,80 a 1,35).
  - Famílias tipográficas e paleta completa de cores: só foram amostradas as cores de
    fundo dominantes (`#000000`, `#0f0f0f`, `#1e1e1e`, `#323232`).

## Rastreabilidade

- Medidor: [`tools/ui/medir_ui.ps1`](../../../tools/ui/medir_ui.ps1)
- Analisador: [`tools/ui/analisar_layout.py`](../../../tools/ui/analisar_layout.py)
- Dados versionáveis: [`layout-rekordbox-7.2.8.json`](layout-rekordbox-7.2.8.json)
- Captura e árvore cruas: `third_party/rekordbox/ui-workbench/medicoes/` (privado)
- Especificação: [`spec/rekordbox/40-telas.md`](../40-telas.md)
