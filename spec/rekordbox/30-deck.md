# 30 — Transporte do deck

## Escopo

Botões PLAY/PAUSE, CUE, MT, BEAT SYNC, MASTER, KEY SYNC e SLIP de um deck
operados pela tela, com uma faixa carregada, em dois decks.

Fora do escopo, por falta de evidência: efeito do QUANTIZE, Auto Cue,
controladoras MIDI/HID, reverse, reprodução em sombra do SLIP, efeito do MT no
tom sob ajuste de andamento, desligar BEAT SYNC e KEY SYNC, fader de andamento.
Loops e hot cues: `32-loops-cues.md`.

## Glossário

- **Ponto de cue:** posição, em amostras da faixa, para onde o CUE volta.
- **Espera no cue:** deck parado exatamente no ponto de cue.
- **Prévia de cue:** reprodução a partir do ponto de cue enquanto o CUE está pressionado.
- **Deck master:** deck de referência de andamento; só um por vez.
- **Velocidade:** razão entre o andamento tocado e o BPM da faixa.
- **Deslocamento de tom:** semitons somados ao tom da faixa (`±0`, `-2`).

## Comportamento

### PLAY/PAUSE

| Estado | Ação | Resultado |
| --- | --- | --- |
| parado (inclusive em espera no cue) | clique em PLAY | toca a partir da posição atual; sai da espera no cue |
| tocando | clique em PLAY | pausa na posição atual |

### CUE

O CUE reage ao pressionar e ao soltar separadamente.

| Estado ao pressionar | Pressionar | Soltar |
| --- | --- | --- |
| parado fora do ponto de cue | grava o ponto de cue na posição atual; fica parado nele (espera no cue) | volta ao ponto de cue, parado (sem efeito novo) |
| em espera no cue | começa a prévia de cue | volta ao ponto de cue e para |
| tocando | volta ao ponto de cue e para | volta ao ponto de cue, parado (sem efeito novo) |

### MT, BEAT SYNC, MASTER, KEY SYNC e SLIP

| Botão | Estado | Ação | Resultado | Classe |
| --- | --- | --- | --- | --- |
| MT | qualquer | clique | alterna o Master Tempo do deck | OBSERVADO |
| BEAT SYNC | deck é o master | ligar | só o indicador; andamento não muda | OBSERVADO |
| BEAT SYNC | deck não é o master | ligar | deck assume o andamento do master | OBSERVADO |
| BEAT SYNC | ligado | desligar | só o indicador; andamento atual mantido | DESCONHECIDO (escolha provisória) |
| MASTER | outro deck é master | clique | este deck vira o master; o deck em sync passa a seguir o novo master | OBSERVADO |
| KEY SYNC | tons no mesmo modo | ligar | desloca o tom para o tom do outro deck | OBSERVADO |
| KEY SYNC | modos diferentes (menor e maior) | ligar | sem regra; nada muda | DESCONHECIDO |
| KEY SYNC | ligado | desligar | só o indicador; deslocamento mantido | DESCONHECIDO (escolha provisória) |
| SLIP | qualquer | clique | alterna o estado | OBSERVADO |

Regras:

- MASTER é estado da aplicação, não do motor: a troca de master não aciona
  nenhuma operação no deck (OBSERVADO). Início: deck 1 master, como na captura.
- BEAT SYNC: velocidade = BPM do master ÷ BPM da faixa; a tela mostra o
  andamento resultante e a variação em porcentagem com uma casa (OBSERVADO:
  130,00 → 130,14 BPM, `+0,1 %`).
- KEY SYNC: o deslocamento é a menor distância em semitons, na faixa −6..+5
  (INFERIDO de um único caso: `Gm ±0` → `Fm -2`). A escolha no empate de 6
  semitons e a grafia de notas alteradas são DESCONHECIDAS.
- A referência do KEY SYNC é o outro deck, não o master de andamento: na
  captura o deck 1 era o master e casou com o tom do deck 2 (INFERIDO).
- SLIP: só o estado. Se o deck continua em sombra e volta ao soltar é
  DESCONHECIDO; o corte dos samplers ao ligar é efeito colateral (INFERIDO).
- MT: só o estado; o efeito no tom com o andamento ajustado é DESCONHECIDO.
- Nenhum dos cinco aplica processamento de tom ou sombra ao som nesta versão.

## Constantes funcionais

| Nome | Valor | Unidade | Função |
| --- | --- | --- | --- |
| unidade de posição | 1 | amostra da faixa | posição do deck e do ponto de cue |
| decks master | 1 | deck | só um deck é a referência de andamento |
| casas da variação de andamento | 1 | % | `+0,1 %` |

## Casos de teste

Origem: execução do rekordbox 7.2.8 com mouse no deck 1 (dossiê `evidence/p6-0-transporte.md`).

1. Faixa carregada, parado em 0. PLAY → tocando. Após ~8,9 s, PLAY → parado em ~8,9 s.
2. Parado em 8,915 s, ponto de cue em 0. Pressionar CUE → ponto de cue = 8,915 s, parado nele.
3. Em espera no cue em 8,915 s. Pressionar CUE → toca a partir de 8,915 s. Soltar → volta a 8,915 s, parado.
4. Tocando, ponto de cue em 8,915 s. Clique em CUE → volta a 8,915 s, parado.

Origem: rekordbox 7.2.8, deck 1 a 130,00 BPM e deck 2 a 130,14 BPM, botões
acionados por automação de interface (dossiê `evidence/p6-2-sync-slip-quantize.md`).

5. Deck 1 master, BEAT SYNC no deck 1 → 130,00 BPM, sem mudança.
6. BEAT SYNC ligado no deck 1, MASTER no deck 2 → deck 1 em 130,14 BPM, `+0,1 %`.
7. Deck 1 em `Gm ±0`, KEY SYNC → deck 1 em `Fm -2`.
8. SLIP ligar e desligar; MT alternar → só o estado muda.

## Rastreabilidade

- Transporte: `evidence/p6-0-transporte.md`.
- MT, BEAT SYNC, MASTER, KEY SYNC, SLIP: `evidence/p6-2-sync-slip-quantize.md`.
- Testes: `vilelabox-audio/tests/transport_spec_tests.rs`,
  `vilelabox-audio/tests/sync_slip_quantize_spec_tests.rs`, `vilelabox-app/src/sync.rs`.

## Perguntas abertas

- Diferença de ~38 ms entre a posição gravada como cue e a posição para onde o
  deck foi movido no caso 2.
- Comportamento com QUANTIZE ligado, loop ativo e Auto Cue.
- Comportamento por controladora.
- BEAT SYNC no deck que já é master: o que ele dispara além do indicador.
- Desligar BEAT SYNC e KEY SYNC; KEY SYNC entre modos diferentes.
- Reprodução em sombra do SLIP; efeito do MT com o andamento ajustado.
