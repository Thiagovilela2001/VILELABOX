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

### Segmentação e rótulos (revisão de 2026-10-12)

- Fronteiras: novidade espectral nos limites de 4 compassos, mais a entrada ou
  saída de voz sustentada (janelas de 8 compassos, com o detector Demucs).
  Trechos com mais de 32 compassos e com menos de 8 custam mais, então blocos
  muito longos são divididos nos limites de 8 compassos.
- Níveis medidos entre os percentis 10 e 90 da própria faixa (masters altas e
  planas também têm INTRO e DOWN).
- CHORUS: trecho alto. INTRO/OUTRO: primeiro/último trecho não alto, com pouca
  voz (ou até 16 compassos). DOWN: trecho baixo depois de um CHORUS. UP:
  trecho que sobe antes de um CHORUS (estendido para trás se a subida ocupa
  mais de uma frase). VERSE: o resto; voz aumenta a confiança.
- Histerese: uma frase de até 8 compassos perto do limiar herda o rótulo dos
  dois vizinhos, sem apagar fronteiras.

Conferência em faixas reais com `DISABLED_RealTrackHarness` (ver o teste). Em
`pulse_na_base`, a primeira versão dava INTRO de 64 compassos com voz desde os
14 s; agora: VERSE 32 + VERSE 32 + frase baixa de 8 + CHORUS 32. A correção
musical dos rótulos ainda depende da validação auditiva (VALIDACAO.md).

### Cache e memória

- Dois arquivos por faixa em `analysis/phrases/`: `<hash>.json` com as
  características, gravado uma vez por análise, na thread do analisador; e
  `<hash>.phrases.json` com frases e grid, regravado depois de edições de grid.
  Editar o grid nunca regrava as características.
- Os caches da primeira versão (um arquivo só, com tudo junto) continuam
  válidos: são lidos como arquivo de características e as frases embutidas são
  ignoradas, porque sempre são recalculadas a partir do grid atual.
- Edições de grid gravam uma única vez, 2 s depois da última, numa única thread
  de escrita, em ordem. Cada escrita tem um número de sequência por faixa, e
  uma escrita mais antiga nunca substitui uma mais nova.
- Ao fechar o programa, as escritas na fila terminam e as edições pendentes são
  gravadas antes de sair.
- Se a gravação falhar (sem permissão, disco cheio), o resultado continua na
  memória, um aviso vai para o log e a próxima edição tenta de novo.
- Memória: as características ocupam 184 bytes por bloco de 50 ms, cerca de
  13 MB por hora de áudio (1,1 MB por faixa de 5 minutos). Ficam na memória as
  faixas carregadas nos decks e as 32 analisadas mais recentemente; as outras
  são liberadas e relidas do disco quando a faixa é carregada.
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
