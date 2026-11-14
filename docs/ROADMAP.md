# Roadmap

O detalhamento executável, com dependências, tarefas e gates, está em
[`IMPLEMENTATION-PLAN.md`](IMPLEMENTATION-PLAN.md).

Toda implementação parte de evidência de engenharia reversa e decompilação no
Ghidra. O Gate G0 do plano bloqueia funcionalidades sem proveniência documentada.

## C2 — Formatos

- [x] interpretar contêiner, caminho, grade e frases ANLZ;
- [x] interpretar cues e loops `PCOB`/`PCPT`;
- [x] converter ANLZ para o esquema canônico;
- [x] comparar o leitor com 40 cópias de análises reais;
- [x] implementar o exportador ANLZ em Rust;
- [x] interpretar labels e cores das variantes de cues;
- [x] especificar o catálogo principal (`10-master-db.md` + G0/P2.1);
- [x] implementar `vilelabox-xml` (leitura, escrita e integração ao app; G0/P2.2);
- [x] implementar `vilelabox-catalog` (leitura da cópia sem criptografia; `tools/catalog/exportar_catalogo.py`);
- [ ] exportação USB.

## C3 — Análise

- [x] BPM e grade: passadas A, B e C (15 extratores), estimador `BeatAnalyzer_1_0` (`AnalisadorDeBatidas` modos 0 e 1, `RefinadorDeEnvelope` e `AjusteDeBatidas`) e primitivas de frases (`vilelabox-dsp::frases`) portados;
- [x] pipeline reproduzível e cancelável `vilelabox-analysis` (BPM/grade, frases, tonalidade, energia e waveform — Gate G4);
- comparação diferencial completa com lote de 40 análises reais e benchmarks reproduzíveis.

## C3b — Biblioteca e preparação

- [x] banco próprio (`vilelabox-storage`, P3.1);
- [x] coleções, playlists, busca e preparação (`vilelabox-library`, P3.2);
- [x] núcleo headless com comandos e eventos (`vilelabox-core`, P3.3);
- [x] CLI headless com o fluxo de preparação (`vilelabox-cli`, P3.4 + G3).

## C5 — Interface

- [x] dividir a interface Slint em componentes menores (`G0/P0.1` + `P0.1`);
- [ ] composição das telas;
- [ ] navegador da biblioteca;
- [ ] waveform e interação de preparação.

## C4 — Áudio

- transporte, mixer, sync, quantização, loops e cues;
- efeitos, sampler, separação, gravação e vídeo;
- critérios de tempo real e validação auditiva.

## C6 — Dispositivos

- MIDI, HID e mapeamentos;
- rede e iluminação.
