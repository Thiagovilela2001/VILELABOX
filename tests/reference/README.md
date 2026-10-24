# Conjunto de Referência e Tolerâncias — VILELABOX

Este diretório contém os resultados esperados de referência para validação contínua e comparação com futuras implementações (como a transição para Rust), sem a necessidade de versionar arquivos de áudio protegidos por direitos autorais.

## Estrutura do Diretório

- `tests/reference/synthetic/`: matriz determinística de R1.3 gerada por `tools/reference/make_synthetic.py` (sem gravações externas), incluindo `playlists_tree.json`.
- `tests/reference/synthetic/wavs/`: diretório local para os WAVs recriados (ignorado pelo git).
- `tests/reference/mixxx/`: exportação local de um perfil do Mixxx por `tools/reference/export_reference.py` (ignorada pelo git).
- `tests/reference/rekordbox/`: exportação local das análises ANLZ por `tools/reference/export_rekordbox_anlz.py` (ignorada pelo git).

Cada diretório tem um `index.json` com a lista `tracks` (só SHA-256 e números, sem artista nem título).

## Esquema canônico v1

Todos os JSONs de faixa seguem `ReferenceTrack` em `rust/vilelabox-reference/src/schema.rs`, com `"schema_version": 1`. Campos desconhecidos e outras versões são rejeitados; o CI regenera a matriz sintética e falha se ela divergir do que está versionado.

- Tempos em segundos: `first_beat_time_s`, frases (`start`, `end`), cues (`position_s`, `length_s`) e segmentos vocais. Cues não usam frames, então posições fracionárias e fontes sem taxa de amostragem (rekordbox) ficam sem perda.
- `downbeat_index` é a fase do compasso em 4/4 (0..=3). O Mixxx guarda o índice absoluto da batida; o exportador grava o resto da divisão por 4.
- Campo ausente significa "a fonte não sabe" (ex.: o ANLZ não tem vocal nem taxa de amostragem). O comparador só cobra o que a referência afirma.
- Cues: `type` é `hotcue`, `cue`, `memory`, `loop`, `jump`, `intro` ou `outro`; `hotcue` é o slot a partir de 0.
- Playlists (`ReferencePlaylistSet`): árvore de `name`, `tracks` (SHA-256 em ordem) e `children`. A identidade é o caminho com a posição entre irmãos, para distinguir homônimas.
- No rekordbox 7 as seções `PCOB`/`PCO2` do ANLZ local vêm vazias: cues do rekordbox precisam vir do XML ou do `master.db`.

## Tolerâncias de Comparação

Qualquer implementação (C++ atual, módulo em Rust ou testes de regressão) deve satisfazer as seguintes tolerâncias em relação aos dados de referência:

| Métrica | Tolerância Máxima | Fonte / Evidência no Código | Justificativa |
| --- | --- | --- | --- |
| **Andamento (BPM)** | **±0,01 BPM** | [GRID.md](../../GRID.md#editor), [src/engine/controls/bpmcontrol.cpp](../../src/engine/controls/bpmcontrol.cpp) | Resolução nativa de duas casas decimais suportada pelo editor de grid e exibição. |
| **Alinhamento de Grid (batidas)** | **≤ 1,0 ms** (áudio sintético limpo) / **≤ 25 ms** (áudio real complexo) | [GRID.md](../../GRID.md#verificação-e-restauração) (erro mediano medido entre 0 e 0,36 ms), [src/analyzer/plugins/analyzerqueenmarybeats.cpp:169](../../src/analyzer/plugins/analyzerqueenmarybeats.cpp#L169) (raio de busca de 25 ms) | A resolução de ataques do analisador refinado é de cerca de 1 ms; variações acima de 25 ms indicam troca ou salto de batida. |
| **Compasso 1 (Downbeat)** | **Exato (0 batidas de desvio)** | [GRID.md](../../GRID.md#editor) (função `SET 1`), [src/proto/beats.proto:32](../../src/proto/beats.proto) (`first_downbeat_index`) | A fase de compasso em 4/4 é discreta (índice 0, 1, 2 ou 3). Qualquer divergência altera a sincronização do BarSync e contagem visual. |
| **Limites de Frase (início / fim)** | **Até 1 compasso** | [FRASES.md:90](../../FRASES.md#L90) (meta: 80% dos limites a até 1 compasso, em 20 faixas), [VALIDACAO.md](../../VALIDACAO.md#frases-como-medir) | Frases musicais estruturam-se em múltiplos de 4 e 8 compassos; uma variação de até 1 compasso em transições com pickup/anacruse é musicalmente aceitável. |
| **Acurácia de Rótulo de Frase** | **Sem meta definida** (medida informativa) | [VALIDACAO.md](../../VALIDACAO.md#resultado) | A meta de 80% de [FRASES.md:90](../../FRASES.md#L90) vale para limites, não para rótulos. `tools/phrases/score_csv.py` informa a acurácia de rótulo, mas não a usa para aprovar. |
| **Energia (escala 1–10)** | **±0,05** (provisória) | [src/analyzer/analyzerenergy.cpp](../../src/analyzer/analyzerenergy.cpp) (`1 + 9 × (0,6 loudness + 0,25 ataques + 0,15 graves)`), [src/library/related/relatedtracks.cpp](../../src/library/related/relatedtracks.cpp) (cache em `analysis/blackmesa.sqlite`) | Sem meta medida. O port Rust usa a mesma fórmula, então a diferença esperada é só numérica; revisar em R4.2 com a matriz real. |
| **Detecção de Vocal (intervalos)** | **Tolerância de 250 ms** nas bordas | [VOCALS.md](../../VOCALS.md#motor-demucs-preferido), [res/vocals/vocal_detector.py:206](../../res/vocals/vocal_detector.py#L206) | Filtro de votação temporal de 250 ms para evitar oscilações em pausas curtas de fala e respiração. |

## Como Recriar os Casos Sintéticos

Para gerar os WAVs e atualizar os JSONs no Windows ou Linux:

```bash
python tools/reference/make_synthetic.py
```

## Como Exportar Referências Reais

Com um perfil do Mixxx com faixas analisadas, e com o rekordbox instalado:

```bash
python tools/reference/export_reference.py --profile ~/.mixxx --output tests/reference/mixxx
python tools/reference/export_rekordbox_anlz.py --mixxx-db <perfil>/mixxxdb.sqlite
cargo test -p vilelabox-reference --test fixtures   # em rust/: valida as exportações locais
```

- Faixas cujo arquivo de áudio não existe são ignoradas e contadas em `missing_audio`.
- `tests/reference/mixxx/locations.json` liga caminho a SHA-256 para `rust/vilelabox-mixxx-compat/tests/real_profile.rs` comparar o decodificador de `beats.proto` com o banco. Tem caminhos pessoais e fica só local.
- O mesmo áudio em mais de um caminho gera um só arquivo; as repetições são contadas em `duplicate_audio`.
- O `index.json` não guarda artista nem título, só o SHA-256 do áudio.
- Os segmentos de vocal são achados pelo nome do cache, que depende de
  `res/vocals/model.json` e `res/vocals/vocal_detector.py`. Se o VILELABOX usa
  outra cópia desses arquivos, passe `--resources <dir res/ instalado>`.
