# G0 — SLIP: reprodução em sombra durante loop automático

Duas rodadas idênticas no deck 1, uma com SLIP desligado e outra ligado: tocar,
abrir loop automático de 1 batida, deixar 7 s, sair. Botões por automação de
interface (UIA); tempo decorrido lido da tela (texto `MM:SS`) e chamadas
instrumentadas.

## Artefato e ambiente

| Campo | Valor |
| --- | --- |
| Programa | `rekordbox.exe` 7.2.8.0 (SHA-256 `85e3a585…9991a`), pid 15504 |
| Execução | Frida via REA, 2026-10-25, 134 pontos |
| Evidência REA | `ev_76d40307b2cdcd943ac04ebf12a8d09c969e83a4bfa9826c03f8b65757f404a7` (instrumentação), `ev_4ff007ee003649e2048d013dffc6ca97fdfa9994d507bed4a975f60403d24edb` (sequência) |
| Preparo | modo AU; tamanho reduzido de 16 para 1 batida com ½ quatro vezes (`reqLoopCut(1/2)` ×4) |

## Rodadas

| | Rodada A | Rodada B |
| --- | --- | --- |
| SLIP | desligado | ligado (`unit#14(1)` em 329079, entre as rodadas) |
| Abrir loop | `reqAutoBeatLoop(1/1)`, início 358479 (8,13 s) | `reqAutoBeatLoop(1/1)`, início 586794 (13,31 s) |
| Tela durante o loop (+2, +4, +6 s) | `00:08` | `00:13` |
| Sair (`unit#56`) | sem movimento de posição | `corePosMove(889578, tocar)` = 20,17 s, e `reqExitBackCueState` |
| Tela logo após sair | `00:08` | `00:20` |
| Tela 1,5 s depois | `00:10` | `00:22` |

Na rodada B o loop durou 7,01 s (clique em 332840, saída em 339853); 13,31 + 7,01 =
20,32 s. O salto para 20,17 s fica 0,15 s antes, dentro da latência entre o clique
e a execução da tarefa.

## Classificação

- **OBSERVADO:** com SLIP ligado, a reprodução continua em sombra durante o loop
  automático; ao sair, o deck salta para a posição em sombra e segue tocando. Com
  SLIP desligado, a saída continua da posição dentro do loop. A operação de loop é
  a mesma nos dois casos (`reqAutoBeatLoop`); a diferença está só na saída.
- **INFERIDO:** a sombra avança na mesma velocidade da reprodução.
- **DESCONHECIDO:** sombra em loop manual, hot cue, prévia de cue, scratch e
  RELOOP; desligar o SLIP com loop ativo (o VILELABOX descarta a sombra e a saída
  não salta); ligar o SLIP com loop já aberto.

## Rastreabilidade

- Especificação: `spec/rekordbox/30-deck.md` (SLIP)
- Motor: `vilelabox-audio/src/deck.rs` (`sombra`, `auto_loop_at`, `clear_loop`, `process_block`)
- Testes: `vilelabox-audio/tests/slip_sombra_tests.rs`
