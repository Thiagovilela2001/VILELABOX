# Registro de implementação

## Marcas por 16 avos da passada C e reancoragem no 7.2.19 (P4.1) — 2026-11-14

- Reancoragem: o 7.2.8 saiu da máquina, e o projeto Ghidra agora é o 7.2.19. Dos 146 endereços de código do dossiê, 138 foram mapeados por strings de log, contagem por bloco e callgraph (`spec/rekordbox/evidence/p4-1-enderecos-7.2.19.tsv`). Ficaram sem âncora `0x1410bab80` e as rotinas de libm.
- Porte: `0x14118d310` (7.2.19: `0x1412ec9d0`) virou `vilelabox-dsp::passada_c::MarcasPorDezesseisAvos`, conferido no assembly. A busca de alinhamento é `deslocamento_de_alinhamento`, e os argumentos que o Ghidra perdia foram lidos no assembly. O banco de filtros reaproveita `FiltrosBanda`.
- Ferramentas: `tools/re/RefsTo.java` (quem referencia um endereço) e `tools/re/DisasmList.java` (assembly com constantes).
- Validação: 53 testes do `vilelabox-dsp`, `fmt --check` e Clippy com `-D warnings`. Não há comparação com o programa de referência.
- Pendente: a gêmea `0x14118e450` (`0x1412edb10`), `0x141197890` e o fechamento `0x141197d60`.

## Aproximação visual ao rekordbox 7.2.8 — 2026-11-10

- Referência: captura local `third_party/rekordbox/ui-workbench/medicoes/captura.png`, modo 2Deck Horizontal, e medidas de `spec/rekordbox/evidence/p3-1-ui-medido.md`.
- Decks: fundo cinza neutro, capa vazia quadrada com disco desenhado, título `Not Loaded.`, anel interno do prato centralizado e azul, slots de cue escuros.
- Mixer e faixa inferior: fundos cinza; TRIM com anel neutro. Biblioteca e árvore: texto com peso maior, conforme a captura.
- Waveform vazia deixa de exibir agulha. Callbacks e comportamento dos controles preservados.
- Conferência visual pelo app em execução; esta entrega cobre o estado vazio. Capas reais, prévias da tabela e comparação de todos os estados continuam pendentes.

## Preservar edição durante leitura assíncrona (P5.1) — 2026-11-10

- Correção em `state.rs`: cues e grade têm marcadores independentes de edição por deck; uma resposta de preparação não substitui o domínio editado desde a carga.
- Uma nova carga limpa os marcadores para permitir restauração do banco. Transporte e layout permanecem como antes.
- Regressão: resposta atrasada depois de criar cue e deslocar grade preserva ambos e o bloqueio local; recarregar aceita novamente a preparação persistida.
- Validação: 68 testes do app aprovados, formatação e Clippy com `-D warnings`. Sem validação visual nesta entrega.
- Pendente: tabela, busca e playlists servidas pelo banco; importação pela interface; análise própria e validação diferencial.

## Passadas B e C do detector legado de BPM (P4.1) — 2026-11-05

| Campo | Resultado |
| --- | --- |
| Achado principal | a passada B **não** estima o BPM: `0x141173370` monta só as grades das três bandas a partir do BPM já conhecido (`obj+0xb0`). As 15 funções seguintes de `0x141195ed0` só inicializam filtros, buffers, uma FFT e um núcleo de tonalidade. A passada C (`0x1411965b0`) extrai características por batida; o BPM deve ser decidido no fechamento `0x141197d60` (inferência, ainda não aberto) |
| Correção no dossiê | `+0x208` é o deslocamento final da passada A em ms, não BPM × 100 (o porte já estava certo) |
| Passada B | `grades_passada_b` (`0x141173370`); mapa da inicialização (tabela no dossiê) |
| Passada C portada | 15 extratores ao todo; 13 portados e conferidos no assembly; `0x1411885f0` é a mesma `energia_por_quarto` da passada A |
| Novos módulos | `vilelabox-dsp::tonalidade` (transformada de Q constante, `0x14117ff60`/`0x1411803f0`), `espectro` (FFT radix-2 própria e espectro em dB por compasso/batida, `0x14117b1b0`/`0x14117ac80`), `passada_c` (`0x141188050`, `0x14117e850`) |
| Em `estagios` | `EnvelopePorBatida`, `EnergiaPorQuarto40`, `pico_por_batida`, `Bandas500PorBatida`, `AgudosPorBatida`, `PicoMediosPorBatida`, `energia_e_pico_por_quarto`, `PicoAgudosPorOitavo`; núcleos comuns `por_segmento` e `janelas_de_quarto` |
| Limitações registradas | `pow` e `cos` vetoriais próprios (`0x1426086a0`, `0x142608cd0`) trocados por `f64::powf`/`f64::cos` (podem diferir no último bit); `0x1427a51e0` tratado como seno por inferência (função ímpar, par do `cos`) |
| Pendente | `0x14118d310` e `0x14118e450` (marcas por 16 avos, ~620 linhas cada, só mapeados: argumentos perdidos pelo Ghidra exigem leitura do assembly); depois `0x141197890` e o fechamento `0x141197d60` |
| Verificação | `cargo fmt`, `cargo clippy --all-targets -- -D warnings` e `cargo test` em `vilelabox-dsp` (49 testes); `xtask spec-lint` aprovado |

## Duração e ondas de WAVs com tamanho 0xFFFFFFFF — 2026-11-05

| Campo | Resultado |
| --- | --- |
| Sintoma | ao rodar o app sobre uma cópia de 40 análises, 8 faixas `.wav` apareciam com 372:49 na lista e no deck, e a visão geral da onda espremida no início |
| Causa | o cabeçalho desses WAVs declara o bloco `data` com `0xFFFFFFFF`; o rekordbox 7.2.8 aceita o valor e grava grade e ondas para ~22 370 s (2³² bytes a 48 kHz estéreo 16 bits). Exemplo: `ATW - 100%.wav` tem 363,5 s reais |
| Correção | `vilelabox-audio::decode::duracao_wav` lê o cabeçalho limitando o bloco ao tamanho real do arquivo; o scan localiza o áudio pelo nome (o PPTH vem como `?/nome.wav`) e guarda `Track::audio_ms`; `dur_ms` o prefere à grade. No deck, o detalhe é cortado na duração real e a visão geral é refeita a partir dele (`waveform::visao_do_detalhe`). Grade e ondas do rekordbox não são alteradas |
| Validação real | app aberto no modo de captura sobre cópia dos dados: lista com 06:03 e 02:23, deck com −06:03.4 e visão geral ocupando a largura toda |
| Verificação | `cargo fmt`, `cargo clippy --all-targets -- -D warnings` e `cargo test` em `vilelabox-app` e `vilelabox-audio` |
| Fora do escopo | capa, álbum, gênero e tonalidade vazios quando só as análises são abertas: vêm do XML ou do catálogo |

## Banco fora da thread gráfica: serviço do núcleo (P5.1, segunda entrega) — 2026-10-27

| Campo | Resultado |
| --- | --- |
| Desenho | `nucleo::iniciar` cria a thread dona da conexão; `Pedido` (carregar, registrar áudio, gravar cues, gravar grade, sincronizar coleção) entra por canal e `Resposta` volta pelo event loop como `Command::RespostaNucleo` |
| Estado da tela | `AppState::nucleo` virou `Sender<Pedido>`; nenhuma consulta SQLite em callbacks; respostas de faixa que já saiu do deck são descartadas; JSON só quando o banco recusa |
| Coleção | sincronização pedida no `ScanFinished`, pelo mesmo serviço (sem segunda conexão) |
| Limite conhecido | a preparação do banco chega milissegundos depois da carga e substitui cues e grade; edição feita nesse intervalo se perde (anotado no código) |
| Testes | serviço atende em ordem e responde (thread real); estado da tela com pedidos processados como no app (JSON recusado, migração, restauração, resposta descartada); 66 testes do app |
| Validação real | app aberto pela automação de interface: faixa carregada, tocada, pausada em 3,1 s e MEMORY; cue gravado no banco (memory, 3,17 s, origem user) e status com a sincronização da coleção |
| Verificação | `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --workspace --all-features` e `xtask spec-lint` aprovados |
| Próximo gate | snapshots imutáveis da tela; P5.2 com a tabela lida do banco |

## Coleção do app sincronizada no banco próprio (P5.2, primeira entrega) — 2026-10-27

| Campo | Resultado |
| --- | --- |
| Núcleo | `Comando::SincronizarColecao` com `FaixaExterna`: registra faixas que faltam pelo caminho e sincroniza listas pela raiz e pelo nome; storage ganha `limpar_lista` |
| App | depois da varredura, `Ponte::sincronizar_colecao` roda na thread da varredura, com conexão própria; o resumo vai para a barra de status |
| Medição real | 1 574 faixas e 1 574 localizações na coleção do usuário; segunda execução sem duplicatas |
| Correção | o marcador de exibição "Vilelabox / Coleção" deixou de ser gravado como artista (`state::SEM_ARTISTA`); 328 linhas já gravadas foram limpas no banco, com cópia em `vilelabox.db.antes-limpeza-artista` |
| Testes | 1 do núcleo (sincronização idempotente com reordenação), 1 da ponte (coleção, listas e artista ausente); workspace verde |
| Verificação | `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --workspace --all-features` e `xtask spec-lint` aprovados; app rodado duas vezes contra a coleção real |
| Próximo gate | tabela, busca e listas da tela lidas do banco; seletor de pasta |

## App ligado ao núcleo e ao banco próprio (P5.1, primeira entrega) — 2026-10-26

| Campo | Resultado |
| --- | --- |
| App | `vilelabox-app/src/nucleo.rs`: ponte com o `vilelabox-core` sobre `%APPDATA%\vilelabox\vilelabox.db`; conversões segundos/ms ↔ frames para cues e grade |
| Fluxo | carregar no deck registra a faixa e lê do banco cues do usuário, grade e bloqueio (com prioridade sobre o JSON); decodificar o áudio grava taxa, canais e duração e migra os JSON; gravar cues e grade vai para o banco quando a taxa é conhecida, senão para o JSON |
| Núcleo | comandos `RegistrarFaixa`, `RegistrarAudio`, `SubstituirCues`; consultas `faixa_por_caminho` e `taxa`; evento `IdentidadeAtualizada` |
| ADAPTAÇÃO | cues do banco voltam em ordem de posição; rótulo e cor agora são gravados (o JSON não guardava) |
| Testes | 1 do núcleo (registro idempotente, identidade, substituição sem loops), 3 da ponte (ida e volta de cue e grade; recusa sem taxa), 1 do app (JSON migra para o banco e restaura sem ele) |
| Verificação | `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --workspace --all-features` e `xtask spec-lint` aprovados; o app compila; a janela não foi aberta nesta rodada |
| Próximo gate | P5.2: coleção e playlists servidas pelo banco (importação pelo núcleo), busca pelo `Core::pesquisar` e seletor de pasta |

## Passada A do detector legado de ponta a ponta (P4.1) — 2026-10-26

| Campo | Resultado |
| --- | --- |
| Entrega | `grade_de_indices` (`0x141173280`) e `fechamento_passada_a` (`0x141195480`), que encadeia todos os estágios portados na ordem do original e devolve deslocamento, número da batida, molde A, picos confirmados e início audível |
| Achado | a passada A não estima BPM: recebe o BPM conhecido da faixa (`motor+0xb0`). A estimativa está nas passadas B e C/D |
| Testes | 2 unitários novos (grade conferida à mão; passada A sobre cliques sintéticos a 120 BPM, como regressão do porte, sem comparação com o original); workspace com 240 testes |
| Verificação | `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --workspace --all-features` e `xtask spec-lint` aprovados |
| Próximo gate | passada B (`0x141195ed0` → `0x141173370`) e C/D (`0x1411965b0`, `0x141197d60`), onde está o BPM; depois teste diferencial com faixa analisada pelo programa de referência |

## Marcação de picos do detector legado (P4.1, `0x14118b160`) — 2026-10-26

| Campo | Resultado |
| --- | --- |
| Método | assembly da função inteira; a decompilação escondia três FMAs, o reaproveitamento de registradores no alinhamento e a ordem do argmax |
| Recuperado | alinhamento inicial por máximo, encosta e vale; diferenciação; recorte ou deslocamento do sinal; grade de 16 máximos por grupo e marcação em até 6 rodadas |
| Crate | `vilelabox-dsp::estagios::marcar_picos`, `EstadoMarcas` (`motor+0x268`/`0x270`/`0x278`) |
| Lacuna fechada | os estágios já portados que consomem marcas (`classificar_padrao`, `confirmar_picos`, `picos_finos`, `fase_do_compasso`) agora têm o produtor |
| Banda 110–200 Hz | `filtro::FiltrosBanda` (`0x14118be60`) e `marcar_picos_banda` (`0x14118c1b0`), com 12 rodadas, alinhamento só por subida e `anterior` sempre atualizado |
| Padrão entre bandas | `padrao_duas_bandas` (`0x141177b30`) |
| Início e número | `primeira_batida_audivel` (`0x141171760`) e `numero_da_batida` (`0x1411718e0`), com os FMAs do assembly |
| Testes | 5 unitários novos (molde 1000; 6 contra 12 rodadas e `anterior` da banda; padrão encadeado; início audível; número da batida); workspace com 238 testes |
| Verificação | `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --workspace --all-features` e `xtask spec-lint` aprovados |
| Próximo gate | grade de índices `0x141173280` e o orquestrador `0x141195480`, para rodar a passada A de ponta a ponta |

## Fase do compasso do detector legado (P4.1, `0x141172490`) — 2026-10-26

| Campo | Resultado |
| --- | --- |
| Método | fluxo de controle conferido no assembly inteiro da função (`0x141172490`–`0x141173272`), registrador por registrador nos trechos que a decompilação embaralhou |
| Recuperado | varredura de moldes de bytes (A, B, C e o padrão D), escolha da lista de referência e limiar, correção pelo resto 4, período de um quarto de batida, energias com passa-baixas de 400 e 100 Hz, primeiro quarto forte, energia sustentada em blocos de 16 e de 64 quartos, fração de picos fora do tempo forte e as três decisões de deslocamento |
| Crate | `vilelabox-dsp::estagios::fase_do_compasso` e `Fase` |
| Testes | 2 unitários novos (retorno antecipado, molde A, constantes `f32` conferidas bit a bit); workspace com 233 testes |
| Verificação | `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --workspace --all-features` e `xtask spec-lint` aprovados |
| Próximo gate | segunda volta da cadeia: `0x14118be60`, `0x14118c1b0`, `0x141177b30`, `0x141171760`, `0x1411718e0` |

## Estágios da passada A: energia por quarto e onset de agudos (P4.1, A1.1c) — 2026-10-26

| Campo | Resultado |
| --- | --- |
| Método | decompilação Ghidra conferida no assembly com capstone (`desmontar.py`, privado), para argumentos perdidos e ordem das somas |
| Recuperado | energia (soma dos quadrados com FMA e redução exata); filtro de 1ª ordem; energia por quarto de batida e por compasso (`0x1411885f0`); onset de agudos com passa-alta de 300 Hz, valor absoluto e passa-baixa de 5 Hz (`0x141178330`) |
| Crate | `vilelabox-dsp::estagios`: `energia`, `soma`, `PrimeiraOrdem`, `energia_por_quarto`, `onset_agudos`, `classificar_padrao` (moldes rítmicos de `0x141176680`), `picos_de_agudos` (`0x1411786a0`), `confirmar_picos` (`0x141179530`), `picos_finos` (`0x141178df0`), `energia_por_quarto_fixa` (`0x141189170`) e `filtro::passa_baixa_hz` (`0x1411890e0`); esqueleto de `0x141172490` documentado |
| Testes | 10 unitários novos; workspace com 231 testes |
| Verificação | `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test --workspace --all-features` e `xtask spec-lint` aprovados |
| Próximo gate | portar `0x141172490` conferindo o fluxo no assembly; depois a segunda volta da cadeia (`0x14118be60`, `0x14118c1b0`, `0x141177b30`, `0x141171760`, `0x1411718e0`) |

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
