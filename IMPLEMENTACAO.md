# Execução do plano VILELABOX

Base: Mixxx 2.5.6. Branch `blackmesa`. Decisão do usuário: manter 2.5.6 e adiar
stems; nenhuma controladora será mapeada.

## Organização

Os módulos prévios foram preservados em cinco commits: compassos, grid,
tonalidade, vocal/renderização e empacotamento/skins. Renderização compartilha
arquivos de vocal e compassos; o commit registra essa dependência. As skins
RbxOriginal e RbxFlowBars agora têm cópias versionadas em `res/skins/`.
Backups locais, testes auxiliares e links para `~/mixxx-rekordbox` são excluídos
via `.git/info/exclude`.

`analyze-ml.py` foi aposentado e seus modelos proprietários não são importados.
O script anterior foi preservado em backup externo antes da substituição.
O bridge de sincronização foi versionado em `tools/rekordbox/sync.py` e instalado
no destino do link: exige Mixxx fechado, bloqueia escritores concorrentes,
cria backup SQLite consistente (incluindo WAL) e reverte a transação se falhar.
Os quatro testes em `tools/rekordbox/test_sync.py` passaram.

## Módulos

Documentação: FRASES.md, CUES.md, MIXPOINT.md, RELACIONADAS.md e PADS.md.
A fase 6 está adiada por decisão explícita. Os recursos novos são experimentais.

## Verificação inicial

A compilação inicial passou. A suíte completa inicial encontrou quatro falhas
em `ControllerScriptEngineLegacyTimerTest` (singleShot bind/arrow e classes) e
bloqueou em `ControlObjectAliasTest.PlayerManager` ao inicializar o dispositivo
de áudio indisponível; a execução bloqueada foi interrompida. Log inicial:
`/tmp/vilelabox-baseline.log`. Essas falhas precedem os módulos novos.

Validação auditiva de compassos/grid/tom e visual de vocal permanece pendente;
este ambiente não acessa a sessão gráfica e seu dispositivo de áudio.
Também está pendente a anotação manual de 20 faixas reais para frases.
Não há declaração de build limpo ou de aprovação de toda a suíte até concluir
as verificações finais registradas abaixo.

## Verificação final

Compilação completa com `cmake --build build --parallel 4`: aprovada.
22 testes direcionados de frases/cues/energia/sync/pads/link: aprovados.
Quatro testes Python do bridge: aprovados, incluindo rollback, bloqueio de
escritor, Mixxx aberto e backup incluindo WAL. Os XMLs das duas skins são
válidos e os verificadores de templates, imagens e SVGs passaram.

A regressão completa, com `QT_QPA_PLATFORM=offscreen` e saída ALSA nula, executou
874 testes: 870 passaram e as mesmas quatro falhas anteriores de temporizadores
JavaScript permaneceram. Há 13 testes desabilitados no projeto. Portanto, o
critério literal da fase 0 de todos os testes passarem continua pendente dessas
falhas preexistentes; elas não foram ocultadas ou desabilitadas por este mod.
Relatórios: `/tmp/vilelabox-modules.xml` e `/tmp/vilelabox-regression.xml`.

A tentativa de inicialização gráfica em perfil isolado encontrou avisos e
bloqueios de diálogo do ambiente offscreen. Não é validação visual concluída.
É necessário conferir no desktop as skins e ouvir os recursos de áudio.

As skins instaladas e o bridge foram atualizados com backup em
`~/.mixxx/backups-vilelabox/20261009-152303/` e
`20261009-152655/` (indicadores finais do Link). Reinicie o VILELABOX para carregar
as mudanças. Abra PADS no deck para os novos modos; habilite as análises opcionais
nas preferências de BPM, ou use Análise → Reanalisar frases no menu da faixa.
Stems permanecem adiados e `res/controllers/` não foi alterado.

## Plano 3 — consolidação (2026-10-12)

Branch `vilelabox-publico`. Remotos: `origin` é o Mixxx oficial (nunca enviar);
`vilelabox` é https://github.com/Thiagovilela2001/VILELABOX. O hook local
`pre-push` roda `tools/check-third-party-assets.sh`; a existência do remoto não
substitui essa verificação.

| Fase | Commit | Evidência |
| --- | --- | --- |
| 1. Frases fora do callback | `f955b9b` | `BarSyncTest.PhraseStateIgnoresLateUpdatesForPreviousTrack`, `RapidTrackSwapsPublishOnlyTheFinalTrack`; trackLoaded sem alocação, lock ou fila (revisão do código). Validação com buffers pequenos no dispositivo real: pendente (sem acesso ao áudio). |
| 2. Relacionadas assíncronas | `38c6ec6` | `RelatedTracksTest.*`; busca medida: 1.000 faixas 26 ms, 10.000 faixas 201 ms, 50.000 faixas 982 ms (biblioteca sintética, 25% compatíveis). Biblioteca real: 11 faixas. |
| 3. Verificação unificada | `0b2dc72` | `tools/verify-mod.sh`; build do zero desse commit em diretório separado e verificação completa: OK. Workflow `.github/workflows/vilelabox.yml` escrito, não executado no GitHub. |
| 4. Vocal cancelável | `d2cb713` | `AnalyzerVocalsTest.DiagnosesEachFailure`, `TimeoutKillsTheDetector`, `CancellingStopsTheDetectorWithoutWaitingForTimeout` (cancelamento em ~300 ms). |
| 5. Cache e memória | `8a6505d` | `AnalyzerPhrasesTest.GridEditsWriteCacheOnceAfterTheySettle`, `OlderPhraseWriteNeverReplacesNewer`, `ReadsSingleFileCachesFromEarlierVersions`, `MemoryKeepsTracksInUseAndTheMostRecent`, `FailedPhraseWriteKeepsTheResultInMemory`; 184 bytes por bloco de 50 ms (~13 MB/h de áudio). |
| 6. Validação real | — | Preparada: `VALIDACAO.md` e `tools/phrases/export_csv.py`. Sessão com o usuário pendente. |
| 7. Correção manual de frases | — | Aguarda a decisão de ancoragem (tempo, posição musical ou híbrido). |

> **Nota (2026-10-22):** Os remotos e branches citados acima (`blackmesa`, `vilelabox-publico`, `vilelabox`) refletem o histórico do desenvolvimento. Atualmente, há apenas o remoto `origin` = `https://github.com/Thiagovilela2001/VILELABOX` e o branch ativo é `main`. O histórico anterior foi integralmente preservado.

## Plano 4 — Próximos passos (etapas 0 a 3)

| Tarefa | Arquivos alterados | Comando de verificação | Resultado |
| --- | --- | --- | --- |
| A1. Corrigir referência 2.5.6 inexistente | `tools/verify-mod.sh`, `tools/check-third-party-assets.sh` | `grep -n "2\.5\.6\.\." tools/` | Passou (`tools/verify-mod.sh` marcado como não executado no Windows) |
| A2. Docs desatualizados | `VOCALS.md`, `tools/rekordbox/sync.py`, `IMPLEMENTACAO.md` | `python tools/rekordbox/test_sync.py` | Passou (4 testes) |
| A3. CI herdado do Mixxx | `.github/workflows/`, `.github/workflows-mixxx/` | `git status .github/` | Pendente: workflows herdados movidos para `.github/workflows-mixxx/`, `vilelabox.yml` e `pre-commit.yml` ativos. O critério de pronto (registro da primeira execução do `vilelabox.yml`) depende de commit/push e não foi cumprido |
| A4. Canais de contribuição | `CONTRIBUTING.md`, `CONTRIBUTING.mixxx.md`, `.github/ISSUE_TEMPLATE/config.yml`, `README.md` | `grep -rn "zulip\|mixxx.org/forums" CONTRIBUTING.md .github/ISSUE_TEMPLATE` | Passou (nenhum resultado) |
| B1. Medidor da anotação de frases | `tools/phrases/score_csv.py`, `tools/phrases/test_score_csv.py`, `tools/verify-mod.sh` | `python tools/phrases/test_score_csv.py` | Passou (4 testes) |
| B2. Roteiro de validação | `VALIDACAO.md` | Revisão do fluxo de uso em `VALIDACAO.md` | Passou |
| C1. Inventário técnico | `INVENTARIO.md` | Leitura estática de código e mapeamento dos 15 recursos | Corrigido em 2026-10-22: citações refeitas a partir de `git diff 3ebac449e7 HEAD` (várias apontavam para linhas inexistentes ou sem relação) |
| D1. Exportador de resultados | `tools/reference/export_reference.py`, `tools/reference/test_export_reference.py`, `tools/verify-mod.sh` | `python tools/reference/test_export_reference.py` | Passou (exportação de faixas reais da biblioteca do usuário pendente de execução no Linux) |
| D2. Casos sintéticos | `tools/reference/make_synthetic.py`, `tests/reference/synthetic/*.json` | `python tools/reference/make_synthetic.py` | Passou (3 casos sintéticos gerados no Windows com WAVs ignorados no git) |
| D3. Tolerâncias | `tests/reference/README.md` | Documentação das tolerâncias com evidências citadas | Passou |
| E1. Workspace Rust e crate rekordbox | `rust/Cargo.toml`, `rust/README.md`, `rust/vilelabox-rekordbox/Cargo.toml`, `.gitignore` | Leitura da configuração e manifesto de workspace | Passou |
| E2. Reimplementação da ponte em Rust | `rust/vilelabox-rekordbox/src/lib.rs`, `rust/vilelabox-rekordbox/src/main.rs` | Inspeção de código e checagem de paridade com `sync.py` | Não executado: a primeira versão não compilava (derive inexistente) e divergia do Python; ver correções abaixo |
| E3. Testes de equivalência Rust | `rust/vilelabox-rekordbox/tests/sync_test.rs`, `rust/vilelabox-rekordbox/tests/compare_with_python.py` | `cargo test` no Windows | Passou em 2026-10-22 após as correções abaixo (Rust 1.99.1 MSVC) |

### Correções da revisão (2026-10-22)

| Item | Arquivos | Verificação | Resultado |
| --- | --- | --- | --- |
| Exportador: vocal pelo nome real do cache (identidade de `analyzervocals.cpp`), lista de batidas do BeatMap, frames int32 negativos, frases lidas uma vez, faixas sem áudio ignoradas e contadas, índice sem artista/título | `tools/reference/export_reference.py`, `tools/reference/test_export_reference.py`, `tests/reference/README.md` | `python tools/reference/test_export_reference.py` | Passou (4 testes) |
| Tolerâncias: fontes corrigidas (`analyzerqueenmarybeats.cpp:169`, `vocal_detector.py:206`, `FRASES.md:90`), links relativos, rótulo de frase sem meta | `tests/reference/README.md` | Conferência das linhas e âncoras | Passou |
| Inventário e estado do Rust nos docs | `INVENTARIO.md`, `README.md`, `CLAUDE.md` | Toda citação `arquivo:linha` dentro do arquivo e no trecho do diff do mod | Passou |
| Rust: correções de compilação (derive removido, `config_mut().trim_text`), trava antes do backup (`protected_import`), checagem por `exe`, URL igual a `quote(safe="/")` + `abspath`, posições REAL, unescape de atributos, `<TRACK/>` sem cues, playlist vazia, erros SQL não engolidos, padrões `or` do Python, `round` com empate par, pasta de histórico; 4 testes do Python portados; comparador sem reordenar filhos; `serde` removido | `rust/vilelabox-rekordbox/**`, `rust/README.md` | `cargo check`, `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` | Passou (Windows, Rust 1.99.1): 8 testes. Primeira execução falhou no teste de ida e volta (caminho `C:\` tratado como relativo); corrigido com suporte a unidade Windows (`/C:/...`, como o rekordbox) |

## Plano de layout — skin RbxNative

| Fase | Decisões / Conteúdo | Arquivos | Verificação | Resultado |
| --- | --- | --- | --- | --- |
| 0. Decisões do usuário | (1) Layout alvo: 2Deck Horizontal; (2) Resolução alvo: 1920×1080 (mínimo 1600×860); (3) Armazenamento do XML: tudo local em `third_party/rekordbox/skins/RbxNative/` | `IMPLEMENTACAO.md` | Respostas registradas | Passou |
| 1. Captura de referência | Roteiro das 8 capturas em tela cheia na resolução 1920×1080 com rekordbox 7.2.8 | `third_party/rekordbox/ref/README.md` | Diretório preparado; aguarda capturas manuais no rekordbox pelo usuário | Pendente (usuário) |
| 2. Medição das regiões | Tabela de coordenadas, largura e altura em pixels de cada painel baseada nas capturas da Fase 1 | `third_party/rekordbox/ref/regioes.md` | Aguarda imagens da Fase 1 | Pendente da Fase 1 |
| 3. Mapa de arte | Mapeamento completo das 94 imagens da RbxOriginal contra a pasta de skins do rekordbox 7.2.8 | `third_party/rekordbox/skins/arte.csv` | Todas as 94 imagens listadas como mapeadas (38) ou sem equivalente (56) | Passou |
| 4. Montagem e instalação | Cópia do template RbxNative com grupo `[RbxNative]` e script de instalação com verificação estrita de segurança e cópia para `rb-native/` | `third_party/rekordbox/skins/RbxNative/`, `third_party/rekordbox/install_rbxnative.py` | Execução de teste do instalador; nenhum arquivo gerado fora de `third_party/rekordbox/` | Passou |
| 5. Geometria | Ajuste fino de posições e dimensões dos painéis baseado em `regioes.md` | `third_party/rekordbox/skins/RbxNative/` | Aguarda medições da Fase 2 | Pendente da Fase 2 |
| 6. Recursos sem equivalente | Ocultar ou deixar vazios espaços de botões não existentes no mod | `third_party/rekordbox/skins/RbxNative/` | Aguarda alinhamento com a geometria | Pendente da Fase 5 |
| 7. Verificação | Validação no Linux com `check_refs.py`, log limpo no perfil de teste e ausência de arquivos proprietários no git | `third_party/rekordbox/install_rbxnative.py`, `tools/check-third-party-assets.sh` | Aguarda fases anteriores e teste em sessão real | Pendente |
| Equivalência Python × Rust na exportação | `rust/vilelabox-rekordbox/tests/equivalence.py`, `rust/Cargo.lock` | `python rust/vilelabox-rekordbox/tests/equivalence.py` | Passou: XML idêntico em banco com caracteres especiais, posições fracionárias, loop, memory cue, faixa apagada, BPM 0, empate de arredondamento e histórico. O comparador acusa uma mudança de atributo (teste de mutação manual). Importação comparada só pelos testes Rust, não contra o Python |

## Build no Windows (2026-10-22)

| Item | Arquivos | Verificação | Resultado |
| --- | --- | --- | --- |
| Scripts de build e execução | `build-windows.bat`, `run-windows.bat` | `build-windows.bat` | Passou: `build-win/mixxx.exe` gerado (MSVC 14.44, CMake 4.4.4, Ninja 1.13.2, dependências `mixxx-deps-2.5-x64-windows-c15790e`, SHA-256 conferido). `BUILD_TESTING=OFF` e `BUILD_BENCH=OFF`: testes do mod usam `/tmp` e `::kill` (`src/test/analyzergrid_test.cpp:301`, `src/test/phrasestest.cpp:385`) |
| Compatibilidade MSVC | `src/track/autophrasecues.cpp:65` | compilação | `near` é macro do `windef.h`; renomeado para `isNear`, sem mudança de comportamento. gtest no Linux: não executado |
| Execução no Windows | — | abrir com `run-windows.bat` | Não executado |

## Mixer da RbxOriginal no padrão rekordbox (2026-10-22)

Referência: o mixer nas capturas de tela do próprio instalador do rekordbox 7.2.8
(`tutorial/12_17.png`, `tutorial/DDJ200/mix_10.png`, `tutorial/DDJ200/setup_load_001.jpg`).

| Item | Arquivos | Verificação | Resultado |
| --- | --- | --- | --- |
| Mixer com dois canais estreitos, sem faixa master; knob de anel com traço; rótulo embaixo; HIGH/MID/LOW como botões de kill; COLOR FX entre CFX e CUE; fader fino com linha azul e marcas; medidor que acompanha o fader; fone e crossfader embaixo; caixa MIXER passa a esconder o mixer | `res/skins/RbxOriginal/mixer/{mixer,channel_knobs,channel_fader,eq_knob,fader_ticks}.xml`; removidos `channel.xml`, `master.xml`, `vu_channel.xml`, `vu_master.xml`, `colorfx_btn.xml` | `python tools/check_refs.py` (na skin); XML bem-formado; captura da janela no Windows | Passou: referências resolvem; janela capturada com o layout novo |
| MASTER na barra superior (medidor L/R + slider), como no rekordbox | `res/skins/RbxOriginal/topbar.xml` | captura da janela | Passou |
| Desenhos novos (knob, kill, alças, crossfader, fone, medidores PNG) | `res/skins/RbxOriginal/tools/make_rb.py`, `bases/`, `rb/` | rodar `make_rb.py` e conferir que os arquivos antigos saem idênticos | Passou (só os 14 arquivos novos mudaram) |
| Altura mínima do deck 360 → 480: o conteúdo do deck dependia do mixer antigo, mais alto; sem isso as abas de pads cobriam BAR LOOP | `res/skins/RbxOriginal/decks/deck.xml` | captura antes/depois | Passou |
| Anel dos knobs da skin #2a2d33 → #3a3a3a (o do rekordbox) | `res/skins/RbxOriginal/controls/knob.xml` | captura | Passou |
| BAL, BOOTH e A/THRU/B saíram da skin (o rekordbox não os mostra); os controles do Mixxx continuam | — | — | Decisão registrada no README da skin |
| Linux (`tools/verify-mod.sh`, capturas com `shoot_nested.sh`) e uso real | — | — | Não executado |

## Migração Rust — Fase 0 (G0) e Oráculo rekordbox (R1.2a) (2026-10-22)

| Item | Arquivos | Verificação | Resultado |
| --- | --- | --- | --- |
| R0.1 — Base salva em 4 commits | Repositório git | `git log -n 4 --oneline` | Passou: `6c9833ed4b` (Plano 4 / docs), `46b7a99c7d` (Rust inicial), `e26eba8e6a` (Build Windows / MSVC), `957386a1d0` (Mixer RbxOriginal) |
| R0.2 — Toolchain fixada | `rust-toolchain.toml`, `rust/rust-toolchain.toml`, `rust/vilelabox-rekordbox/Cargo.toml` | `cargo --version`, `cargo check` | Passou: toolchain fixada em 1.99.1 com `rust-version = "1.75"` |
| R0.3 — CI Rust independente | `.github/workflows/rust.yml`, `.github/workflows/pre-commit.yml` | Validação de sintaxe e gatilhos de push/pull_request | Passou: jobs para Windows e Ubuntu com `fmt`, `clippy -D warnings` e `test`. Pre-commit com gatilho direto ativado |
| R0.4 — Regras da ponte no Python e Rust | `tools/rekordbox/sync.py`, `tools/rekordbox/test_sync.py`, `rust/vilelabox-rekordbox/src/lib.rs`, `rust/vilelabox-rekordbox/tests/sync_test.rs`, `tests/equivalence.py` | `python tools/rekordbox/test_sync.py`, `cargo test`, `cargo clippy`, `python tests/equivalence.py` | Passou: 7 regras aprovadas (memory cues ±1ms, hierarquia de playlists `Pasta/Set`, contagem de faixas ativas em `Entries`, grid variável com múltiplos `TEMPO` de `beats.proto`, retenção de 10 backups). 8/8 testes Python, 9/9 testes Rust e equivalência estrita de saída XML aprovados |
| R1.2a — Exportador de análises ANLZ do rekordbox | `tools/reference/export_rekordbox_anlz.py`, `tools/reference/test_export_rekordbox_anlz.py`, `.gitignore` | `python tools/reference/test_export_rekordbox_anlz.py` e extração em faixas reais | Passou: decodificação de `PPTH`, `PQTZ` (beats), `PCOB`/`PCO2` (cues) e `PSSI` (frases com unmasking XOR de chave repetida de 19 bytes). 4/4 testes unitários e extração de análises reais do `%APPDATA%\Pioneer\rekordbox\share\PIONEER\USBANLZ` validados |

## Migração Rust — Fase 1: Contratos e Referência (2026-10-22)

| Item | Arquivos | Verificação | Resultado |
| --- | --- | --- | --- |
| R1.1 — Crate `vilelabox-types` | `rust/vilelabox-types/**`, `rust/Cargo.toml` | `cargo test -p vilelabox-types`, `cargo clippy -p vilelabox-types -- -D warnings` | Passou: newtypes com unidades explícitas (`SampleRate`, `Frame`, `EngineSample`, `Seconds`, `Bpm`), `MusicalKey` (Camelot/OpenKey e harmonia), `BeatGrid`, `Phrase`, `Cue`, `VocalSegment`, `TrackMetadata` e `AudioIdentity`. 5 testes unitários passando |
| R1.1a — Crate `vilelabox-mixxx-compat` | `rust/vilelabox-mixxx-compat/**`, `rust/Cargo.toml` | `cargo test -p vilelabox-mixxx-compat`, `cargo clippy -p vilelabox-mixxx-compat -- -D warnings` | Passou: isolamento de SQLite (`mixxxdb.sqlite`) e decodificador binário de `beats.proto`, convertendo para os tipos puros de `vilelabox-types`. 2 testes unitários de integração passando |
| R1.2 e R1.4 — Crate `vilelabox-reference` | `rust/vilelabox-reference/**`, `rust/Cargo.toml` | `cargo test -p vilelabox-reference`, `cargo clippy -p vilelabox-reference -- -D warnings` | Passou: schema canônico (JSON), normalizador e comparador diferencial com as tolerâncias oficiais (BPM ±0.01, downbeat 0, batidas 1 ms, frases até 1 compasso, cues 1 ms, vocais 250 ms), validado contra a fixture real `fixed_120bpm.json`. 2 testes unitários passando |

