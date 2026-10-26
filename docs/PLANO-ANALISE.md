# Plano — análise musical (P4)

Detalha a fase P4 do [`IMPLEMENTATION-PLAN.md`](IMPLEMENTATION-PLAN.md):
`vilelabox-dsp` (infraestrutura numérica) e `vilelabox-analysis` (pipeline de
etapas), preenchendo no banco próprio a identidade acústica, a grade, as
frases e a tonalidade das faixas importadas pela biblioteca (P3.2).

**Regra de implementação:** nenhuma etapa é criada do zero quando existe
comportamento no programa de referência. Primeiro se recuperam entradas,
saídas, chamadas, parâmetros e tabelas no binário (REA + Ghidra); depois se
traduz a regra documentada para Rust e se compara com referências reais e
sintéticas. O que não foi recuperado permanece pendente; decisões novas são
marcadas como `ADAPTAÇÃO`.

## Ponto de partida

| já existe | falta |
| --- | --- |
| Decodificação de áudio no `vilelabox-audio` (symphonia → PCM estéreo) | contrato PCM explícito por blocos |
| `analysis_results` no storage: cache por faixa, etapa, versão e configuração | crate `vilelabox-dsp` |
| 40 análises reais + fixtures sintéticas (WAV com BPM, grade, cues e loops conhecidos) como oráculo | crate `vilelabox-analysis` |
| Dossiês ANLZ de grade (PQTZ) e frases (PSSI) | algoritmos próprios de BPM/grade, downbeat, tonalidade, energia, waveform e vocal |
| SHA-256 por streaming no `vilelabox-library` | preenchimento da identidade (duração, taxa, canais) das faixas importadas |

## Regras que valem para todo o plano

- **G0 primeiro.** Cada etapa começa por dossiê de evidência e spec; análise
  estática que não prova o comportamento é complementada por teste diferencial.
- **Determinismo.** Mesmo resultado no mesmo backend; tolerâncias registradas
  por métrica; leitura em blocos independente do tamanho do bloco.
- **Sem I/O no núcleo numérico.** Janelas, FFT e estatísticas não tocam disco
  nem alocam por amostra fora do contrato declarado.
- **Robustez.** Cancelamento e arquivos corrompidos não comprometem o banco;
  falha de uma etapa não derruba as demais.

## Fase A1 — `vilelabox-dsp` (P4.1)

| # | tarefa | entrega | teste |
| --- | --- | --- | --- |
| A1.1 | Recuperar com REA e Ghidra tipos numéricos, tamanhos de bloco, janelas, constantes, encadeamento e tratamento de bordas | dossiê G0 + notas na spec | — |
| A1.2 | Contrato PCM | crate `vilelabox-dsp`: blocos estéreo `f32` indexados em frames | propriedade de fatiamento |
| A1.3 | Janelas e FFT | Hann e FFT (rustfft) sem I/O | contra fixture de referência |
| A1.4 | Invariância de bloco | mesmo resultado com blocos de tamanhos diferentes | propriedade |
| A1.5 | Benchmarks reproduzíveis | harness registrado no repositório | números anexados à entrada do IMPLEMENTACAO |

## Fase A2 — Pipeline `vilelabox-analysis` (P4.2)

Cada etapa é cancelável, cacheada por (hash do áudio, versão do algoritmo,
configuração) em `analysis_results`, e escreve seu resultado definitivo nas
tabelas próprias do banco.

| # | etapa | entrada | saída no storage | oráculo |
| --- | --- | --- | --- | --- |
| A2.1 | Identidade | arquivo de áudio | `sha256`, duração, taxa e canais na faixa | metadados conhecidos das fixtures |
| A2.2 | BPM e grade | PCM | `beatgrids` + `grid_beats`/`grid_tempo` | 40 análises reais + fixtures WAV |
| A2.3 | Downbeat e frases | PCM + grade | `phrases` | ANLZ real (`PSSI`) |
| A2.4 | Tonalidade | PCM | `tracks.key_text` | faixas com tom conhecido |
| A2.5 | Energia e waveform | PCM | `analysis_results` | comparação com `.EXT` real |
| A2.6 | Vocal | PCM | `analysis_results` | `.EXT` quando a evidência existir |

### Gate G4 — Análise reproduzível

- resultados determinísticos no mesmo backend;
- tolerâncias registradas por métrica;
- cancelamento e arquivos corrompidos não comprometem o banco.

## Ligação com o que vem depois

- P3.3 ([`PLANO-CORE-HEADLESS.md`](PLANO-CORE-HEADLESS.md)) chama o pipeline
  como tarefa e acompanha por `TarefaAnaliseAtualizada`.
- P5 passa a exibir waveform e grade calculados pelo próprio VILELABOX, sem
  depender de análises externas.
- P6 usa a grade própria para sync, quantização e metrônomo.
