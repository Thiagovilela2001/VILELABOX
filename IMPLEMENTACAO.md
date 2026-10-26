# Registro de implementação

## Estágios da passada A: energia por quarto e onset de agudos (P4.1, A1.1c) — 2026-10-26

| Campo | Resultado |
| --- | --- |
| Método | decompilação Ghidra conferida no assembly com capstone (`desmontar.py`, privado), para argumentos perdidos e ordem das somas |
| Recuperado | energia (soma dos quadrados com FMA e redução exata); filtro de 1ª ordem; energia por quarto de batida e por compasso (`0x1411885f0`); onset de agudos com passa-alta de 300 Hz, valor absoluto e passa-baixa de 5 Hz (`0x141178330`) |
| Crate | `vilelabox-dsp::estagios`: `energia`, `PrimeiraOrdem`, `energia_por_quarto`, `onset_agudos` |
| Testes | 5 unitários novos; workspace com 226 testes |
| Verificação | `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --workspace --all-features` e `xtask spec-lint` aprovados |
| Próximo gate | `0x141176680`, `0x1411786a0`, `0x141179530`, `0x141178df0` e `0x141172490` |

## Filtros do detector legado de BPM (P4.1, A1.1c) — 2026-10-26

| Campo | Resultado |
| --- | --- |
| Evidência | `spec/rekordbox/evidence/p4-1-analise-batida.md`, seções "Filtros do detector legado" e "Cadeia do fechamento da passada A" |
| Recuperado | biquad em forma direta II transposta; bilinear com pré-distorção por `tan` (confirmada pela série `x + x³/3`); Butterworth de 8ª ordem passa-baixa e passa-alta; decimação pela última amostra de cada grupo; bandas da passada A (fs/8·0,4, ÷8, até 210 s) e da passada B (fs/2·0,55 ÷2, fs/8·0,4 ÷8, fs/32·0,4 ÷32); grade de índices; o BPM e a 1ª batida de partida vêm dos dados conhecidos da faixa |
| Crate | `vilelabox-dsp::filtro`: `Biquad`, `Butterworth8`, `filtro_passada_a`, `bandas_passada_b`, `FiltrosEnvelope` (200, 10 e 100 Hz) com `envelope` e `diferenca`; constantes conferidas bit a bit com o binário |
| Testes | 7 unitários novos; workspace com 221 testes |
| Verificação | `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --workspace --all-features` e `xtask spec-lint` aprovados |
| Próximo gate | marcação de picos de `0x14118b160` lida no assembly; depois `0x1411885f0` e os demais estágios da passada A |

## Detector legado de BPM e posição de entrada (P4.1, A1.1b) — 2026-10-26

| Campo | Resultado |
| --- | --- |
| Evidência | `spec/rekordbox/evidence/p4-1-analise-batida.md`, seções novas de A1.1b |
| Recuperado | refinador de grade só no caminho de alta precisão; alta precisão = inferência de modelo (fora do escopo); detector legado na `CBeatAnalyzeThread` com duas passadas por bloco; entrada `f32` planar misturada em mono `f64` (0,99/32768 e 0,01/32768); bloco de `round(taxa · 1,70667)` frames; algoritmo completo da posição de entrada |
| Correção | a conclusão anterior "buffers `i16`, não `f32`" valia só para as somas de janela; a entrada do detector é `f32` |
| Crate | `vilelabox-dsp`: `posicao_de_entrada`, `mixar_mono`, `frames_por_bloco`, `PESO_CANAL_A`/`B` |
| Testes | 2 unitários novos; workspace com 214 testes |
| Verificação | `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --workspace --all-features` e `xtask spec-lint` aprovados |
| Próximo gate | A1.1c: passadas A/B (`0x1411953b0`, `0x141195480`, `0x141195ed0`) e C/D (`0x1411965b0`, `0x141197d60`) |

## Análise de BPM: dossiê p4-1 e `vilelabox-dsp` (P4.1, A1.1a) — 2026-10-26

| Campo | Resultado |
| --- | --- |
| Método | strings `__FUNCSIG__` do `analyzer::AnalyzeJobBeatDetector` convertidas em VA e usadas como `xref:` no projeto Ghidra; constantes lidas do `.rdata` |
| Evidência | `spec/rekordbox/evidence/p4-1-analise-batida.md`; nota na `20-bpm-grid.md` |
| Recuperado | modos legado e alta precisão; motor com dois tipos; BPM × 100 = `(int)(bpm·100 + 0,5)`; batida = posição, intervalo em ms e número; tabela de faixas de BPM (70–180 padrão e nove alternativas); dobra e divisão até a faixa; soma de janela `i16` de uma batida e meia batida |
| Crate | `vilelabox-dsp`: `FAIXAS_BPM`, `bpm_centesimos`, `bpm_centesimos_do_intervalo`, `ajustar_a_faixa`, `soma_batida`, `soma_meia_batida`; desvios marcados como ADAPTAÇÃO |
| Achado | o motor analisa buffers `i16`, não blocos `f32`; nenhuma FFT observada; A1.2 e A1.3 suspensos |
| Testes | 3 unitários novos; workspace com 212 testes (inclui a correção da ajuda da CLI) |
| Verificação | `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --workspace --all-features` e `xtask spec-lint` aprovados |
| Próximo gate | A1.1b: chamadores das somas de janela (objeto dos buffers `i16`) e o detector que roda sobre o PCM; `0x14247a210` é refinamento de grade, não detecção; depois teste diferencial com fixture WAV |

## Núcleo headless `vilelabox-core` e CLI (P3.3/P3.4, Gate G3) — 2026-10-26

| Campo | Resultado |
| --- | --- |
| Crate `vilelabox-core` | `Core::executar(Comando)` como ponto único de mutação; consultas `pesquisar`, `listas`, `faixas_da_lista`, `selecao`, `deck`; fila `drenar_eventos()` em ordem de emissão; ADAPTAÇÃO (desenho próprio) |
| Comandos | importar diretório/catálogo/XML, verificar localizações, buscar movidos, selecionar, criar lista, adicionar e mover na lista, carregar no deck, criar/mover/remover cue (origem `User`; loop move inteiro), salvar grade com bloqueio |
| Eventos | `BibliotecaAlterada`, `SelecaoAlterada`, `FaixaCarregada`, `PreparacaoAlterada`, `TarefaAnaliseAtualizada { pendentes }` (faixas sem identidade acústica) |
| Erros | `ErroCore` tipado: faixa inexistente, deck vazio, cue inexistente, posição inválida na playlist |
| Storage | `mover_na_lista`: reordenação 0-based numa transação, renumerando `track_no` |
| Crate `vilelabox-cli` | binário de desenvolvimento; cada chamada é um processo novo sobre o mesmo banco |
| Testes | 8 novos: 7 do core (eventos, seleção, playlist, erros, fluxo de cues, loop, grade) e `tests/fluxo.rs` (Gate G3: estado preservado entre processos) |
| Verificação | `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --workspace --all-features` (208 testes) e `xtask spec-lint` aprovados |
| Próximo gate | P4.1 `vilelabox-dsp` (fase A1 do `PLANO-ANALISE.md`), começando pelo dossiê G0 |

## Planos dos próximos passos — 2026-10-25

| Campo | Resultado |
| --- | --- |
| Documentos novos | `docs/PLANO-CORE-HEADLESS.md` (P3.3/P3.4: fases C1–C2 e Gate G3) e `docs/PLANO-ANALISE.md` (P4: fases A1–A2 e Gate G4) |
| Plano-mestre | `docs/IMPLEMENTATION-PLAN.md` com status 2026-10-25: estado atual do projeto, bloqueio do P2.3 pelo Gate G0 com as duas vias de desbloqueio, P3.1/P3.2 marcados como concluídos, P3.3/P3.4 e P4 apontando para os planos detalhados e "Próximos pacotes imediatos" reordenado (P3.3 → P3.4/G3 → P4.1 → P4.2/G4 → P5 → P6.3/G6 → P2.3 condicional → P7) |
| Verificação | documentação apenas; gates de código seguem verdes do commit anterior (200 testes) |
| Próximo gate | execução do P3.3 (fase C1 do `PLANO-CORE-HEADLESS.md`) |

## Biblioteca `vilelabox-library` (P3.2) — 2026-10-25

| Campo | Resultado |
| --- | --- |
| Crate | `vilelabox-library`: importação, busca, playlists e identidade de arquivos sobre o `vilelabox-storage` |
| Importadores | catálogo (`vilelabox-catalog`, que passou a projetar `SampleRate`) e XML (`vilelabox-xml`): faixas com metadados, cues convertidos para frames (origem `Imported`, preservando os do usuário), grade dos `TEMPO` e árvore de playlists; correspondência por caminho — reimportar atualiza, não duplica |
| Varredura | diretórios de áudio recursivos: faixa mínima pelo nome do arquivo, localização registrada, conhecidos só marcados como presentes |
| Busca | texto caixa-insensível em título/artista, universo da coleção ou de uma playlist, ordenação por coluna com desempate estável — mesma semântica do app |
| Identidade | SHA-256 em streaming (64 KiB), duplicatas por hash, marcação de ausentes no disco e reencontro de movidos pelo nome normalizado (caso único; ambíguo fica para o usuário) |
| Storage | cresceu para sustentar a biblioteca: `listar`, `atualizar`, `atualizar_identidade`, `id_por_localizacao`, `listar_localizacoes`, `marcar_localizacao` e `substituir_cues_por_origem` |
| Testes | 11 testes novos (5 importadores, 6 biblioteca): idempotência, preservação de cue do usuário, Rating 0..255 → 0..5, vetor SHA-256, movidos e duplicados |
| Verificação | `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --workspace --all-features` (200 testes) e `xtask spec-lint` aprovados |
| Próximo gate | `vilelabox-core` headless (P3.3): comandos, eventos e CLI de fluxo ponta a ponta |

## Banco próprio `vilelabox-storage` (P3.1) — 2026-10-25

| Campo | Resultado |
| --- | --- |
| Crate | `vilelabox-storage`: SQLite próprio (rusqlite bundled) sobre os tipos de `vilelabox-types` |
| Esquema v1 | `tracks`, `track_locations`, `cues`, `beatgrids` + `grid_beats`/`grid_tempo`, `phrases`, `playlists` + `playlist_tracks`, `analysis_results`; chaves estrangeiras com remoção em cascata |
| ADAPTAÇÃO | esquema próprio sem a máquina `rb_*` do catálogo de referência; registrada em `spec/rekordbox/10-master-db.md` |
| Migrações | `PRAGMA user_version`, uma transação por versão, rollback automático em falha, lacuna de versão rejeitada, continuação da versão parcial; backup `<nome>.backup-v<N>.db` antes de migrar arquivo preexistente |
| P2.3 bloqueado | exportação USB retornou à engenharia reversa (Gate G0): não há dossiê do fluxo de exportação nem artefato `export.pdb` no ambiente; desbloqueio exige sessão Ghidra ou mídia exportada pelo programa de referência |
| Testes | 19 testes novos (9 migrações, 7 integridade, 3 unitários): upgrade, rollback, interrupção, cascata, cache de análise, banco corrompido e inválido |
| Verificação | `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --workspace --all-features` (189 testes) e `xtask spec-lint` aprovados |
| Próximo gate | `vilelabox-library` (P3.2): importadores, busca, ordenação, playlists e detecção de arquivos movidos |

## Beatgrid, metrônomo e scratch na waveform — 2026-10-25

| Campo | Resultado |
| --- | --- |
| Especificação | `spec/rekordbox/20-bpm-grid.md`: contrato GRID/PHRASE EDIT do rekordbox 7.2.8 (manual pp. 134–136) |
| Editor de grade | `vilelabox-app/src/beatgrid.rs`: downbeat 1.1, deslocamento de 3 ms (1 ms fino), estreitar/alargar, ÷2/×2, TAP, modos inteiro/parcial e undo/redo com histórico de 64 estados |
| Persistência | `vilelabox-app/src/grid_store.rs`: sidecar JSON atômico por faixa, incluindo o bloqueio de edição |
| Interface | `ui/components/grid_edit.slint` operável por mouse; arrasto da waveform em `deck_wave.slint` com scratch/backspin bidirecional e `Shift` a 15% de sensibilidade |
| Motor de áudio | clique de metrônomo com três volumes e acento no 1 gerado em `vilelabox-audio/src/deck.rs`; leitura bidirecional com interpolação linear durante o gesto |
| Lacunas | âncora parcial e filtragem do TAP não recuperadas do binário (indexação Ghidra do executável de 573 MB excedeu a sessão); regras determinísticas documentadas como ADAPTAÇÃO |
| Testes | `metronome_tests.rs` e `scratch_tests.rs` no `vilelabox-audio`; 170 testes aprovados na workspace |
| Verificação | `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --workspace --all-features` e `xtask spec-lint` aprovados; lints do Rust 1.99.1 saneados no commit anterior |
| Próximo gate | exportação USB (P2.3) e banco próprio `vilelabox-storage` (P3.1) |

## Medição da UI em execução — 2026-10-24

| Campo | Resultado |
| --- | --- |
| Ferramentas | `tools/ui/medir_ui.ps1` (Win32 + UI Automation + captura) e `tools/ui/analisar_layout.py` (só biblioteca padrão) |
| Ambiente | 1920×1080, DPI 120 (1,25); cliente 1920×1051 px físicos = 1536×841 lógicos |
| Regiões | menus 28 px, barra global 36 px, reprodução 517 px, navegador 439 px, status 35 px |
| Painéis | deck 1 890 px (46,35%), faixa de mixer 134 px (6,98%), deck 2 890 px (46,35%) |
| Navegador | árvore 405 px, tabela 1461 px, linhas de 33–34 px, onze colunas com largura medida |
| Correção encontrada | `PrintWindow` devolve a área de reprodução em preto com a janela ocluída; a captura precisa de foco breve e devolve o foco à janela anterior |
| Dossiê | `spec/rekordbox/evidence/p3-1-ui-medido.md` e `layout-rekordbox-7.2.8.json` |
| Especificação | `spec/rekordbox/40-telas.md` rev. 2: resolução de referência e escala fixadas |
| Próximo gate | aplicar a geometria medida na UI Slint e medir exportação, quatro decks e painéis abertos |

## Reorganização Rust independente — 2026-10-24

| Campo | Resultado |
| --- | --- |
| Direção | VILELABOX passa a ser uma aplicação Rust independente |
| Legado | Preservado na branch `legacy-mixxx` |
| Workspace | Promovido de `rust/` para a raiz |
| Removido do `main` | C++/Qt, build CMake, empacotamento e adaptadores específicos do legado |
| Clean-room | `spec/rekordbox`, `xtask`, ferramentas de referência e barreira de terceiros preservados |
| Ativos de terceiros | Movidos para `third_party/rekordbox/ui-workbench`, ignorado pelo Git |
| Próximo gate | Concluir cues, conversão canônica e paridade do leitor ANLZ |

O histórico detalhado da implementação anterior permanece acessível na
branch de legado.

## ANLZ: cues e conversão canônica — 2026-10-24

| Campo | Resultado |
| --- | --- |
| Parser | `PCOB`/`PCPT`, memory cues, hot cues e loops com validação de limites |
| Modelo | `RawCue`, `canonical_cues`, `canonical_phrases` e `track_from_anlz` |
| Segurança | comprimento menor que o cabeçalho e buffers truncados encerram a leitura |
| Diferencial | 40 pares `.DAT`/`.EXT` copiados para a área ignorada e comparados com referências locais |
| Testes | 47 testes Rust, Clippy com `-D warnings`, formatação e `spec-lint` aprovados |
| Correção encontrada | memory cue com slot zero não avalia mais uma subtração que causava overflow |
| Próximo gate | exportador ANLZ em Rust e variantes com label/cor |

## Gate G0, P0.1, P1.1, P1.2 e G0/P2.1 — 2026-10-24

| Campo | Resultado |
| --- | --- |
| Artefato de referência | `rekordbox.exe` `7.2.8.0` (SHA-256 `85e3a58524c5cd1e870ad59e23cc8806523175f80fb82d6059c68d57bdf9991a`) |
| Dossiês G0 privados | `third_party/rekordbox/re/evidence/{40-telas,11-anlz,10-master-db}.md` |
| Dossiês G0 versionáveis | `spec/rekordbox/evidence/{p0-1-interface,p1-anlz,p2-1-master-db}.md` |
| Especificações atualizadas | `spec/rekordbox/40-telas.md`, `spec/rekordbox/11-anlz.md` e `spec/rekordbox/10-master-db.md` |
| P0.1 — Interface Slint | Modularizada em `vilelabox-app/ui/{theme,app}.slint`, `ui/components/{navigation,deck,mixer,library,status}.slint` e `src/presentation.rs` (`main.rs` reduzido a 49 linhas) |
| P1.1 — Exportador ANLZ | `vilelabox-anlz/src/export.rs` e `vilelabox-anlz-export` com confinamento a `--music-dir`, SHA-256 em streaming, escrita atômica e 100% de paridade nas 40 análises reais |
| P1.2 — Variantes de cues | Suporte a `PCO2`/`PCP2` (`parse_cues_extended`, `cue_sources`) com decodificação de `label` UTF-16BE, `color` RGB, múltiplos bancos e fallback seguro para `PCOB` |
| Verificação | `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --workspace --all-features` e `xtask spec-lint spec/rekordbox` aprovados |
| Próximo gate | Implementar `vilelabox-catalog` somente leitura (P2.1), `vilelabox-xml` (P2.2) e exportação USB (P2.3) |


## Fase U0 e U1/U3 — Funções do layout (P5.1 / PLANO-UI-FUNCOES) — 2026-10-24

| Campo | Resultado |
| --- | --- |
| Arquitetura | Princípio 1 estrito: UI só emite comandos e recebe snapshots. Nenhuma lógica de negócio ou SQL no Slint. |
| U0.1 — Estado fora do Slint | `AppState` em `vilelabox-app/src/state.rs` mantendo faixas, índices filtrados/ordenados, seleção, decks e relógio |
| U0.2 — Comandos | `enum Command` (`ScanStarted`, `ScanFinished`, `SelectTrack`, `LoadToDeck`, `LoadSelectedToDeck`, `Search`, `SortBy`, `TickClock`) e `apply(&mut AppState, Command)` puramente testável em Rust |
| U0.3 — Snapshot → tela | `presentation::render(&AppState, &AppWindow)` unifica a atualização da UI em um único ponto |
| U0.4 — Estado por deck | Propriedades `deck1_*` e `deck2_*` separadas em `app.slint`: Deck 1 carrega a 1ª faixa por conveniência e Deck 2 permanece "Not Loaded." até comando de carga, eliminando o bug de espelhamento |
| U0.5 — Thread de background | Varredura de análises executada em `std::thread::spawn` e despachada via `slint::invoke_from_event_loop`: a janela abre instantaneamente |
| U0.6 — Desabilitação visual | Propriedade `audio_ready: false` em decks e mixer estiliza controles sem backend (CUE, play, loop, hot cues, sync, pitch e faders) com baixa opacidade e cores apagadas (Princípio 2) |
| U1.1 — Busca por texto | Campo de busca com `TextInput` interativo e placeholder reativo, filtrando título e artista em tempo real |
| U1.2 — Ordenação de colunas | Cabeçalhos `#`, título, artista, BPM, tempo e compassos clicáveis com indicação visual de direção (`▲`/`▼`) |
| U1.6 — Seleção vs Carga | Clique na tabela apenas seleciona; carga nos decks é acionada via botões dedicados "LOAD" nos decks e "LOAD 1"/"LOAD 2" na barra |
| U1.7 — Estados vazios | Mensagem informativa ao filtrar sem resultados ou com lista vazia |
| U3.2 — Relógio do sistema | Relógio na `PerformanceToolbar` atualizado periodicamente com a hora real local via `chrono` |
| U3.3 — Limpeza de toolbar | Removido o badge estático "LINK ATIVO" até a fase de rede (C6) |
| Verificação | 77 testes aprovados na workspace, Clippy sem alertas com `-D warnings`, `cargo fmt -- --check` e `xtask spec-lint` aprovados |
