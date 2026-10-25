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

- BPM e grade;
- frases, tonalidade, energia, waveform e vocal;
- comparação diferencial e benchmarks reproduzíveis.

## C3b — Biblioteca e preparação

- [x] banco próprio (`vilelabox-storage`, P3.1);
- [ ] coleções, playlists, busca e preparação;
- [ ] núcleo headless com comandos e eventos.

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
