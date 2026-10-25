# G0 — Transporte do deck: PLAY/PAUSE e CUE

Observação em execução do rekordbox com instrumentação das funções do motor do
deck, enquanto o usuário operava os botões da tela. Complementada por
decompilação no Ghidra para ligar cada chamada à função de transporte.

## Artefato e ambiente

| Campo | Valor |
| --- | --- |
| Programa | `rekordbox.exe` 7.2.8.0 (SHA-256 `85e3a58524c5cd1e870ad59e23cc8806523175f80fb82d6059c68d57bdf9991a`) |
| Análise estática | Ghidra 12.1.4, análise automática completa (projeto privado `rekordbox728`, fora do repositório) |
| Execução | Frida via REA, anexado ao processo em execução, 2026-10-25; botões clicados com o mouse no deck 1, faixa carregada |
| Evidência REA | `ev_69acaf76c0002b97f9458ee9e38cbb8e63be70b56ad9a84874d0f9accd179464` (instrumentação, 110 pontos), `ev_2377472f811609e040ebf0d867b1d6406c212949020e0438773e62cfe5d28566` (sequência capturada) |

## Pontos instrumentados

Endereços com base de imagem `0x140000000`.

| Rótulo | Endereço | Identificação |
| --- | --- | --- |
| `unit#31` | slot 31 da vtable `rbxfrm::DjPlayerUnit` @ `0x1454066b0` | dispara no PLAY a partir de parado |
| `unit#32` | slot 32, mesma vtable | dispara no PLAY com o deck tocando |
| `unit#50` | `0x142c919c0` (slot 50) | volta ao cue sem tocar; só age se um campo de estado do deck for zero |
| `unit#51` | `0x142c919a0` (slot 51) | volta ao cue tocando |
| `unit#52` | slot 52 | grava o cue na posição atual |
| `unitBackToCue` | `0x142c91a10` | comum a 50 e 51; repassa a flag "tocar" |
| `reqBackToCurrentCue` | `0x142ab5380` | cria `doBackToCurrentCueTask` (vtable `0x1453ecc00`) |
| `doBackToCurrentCue` | `0x142ab4ef0` | lógica de volta ao cue (`CueBehavior`) |
| `reqSetCurrentCue` | `0x142ab65b0` | cria `doSetCurrentCueTask` (vtable `0x1453ecad0`) |
| `reqExitBackCueState` | `0x142ab25c0` | cria `doExitBackCueStateTask` (vtable `0x1453ed020`) |
| `corePosMove` | `0x142b6fdc0` | move a posição do deck; 3º argumento = continuar tocando |

Cadeia estática confirmada no Ghidra: slot 50/51 → `0x142c91a10` →
`SlipBehavior::triggerBackToCurrentCue` (`0x142c2f5f0`) →
`reqBackToCurrentCue` → `doBackToCurrentCue` → `corePosMove`.

Em `corePosMove`, quando a posição pedida já é a atual: com "continuar tocando"
desligado e o deck tocando, chama o slot `+0x58` do núcleo; com a flag ligada e
o deck parado, chama o slot `+0x48`.

## Sequência capturada

Pausa de ~3 s entre passos; horários em ms (últimos dígitos do relógio do processo).

| Passo do usuário | Horário | Chamadas, em ordem |
| --- | --- | --- |
| PLAY, parado no cue | 79509 | `unit#31`, `reqExitBackCueState` |
| PLAY, tocando | 88426 | `unit#32` |
| CUE (clique), parado fora do cue | 95153 | `unit#52`, `reqSetCurrentCue(pos=394816)`, `corePosMove(pos=393151, tocar=0)`, `unit#50`, `unitBackToCue(tocar=0)`, `reqBackToCurrentCue`, `doBackToCurrentCue`, `corePosMove(393151, 0)` |
| CUE pressionado, no cue | 100192 | `unit#51`, `unitBackToCue(tocar=1)`, `reqBackToCurrentCue(1)`, `doBackToCurrentCue(1)`, `corePosMove(393151, 1)` |
| CUE solto | 105577 | `unit#50`, `unitBackToCue(0)`, `doBackToCurrentCue(0)`, `corePosMove(393151, 0)` |
| PLAY, parado no cue | 110245 | `unit#31`, `reqExitBackCueState` |
| CUE (clique), tocando | 119604 e 119688 | `unit#50` duas vezes (pressionar e soltar), cada uma com `doBackToCurrentCue(0)` e `corePosMove(393151, 0)` |

Coerência de unidade: entre o 1º e o 2º PLAY passaram 8,92 s; a posição do cue
gravado no 3º passo é 393151, que a 44,1 kHz são 8,915 s. A posição é contada
em amostras da faixa.

## Classificação

- **OBSERVADO:**
  - PLAY parado → toca e sai do estado de espera no cue; PLAY tocando → pausa.
  - CUE parado fora do cue → grava o cue na posição atual e posiciona nele, parado.
  - CUE pressionado parado no cue → toca a partir do cue enquanto pressionado.
  - CUE solto depois disso → volta ao cue e para.
  - CUE tocando → volta ao cue e para; soltar repete a volta ao cue, sem efeito novo.
- **INFERIDO:** a escolha entre `unit#50`, `unit#51` e `unit#52` é feita pela camada
  de interface conforme o estado do deck (o ponto de decisão não foi decompilado).
  A diferença de 1665 amostras entre `reqSetCurrentCue` e `corePosMove` no 3º passo
  não foi explicada (possível ajuste de quantização ou de quadro).
- **DESCONHECIDO:** efeito do QUANTIZE ligado; CUE com loop ativo; CUE com
  "Auto Cue"; comportamento com controladora (MIDI/HID) em vez do mouse.

## Rastreabilidade

- Especificação: `spec/rekordbox/30-deck.md`
- Implementação: `vilelabox-audio/src/deck.rs` (`play_pause`, `cue_down`, `cue_up`)
- Scripts privados: `third_party/rekordbox/re/ghidra/{ExportTransport,DecompileAddrs,FindVCalls}.java`
