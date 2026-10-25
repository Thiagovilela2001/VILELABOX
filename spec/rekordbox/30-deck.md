# 30 — Transporte do deck

## Escopo

Botões PLAY/PAUSE, CUE, MT, BEAT SYNC, MASTER, KEY SYNC e SLIP de um deck
operados pela tela, com uma faixa carregada, em dois decks.

Fora do escopo, por falta de evidência: efeito do QUANTIZE, Auto Cue,
controladoras MIDI/HID, reverse, sombra do SLIP fora do loop automático, efeito do MT no
tom sob ajuste de andamento, fader de andamento.
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
| MT | velocidade ≠ 1 | ligado | tom exibido é o da faixa (`A ±0`) | OBSERVADO (p6-8) |
| MT | velocidade ≠ 1 | desligado | tom exibido soma o efeito da velocidade (`Bb +1`) | OBSERVADO (p6-7, p6-8) |
| BEAT SYNC | deck é o master | ligar | só o indicador; andamento não muda | OBSERVADO |
| BEAT SYNC | deck não é o master | ligar | deck assume o andamento do master | OBSERVADO |
| BEAT SYNC | ligado | desligar | só o indicador; andamento atual mantido | OBSERVADO (p6-7) |
| MASTER | outro deck é master | clique | este deck vira o master; o deck em sync passa a seguir o novo master | OBSERVADO |
| KEY SYNC | tons no mesmo modo | ligar | desloca o tom para o tom do outro deck | OBSERVADO |
| KEY SYNC | modos diferentes (menor e maior) | ligar | sem regra; nada muda | DESCONHECIDO |
| KEY SYNC | ligado | desligar | só o indicador; deslocamento mantido | OBSERVADO (p6-7) |
| KEY SYNC | A contra F, qualquer deslocamento anterior | ligar | deck vai para `Bb +1` (subdominante de F) | OBSERVADO (p6-10) |
| KEY SYNC | já em `Bb` contra F | ligar | nada muda: o valor calculado já é o que está aplicado | OBSERVADO (p6-7, p6-10) |
| deslocamento de tom (`‹` `›`) | qualquer | clique | ±1 semitom por clique; o tom exibido transpõe junto | OBSERVADO (p6-10, `unit#41`) |
| KEY SYNC | A contra F (−4), deck de referência a 0,0 % de velocidade | ligar/desligar | chamada ao motor confirmada em memória (ponteiro do deck, argumento `0x1`); tom inalterado | OBSERVADO (p6-9) |
| SLIP | qualquer | clique | alterna o estado | OBSERVADO |
| SLIP | ligado, loop automático | sair do loop | salta para onde a música estaria sem o loop e segue tocando | OBSERVADO |
| SLIP | desligado, loop automático | sair do loop | segue da posição dentro do loop | OBSERVADO |

Regras:

- MASTER é estado da aplicação, não do motor: a troca de master não aciona
  nenhuma operação no deck (OBSERVADO). Início: deck 1 master, como na captura.
- BEAT SYNC: velocidade = BPM do master ÷ BPM da faixa; a tela mostra o
  andamento resultante e a variação em porcentagem com uma casa (OBSERVADO:
  130,00 → 130,14 BPM, `+0,1 %`).
- No VILELABOX, ligar BEAT SYNC ou iniciar um deck escravo já sincronizado casa
  também o número e a fração da batida usando as posições do próprio callback
  de áudio. A waveform compensa a velocidade de reprodução, para que grades
  com BPM original diferente apareçam alinhadas na mesma escala temporal.
- KEY SYNC: o destino é a tonalidade do outro deck, sua dominante (+7 semitons)
  ou sua subdominante (+5) — as três vizinhas no círculo de quintas — e vale o de
  menor deslocamento absoluto a partir do tom da própria faixa (INFERIDO de dois
  pares: `Gm` → `Fm -2` em p6-2 e `A` → `Bb +1` em p6-10). O deslocamento que o
  deck já tinha é ignorado: partindo de `G -2`, de `C +3` e de `F -4` contra um
  deck em `F`, o resultado foi `Bb +1` nas três (OBSERVADO, p6-10). O caso `Bb`
  contra `F` de p6-7 é o valor já aplicado, por isso "nada mudou"; a chamada ao
  motor nesses casos foi confirmada em memória (p6-9). Modo diferente (maior ×
  menor), desempate de ±6 semitons e grafia das notas alteradas além de `Bb`:
  DESCONHECIDOS.
- A referência do KEY SYNC é o outro deck, não o master de andamento: na
  captura o deck 1 era o master e casou com o tom do deck 2 (INFERIDO).
- SLIP: com o SLIP ligado, ao abrir um loop automático a reprodução segue em
  sombra, na mesma velocidade (INFERIDO), e a saída do loop salta para ela
  (OBSERVADO). Sombra em loop manual, hot cue, prévia de cue e RELOOP é
  DESCONHECIDA. Desligar o SLIP com loop ativo: DESCONHECIDO; o VILELABOX
  descarta a sombra. O corte dos samplers ao ligar é efeito colateral (INFERIDO).
- MT: com MT desligado, o tom exibido soma o efeito da velocidade,
  12 · log₂(velocidade) arredondado ao semitom (OBSERVADO: A a 172/163 →
  Bb +1); com MT ligado o tom exibido é o da faixa (OBSERVADO, alternado quatro
  vezes com o andamento em +5,5 %: p6-8). Grafia Bb observada; demais notas
  alteradas DESCONHECIDAS. O processamento do som com MT ligado (tom preso) não
  existe nesta versão.
- O controle de MT aparece na automação de interface ora como `MT`, ora com o
  texto `KEY RESET`, e o clique no segundo zera o deslocamento exibido
  (OBSERVADO, p6-8); se é o mesmo controle por outro caminho é INFERIDO.
- MT e KEY SYNC não aplicam processamento de tom ao som nesta versão.

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

Origem: rekordbox 7.2.8, automação de interface (dossiê evidence/p6-6-slip-sombra.md).

9. SLIP desligado; loop de 1 batida aberto em 8,13 s; 7 s depois, sair → segue de dentro do loop (~8 s).
10. SLIP ligado; loop de 1 batida aberto em 13,31 s; 7 s depois, sair → 20,3 s, tocando.

Origem: rekordbox 7.2.8, automação de interface (dossiê evidence/p6-8-mt-tom-exibido-keysync.md).

11. Deck 1 em `A` com andamento em +5,5 %, MT desligado → `Bb +1`; MT ligado → `A ±0`; MT desligado → `Bb +1`; o andamento não muda.
12. Deck 1 em `A` contra deck 2 em `F`, com BEAT SYNC ligado ou desligado e MT ligado ou desligado, KEY SYNC → tom e andamento dos dois decks inalterados.
13. Deck 2 em `F` contra deck 1 em `A` (mesmas quatro combinações), KEY SYNC → `F ±0` inalterado.

Origem: rekordbox 7.2.8, automação de interface e leitura de memória via REA/Frida
(dossiê evidence/p6-9-keysync-par-distante-fechado.md).

14. Deck 1 em `A` contra deck 2 em `F` (−4), deck 2 a 0,0 % de velocidade, KEY SYNC
    ligar e desligar → chamada `unit#45` confirmada com o ponteiro do deck 1,
    tom inalterado nos dois decks.

Origem: rekordbox 7.2.8, automação de interface e instrumentação REA/Frida
(dossiê evidence/p6-10-keysync-par-de-tonalidades.md).

15. Setas de deslocamento de tom no deck 1: cada clique muda 1 semitom e a tela
    transpõe o tom (`A ±0` → `Ab -1` → `G -2`; `A ±0` → `B +2` → `C +3`);
    chamada `unit#41` com o valor alvo.
16. Deck 1 em `A` contra deck 2 em `F`, partindo de `G -2`, de `C +3` e de
    `F -4`: KEY SYNC → `Bb +1` nas três; o deslocamento anterior é ignorado.
17. Deck 1 em `Bb` contra deck 2 em `F`: KEY SYNC → nada muda (o valor calculado
    já é `Bb +1`).

## Rastreabilidade

- Transporte: `evidence/p6-0-transporte.md`.
- MT, BEAT SYNC, MASTER, KEY SYNC, SLIP: `evidence/p6-2-sync-slip-quantize.md`.
- Sombra do SLIP: `evidence/p6-6-slip-sombra.md`; teste `vilelabox-audio/tests/slip_sombra_tests.rs`.
- Desligar BEAT SYNC e KEY SYNC, tom com MT desligado: `evidence/p6-7-desligar-sync.md`.
- MT no tom exibido e KEY SYNC entre tons distantes: `evidence/p6-8-mt-tom-exibido-keysync.md`.
- KEY SYNC com um deck a 0,0 % de velocidade, confirmação em memória:
  `evidence/p6-9-keysync-par-distante-fechado.md`.
- Regra do KEY SYNC e deslocamento manual de tom (`unit#41`):
  `evidence/p6-10-keysync-par-de-tonalidades.md`; teste
  `sync::tests::key_sync_escolhe_a_tonalidade_compativel_mais_proxima`.
- Testes: `vilelabox-audio/tests/transport_spec_tests.rs`,
  `vilelabox-audio/tests/sync_slip_quantize_spec_tests.rs`, `vilelabox-app/src/sync.rs`.

## Perguntas abertas

- Diferença de ~38 ms entre a posição gravada como cue e a posição para onde o
  deck foi movido no caso 2.
- Comportamento com QUANTIZE ligado, loop ativo e Auto Cue.
- Comportamento por controladora.
- BEAT SYNC no deck que já é master: o que ele dispara além do indicador.
- KEY SYNC entre modos diferentes; desempate de ±6 semitons; grafia das notas
  alteradas além de `Bb`; e a confirmação da regra atual (compatíveis com o outro
  deck, menor deslocamento a partir do tom da própria faixa, hoje INFERIDO de dois
  pares) num terceiro par de tonalidades — de preferência um em que a previsão
  difira da distância direta entre os dois tons.
- Por que o MASTER não troca nesta disposição de tela: o clique no MASTER do deck 1
  não tirou o master do deck 2, o que impede usar o deck 2 como seguidor e medir o
  par invertido sem trocar faixa (p6-10).
- Sombra do SLIP fora do loop automático.
