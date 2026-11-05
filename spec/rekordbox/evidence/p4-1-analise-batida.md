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
da faixa no init). A estimativa do BPM está nas passadas B e C/D
(`0x141173370`, `0x1411965b0`, `0x141197d60`), ainda não abertas.

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

## Próximos alvos

1. Extratores da passada C, um por vez, do menor ao maior (tabela em
   "Passada C por bloco"), cada um conferido no assembly; depois
   `0x141197890` e o fechamento `0x141197d60`, que deve ser onde o BPM é
   decidido.
2. Passadas C/D: `0x1411965b0` (682 linhas) e `0x141197d60` (2 067 linhas),
   com cerca de 100 funções auxiliares no total (as maiores: `0x141192c60`,
   `0x1411913e0`, `0x1411c1d70`, `0x1411c0b90`, `0x1411bdfc0`).
3. Ordem dos canais na chamada de `0x1411705c0`, a partir de `0x1410bab80`.
4. Teste diferencial: analisar uma fixture WAV no programa de referência e
   comparar deslocamento, número da batida e grade com o porte.
