# p6-4 — Onde o MEMORY grava: banco `djmdCue`, não ANLZ

Dossiê de proveniência. Fecha a pergunta que o `p6-3` deixou como INFERIDO ("grava fora do
motor, na camada de coleção/banco") e mostra que o arquivo de análise não recebe o cue.

## Artefato e método

| Campo | Valor |
| --- | --- |
| Banco | `%APPDATA%\Pioneer\rekordbox\master.db` (19.730.432 bytes) + `-wal` |
| Cópia usada | cópia em `dbcopy/`, sempre somente leitura (`pyrekordbox` 0.5.0, `MasterDatabase`) |
| ANLZ da faixa | `share\PIONEER\USBANLZ\5f4\8d012-2459-4ea8-a587-8306c247bf49\` |
| Faixa | Flim — Aphex Twin, `ContentID` 162734337, 147,98 BPM |

A pasta ANLZ tem mtime **14:43** (criação, quando a faixa foi carregada) e nenhum arquivo
mudou depois. `ANLZ0000.DAT` e `ANLZ0000.EXT` têm dois `PCOB` cada; lidos no cabeçalho, os
quatro estão **vazios**: `len_tag = 24`, `len_entry = 0` ou `1`, `cues = 0`. O arquivo de
análise não guarda cue nenhum.

## O que o banco tem

`djmdCue`, faixa Flim, em ordem de `updated_at` (UTC; hora local = UTC−3):

| updated_at (UTC) | Kind | InMsec | OutMsec | Leitura |
| --- | --- | --- | --- | --- |
| 17:50:08 | **0** | 141081 | 147568 | o aperto de MEMORY dentro da janela do `p6-3` (14:50:08 local) |
| 17:49:47 | 9 | 136215 | −1 | tipo não identificado, ainda |
| 17:48:17 | **0** | 19445 | −1 | memory cue simples |
| 17:47:07 | **0** | 19445 | 25932 | memory cue com ponto de saída |
| 17:46:53 | **1** | 8093 | −1 | hot cue |
| 17:43:53 | **0** | 0 | −1 | memory cue no início |

Colunas relevantes de `djmdCue`: `ContentID`, `InMsec`, `OutMsec`, `InFrame`,
`InMpegFrame`, `Kind`, `Color`, `ActiveLoop`, `Comment`, `BeatLoopSize`, `CueMicrosec`,
`UUID`, `rb_local_usn`, `updated_at`.

## Conclusão

- **OBSERVADO**: o `MEMORY` grava uma linha em `djmdCue` do `master.db`, com `Kind = 0` e a
  posição em `InMsec`. `OutMsec = −1` quando é cue simples; quando há ponto de saída, ele
  aparece em `OutMsec` (19445 → 25932).
- **OBSERVADO**: hot cue usa a mesma tabela, `Kind = 1`.
- **OBSERVADO**: o ANLZ da faixa não é tocado (`PCOB` vazio, mtime de criação). No 7.2.8 o
  banco é a fonte do cue; o ANLZ só recebe cue na exportação para mídia.
- **DESCONHECIDO**: `Kind = 9` — apareceu uma vez, posição 136215 ms, e não foi identificado.
- **DESCONHECIDO**: a função interna que escreve (deixou de importar para o produto: o
  destino e o formato já são conhecidos).

## O que isso muda no VILELABOX

Persistir cue deixa de ser "entender o ANLZ" e passa a ser "escrever no banco da coleção";
o leitor de ANLZ do repositório não serve para isso. O caminho de ida e volta a implementar
é: gravar a linha, recarregar a faixa, o cue continuar na lista.

## Nota de segurança

O banco foi aberto **apenas na cópia**, em modo somente leitura. Abrir o original com escrita
pode corromper a biblioteca do rekordbox (o processo estava rodando durante a leitura).
