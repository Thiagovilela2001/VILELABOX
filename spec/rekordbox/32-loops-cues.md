# 32 — Loops e pontos de disparo

## Escopo

Hot cues A..H, chamada e gravação (MEMORY) de memory cue, loop automático por
batidas com ½ e ×2, modo da caixa de loop (AU/MA) e o estado do Quantize,
operados pela tela.

Fora do escopo, por falta de evidência: escrita do MEMORY no banco, RELOOP, efeito de `IN`/`OUT`/`EXIT` do loop
manual, slip.

## Glossário

- **Hot cue:** ponto gravado num pad A..H.
- **Memory cue:** ponto salvo na lista de memória da faixa.
- **Tamanho do loop:** número de batidas da grade (frações abaixo de 1).
- **Modo AU:** caixa de loop mostra o tamanho em batidas.
- **Modo MA:** caixa de loop mostra `IN`, `OUT` e `EXIT`.

## Comportamento

### Hot cue

| Estado do pad | Ação | Resultado |
| --- | --- | --- |
| vazio | apertar | grava o hot cue na posição atual |
| com cue | apertar | posiciona no hot cue e toca, estando o deck parado ou tocando |
| qualquer | soltar | nenhum efeito observado |

### Memory cue

Chamar um memory cue define o ponto de cue nele e deixa o deck parado nele
(espera no cue).

| Ação | Resultado | Classe |
| --- | --- | --- |
| MEMORY | grava um memory cue na posição atual e a linha aparece na lista | OBSERVADO |
| MEMORY | o deck (transporte) não é acionado | OBSERVADO |
| MEMORY | não passa por nenhuma função de cue do motor: nem o slot 11 quantizado, nem o normal, nem `reqHotCueRec`, nem `reqSetCue` | OBSERVADO |
| MEMORY | a gravação acontece fora do motor: linha em `djmdCue` do banco da coleção, `Kind = 0`, posição em `InMsec` | OBSERVADO |
| hot cue | persiste na mesma tabela, `Kind = 1` | OBSERVADO |
| MEMORY | o ANLZ da faixa não é alterado; `PCOB` continua vazio | OBSERVADO |
| MEMORY | `Kind = 9` (uma ocorrência, 136215 ms) | DESCONHECIDO |
| MEMORY | qual função interna escreve | DESCONHECIDO |

A persistência no VILELABOX é decisão de produto, com armazenamento próprio (`docs/armazenamento-de-cues.md`); o que o rekordbox faz está em `evidence/p6-4-memory-persistencia.md`.

### Loop automático

| Ação | Resultado |
| --- | --- |
| clicar no tamanho, sem loop | abre loop desse tamanho começando na posição atual |
| ½ com loop | tamanho ÷ 2, mesmo início |
| ×2 com loop | tamanho × 2, mesmo início |
| clicar no tamanho, com loop | encerra o loop; a reprodução segue |
| clicar de novo depois | reabre com o último tamanho, na posição atual |

O tamanho é guardado por deck e vale entre faixas.

Tamanhos codificados como fração: numerador nos 16 bits baixos, denominador
nos 16 altos (`0x10010` = 16/1 = 16 batidas).

### AU/MA

| Estado | Ação | Resultado | Classe |
| --- | --- | --- | --- |
| AU | clicar em MA | caixa de loop passa a `IN`/`OUT`/`EXIT`; o deck não é acionado | OBSERVADO |
| MA | clicar em AU | caixa de loop volta ao tamanho em batidas | INFERIDO (inverso do observado) |
| MA | `IN`, `OUT`, `EXIT` | sem efeito implementado | DESCONHECIDO |

A troca de modo não altera o tamanho nem o loop ativo.

### Quantize

| Ação | Resultado | Classe |
| --- | --- | --- |
| clique | alterna ligado/desligado no deck | OBSERVADO |
| loop com Q ligado ou desligado | a operação de loop recebe o mesmo tamanho | OBSERVADO |
| Q ligado | troca o comportamento de cue/loop por um comportamento quantizado | INFERIDO |
| Q ligado | o ponto de cue é referenciado a um índice de batida da grade, limitado a `[0, n−2]` | INFERIDO (estático) |
| Q ligado | regra de arredondamento: **para baixo**, na batida da grade imediatamente antes ou sobre a posição | OBSERVADO |

Com Q ligado, o início do loop automático e o hot cue gravado caem na batida da grade
imediatamente antes ou exatamente sobre a posição; com Q desligado, a posição não é
ajustada. Sem grade de batidas (ou antes da primeira batida) não há nó anterior e a
posição fica intacta. No VILELABOX a grade vive no app, que quantiza e passa o início
pronto ao motor; o motor não conhece a grade.

O comportamento quantizado é um prefixo do normal, não uma implementação paralela:
quando as precondições dele não valem, ele delega à versão sem quantização com os
mesmos argumentos (`spec/rekordbox/evidence/p6-2-sync-slip-quantize.md`, seção
"Comparação estática"). Por isso a chamada de loop sai idêntica nas duas direções do Q.

Pendência que existia e foi resolvida em execução: o dossiê `p6-1` associa o slot 11 do
comportamento quantizado a uma captura feita com Q **desligado**. A medição mostrou que os
rótulos estão corretos — com Q ligado roda o prefixo quantizado, com Q desligado roda o
normal —, logo o rótulo do `p6-1` é que estava errado. Detalhe e saída em
`spec/rekordbox/evidence/p6-2-sync-slip-quantize.md`, seção "Medição em execução (rodada 2)".

## Constantes funcionais

| Nome | Valor | Unidade | Função |
| --- | --- | --- | --- |
| pads de hot cue | 8 | pad | A..H |
| fatores de ½ e ×2 | 1/2, 2 | — | multiplicam o tamanho do loop |
| tamanhos observados | ½, 1, 2, 4, 8, 16 | batida | sequência alcançada com ½ e ×2 |
| bits do numerador do tamanho | 16 baixos | bit | fração do tamanho do loop |
| bits do denominador do tamanho | 16 altos | bit | fração do tamanho do loop |

## Casos de teste

Origem: rekordbox 7.2.8 (dossiê `evidence/p6-1-loops-cues.md`).

1. Tocando em 37,41 s, loop 4 → loop de 4 batidas iniciando em 37,41 s; ×2 → 8;
   ×2 → 16; início sempre 37,41 s.
2. Loop 8; ½ quatro vezes → 4, 2, 1, ½ batida.
3. Hot cue A em ponto conhecido; deck parado em outro lugar; apertar A → toca a partir de A.
4. Memory cue em 25 ms, deck tocando; chamar → parado em 25 ms, em espera no cue.

Origem: rekordbox 7.2.8 (dossiê `evidence/p6-2-sync-slip-quantize.md`).

5. Q ligado, loop de 16 batidas (`0x10010`) → a operação de loop recebe o mesmo
   tamanho que com Q desligado.
6. AU → MA → caixa mostra `IN`/`OUT`/`EXIT`; nenhuma operação no deck.
7. MEMORY em 00:51.718 → nova linha de memory cue na lista; nenhuma operação no deck.

Origem: rekordbox 7.2.8 (dossiê `evidence/p6-2-sync-slip-quantize.md`, seção
"A regra de arredondamento"). Grade a 120 BPM com batidas em 1,0 / 1,5 / 2,0 s
(44,1 kHz: uma batida = 22 050 frames).

8. Q ligado, posição 1,7 s → início do loop em 1,5 s (66 150 frames); em 1,999 s
   também 1,5 s, nunca 2,0 s.
9. Q ligado, posição 1,5 s (sobre a batida) → início em 1,5 s.
10. Q desligado, posição 1,7 s → início em 1,7 s (74 970 frames), como antes.
11. Hot cue gravado em 1,7 s → 1,5 s com Q ligado; 1,7 s com Q desligado.
12. Sem grade, Q ligado, posição 1,7 s → 1,7 s; com a primeira batida em 1,0 s,
    posição 0,3 s → 0,3 s.

## Rastreabilidade

- Loops e cues: `evidence/p6-1-loops-cues.md`.
- Quantize, AU/MA, MEMORY: `evidence/p6-2-sync-slip-quantize.md`.
- Caminho da gravação do MEMORY: `evidence/p6-3-memory.md`.
- Persistência (banco vs ANLZ): `evidence/p6-4-memory-persistencia.md`.
- Testes: `vilelabox-audio/tests/loops_cues_spec_tests.rs`,
  `vilelabox-audio/tests/sync_slip_quantize_spec_tests.rs`, `vilelabox-app/src/sync.rs`, `vilelabox-app/src/loops.rs`.
- Casos de teste no fluxo do app (ações de `vilelabox-app/src/acoes.rs` aplicadas ao
  `AudioMixer`), todos em `vilelabox-app/src/loops_cues_e2e.rs`:

| Caso | Teste |
| --- | --- |
| 1 | `caso_1_loop_tocando_x2_duas_vezes_mantem_inicio` |
| 2 | `caso_2_loop_8_metade_quatro_vezes` |
| 3 | `caso_3_hot_cue_a_volta_e_toca_do_ponto` |
| 4 | `caso_4_memory_cue_tocando_chamar_para_em_espera` |
| 5 | `caso_5_q_nao_muda_o_tamanho_do_loop` |
| 6 | `caso_6_au_ma_so_troca_o_estado_da_caixa` (estado que alimenta a caixa; o desenho de `IN`/`OUT`/`EXIT` é da tela Slint e não é testado) |
| 7 | `caso_7_memory_grava_linha_sem_transporte` |
| 8, 9, 10 | `casos_8_9_10_quantize_do_inicio_do_loop` |
| 11 | `caso_11_hot_cue_quantizado_com_q` |
| 12 | `caso_12_sem_no_anterior_posicao_intacta` |
| persistência com loop e Q | `persistencia_com_loop_e_q_volta_ao_recarregar` |

## Perguntas abertas

- Escrita do MEMORY no banco; RELOOP.
- Efeito e ordem das operações de `IN`, `OUT` e `EXIT` no modo MA.
- Limites mínimo e máximo do tamanho do loop.
