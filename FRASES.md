# Frases no VILELABOX (experimental)

Base mantida: Mixxx 2.5.6. Nenhum modelo proprietário é executado.

Nas preferências de análise de BPM, habilite **Analisar frases musicais**.
Para analisar uma faixa específica, use o menu Análise → **Reanalisar frases**,
mesmo com a análise automática desligada. O analisador aproveita a decodificação
do Mixxx, em blocos de aproximadamente 50 ms: RMS, graves abaixo de 150 Hz,
fluxo espectral, croma e distribuição espectral em seis bandas.

As características por tempo ficam em `analysis/phrases/<hash>.json` no perfil.
O hash inclui caminho, tamanho, data do arquivo e versão do algoritmo. O arquivo
contém também as frases e a serialização do grid, incluindo a referência de
compasso. Alterações de grid ou SET 1 recalculam a segmentação sem ler o áudio.
Esse recálculo é agendado na fila da interface, sem escrever cache no callback
de áudio que recebeu a edição do grid.
A segmentação agrega características por compasso, calcula novidade com um kernel
Foote local de oito compassos e escolhe limites por programação dinâmica em
múltiplos de quatro, preferindo oito. A matriz local evita memória quadrática.
Sem grid não há frases; sem SET 1 o resultado tem baixa confiança e não recebe
rótulos. Silêncio não gera frases.

Em `[ChannelN]`:

| Controle | Uso |
| --- | --- |
| `phrase_type` | 0 desconhecido, 1 Intro, 2 Up, 3 Chorus, 4 Down, 5 Verse, 6 Outro |
| `phrase_index` | Índice atual a partir de 0; −1 fora das frases |
| `phrase_bars_left` | Compassos restantes, arredondados para cima |
| `phrase_valid` | Há segmentação disponível |
| `phrase_jump_next`, `phrase_jump_prev` | Próxima frase / início da atual; preservam a fase do compasso |
| `phrase_sync` | Persistente, padrão 0; alinha ao dar PLAY, com QUANTIZE ou SYNC |

PhraseSync compara as batidas até o fim das frases e recua de 0 a 31 batidas,
com equivalência módulo 32. Para frases de durações diferentes, isso alinha a
fase de oito compassos; não garante finais simultâneos para qualquer duração.
Sem duas análises e referências válidas, usa `bar_sync` quando habilitado.
Saltos durante a reprodução continuam seguindo a quantização normal do Mixxx.

As skins versionadas RbxOriginal e RbxFlowBars mostram o botão PHRASE, o tipo
mais a contagem restante e uma faixa colorida no topo da waveform e overview.
`ShowPhraseRegions` habilita a faixa. `PhraseColors` aceita sete cores separadas
por vírgulas, na ordem da tabela incluindo desconhecido. Esses elementos são
opt-in para outras skins. Os marcadores regulares de oito compassos continuam
separados da segmentação musical.

Os rótulos são heurísticos, não uma identificação semântica garantida: energia,
graves, contexto e presença provável de voz podem confundir Verse e Chorus.
O teste em 20 faixas reais e a meta de 80% de limites a até um compasso permanecem
pendentes de anotação e escuta. Não foi medido um índice de acerto real.

Testes: `QT_QPA_PLATFORM=offscreen build/mixxx-test --resource-path "$PWD/res"
--gtest_filter='PhrasesTest.*:AnalyzerPhrasesTest.*:BarSyncTest.*'`.

Backups das skins antes das edições: `backups-phrases/`.
