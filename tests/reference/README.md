# Conjunto de Referência e Tolerâncias — VILELABOX

Este diretório contém os resultados esperados de referência para validação contínua e comparação com futuras implementações (como a transição para Rust), sem a necessidade de versionar arquivos de áudio protegidos por direitos autorais.

## Estrutura do Diretório

- `tests/reference/<sha256>.json`: resultado de referência exportado de bibliotecas reais pelo script `tools/reference/export_reference.py`.
- `tests/reference/index.json`: catálogo consolidado de todas as faixas exportadas.
- `tests/reference/synthetic/`: casos de teste determinísticos gerados sinteticamente por `tools/reference/make_synthetic.py` (sem dependência de gravações externas).
- `tests/reference/synthetic/wavs/`: diretório local para os WAVs recriados (ignorado pelo git).

## Tolerâncias de Comparação

Qualquer implementação (C++ atual, módulo em Rust ou testes de regressão) deve satisfazer as seguintes tolerâncias em relação aos dados de referência:

| Métrica | Tolerância Máxima | Fonte / Evidência no Código | Justificativa |
| --- | --- | --- | --- |
| **Andamento (BPM)** | **±0,01 BPM** | [GRID.md](../../GRID.md#editor), [src/engine/controls/bpmcontrol.cpp](../../src/engine/controls/bpmcontrol.cpp) | Resolução nativa de duas casas decimais suportada pelo editor de grid e exibição. |
| **Alinhamento de Grid (batidas)** | **≤ 1,0 ms** (áudio sintético limpo) / **≤ 25 ms** (áudio real complexo) | [GRID.md](../../GRID.md#verificação-e-restauração) (erro mediano medido entre 0 e 0,36 ms), [src/analyzer/plugins/analyzerqueenmarybeats.cpp:169](../../src/analyzer/plugins/analyzerqueenmarybeats.cpp#L169) (raio de busca de 25 ms) | A resolução de ataques do analisador refinado é de cerca de 1 ms; variações acima de 25 ms indicam troca ou salto de batida. |
| **Compasso 1 (Downbeat)** | **Exato (0 batidas de desvio)** | [GRID.md](../../GRID.md#editor) (função `SET 1`), [src/proto/beats.proto:32](../../src/proto/beats.proto) (`first_downbeat_index`) | A fase de compasso em 4/4 é discreta (índice 0, 1, 2 ou 3). Qualquer divergência altera a sincronização do BarSync e contagem visual. |
| **Limites de Frase (início / fim)** | **Até 1 compasso** | [FRASES.md:90](../../FRASES.md#L90) (meta: 80% dos limites a até 1 compasso, em 20 faixas), [VALIDACAO.md](../../VALIDACAO.md#frases-como-medir) | Frases musicais estruturam-se em múltiplos de 4 e 8 compassos; uma variação de até 1 compasso em transições com pickup/anacruse é musicalmente aceitável. |
| **Acurácia de Rótulo de Frase** | **Sem meta definida** (medida informativa) | [VALIDACAO.md](../../VALIDACAO.md#resultado) | A meta de 80% de [FRASES.md:90](../../FRASES.md#L90) vale para limites, não para rótulos. `tools/phrases/score_csv.py` informa a acurácia de rótulo, mas não a usa para aprovar. |
| **Detecção de Vocal (intervalos)** | **Tolerância de 250 ms** nas bordas | [VOCALS.md](../../VOCALS.md#motor-demucs-preferido), [res/vocals/vocal_detector.py:206](../../res/vocals/vocal_detector.py#L206) | Filtro de votação temporal de 250 ms para evitar oscilações em pausas curtas de fala e respiração. |

## Como Recriar os Casos Sintéticos

Para gerar os WAVs e atualizar os JSONs no Windows ou Linux:

```bash
python tools/reference/make_synthetic.py
```

## Como Exportar Referências Reais

Em uma máquina Linux com perfil do Mixxx configurado e faixas analisadas:

```bash
python tools/reference/export_reference.py --profile ~/.mixxx --output tests/reference
```

- Faixas cujo arquivo de áudio não existe são ignoradas e contadas em `missing_audio`.
- O `index.json` não guarda artista nem título, só o SHA-256 do áudio.
- Os segmentos de vocal são achados pelo nome do cache, que depende de
  `res/vocals/model.json` e `res/vocals/vocal_detector.py`. Se o VILELABOX usa
  outra cópia desses arquivos, passe `--resources <dir res/ instalado>`.
