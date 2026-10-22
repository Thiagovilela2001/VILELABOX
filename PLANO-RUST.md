# Plano de implementação da migração para Rust

Plano operacional para agents migrarem o VILELABOX do fork atual do Mixxx
(C++20/Qt 6) para um aplicativo Rust independente. Este documento começa no
estado real de outubro de 2026: o único código Rust existente é o rascunho de
`vilelabox-rekordbox`; o Mixxx continua sendo a implementação executável e a
referência de comportamento.

Este plano complementa `PLANO-4.md`. O Plano 4 prepara inventário, referências
e a primeira crate. Este documento cobre a migração completa e define como os
agents devem produzir, revisar e entregar o código Rust.

## Resultado pretendido

Ao final da migração, o workspace deverá conter um aplicativo desktop capaz de:

- importar, organizar e pesquisar a biblioteca musical;
- analisar BPM, grid, downbeat, energia, frases e vocal;
- preservar cues, loops, playlists e dados compatíveis com `rekordbox.xml`;
- preparar e reproduzir faixas em dois ou mais decks;
- executar sync, mixer, loops, Mix Point, pads e efeitos sem falhas de áudio;
- oferecer uma interface própria, sem depender do Qt, das skins XML ou das
  classes do Mixxx;
- receber controladoras MIDI por uma camada separada do motor de áudio.

O objetivo não é traduzir classes Qt linha por linha. A migração deve extrair
regras musicais e contratos de dados, provar paridade e só depois substituir a
infraestrutura do Mixxx.

## Estado inicial e restrições

### Implementado hoje

- O produto funcional é o fork do Mixxx em `src/` e `res/`.
- O workspace `rust/` possui `vilelabox-rekordbox` com paridade estrita e correções da ponte aprovadas (R0.4). Em 2026-10-22 passou em `cargo check/test/clippy/fmt` no Windows (Rust 1.99.1), CI independente configurado (.github/workflows/rust.yml) e equivalência comprovada ao `sync.py`. Fase 0 concluída (G0 atingido).
- Há casos sintéticos e um formato de referência em `tests/reference/`.
- O oráculo externo das análises do rekordbox (R1.2a) foi implementado em `tools/reference/export_rekordbox_anlz.py` e validado contra análises reais em USBANLZ.
- O workflow `.github/workflows/vilelabox.yml` foi escrito para compilar o mod
  no Ubuntu e executar `tools/verify-mod.sh`; a primeira execução no GitHub está
  pendente (`IMPLEMENTACAO.md`, Plano 4, A3).
- A validação com músicas reais, hardware de áudio e rekordbox real está
  pendente.

### Restrições obrigatórias

- Preservar GPL-2.0-or-later e os avisos de autoria do código derivado.
- Não alterar `lib/`, `src/controllers/` nem `res/controllers/` sem autorização
  explícita.
- Não mudar o esquema do `mixxxdb.sqlite`; a aplicação Rust terá banco próprio
  e adaptadores de importação.
- Nunca usar o perfil real `~/.mixxx` em testes automatizados.
- Nenhum módulo C++ é removido antes da aprovação do gate de paridade
  correspondente.
- Nenhum resultado de teste é declarado como aprovado se o comando não foi
  realmente executado.
- Áudio real e materiais de terceiros não entram no Git. Fixtures públicas
  precisam ser sintéticas, próprias ou com licença registrada.
- Material do rekordbox (arte, modelos, mapeamentos, binários, análises) pode ser
  usado localmente por decisão do usuário (`CLAUDE.md`, 2026-10-22), sempre em
  `third_party/rekordbox/` (ignorado pelo git) ou lido da instalação do usuário.
  Nenhuma crate pode exigir esse material para compilar, testar no CI ou rodar;
  quando usado, entra por adaptador opcional, desligado por padrão.
- Mudanças de commit, push, PR, instalação de toolchain e publicação continuam
  dependendo de autorização do usuário.

## Estratégia de migração

Será usado o padrão **strangler**:

1. congelar e caracterizar o comportamento existente;
2. definir um contrato independente de Qt/Mixxx;
3. implementar a mesma regra em Rust;
4. comparar C++/Python e Rust sobre as mesmas entradas;
5. executar Rust em modo sombra, sem alterar o resultado usado pelo programa;
6. ativar Rust por feature flag;
7. validar dados, desempenho e uso real;
8. remover a implementação antiga somente após aprovação explícita do gate.

Uma tradução que apenas compila não conta como migração concluída. A unidade de
progresso é uma capacidade cujo comportamento, persistência e desempenho foram
comparados.

Cada capacidade percorre estados oficiais, registrados no backlog e no log de
execução:

```text
Planejada -> Espelhada -> Verificada -> Integrada opt-in -> Padrão -> Legado removido
```

`Espelhada` significa que existe uma implementação Rust; não significa que a
migração terminou. A promoção de estado exige a evidência do gate correspondente.

## Arquitetura Rust de destino

### Regra de dependências

As dependências apontam para dentro, nunca para a interface:

```text
vilelabox-types
├── vilelabox-music
├── vilelabox-dsp
│   ├── vilelabox-analysis
│   └── vilelabox-audio
├── vilelabox-storage
│   ├── vilelabox-library
│   └── vilelabox-mixxx-compat
├── vilelabox-rekordbox ──> vilelabox-mixxx-compat
├── vilelabox-core ──> vilelabox-music, library, analysis
├── vilelabox-audio ──> vilelabox-core
├── vilelabox-midi  ──> vilelabox-core
└── vilelabox-ui    ──> vilelabox-core
```

Nenhuma crate de domínio pode depender de Qt, CPAL, framework de UI ou banco de
dados. Adaptadores convertem dados externos para os tipos do domínio.

### Crates previstas

| Crate | Responsabilidade | Quando criar |
| --- | --- | --- |
| `vilelabox-types` | IDs, tempo, frames, BPM, grid, downbeat, tom, frases, cues e erros comuns | Fase 1 |
| `vilelabox-reference` | Esquema versionado de fixtures, normalização e comparação diferencial | Fase 1 |
| `vilelabox-mixxx-compat` | Único adaptador autorizado a conhecer schema SQLite e `beats.proto` do Mixxx | Fase 1 |
| `vilelabox-rekordbox` | XML do rekordbox, transações de sincronização e CLI | Já existe; concluir na Fase 2 |
| `vilelabox-music` | TempoOctave, tom/Camelot, matemática de grid, frases, Mix Points, auto cues e ranking | Fase 3 |
| `vilelabox-dsp` | PCM, janelas, FFT e primitivas numéricas puras, sem I/O ou cache | Fase 4 |
| `vilelabox-analysis` | Energia, características espectrais, grid/downbeat e pipeline batch | Fase 4 |
| `vilelabox-vocals` | Orquestração do detector, cache, timeout e cancelamento | Fase 4 |
| `vilelabox-storage` | Conexões, transações, migrations e caches persistentes | Fase 6 |
| `vilelabox-library` | Consultas, playlists e repositórios sobre a abstração de storage | Fase 6 |
| `vilelabox-core` | Estado dos decks, comandos, eventos, preparação e coordenação | Fase 6 |
| `vilelabox-audio` | Decodificação, transporte, mixer, sync, loops, pads e efeitos RT | Fase 8 |
| `vilelabox-ui` | Aplicativo desktop e waveform | Fase 7 |
| `vilelabox-midi` | Dispositivos, mapeamentos e comandos de controladoras | Fase 9 |
| `vilelabox-app` | Composição final, configuração, lifecycle e binário | Fase 7 |
| `xtask` | Geração de fixtures, comparação diferencial, auditorias e benchmarks reproduzíveis | Fase 1 |

Crates não devem ser criadas vazias. Cada nova crate nasce na tarefa que entrega
sua primeira capacidade testada.

### Contratos centrais

`vilelabox-types` deverá usar newtypes, unidades explícitas e invariantes:

- `SampleRate`, `Frame`, `EngineSample`, `Seconds` e `Duration` não podem ser
  misturados implicitamente;
- `Bpm` aceita somente valor finito e positivo;
- `BeatGrid` contém taxa de amostragem, batidas ordenadas e referência opcional
  de downbeat;
- `Phrase` usa intervalo semiaberto `[start, end)` e tipo enumerado;
- `Cue` separa identificador, slot opcional, tipo, intervalo, cor e origem
  (`User`, `Automatic`, `Imported`);
- formatos persistidos carregam `schema_version` e `algorithm_version`;
- erros de entrada, I/O, persistência e invariantes são distinguíveis, sem
  `panic!` para dados fornecidos pelo usuário.

## Protocolo de trabalho dos agents

### Papéis

| Papel | Entrega | Pode escrever no checkout principal? |
| --- | --- | --- |
| Agent coordenador | Define pacote, contrato, ownership e gate | Não; entrega ao agent principal |
| Agent de caracterização | Extrai comportamento e cria proposta de fixtures/testes | Não |
| Agent de portabilidade | Produz implementação Rust e testes em patch de handoff | Não |
| Agent diferencial | Compara legado e Rust e relata divergências | Não |
| Agent especialista RT | Revisa callback, atomics, buffers e orçamento de tempo | Não |
| Agent revisor | Confere código, evidências, licenças e critérios de pronto | Não |
| Agent principal | Único que aplica mudanças, executa verificações e atualiza o repo | Sim |

Essa divisão preserva a regra atual de que subagents não editam o repositório.
Agents entregam código como patch unificado e relatório; o principal inspeciona
e aplica somente o que estiver dentro do escopo.

### Modos de colaboração

**Modo vigente — handoff de patch:** todos os subagents são somente leitura. Eles
produzem patch unificado, evidências e relatório; o agent principal é o único
writer. Este é o modo usado enquanto `CLAUDE.md` mantiver a governança atual.

**Modo acelerado — worktrees isoladas:** só pode ser ativado após autorização
explícita do usuário e atualização da governança. Cada agent recebe uma branch e
uma worktree exclusiva, ownership sem sobreposição e proibição de push, PR ou
merge. O agent principal continua dono de `Cargo.toml` do workspace,
`Cargo.lock`, contratos compartilhados, schemas e CI; também integra e reexecuta
todos os gates. Nenhum agent aprova o próprio trabalho.

### Pacote de tarefa obrigatório

Antes de iniciar um port, o agent principal publica um pacote com:

1. identificador, por exemplo `R3.4-grid-math`;
2. arquivos-fonte legados autorizados para leitura;
3. crate e arquivos Rust que podem ser alterados;
4. comportamento de entrada e saída;
5. invariantes e tolerâncias;
6. fixtures e testes que funcionam como oráculo;
7. comandos obrigatórios de verificação;
8. arquivos explicitamente fora do escopo;
9. dependências permitidas;
10. gate que a tarefa pretende liberar.

Sem esse pacote, o agent não começa a portar.

### Handoff obrigatório

Cada agent devolve:

```text
Tarefa:
Base lida:
Arquivos propostos:
Patch:
Testes adicionados:
Comandos executados e resultados:
Divergências do legado:
Riscos e pendências:
Licenças/dependências novas:
```

Código sem teste ou afirmação sem `arquivo:linha` volta para revisão.

### Ondas de execução

- Até três agents podem trabalhar em paralelo apenas quando possuem arquivos e
  contratos sem sobreposição.
- `Cargo.toml` do workspace, `Cargo.lock`, tipos compartilhados, schemas e
  workflows pertencem exclusivamente ao agent principal.
- Dois agents nunca propõem mudanças para o mesmo arquivo na mesma onda.
- Uma onda termina com integração, testes e revisão antes da próxima começar.
- Alterações em contrato público invalidam handoffs ainda não integrados; o
  coordenador precisa reemitir os pacotes afetados.

### Ciclo por módulo

1. **Caracterizar:** agent A descreve comportamento e casos-limite.
2. **Congelar contrato:** principal aprova tipos, schema e tolerâncias.
3. **Portar:** agent B entrega Rust e testes unitários.
4. **Comparar:** agent C cria ou executa testes diferenciais.
5. **Revisar:** agent revisor procura divergências, panics e violações de
   ownership.
6. **Integrar:** principal aplica o patch, formata e executa a matriz.
7. **Modo sombra:** legado continua ativo; Rust apenas mede e registra.
8. **Cutover:** Rust vira padrão por feature flag depois do gate.
9. **Retirar legado:** somente em tarefa separada e autorizada.

## Gates globais

| Gate | Condição mínima |
| --- | --- |
| G0 — Base controlada | Working tree compreendido, toolchain instalada, `Cargo.lock` presente e CI Rust verde |
| G1 — Oráculo confiável | Schemas versionados, fixtures sintéticas e referências reais autorizadas exportadas |
| G2 — Interoperabilidade segura | XML/SQLite sem perda nos casos de colisão e round-trip Rust aprovado |
| G3 — Domínio musical paritário | Algoritmos puros Rust dentro das tolerâncias em toda a matriz diferencial |
| G4 — Análise paritária | BPM, grid, energia, frases e vocal dentro das metas e do orçamento de desempenho |
| G5 — Preparação independente | Biblioteca, importação, análise, cues e playlists funcionam sem abrir o Mixxx |
| G6 — Interface independente | Fluxo completo de preparação disponível no aplicativo Rust |
| G7 — Áudio RT estável | Reprodução/mixagem sem underruns nas condições definidas e sem violações RT |
| G8 — Produto substituto | UI, áudio, MIDI e dados validados; rollback e migração documentados |

Todo gate funcional também precisa satisfazer quatro gates transversais:

- **Dados:** transação atômica, `PRAGMA integrity_check = ok`, backup restaurável
  e teste de falha no meio da escrita.
- **Supply chain:** licenças compatíveis, advisories tratados, dependências
  fixadas e SBOM gerável para o release.
- **FFI:** ABI versionada, nenhum unwind atravessando a fronteira, ownership
  documentado e testes de leak/sanitizers; Miri onde aplicável.
- **Tempo real:** callback sem alocação, mutex, syscall, I/O, log ou panic; a
  instrumentação deve ficar fora do caminho RT.

## Fase 0 — Saneamento e baseline (Concluída em 2026-10-22)

### R0.1 — Resolver o estado do repositório
- Inventariar todas as alterações versionadas, staged e untracked.
- Separar trabalho do Plano 4 de novas tarefas Rust.
- Pedir ao usuário a decisão sobre commit antes de criar branches ou worktrees.
- Não limpar nem sobrescrever arquivos locais.
- **Concluído:** Base salva nos commits `6c9833ed4b` (Plano 4/docs), `46b7a99c7d` (Rust), `e26eba8e6a` (Windows/MSVC) e `957386a1d0` (Mixer RbxOriginal).

### R0.2 — Instalar e fixar a toolchain
- ADR de governança de licenças e compatibilidade documentada.
- Toolchain fixada em Rust 1.99.1 via `rust-toolchain.toml` na raiz e em `rust/`.
- MSRV fixada em `rust-version = "1.75"` em `vilelabox-rekordbox/Cargo.toml`.
- `Cargo.lock` gerado e versionado.
- **Concluído.**

### R0.3 — CI Rust independente
- Adicionado `.github/workflows/rust.yml` executando em Linux (Ubuntu) e Windows:
  `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `cargo test --workspace --all-features` e `python rust/vilelabox-rekordbox/tests/equivalence.py`.
- Ativado caller com gatilho direto de push e pull_request em `.github/workflows/pre-commit.yml`.
- **Concluído.**

### R0.4 — Corrigir o oráculo antes de copiá-lo
- 7 regras aprovadas pelo usuário implementadas e testadas tanto em Python quanto em Rust:
  1. Múltiplos memory cues (`hotcue = -1`): casados por proximidade temporal (tolerância ±1 ms).
  2. Hierarquia e nomes de playlists colidentes: caminho preservado com barras (ex: `Pasta/Set`).
  3. Contagem `Entries`: apenas faixas válidas e ativas exportadas (exclui `fs_deleted = 1`).
  4. Grid variável / BeatMap: decodificação de `beats.proto` gerando múltiplos elementos `<TEMPO>` para mudanças de andamento.
  5. Metadados na importação: mantido sem importar (importação apenas de cues e playlists).
  6. Backups automáticos: retenção dos 10 últimos backups `.bak`.
- Testes: `test_sync.py` (8/8 OK), `sync_test.rs` (9/9 OK) e equivalência estrita de saída XML (`equivalence.py`) confirmada.
- **Concluído.**

**Gate da fase:** G0 — Aprovado em 2026-10-22.

## Fase 1 — Contratos e referência

### R1.1 — Criar `vilelabox-types`

Portar somente modelos e validação, sem algoritmos. Cobrir valores inválidos,
conversões de unidade, ordenação e serialização.

### R1.1a — Isolar compatibilidade com o Mixxx

Criar `vilelabox-mixxx-compat` como o único crate que conhece tabelas do
`mixxxdb.sqlite`, serialização `beats.proto` e peculiaridades de versão do
Mixxx. Ele converte para `vilelabox-types`; nenhum crate de domínio importa
tipos do Mixxx.

### R1.2 — Versionar o esquema de referência

Criar schema canônico para:

- identidade do áudio sem caminho pessoal;
- metadados;
- grid fixo e variável;
- downbeat;
- energia;
- frases;
- segmentos vocais;
- cues e loops;
- playlists e hierarquia.

O exportador Python atual vira produtor legado. `vilelabox-reference` lê,
valida, normaliza e compara.

### R1.2a — Oráculo externo: análises do rekordbox

A instalação local do rekordbox 7.2.8 tem análises de cerca de 2.650 faixas em
`%APPDATA%\Pioneer\rekordbox\share\PIONEER\USBANLZ` (2.661 `.DAT`, 2.646
`.EXT`). Numa amostra de 300 `.EXT`, 223 têm `PSSI` (estrutura de frases) e todas
têm cues (`PCOB`/`PCO2`) e grid de forma de onda; os `.DAT` trazem o grid
(`PQTZ`). O formato já é lido pela especificação Kaitai em
`lib/rekordbox-metadata/rekordbox_anlz.ksy`.

- Exportar essas análises para o schema canônico, só localmente: os resultados
  identificam a biblioteca do usuário e não entram no Git.
- Uso: segunda referência para grid, downbeat, frases (com rótulos) e cues em
  música real, ao lado do Mixxx. Não é verdade absoluta: divergências entre os
  dois viram estatística, e a verdade continua sendo a anotação de ouvido.
- Não verificado: se o corpo do `PSSI` está mascarado nesta versão; a `.ksy` do
  repositório não tem desmascaramento.

Isso também ajuda a decisão pendente sobre rótulos de frase (matriz de
aceitação): permite medir a concordância de rótulos com o rekordbox em centenas
de faixas antes da anotação manual de 20.

### R1.3 — Matriz de fixtures

Incluir pelo menos:

- 120 BPM fixo;
- mudança 120 → 140 BPM;
- introdução silenciosa;
- primeira batida antes do frame zero;
- grade sem downbeat;
- áudio curto e silêncio;
- cues fracionários, oito hot cues, dois memory cues e loops;
- playlists vazias, aninhadas e homônimas;
- frases de 8, 16 e 32 compassos;
- faixa instrumental e faixa com segmentos vocais.

### R1.4 — Comparador diferencial

O comparador deve emitir JSON legível por máquina contendo métrica, esperado,
obtido, tolerância e resultado. Não esconder diferenças por reordenação quando a
ordem for semanticamente relevante.

**Gate da fase:** G1.

## Fase 2 — Interoperabilidade e biblioteca externa

### R2.1 — Fazer o crate existente compilar

- Corrigir erros reais apontados por `cargo check`, sem alterações especulativas.
- Cobrir CLI, parser XML, backup WAL, lock de escritor e rollback.
- Impedir importação no banco real durante testes.

### R2.2 — Preservação sem perda

- Modelar playlist por caminho/hierarquia, não apenas nome.
- Modelar memory cue sem usar `-1` como identidade única.
- Validar números finitos, cores, intervalos e sample rate.
- Rejeitar atributos e números XML inválidos com diagnóstico; não convertê-los
  silenciosamente para defaults.
- Normalizar e testar caminhos com drive Windows, UNC, Unicode, espaços e
  diferenças de caixa nos sistemas em que caixa não é significativa. Hoje
  `path_to_file_url` reproduz o `os.path.abspath` POSIX do `sync.py` e trata
  `C:\...` como caminho relativo.
- Rejeitar ou diagnosticar referências de faixa desconhecidas.
- Fazer backup antes da primeira escrita e limpar backup parcial em falha.
- Escrever XML por arquivo temporário no mesmo volume, sincronizar quando
  aplicável e substituir o destino atomicamente.

### R2.3 — Paridade semântica

Comparar o significado do XML e do banco, não apenas whitespace. O round-trip
deve provar que cues, loops, ordem de playlists e posições fracionárias não se
perdem.

O teste diferencial executa as duas CLIs sobre cópias idênticas e compara XML
normalizado, saída estruturada e estado final do SQLite. Um comparador que olha
somente XML manualmente não libera G2.

### R2.4 — Cutover da CLI

- Manter `sync.py` disponível como fallback durante um ciclo de validação.
- Executar Python e Rust em cópias do mesmo banco e comparar resultados.
- Só tornar Rust o comando documentado após teste com um XML real autorizado.

**Gate da fase:** G2.

## Fase 3 — Domínio musical puro

Cada item começa por testes de caracterização C++ e termina com testes
diferenciais Rust.

### R3.1 — TempoOctave e BPM

Fonte: `src/track/tempooctave.h`.

- Portar normalização 1/2× e 2× e faixas preferidas.
- Usar testes de tabela e propriedades: idempotência, positividade e limite da
  faixa escolhida.
- Exigir igualdade exata onde a operação é discreta e ±0,01 BPM na saída
  apresentada ao usuário.

### R3.2 — Tonalidade e Camelot/Open Key

- Extrair conversões e compatibilidade de `KeyUtils` sem dependência de widget.
- Cobrir todas as tônicas, modos, vizinhos e relativos.
- Fixar representação canônica; strings localizadas pertencem à UI.

### R3.3 — BeatGrid e downbeat

- Criar iterador Rust para grid fixo e variável.
- Portar busca de batida, translação, escala, posição no compasso e fase.
- Ler o protobuf do Mixxx apenas no adaptador; domínio usa seus próprios tipos.
- Cobrir índices negativos de downbeat e grades antes do início da faixa.

### R3.4 — Segmentação de frases

Fonte: `src/track/phrases.cpp`.

- Primeiro portar `segmentPhrases` recebendo características prontas.
- Manter intervalos semiabertos e comportamento sem grid/downbeat.
- Comparar limites, tipos, número de compassos e confiança.
- Não juntar esta tarefa à extração FFT da Fase 4.

### R3.5 — Mix Points e cues automáticos

Fontes: `src/track/mixpoints.cpp` e `src/track/autophrasecues.cpp`.

- Separar cálculo de pontos da mutação da faixa.
- Retornar um plano declarativo de operações em cues.
- Cobrir preservação de slots do usuário, idempotência e undo.

### R3.6 — Relacionadas

- Separar score puro da consulta SQLite.
- Cobrir metade/dobro de BPM, limite de 6%, compatibilidade harmônica,
  desempate determinístico e ausência de energia.
- O adaptador da biblioteca aplica paginação e cancelamento.

**Gate da fase:** G3.

## Fase 4 — Análise de áudio em lote

Primitivas de PCM e FFT pertencem a `vilelabox-dsp`; coordenação e algoritmos
de alto nível pertencem a `vilelabox-analysis`; caches persistentes pertencem a
`vilelabox-storage`. Essa separação permite reutilizar DSP no engine sem levar
SQLite, subprocessos ou logs ao caminho de áudio.

### R4.1 — Contrato PCM

Fixar canais, interleaving, faixa numérica, endianness, sample rate e política
para NaN/Inf. Toda análise recebe blocos e precisa produzir o mesmo resultado
independentemente do tamanho dos blocos.

### R4.2 — Energia

- Portar loudness, ataques e proporção de graves.
- Decidir entre implementação própria e biblioteca Rust após auditoria de
  licença e precisão.
- Comparar score final e tempo de execução em áudio sintético e real autorizado.

### R4.3 — Características de frases

- Portar janelamento, FFT, RMS, bass, flux, chroma e timbre.
- Testar 44,1/48/96 kHz, blocos quebrados em posições diferentes e faixa curta.
- Só então conectar ao `segmentPhrases` já aprovado.

### R4.4 — Grid e downbeat

- Encapsular a implementação Queen Mary existente como oráculo.
- Portar refinamento e detecção sem copiar dependências incompatíveis.
- Aprovar ≤1 ms no sintético e ≤25 ms no conjunto real de referência.
- Downbeat deve ter zero batidas de diferença.

### R4.5 — Vocal

Primeira versão Rust apenas orquestra o processo Python/Demucs:

- descoberta do interpretador;
- probe de engine e versão;
- timeout, cancelamento e encerramento de filhos;
- cache que inclui script, modelo, engine e versão efetiva;
- validação estrita do JSON.

Portar a inferência para Rust é uma decisão posterior; não bloquear o restante
da migração nisso.

### R4.5a — Modelos do rekordbox como oráculo (opcional)

`models/` do rekordbox 7.2.8 tem `detect_beat` (Keras), `detect_cue_model`,
`model_energy`, `model_genre` e `umap_embedder` (ONNX). Podem ser executados
localmente, por `xtask` ou script, para comparar resultados nos casos de
`tests/reference/`. Nunca são dependência do produto nem do CI. Instalar
`onnxruntime` depende de autorização do usuário.

### R4.6 — CLI de análise

Criar um executável batch que recebe áudio/fixture e devolve o schema canônico.
Ele será usado pelo CI, pelo modo sombra no Mixxx e pela futura biblioteca.

**Gate da fase:** G4.

## Fase 5 — Integração transitória com o Mixxx

O objetivo é provar Rust no produto atual antes de depender da nova UI.

### R5.1 — Spike de fronteira

Comparar duas opções em um protótipo pequeno:

- processo externo com JSON para análises batch;
- biblioteca estática com ABI C estreita para algoritmos síncronos puros.

Escolher com medidas de build, diagnóstico, cópia de memória e portabilidade.
Não expor `QString`, `QVector`, `std::shared_ptr` ou tipos Rust pela ABI.

### R5.2 — Regras da ABI

- estruturas `repr(C)` versionadas;
- buffers pertencem ao chamador;
- erros retornam código e mensagem copiada para buffer fornecido;
- nenhuma exceção ou panic atravessa a fronteira;
- funções da ABI nunca guardam ponteiros de objetos Qt;
- nenhuma chamada FFI por amostra ou por buffer do callback de áudio.

### R5.3 — Modo sombra

Para cada módulo aprovado:

- C++ continua produzindo a resposta ativa;
- Rust recebe a mesma entrada em thread não-RT;
- diferenças são gravadas de forma limitada e sem dados pessoais;
- um relatório agrega taxa e magnitude das divergências.

### R5.4 — Feature flags

Cada substituição terá três estados: `legacy`, `shadow`, `rust`. O padrão só
muda para `rust` depois do gate específico; o rollback não exige recompilar.

**Gate da fase:** dois ciclos completos de comparação sem divergência fora da
tolerância e aprovação do usuário para ativar Rust.

## Fase 6 — Biblioteca e núcleo independentes

### R6.1 — Banco próprio

- Criar schema e migrations versionadas do VILELABOX.
- Guardar localização, identidade de arquivo, metadados, análises, cues,
  playlists e histórico.
- Importar `mixxxdb.sqlite` somente por conexão read-only.
- Nunca usar o banco do Mixxx como banco de produção do aplicativo Rust.

### R6.2 — Repositórios e serviços

- APIs de domínio independentes de SQLite.
- Transações explícitas para alterações de playlist/cues.
- Jobs de análise idempotentes e canceláveis.
- Limite e política de invalidação para caches.

### R6.3 — `vilelabox-core`

Modelar comandos e eventos:

- carregar/ejetar faixa;
- play, pause, seek, cue e loop;
- alterar grid, tom e tempo;
- iniciar/cancelar análise;
- aplicar plano de cues;
- selecionar relacionada e preparar transição.

O core não conhece widgets nem dispositivos de áudio concretos.

### R6.4 — Aplicativo headless

Antes da UI, um binário deve importar uma biblioteca, analisar fixtures,
pesquisar, editar playlists e exportar os resultados.

**Gate da fase:** G5.

## Fase 7 — Interface desktop e waveform

### R7.0 — Especificação de layout

`PLANO-LAYOUT.md` produz, para a skin RbxNative do Mixxx, uma tabela de regiões
em pixels, a paleta medida e o mapa de arte (`third_party/rekordbox/`). Esses
artefatos são independentes de framework e servem de especificação de tela para
os spikes do R7.1; a skin XML em si é descartada com o Mixxx.

A arte do rekordbox não pode ser distribuída com o aplicativo GPL. A UI Rust
nasce com arte própria; carregar um tema local a partir da instalação do usuário
é uma opção a decidir, desligada por padrão.

### R7.1 — Escolher framework

Fazer spikes equivalentes em egui/eframe e Slint usando a mesma tela: biblioteca,
dois decks sem áudio e waveform estática. Medir tamanho, consumo, acessibilidade,
integração com GPU e empacotamento. A escolha é uma decisão do usuário baseada
nos resultados, não preferência do agent.

### R7.2 — Fluxo de preparação

Implementar nesta ordem:

1. biblioteca, pesquisa e playlists;
2. painel de metadados e análise;
3. waveform/overview com grid, compassos, frases, vocal e cues;
4. editor de grid e downbeat;
5. cues, loops, relacionadas e configurações.

### R7.3 — Modelo UI/core

- UI envia comandos e recebe snapshots/eventos.
- Nenhum widget acessa SQLite diretamente.
- Operações longas são assíncronas e canceláveis.
- Estado visual não altera o modelo musical implicitamente.

**Gate da fase:** G6, incluindo validação visual em resoluções suportadas.

## Fase 8 — Motor de áudio em tempo real

Esta fase começa apenas com G5 e G6 aprovados.

### R8.1 — Regras RT

No callback de áudio são proibidos:

- alocação/desalocação;
- mutex, filesystem, SQLite, logs e formatação de strings;
- chamadas de rede ou subprocessos;
- crescimento de coleções;
- panic/unwind;
- FFI por buffer para o engine legado.

Comandos entram por fila limitada; métricas saem por atomics ou fila lock-free.

### R8.2 — Sequência do engine

1. enumerar dispositivo e tocar silêncio/tom;
2. decodificar e tocar uma faixa;
3. play/pause/seek/cue com um deck;
4. dois decks e mixer;
5. rate, pitch e keylock;
6. beat sync e BarSync;
7. loops, beatjump e Slicer;
8. Mix Point Link;
9. Release FX e demais efeitos;
10. gravação e roteamento, se mantidos no escopo.

Cada degrau tem benchmark e teste de longa duração antes do próximo.

### R8.3 — Critérios RT iniciais

- callback p99 abaixo de 25% da duração do buffer no equipamento de referência;
- zero underruns internos em teste contínuo de duas horas;
- validar 44,1 e 48 kHz, buffers de 128 e 256 frames;
- troca de faixa, seek e ativação de efeitos sem estalo audível;
- Mix Point dentro de um buffer de saída do ponto calculado;
- nenhuma alocação detectada no callback em build de teste.

Os números devem ser registrados junto com hardware, backend e versão do SO.
Antes do release candidate, ampliar o soak para oito horas, registrar p99.99,
xruns, CPU e memória, e incluir buffer de 64 frames quando o hardware de
referência o suportar.

### R8.4 — Shadow e cutover

O novo engine não roda simultaneamente no mesmo dispositivo com o Mixxx. A
comparação usa renderização offline determinística e sessões A/B separadas.

**Gate da fase:** G7 e preenchimento da validação real.

## Fase 9 — MIDI e performance

### R9.1 — Contrato de comandos

Mapeamentos MIDI geram os mesmos comandos tipados usados pela UI. Eles não
acessam o engine diretamente.

### R9.2 — Formato de mapeamento

Definir schema versionado para mensagem, canal, faixa, curva, feedback e comando.
Validar entradas e impedir comandos inexistentes.

### R9.3 — Primeiro dispositivo

Selecionar uma controladora com o usuário e validar transporte, mixer, cues,
loops, sync, pads, LEDs e reconexão. Os mapeamentos do rekordbox
(`MidiMappings/*.midi.csv`, 67 controladoras) e as definições de pads
(`pad/*.pad.csv`) podem servir de referência local; não são versionados. Suporte genérico sem hardware real não
aprova a fase.

## Fase 10 — Cutover e retirada do legado

### R10.1 — Migração de dados

- backup verificável;
- dry-run com relatório;
- importação repetível sem duplicar dados;
- rollback documentado;
- nunca apagar automaticamente o perfil do Mixxx.

### R10.2 — Release candidate

Executar a matriz completa: Linux, Windows, fixtures, 20 faixas anotadas,
hardware de áudio, controladora e XML real autorizado.

### R10.3 — Remoção do C++

A remoção é uma tarefa própria. Exige G8, autorização do usuário e confirmação
de que nenhuma funcionalidade necessária depende mais do executável do Mixxx.
O histórico e os documentos de atribuição permanecem.

## Matriz de aceitação por capacidade

| Capacidade | Critério mínimo |
| --- | --- |
| BPM | diferença ≤0,01 BPM |
| Grid sintético | erro ≤1 ms |
| Grid real | erro ≤25 ms contra referência validada |
| Downbeat | zero batidas de desvio |
| Frases | ≥80% dos limites a até um compasso em pelo menos 20 faixas |
| Rótulos de frase | informar acurácia; meta depende de decisão posterior |
| Vocal | bordas dentro de 250 ms e matriz de falsos positivos/negativos |
| Mix Points | mesmo compasso/ponto segundo as regras congeladas |
| Auto cues | zero alteração de cue do usuário; operação idempotente |
| Relacionadas | mesma elegibilidade e ordem determinística para scores iguais |
| Rekordbox XML | round-trip sem perda nos campos suportados |
| Biblioteca | importação idempotente e rollback após falha injetada |
| Áudio | critérios RT da Fase 8 e validação auditiva |

## Definition of Done de qualquer tarefa Rust

Uma tarefa só recebe status **feito** quando:

- comportamento e não-objetivos estão documentados;
- código não usa `unwrap`/`expect` em caminhos alimentados pelo usuário;
- testes unitários cobrem caminho feliz, limites e entrada inválida;
- teste diferencial ou justificativa de inexistência foi registrado;
- `cargo fmt`, `cargo clippy` e `cargo test` passaram;
- dependências novas têm finalidade, licença e versão justificadas;
- formato persistido possui versão e política de migração;
- desempenho foi medido quando a tarefa toca análise, consulta ou áudio;
- docs distinguem implementado, experimental e planejado;
- agent revisor conferiu o handoff e o principal repetiu os testes.

## Backlog inicial para os agents

| Ordem | Pacote | Agent de portabilidade | Agent de validação | Dependência |
| --- | --- | --- | --- | --- |
| 1 | `R0-toolchain-ci` | Tooling | Revisor CI | autorização para instalar Rust |
| 2 | `R0-rekordbox-spec` | Interoperabilidade | Dados/SQLite | decisão sobre conflitos |
| 3 | `R1-types` | Domínio | Property tests | G0 |
| 4 | `R1-mixxx-compat` | Adaptador legado | Formato/segurança | `R1-types` |
| 5 | `R1-reference-xtask` | Fixtures/tooling | Diferencial | `R1-types` |
| 6 | `R2-rekordbox` | Interoperabilidade | Segurança de dados | G1 |
| 7 | `R3-tempo-key` | Domínio musical | Diferencial | `R1-types` |
| 8 | `R3-beatgrid` | Grid | Diferencial | `R1-types` |
| 9 | `R3-phrases` | Estrutura musical | Diferencial | `R3-beatgrid` |
| 10 | `R3-mix-cues` | Preparação | Integridade de cues | `R3-phrases` |
| 11 | `R3-related` | Biblioteca musical | Performance/ordem | tempo, key e energia |
| 12 | `R4-dsp-energy-features` | DSP batch | Numérico/performance | G3 |
| 13 | `R4-grid-analysis` | DSP/grid | Tolerâncias | contratos de grid |
| 14 | `R4-vocals` | Processos/cache | Cancelamento | schema de referência |
| 15 | `R5-mixxx-shadow` | Integração | Revisor C++/Rust | G4 |
| 16 | `R6-storage-library-core` | Persistência/core | Migrações | G2 e G3 |
| 17 | `R7-ui-spike` | UI | UX/performance | G5 |
| 18 | `R8-audio` | Áudio RT | Especialista RT | G5 e G6 |
| 19 | `R9-midi` | MIDI | Hardware | G7 |

## Decisões que pertencem ao usuário

O trabalho deve pausar antes destas decisões:

1. autorizar instalação/fixação do Rust e geração de `Cargo.lock`;
2. decidir se mantém handoffs de patch ou autoriza worktrees isoladas e a
   alteração de governança necessária para agents implementadores;
3. definir a semântica desejada da ponte para grid, metadados, pastas e
   conflitos;
4. autorizar o primeiro cutover Python → Rust;
5. escolher ABI/processo após o spike da Fase 5;
6. escolher egui/eframe ou Slint após o spike da UI;
7. escolher biblioteca/backend do motor de áudio depois da prova técnica;
8. escolher a primeira controladora;
9. autorizar remoção de cada implementação legada e, por último, do fork C++;
10. stems: migrar a base para o Mixxx 2.6 ou deixar para o motor Rust (o README
    registra que as duas coisas competem; o rekordbox traz `Demucs3.dll` e
    `Spleeter.dll`, mas não há decisão);
11. prioridade entre a skin RbxNative (Mixxx, descartável) e a Fase 0 deste
    plano.

## Registro de execução

Criar em `IMPLEMENTACAO.md` uma seção por gate. Cada pacote integrado registra:

| Campo | Conteúdo |
| --- | --- |
| Pacote | ID da tarefa |
| Fonte legada | arquivos e commits lidos |
| Destino Rust | crate e arquivos |
| Agent/handoff | identificação do handoff revisado |
| Testes | comandos e resultados reais |
| Diferencial | fixture, tolerância e divergências |
| Desempenho | ambiente e medidas, quando aplicável |
| Status | feito, bloqueado ou revertido |
| Próximo gate | condição restante |

O plano avança por evidência e gates, não por quantidade de código traduzido.
