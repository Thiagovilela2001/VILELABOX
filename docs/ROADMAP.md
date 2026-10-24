# Roadmap

O detalhamento executável, com dependências, tarefas e gates, está em
[`IMPLEMENTATION-PLAN.md`](IMPLEMENTATION-PLAN.md).

## C2 — Formatos

- [x] interpretar contêiner, caminho, grade e frases ANLZ;
- [x] interpretar cues e loops `PCOB`/`PCPT`;
- [x] converter ANLZ para o esquema canônico;
- [x] comparar o leitor com 40 cópias de análises reais;
- [ ] implementar o exportador ANLZ em Rust;
- [ ] interpretar labels e cores das variantes de cues;
- [ ] especificar e implementar catálogo, XML e exportação USB.

## C3 — Análise

- BPM e grade;
- frases, tonalidade, energia, waveform e vocal;
- comparação diferencial e benchmarks reproduzíveis.

## C3b — Biblioteca e preparação

- banco próprio;
- coleções, playlists, busca e preparação;
- núcleo headless com comandos e eventos.

## C5 — Interface

- dividir a interface Slint em componentes menores;
- composição das telas;
- navegador da biblioteca;
- waveform e interação de preparação.

## C4 — Áudio

- transporte, mixer, sync, quantização, loops e cues;
- efeitos, sampler, separação, gravação e vídeo;
- critérios de tempo real e validação auditiva.

## C6 — Dispositivos

- MIDI, HID e mapeamentos;
- rede e iluminação.
