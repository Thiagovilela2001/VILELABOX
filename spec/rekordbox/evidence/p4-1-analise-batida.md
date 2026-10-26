# p4-1 — Análise de BPM e grade: encadeamento, tipos e constantes

Dossiê de proveniência da fase A1.1 do `docs/PLANO-ANALISE.md`. Registra o que
foi recuperado por análise estática, com endereços, e o que continua
desconhecido. Nenhum comportamento daqui foi medido em execução ainda.

## Artefato

| Campo | Valor |
| --- | --- |
| Binário | `C:\Program Files\rekordbox\rekordbox 7.2.8\rekordbox.exe` |
| Versão | 7.2.8.0 |
| SHA-256 | `85e3a58524c5cd1e870ad59e23cc8806523175f80fb82d6059c68d57bdf9991a` |
| Imagem base | `0x140000000`; `.rdata` em VA `0x14323a000`, arquivo `0x3239400` |
| Projeto Ghidra | `C:\tools\ghidra_projects\rekordbox728` (somente leitura) |
| Script | `DecompileAddrs.java` (privado, `third_party/rekordbox/re/ghidra/`) |
| Saídas | `third_party/rekordbox/re/export/p4/` (privadas, fora do Git) |

## Método

As funções do job de batida não têm símbolo, mas registram o próprio
`__FUNCSIG__` no log. As strings foram localizadas no arquivo, convertidas
para VA e usadas como `xref:` no Ghidra. Constantes foram lidas direto do
`.rdata` do binário.

| String de log (VA) | Função |
| --- | --- |
| `AnalyzeJobBeatDetector::Initialize` (`0x1434eac50`) | `0x14112eef0` |
| `AnalyzeJobBeatDetector::Detect` (`0x1434ead70`) | `0x14112ef90` |
| `AnalyzeJobBeatDetector::getFirstBeatPos` (`0x1434eaef0`) | `0x141130000` |
| `AnalyzeJobBeatDetector::legacyBeatAnalyze` (`0x1434eaf50`) | `0x14112fd70` |
| `AnalyzeJobBeatDetector::highPrecisionBeatAnalyze` (`0x1434eb180`) | `0x14112f0c0` |
| `AnalyzeJobKey::Initialize` / `Detect` | `0x141133710` / `0x1411337b0` |

## Encadeamento

1. `Detect` (`0x14112ef90`) lê o modo em `config+0x60`. Modos 2, 3 e 4 vão para
   `highPrecisionBeatAnalyze`; os demais, para `legacyBeatAnalyze`.
2. `highPrecisionBeatAnalyze` fixa uma flag no detector conforme o modo
   (2 = valor atual do detector, 3 = 0, 4 = 1) e chama `0x141134550`, que
   devolve um documento JSON com `beatList` (campos de três letras e `numBar`).
3. `legacyBeatAnalyze` pede ao detector o BPM e a lista de batidas, aloca
   12 bytes por batida e chama o pós-processamento `0x1411301d0`.
4. O detector (`job+0x98`) é um envelope com dois motores, escolhidos por
   `detector+0x14` (1 ou 2). O motor 2 roda em `0x14247a210` (~12 900 linhas
   decompiladas, vetorizado com AVX).

## Tipos e saída do motor

| Item | Valor | Endereço |
| --- | --- | --- |
| BPM do motor | `f32` em `motor+0x48` (motor 1) e `motor+0x4c` (motor 2) | `0x14107c200`, `0x14247a220` |
| BPM entregue | `(int)(bpm * 100.0f + 0.5f)`, BPM × 100 | idem; constantes `0x145415b68` = 100,0 e `0x145414d20` = 0,5 |
| Batida do motor | registro de 24 bytes: `i32` posição, `f32` intervalo em ms, `i32` número da batida | `0x14247a290` |
| Batida entregue | registro de 12 bytes: posição, número, `(int)(6000000.0f / intervalo + 0.5f)` | `0x14247a290`; `0x14364da78` = 6 000 000,0 |
| Sinal analisado | buffers `i16` em `motor+0x78` e `motor+0x80`, comprimento `i32` em `motor+0x1c`, taxa `i32` em `motor+0xc` | `0x1424768e0`, `0x142476a40` |

## Soma de janela por batida

`0x142476a40` (batida) e `0x1424768e0` (meia batida) têm comportamento
completo:

1. Se `|bpm| <= FLT_EPSILON` (`0x145414798`) ou `pos / 4 < 0` (divisão inteira
   truncada), devolve −1,0 (`0x145415e58`).
2. `n = (int)(K / bpm * taxa + 0.5)`, com `K` = 60,0 (`0x145415638`) para a
   batida e 30,0 (`0x1454155d8`) para a meia batida.
3. `inicio = (int)(pos * 0.25 + 0.5)`.
4. Se `comprimento < n - 1 + inicio`, devolve −1,0.
5. Se `n < 1`, devolve 0,0.
6. Devolve a soma em `f64` de `n` amostras `i16` a partir de `inicio`; a
   batida usa o buffer em `+0x80` e a meia batida o de `+0x78`.

## Pós-processamento da faixa de BPM (`0x1411301d0`)

- Tabela em `0x1434eb4b0`, 11 pares `i32` (mínimo, máximo), indexada pela
  preferência em `config+0x6c`; fora de 1..10, a preferência é redefinida.
- Valores: índice 0 e 1 = 70–180; 2 = 48–95; 3 = 58–115; 4 = 68–135;
  5 = 78–155; 6 = 88–175; 7 = 98–195; 8 = 108–215; 9 = 118–235; 10 = 128–255.
- Abaixo de `mínimo × 100`, o BPM × 100 dobra até entrar na faixa; acima de
  `máximo × 100`, é dividido por dois (divisão inteira) até entrar.
- Posições saem em milissegundos: `pos * 1000000 / taxa`, separados em parte
  inteira (`/ 1000`) e resto em milésimos.
- `getFirstBeatPos` (`0x141130000`) usa 6 000 000,0 (`0x1434e4180`) dividido
  pelo BPM × 100, ou seja o intervalo de batida em ms.

## Classificação

- **OBSERVADO:** encadeamento do job, escolha do modo, tabela de faixas de
  BPM, arredondamento do BPM × 100, layout das batidas do motor e da saída,
  buffers `i16` e a soma de janela por batida.
- **INFERIDO:** os buffers `i16` em `+0x78` e `+0x80` são um sinal derivado do
  áudio e não o PCM bruto, porque o índice de início é a posição dividida por
  4. A taxa em `+0xc` seria a taxa desse sinal. Não há prova ainda.
- **INFERIDO:** os índices 1..10 da tabela correspondem às opções de faixa de
  BPM das preferências de análise.
- **DESCONHECIDO:** como o áudio vira os buffers `i16`; o algoritmo de
  `0x14247a210`; o motor 1; o significado das tabelas de razões em
  `0x143b107b8`–`0x143b10880` (0,008 a 2,23) usadas por `0x14247a210`; as
  rotinas de dobrar e dividir a grade (`0x1411308c0`, `0x141130d90`); o modo
  de alta precisão por dentro de `0x141134550`; a análise de tom.
- **Não observado:** nenhuma FFT nem janela de Hann apareceu até aqui no
  caminho do motor 2. O contrato `f32` com FFT do plano A1.2/A1.3 não tem
  evidência e fica suspenso.

## Próximos alvos

1. Quem preenche `motor+0x78` e `motor+0x80` (escritas nesses offsets a partir
   de `0x14247a210` e do construtor do motor 2).
2. `0x142473620`, `0x142473710` e `0x142475700`, chamados com a lista de
   batidas e 900 × taxa.
3. Teste diferencial: rodar a análise numa fixture WAV conhecida e comparar a
   grade do `.DAT` com a soma de janela reimplementada.
