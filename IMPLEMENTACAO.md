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

