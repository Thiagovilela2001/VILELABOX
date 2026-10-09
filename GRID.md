# Grid no VILELABOX

## Editor

O painel junto à visão geral de cada deck reúne GRID (alinhar uma batida à posição atual), setas para deslocar todo o grid, SET 1 (compasso 1 / tempo 1), LOCK, BPM− / BPM+ (0,01 BPM), TAP, ½×, 2×, UNDO e leitura do BPM original com duas casas decimais. Botão direito em GRID alinha com o outro deck; em SET 1 remove a referência de compasso.

LOCK protege as edições. Os controles de grid alteram as marcações da faixa; os controles de tempo de reprodução continuam separados.

## Análise automática

Detector selecionado: **VILELABOX — Grid preciso e primeiro tempo**, nas preferências de análise de BPM. Base Queen Mary com refinamento dos ataques em janelas de aproximadamente 1 ms, busca limitada a 25 ms em torno das batidas e estimação de primeiro tempo em 4/4 pelo detector espectral DownBeat. Uma margem mínima entre os candidatos evita aceitar fases de compasso indistinguíveis; quando não há resultado suficiente, continua disponível a referência automática do CUE e a correção por SET 1.

- **Normal**: gera um grid de BPM fixo, apropriado para andamento constante.
- **Dinâmico**: acompanha mudanças de andamento.

Na biblioteca, use o menu da faixa → análise → reanalisar. O modo oposto ao padrão aparece como **Reanalisar — Normal (BPM fixo)** ou **Reanalisar — Dinâmico (BPM variável)**. Destrave o BPM antes de reanalisar.

A análise rápida foi desativada para analisar a faixa inteira. Os grids anteriores não são substituídos automaticamente. O detector Queen Mary original e SoundTouch seguem disponíveis nas preferências.

## Referência do rekordbox e limites

Observação local estática, sem execução do rekordbox:

- G1: `Downloads/rekordbox_extracted/MidiMappings/DDJ-RX.midi.csv`, linhas 65–68, declara GridAdjust, GridSlide e ajustes ½× / 2×.
- G2: os recursos `skins/Grid_Button_Set1stBeatOfBar.svg`, `Grid_Button_MoveLeft.svg`, `Grid_Button_ExpandIntvl.svg`, `grid_beatHalfBtn.svg`, `grid_beat2timesBtn.svg` e `Grid_Export_Button_Undo.svg` indicam controles do editor.
- G3: `rekordbox.exe` contém `Use high precision BeatGrid analysis` no offset de arquivo 0x3b48768; os caminhos `models/detect_beat/model7/` e `model8/` nos offsets 0x3cdb7b0 e 0x3cdb7d0; e o nome RTTI `RNNDownBeatTrackingObservationModel@madmom` em 0x5e38c20. Isso indica componentes de análise neural, sem estabelecer seu pré-processamento, fluxo completo ou qualidade em execução.
- G4: os dois diretórios de modelo contêm SavedModel TensorFlow e variáveis. SHA-256 dos saved_model.pb: model7 `ea7b5e7c3567525bb9b0eff1238c13b9d4ab8336399e0f2dd3301528a9dc936a`; model8 `09baf025fb561d27264a6f26522e12037fd2f37bf1db378c7edf9366116c49dd`.

A implementação usa o motor aberto disponível no projeto e adapta a experiência de edição e os modos de análise. Não executa os modelos proprietários do rekordbox nem garante resultados idênticos. A precisão em músicas reais depende do arranjo e da clareza das batidas; SET 1, ½× e 2× permitem corrigir as ambiguidades.

## Verificação e restauração

Validação final: **91 testes passaram**, sem falhas. Testes de áudio gerado com BPM conhecido em 44,1 e 48 kHz verificam alinhamento, BPM fixo, mudança de andamento, silêncio e reutilização do analisador. Nos sinais de 90, 120 e 150 BPM, o erro mediano de alinhamento ficou entre 0 e 0,36 ms; essa medida é restrita aos sinais sintéticos e não estabelece a precisão em músicas reais. O teste também compara com o detector original e confirma melhora no alinhamento. O modo Dinâmico foi verificado com uma mudança de 120 para 90 BPM. Testes existentes verificam edição, trava, desfazer e referências de compasso. A validação auditiva e visual na sessão gráfica do usuário permanece pendente, pois este ambiente não acessa sua tela nem o servidor de áudio.

Backups em `backups-grid/20261009-084954` (painel), `backups-grid/20261009-085353` (fontes) e `backups-grid/mixxx.cfg-20261009-085442` (configuração).

Reabra o VILELABOX pelo atalho habitual para carregar o binário recompilado.
