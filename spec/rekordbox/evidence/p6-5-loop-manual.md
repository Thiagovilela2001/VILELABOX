# G0 — Loop manual: IN, OUT, EXIT e RELOOP

Observação em execução, deck 1 no modo MA, Quantize no estado em que o
rekordbox estava. Botões acionados por automação de interface (invocação UIA na
janela, script privado `uia_click.ps1`); horário de cada clique registrado pelo
mesmo relógio do sistema que a instrumentação usa.

## Artefato e ambiente

| Campo | Valor |
| --- | --- |
| Programa | `rekordbox.exe` 7.2.8.0 (SHA-256 `85e3a585…9991a`), pid 15504 |
| Execução | Frida via REA, 2026-10-25, 150 pontos (mesmo conjunto de `p6-1`) |
| Evidência REA | `ev_ea67c4b0269f34481bdb4b9ed5eb024261a05ff47579ff7c9f7245ceedbad342` (instrumentação), `ev_089cc9f39a612bb9b7cef29a42aa05e027c37806c66eb949590515bdcb362cc3` (sequência) |
| Tela | nomes UIA: `IN` e `OUT` no lugar da caixa de tamanho; `RELOOP` no lugar do ‹, que vira `EXIT` com loop ativo; tempo decorrido do deck como texto `MM:SS` |

## Sequência capturada

| Clique (UIA) | Horário | Chamadas |
| --- | --- | --- |
| `MA` | 720458 | nenhuma no deck |
| `Reproduzir/Pause` | 731246 | `unit#31`, `reqExitBackCueState` |
| `IN` | 734447 | `unit#52` → `reqSetCurrentCue(entrada=141120, saída=−1)` |
| `OUT` | 738665 | `unit#53` → `reqSetCurrentCue(entrada=−1, saída=327164)` → `doLoop` (`QuantizedCueBehavior` slot 8, flag 0) |
| `EXIT` | 752087 | `unit#56`, `unit#61(0, 0x10000)` |
| `RELOOP` | 756802 | `unit#55` → `reqReloop` `0x142ab4890` → `QuantizedCueBehavior` slot 7 → slot 8 (flag 1) |
| `EXIT` | 761485 | `unit#56`, `unit#61(0, 0x10000)` |
| `Reproduzir/Pause` | 764129 | `unit#32` |

Unidades: 141120 amostras = 3,20 s (3,2 s depois do PLAY); 327164 = 7,42 s
(4,2 s depois do IN). As posições são as do deck no clique.

## Efeito na tela

Segunda execução, lendo o tempo decorrido do deck 1 por UIA:

| Momento | Tempo |
| --- | --- |
| tocando, depois do OUT | `00:12` |
| logo depois de `RELOOP` | `00:03` |
| 5 s depois | `00:04` (3,2 + 5,3 = 8,5 s; passou do OUT em 7,42 s e voltou a 4,3 s) |

## Classificação

- **OBSERVADO:** IN grava a entrada na posição e a torna o ponto de cue, sem
  mexer no transporte; OUT grava a saída na posição e começa a repetir IN..OUT;
  EXIT (mesma chamada da saída do loop automático) encerra e a reprodução segue;
  RELOOP volta à entrada e repete o último IN..OUT; RELOOP e EXIT são o mesmo
  botão, conforme haja loop ativo.
- **INFERIDO:** a flag 1 do slot 8 no RELOOP pede o salto para a entrada (a flag
  0 no OUT dispensa o salto, porque a posição já está na saída).
- **DESCONHECIDO:** OUT sem IN; OUT antes do IN; IN com loop ativo (ajuste da
  entrada); RELOOP com o deck parado; RELOOP depois de um loop automático;
  efeito do Quantize ligado no IN/OUT; a diferença entre a posição passada ao
  comportamento quantizado (141120) e ao normal (148484) no IN.

## Rastreabilidade

- Especificação: `spec/rekordbox/32-loops-cues.md`, seção "Loop manual"
- Motor: `vilelabox-audio/src/deck.rs` (`set_cue_point`, `set_loop`, `seek_frame`, `clear_loop`)
- App: `vilelabox-app/src/acoes.rs` (`loop_in`, `loop_out`, `loop_exit_reloop`), tela em `deck.slint`
- Teste: `vilelabox-app/src/loops_cues_e2e.rs` (`loop_manual_in_out_exit_reloop`)
