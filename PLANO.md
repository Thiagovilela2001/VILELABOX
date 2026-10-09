> Documento gerado por IA como assistência ao usuário.

# Plano de implementação do blackmesa / VILELABOX

Base: Mixxx 2.5.6 com o mod atual (COMPASSOS, GRID, TONALIDADE, VOCALS).
Referência de recursos: `~/Downloads/rekordbox_extracted` (mapeamentos MIDI,
modos de pad, ícones da skin). Como nos módulos anteriores, o rekordbox serve só
para dizer **quais** recursos existem. Os algoritmos são próprios ou abertos, e
os modelos proprietários (`models/`, `vd_param.pb`, `Demucs3.dll` etc.) não são
executados nem redistribuídos.

Ordem das fases: 0 → 1 → 2 → 3 são dependentes. As fases 4, 5 e 6 são
independentes entre si e podem ser feitas em qualquer ordem depois da fase 1.

| Fase | Entrega | Depende de | Tamanho |
| --- | --- | --- | --- |
| 0 | Branch, commits, política do ML, validação pendente | — | P |
| 1 | Análise de frases + PhraseSync | 0 | G |
| 2 | Cues automáticos por frase | 1 | M |
| 3 | Mix Point Link | 1 (2 ajuda) | G |
| 4 | Faixas relacionadas | 0 | M |
| 5 | Modos de pad (Slicer, KeyShift, Release FX) | 0 | M |
| 6 | Stems / Part Isolator | decisão 2.5.6 × 2.6 | GG |

---

## Fase 0 — Organização (antes de qualquer recurso)

1. **Controle de versão.** O repositório está em HEAD solto, com 35 arquivos
   alterados e cerca de 25 arquivos novos fora do git.
   - `git switch -c blackmesa`
   - Um commit por módulo: compassos, grid, tonalidade, vocal, empacotamento
     (`packaging/`, `run-*.sh`, ícone). Os scripts `backups-*/`, `bar-tests/` e
     os links simbólicos para `~/mixxx-rekordbox` ficam fora (`.git/info/exclude`).
   - As skins em `~/.mixxx/skins/RbxOriginal` e `RbxFlowBars` ficam fora do
     repositório. Copiar para `res/skins/` ou criar um repositório próprio para
     que as mudanças de skin das próximas fases também tenham histórico.
2. **`analyze-ml.py`**: executa os ONNX do rekordbox, o que contradiz a regra
   seguida em GRID.md e VOCALS.md. A previsão de hot cues dele nem usa o
   `samplecnn.onnx`; usa librosa e regras fixas. Proposta: aposentar o script.
   A fase 2 substitui os cues e a fase 4 substitui energia e similaridade.
3. **Escrita direta no banco** (`sync-rekordbox.py import`): recusar a execução
   com o Mixxx aberto (verificar o processo ou o lock do SQLite) e fazer backup
   automático do `mixxxdb.sqlite` antes de gravar.
4. **Validação pendente das fases anteriores**: teste auditivo de sync por
   compasso, grid e tonalidade, e teste visual da faixa de vocal na sessão real.
   Anotar os problemas antes de construir em cima.

Critério de saída: `git log` com o mod em commits e build limpo
(`cmake --build build --parallel 4`) com todos os testes atuais passando.

---

## Fase 1 — Análise de frases e PhraseSync

Referência: `PhraseSync` nos mapeamentos e frases Intro / Up / Chorus / Down /
Outro do rekordbox. Base já existente: compasso 1 automático, marcas de frase a
cada 8 compassos, detector de vocal.

### 1.1 Analisador `AnalyzerPhrases`

Novos `src/analyzer/analyzerphrases.{h,cpp}`, registrados em
`analyzerthread.cpp` como o `AnalyzerVocals`.

- `processSamples()`: acumula, em blocos de cerca de 50 ms, energia RMS,
  energia de graves (< 150 Hz), fluxo espectral e croma (12 bins). O grid ainda
  não existe nesse momento, porque os analisadores recebem o áudio em paralelo.
  Por isso as características são guardadas por tempo, não por compasso.
- `storeResults()`: lê `track->getBeats()`, já gravado pelo `AnalyzerBeats`.
  Sem `hasDownbeat()`, usa a primeira batida e marca o resultado como de baixa
  confiança.
- O resultado são **características por compasso**, não frases prontas. Assim,
  quando o usuário corrige o grid ou o SET 1, as frases são recalculadas sem
  decodificar o áudio de novo.

### 1.2 Segmentação e rótulos (`src/track/phrases.{h,cpp}`, puro e testável)

1. Matriz de auto-similaridade entre compassos (croma + timbre) e curva de
   novidade com kernel em tabuleiro (Foote).
2. Candidatos a limite apenas em múltiplos de 4 compassos a partir do compasso 1,
   com preferência por múltiplos de 8. Programação dinâmica escolhe os limites,
   penalizando frases menores que 4 compassos.
3. Rótulos por regras:
   - **Intro**: segmento inicial com graves abaixo da mediana da faixa.
   - **Outro**: espelho da Intro no final.
   - **Chorus**: segmentos de energia e graves mais altos. Ter vocal (pelo
     `VocalAnalysisStore`) aumenta a pontuação.
   - **Down**: queda de graves depois de um Chorus (breakdown).
   - **Up**: subida de energia antes de um Chorus (build-up).
   - **Verse**: o restante.
4. Cada frase registra uma confiança (0–1). Abaixo do limiar, a frase é
   mostrada sem rótulo.

### 1.3 Persistência

`PhraseAnalysisStore`, no mesmo padrão do `VocalAnalysisStore`. O cache fica em
`~/.mixxx/analysis/phrases/<hash>.json`.

- O hash inclui caminho, tamanho e data do arquivo, além da versão do algoritmo.
- O JSON guarda as características por compasso, as frases calculadas e o
  identificador do grid usado (BPM, primeira batida, `first_downbeat_index`).
- Ao mudar o grid (`Track::beatsUpdated`), recalcula as frases a partir das
  características guardadas.
- Não altera `mixxxdb.sqlite` nem o protobuf. Um Mixxx sem o mod continua
  compatível com o banco.

### 1.4 Controles em `[ChannelN]` (novo `PhraseControl`, em `src/engine/controls/`)

| Controle | Função |
| --- | --- |
| `phrase_type` | 0 nenhum, 1 Intro, 2 Up, 3 Chorus, 4 Down, 5 Verse, 6 Outro (somente leitura) |
| `phrase_index` | Índice da frase atual (somente leitura) |
| `phrase_bars_left` | Compassos até o fim da frase (somente leitura) |
| `phrase_valid` | Análise disponível (somente leitura) |
| `phrase_jump_next` / `phrase_jump_prev` | Salta para o início da próxima ou da frase atual, mantendo o tempo do compasso |
| `phrase_sync` | Liga ou desliga o PhraseSync (padrão 0, persistente) |

### 1.5 PhraseSync

Estende `BpmControl::getBarMatchOffset()`. Com `phrase_sync` ligado e as duas
faixas com frases válidas, ao dar PLAY o deck parado é alinhado para que
`compassos até o fim da frase` coincida com o do outro deck. O recuo vai de 0 a
31 batidas. O comportamento atual (`bar_sync`, recuo de 0 a 3) continua sendo o
padrão. A regra é a mesma de hoje: o alinhamento acontece só ao dar PLAY.

### 1.6 Interface

- Waveform e overview: faixa colorida por tipo de frase, reaproveitando o
  desenho da faixa de vocal (`woverview.cpp:706`, `waveformrenderbeat.cpp`).
  Novas propriedades de skin: `ShowPhraseRegions` e `PhraseColors`.
- Skins RbxOriginal e RbxFlowBars: rótulo `CHORUS · 3` (tipo e compassos
  restantes) junto ao contador `compasso:tempo`, mais o botão `PHRASE` ao lado
  do `N · BAR`.
- Menu da faixa → análise → "Reanalisar frases".

### 1.7 Testes (`src/test/phrasestest.cpp`, `analyzerphrases_test.cpp`)

- Áudio sintético a 120 BPM: 16 compassos só de agudos, 32 com bumbo e baixo,
  8 sem graves e 32 com bumbo e baixo. Esperado: Intro, Chorus, Down e Chorus,
  com limites exatos no compasso.
- Recalcular depois de SET 1 deslocado em 1 batida e de ½× e 2×.
- PhraseSync: integração no estilo do `BarSyncTest`, com dois decks e
  posições conhecidas.
- Cache: invalidação por mudança do arquivo e da versão do algoritmo.
- Validação manual: 20 faixas da biblioteca, anotando os limites certos e
  errados. A meta é ≥ 80% dos limites a no máximo 1 compasso de distância. O
  número registrado vale só para essa amostra.

---

## Fase 2 — Cues automáticos por frase

Substitui a parte de cues do `analyze-ml.py`.

- Ao concluir a fase 1, se a faixa não tiver os cues correspondentes:
  - `CueType::Intro`: do início da Intro ao início da primeira frase seguinte.
  - `CueType::Outro`: do início da Outro ao fim do áudio.
  - Hot cues nos inícios de Up, Chorus e Down, **apenas em slots vazios**, com
    rótulo e cor por tipo (padrão rekordbox: Chorus vermelho, Down azul, Up
    amarelo).
- Nunca sobrescreve cues do usuário. Respeita LOCK. Pode ser desfeito
  (seguindo o padrão do SET 1 automático).
- Controle `[ChannelN],auto_phrase_cues` (padrão 0, persistente) e opção nas
  preferências de análise.
- Testes: slots ocupados preservados, LOCK respeitado, faixa sem frases válidas
  não recebe cues.

---

## Fase 3 — Mix Point Link

Referência: `MixPointLink`, `MixPointLinkSilent`, `MixPointSelectNext/Prev` e
os ícones `mix_point_*` e `mpl_mix_in/out`.

- **Pontos**: saída = início do cue Outro (ou da última frase); entrada = início
  do cue Intro (ou CUE principal). `mix_point_select_next/prev` percorre os
  inícios de frase para escolher outro ponto.
- **Novo `MixPointControl`** (EngineControl) por deck:
  - `mix_point_link`: arma o deck parado. A cada callback, compara a posição do
    outro deck com o seu ponto de saída. Ao chegar lá, dá PLAY neste deck no
    ponto de entrada, alinhado por compasso (e por frase, se `phrase_sync`).
  - `mix_point_link_silent`: mesma coisa com o volume do canal em zero (pré-escuta).
  - `mix_point_link_exit`: desarma.
  - `mix_point_link_armed` / `mix_point_bars_to_go`: indicadores para a skin.
- Requer o mesmo BPM nos dois decks (SYNC ou tempo igual). Sem isso, avisa e não arma.
- Overview: marcadores de entrada e saída. Botão `LINK` no painel do deck.
- Testes: disparo no compasso exato, cancelamento por seek ou stop do outro
  deck, ausência de pontos.

---

## Fase 4 — Faixas relacionadas

Referência: `RelatedTracks` e os ícones `brows_trackSug*` e `trackSuggestio*`.

- **Métrica de energia própria**: um analisador leve (loudness integrada,
  densidade de ataques, razão graves/total) gera uma nota de 1 a 10. Fica em
  `~/.mixxx/analysis/blackmesa.sqlite` (banco separado; não migra o
  `mixxxdb.sqlite`).
- **Pontuação** em relação à faixa do deck ativo:
  - tom compatível na roda Camelot (mesmo tom, ±1, relativo maior/menor), via `KeyUtils`;
  - BPM a no máximo ±6%, considerando ½× e 2×;
  - diferença de energia;
  - opcional: gênero e artista.
- **Biblioteca**: novo `RelatedTracksFeature` (`src/library/related/`) na barra
  lateral ("Relacionadas"), que se atualiza ao carregar uma faixa num deck.
  Coluna de pontuação e de motivo (por exemplo `8A→9A · +2 BPM`).
- Testes: tabela de compatibilidade Camelot e BPM com dobro e metade.

---

## Fase 5 — Modos de pad

Referência: `pad/Slicers`, `KeyShift`, `Keyboard`, `PadFx*`, `*ReleaseFX*`
e `BeatJump`.

**Pergunta em aberto:** qual controladora é usada? Os pads físicos dependem de
um mapeamento JS (`res/controllers/`). Sem controladora, os modos ficam como
botões na skin.

- **Slicer** (`SlicerControl`): `slicer_enable` divide os 8 tempos atuais
  (2 compassos, alinhados ao compasso 1) em 8 fatias, e `slicer_slice_1..8`
  toca cada fatia. Ao soltar, volta à posição de origem, como no slip.
  `slicer_loop` repete a janela.
- **KeyShift / Keyboard**: `keyshift_N` (−6…+6) define `pitch_adjust` com a
  trava de tom ativa. O modo Keyboard também salta para o hot cue escolhido.
  Aproveita o código da fase de tonalidade.
- **Release FX**: `release_fx` desliga as unidades de efeito do deck, aplica
  echo ou vinyl brake de 1 compasso e, opcionalmente, para o deck. Implementar
  como cadeia de efeitos própria + controle.
- **BeatJump em pad**: já existe (`beatjump_N`). Basta a página na skin ou no mapeamento.
- Testes: fatias alinhadas ao compasso e retorno ao soltar.

---

## Fase 6 — Stems / Part Isolator

Referência: `ActivePart*`, `PartIsolator*`, `pad/Stems`, `DrumSwap*`.

Decisão necessária antes de começar:

- **A) Migrar o mod para o Mixxx 2.6**, que lê arquivos NI STEM e tem controles
  por stem. Custo: portar cerca de 1.500 linhas e as skins. Benefício: o motor de
  stems vem pronto e mantido.
- **B) Portar o suporte a stems da 2.6 para a 2.5.6.** O código toca a
  decodificação, o EngineBuffer e a mixagem, então o risco de conflito é alto.

Recomendação: **A**, depois de confirmar que já existe uma 2.6 estável.

Separação: Demucs aberto (htdemucs, MIT) offline, em processo auxiliar como o
detector de vocal. O resultado são 4 trilhas empacotadas em `.stem.mp4`, em
cache próprio sem tocar no arquivo original. Exige GPU ou vários minutos de CPU
por faixa. Rodar em fila, sob demanda (menu da faixa → "Separar stems").

Controles da skin: mute e solo por parte (Vocal, Drums, Bass, Inst) e filtro de
efeitos por parte (`FXPartSelect*`).

---

## Convenções para todas as fases

- Um documento por módulo (`FRASES.md`, `MIXPOINT.md`…), no mesmo formato de
  COMPASSOS.md, com controles, uso, limites e verificação.
- Backup antes de mexer em skin ou configuração (`backups-<módulo>/<data>`).
- Recursos opt-in na skin e controles persistentes com padrão conservador.
- Testes gtest com áudio sintético e filtros documentados, por exemplo
  `QT_QPA_PLATFORM=offscreen build/mixxx-test --gtest_filter='Phrase*'`.
- Um commit por fase na branch `blackmesa`.

> Fim do documento gerado por IA.
