# Mix Point Link

Abra PADS no deck parado e use LINK. A saída do deck que toca vem do início do
cue Outro, ou do início da última frase. A entrada do deck parado vem do cue
Intro ou do CUE principal. Sem pontos válidos, o Link não arma.

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
O motor verifica o cruzamento do ponto em cada callback e solicita seek e PLAY;
a resolução de disparo é de um callback, não um agendador de amostra individual.
O PLAY segue o alinhamento por compasso/frase quando QUANTIZE/SYNC e os controles
correspondentes estão ligados. Para precisão audível, confirme com seu buffer de
áudio real. O Link não efetua uma transição automática completa.

O overview mostra IN/OUT com `ShowMixPointMarkers`, habilitado nas duas skins.
Testes de integração ficam na suíte `BarSyncTest`, prefixo `MixPoint`.
