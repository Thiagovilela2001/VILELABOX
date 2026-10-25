# p6-3 — MEMORY: por onde a gravação do memory cue passa

Dossiê de proveniência. Registra o que foi observado em execução, com o artefato, o método e
o que continua desconhecido.

## Artefato

| Campo | Valor |
| --- | --- |
| Binário | `C:\Program Files\rekordbox\rekordbox 7.2.8\rekordbox.exe` |
| Versão | 7.2.8.0 |
| SHA-256 | `85e3a58524c5cd1e870ad59e23cc8806523175f80fb82d6059c68d57bdf9991a` |
| Imagem base | `0x140000000` |
| Processo da medição | pid 8032, iniciado 14:42 |
| Faixa carregada | Film — Aphex Twin, 147,98 BPM, tom Bb |
| Deck | 1 (o único carregado), parado |

## Instrumentação

Script `probe3.py` (privado), anexado por `frida-python` 17.24.3. Cinco ganchos, todos
verificados como ativos porque já dispararam em rodadas anteriores do mesmo dia:

| Endereço | O que é |
| --- | --- |
| `0x142c7d6f0` | slot 11 do comportamento **quantizado** (gravar cue com Q ligado) |
| `0x142ab0740` | slot 11 do comportamento **sem quantização** (gravar cue) |
| `0x142ab0e40` | `reqHotCueRec` |
| `0x142c22620` | `reqSetCue` (primeiro) |
| `0x142c25640` | `reqSetCue` (segundo) |

O deck foi localizado pelo gancho no slot 85 (chamado continuamente), que entrega o
ponteiro no primeiro argumento — `0x26225f0f5a8` nesta execução.

## Resultado

Janela de gravação: **14:48:40 → 14:50:40**. Dentro dela o botão `MEMORY` foi apertado e a
captura seguinte, às **14:50:29**, mostra a lista MEMORY CUE com uma linha nova em
**02:21.081** (além das anteriores em 00:00.000 e 00:19.445). Ou seja: o aperto aconteceu
**dentro** da janela e o cue foi gravado.

Nessa janela inteira, **nenhuma das cinco funções foi chamada**: zero eventos de slot 11
(quantizado ou normal), zero de `reqHotCueRec`, zero de `reqSetCue`. O log só contém a
descoberta do deck e o ruído da consulta periódica usada como sinal de vida.

## Conclusão

- **OBSERVADO**: o `MEMORY` grava o memory cue **sem passar por nenhuma das funções de cue
  do motor**. Não é o slot 11 (nem o normal nem o quantizado), não é `reqHotCueRec`, não é
  `reqSetCue`. Confirma e reforça a linha já registrada no `32-loops-cues.md` de que o deck
  não é acionado, agora com a lista exaustiva das cinco funções do caminho de cue.
- **INFERIDO**: a gravação acontece fora do motor — na camada de coleção/banco, coerente com
  o cue reaparecer na lista e sobreviver à troca de faixa. Não foi observada a chamada de
  escrita.
- **DESCONHECIDO**: qual função grava (e em que tabela/campo). Uma rodada futura pode
  instrumentar a escrita em arquivo do processo, ou a API de banco, e disparar `MEMORY` com
  o mesmo método.

## Observações de método que valem para a próxima rodada

- O processo do rekordbox **reiniciou três vezes** durante a sessão (pids 17796 → 13788 →
  18512 → 8032). Anexar a um pid morto devolve `VirtualAllocEx returned 0x5`; conferir o pid
  logo antes de anexar.
- Clique por invocação UIA não chega no botão `MEMORY`: o retorno reporta um elemento em
  (483,529) quando o botão está em (366,401). Para apertar botão de deck, o caminho confiável
  é o aperto humano; o duplo clique por ferramenta foi bloqueado por falta de consentimento.
- A janela do rekordbox pode estar minimizada sem janela "on-screen"; `ShowWindowAsync` com
  `SW_RESTORE` (9) no `MainWindowHandle` a traz de volta.
