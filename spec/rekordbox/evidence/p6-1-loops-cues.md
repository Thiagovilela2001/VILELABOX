# G0 — Hot cues, memory cues, loops e quantize

Observação em execução do rekordbox com instrumentação, mesmo método de
`p6-0-transporte.md`, mais o relato do usuário sobre o que a tela mostrou.

## Artefato e ambiente

| Campo | Valor |
| --- | --- |
| Programa | `rekordbox.exe` 7.2.8.0 (SHA-256 `85e3a585…9991a`) |
| Execução | Frida via REA, 2026-10-25, mouse no deck 1, faixa com grade |
| Evidência REA | captura 1: `ev_7bd5d9cdc8e22ec08da282a98215bb1eaea5a6283a2d205a459dc1bbd78ed76f` (170 pontos), `ev_9b734921b09743cac22abbfe4aa423c39c3b020339804d4e11d3e91e07378f82`; captura 2: `ev_33ff36f426e4c56d3bfabfc98bf9d292b8d835563f5a850fec06d5bed931d876` (154 pontos), `ev_65721e6f7f76239fb1e7de076635c70be27cfc3af6c7099f2ee2242f17271179` |

## Métodos do deck identificados

Slots da vtable `rbxfrm::DjPlayerUnit` @ `0x1454066b0`; funções `req*` por
referência à vtable da tarefa que criam (resolvidas no Ghidra).

| Slot | Observado ao | Chamadas seguintes |
| --- | --- | --- |
| `unit#71(n)` | apertar pad vazio, tocando | `reqHotCueRec` `0x142ab0e40` (n, 0) → `QuantizedCueBehavior` slot 11 |
| `unit#69(n)` | apertar pad com cue | `QuantizedCueBehavior` slot 10 (lançamento) |
| `unit#70(n)` | soltar pad | `reqHotCueSamplerStop` `0x142ab1400` (n, 1) |
| `unit#61(1, fração)` | clicar no tamanho de loop | `reqAutoBeatLoop` `0x142aafd10`; início = posição atual |
| `unit#61(0, …)` + `unit#56` | sair do loop | — |
| `unit#57` | ½ | `reqLoopCut` `0x142ab32f0` com fração 1/2 |
| `unit#58` | ×2 | `reqLoopCut` com fração 2/1 |
| `unit#54(entrada_ms, saída_ms)` | setas de memory cue | `reqSetCurrentCue` + volta ao cue parado |
| `unit#9(0/1)` | Q | `reqSetBehavior` (troca o comportamento de cue) |

Frações em `u32`: numerador nos 16 bits baixos, denominador nos altos
(`0x20001` = 1/2, `0x10008` = 8).

## Sequências relevantes

- Loop 4 aberto com posição 1649781 (37,41 s); ×2 duas vezes: `reqLoopCut(2/1)`
  e novo loop de 8 e 16 batidas **com o mesmo início** 1649781.
- Loop 8; ½ quatro vezes: 4, 2, 1, ½ batida; início inalterado.
- Sair do loop e clicar de novo: `reqAutoBeatLoop` com o **último tamanho** (16 e ½
  nas capturas).
- Memory cue único em 25 ms (1102 amostras): `unit#54(25, -1)` →
  `reqSetCurrentCue(1102)` → `corePosMove(1102, tocar=0)`.

## Relato do usuário (tela do rekordbox)

- Pad A com cue, música tocando **ou parada**: pula para A e continua tocando.
- Loop com Quantize desligado: começa exatamente na posição atual.

## Classificação

- **OBSERVADO:** gravar hot cue em pad vazio; lançar hot cue = pular e tocar;
  loop por batidas a partir da posição (Q desligado); ½ e ×2 mantêm o início;
  sair encerra; reabrir usa o último tamanho; seta de memory cue define o cue e
  para nele; Q alterna um estado.
- **INFERIDO:** duração da batida tirada da grade na posição (o programa recebe a
  fração em batidas; a conversão para amostras não foi lida). Rajada de
  `unit#73` com pads 1..15 logo após abrir loop: atualização interna, não ação.
- **DESCONHECIDO:** efeito do Q ligado (loop e hot cue com quantização); gravar
  MEMORY (nenhum método do deck disparou; provável escrita direta no banco);
  RELOOP (nenhuma chamada própria); soltar o pad (`reqHotCueSamplerStop`) sem
  efeito visível no modo hot cue; limites de tamanho de loop; ordem das setas
  com vários memory cues.

## Rastreabilidade

- Especificação: `spec/rekordbox/32-loops-cues.md`
- Motor: `vilelabox-audio/src/deck.rs` (`jump_and_play`, `set_cue_and_standby`, `auto_loop`, `loop_cut`, `clear_loop`)
- App: `vilelabox-app/src/main.rs` (tratadores de loop, quantize e linhas de cue), `vilelabox-app/src/loops.rs`
- Testes: `vilelabox-audio/tests/loops_cues_spec_tests.rs`
