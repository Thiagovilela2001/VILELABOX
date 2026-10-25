# 11 — ANLZ (`.DAT` e `.EXT`)

Especificação limpa do formato binário de análise musical (`ANLZ0000.DAT` e
`ANLZ0000.EXT`), do fluxo de descoberta e exportação para o esquema canônico e
das variantes de cues (`PCOB`/`PCPT` e `PCO2`/`PCP2`).

Proveniência G0: [`evidence/p1-anlz.md`](evidence/p1-anlz.md).

## Escopo e fora do escopo

**Escopo.** Contêiner `PMAI`, seção de caminho `PPTH`, grade de batidas `PQTZ`,
estrutura de frases musicais `PSSI` (com e sem máscara XOR), listas de cues e
loops `PCOB`/`PCPT` e `PCO2`/`PCP2` (incluindo comentários UTF-16BE e cor RGB),
regras de precedência entre `.DAT` e `.EXT`, e contrato do exportador em lote.

**Fora do escopo.** Renderização gráfica de formas de onda (`PWAV`, `PWV2`,
`PWV3`, `PWV4`, `PWV5`), tratada em `24-waveform`, e geração de exportação USB,
tratada em `13-export-usb`.

## Descoberta e associação `.DAT` / `.EXT`

Cada faixa analisada produz um par de arquivos dentro da árvore `PIONEER/USBANLZ`:

- `ANLZ0000.DAT` (ou `ANLZXXXX.DAT` numerado em hexadecimal): contêiner base com
  `PMAI`, `PPTH`, `PQTZ` e bancos `PCOB`;
- `ANLZ0000.EXT`: contêiner irmão no mesmo diretório com `PMAI`, `PPTH`, `PCOB`,
  `PCO2`, `PSSI` e seções estendidas.

Regras de associação:

1. A varredura recursiva localiza arquivos `.DAT` sem diferenciar maiúsculas e
   minúsculas e ordena os caminhos relativos de forma determinística;
2. Para cada `.DAT`, o arquivo `.EXT` irmão é resolvido trocando a extensão no
   mesmo diretório;
3. O caminho do arquivo de áudio (`PPTH`) é lido preferencialmente de `.EXT`
   quando presente e não vazio, caindo para `.DAT`.

## Contêiner `PMAI` e seções

Todo arquivo `.DAT` ou `.EXT` inicia com o cabeçalho `PMAI` em big-endian:

| Offset | Tamanho | Tipo | Descrição |
| --- | --- | --- | --- |
| `0x00` | 4 | ASCII | Assinatura `"PMAI"` |
| `0x04` | 4 | `u32` BE | `len_header`: tamanho do cabeçalho `PMAI` (tipicamente `28` bytes) |
| `0x08` | 4 | `u32` BE | `len_file`: tamanho total do arquivo em bytes |

As seções começam em `len_header` e repetem o envelope de 12 bytes:

| Offset | Tamanho | Tipo | Descrição |
| --- | --- | --- | --- |
| `0x00` | 4 | ASCII | Identificador FourCC da seção (`PPTH`, `PQTZ`, `PCOB`, `PCO2`, `PSSI`, etc.) |
| `0x04` | 4 | `u32` BE | `len_header`: tamanho total do cabeçalho da seção |
| `0x08` | 4 | `u32` BE | `len_tag`: tamanho total da seção (mínimo `12` bytes) |
| `0x0c` | `len_tag - 12` | bytes | Payload da seção |

Regras de segurança de leitura:

- Se `len_tag < 12` ou `pos + len_tag > data.len()`, a varredura de seções é
  encerrada sem pânico;
- Seções com FourCC desconhecido são ignoradas preservando o avanço por
  `len_tag`;
- Como um mesmo arquivo pode conter mais de uma seção com o mesmo FourCC (por
  exemplo, dois bancos `PCOB` ou dois bancos `PCO2`), o leitor preserva tanto a
  lista ordenada (`all_sections`) quanto o mapa da última ocorrência (`sections`).

## Seção `PPTH` (caminho do áudio)

No payload de `PPTH` (após os 12 bytes iniciais da seção):

| Offset no payload | Tamanho | Tipo | Descrição |
| --- | --- | --- | --- |
| `0x00` | 4 | `u32` BE | `len_path`: tamanho em bytes da string UTF-16BE (incluindo terminador nulo) |
| `0x04` | `len_path` | UTF-16BE | Caminho do arquivo de áudio terminado por `0x0000` |

Se `payload.len() < 4 + len_path`, se `len_path - 2` for ímpar ou se a sequência
UTF-16BE for inválida, o leitor devolve `None`.

## Seção `PQTZ` (grade de batidas)

No payload de `PQTZ`:

| Offset no payload | Tamanho | Tipo | Descrição |
| --- | --- | --- | --- |
| `0x00` | 8 | bytes | Reservado / metadados do cabeçalho |
| `0x08` | 4 | `u32` BE | `num_beats`: quantidade de batidas |
| `0x0c` | `num_beats * 8` | entradas | Lista de batidas de 8 bytes cada |

Cada entrada de batida (8 bytes):

| Offset na entrada | Tamanho | Tipo | Descrição |
| --- | --- | --- | --- |
| `0x00` | 2 | `u16` BE | `beat_number`: posição no compasso (`1..=4`) |
| `0x02` | 2 | `u16` BE | `tempo_raw`: BPM multiplicado por `100` |
| `0x04` | 4 | `u32` BE | `time_ms`: instante da batida em milissegundos |

Regras derivadas:

- `bpm = round2(tempo_raw / 100.0)` e `time_s = round3(time_ms / 1000.0)`;
- `downbeat_index` é o índice (`0`-based) da primeira batida com
  `beat_number == 1` (ou `0` se ausente);
- `tempo_type` é `"variable"` quando existem dois valores distintos de BPM na
  grade, e `"fixed"` caso contrário.

## Seção `PSSI` (estrutura de frases musicais)

No payload de `PSSI`:

| Offset no payload | Tamanho | Tipo | Descrição |
| --- | --- | --- | --- |
| `0x00` | 4 | `u32` BE | `len_entry_bytes`: tamanho de cada entrada de frase (mínimo `6`) |
| `0x04` | 2 | `u16` BE | `len_entries`: número de frases |
| `0x06` | variável | bytes | Corpo (`body`), possivelmente mascarado por XOR |

### Desmascaramento XOR do corpo

Se os dois primeiros bytes de `body` (`raw_mood` como `u16` BE) forem maiores
que `20`, cada byte `body[i]` é desmascarado com `mask[i % 19]`, onde cada
elemento de `mask` é `(BASE[k] + (len_entries & 0xFF)) & 0xFF` sobre o vetor base
de 19 bytes:

<!-- tabela-aprovada: 2026-10-24 -->
```text
0xCB, 0xE1, 0xEE, 0xFA, 0xE5, 0xEE, 0xAD, 0xEE, 0xE9, 0xD2,
0xE9, 0xEB, 0xE1, 0xE9, 0xF3, 0xE8, 0xE9, 0xF4, 0xE1
```

Após o desmascaramento, `mood = be_u16(body[0..2])` seleciona a tabela de nomes
de frase (`1` = High: `intro`, `up`, `down`, `chorus`, `outro`; `2` = Mid e
demais = Low: `intro`, `verse_1`..`verse_6`, `bridge`, `chorus`, `outro`), e as
entradas começam em `body[14..]` com `index` (`u16` BE), `beat` (`u16` BE) e
`kind_id` (`u16` BE). Na conversão canônica, prefixos `verse_*` normalizam para
`"VERSE"` e o número de compassos é `(next.beat - current.beat) / 4`.

## Seções de cues e loops (`PCOB`/`PCPT` e `PCO2`/`PCP2`)

Um arquivo ANLZ armazena cues em dois bancos separados por tipo:

- `banco = 1`: hot cues (slots A..H);
- `banco = 0`: memory cues e loops de memória.

### Variante padrão: `PCOB` / `PCPT`

O cabeçalho da seção `PCOB` tem `0x18` (24) bytes totais (`12` bytes de payload
inicial antes das entradas `PCPT`). Cada entrada `PCPT` possui cabeçalho de 12
bytes (`"PCPT"`, `len_header = 0x1c`, `len_entry = 0x38`) seguido pelo corpo
(`entry = tag[12..]`, mínimo 28 bytes):

| Offset no corpo | Tamanho | Tipo | Descrição |
| --- | --- | --- | --- |
| `0x00` | 4 | `u32` BE | `hot_cue`: `0` para memory cue/loop, `1..=8` para hot cue A..H |
| `0x04` | 4 | `u32` BE | `status`: indicador de estado da entrada |
| `0x10` | 1 | `u8` | `kind`: `1` = cue simples, `2` = loop |
| `0x14` | 4 | `u32` BE | `time_ms`: posição inicial em milissegundos |
| `0x18` | 4 | `u32` BE | `loop_ms`: posição final do loop em milissegundos |

### Variante estendida: `PCO2` / `PCP2` (labels e cores)

O cabeçalho da seção `PCO2` tem `0x14` (20) bytes totais (`8` bytes de payload
inicial: `u16` zero, `u16` banco `0`/`1`, `u16` `num_cues`, `u16` padding). Cada
entrada `PCP2` possui cabeçalho de 12 bytes (`"PCP2"`, `len_header = 0x10`,
`len_entry >= 0x2c`) seguido pelo corpo (`entry = tag[12..]`):

| Offset no corpo | Tamanho | Tipo | Descrição |
| --- | --- | --- | --- |
| `0x00` | 4 | `u32` BE | `hot_cue`: `0` para memory cue/loop, `1..=8` para hot cue A..H |
| `0x04` | 1 | `u8` | `kind`: `1` = cue simples, `2` = loop |
| `0x06` | 2 | `u16` BE | Escala fixa (`1000`) |
| `0x08` | 4 | `u32` BE | `time_ms`: posição inicial em milissegundos |
| `0x0c` | 4 | `u32` BE | `loop_ms`: posição final do loop em milissegundos |
| `0x10` | 1 | `u8` | `color_id`: índice de paleta da cue |
| `0x14` | 4 | `u16`x2 BE | Numerador e denominador de loop quantizado |
| `0x1e` | 2 | `u16` BE | `len_comment`: tamanho em bytes do comentário UTF-16BE |
| `0x20` | `len_comment` | UTF-16BE | Comentário da cue (quando `len_comment > 0`), terminado por `\0` |
| `0x20 + len_comment` | 4 | `u8`x4 | Cauda opcional de cor: `[color_code, red, green, blue]` |

Regras de decodificação e conversão canônica (`cue_sources` e `canonical_cues`):

1. **Label (`Option<String>`):** se `entry.len() >= 32 + len_comment`,
   `len_comment > 0` e `len_comment % 2 == 0`, decodifica UTF-16BE, remove `\0`
   nas extremidades e produz `Some(texto)` quando não vazio; se `32 + len_comment`
   ultrapassar `entry.len()`, a entrada truncada é descartada sem pânico;
2. **Cor (`Option<u32>`):** quando os 4 bytes após o comentário existem e
   `rgb = (r << 16) | (g << 8) | b` ou `color_code` não é zero, `color = Some(rgb)`;
3. **Precedência de fontes (`cue_sources`):**
   - Usa todas as seções `PCO2` de `.EXT` (via `all_sections`) quando produzirem
     ao menos uma cue;
   - Caso contrário, tenta todas as seções `PCO2` de `.DAT`;
   - Caso contrário, tenta todas as seções `PCOB` de `.DAT` (evitando que uma
     `PCOB` vazia em `.EXT` esconda cues presentes em `.DAT`);
   - Por fim, tenta todas as seções `PCOB` de `.EXT`.
4. **Ordenação e deduplicação:** as cues canônicas são ordenadas por
   `(position_s, type_name, hotcue)` e deduplicadas preservando a entrada com
   `label`/`color`.

## Exportador em lote (`vilelabox-anlz-export`)

Contrato operacional:

1. Percorre `--anlz-dir` recursivamente em busca de `.DAT`;
2. Resolve o arquivo de áudio referenciado em `PPTH` **exclusivamente** dentro
   da raiz `--music-dir` (caminhos absolutos fora de `--music-dir` ou tentativas
   de travessia `..` são rejeitados e contados em `missing_audio`);
3. Calcula o SHA-256 do áudio em streaming (blocos de 64 KiB);
4. Converte cada par `.DAT`/`.EXT` via `track_from_anlz` e grava `<sha256>.json`
   e `index.json` atomicamente (arquivo temporário no diretório de saída seguido
   de `rename`);
5. Recusa execução quando `--output` coincide ou se sobrepõe a `--anlz-dir` ou
   `--music-dir`;
6. Em caso de falha parcial durante o lote, remove os arquivos temporários e não
   publica `index.json` incompleto.
