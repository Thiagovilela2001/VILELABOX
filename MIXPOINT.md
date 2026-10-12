# Mix Point Link

Abra PADS no deck parado e use LINK. Sem pontos válidos, o Link não arma.

### Pontos automáticos

- **Entrada** (deck parado): cue Intro; senão o compasso 1 (SET 1), onde a
  música começa; senão o CUE principal. Sempre no início de um compasso.
- **Introdução**: compassos da entrada até o primeiro drop (frase CHORUS ou UP)
  ou até a primeira voz, o que vier antes, em múltiplos de 4 (4 a 64).
- **Saída** (deck tocando), do início ao fim da passagem: cue Outro até o fim;
  senão a frase OUTRO; senão o primeiro limite de frase de 8 compassos **depois
  da última voz**, para as vozes não se sobreporem (com pelo menos 8 compassos
  até o fim); senão 32 compassos antes do fim.
- **Disparo**: o deck parado começa de modo que o drop dele caia no fim da saída
  do outro deck (fim − introdução), no máximo 32 compassos antes do início da
  saída. Sem introdução conhecida, dispara no início da saída. Se esse ponto já
  passou, usa o início da saída ou o próximo limite de frase.
- Uma saída escolhida à mão (`mix_point_select_next/prev` com o deck tocando)
  dispara exatamente nela, como no rekordbox.

Indicadores novos: `mix_point_out_end_position`, `mix_point_out_manual` e
`mix_point_intro_bars`. Testes: `MixPointsTest.*`.

O BPM efetivo dos dois decks deve coincidir (tolerância 0,05 BPM). Use SYNC ou
ajuste os tempos antes de armar. O Link não ajusta BPM, volume nem crossfader
automaticamente. SILENT dispara com volume do canal em zero para pré-escuta;
aumentar esse volume continua sendo uma ação do usuário.

| Controle `[ChannelN]` | Uso |
| --- | --- |
| `mix_point_link` | Arma; novo acionamento cancela |
| `mix_point_link_silent` | Arma com lançamento silencioso |
| `mix_point_link_exit` | Cancela |
| `mix_point_link_armed` | Indicador de armado |
| `mix_point_bars_to_go` | Compassos até a saída do outro deck |
| `mix_point_select_next`, `mix_point_select_prev` | Selecionam início de frase: entrada parado, saída tocando |
| `mix_point_in_position`, `mix_point_out_position` | Posições em amostras estéreo para marcadores |
| `mix_point_error` | 0 nenhum, 1 deck inválido/tocando, 2 pontos ausentes, 3 BPM diferente, 4 cancelado por fonte |

Seek, stop, troca de faixa ou alteração de BPM na fonte cancelam o armado.
Seek no destino também cancela. Loops que provocam seek na fonte cancelam.
O motor verifica o cruzamento do ponto em cada callback e solicita seek e PLAY.
Como o deck só começa no callback seguinte, o disparo prevê onde o outro deck
estará nesse momento e entra o mesmo número de batidas depois do ponto de entrada.
No teste com buffer de 512 frames, a diferença de fase entre os decks ficou abaixo
de 1 ms com QUANTIZE desligado. Os pontos de entrada e saída são recalculados na
thread da interface quando os cues ou as frases mudam; o processamento de áudio
só lê valores prontos e não trava a faixa.
O PLAY segue o alinhamento por compasso/frase quando QUANTIZE/SYNC e os controles
correspondentes estão ligados. Para precisão audível, confirme com seu buffer de
áudio real. O Link não efetua uma transição automática completa.

O overview mostra IN/OUT com `ShowMixPointMarkers`, habilitado nas duas skins.
Testes de integração ficam na suíte `BarSyncTest`, prefixo `MixPoint`.
