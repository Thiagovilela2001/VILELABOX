# Inventário Técnico — VILELABOX

Inventário estrutural e de acoplamento dos recursos adicionados ou modificados pelo mod **VILELABOX** (base upstream: commit `3ebac449e7`, Mixxx 2.5.6).

Cada recurso está documentado com os arquivos próprios adicionados (`A`), arquivos do Mixxx modificados (`M`), testes correspondentes, dependências de subsistemas, formato de persistência, nível de acoplamento e avaliação para migração para Rust.

---

## 1. Ponte XML (rekordbox.xml)

Ponte bidirecional de sincronização de biblioteca, hot cues, loops, beatgrid e playlists entre Mixxx e rekordbox.

| Campo | Conteúdo |
| --- | --- |
| **Arquivos próprios** | `tools/rekordbox/sync.py`, `tools/rekordbox/test_sync.py`, `tools/rekordbox/analyze_ml_retired.py` |
| **Arquivos do Mixxx alterados** | Nenhum. Opera externamente sobre o SQLite e arquivos XML. |
| **Testes** | `tools/rekordbox/test_sync.py` (4 testes unittest: rollback de transação, bloqueio de concorrência com Mixxx aberto, verificação de backup com WAL e parsing de XML). |
| **Dependências do Mixxx** | Nenhuma dependência em tempo de compilação ou de classes C++. Depende unicamente do esquema relacional de tabelas (`library`, `track_locations`, `cues`, `Playlists`, `PlaylistTracks`) de `mixxxdb.sqlite`. |
| **Dados persistidos** | `mixxxdb.sqlite`, `rekordbox.xml`, backups transacionais com WAL em `~/.mixxx/backups-rekordbox/`. |
| **Acoplamento** | **Baixo** (processo externo autônomo, desacoplado de motor de áudio, Qt e threads do Mixxx). |
| **Candidato a Rust** | **Sim** (candidato imediato nº 1, Etapa 3 do README / Etapa E do Plano 4). |

---

## 2. Faixa de BPM (TempoOctave)

Cálculo de oitavas de andamento (1/2x, 2x) e limiares de busca de BPM configuráveis.

| Campo | Conteúdo |
| --- | --- |
| **Arquivos próprios** | `src/track/tempooctave.h` (funções inline puras de cálculo de faixas de BPM e ajuste de oitavas). |
| **Arquivos do Mixxx alterados** | `src/analyzer/analyzerbeats.cpp:233-240` (faixa de BPM preferida, chama `vilelabox::normalizeTempoOctave`), `src/preferences/dialog/dlgprefbeats.cpp` / `dlgprefbeats.h` / `dlgprefbeatsdlg.ui` (opções de faixa de BPM). |
| **Testes** | `src/test/beatstest.cpp` (`TempoOctaveTest.*`). |
| **Dependências do Mixxx** | `mixxx::Bpm`, `mixxx::Beats`. |
| **Dados persistidos** | `mixxx.cfg` (chave `[BPM],BpmRange`), coluna `bpm` na tabela `library` de `mixxxdb.sqlite`. |
| **Acoplamento** | **Baixo** (funções puras de cálculo matemático e conversão de escala em `tempooctave.h`). |
| **Candidato a Rust** | **Sim** (biblioteca auxiliar pura e trivialmente portada para Rust). |

---

## 3. Energia da Faixa

Cálculo de métricas espectrais e de energia sonora (RMS, distribuição espectral) para ranqueamento de faixas relacionadas.

| Campo | Conteúdo |
| --- | --- |
| **Arquivos próprios** | `src/analyzer/analyzerenergy.cpp`, `src/analyzer/analyzerenergy.h`. |
| **Arquivos do Mixxx alterados** | `src/analyzer/analyzerthread.cpp:123` (registro do `AnalyzerEnergy`). `src/preferences/dialog/dlgprefvilelabox.cpp` é arquivo próprio (status `A`). |
| **Testes** | `src/test/beatstest.cpp` (testes de processamento de energia integrados). |
| **Dependências do Mixxx** | `AnalyzerTrack`, `SoundSource`. |
| **Dados persistidos** | Arquivos JSON em `~/.mixxx/analysis/energy/*.json`, chave `[Vilelabox],AnalyzeEnergy`. |
| **Acoplamento** | **Baixo** (processamento de sinal DSP sobre buffer de áudio em batch, gerando cache estático em JSON). |
| **Candidato a Rust** | **Sim** (candidato de alto desempenho na Etapa 4 de análise de áudio). |

---

## 4. Frases Musicais

Detecção algorítmica da estrutura de frases musicais (Intro, Up, Chorus, Down, Verse, Outro) em blocos de compassos e PhraseSync.

| Campo | Conteúdo |
| --- | --- |
| **Arquivos próprios** | `src/analyzer/analyzerphrases.cpp`, `src/analyzer/analyzerphrases.h`, `src/analyzer/phraseanalysisstore.cpp`, `src/analyzer/phraseanalysisstore.h`, `src/track/phrases.cpp`, `src/track/phrases.h`, `src/engine/controls/phrasecontrol.cpp`, `src/engine/controls/phrasecontrol.h`, `src/widget/wphrasestatus.cpp`, `src/widget/wphrasestatus.h`, `src/test/phrasestest.cpp`, `tools/phrases/export_csv.py`, `tools/phrases/score_csv.py`, `tools/phrases/test_score_csv.py`. |
| **Arquivos do Mixxx alterados** | `src/analyzer/analyzerthread.cpp:122` (registro do `AnalyzerPhrases`), `src/analyzer/analyzertrack.h:13` (`forcePhrases`), `src/engine/controls/bpmcontrol.cpp:283` e `:1066-1090` (PhraseSync), `src/mixer/basetrackplayer.cpp:680-683` (chama `trackLoadedOnGuiThread`), `src/engine/enginebuffer.cpp:1576-1578`, `src/widget/wtrackmenu.cpp:539-544` e `:725` (Reanalisar frases). |
| **Testes** | `src/test/phrasestest.cpp` (`PhrasesTest.*`, `AnalyzerPhrasesTest.*`), `tools/phrases/test_score_csv.py` (4 testes unitários). |
| **Dependências do Mixxx** | `AnalyzerTrack`, `mixxx::Beats`, `ControlObject`, `TrackPointer`, `UserSettings`. |
| **Dados persistidos** | `~/.mixxx/analysis/phrases/*.phrases.json`, chave `[Vilelabox],AnalyzePhrases`. |
| **Acoplamento** | **Médio** (algoritmo analítico puro que gera JSON desacoplado; a publicação de controle em tempo de execução usa `ControlObject`). |
| **Candidato a Rust** | **Sim** (Etapa 4 do README; motor analítico perfeito para Rust). |

---

## 5. Mix Point (Mix In / Mix Out / Mix Point Link)

Determinação de pontos de entrada (`mixInPoint`) e saída (`mixOutRange`) de mixagem baseados na análise de frases e cues.

| Campo | Conteúdo |
| --- | --- |
| **Arquivos próprios** | `src/track/mixpoints.cpp`, `src/track/mixpoints.h`, `src/engine/controls/mixpointcontrol.cpp`, `src/engine/controls/mixpointcontrol.h`. |
| **Arquivos do Mixxx alterados** | `src/engine/enginebuffer.cpp:235` (criação do `MixPointControl`) e `:1578` (estado montado na thread da GUI). A lógica fica em `src/engine/controls/mixpointcontrol.cpp` (arquivo próprio). |
| **Testes** | `src/test/phrasestest.cpp` (`MixPointsTest.*`). |
| **Dependências do Mixxx** | `ControlObject`, `TrackPointer`, `mixxx::audio::FramePos`. |
| **Dados persistidos** | Derivado dinamicamente de frases (`analysis/phrases/`) e cues do banco SQLite. |
| **Acoplamento** | **Médio** (cálculo posicional puro em `mixpoints.cpp`; sincronismo e disparo via `MixPointControl` ligado a COs). |
| **Candidato a Rust** | **Sim** (fácil de implementar como crate junto ao motor de frases). |

---

## 6. Cues Automáticos (Auto Phrase Cues)

Criação automática não-destrutiva de Hot Cues nos pontos estruturais (MIX IN, DROP, BREAK, DROP 2, MIX OUT) ao carregar a faixa.

| Campo | Conteúdo |
| --- | --- |
| **Arquivos próprios** | `src/track/autophrasecues.cpp`, `src/track/autophrasecues.h`. |
| **Arquivos do Mixxx alterados** | Nenhum direto: `src/engine/controls/phrasecontrol.cpp:64` e `:85` (arquivo próprio) chamam `AutoPhraseCues::apply`; o gancho no Mixxx é o mesmo das frases (`src/engine/enginebuffer.cpp:1577`). |
| **Testes** | `src/test/phrasestest.cpp` (`AnalyzerPhrasesTest.AutomaticCuesPreserveUserSlotsLockAndUndoEdits`). |
| **Dependências do Mixxx** | `TrackPointer`, `CuePointer`, `mixxx::Beats`, `vilelabox::MixPoints`. |
| **Dados persistidos** | Registros de hot cue na tabela `cues` do `mixxxdb.sqlite`. |
| **Acoplamento** | **Médio** (lógica de decisão musical de slots pura; depende dos objetos `Track` e `Cue` do Mixxx para gravação). |
| **Candidato a Rust** | **Sim** (lógica de seleção e preservação de slots transferível para o backend de preparação). |

---

## 7. Faixas Relacionadas (Related Tracks)

Recomendação assíncrona de faixas compatíveis por harmonia musical (chave Camelot/OpenKey), andamento (BPM/oitavas) e nível de energia.

| Campo | Conteúdo |
| --- | --- |
| **Arquivos próprios** | `src/library/related/relatedtracks.cpp`, `src/library/related/relatedtracks.h`, `src/library/related/relatedtracksfeature.cpp`, `src/library/related/relatedtracksfeature.h`. |
| **Arquivos do Mixxx alterados** | `src/library/library.cpp:10` e `:156` (inclusão e `addFeature(new RelatedTracksFeature(...))`). |
| **Testes** | `src/test/beatstest.cpp` (`RelatedTracksTest.*`). |
| **Dependências do Mixxx** | `LibraryFeature`, `TrackModel`, `TrackPointer`, `KeyUtils`, `QFuture` / `QtConcurrent`. |
| **Dados persistidos** | Consulta direta à tabela `library` de `mixxxdb.sqlite` e leitura de `analysis/energy/*.json`. |
| **Acoplamento** | **Médio** (algoritmo de pontuação e compatibilidade puro; amarração de dados feita via `LibraryFeature` do Qt). |
| **Candidato a Rust** | **Sim** (motor de busca e ranking de compatibilidade em Rust alimentado por SQLite e cache de energia). |

---

## 8. Detector de Vocal

Análise de trechos cantados via modelo Demucs (`htdemucs`) em processo Python local, com classificador neural de reserva.

| Campo | Conteúdo |
| --- | --- |
| **Arquivos próprios** | `src/analyzer/analyzervocals.cpp`, `src/analyzer/analyzervocals.h`, `src/analyzer/vocalanalysisstore.cpp`, `src/analyzer/vocalanalysisstore.h`, `src/widget/wvocalstatus.cpp`, `src/widget/wvocalstatus.h`, `res/vocals/vocal_detector.py`, `res/vocals/model.json`, `tools/vocals/*` (scripts de validação, manifesto e dataset sintético). |
| **Arquivos do Mixxx alterados** | `src/analyzer/analyzerthread.cpp:121` (registro do `AnalyzerVocals`), `src/skin/legacy/legacyskinparser.cpp:626-629` (widget `VocalStatus`), `src/widget/woverview.cpp:185-198` (redesenho quando a análise de vocal muda). |
| **Testes** | `src/test/beatstest.cpp` (`VocalAnalysisTest.*`, `AnalyzerVocalsTest.*`), `tools/vocals/test_detector.py`. |
| **Dependências do Mixxx** | `AnalyzerTrack`, `QProcess`, `UserSettings`, `WWidget`. |
| **Dados persistidos** | `~/.mixxx/analysis/vocals/*.json`, chaves `[Vilelabox],VocalPython` e `[Vilelabox],VocalTimeoutSeconds`. |
| **Acoplamento** | **Médio** (gerenciador de processo e leitor de cache em JSON; comunicação por pipes/arquivos). |
| **Candidato a Rust** | **Depois** (orquestração de subprocesso Python e parsing de JSON podem ir para Rust quando toda a pipeline de análise for portada). |

---

## 9. Grid e Downbeat

Alinhamento flexível de beatgrid, detecção do primeiro tempo do compasso (downbeat) e correção de grade.

| Campo | Conteúdo |
| --- | --- |
| **Arquivos próprios** | `src/test/analyzergrid_test.cpp`. |
| **Arquivos do Mixxx alterados** | `src/proto/beats.proto:26` e `:32` (`first_downbeat_index` em BeatMap e BeatGrid), `src/track/beats.cpp:298-380` e `:656-776` (leitura/escrita do índice de compasso), `src/analyzer/analyzerbeats.cpp:262-271` (compasso 1 da análise), `src/analyzer/plugins/analyzerqueenmarybeats.cpp:94-111` e `:166-221` (modo de precisão e `refineBeats`). |
| **Testes** | `src/test/analyzergrid_test.cpp` (`AutoDownbeatTest.*`, `AnalyzerGridTest.*`), `src/test/beatstest.cpp` (`BeatGridTest.*`, `BeatMapTest.*`, `BeatsTranslateTest.*`). |
| **Dependências do Mixxx** | `mixxx::Beats`, `mixxx::BeatGrid`, `mixxx::BeatMap`, `AnalyzerTrack`, `SoundSource`. |
| **Dados persistidos** | Protobuf de batidas (`src/proto/beats.proto`) armazenado em `mixxxdb.sqlite` e metadados de track. |
| **Acoplamento** | **Médio** (matemática de grade e alinhamento de batidas desacoplável, mas serialização amarrada ao Protobuf do Mixxx). |
| **Candidato a Rust** | **Sim** (Etapa 4 do README; representação de grid e downbeat é essencial para o motor de análise em Rust). |

---

## 10. Tonalidade (Key Display & Camelot)

Exibição visual estendida de escala musical (notação alfanumérica Camelot/OpenKey) e coloração por compatibilidade harmônica.

| Campo | Conteúdo |
| --- | --- |
| **Arquivos próprios** | Nenhum (aproveitamento da estrutura interna de tonalidades do Mixxx). |
| **Arquivos do Mixxx alterados** | `src/widget/wkey.cpp:34` (`EditableKey` na skin), `:65-67` e `:81-140` (menu de tom: travar, semitons, restaurar, notação), `src/preferences/dialog/dlgprefbeats.cpp`. |
| **Testes** | `src/test/beatstest.cpp` (`KeyUtilsTest.*`). |
| **Dependências do Mixxx** | `mixxx::Key`, `WLabel`, `ControlObject`. |
| **Dados persistidos** | Coluna `key` na tabela `library` de `mixxxdb.sqlite`. |
| **Acoplamento** | **Médio** (formatação e mapeamento de tons puros, porém acoplados a widgets Qt `WKey`). |
| **Candidato a Rust** | **Depois** (estruturas de dados de notação musical podem ir para Rust com a biblioteca musical). |

---

## 11. Configurações (Painel VILELABOX)

Aba dedicada nas preferências do Mixxx para controle de todos os módulos experimentais do mod.

| Campo | Conteúdo |
| --- | --- |
| **Arquivos próprios** | `src/preferences/dialog/dlgprefvilelabox.cpp`, `src/preferences/dialog/dlgprefvilelabox.h`, `src/test/preferencesvilelabox_test.cpp`. |
| **Arquivos do Mixxx alterados** | `src/preferences/dialog/dlgpreferences.cpp:270-273` (página VILELABOX) e `:301-505` (`setupSettingsInterface`), `src/mixxxmainwindow.cpp:306-314` (controle `[App],show_preferences`). |
| **Testes** | `src/test/preferencesvilelabox_test.cpp` (`VilelaboxPreferencesTest.*`). |
| **Dependências do Mixxx** | `DlgPreferences`, `UserSettings`, `QDialog`, `QWidget`. |
| **Dados persistidos** | Arquivo de configuração `mixxx.cfg` na seção `[Vilelabox]`. |
| **Acoplamento** | **Médio** (fortemente acoplado aos diálogos Qt e à classe `UserSettings` do Mixxx). |
| **Candidato a Rust** | **Não** (será substituído inteiramente pela nova UI desktop do VILELABOX). |

---

## 12. Skins (RbxOriginal e RbxFlowBars)

Skins completas inspiradas no fluxo do rekordbox, com contadores de compasso, botões de grid, pads e layouts dedicados.

| Campo | Conteúdo |
| --- | --- |
| **Arquivos próprios** | Arquivos XML/SVG em `res/skins/RbxOriginal/`, scripts em `res/skins/RbxOriginal/tools/*`. |
| **Arquivos do Mixxx alterados** | `src/skin/legacy/legacyskinparser.cpp:622-629` (nós `PhraseStatus` e `VocalStatus`). |
| **Testes** | `res/skins/*/tools/check_refs.py`, `res/skins/*/tools/check_sizes.py`. |
| **Dependências do Mixxx** | `LegacySkinParser`, parser XML de skins do Mixxx. |
| **Dados persistidos** | `mixxx.cfg` (`[Config],skin`). |
| **Acoplamento** | **Médio** (específico do parser legado de templates XML do Mixxx). |
| **Candidato a Rust** | **Não** (a interface do VILELABOX em Rust na Etapa 5 usará framework nativo de UI). |

---

## 13. Compassos (BarSync e Contadores)

Sincronização por compassos musicais inteiros (BarSync), contador de compassos no deck e marcadores visuais.

| Campo | Conteúdo |
| --- | --- |
| **Arquivos próprios** | `src/widget/phrasecolors.h`. |
| **Arquivos do Mixxx alterados** | `src/engine/controls/bpmcontrol.cpp:1142-1169` (`getBarMatchOffset`) e `:1384-1390` (`updateBarPosition`), `src/engine/enginebuffer.cpp:800` e `:1335` (salto alinhado ao compasso), `src/waveform/renderers/waveformrenderbeat.cpp`, `src/waveform/renderers/allshader/waveformrenderbeat.cpp`. |
| **Testes** | `src/test/beatstest.cpp` (`BarSyncTest.*`, `BarJumpLoopTest.*`). |
| **Dependências do Mixxx** | `BpmControl`, `EngineBuffer`, `ControlObject`, `mixxx::Beats`. |
| **Dados persistidos** | `beats.proto` (definição do beat 1), estado transiente de `ControlObject`. |
| **Acoplamento** | **Alto** (interfere diretamente no loop de áudio em tempo real e na máquina de estados de sincronização do engine). |
| **Candidato a Rust** | **Depois** (migrará quando o motor de áudio for portado na Etapa 6). |

---

## 14. Pads de Performance

Modos estendidos de acionamento de performance: Slicer, Release FX (Break/Echo/Spin), Hot Cue e Beat Jump.

| Campo | Conteúdo |
| --- | --- |
| **Arquivos próprios** | `src/engine/controls/padcontrol.cpp`, `src/engine/controls/padcontrol.h`. |
| **Arquivos do Mixxx alterados** | `src/engine/enginebuffer.cpp:237` (criação do `PadControl`) e `:1151` (`processAudio`). `src/engine/controls/padcontrol.cpp` é arquivo próprio; o mod não altera `basetrackplayer.cpp` para pads. Skin: `res/skins/RbxOriginal/decks/vilelabox_pads.xml`. |
| **Testes** | `src/test/beatstest.cpp` (`BarJumpLoopTest.*`). |
| **Dependências do Mixxx** | `ControlObject`, `EngineBuffer`, `LoopingControl`. |
| **Dados persistidos** | Nenhum (apenas `ControlObject` transientes de controle). |
| **Acoplamento** | **Alto** (disparo de loops curtos, saltos e pausas em tempo real no buffer de áudio). |
| **Candidato a Rust** | **Depois** (pertence à Etapa 6 de motor de áudio e performance). |

---

## 15. Waveform RGB e Renderizadores

Renderização de waveforms em alta velocidade via OpenGL, com marcação de blocos vocais e cores de frases.

| Campo | Conteúdo |
| --- | --- |
| **Arquivos próprios** | `src/widget/phrasecolors.h`. |
| **Arquivos do Mixxx alterados** | `src/waveform/renderers/allshader/waveformrendererrgb.cpp:28-29`, `:90-107` e `:213-237` (modo RGB e degradê), `src/waveform/renderers/waveformrenderbeat.cpp`, `src/widget/woverview.cpp:169-183`, `:717-751` e `:769-800` (marcadores de compasso e frase). O renderizador RGB não-allshader não foi alterado. |
| **Testes** | Validação visual das skins e compilação de shaders. |
| **Dependências do Mixxx** | `WaveformRenderer`, `QOpenGLFunctions`, `VisualPlayPosition`, `TrackPointer`. |
| **Dados persistidos** | Cache de renderização de waveform em disco do Mixxx. |
| **Acoplamento** | **Alto** (diretamente acoplado ao pipeline gráfico Qt OpenGL e à sincronização com o cursor de áudio). |
| **Candidato a Rust** | **Depois** (será refeito na Etapa 5 com shaders/wgpu modernos). |

---

## Ordem de extração sugerida (do menor para o maior acoplamento)

Com base nas evidências levantadas acima, a extração de código e recursos para crates Rust deve seguir a seguinte ordem de prioridade:

```mermaid
graph TD
    subgraph Fase 1: Isolamento Total (Acoplamento Baixo)
        P1[1. Ponte rekordbox.xml / sync.py]
        P2[2. TempoOctave / Faixas de BPM]
        P3[3. DSP de Energia / RMS]
    end
    subgraph Fase 2: Análise e Metadados (Acoplamento Médio)
        P4[4. Detecção de Frases Musicais]
        P5[5. Mix Points e Auto Phrase Cues]
        P6[6. Busca e Ranking de Faixas Relacionadas]
        P7[7. Algoritmos de Grid e Downbeat]
        P8[8. Orquestração do Detector de Vocal]
    end
    subgraph Fase 3: Interface e Motor (Acoplamento Alto)
        P9[9. Interface Desktop & Waveform Shaders]
        P10[10. BarSync, Pads e Motor de Áudio em Tempo Real]
    end

    P1 --> P4
    P2 --> P7
    P3 --> P6
    P4 --> P5
    P5 --> P6
    P6 --> P9
    P7 --> P10
    P8 --> P9
```

### Justificativas por evidência:

1. **Ponte XML (`tools/rekordbox/sync.py`):**
   * *Evidência:* zero chamadas a APIs C++ do Mixxx; apenas transações SQLite e leitura/geração de XML.
   * *Risco:* zero para o motor de áudio do mod.
2. **TempoOctave (`src/track/tempooctave.h`):**
   * *Evidência:* funções matemáticas puras sem efeitos colaterais.
3. **Energia (`src/analyzer/analyzerenergy.cpp`):**
   * *Evidência:* opera sobre array contíguo de amostras de áudio PCM mono e gera JSON estático.
4. **Frases (`src/track/phrases.cpp`, `src/analyzer/analyzerphrases.cpp`):**
   * *Evidência:* opera com base em lista de batidas e envelopes de energia; a persistência é um arquivo JSON independente (`analysis/phrases/`).
5. **Mix Points e Cues Automáticos (`src/track/mixpoints.cpp`, `src/track/autophrasecues.cpp`):**
   * *Evidência:* depende diretamente da estrutura de frases da etapa anterior para calcular intervalos de transição.
6. **Faixas Relacionadas (`src/library/related/relatedtracks.cpp`):**
   * *Evidência:* combina os metadados de tom, BPM e scores de energia via consultas estáticas.
7. **Grid e Downbeat (`src/track/beats.cpp`, `src/analyzer/plugins/analyzerqueenmarybeats.cpp`):**
   * *Evidência:* exige desacoplar os tipos de dados do Protobuf do Mixxx (`src/proto/beats.proto`).
8. **Vocal (`src/analyzer/analyzervocals.cpp`):**
   * *Evidência:* gerencia subprocessos externos (`QProcess`) e depende de ambiente Python local com PyTorch/Demucs.
9. **Interface e Waveforms (`src/waveform/*`, `src/widget/*`):**
   * *Evidência:* amarrada à arquitetura de widgets Qt6 e shaders legados do Mixxx.
10. **Motor de áudio, BarSync e Pads (`src/engine/*`, `EngineBuffer`):**
    * *Evidência:* código crítico de tempo real sob threads de áudio de baixa latência e travas estritas (`prctl`, lock-free rings). Deve ser a última camada a ser portada para garantir estabilidade absoluta.
