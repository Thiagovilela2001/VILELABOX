# G0 — Desligar BEAT SYNC e KEY SYNC; tom exibido com MT desligado

Deck 1 com faixa de 163,00 BPM em `A`; deck 2 com faixa de 172,00 BPM em `F`.
Botões por automação de interface (UIA); BPM e tom lidos do cabeçalho do deck 1
na tela; chamadas instrumentadas na mesma sessão de `p6-6`.

## Artefato e ambiente

| Campo | Valor |
| --- | --- |
| Programa | `rekordbox.exe` 7.2.8.0 (SHA-256 `85e3a585…9991a`), pid 15504 |
| Evidência REA | `ev_76d40307b2cdcd943ac04ebf12a8d09c969e83a4bfa9826c03f8b65757f404a7` (instrumentação), `ev_b741ff36d1fd5f3c0730ff1203766c9cb5062c739c7386098c8ff5171fce0e1d` (sequência) |
| Tela | cabeçalho: BPM da faixa (`163.00`, não muda com o andamento) e tom com deslocamento (`A ±0`) |

## Sequência

| Clique | Horário | Chamadas | Tela, deck 1 |
| --- | --- | --- | --- |
| início | — | — | `163.00`, `A ±0` |
| MASTER no deck 2 | 543073 | nenhuma no deck | `A ±0` |
| BEAT SYNC no deck 1 (ligar) | 545012 | `unit#22(1)`, `reqSetBehavior` | **`Bb +1`** |
| BEAT SYNC no deck 1 (desligar) | 546935 | `unit#22(0)`, `reqSetBehavior` | **`Bb +1`** |
| KEY SYNC no deck 1 (ligar) | 548818 | `unit#45(…, 1)` | `Bb +1` |
| KEY SYNC no deck 1 (desligar) | 550714 | nenhuma no deck | `Bb +1` |
| MASTER no deck 1 | 552589 | nenhuma no deck | `Bb +1` |

## Leitura

- Ao ligar o BEAT SYNC, o deck 1 passou à velocidade 172 / 163 = 1,0552. O tom
  subiu 12 · log₂(1,0552) = 0,93 semitom, exibido arredondado como `+1` e com a
  nota transposta (`A` → `Bb`). O tom acompanhar a velocidade indica MT desligado
  no deck 1.
- Ao desligar o BEAT SYNC, o tom continuou `Bb +1`: a velocidade sincronizada foi
  mantida. A única chamada foi a troca do comportamento de sync; nenhuma chamada
  de andamento.
- Desligar o KEY SYNC não fez nenhuma chamada no deck, e o tom ficou como estava.
- Ligar o KEY SYNC fez a chamada, mas o tom não mudou (`Bb` com o deck 2 em `F`).

## Classificação

- **OBSERVADO:** desligar BEAT SYNC mantém a velocidade; desligar KEY SYNC mantém o
  deslocamento; com MT desligado, o tom exibido soma o efeito da velocidade,
  arredondado ao semitom; grafia `Bb` para a nota entre A e B.
- **INFERIDO:** MT desligado no deck 1 durante a captura.
- **DESCONHECIDO:** por que o KEY SYNC não deslocou `Bb` → `F`. A regra inferida
  em `p6-2` (menor distância em semitons) daria −5. Hipóteses: tons vizinhos no
  círculo de quintas não são deslocados, ou o KEY SYNC depende do MT ligado. Grafia
  das demais notas alteradas.

## Rastreabilidade

- Especificação: `spec/rekordbox/30-deck.md`
- App: `vilelabox-app/src/sync.rs` (`shift_exibido`, `transpor`), `presentation.rs`
- Teste: `sync::tests::mt_desligado_tom_segue_a_velocidade`
