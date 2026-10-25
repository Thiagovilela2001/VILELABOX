# G0 — Dossiê de proveniência: Contêiner, Exportação e Cues ANLZ (P1.1 e P1.2)

## Artefato analisado

- **Arquivo:** `rekordbox.exe`
- **Versão:** `7.2.8.0`
- **SHA-256:** `85e3a58524c5cd1e870ad59e23cc8806523175f80fb82d6059c68d57bdf9991a`
- **Registro privado detalhado:** `third_party/rekordbox/re/evidence/11-anlz.md`

## Identificadores de classes, tabelas virtuais e rotinas observadas

| Subsistema | Símbolo RTTI / String | Vtable / Dado (RVA) | Rotinas analisadas (RVA) |
| --- | --- | --- | --- |
| Arquivo de análise | `analyzer::CMusicAnalyzeFile` | `vft_34db2d8` | `rva_111df30`, `rva_111de40` |
| Bloco ANLZ | `ANLZBlock` | `vft_34dc978` | `rva_1112b60`, `rva_1112c90` |
| Cues estendidas (`PCP2`) | `analyzer::CUE_EXT_DATA` | `vft_34ea258` | `rva_111c460`, `rva_111c6f0`, `rva_111cb40`, `rva_111ebb0` |
| Cues legadas (`PCPT`) | `PCOB` / `PCPT` | `tag_34ea1e0` | `rva_111c000`, `rva_111c260`, `rva_1112110`, `rva_111e7d0` |
| Estrutura de frases (`PSSI`) | `analyzer::SongStructInfo` / `analyzer::PhraseInfo` | `vft_34ea268` / `vft_34ea278` | `rva_111daf0`, `rva_111f790`, `rva_111d530`, `rva_111d690` |
| Resolução `USBANLZ` | `PIONEER/USBANLZ`, `ANLZ0000.DAT`, `ANLZ0000.EXT` | `str_3972f80`, `str_34e41e8`, `str_34e41f8` | `rva_10c40f0`, `rva_10c42c0`, `rva_10c46d0`, `rva_1127140` |

## Cadeias de chamadas e estados observados

1. **Descoberta e associação `.DAT` / `.EXT` (`rva_10c42c0`, `rva_10c46d0`, `rva_1127140`):**
   - `rva_10c42c0` resolve o diretório relativo sob `PIONEER/USBANLZ` e seleciona `ANLZ0000.DAT` (`str_34e41e8`) para o arquivo base ou `ANLZ0000.EXT` (`str_34e41f8`) para o arquivo estendido irmão no mesmo diretório.
   - `rva_10c46d0` percorre sufixos hexadecimais `ANLZ%04X.DAT` (`str_34e4218`, intervalo `0..65536`) quando há múltiplos arquivos de análise no mesmo diretório.
2. **Despachante do contêiner `PMAI` (`rva_111df30`, `rva_1112b60`, `rva_1112c90`, `rva_111de40`):**
   - `rva_1112b60` valida e grava o cabeçalho `PMAI` (`0x1c` = 28 bytes) com tamanho total big-endian no offset `+0x08`.
   - `rva_1112c90` processa a seção `PPTH` (`len_header = 0x10`), armazenando o comprimento em bytes da string UTF-16BE em `+0x0c` e convertendo cada unidade `u16` em big-endian.
   - `rva_111df30` coordena as seções do arquivo: `PMAI`, `PPTH`, `PQTZ`, dois bancos `PCOB` (offsets `+0x50` para hot cues e `+0x58` para memory cues), dois bancos `PCO2` (offsets `+0x60` para hot cues e `+0x68` para memory cues), `PQT2` (`+0x70`), formas de onda (`+0x78`, `+0x80`, `+0x88`), `PSSI` (`+0x90`) e preservação de blocos desconhecidos (`+0x98`, `rva_111de40`).
3. **Leitura de `PCOB` / `PCPT` (`rva_111c000`, `rva_111c260`, `rva_1112110`):**
   - `rva_111c000` lê os 24 bytes (`0x18`) do cabeçalho `PCOB`, identifica o tipo de banco em `+0x0e` (`1` = hot cues, `0` = memory cues) e a contagem `u16` em `+0x12`.
   - `rva_111c260` lê cada entrada `PCPT` (`0x38` = 56 bytes) via `rva_1112110`. Quando o banco é de hot cues (`1`), entradas posteriores com o mesmo slot (`+0x0e`) substituem a anterior.
4. **Leitura de `PCO2` / `PCP2` com labels e cores (`rva_111c460`, `rva_111c6f0`, `rva_111cb40`, `rva_111ebb0`):**
   - `rva_111c460` lê os 20 bytes (`0x14`) do cabeçalho `PCO2`, exige `len_header == 0x14` e `u16` zero em `+0x0c`, identifica o banco em `+0x0e` (`1` = hot cues em `+0x60`, `0` = memory cues em `+0x68`) e a contagem `num_cues` (`u16` BE) em `+0x10`.
   - `rva_111cb40` lê o prefixo fixo de 44 bytes (`0x2c`) da entrada `PCP2`, exige `len_header == 0x10` (16) e `u16` zero em `+0x0c`, e extrai:
     - `+0x0e..0x10` (`entry[2..4]`, equivalente a `entry[0..4]` como `u32` BE): `hot_cue` (`0` = memory cue, `1..=8` = hot cue A..H);
     - `+0x10` (`entry[4]`): `kind` (`1` = cue, `2` = loop);
     - `+0x12..0x14` (`entry[6..8]`): constante `1000` (`0x03e8`);
     - `+0x14..0x18` (`entry[8..12]`): `time_ms` (`u32` BE);
     - `+0x18..0x1c` (`entry[12..16]`): `loop_ms` (`u32` BE);
     - `+0x1c` (`entry[16]`): `color_id` (`u8`);
     - `+0x20..0x24` (`entry[20..24]`): numerador e denominador de loop (`u16` BE);
     - `+0x2a..0x2c` (`entry[30..32]`): `len_comment` (`u16` BE, tamanho em bytes do texto UTF-16BE).
   - `rva_111c6f0` valida os bytes restantes `remaining = len_entry - 0x2c`:
     - Se `len_comment > 0`, exige `len_comment <= remaining` e `len_comment > 1`, decodifica `len_comment / 2` unidades `u16` big-endian em UTF-16 (`entry[32 .. 32 + len_comment]`) e desconta `len_comment` de `remaining`.
     - Se `remaining > 0`, exige `remaining >= 4` e lê 4 bytes de cor a partir de `entry[32 + len_comment]`: `[color_code, red, green, blue]`.

## Classificação das conclusões

- **OBSERVADO:**
  - Layout binário big-endian de `PMAI`, `PPTH`, `PQTZ`, `PSSI` (com máscara XOR de 19 bytes em `rva_111daf0`), `PCOB`/`PCPT` e `PCO2`/`PCP2`.
  - Separação de hot cues (`banco = 1`) e memory cues (`banco = 0`) em duas seções `PCOB` e duas seções `PCO2` por arquivo.
  - Posições de `hot_cue`, `kind`, `time_ms`, `loop_ms`, `len_comment`, comentário UTF-16BE e tupla `[color_code, r, g, b]` em `PCP2`.
- **INFERIDO:**
  - Regra de seleção `cue_sources`: priorizar todas as seções `PCO2` (quando contiverem entradas), caindo para todas as seções `PCOB` de `.DAT` e depois `.EXT`, evitando duplicar a mesma cue sem perder `label` e `color`.
- **DESCONHECIDO:**
  - Significado operacional dos 40 bytes extras após `[color_code, r, g, b]` quando a cauda opcional de `PCP2` tem `0x2c` bytes em `rva_111c6f0`.
- **ADAPTAÇÃO:**
  - No exportador `vilelabox-anlz-export`, a resolução do caminho `PPTH` é estritamente confinada à raiz `--music-dir` fornecida na linha de comando, e a escrita de `<sha256>.json` e `index.json` é feita de forma atômica com limpeza em caso de falha parcial.

## Limitações da análise estática e confirmação em cópias reais

- As 40 cópias reais em `third_party/rekordbox/anlz-test-copy-3b851411ede04778b5cf81909bae9f8e` e os arquivos de `USBANLZ` local possuem cabeçalhos `PCOB` (`0x18` bytes) e `PCO2` (`0x14` bytes) com `num_cues = 0` (pois cues locais ficam em `master.db` até exportação).
- O comportamento de `PCPT` e `PCP2` com `label` UTF-16BE, `color` RGB, múltiplos bancos `PCO2` e entradas truncadas foi traduzido da análise estática e coberto por fixtures sintéticas. As 40 faixas reais mantêm 100% de paridade somente para os campos efetivamente presentes nelas; como possuem `num_cues = 0`, ainda falta confirmação diferencial com uma cópia real que contenha cues.

## Rastreabilidade para código e testes

- **Especificação:** [`spec/rekordbox/11-anlz.md`](../11-anlz.md)
- **Implementação Rust:** `vilelabox-anlz/src/{container,grid,phrases,cues,canonical,export}.rs`, `vilelabox-anlz/src/bin/vilelabox-anlz-export.rs`
- **Testes:** `vilelabox-anlz/src/cues.rs`, `vilelabox-anlz/src/canonical.rs`, `vilelabox-anlz/tests/export.rs`, `vilelabox-anlz/tests/export_parity.rs`, `vilelabox-anlz/tests/real_parity.rs`
