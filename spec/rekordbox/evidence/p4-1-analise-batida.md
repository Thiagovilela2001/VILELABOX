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
   `detector+0x14` (1 ou 2). A rotina `0x14247a210` do motor 2 (~12 900 linhas
   decompiladas, vetorizada com AVX) **não lê PCM**: recebe uma lista de
   batidas já detectadas (registros de três `i32`: posição, número e BPM × 100;
   quantidade no 6º argumento), copia-a para um vetor de 24 bytes em
   `objeto+0x70`, grava `interval.csv` de depuração com `"%.3f, %u"` (intervalo
   em ms e BPM × 100) e então refina. É um estágio de ajuste da grade.

## Tipos e saída do motor

| Item | Valor | Endereço |
| --- | --- | --- |
| BPM do motor | `f32` em `motor+0x48` (motor 1) e `motor+0x4c` (motor 2) | `0x14107c200`, `0x14247a220` |
| BPM entregue | `(int)(bpm * 100.0f + 0.5f)`, BPM × 100 | idem; constantes `0x145415b68` = 100,0 e `0x145414d20` = 0,5 |
| Batida do motor | registro de 24 bytes: `i32` posição, `f32` intervalo em ms, `i32` número da batida | `0x14247a290` |
| Batida entregue | registro de 12 bytes: posição, número, `(int)(6000000.0f / intervalo + 0.5f)` | `0x14247a290`; `0x14364da78` = 6 000 000,0 |
| Sinal das somas | buffers `i16` em `obj+0x78` e `obj+0x80`, comprimento `i32` em `obj+0x1c`, taxa `i32` em `obj+0xc`, onde `obj` é o 1º argumento das somas | `0x1424768e0`, `0x142476a40` |

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
   batida usa o buffer em `obj+0x80` e a meia batida o de `obj+0x78`.

O objeto `obj` das somas **não** é o mesmo de `0x14247a210`: lá, `+0x70` e
`+0x78` são início e fim de um vetor de registros de 24 bytes.

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

## Quem usa as somas e o refinador (A1.1b)

- `0x14107c340` (refinador do motor 2) só é chamado por
  `highPrecisionBeatAnalyze`. O refinamento de grade pertence ao caminho de
  alta precisão.
- As somas de janela são chamadas por `0x142470b60` e pelo refinador gêmeo
  `0x14245c630` (mesma assinatura de `0x14247a210`, grava `interval.csv` a
  `interval5.csv`), sempre com `obj = motor+0x640`.

## Posição de entrada (`0x142470b60`)

Comportamento completo; portado em `vilelabox-dsp::posicao_de_entrada`.

1. Com a lista de batidas em `motor+0x58` (registros de 24 bytes, posição no
   primeiro `i32`) e `n` batidas, calcula para as `n - 1` primeiras a soma de
   meia batida em `motor+0x640` com o BPM `motor+0x48`. Um −1,0 (fora do
   sinal) é trocado pelo valor anterior, exceto na primeira batida.
2. Normaliza pelo máximo, com piso 0, se `|máximo| > DBL_EPSILON`. Os primeiros
   `m - m % 16` valores são divididos (laço vetorial, só quando `m > 15`); os
   demais são multiplicados por `1 / máximo`.
3. `d[i] = x[i] - x[i-1]` para `1 <= i <= n - 2`; `d[0] = d[n-1] = 0`.
4. Máquina de estados de `i = 0` até `n - 3`, com limiares 0,4
   (`0x145415250`), 0,2 (`0x1454151e0`), 0,15 (`0x1454151a8`) e −0,4
   (`0x1435aeb80`): uma subida (`x > 0,2` e `d > 0,15`) abre candidato se não
   houve nenhum antes ou se passaram mais de 32 trechos quietos; batidas fortes
   seguidas somam; mais de 12 encerram e fixam o candidato na última subida;
   trecho quieto (`x` < 0,2 em três batidas, ou `d < -0,4` seguido de duas
   `d` < 0,2) zera a contagem.
5. Grava `(60 / bpm) * (índice - 4) * motor+0x14` em `motor+0x50`, com
   negativos virando 0.

## Detector legado sobre o PCM (`CBeatAnalyzeThread`)

- A thread `0x1410b3cb0` cria o analisador de 0x1d80 bytes (`0x14116f000`) e o
  inicializa em `0x141170020` com o total de frames, a taxa de amostragem e um
  terceiro valor ainda não identificado. Taxas tratadas no fluxo: 32 000,
  44 100, 48 000, 88 200, 96 000, 176 400 e 192 000.
- O init grava `taxa/2`, `taxa/8` e `taxa/32` (`0x143537e90` = 0,5,
  `0x143537e98` = 0,125, `0x1454150c0` = 0,03125), o estado 1 e o bloco de
  leitura `(int)(taxa * 1,70667 + 0,5)` frames (`0x143537e80`), ou seja
  75 264 frames a 44,1 kHz. Portado em `frames_por_bloco`.
- Entrada de PCM (`0x1411705c0`): dois canais **`f32` planares** e a
  contagem, misturados em mono **`f64`** como
  `b * 0,01/32768 + a * 0,99/32768` (`0x143537e70`, `0x143537e78`). A escala
  supõe amostras em ±32768. Portado em `mixar_mono`.
- Duas passadas por bloco: `0x1411702d0` (estados 1 e 2, "A/B", fecha com
  `0x141195480` e `0x141195ed0`) e `0x141170430` (estados 3 e 4, "C/D", fecha
  com `0x141197d60`). Estado −1 é erro e 5 é concluído.

## Filtros do detector legado (A1.1c)

Portados em `vilelabox-dsp::filtro`.

- **Biquad** (`0x141166570`): forma direta II transposta, coeficientes
  `[b0, b1, b2, a1, a2]` e estado de dois termos mantido entre blocos.
- **Coeficientes** (`0x141165eb0`): transformada bilinear do protótipo
  `(b2·s² + b1·s + b0) / (a2·s² + a1·s + a0)` com `T = 1/fs`, termos `4·a2`
  e `4·b2`, normalizados pelo denominador.
- **Pré-distorção** (`0x141165dc0`): `Ω = tan(ω / 2fs) · 2fs`, `ω = 2π·corte`
  (`0x1454154f8` = 2π). A função `0x14279adf0` é `tan`: a aproximação para
  argumento pequeno é `x + x³ · 0,3333…` (`0x143d0b0f0`).
- **Butterworth de 8ª ordem** (`0x141191270` passa-baixa, `0x14118ad30`
  passa-alta): quatro seções com `a1 = 2·cos(i·π/8 + π/16)·Ω`, i = 0..3
  (`0x1435380a8`, `0x143538388`; `0x1427a2fa0` é `cos`), `a0 = Ω²`, `a2 = 1`.
  Numerador `(0, 0, Ω²)` no passa-baixa e `(1, 0, 0)` no passa-alta. Exige
  `fs > 0` e decimação ≥ 2.
- **Decimação** (`0x14117cbb0`): cada seção filtra o bloco inteiro; a saída
  são `n / M` amostras, a de índice `i·M + M − 1`. O resto do bloco é
  descartado.
- **Passada A** (`0x1411952f0`): passa-baixa em `fs/8 · 0,4`
  (`0x145415250` = 0,4) e acumulação até `min(fs/8 · 210, frames / 8)`
  amostras (`0x1435384d0` = 210, ou seja 210 s do sinal decimado).
- **Passada B** (`0x141195ed0`): três bandas, passa-alta em `fs/2 · 0,55`
  com ÷2 (`0x1454152d0` = 0,55), passa-baixa em `fs/8 · 0,4` com ÷8 e
  passa-baixa em `fs/32 · 0,4` com ÷32; buffers de `40 s` por banda
  (`0x145415608` = 40). Depois chama `0x141173370` com as três taxas.

## Grades da passada B (`0x141173370`)

Portada em `vilelabox-dsp::grades_passada_b`; conferida no assembly.

- Chamada por `0x141195ed0` como `(obj, n1, fs1, n2, fs2, n3, fs3, bpm, ms)`,
  com `ni = frames (obj+0xc0) / Mi` em `f64` (`Mi` = `i32` em `+0x78`,
  `+0x7c`, `+0x80`), `fsi` = `f64` em `+0x90`, `+0x98`, `+0xa0`, BPM em
  `+0xb0` e `ms` = 2º argumento do chamador. O laço por bloco (`0x1411702d0`) chama `0x141195ed0(obj, +0x208, +0x20c)` ao fechar a passada A, salvo com `+0xc9` ligado: `ms` é o deslocamento final da A e `arg3` o número da batida. **Não estima BPM**: o BPM chega
  pronto, como na passada A.
- Por banda: passo `(60/bpm)·fs` (`0x145415638` = 60), início
  `(int)fma(ms·0,001, fs, 0,5)` (`0x145414e88`, `0x1454152a0`), quantidade
  `(int)((n − início) / passo)`.
- `q` = mínimo das três quantidades, gravado em `+0x1180`; cada grade tem
  `q + 1` índices `(int)((i·passo + início) + 1,5)` (`0x145415408`) em
  `+0x1120`, `+0x1140` e `+0x1160` (vetores de `0x141167dc0`).
- O índice `q − 1` (penúltimo) é limitado a `(int)n`. Devolve falso só em
  falha de alocação (`q + 1 < 0`).
- Depois, o chamador grava `+0xb48 = 60/bpm`, `+0xb50 = ms·0,001`, liga
  `+0xc8` se `q − número + 1 >= 32`, guarda `ms`/`arg3` em `+0xb90..+0xb98`,
  aloca cerca de 25 vetores e encadeia `0x141187f60`, `0x141187a50`,
  `0x141188560`, `0x141188c50`, `0x141189410`, `0x1411898c0`, `0x14118e120`,
  `0x14118cf70`, `0x14117e830`, `0x14117ff60`, `0x14117b170`, `0x14117ac40`,
  `0x141194730`, `0x14117a050` e `0x14117e1b0`.

## Inicialização da passada B (cadeia de `0x141195ed0`)

Todas as 15 funções após `0x141173370` só **configuram** estado (filtros,
buffers, núcleos); nenhuma processa sinal nem estima BPM. Filtros via
`0x141165dc0` (pré-distorção), `0x141165eb0` (biquad bilinear),
`0x141165e30` (1ª ordem bilinear), `0x141165da0` (zera estado) e
`0x141165d70(destino, origem)`. `fs` é o argumento recebido
(`taxa/32` = `obj+0xa0` ou `taxa/8` = `obj+0x98`, conforme o chamador).
Passa-baixa de 2ª ordem = numerador `(0, 0, Ω²)`, `a1 = √2·Ω`
(`0x1454153e8`); passa-alta de 2ª ordem = numerador `(1, 0, 0)`.

| Função | fs | Estado configurado |
| --- | --- | --- |
| `0x141187f60` | taxa/32 | PB2 100 Hz (`0x1434e55e8`) em `+0x1188`; PB1 5 Hz (`0x1435380c8`) em `+0x11c0` |
| `0x141187a50` | taxa/32 | PB2 150 Hz (`0x143538340`) em `+0x470`; PB1 5 Hz em `+0x4a8` |
| `0x141188560` | taxa/32 | ver "Filtros do envelope" |
| `0x141188c50` | taxa/32 | PB2 40 Hz (`0x143538368`) em `+0x1258`, repassado a `+0x1290`, `+0x12c8`, `+0x1300` |
| `0x141189410` | taxa/8 | PA2 500 Hz (`0x1435380f0`) em `+0x1378`; PB1 5 Hz em `+0x13b0` |
| `0x1411898c0` | taxa/8 | duas seções Butterworth (`a1 = 2·cos(i·π/4 + π/8)·Ω`, `0x1435380b0`, `0x1435380a8`): PA 400 Hz (`0x1434e5608`) em `+0x1408`/`+0x1440` e PB 1 kHz (`0x143538378`) em `+0x1478`/`+0x14b0`; PB1 10 Hz (`0x143538160`) em `+0x14e8` |
| `0x14118e120` | taxa/8 | PB 200 Hz (`0x1434e55f0`) em `+0x15a8`, ordem em `+0x15a0` pela mesma tabela de `fs` de `FiltrosEnvelope` (`0x1435383b0`–`0x143538408`); PB2 10 Hz em `+0x1688` (fs 0 vira 5512,5, `0x1435380d0`); guarda args em `+0x16c0..+0x16d0` |
| `0x14118cf70` | taxa/8 | idem em `+0x1700`/`+0x16f8`; PB2 10 Hz em `+0x17e0` (copiado a `+0x18f8`); PA4 110 Hz (`0x143538410`) em `+0x1818`; args em `+0x1930..+0x1940` |
| `0x14117e830` | — | grava os dois args (0, 0) em `+0x1968`, `+0x1970` |
| `0x14117ff60` | taxa/8 | núcleo de **tonalidade**: 73 frequências `27,5 · 2^(k/12)` (`0x143538200`, `0x1435381e0`, `0x1435381c0` = 1/12), 72 bins (6 oitavas × 12), cos/sen de `2π·f·t`; buffer `fs·3` (`0x1454154a0`) em `+0x1998` |
| `0x14117b170`, `0x14117ac40`, `0x141194730` | — | `0x1411645f0` + `0x141165390` com 4096, 2048 e 256 em `+0x1a38`, `+0x1a88`, `+0x1ad8` |
| `0x14117a050` | taxa/32 e taxa/8 | PB2 500 Hz em `+0x1b28` (fs 1º arg) e PA2 500 Hz em `+0x1b60` (fs 2º arg); PB1 5 Hz em `+0x1b98`/`+0x1bd0` |
| `0x14117e1b0` | taxa/2 | Butterworth de duas seções: PA 5 kHz (`0x143538168`) em `+0x1c48`, PB 7 kHz (`0x143538170`) em `+0x1cb8`; PB1 10 Hz em `+0x1d28` |

Conclusão: a estimativa de BPM não está na passada B nem na sua
inicialização. Falta portar os **consumidores** desse estado, no
processamento por bloco (`0x1411702d0`, estados 1 e 2) e nas passadas C/D.

## Laço por bloco das passadas C/D (`0x141170430`)

Estados 3 e 4. Exige `+0xc8` ligado (grade da B com ≥ 32 batidas desde o
número da batida). No estado 3, cada bloco vai a `0x1411965b0` até
`+0xb40 == 2`; então o estado vira 4 e fecha com `0x141197d60` (estado 5).
No estado 2, `0x1411702d0` fecha a A e chama a init da B; não há consumidor
da B ali.

## Passada C por bloco (`0x1411965b0`) — mapa observado

Estrutura apenas; a semântica de cada característica ainda não foi aberta.

1. Filtra o bloco nas três bandas da B (`0x14118ae80` em `+0x768`,
   `0x1411913c0` em `+0x850` e `+0x938`) e acumula em `+0xa80` (taxa/2),
   `+0xaa0` (taxa/8) e `+0xac0` (taxa/32).
2. Avança o relógio da grade (`+0xb48` = 60/bpm, `+0xb50`/`+0xb58`/`+0xb60`,
   contadores `+0xb68`–`+0xb7c`). Ao cobrir a janela, `+0xb40 = 1` e os sinais
   são recortados a partir das grades `+0x1120`/`+0x1140`/`+0x1160`; no fim do
   sinal, `+0xb40 = 2`.
3. Por lote de batidas (cursor `+0xde0`, via `0x1411c5cb0`), cada extrator
   grava no buffer indicado:

| Extrator | Banda | Saída |
| --- | --- | --- |
| `0x141188050` | `+0xac0` + grade `+0x1160` | `+0xc00`, `+0xc40` (×4 por batida), `+0xc20` |
| `0x141187b40` | `+0xac0` | `+0xca0` |
| `0x1411885f0`, `0x141188d40` | `+0xac0` | `+0xba0`, `+0xd20` |
| `0x141189500` | `+0xaa0` | `+0xcc0` |
| `0x141189ab0` | `+0xaa0` | `+0xc60` |
| `0x1411711f0` | `+0xaa0` | `+0xd40` |
| `0x141171510` | `+0xaa0` | `+0xbc0`, `+0xbe0` |
| `0x14118e450` | `+0xaa0` | `+0xd80` |
| `0x14118d310` | `+0xaa0` | `+0xda0` |
| `0x14117e850` | `+0xa80` + grade `+0x1120` | `+0xdc0` |
| `0x1411803f0` | `+0xaa0` | `+0xdf8` (72 colunas, via `+0x1a18`; usa o núcleo de tonalidade) |
| `0x14117b1b0` | `+0xaa0` | `+0xe10` (via `+0x1a70`) |
| `0x14117ac80` | `+0xaa0` | `+0xe28` (via `+0x1ac0`) |
| `0x14117a200` | `+0xac0` e `+0xaa0` | `+0xce0`, `+0xd00` |
| `0x14117e3a0` | `+0xa80` | `+0xd60` |

4. Com `+0xb40 == 2`, chama `0x141197890(obj, …)` para o resto.

### Envelope por batida (`0x141187b40`)

Portado em `vilelabox-dsp::EnvelopePorBatida`; conferido no assembly.
Argumentos: banda `+0xac0`, BPM `+0xb0`, `fs = taxa/32` (`+0xa0`), grade
`+0x1160`, `n` batidas do lote, saída `+0xc80` (copiada para `+0xca0`).

1. Sinal → passa-baixa 2ª ordem 150 Hz (`+0x470`, `0x141166570`) → valor
   absoluto (`0x141167080`) → 1ª ordem 5 Hz (`+0x4a8`, `0x141166430`). Os
   estados dos filtros persistem no objeto entre lotes.
2. `limite = grade[n − 1] − (int)((−60/bpm)·fs)` (`0x143537ea0` = −60), isto
   é, a última batida mais uma; se `limite − 1 > len`, vira `len`. A grade
   local tem `n + 1` entradas com `limite` no fim.
3. Batida `i`: `soma(env[g[i] − 1 .. g[i + 1] − 1])` (`0x141167150`) se
   `0 < g[i] <= len` e `g[i + 1] − 1 <= len`; senão **0,0** (`xmm6` zerado).
   Segmento de comprimento negativo faz `0x141166c30` falhar e a função
   devolver falso.
4. Todas as saídas × 4 (`0x1454154c0`).

### Energia por quarto na passada C (`0x1411885f0`, `0x141188d40`)

- `0x1411885f0` é a mesma função da passada A (`energia_por_quarto`), agora
  com a banda `+0xac0`, `fs = taxa/32`, grade `+0x1160` limitada às `n`
  batidas do lote e o passa-baixa de 100 Hz em `+0x3a0` reprojetado em
  taxa/32 e zerado por `0x141188560` na init da B. Quartos em `+0x3d8`,
  copiados a `+0xba0`. Não há código novo.
- `0x141188d40`, portado em `EnergiaPorQuarto40` e conferido no assembly:
  quatro passa-baixas de 40 Hz em cascata (`+0x1258`, `+0x1290`, `+0x12c8`,
  `+0x1300`), depois o mesmo cálculo (janelas `(int)(fs/bpm·15 + 0,5)`,
  `energia` = `0x1411671d0`, × 4 fora de 5512,45–5512,55, mesma árvore de
  16 termos por compasso). Única diferença: a quarta janela usa
  `grade[i + 1]` enquanto `i + 1 < n`, contra `i + 2 < n` em `0x1411885f0`
  (contador iniciado em 2 em `0x1411886ea`). Saídas `+0x1338` (copiada a
  `+0xd20`) e `+0x1358`.

### Pico por batida (`0x1411711f0`)

Portado em `vilelabox-dsp::pico_por_batida`; conferido no assembly.
Banda `+0xaa0`, `fs = taxa/8` (`+0x98`), BPM `+0xb0`, grade `+0x1140`.

- Sem filtro: cópia do sinal (`0x141166a30`) e valor absoluto.
- Limite como no envelope por batida (`grade[n − 1]` mais uma batida,
  `0x143537ea0` = −60), mas a grade local é a grade **inteira**
  (`0x141168020`) com o limite acrescentado (`0x141168220`): a última batida
  do lote termina em `grade[n]` quando ele existe.
- Batida `i < n`: máximo do segmento (`0x141167280` → `0x142604de0`, parte
  de −∞ em `0x145415fa0`; vazio dá 0) ou 0,0 fora do sinal. Sem fator.
- Saída `+0x1540`, copiada a `+0xd40`.

### Graves e agudos de 500 Hz por batida (`0x14117a200`)

Portado em `vilelabox-dsp::Bandas500PorBatida`; conferido no assembly.

- Banda `+0xac0` (taxa/32) → passa-baixa 2ª ordem 500 Hz (`+0x1b28`,
  `0x1435380f0`) → valor absoluto → 1ª ordem 5 Hz (`+0x1b98`). Banda `+0xaa0`
  (taxa/8) → passa-alta 2ª ordem 500 Hz (`+0x1b60`) → valor absoluto → 5 Hz
  (`+0x1bd0`). Projeto em `0x14117a050`; estados mantidos entre lotes.
- Grades `+0x1160` (graves) e `+0x1140` (agudos), `n` do lote. Batida
  `i < n − 1`: `0x141167180(env, g[i] − 1, g[i + 1] − 2)`; a última vai até
  `len − 1`. `0x141167180(v, a, b)` = soma do intervalo fechado
  (`0x14116ef60`) se `0 <= a <= b < len`, senão 0.
- Saídas `+0x1c08` (copiada a `+0xce0`) e `+0x1c28` (copiada a `+0xd00`).
  Com `n = 0` o original grava no índice −1; o porte devolve `None`.

### Agudos por batida (`0x141189500`)

Portado em `vilelabox-dsp::AgudosPorBatida`; conferido no assembly. Mesmo
molde de `0x1411711f0` (grade inteira `+0x1140` mais o limite), com banda
`+0xaa0` → passa-alta 2ª ordem 500 Hz (`+0x1378`) → valor absoluto → 1ª
ordem 5 Hz (`+0x13b0`), projetados em `0x141189410`; métrica `soma`
(`0x141167150`), inválido 0,0, sem fator. Saída `+0x13e8`, copiada a
`+0xcc0`. Os três extratores de segmento (`0x141187b40`, `0x1411711f0`,
`0x141189500`) compartilham `por_segmento` no porte.

### Pico de 400 Hz a 1 kHz por batida (`0x141189ab0`)

Portado em `vilelabox-dsp::PicoMediosPorBatida`; conferido no assembly.
Banda `+0xaa0` (taxa/8) → passa-altas de 400 Hz (`+0x1408`, `+0x1440`) →
passa-baixas de 1 kHz (`+0x1478`, `+0x14b0`), seções de `0x1411898c0` →
valor absoluto → 1ª ordem 10 Hz (`+0x14e8`). Máximo por batida sobre a grade
inteira `+0x1140` mais o limite; inválido 0,0. Saída `+0x1520`, copiada a
`+0xc60`.

### Energia e pico por quarto, sem filtro (`0x141171510`)

Portado em `vilelabox-dsp::energia_e_pico_por_quarto`. Banda `+0xaa0`
(taxa/8) crua, grade `+0x1140` limitada a `n`, as mesmas quatro janelas de
`0x141188d40` (quarta até `grade[i + 1]` enquanto `i < n − 1`). Cada janela
é copiada (`0x141166bc0`); grava `energia` (`0x1411671d0`) em `+0x1560`
(copiada a `+0xbc0`) e, após valor absoluto na cópia, o máximo
(`0x141167280`) em `+0x1580` (copiada a `+0xbe0`). Sem fator × 4 e sem
somas por compasso. As janelas dos três extratores por quarto ficam em
`janelas_de_quarto` no porte.

### Pico de 5 a 7 kHz em oitavos de batida (`0x14117e3a0`)

Portado em `vilelabox-dsp::PicoAgudosPorOitavo`; conferido no assembly.
Chamada `(obj, +0xa80, fs = taxa/2, bpm, grade +0x1120, n)` — aqui `fs`
vem antes do BPM.

- Banda `+0xa80` → passa-altas de 5 kHz (`+0x1c48`, `+0x1c80`) →
  passa-baixas de 7 kHz (`+0x1cb8`, `+0x1cf0`), seções de `0x14117e1b0` →
  valor absoluto → 1ª ordem 10 Hz (`+0x1d28`).
- Grade copiada, truncada a `n + 1` (`0x141167f00`) com
  `g[n] = g[n − 1] + (int)((60/bpm)·fs)`, trocado por `len` se `g[n] − 1 > len`.
- Oitavo `e = (int)fma((int)(batida + 0,5), 0,125, 0,5)` (`0x145415198`).
  Por batida, oito janelas fechadas 1-based: `[g, g + e]`,
  `[g + 1 + k·e, g + (k + 1)·e]` (k = 1..6) e `[g + 7e + 1, g' − 1]`; o
  máximo de cada uma vai a `+0x1d60` (8 por batida, copiada a `+0xd60`).

### Tonalidade: transformada de Q constante (`0x14117ff60`, `0x1411803f0`)

Portada em `vilelabox-dsp::tonalidade`; conferida no assembly.

**Init** (`0x14117ff60`, `fs = taxa/8`):

- Sobra `+0x1998` com capacidade `(int)(fs·3)` (`0x1454154a0`), quantidade
  `+0x19b8 = 0`.
- 73 frequências: `f0 = 27,5`, `fk = pow(2, k·(1/12))·27,5`
  (`0x1426086a0`, `pow` vetorial de 4 vias; `0x145416b40` = 0, 1, 2, 3).
- `Lk = (int)(2fs / (f(k+1) − fk))` em `+0x19c0`; pesos `+0x19e0`:
  `w0 = 1`, `wk = L0/Lk`.
- Núcleos `+0x1a00` (72 × `2·L0`): bins 0–11 `(cos, sen)((2π·fk)·t)`,
  `t = (j + 1)·(1/fs)`; bins 12–71 = quadrado complexo do bin `k − 12`
  (`re = fma(re, re, −im²)` via `vfmsub231sd`, `im = (re + re)·im`), antes
  da janela.
- Janela de Hann `0x14117dfc0` de `Lk`: `(1 − cos(i·2π·(1/(L+1))))·0,5`
  espelhada; meio ímpar `fma(−cos(m·2π/(L+1)), 0,5, 0,5)`. O laço que a
  aplica vai até `L0`, reaproveitando o buffer da janela anterior; o
  consumidor só lê as `Lk` primeiras amostras, então isso não altera o
  resultado.
- `0x1427a51e0` (par de `cos`) é tratado como seno: INFERIDO pela função
  ímpar (guarda o sinal em `r11`) com a mesma redução de argumento de `cos`.

**Lote** (`0x1411803f0`, chamado com `(obj, +0xaa0, bpm, 0, p5, fs, +0xb40)`):

- `+0x1a34 = ceil(fma(1/bpm, (p5 − 1)·0,234375, p4·3,90625e−6)·fs)`
  (`0x1435381b8`, `0x1435381b0`, `vroundsd` modo 2).
- Sinal = sobra + banda; `quadros = (int)((len − L0)·(1/256))`
  (`0x145414f10`); matriz `+0x1a18` (72 × quadros, zerada).
- Quadro `m`, bins 23–71 (`0x17`–`0x47`): segmento a partir de
  `(int)(L0·0,5) − (int)(Lk·0,5) + 256m`, `Lk` amostras; produto interno
  em blocos de 4 (`re += x0·k0`, `re = x1·k1 + re`, `fma(x2, k2)`,
  `fma(x3, k3)`), cauda sem fusão; valor `sqrt(fma(im, im, re²))·wk`.
  Bins 0–22 ficam 0.
- Com `+0xb40 < 2`, guarda `x[256·quadros..]` como sobra; `+0x1a30 =
  quadros`. O chamador acumula `+0x1a30` em `+0xde8`, copia cada linha para
  `+0xdf8` e, com `+0xb80 == 1`, grava `+0x1a34` em `+0xdec`.
- Limitação: `f64::powf` no lugar do `pow` vetorial pode diferir no último
  bit.

### FFT e espectro por compasso (`0x14117b1b0`)

Portados em `vilelabox-dsp::espectro`; conferidos no assembly; a FFT bate
com a DFT direta nos testes.

**FFT** (objeto em `+0x1a38`, `N = 4096`, init `0x14117b170`):

- `0x1411645f0`: exige `N > 7` potência de 2; `1/N` em `+0x10`; tabela
  `sen((2π/N)·i)`, `i < N + N/4` (`+0x30`), com `0x1427a51e0`; pares de
  troca por bits invertidos (`+0x18`, `+0x20`, quantidade `+0x28`).
- `0x141165390`: janela `fma(cos((i − h)·π/(h + 1)), 0,5, 0,5)`,
  `h = (N − 1)·0,5` (`0x143537e00` = π, `0x145416f80` = 0,5), com `cos`
  vetorial `0x142608cd0` (porte usa `f64::cos`: pode diferir no último bit).
- `0x1411655f0`: multiplica real e imaginário pela janela.
- `0x141164e90`: trocas e borboletas radix-2 DIT; por meia largura `l`,
  `(c, s) = (tab[m·N/(2l) + N/4], tab[m·N/(2l)])`, `tr = fma(s, im, c·re)`,
  `ti = fma(c, im, −s·re)`, `x[j] = x[j−l] − t`, `x[j−l] = t + x[j−l]`.

**Espectro** (`0x14117b1b0`, chamada `(obj, +0xaa0, taxa/8, bpm, s, fim,
grade +0x1140)`):

- `C = (fim − s + 1)/4`; buffer de `4·ceil((60/bpm)·fs) + 0x1001`
  amostras, zerado por compasso.
- Compasso `c`: `g0 = grade[s − 1 + 4(c − 1)]`, próximo `grade[s + 3 + 4(c −
  1)]` (último: `g0 + len_buffer`); copia o sinal de `g0` até
  `min(próximo − 1, len)`.
- Duas FFTs, a partir de 0 e de `grade[s + 1 + 4(c − 1)] − g0`, 4096
  amostras reais; linha `2(c − 1) + k` (contador iniciado em 2 em
  `0x14117b2fa`).
- 2048 bins: `20·log10(sqrt(fma(im, im, re²)))` se `>= 1e−10`
  (`0x143538100`, `0x1454155b0`), senão −200 (`0x143538108`).
  `0x1427a25f0` é `log10` (usa log10(e) e log10(2) em `0x143d0bfc0`,
  `0x143d0bff0`).
- Matriz `+0x1a70` (`2C × 2048`), copiada pelo chamador a `+0xe10`.

**Gêmeo por batida** (`0x14117ac80`): mesma rotina com a FFT de 2048 em
`+0x1a88` (init `0x14117ac40`), buffer `4·ceil(batida) + 0x801`, quatro FFTs
por compasso a partir de `grade[s − 1 + r] − g0` com `r = 4(c − 1) + k`
(linha `r`), 1024 bins, matriz `+0x1ac0` (`4C × 1024`), copiada a `+0xe28`.
No porte, os dois são `espectro_por_compasso` com `por_compasso` 2 e 4.

### Picos de graves e marcas por batida (`0x141188050`)

Portado em `vilelabox-dsp::passada_c::FiltroGraves`; conferido no assembly.
Chamada `(obj, +0xac0, bpm, _, fs = taxa/32, grade +0x1160, n, +0xb88)`.

- Banda → passa-baixa 2ª ordem 100 Hz (`+0x1188`) → valor absoluto → 1ª
  ordem 5 Hz (`+0x11c0`), de `0x141187f60`.
- Grade local `g[..n]` mais `g[n − 1] + (int)((60/bpm)·fs)`, **sem** limite
  no comprimento.
- Pico por batida (máximo, `0x141167280`; 0 fora do sinal) em `+0x11f8`
  (copiado a `+0xc00`; o último vira `+0xb88`).
- Pico por quarto: `q = (int)fma(g' − g, 0,25, 0,5)` (`0x145415208`),
  janelas `[g + kq, g + (k + 1)q − 1]`, a quarta até `g' − 1`; em `+0x1218`
  (copiado a `+0xc40`).
- Marca por batida em `+0x1238` (copiada a `+0xc20`), contra o pico anterior
  `p` (na primeira, `+0xb88`, pulada se 0): 0,5 se `v < p` e `2v < p`; 1,0
  se `v > p·1,77` (`0x143538350`); × 0,8 (`0x145415338`) quando `v` e `p`
  são `<= 0,09` (`0x143538238`).

### Ataques por batida (`0x14117e850`)

Portado em `vilelabox-dsp::passada_c::ataques_por_batida`; conferido no
assembly. Chamada `(obj, +0xa80, bpm, _, fs = taxa/2, grade +0x1120, n)`.

- `x = |banda|` (cópia, sem filtro).
- Seguidores rápido e lento a partir de `+0x1968`/`+0x1970` (0 em
  `0x14117e830`); coeficientes `2000/fs` ou `20/fs` (rápido) e `1000/fs` ou
  `13,33…/fs` (lento), o primeiro quando o seguidor está `<= x`.
- Duas aritméticas no mesmo laço: blocos de 4 com
  `fma(a, x, fma(−y, a, y))` (`vfnmadd231sd` + `vfmadd231sd`) e as
  `len mod 4` amostras finais com `(1 − a)·y + x·a` sem fusão.
- Ataque em `i + 1` quando rápido > lento e rápido > 0,06 (`0x143538178`);
  descartado se a distância ao último mantido não passar de
  `(13,33…/bpm)·fs` (`0x143538180`).
- Contagem `u8` por batida em `+0x1978` (copiada a `+0xdc0`):
  `g[j] <= pos < g[j + 1]`, e a última recebe `pos >= g[n − 1]`.

### Marcas por 16 avos de compasso (`0x14118d310`, `0x14118e450`)

As duas funções estão portadas em `vilelabox-dsp::passada_c` e conferidas
no assembly do 7.2.19 (2026-11-14):

- `0x14118d310` (`0x1412ec9d0`): `MarcasPorDezesseisAvos`;
- `0x14118e450` (`0x1412edb10`): `MarcasPorDezesseisAvosGraves`.

O mapa abaixo veio do decompilado do 7.2.8. A leitura do assembly e as
diferenças da gêmea vêm depois dele.

- Chamadas `(obj, +0xaa0, bpm, ms, s = +0xb98, fs = taxa/8, +0xb80, grade
  +0x1140, n)`. Saída: um byte por 16 avo de compasso, `+0x1948`
  (`0x14118d310`) e `+0x16d8` (`0x14118e450`), copiados pelo chamador a
  `+0xda0` e `+0xd80` a partir de `16·(int)((de − b94 + 1)·0x1435384e8)`.
- Estado entre lotes: última amostra filtrada (`+0x1930` / `+0x16c0`),
  posição (`+0x1938` / `+0x16c8`) e pico de referência (`+0x1940` /
  `+0x16d0`, reiniciado com `0x145414da0` quando `+0xb80 <= 1`).
- `0x14118d310`: passa-baixa 200 Hz de 4ª ou 2ª ordem conforme `fs`
  (`+0x1700`…`+0x17a8`, ordem em `+0x16f8`), valor absoluto, passa-baixa
  10 Hz (`+0x17e0`); depois passa-alta 110 Hz em quatro seções (`+0x1818`…
  `+0x18c0`), valor absoluto, 10 Hz (`+0x18f8`) e diferença de 1ª ordem com
  a amostra anterior do lote. `0x14118e450` usa só a primeira cadeia
  (`+0x15a8`…, ordem `+0x15a0`, 10 Hz em `+0x1688`).
- Batida: `(60/bpm)·fs`; 16 avo `(int)(batida·0,25)`; início
  `(int)(batida·(s − 1) + (int)(fs·ms·0,001 + 0,5) + 1)`; compassos
  `C = (n − s + 1)/4`, um a menos se o último não couber no sinal.
- Com `C >= 2`, busca de alinhamento em até 8 candidatos (limiares
  `0x1454151e0`, `0x143538398`, `0x1435383a8`, `0x143537ec0`,
  `0x143538128`), que pode deslocar o início.
- Por compasso: 16 máximos (4 batidas × 4 avos) em janelas de
  `(int)(batida·0,25)`; até 12 rodadas escolhem o maior; marca 1 no avo se
  ele passar `0x145414da0`, metade do primeiro máximo da rodada
  (`0x1454152a0`) e `ref·0x143538390`, e zera o valor. `ref` vira o máximo
  entre ele e o primeiro máximo do compasso.

**Leitura do assembly (7.2.19).** Os endereços abaixo são do 7.2.19.

- Argumentos: `RCX` = objeto, `RDX` = banda, `XMM2` = BPM, `R9D` = `ms`
  inteiro. Na pilha: `s`, `fs`, `lote` (`+0xb80`), grade e `n`. O chamador
  (`0x1412f5c70`) confirma a ordem. A init (`0x1412ec630`) é chamada pela da
  B com estado `(0, 0, 1e−4)` e `fs = obj+0x98`.
- Filtros: idênticos ao banco `FiltrosBanda` de `0x14118be60`. São o
  passa-baixa de 200 Hz (4ª ordem em `[s0, s1, s0, s1]` ou 2ª, pela mesma
  tabela de taxas), 10 Hz em `+0x17e0`, passa-alta de 110 Hz em
  `[h0, h1, h0, h1]` e a cópia do 10 Hz em `+0x18f8`. Com outra taxa, o
  buffer de 200 Hz fica sem escrever; o porte devolve `None`.
- `ref` = `+0x1940` com `lote > 1`, senão 1e−4 (`vpcmpgtd` + `vblendvpd`).
  `+0x1938` é zerado na entrada.
- `u = (int)fma(batida, s − 1, (int)fma(ms·0,001, fs, 0,5) + 1)`.
  `limite = g[n−1] − 1 + (int)batida`, ou `g[n−1]` se passar do comprimento.
  `C = (n − s + 1)/4` truncado, menos um se `barI·C − 1 + u > len`, com
  `barI = (int)(4·batida)`.
- `C <= 0`: saída de um byte 0. `+0x1930` = último valor do envelope de
  **200 Hz** (o outro caminho grava o de 110 Hz). `+0x1938` restaurado; o
  passa-alta não roda.
- Busca de alinhamento (`C >= 2`): ver `deslocamento_de_alinhamento`. Os
  argumentos perdidos pelo Ghidra são estes. `0x1412c6110(janela, b + c + u
  − 2, barI + 1)` empresta a janela do mínimo. `0x1412c6820(janela)` é o
  mínimo. `0x1412c67a0(seg, &idx[k], 1)` devolve o máximo e o índice base 1.
  O laço "descendo" do decompilado nunca roda: o teste exige quatro
  amostras não decrescentes, então `R9D` é sempre 1. Limiares: 0,2,
  0,595662, 1,25·batida, 1,12, batida/16 quietas e 2,11. O deslocamento é
  `min((int)(c − j + 0,5), u)`, e vale 0 sem decisão ou com vale 0.
- O deslocamento novo vai a `+0x1938`. Com `lote > 1`, o recorte usa o
  valor da entrada (`cmovg`).
- Diferença: `d[i] = e[i] − e[i−1]`, `d[0] = e[0] − +0x1930` antigo.
  `+0x1930` novo = `e[len − 1]` antes da diferença.
- Recorte e marcação: conforme `MarcasPorDezesseisAvos::processar`. A
  janela começa em `(int)((e·batida/4 + (g − u)) + 0,5)`, sem fusão, e tem
  `w` amostras. As 12 rodadas usam o primeiro máximo por `>` estrito; a
  referência final é `vmaxsd(primeiro, ref)`.
- Limitação: o porte não tem fixture do programa de referência. Os testes
  cobrem a busca com envelope feito à mão e as marcas com rajadas
  sintéticas.

**Gêmea `0x14118e450` (7.2.19: `0x1412edb10`).** O assembly tem o mesmo
molde, com estas diferenças:

- Há uma cadeia só: 200 Hz (`+0x15a8`…, ordem em `+0x15a0`) → `|·|` → 10 Hz
  (`+0x1688`), no próprio buffer. O mesmo envelope serve à busca e à
  diferença.
- `+0x16c8` não é zerado na entrada.
- A busca também aceita uma descida estrita `v0 > x0 > x1 > x2`. Nesse caso o
  pico é procurado para trás: `b[c+u−3−r] < b[c+u−2−r]`, com `pos = c − r`.
  O laço do vale e os limiares são idênticos.
- A marcação tem 6 rodadas (`R11D = 6`) e fator 0,596 (`0x143826760`), em
  vez de 12 e 0,5.
- A init `0x14118e120` (`0x1412ed7e0`) recebe da init da B o estado
  `(0, 0, 1e−4, fs = obj+0x98)`.

### Cópia das características (`0x141197890`; 7.2.19: `0x1412f6f50`)

Não há cálculo. Com `+0xb40 == 2`, copia as saídas acumuladas da passada C
para os campos de entrada do fechamento e libera os buffers de trabalho.
São duas chamadas em `0x1412f5c70`, com o `ms` dos extratores (`R15D`) como
3º ou como 2º argumento. O porte não precisa de código; o mapa abaixo
define a entrada do fechamento.

| Destino | Origem (extrator) |
| --- | --- |
| `+0xe58` | `+0xba0` (`energia_por_quarto`) |
| `+0xe78`, `+0xea0` | `+0xbc0`, `+0xbe0` (`energia_e_pico_por_quarto`) |
| `+0xe98`, `+0x1070` | 2º argumento |
| `+0xec0`, `+0xf20`, `+0xfc0` | `+0xc00`, `+0xc20`, `+0xc40` (`FiltroGraves`) |
| `+0xee0` | 3º argumento |
| `+0xee8` | `+0xe28` (espectro por batida) |
| `+0xf00` | `+0xc60` (`PicoMediosPorBatida`) |
| `+0xf40` | `+0xdc0` (`ataques_por_batida`, `u8`) |
| `+0xf60` | `+0xd80` (`MarcasPorDezesseisAvosGraves`, `u8`) |
| `+0xf80` | `+0xca0` (`EnvelopePorBatida`) |
| `+0xfa0` | `+0xda0` (`MarcasPorDezesseisAvos`, `u8`) |
| `+0xfe0` | `+0xcc0` (`AgudosPorBatida`) |
| `+0x1000`, `+0x1020` | `+0xce0`, `+0xd00` (`Bandas500PorBatida`) |
| `+0x1040` | `+0xe40` (matriz; origem DESCONHECIDA) |
| `+0x1058` | `+0xe10` (espectro por compasso) |
| `+0x1078` | `+0xd20` (`EnergiaPorQuarto40`) |
| `+0x1098`, `+0x10b0`, `+0x10b4` | `+0xdf8`, `+0xde8`, `+0xdec` (tonalidade) |
| `+0x10b8` | `+0xd40` (`pico_por_batida`) |
| `+0x10d8` | `+0xd60` (`PicoAgudosPorOitavo`) |
| `+0x10f8` | `+0xdf0` |
| `+0x1100` | `+0x1140` (grade de taxa/8) |

## Fechamento das passadas C/D (`0x141197d60`; 7.2.19: `0x1412f7420`) — mapa

Função de 23 378 bytes (3 841 instruções) com 79 callees: 4 já portados,
1 citado e 73 nunca abertos. **Não estima o BPM.** `+0xb0` é lido uma vez
(`0x1412f7834`) e nunca gravado. O resultado é validado campo a campo com
mensagens `!result->…`: `structure_name`, `beat_4`, `moriage`,
`moriage_pos_judge`, `moriage_pos_{4,8,16}_pos_start`, `aori`, `fill_in` e
`fill_in_pos_start`. É a análise de **estrutura de frases** (fase A2.3 do
`PLANO-ANALISE.md`), não de BPM. INFERIDO: `moriage` (盛り上げ) e `aori`
(煽り) são subida de energia e preparação.

- Há dois ramos, escolhidos por um campo local (`local_23c != 1`). Um
  delega a `0x141211610(…, obj+0x68)` e preenche o resultado. O outro roda
  a cadeia própria de `0x1412dff80`, `0x1412e7460`, `0x1412e8de0`,
  `0x1412e93f0`, `0x1412d0aa0` e outras.
- Cadeia completa: `third_party/rekordbox/re/export/p4-7.2.19/decomp/1412f7420.c` e `disasm/` (privados, fora do Git). Abrir
  por ramo, a começar pelo próprio.

## Filtros do envelope (`0x14118aea0`, `0x141188560`)

Projetados à taxa decimada `fs = taxa/8`; portados em `FiltrosEnvelope`.

- Passa-baixa de 200 Hz (`ω` = 1256,637 = 2π·200, `0x1434e55f0`) em
  `+0x280`, ordem gravada em `+0x360`:
  - 4ª ordem (duas seções, `a1 = 2·cos(i·π/4 + π/8)·Ω`, copiadas para
    `+0x2f0`) quando `fs` está a ±0,01 de 4000, 5512,5 ou 6000, isto é, áudio
    a 32, 44,1 ou 48 kHz;
  - 2ª ordem (`a1 = √2·Ω`, `0x1454153e8`) perto de 1000, 1378,125 ou 1500,
    isto é, 8, 11,025 ou 12 kHz;
  - nenhum filtro nas demais taxas.
- Passa-baixa de 10 Hz de 2ª ordem em `+0x368` (`0x143538160` = 2π·10).
- Passa-baixa de 100 Hz de 2ª ordem em `+0x3a0` (`0x1434e55e8` = 2π·100).
- `0x14118aea0` também grava seus argumentos em `+0x268`, `+0x270` e
  `+0x278`; o fechamento da passada A passa 0, 0 e 0,0001 (`0x145414da0`).

## Envelope e marcação de picos (`0x14118b160`)

Argumentos vistos no fechamento da passada A: sinal decimado, BPM
(`motor+0xb0`), deslocamento em ms (`motor+0xac`), 1, a taxa decimada e um
vetor de saída de bytes.

- **OBSERVADO, portado** (`FiltrosEnvelope::envelope` e `diferenca`): as
  seções de 200 Hz em sequência, valor absoluto (`0x141167080`, máscara
  `0x7fff…`), passa-baixa de 10 Hz e diferença de primeira ordem com
  `d[0] = y[0] − motor+0x268`. Com ordem 0 o original não copia a entrada.
- **Portado em `marcar_picos`**, lido no assembly (`0x14118b160`–`0x14118be5x`).
  A decompilação escondia FMAs: batida `p = (int)fma(60/bpm, fs, 0,5)`,
  início `(int)fma(ms·0,001, fs, 0,5)` e base do grupo `fma(g, 4p, 1)`.
  Antes da grade, o sinal é diferenciado e, se o alinhamento achou
  deslocamento, recortado ou empurrado com zeros à frente. O sinal é varrido
  em grupos de quatro batidas; em cada grupo, 16
  janelas de `round(p/4)` amostras dão o máximo (`0x141167280` =
  `max_element`). Até seis vezes por grupo, o maior dos 16 é marcado com 1 no
  vetor de saída (16 bytes por grupo) e zerado, se for maior que 0,0001, que
  0,596 × o primeiro máximo do grupo (`0x1435383a0`) e que 0,178 × o máximo
  corrente (`0x143538390`). O máximo corrente fica em `motor+0x278`.
- **Alinhamento inicial** (`0x14118b4a9`–`0x14118b834`, com dois grupos ou
  mais), portado: em até 8 voltas, toma o máximo de uma cópia do envelope a
  partir do 2º compasso (zerando o máximo anterior), alinha-o ao compasso
  seguinte, exige valor acima de 0,2 × máximo, escolhe subida ou descida pelos
  vizinhos, caminha até a inversão (até um quarto), e com o ponto acima de
  0,595662 × máximo (`0x143538398`) procura para trás um vale abaixo da média
  entre máximo e mínimo da janela, sustentado por `(int)(p/4 · 0,25)`
  amostras sem passar de 1,12 × o vale (`0x143537ec0`). Vale nulo dá
  deslocamento 0; ponto acima de 2,11 × vale (`0x1435383a8`) dá
  `(int)((candidato − vale) + 0,5)`, limitado ao primeiro índice. Sem
  inversão, `r13` e `xmm12` mantêm o valor da volta anterior (zerados no
  início), como no original.
- `0x1411672c0` é `max_element` com índice 1-based e `0x141167340` é
  `min_element` (`0x142604de0`, `0x142604f60`; STL vetorizada, primeira
  ocorrência).

## Estágios da passada A (A1.1c, segunda rodada)

Portados em `vilelabox-dsp::estagios`; ordem das operações conferida no
assembly com `desmontar.py` (capstone, privado).

- **Energia** (`0x1411671d0`): soma dos quadrados. A decompilação perde o
  corpo; o assembly mostra blocos de 8 com `vfmadd231pd` em dois acumuladores
  (`ymm1` posições 0–3, `ymm2` 4–7), redução `(s2 + s3) + (s0 + s1)` com
  `s = ymm2 + ymm1`, e cauda `vmulsd` + `vaddsd` sem fusão.
- **Filtro de 1ª ordem** (`0x141165e30` projeto, `0x141166430` processo):
  bilinear de `(b1·s + b0)/(a1·s + a0)` e forma direta II transposta.
- **Energia por quarto de batida** (`0x1411885f0`): sinal pelo passa-baixa de
  100 Hz (`+0x3a0`); por batida da grade, três janelas de
  `(int)(fs·(1/bpm)·15 + 0,5)` amostras (`0x145415578` = 15) a partir do
  índice 1-based e a quarta até a batida seguinte, ou até
  `início + (int)(fs·(1/bpm)·60 + 0,5) − 1` nas duas últimas batidas. Fora de
  `5512,45 < fs < 5512,55` (`0x143538358`, `0x143538360`) todos os valores
  são multiplicados por 4 (`0x1454154c0`). Resultado em `+0x3d8`; compassos
  (16 valores, `n/4` grupos) em `+0x3f8`, somados na árvore do assembly
  `0x141188860`–`0x1411888e6`.
- **Onset de agudos** (`0x141178330`): argumentos confirmados no assembly
  (convenção x64 da Microsoft). Passa-alta Butterworth de 300 Hz
  (`0x1434e5600`), numerador `(1, 0, 0)`, `a1 = 2·cos(i·π/4 + π/8)·Ω`, duas
  seções duplicadas; valor absoluto; passa-baixa de 1ª ordem
  `Ω/(s + Ω)` em 5 Hz (`0x1435380c8`); diferença com `d[0] = 0`. Taxa 0 vira
  5512,5. Resultado em `+0x418`.

- **Soma** (`0x141167150` → `0x14116ef60`): blocos de 8 em dois
  acumuladores, `s = a1 + a0`, redução `(s2 + s3) + (s0 + s1)` somada a 0 e
  cauda escalar.
- **Padrão de picos** (`0x141176680`), portado em `classificar_padrao`:
  compasso inicial = primeiro com energia acima da média, procurando do 17º
  (ou do 1º com menos de 17 compassos) até o antepenúltimo; compassos
  disponíveis `−(int)(bytes · −0,063) − inicial` (`0x143538048`); até 8
  blocos de 8 compassos, contando só as posições 2, 3, 6 e 7 de cada bloco;
  um compasso conta para um molde quando os 16 bytes batem. Moldes:
  `1000×4` ou `0010×4`, `1000 0010 1000 0000`, `1000 0000 1000 0000` e
  `1000 0000 1010 0000`. Decisão com `t = disponíveis > 63`: dois ou mais
  blocos com o primeiro molde ligam o 5º argumento; senão o último molde em
  mais de `t` blocos liga o 8º; senão o segundo, abaixo de 165 BPM
  (`0x143538040`), liga o 6º; senão o terceiro em mais de `t` blocos liga o
  7º. O uso desses sinalizadores adiante é DESCONHECIDO.

- **Ordenações** (`0x1411678b0`, `0x141168940`): pares (valor, índice
  1-based). Valores `f64` em ordem decrescente e estável (inserção em
  `0x14116e4c0` move enquanto `valor > anterior`); inteiros em ordem crescente
  e estável (`0x14116e630`, `jl`). Acima de 32 elementos, merge sort estável
  com o mesmo comparador.
- **Picos de agudos** (`0x1411786a0`), portado em `picos_de_agudos`: para os
  `grade/8` primeiros compassos (passo de 4 batidas), o onset de agudos do
  compasso é ordenado; o candidato `k` é aceito quando
  `v[k+1] < 0,9886·v[k]` (`0x1435380b8`) e seu índice não está a
  `±((int)(fs·15/bpm + 0,5) − 20)` dos primeiros `aceitos` índices da lista
  ordenada; para quando não há queda em `k` nem em `k+1`. Capacidade de
  8 × compassos. Em seguida, em ordem crescente, picos sem outro a duas
  batidas, com tolerância `(int)(fs·1,875/bpm + 0,5)` (`0x1435380c0`), são
  zerados em sequência; nova ordenação; os válidos perdem
  `(int)(ms·(fs·0,001) + 0,5)` nos blocos de 16 do laço vetorial e
  `(int)((fs·ms)·0,001 + 0,5)` no resto. BPM 0 vira 120 (`0x145415678`) e
  taxa 0 vira 5512,5. Saídas: posições, quantidade de válidos e primeiro
  válido 1-based. O vetor `f64` paralelo (`param_10`) é alocado zerado e só
  recebe zeros.

- **Confirmação de picos** (`0x141179530`), portado em `confirmar_picos`:
  repete a seleção sobre as listas ordenadas guardadas por `0x1411786a0`, com
  vizinhança `±((int)(batida·0,25 + 0,5) − 20)` contra os picos já aceitos de
  todos os compassos e teste com `índice + início`. Em ordem crescente e em
  sequência: sem par a duas batidas (tolerância `(int)(batida·0,125 + 0,5)`,
  `0x145415198`) o pico sai da lista interna; com par, sai se as marcas de
  `0x14118b160` em volta do seu quarto de batida não formarem `(−4, +2)`,
  `(−6, +5)` ou `(−4, +4)` ou se o próprio quarto estiver marcado (só quando
  `5 < quarto < marcas − 5`). A lista de saída (`param_12`, `param_13`) só
  perde os que também não têm par a quatro batidas (tolerância `−¼`, `+⅛`
  de batida). Sem o sinalizador de semínimas de `0x141176680`, a saída perde
  os picos cujo quarto `(p − desloc − 1) / quarto` está marcado. Saída
  ordenada, deslocada por `(int)((fs·ms)·0,001 + 0,5)`, com quantidade de
  válidos e primeiro válido.

- **Picos finos** (`0x141178df0`), portado em `picos_finos`: mesma seleção
  da confirmação, com capacidade de 16 por compasso e razão de queda
  0,998849369936505 (`0x1435380d8`). Em sequência, um pico fica só se tiver
  par a duas batidas e, com `6 <= quarto < marcas − 5`, se as marcas formarem
  `(−4, +2)`, `(−6, +5)` ou `(−4, +4)` sem marca no próprio quarto. O vetor
  `f64` de saída é alocado zerado e só recebe zeros. Deslocamento final como
  em `0x1411786a0`.

- **Auxiliares de `0x141172490`**: `0x141170790` compara bytes (como
  `memcmp`); `0x1411686b0` é `max_element` de inteiros (`0x142604d40`, guarda
  o primeiro máximo, `cmovl`); `0x141168630` soma inteiros (`0x140f199b0`);
  `0x1411890e0` projeta passa-baixa Butterworth de 2ª ordem com corte em Hz
  em `motor+0x438` (portado em `filtro::passa_baixa_hz`); `0x141189170` é a
  energia por quarto em grade fixa a partir do deslocamento, índices 0-based,
  sempre × 4 (portado em `energia_por_quarto_fixa`).

## Fase do compasso (`0x141172490`)

**Portado em `fase_do_compasso`**, com o fluxo conferido no assembly
(`0x141172490`–`0x141173272`): a decompilação mostrava saltos para dentro de
laços e reaproveitamento de registradores. Pontos que só o assembly mostrou:
o contador de 12 da varredura em blocos herda o valor do laço de 16; os
recuos de 500 e 1000 usam FMA (`vfmadd213sd`); o sinalizador do resto 3 parte
de `r12b` zerado em `0x141172807`; o limiar de metade do máximo é 0,5
(`xmm12`).

Entradas: marcas de pico (16 por compasso), BPM, deslocamento em ms (`p5`),
a lista de `picos_de_agudos` (`p10`, com quantidade `p11` e primeiro válido
`p12`). Saídas: novo deslocamento em ms (`p13`), modo de correção (`p14`:
0, 1, 2 ou 3), sinalizador de molde (`p15`) e sinalizador de deslocamento
mantido (`p16`).

1. Com menos de dois blocos de 8 batidas ou `(int)(bytes · 0,03125) < 2`
   (menos de 64 bytes de marcas, 4 compassos), devolve o deslocamento de
   entrada e modo 0.
2. Varre cada byte de marca `i` e anota:
   - `A`: os 7 bytes desde `i` são `1000001` (`0x143537ec8`) e os 9 desde
     `i+7` são zero (`0x143537ed0`);
   - `B`: os 7 bytes desde `i` são `1000001` e os 2 desde `i+8` são zero
     (`0x143537edc`), independente de `A`;
   - `C`: os 24 desde `i` são `100000101000000010100010`
     (`0x143537ee0`);
   - `D`: `m[i] = 1`, `m[i+1] = 0`, `m[i+3]` ou `m[i+7]` = 1 e `m[i+6]` ou
     `m[i+10]` = 1.
   `A` em qualquer posição liga `p15`.
3. Escolha da lista de referência: `C` com 5 ou mais ocorrências usa as
   posições de `C` e o limiar 0,2; senão `A` com 5 ou mais usa `A` e 0,4;
   senão, com 16 ou menos ocorrências de `D`, usa a tabela fixa 128, 144, …,
   496 (`0x143537f00`, 24 valores); senão calcula `(D + 1) mod 4`, toma o
   máximo, conta quantos são iguais a ele e, se o máximo é 3 e a fração
   passa 0,65 (até 25 ocorrências), 0,41 (26 a 60) ou 0,30 (61 ou mais)
   (`0x143537f80`, `0x143537f68`, `0x143537f60`), marca correção. `B` com
   5 ou mais, sem as anteriores, usa `B` e 0,45.
4. Período `(int)(60/bpm · 250)` ms, um quarto de batida (`0x1454156e8`), ou
   125 com BPM nulo; o deslocamento é reduzido por esse período enquanto o
   passar.
5. Energia por quarto em grade fixa com passa-baixa de 400 Hz
   (`0x143537f88`) a partir do deslocamento reduzido e com 100 Hz
   (`0x145415668`) a partir do original; análise dos máximos com limiares
   0,48 (`0x145415290`), 0,6 (`0x143537f70`), 0,605 (`0x143537f78`) e 70
   (`0x145415640`), em blocos de 64 quartos e contagens de 12 e 16.
6. Com lista de referência, conta as posições de `p10` além de `ref · ¼
   batida`, sem repetir, e mede a fração cujo resto pela batida fica entre
   `¼` e `0,75` (`0x145415328`) da batida.
7. Decisão: deslocamento somado de `k` períodos (modo 2 ou 3), ou recuo de
   250 ms (`p5 − 0xfa`) ou de `60/bpm · 500` com retorno de
   `60/bpm · 1000` se negativo (`0x145415730`, `0x1454157b8`) (modo 1).

## Banda de 110 a 200 Hz (`0x14118be60`, `0x14118c1b0`)

- **Filtros** (`0x14118be60`), portados em `filtro::FiltrosBanda`: o mesmo
  passa-baixa de 200 Hz (ordem em `+0x5d8`) e de 10 Hz (`+0x5e0`) de
  `0x14118aea0`, um passa-alta Butterworth de 110 Hz (`0x143538410`,
  691,15 = 2π·110) em duas seções duplicadas (`+0x618`–`+0x6c0`) e uma cópia
  do passa-baixa de 10 Hz (`+0x6f8`). Estado em `+0x4e0`, `+0x4e8`, `+0x4f0`.
- **Marcação** (`0x14118c1b0`), portada em `marcar_picos_banda`: comparada
  instrução a instrução com `0x14118b160` (diferença normalizada dos
  mnemônicos). O alinhamento usa `|200 Hz| → 10 Hz` e só o caso de subida
  (`r14d = 1` fixo; o de descida é inalcançável); a grade usa
  `|110 Hz(200 Hz)| → 10 Hz`, com o passa-alta aplicado no lugar sobre a
  saída de 200 Hz; até 12 rodadas por grupo (`cmp edx, 0xc`); a última
  amostra do envelope da banda é gravada em `+0x4e0` a cada chamada.

## Padrão entre as duas bandas (`0x141177b30`)

Portado em `padrao_duas_bandas`. Com as marcas da banda larga (`a`) e da
banda de 110 a 200 Hz (`b`), para cada `i` de 0 a `len(a) − 33`: ambas em
`i`; vazias em `i+2..=i+6`; ao menos uma posição de `i+7..=i+9` só em `b`;
vazias em `i+10..=i+14`; ao menos uma de `i+15..=i+17` em ambas. Uma
ocorrência conta quando a anterior não contada ficou 15 a 17 posições antes
(a referência guardada é `anterior + 17` e começa em 0, então uma ocorrência
em `i <= 2` já conta). Mais de uma contada liga o 5º argumento
(`local_204`).

## Início audível e número da batida (`0x141171760`, `0x1411718e0`)

- **Início audível** (`0x141171760`), portado em `primeira_batida_audivel`:
  sobre os primeiros 30 s do sinal da passada A em valor absoluto, com
  `início = (int)fma(ms·0,001, fs, 0,5)` (a decompilação mostrava outra
  ordem) e `batida = (int)fma(60/bpm, fs, 0,5)` (0,5 com BPM nulo), acha o
  primeiro `k` com amostra acima de 0,0005 (`0x143537eb0`) e devolve
  `(int)(k/batida + 0,5) + 1` (fator 1/2756,25 com batida nula).
- **Número da batida** (`0x1411718e0`), portado em `numero_da_batida`:
  desloca os picos de `picos_finos` pela diferença de deslocamento (blocos
  vetoriais `d·(fs·0,001)`, resto `fma(d·0,001, fs, pos)`), classifica pelo
  resto num ciclo de duas batidas (`batida = (int)fma(60/bpm, fs, 0,5)` com
  BPM 0 → 0,1, `q = (int)fma(batida, 0,25, 0,5)`), escolhe a paridade
  (até 6 picos: `(motor+0xbc − 1) mod 2`; razão acima de 0,886
  (`0x143537eb8`): 1; senão 0), mede a energia por quarto com passa-baixa de
  100 Hz (`motor+0x730`, `0x141188950`/`0x1411889e0`, fator 4 sempre) com
  limiar `fma(média, 0,6, máximo·0,4)` e liga a segunda metade quando o
  primeiro quarto forte cai nas posições 5 a 8 de 16. O número final é
  `1 + 2·segunda_metade + paridade`, salvo a exceção pelo modo da fase.

## Orquestração da passada A (`0x141195480`)

Portada em `fechamento_passada_a`, com a grade `0x141173280` em
`grade_de_indices` (início `(int)fma(ms·0,001, fs, 0,5)` no assembly).
Ordem: grade; filtros de envelope e marcas; passa-baixa de 100 Hz e
energia por quarto; padrão rítmico; onset de agudos; três rodadas de picos;
fase do compasso; com `motor+0xca`, o deslocamento de partida é mantido e o
modo zerado; nova grade e marcas com o deslocamento corrigido; banda de 110 a
200 Hz e padrão entre bandas; início audível nos primeiros 30 s; número da
batida (ou `motor+0xbc` quando não há picos finos, com `motor+0xcb` ou com o
deslocamento mantido pela fase). Saídas em `motor+0x208`–`+0x260`.

**A passada A recebe o BPM pronto** (`motor+0xb0`, vindo do BPM já conhecido
da faixa no init). A hipótese de que o BPM seria estimado nas passadas B e
C/D foi **refutada** em 2026-11-14 (ver "Fechamento das passadas C/D"):
nenhuma delas grava `+0xb0`.

## Cadeia do fechamento da passada A (`0x141195480`)

O sinal decimado (até 210 s) passa, nesta ordem, por: `0x141173280` (grade
de índices: passo `(60/bpm)·fs`, início `fs · deslocamento_ms · 0,001 + 0,5`,
índice `i·passo + início + 1,5`, último limitado ao fim), `0x14118aea0`
(biquads auxiliares com cortes dependentes da taxa), `0x14118b160`,
`0x141188560`, `0x1411885f0`, `0x141176680`, `0x141178330`, `0x1411786a0`,
`0x141179530`, `0x141178df0`, `0x141172490`, nova grade, `0x14118be60`,
`0x14118c1b0`, `0x141177b30`, `0x141171760` (sobre 30 s) e, quando aplicável,
`0x1411718e0`. Resultados em `+0x208` (deslocamento final em ms, copiado de `+0xac`), `+0x20c` (número da batida, de `+0xbc`), `+0x210`,
`+0x218`, `+0x238` e `+0x258`..`+0x260`.

O BPM, o deslocamento e o tom de partida vêm da thread, que os lê dos dados
já conhecidos da faixa (vtable de `thread+0x2e0`, slots `+0xa0`, `+0xa8`,
`+0xb0`). Sem dado prévio o BPM é 0, que o init aceita.

## Estimador de BPM: `BeatAnalyzer_1_0` (motor 1) — mapa (7.2.19, 2026-11-14)

O BPM é estimado por uma biblioteca ligada estaticamente, com RTTI própria.
Há duas versões, nos namespaces `BeatAnalyzer_1_0` (motor 1, faixa
`0x142600000`–`0x14261d000`) e `BeatAnalyzer_2_0` (motor 2). As classes com
vtable são: `Analyzer`, `PreChecker`, `peakss::BeatSensorSignal`,
`peakss::BeatListSignal`, `peakss::BpmCountTbl`, `UnitBeatAdjust`,
`SimpleEnvelopeMaker`, `UnitEnvelopeMaker`, `LoudViewMaker`/`LVMakeBase`,
`ZoomViewMaker`, `balib::IirFilter`, `IirPassFilter`, `LowPathFilter2D`,
`HighPathFilter1D` e `HighPathFilter2D`. As vtables quase só têm
destrutores; o cálculo está em métodos não virtuais. Todos os endereços
desta seção são do 7.2.19.

**Envelope `analyzer::BALibWrapper`** (18 bytes; motor em `+8`, tipo em
`+0x14`):

- A thread `CBeatAnalyzeThread` (`0x141213840`) constrói o envelope com
  tipo 1 (`0x1411db2e0`). Troca para o tipo 2 (`0x1411db330`) quando o modo
  em `+0x2c58` é 2, 3 ou 4, os mesmos modos que `Detect` manda para a alta
  precisão.
- `0x1411db350` cria o motor. São quatro fábricas, escolhidas por uma flag e
  por `taxa == 44 100`; no motor 1: `0x14260b590`, `0x14260b640`,
  `0x14260b7a0` e `0x14260b850`, que usa `0x142606670` e `0x142606880`.
- `0x1411db540` vai por salto de cauda a `0x142606e40`, a alimentação.
  `0x1411db590` vai a `0x142607460` → `0x14262a830`. `0x1411dbac0` destrói o
  motor.
- `legacyBeatAnalyze` lê o BPM com `0x1411db6c0`:
  `(int)(motor+0x48 · 100 + 0,5)`. Lê a quantidade de batidas com
  `0x1411db6f0` (vetor de 24 bytes em `motor+0x50`). Converte a lista com
  `0x1411db720` → `0x14260ba80`: registros `(posição, número,
  (int)(6 000 000/intervalo + 0,5))`.
- O callgraph não vê as chamadas por salto de cauda nem as virtuais. Os
  alvos acima foram lidos no assembly.

**Alimentação `0x142606e40(motor, L, R, n)`** (`Analyzer`):

- Mistura em mono: `(L + R)·1,5258789e−5` (`0x1457fed94` = 0,5/32768), em
  `f32` em `+0x30`. Guarda também o estéreo intercalado em `i16` em `+0x38`.
  O bloco tem `+0x18` frames e o total esperado é `+0x1c`.
- A cada bloco cheio, roda os processadores conforme as flags `+0xa`–`+0xd`:
  - `0x1426080d0(+200, mono)`: sensor de batidas; a lista fica em `+0xb0`;
  - a vista de volume, por chamada virtual em `+0x678`;
  - `0x142618f80(+0x558, i16)`;
  - `0x142619ea0(+0x5a0, i16)`: envelope;
  - `0x142609260(+0x608, i16)`, salvo com `+0x2c`;
  - `0x14261c8e0(+0x3998, i16)`: vista de zoom.
- Ao atingir `+0x1c` frames, chama o fechamento `0x142607110`.

**Fechamento `0x142607110`** — aqui o BPM é decidido:

1. Copia a lista do sensor (`+0xb0`, registros de 24 bytes).
2. `0x142613100(tabela, {+200, lista, +0x14}, +0x50, total, 4, faixa…,
   +0x2c)` usa a `BpmCountTbl` (`0x1426130e0`). Devolve o BPM em `XMM0`,
   gravado como `f32` em `+0x48` (`0x14260722e`).
3. Com `+0xa`, `0x142619120(+0x558, bpm, +0x50)` corrige o BPM.
4. `0x14261a080(+0x5a0, bpm, +0x50)` refina a lista de batidas.
5. Sem `+0x2c`, `0x142609560(+0x608, &bpm, +0x50)` faz o ajuste final.

Tamanho: a subárvore da alimentação tem 45 funções e cerca de 52 KB. As
maiores são `0x142613100` (15 032 bytes), `0x14261a080` (8 943),
`0x1426080d0` (2 312) e `0x142619120` (2 018).

### Porte do estimador — etapa 1 (2026-11-14)

Portado em `vilelabox-dsp::balib` e conferido no assembly, exceto onde
indicado:

- **Alimentação `0x142606e40`** (`Alimentador`, conferida no decompilado).
  O bloco tem `taxa/75` frames (588 a 44,1 kHz). O último bloco **não** é
  cortado no total de frames: enche até 588 e só então fecha. A init
  (`0x142606880`) aceita só 44 100 e 48 000 Hz e liga flush-to-zero
  (`MXCSR | 0x8000`). O porte não emula o flush-to-zero (ADAPTAÇÃO).
- **Banco do sensor**: `0x142608000` chama `0x142612650` com 6 bandas
  (`[rsp+0x28] = 6`) e média de 8 amostras (`[rsp+0x20] = 8`). As bandas
  estão em `BancoDeBandas`.
  - Os biquads seguem as receitas RBJ, com `w = (f·6,28318530717958)/fs`.
    A constante `0x143ea0760` é 2π truncado. `f`, `fs` e `Q` são `f32`. A
    init usa `sincos` da CRT (`0x14280e240`) e `sinh` (`0x1429b5400`).
  - Passa-baixa (`0x14260c280`) e passa-alta (`0x14260c8b0`/`0x14260cb40`)
    usam `a0 = fma(k, s, 1)` e `1 − s·k` fundido. O passa-faixa
    (`0x14260cfa0`) não usa FMA.
  - A seção (`0x14260c520`) é de forma direta I com FMA, na ordem `b0, b1,
    b2, −a1, −a2`. Há duas exceções que começam por `b1·x[n−1]`: a amostra
    1 e a terceira de cada grupo de 4 do laço desenrolado. Por isso o
    resultado depende do tamanho do bloco.
  - O aplicador `0x14260d2d0` converte `f32` → `f64`, alterna dois buffers
    entre as seções e volta com `vcvtpd2ps`. O sensor tira o valor
    absoluto (`& 0x7fffffff`).
  - As bandas 4 e 5 copiam as bandas 0 e 3 (`+0x54`) e aplicam
    `y = max(x − média, 0)`. A janela tem `(int)(taxa·ms·0,001 + 0,5)`
    amostras (44 e 9 a 44,1 kHz).

### Sensor — etapa 2 (portada)

Portados em `vilelabox-dsp::balib` (`SeguidorDePico`,
`RegistroDeIntervalos::casar`/`agrupar` e `SensorDeBatidas`), conferidos no
assembly. Detalhes do assembly que o decompilado escondia:

- A janela de contagem vai de 0 a `+0x1c` do analisador (`R8D = 0`,
  `R9D = +0x1c` na chamada de `0x142608000`). A amostra que atinge o fim
  não entra.
- Na queda, `limiar − 0,5·pico` e `limiar − 0,5·limiar` usam
  `vfnmadd231ss`. A duração e o Δ do intervalo são convertidos como sem
  sinal.
- `0x142611f20` chama `0x1426120b0(reg, x, anel, 4,0, 20,0, início, &média,
  &erro, &contagem)` até devolver −1. Guarda o resultado de maior contagem
  com média `>= 333 ms`. O 20,0 da pilha não é usado: a tolerância lê
  `0x1457ffb70`.
- `0x1426120b0`: as regras estão em `RegistroDeIntervalos::casar`. A
  entrada mais antiga permitida é a de índice `quantidade − 2`, porque o
  teste `j + 1 >= quantidade` vem antes da leitura. A extensão lê o anel
  mesmo além da quantidade, com zeros no início.

Por amostra: com o contador `[0x16]` entre o início `[0x17]` e o fim
`[0x18]`, acumula cada banda. A cada 8 amostras (`[0xe]`) tira a média `v`
e roda o seguidor de pico da banda. O seguidor tem 0x78 bytes. O
construtor `0x142618950` grava piso `+0x14` = 0,001, `+0x20` = 0,75,
`+0x24` = `+0x28` = 0,5 e fonte `+0x54` = −1. A init da taxa grava `+4` =
0,29 s, `+0x2c` = 1,7 s e `+0x30` = `+0x34` = 1,3 s (em amostras, `f32`),
`+0x40` = 8 e `+0x50` = taxa.

- O limiar `+8` cai `+0x10` por passo. No estado 0, o piso é `+0x14`.
  Quando passam `+0x1c` amostras desde `+0x38`, a queda é recalculada:
  primeiro rumo a `0,5·pico` em `+0x30`, depois rumo a `0,5·limiar` em
  `+0x34`.
- `v >= limiar` abre um pico. O limiar vira `0,75·v` e o pico é guardado em
  `+0x44`, com a posição em `+0x48`/`+0x4c`.
- No estado 1, um valor maior renova o pico. O pico fecha depois de
  `+4` (0,29 s) sem renovar. Então o registro de intervalos da banda (0x40
  bytes, construtor `0x1426188a0`, anel de 32 entradas de 12 bytes) recebe
  `(posição, pico, intervalo em ms)`, com `intervalo = (1000/taxa)·Δ`.
- Cada intervalo passa por `0x142611f20` → `0x1426120b0`. Essas funções
  agrupam os intervalos recentes por razões de batida, com duas tabelas de
  trincas `(mín ms, máx ms, razão)` lidas do executável:
  - `0x143ea0b40`: (500, 1276, 1,5) (468, 1071, 1,25) (923, 2142, 2,5)
    (290, 638, 0,75) (320, 860, 1) (290, 428, 0,5) (666, 1714, 2)
    (1500, 3333, 4) (1125, 2580, 3);
  - `0x143ea0bb0`: a mesma lista em outra ordem, de 4 a 0,5 batida, e 3.
  - Constantes: 4,0, 20,0, 333,0, 0,04, 0,25, 0,5, 2,5 e `FLT_MAX`.
  - Lido no assembly em 2026-11-14; ver o início desta seção.
- **Escolha da banda `0x142608b20`** (`HistogramaBpm::escolher`, conferida
  no assembly):
  - O histograma tem 256 caixas inteiras de BPM em `sensor+0x64`, com
    mínimo e máximo usados em `+0x464`/`+0x468` (início 200/40; `60000` em
    `0x1458002e4`).
  - Cada banda válida vota em `(int)(60000/intervalo + 0,5)` com peso
    `max((int)(4/(1 + erro)), 1)`. A melhor banda é a de menor erro, e a
    moda é o primeiro máximo.
  - Se a melhor banda não estiver perto da moda, do dobro ou da metade (com
    `min(intervalo·0,04, 20)`; a metade usa `vfmsub` fundido) e sua caixa
    tiver ao menos 3 votos, as caixas incompatíveis são zeradas e a moda é
    recalculada. Na poda, a metade **não** é fundida. Com menos de 3 votos
    na caixa da melhor, não há escolha.
  - No fim, a moda precisa de ao menos 3 votos. As bandas incompatíveis
    com ela perdem a validade e a melhor é devolvida.
  - O sensor guarda o intervalo, a posição e o pico da escolhida em
    `[0x10]`–`[0x12]`, zera `[0x13]` (amostras desde a escolha) e chama o
    callback para cada banda ainda válida.
- **Lista `BeatListSignal::[1]` (`0x142607b60`)** (`ListaDeBatidas`,
  conferida no assembly):
  - Registros de 24 bytes: `(pos, quartos de batida, intervalo médio,
    etiqueta, banda)`. A etiqueta em `objeto+0x1c` soma `contagem + 2` na
    entrada (o byte baixo é a força) e, ao fim, zera a força e avança a
    geração (`+0x100`).
  - Grava as `contagem + 2` entradas do anel, da mais antiga para a mais
    recente, em ordem de posição. Na mesma posição exata, a de maior força
    fica quando os BPM×10 coincidem (`600000/iv`, dobro ou metade) ou
    quando os intervalos são incompatíveis. Nos outros casos a busca
    continua para trás.
  - Bandas diferentes marcam o mesmo clique com atrasos de filtro
    diferentes; essas entradas não são deduplicadas.

### Contagem `0x142613100` — mapa e porte parcial (2026-11-14)

Função de 3 519 instruções. Ela recebe `(param_1, {+200, lista}, saída =
motor+0x50, total = +0x1c, 4, 16, 12, flag = +0x2c)`, com os valores
padrão de `+0x28 = 32`. Devolve o BPM em `XMM0` e preenche a grade em
`motor+0x50`. O log de depuração (`fprintf` em `param_1+8`, `0x140f029c0`)
fica fora do porte.

- **Flag `+0x2c`.** As fábricas recebem `modo == 1 || modo == 4` (thread,
  `+0x2c58`). No motor 1, o modo 0 usa a flag 0 e o modo 1 usa a flag 1.
  Os dois ramos existem.
- **`BpmCountTbl`** (portada em `TabelaBpm`):
  - layout `{n (+8), caixas (+0x10), decimais (+0x18), escala = pow(10,
    d) (+0x20)}`;
  - contagem `0x142612e10`: caixa `(int)fma(600000/iv, escala, 5)/10`, com
    a guarda da primeira caixa sempre verdadeira no original;
  - refino `0x142612fb0`: busca `[(s·b − s)·10^−d, (s·b + s)·10^−d)` com
    FMA.
- **Modo 0, linhas 192–361** (portado em `estimar_modo0`):
  - tabelas de 400 e 4 000 caixas, com refino a `decimais = 0`;
  - `iv0 = (f32)(600000/caixa)`;
  - poda por `na_janela`, com janelas estritas e a metade sem FMA. Na
    primeira batida, o "anterior" é a tolerância;
  - o primeiro segmento é a primeira batida com `intervalo = iv0`.
- **Trecho comum, refino por segmento, linhas 817–950** (portado em
  `refinar_segmentos`): reconta cada segmento até o próximo. Se o novo
  intervalo cai na janela do anterior, funde os dois e reconta o anterior.
- **Deduplicação, linhas 996–1104** (portada em `deduplicar`):
  `força[geração]`, distância `(int)(((f32)taxa·iv)·1,25e−4)`
  (`0x143ea0c30`), uma passada só pela lista. Sai a batida de menor força;
  em empate, a de trás.
- **Segmento mais longo, linhas 1124–1205** (portado em
  `bpm_do_mais_longo`): `60000/iv` do primeiro segmento de maior extensão,
  guardado em `[rsp+0xa0]`.
- **Vetores por segmento, linhas 953–994**: 6 doubles por segmento,
  iniciados em `[-30, -20, -3, -3, -20, -3]` (lidos de `0x143ea0ca0`…
  `0x143ea0cb8` e `0x143ea0c80`/`c88`; a redução por máximo parte de
  `-DBL_MAX`, `0x1458010e0`). Na grade, são indexados pela banda da batida
  (`< 6`) e multiplicados por `taxa·0,001`. INFERIDO: compensação de
  atraso por banda, em ms.
- **Busca de fase por segmento `0x142617530`** (portada em
  `ajustar_fase`, com `mais_proxima` = `0x142616fc0` e `pontuar_grade` =
  `0x142617c00`):
  - testa deslocamentos de `0, +1, −1, +2, −2` ms no intervalo e
    subdivisões `k = 4, 2, 1`;
  - as fases candidatas andam a cada 96 amostras (`0x143ea0c58`), com
    janela `max(taxa/20, 0,125·passo)`;
  - a pontuação soma `w − |pos − t|` e é reescalada ×2 a cada meia
    subdivisão. Só no deslocamento 0 há parada antecipada;
  - há uma FMA só, o `fnmadd` do recuo com casamento;
  - o segmento sai com `pos = fase`, o intervalo, `campo_c = 4k` e
    `campo10 = recuo`.
- **Geração da grade, linhas 1254–1888** (portada em `gerar_grade`,
  conferida no assembly):
  - âncora `q = fma(ms, compensação, q)`, com a compensação da banda casada
    ou o máximo das 6;
  - passo por quarto `p = fma(ivs, 0,25, p)`, com casamento a cada
    `campo_c` quartos;
  - a cada 128 quartos, reestimativa
    `fma(ms·pont[b], 4, (p − q)·4)·(0,25/32)`;
  - emissões para trás (uma vez), da âncora (flag `+0x14`), 16 adiante
    (uma vez) e 4 por reestimativa, numa janela deslizante de 16 quartos;
  - fim do segmento com `passo = X/quartos` e preenchimento até o fim
    dentro do bloco `p < fim`;
  - primeiro e último tempo forte com banda válida em
    `local_1c8`/`local_160`.
- **Fim do modo 0, linhas 1930–2002** (portado em `regrade_uniforme`,
  `numerar` e `contar_bpm_modo0`):
  - `primeira_pos` é a posição do primeiro tempo forte, ou 0;
  - `0x1426182f0` refaz a grade uniforme entre o primeiro e o último tempo
    forte, com `passo = (pb − pa)/n`, início `fma(−passo, k, pa)` e
    intervalo `(f32)((1000/taxa)·passo)`;
  - BPM devolvido: `60000f/grade[0].intervalo`, calculado antes da
    numeração (`0x1426165af`);
  - numeração `1..=max(4, 4)` a partir da batida de `primeira_pos`;
  - saem as batidas com `pos` fora de `[0, total]`.
- **Modo 1 (`0x142613100` linhas 362–816 e 1900–2080)** (portado em
  `estimar_modo1`, `primeira_na_caixa`, `ajustar_grade_modo1` e
  `contar_bpm_modo1`):
  - linhas 362–816: acumula corridas de batidas (`0x142616e30` e
    `0x142618a70`), segmenta em janelas de `8 · taxa` e guarda o BPM do
    segmento mais longo (`[rsp+0xa0]`, `0x14261653b`);
  - linhas 1900–2080: `0x142617e60` (`ajustar_grade_modo1`) faz correções
    locais em janelas de 4 batidas (`±passo/2`) sobre a caixa de todas as
    bandas e recalcula os intervalos locais em milissegundos, devolvendo o
    BPM do segmento mais longo e numerando a grade a partir de `primeira_pos`.

### Refinadores do fechamento `0x142607110` (2026-11-14)

- **`0x142619120` (`+0x558`):** só roda com a flag `+0xa`. O construtor a
  zera e só há setter dessa família no motor 2 (`0x1411db7a0`); no motor 1,
  não roda.
- **Envelope `0x14261a080` (`+0x5a0`)**, com init `0x142619aa0` e
  alimentação `0x142619ea0` (portado em `RefinadorDeEnvelope` e
  `PassaAlta1D`, conferido em `0_14261a080.c` e `0_14260dd20.c`):
  - roda antes do `UnitBeatAdjust`, com o BPM da contagem;
  - decimação por 128 (`taxa_env = taxa / 128`), duas seções passa-baixas RBJ
    de 150 Hz, retificação, passa-baixas RBJ de 5 Hz e filtro passa-alta 1D de
    1ª ordem (`BeatAnalyzer_1_0::HighPathFilter1D`, `0x14260dd20` /
    `0x14260dd80` / `0x14260de00`, `fc = 0,5 Hz`, `ω = 2π·fc/fs`,
    `b0 = 2/(ω+2)`, `a1 = (ω−2)/(ω+2)`);
  - corrige inversão de meio-tempo (180°) quando mais da metade dos picos de
    envelope cai fora das batidas (`fora > selecionados.len() / 2` via busca
    binária `batida_mais_proxima`) e define a fase `1..4` do compasso a partir
    dos picos mais altos (`0,9` e `0,6 · max_pico`).
- **`UnitBeatAdjust` (`+0x608`), só no modo 0** (portado em
  `AjusteDeBatidas`, conferido no assembly):
  - init `0x142609070`: `fs4 = taxa/4` e envelope limitado a
    `min(total/4, fs4·900)`; passa-alta 2D de 2 000 Hz e passa-baixa 2D de
    1 000 Hz (`0x14260d9e0`/`0x14260d630`), em `f64`, com `k = 1/√2` fixo;
  - alimentação `0x142609260`: 1 de cada 4 frames estéreo,
    `(R + L)·0,5` → passa-alta → `|·|` → passa-baixa, limitado a
    `[0, 32767]` em `i16`;
  - busca `0x142609780` com `0x14260a270` e `0x142609b50`: candidatos
    `fma(i, 0,002, início)` limitados a 70–180, cada um com grade
    `fma(passo, k, início)` a `fs/4` e nota = pico da soma do envelope em
    ±0,05 s. A busca começa em ±0,016, estende a ±0,05 quando o melhor cai
    na borda e refina a ±0,001;
  - aplicação `0x14260a560`: com mudança no milésimo, a grade vira a
    candidata (posições ×4, intervalo `60000/bpm`) e o BPM (`motor+0x48`),
    `(f32)melhor`;
  - deslocamento `0x14260ad60`: `(pico − w)·4` amostras (menos 1 ms em duas
    condições), apara as batidas fora do sinal e estende as bordas em
    passos de uma batida (`iv·0,004·fs4`), renumerando.
- **Orquestrador `AnalisadorDeBatidas` (`0x142606e40` / `0x142607110`)**:
  alimenta `SensorDeBatidas`, `RefinadorDeEnvelope` e `AjusteDeBatidas` a
  cada bloco completo de `taxa / 75` frames do `Alimentador` (garantindo
  invariância exata ao tamanho do bloco de entrada) e executa o fechamento
  completo nos modos 0 e 1.

### Primitivas de análise de frases A2.3 em `vilelabox-dsp::frases` (2026-11-14)

- **`0x1412d0aa0` (`normalizar_e_agrupar_compassos`)**: normalização por pico
  com sentinela `-100.0`, piso `1e-5` e agregação por grupos de 4 batidas.
- **`0x1412e8de0` e `0x1412e93f0` (`detectar_transicoes_energia`)**: média
  local em janela `[-w, +w]`, supressão de bordas `3·w`, suavização
  `0,25·E[i-1] + 0,5·E[i] + 0,25·E[i+1]` e pareamento de subidas/quedas
  acima de `0,05`.
- **`0x1412e7460` (`detectar_transicoes_razao`)**: razão frente/trás em
  janela `w`, com limiar de subida `> 1,35` e queda `< 0,65`.
- **`0x1412dff80` (`extrair_picos_espectrais` e
  `agregar_picos_espectrais_compassos`)**: máquina de estados de confirmação
  de picos em dois passos (`0,9 · pico`, `> 0,001`), filtro de amplitude
  `> 0,1` e agregação por compassos com limiares `0,36`, `1,2`, `0,2` e `1,8`.

## Alta precisão (`0x141134550`)

Decodifica o arquivo para buffers `f32`, roda inferência em blocos
(`0x141134390`, `0x14113fda0`) e devolve o JSON `beatList`. Depende de um
modelo treinado do programa de referência e fica fora do escopo clean-room.

## Classificação

- **OBSERVADO:** encadeamento do job, escolha do modo, tabela de faixas de
  BPM, arredondamento do BPM × 100, layout das batidas, soma de janela,
  posição de entrada, contrato de entrada do detector legado (`f32` planar
  misturado em mono `f64`), tamanho de bloco e laço de duas passadas.
- **INFERIDO:** o sinal `i16` em `motor+0x640` é derivado do áudio, porque o
  índice de início é a posição dividida por 4. Os índices 1..10 da tabela de
  faixas são as opções de faixa de BPM das preferências. A posição de entrada
  marca o início da parte com energia.
- **DESCONHECIDO:** qual canal entra com peso 0,99; o terceiro argumento do
  init; o conteúdo das passadas A/B e C/D (`0x1411953b0`, `0x141195480`,
  `0x141195ed0`, `0x1411965b0`, `0x141197d60`); quem preenche `motor+0x640`;
  o motor 1; as tabelas de razões em `0x143b107b8`–`0x143b10880`; as rotinas
  de dobrar e dividir a grade (`0x1411308c0`, `0x141130d90`); a análise de
  tom.
- **OBSERVADO (A1.1c):** filtros Butterworth de 8ª ordem com decimação, as
  bandas das passadas A e B e a grade de índices `0x141173280`.
- **Não observado:** FFT ou janela de Hann, até aqui. Os estágios do
  fechamento da passada A e a passada C/D ainda não foram abertos.

## Reancoragem para o 7.2.19 (2026-11-14)

O 7.2.8 foi desinstalado. O projeto Ghidra atual analisa
`C:\Program Files\rekordbox\rekordbox 7.2.19\rekordbox.exe` (SHA-256
`c9ed23e974c51c75e2ec069fbd2679a3dbe606be9e79e9cbbd13d6418ab11c37`). Os
endereços deste dossiê continuam sendo os do 7.2.8. O mapa completo dos 146
endereços de código citados aqui e em `vilelabox-dsp` está em
`p4-1-enderecos-7.2.19.tsv`.

Método:

1. As strings `__FUNCSIG__` do log foram localizadas no 7.2.19, e seus
   chamadores lidos com `tools/re/RefsTo.java`. As cinco funções de
   `AnalyzeJobBeatDetector` deram o mesmo delta, `+0x15fc10`. As de
   `AnalyzeJobKey` deram `+0x15fbe0`.
2. Para cada bloco de 1 MB, contou-se quantos endereços antigos caem
   exatamente num início de função do 7.2.19 para cada delta. Um delta
   errado acerta cerca de 1 em 18. Os acertos foram: `0x14116` 29/33,
   `0x14117` 31/34, `0x14118` 27/33 e `0x14119` 10/10. As falhas são
   endereços de laço citados de propósito.
3. O motor 2 foi ancorado por `interval*.csv`, e a alta precisão por
   `beatList`. O motor 1, a thread e a STL foram resolvidos pelo callgraph,
   a partir de chamadores já mapeados.

| Faixa no 7.2.8 | Delta | Âncora |
| --- | --- | --- |
| `0x14107c200`, `0x14107c340` | `+0x15f4c0` | callees de `highPrecisionBeatAnalyze`; o refinador só é chamado por ela |
| `0x1410b3cb0` | `+0x15fb90` | único chamador de `0x14116f000` |
| `0x14112e000`–`0x141131000` | `+0x15fc10` | strings do job de batida |
| `0x141133000`–`0x141134400` | `+0x15fbe0` | strings do job de tom |
| `0x141134550` / `0x14113fda0` | `+0x15fbb0` / `+0x15f3d0` | `beatList`; callees da alta precisão |
| `0x141164000`–`0x141169000` | `+0x15f4e0` | contagem por bloco |
| `0x14116e000`–`0x1411c6000` | `+0x15f6c0` | contagem por bloco; aresta B → `grades_passada_b` conferida |
| `0x14245c630` / `0x142470b60` | `+0x1cf650` / `+0x1cf680` | `interval*.csv`; chamada da soma de meia batida |
| `0x142476000`–`0x14247b000` | `+0x1cf5e0` | 5 de 7 inícios; chamadores conferidos |
| `0x142604d40`–`0x142604f60` | `+0x2054a0` | callees de `0x141167280`, `0x141167340` e `0x1411686b0` |

Alvos pendentes no 7.2.19:

| 7.2.8 | 7.2.19 | Tamanho (bytes) | Conferência |
| --- | --- | --- | --- |
| `0x14118d310` | `0x1412ec9d0` | 3590 | chamada pela passada C (`0x1412f5c70`) |
| `0x14118e450` | `0x1412edb10` | 3431 | — |
| `0x141197890` | `0x1412f6f50` | 1223 | — |
| `0x141197d60` | `0x1412f7420` | 23378 | único chamador `0x1412cfaf0`, que também chama `0x1412cfc80` (= `0x1411705c0`) |

Ficaram sem âncora `0x1410bab80` e as rotinas de libm: `0x14279adf0`,
`0x1427a25f0`, `0x1427a2fa0`, `0x1427a51e0`, `0x1426086a0` e
`0x142608cd0`. O porte já troca as de libm pela biblioteca padrão. As
constantes de `.rdata` (`0x143…`, `0x145…`) não foram reancoradas. O
tamanho das funções antigas não está registrado, por isso a conferência
não compara corpos. Ela usa o início de função, a contagem por bloco e as
arestas do callgraph.

## Próximos alvos

1. **Portar o estimador `BeatAnalyzer_1_0`** (ver a seção do estimador),
   nesta ordem:
   1. alimentação `0x142606e40`;
   2. sensor de batidas `0x1426080d0`;
   3. contagem `0x142613100` com `BpmCountTbl`, que decide o BPM;
   4. refinadores `0x142619120` e `0x14261a080`;
   5. ajuste `0x142609560`.

   Os processadores de vista (volume e zoom) ficam fora do BPM. Falta
   verificar se o BPM "já conhecido" que a thread entrega às passadas A–D
   (vtable de `thread+0x2e0`, slot `+0xa0`) é este `motor+0x48`.
2. O fechamento `0x141197d60` (`0x1412f7420`) pertence à fase A2.3
   (frases). Portá-lo depois do estimador.
3. Ordem dos canais na chamada de `0x1411705c0`, a partir de `0x1410bab80`.
4. Teste diferencial: analisar uma fixture WAV no programa de referência e
   comparar deslocamento, número da batida e grade com o porte.
