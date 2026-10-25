# G0 — MT no tom exibido e KEY SYNC entre tons distantes

Rodada de continuação do dossiê `p6-7`. Dois alvos: (i) fechar o efeito do MT no
tom exibido, que o `p6-7` tinha apenas como INFERIDO; (ii) resolver a regra do
KEY SYNC, que ficou como pendência aberta no fim daquele dossiê.

Deck 1 com faixa de 163,00 BPM em `A` sincronizada com o deck 2 de 172,00 BPM em
`F`; botões acionados por automação de interface (UIA), tom e deslocamento lidos
do cabeçalho dos dois decks na tela, chamadas pelo mesmo conjunto de ganchos do
`p6-6`/`p6-7`.

## Artefato e ambiente

| Campo | Valor |
| --- | --- |
| Programa | `rekordbox.exe` 7.2.8.0 (SHA-256 `85e3a585…9991a`), pid 15504 |
| Faixas | deck 1 `DJ Chap - 1luv` / `Footwork Frenzy` (163,00 BPM, `A`, em +5,5 %); deck 2 `BOMBABOMBABOMBA` / `fr012en` (172,00 BPM, `F`, em 0,0 %) |
| Instrumentação | 134 ganchos (vtable `rbxfrm::DjPlayerUnit` @ `0x1454066b0` e as duas de comportamento), janela de 1 s, mesmos script e argumentos do `p6-6` |
| Evidência REA | `ev_22f90152da8c73a1a6b21aab1185b2383ceef118bc0189f9eeb9f5b07d8a6627` (instrumentação, 134 ganchos), `ev_7045bd7a0152723c58c8351a9794dca6c386529fb15d1bf9da7f27d201ac180e`, `ev_95212589e7a2ecbaf44b0c723b0442cae9ad16bb780a854a4185e3563915848b`, `ev_445f090f5ce0f74e3d468622b25075f89a114b001df7cb9e72627a899c726274` (sequências) |

Estado de partida: KEY SYNC ligado nos dois decks, BEAT SYNC desligado no deck 1,
deck 1 recebendo o andamento do deck 2 (+5,5 %, `5.5 %` no painel, `±10` de faixa
de andamento).

## MT e o tom exibido

Uma mesma sequência, repetida quatro vezes ao longo da rodada. O clique no
controle de MT dispara `unit#15` com o argumento 1 ao ligar e 0 ao desligar.

| Clique | Horário | Chamada | Tela, deck 1 |
| --- | --- | --- | --- |
| — | 101849 | `unit#15(0x1)` | `Bb +1` → **`A ±0`** |
| — | 215454 | `unit#15(0x0)` | `A ±0` → **`Bb +1`** |
| — | 218683 | `unit#15(0x1)` | `Bb +1` → **`A ±0`** |
| — | 221848 | `unit#15(0x0)` | `A ±0` → **`Bb +1`** |

O andamento não mudou em nenhum dos cliques (`172,00` e `5.5 %` nos quatro
estados).

**Leitura.** Com MT ligado o tom exibido é o da faixa (`A ±0`); com MT desligado
ele soma o efeito da velocidade (`Bb +1`, como em `p6-7`). O que o `p6-7` só
tinha como INFERIDO — MT desligado durante a captura — passa a OBSERVADO, porque
aqui a alternância foi feita e o argumento de `unit#15` acompanhou a tela nas
quatro transições.

## Controle de MT na árvore de interface

No mesmo ponto da tela a árvore de UIA expõe ora um botão de 50x35 sem nome com
os textos `KEY` e `RESET` (slot `unit#42`, argumento um ponteiro), ora um botão
de 38x22 com o nome `MT` (`unit#15`, argumentos 1 e 0). Os dois retângulos se
sobrepõem: o de 38x22 começa dentro do de 50x35.

| Clique | Horário | Chamada | Tela |
| --- | --- | --- | --- |
| rótulo `KEY RESET` (828,505) | 212144 | `unit#42(…, 0x7ff609774d10, …)` | `Bb +1` → **`A ±0`**, rótulo passa a `MT` |
| rótulo `KEY RESET` (828,505) | 506197 | `unit#42(…)` | `Bb +1`, sem mudança |

- **OBSERVADO:** as duas representações existem no mesmo ponto e a de 50x35
  aparece com o texto `KEY RESET` nas capturas em que o deslocamento exibido é
  diferente de zero.
- **INFERIDO:** `unit#42` é o mesmo controle do MT por outro caminho (zerar o
  deslocamento exibido equivale a prender o tom no da faixa). A árvore de
  interface não permite separar as duas hipóteses — o retângulo de 38x22 fica
  dentro do de 50x35 e o clique cai no primeiro que a busca encontrar.
- **DESCONHECIDO:** se `unit#42` só zera o deslocamento ou se também inverte o
  MT.

## KEY SYNC entre tons distantes

Dez cliques em KEY SYNC em quatro combinações, lendo o cabeçalho dos dois decks
logo depois de cada um. `unit#45` é o slot do botão; ele recebe o argumento 1
sempre, tanto ao ligar quanto ao desligar (o sentido não está no argumento, como
já registrado para o Q no `p6-2`).

| Deck | Par de tons | BEAT SYNC | MT | Cliques | Chamada | Tela |
| --- | --- | --- | --- | --- | --- | --- |
| 1 (segue o 2) | `A` contra `F` (−4) | desligado | desligado | 961325, 969693, 975457 | `unit#45(1)` | `Bb +1` ↔ `Bb +1`, sem mudança |
| 1 | `A` contra `F` (−4) | ligado (966920) | desligado | 969693 | `unit#45(1)` | sem mudança |
| 1 | `A` contra `F` (−4) | desligado | ligado (101849) | 126098 | `unit#45(1)` | `A ±0` → `Bb +1` |
| 1 | `A` contra `F` (−4) | ligado (142323) | ligado | 146362 | `unit#45(1)` | `Bb +1`, sem mudança |
| 1 | `A` contra `F` (−4) | ligado (363484) | desligado | 369294 | `unit#45(1)` | `Bb +1`, sem mudança |
| 2 (segue o 1) | `F` contra `A` (+4) | desligado | desligado | 323352, 340368 | `unit#45(1)` | `F ±0`, sem mudança |
| 2 | `F` contra `A` (+4) | ligado (408017) | desligado (402172) | 414645 | `unit#45(1)` | `F ±0`, sem mudança |
| 2 | `F` contra `A` (+4) | ligado | desligado (421254) | 427582 | `unit#45(1)` | `F ±0`, sem mudança |

Nem o andamento nem o tom mudaram em nenhuma das dez tentativas; nenhuma chamada
de MT (`unit#15`) ou de andamento apareceu junto. Dois cliques (129961, 138480)
não geraram chamada nenhuma — o botão já estava no estado pedido.

**Leitura.**

- **OBSERVADO:** com `A` contra `F` (−4) e com `F` contra `A` (+4), o KEY SYNC não
  desloca o tom em nenhuma combinação de BEAT SYNC e MT; com `Bb` contra `F` o
  `p6-7` já tinha visto o mesmo. A regra de menor distância em semitons, inferida
  no `p6-2` a partir de um único caso (`Gm` → `Fm`, −2), não descreve nenhum dos
  três pares medidos desde então.
- **INFERIDO:** ao menos um estado que não foi variado controla o deslocamento.
  Candidatos, nenhum verificado: o KEY SYNC só deslocar tons do mesmo modo e a
  menos de dois semitons; depender de o deck estar tocando; depender de o outro
  deck ser o master de andamento; ou o deslocamento ser aplicado no deck que não
  está em foco.
- **DESCONHECIDO:** a regra. A captura não tem, em nenhum momento, dois decks com
  velocidade zero e tons próximos — a condição do único caso positivo conhecido.

## O que a próxima rodada precisa

1. Duas faixas de tons a 1 ou 2 semitons de distância (por exemplo `Am` e `Gm`)
   com os dois decks em 0,0 % de velocidade, para repetir a condição do `p6-2`
   antes de variar MT e BEAT SYNC.
2. Alternar o MT nos dois decks e o MASTER em cada um, um por vez, lendo os dois
   cabeçalhos.
3. Se a tela continuar ambígua, ler o deslocamento direto da memória: o ponteiro
   do deck aparece como terceiro argumento das chamadas (`0x2faabfb46e0` para o
   deck 1 e `0x2faabfb39c0` para o deck 2 nesta execução) e um gancho em
   `unit#45` entrega o objeto no `onEnter`.

## Rastreabilidade

- Especificação: `spec/rekordbox/30-deck.md` (MT e KEY SYNC).
- App: `vilelabox-app/src/sync.rs` (`shift_exibido`, `transpor`, `rotulo_shift`),
  `presentation.rs`.
- Teste: `sync::tests::mt_desligado_tom_segue_a_velocidade`.
- Dossiês anteriores: `p6-2-sync-slip-quantize.md` (slots e o caso `Gm`/`Fm`),
  `p6-7-desligar-sync.md` (desligar BEAT SYNC e KEY SYNC, tom com MT desligado).
