# G0 — KEY SYNC: o deslocamento depende do par de tonalidades, não do deslocamento atual

Rodada de continuação de `p6-8` e `p6-9`. Aqueles dossiês deixaram aberto por que
`Bb` contra `F` e `A` contra `F` não deslocavam o tom. Esta rodada encontra o
controle de deslocamento manual do deck, monta a condição do único caso positivo
conhecido (`Gm` → `Fm`, −2) e mostra que o KEY SYNC **ignora o deslocamento atual
e recalcula sempre o mesmo valor para o par de tonalidades**.

## Artefato e ambiente

| Campo | Valor |
| --- | --- |
| Programa | `rekordbox.exe` 7.2.8.0 (SHA-256 `85e3a585…9991a`), pid 15504 |
| Faixas | deck 1 `DJ Chap - 1luv` / `Footwork Frenzy` (163,00 BPM, tom `A`, em +5,5 %); deck 2 `BOMBABOMBABOMBA` / `fr012en` (172,00 BPM, tom `F`, em 0,00 %) |
| Instrumentação | 134 ganchos, o mesmo script de `p6-6`/`p6-7`/`p6-8` |
| Evidência REA | `ev_1d8f51ade488ca41d213e52e8f10802b64ebdc2f6f8f956fc82023a8bc6d5ea0` (instrumentação), `ev_7acae832fad111b5ef7751d02a73ca4a5a522bc8ae4eff51abf191f839f19eda` (sequência com os cliques e as chamadas) |

## O controle de deslocamento manual do deck

No cabeçalho do deck 1, entre o tom exibido e o deslocamento, há **duas setas de
12x12 px** nas abscissas 688 e 762, y 312. Cada clique muda o deslocamento em um
semitom, e a chamada vai para `unit#41` com **o valor alvo** no segundo argumento
(inteiro de 32 bits com sinal), não um passo:

| Clique | Horário | Chamada | Tela, deck 1 |
| --- | --- | --- | --- |
| seta esquerda | 898374 | `unit#41(deck, 0x0, …)` | `Bb +1` → `A ±0` |
| seta esquerda | 920487 | `unit#41(deck, 0xffffffff, …)` | `A ±0` → `Ab -1` |
| seta esquerda | 27660 | `unit#41(deck, 0xfffffffe, …)` | `Ab -1` → `G -2` |
| seta direita | 115412 | `unit#41(deck, 0x2, …)` | `Bb +1` → `B +2` |
| seta direita | 119445 | `unit#41(deck, 0x3, …)` | `B +2` → `C +3` |
| seta esquerda | 143074 | `unit#41(deck, 0xfffffffd, …)` | `G -2` → `F# -3` |
| seta esquerda | 147291 | `unit#41(deck, 0xfffffffc, …)` | `F# -3` → `F -4` |

- **OBSERVADO:** `unit#41` é o slot do deslocamento manual de tom; o argumento é o
  valor alvo (0, ±1, ±2, ±3, −4) e a tela acompanha exatamente, inclusive na
  transposição (`A` → `Ab` → `G` → `F#` → `F`, `A` → `B` → `C`).
- **OBSERVADO:** o rótulo do controle de MT alterna entre `MT` e `KEY RESET`
  conforme o deslocamento seja zero ou não — o que explica a leitura ambígua de
  `p6-8`. Não é o MT mudando sozinho.

## KEY SYNC com deslocamentos diferentes, mesmo par de tonalidades

O deck 2 (`F`) nunca foi tocado. Em quatro momentos o deck 1 recebeu um
deslocamento manual diferente e, logo depois, um clique em KEY SYNC (`unit#45`).
O deslocamento manual foi feito **pelo** deck, portanto o KEY SYNC partiu de
valores que não são os do `p6-2`:

| Deslocamento no momento do clique | Horário | Chamada | Tela depois |
| --- | --- | --- | --- |
| `G -2` (condição do `p6-2`: 2 semitons de `F`) | 46406 | `unit#45(0x7ff606feab50, 0x1, 0x2faabfb46e0)` | **`Bb +1`** |
| `C +3` | 125427 | `unit#45(…, 0x1, 0x2faabfb46e0)` | **`Bb +1`** |
| `F -4` (o próprio tom do deck 2) | 152717 | `unit#45(…, 0x1, 0x2faabfb46e0)` | **`Bb +1`** |
| `Bb +1` (já no valor calculado) | — | nenhuma mudança | `Bb +1` |

- **OBSERVADO:** partindo de −2, de +3 e de −4, o KEY SYNC levou o deck 1 ao
  **mesmo** valor, `Bb +1`. O deslocamento resultante não depende do deslocamento
  anterior; depende só do par de tonalidades (tom da faixa do deck × tom do outro
  deck). Isso também reinterpreta `p6-7` e `p6-8`: ali o "nada mudou" era o deck
  já estar em `Bb +1`, o valor que o KEY SYNC produz para `A` contra `F`.
- **OBSERVADO:** nenhuma chamada a `unit#41` acompanha o KEY SYNC; a mudança de
  deslocamento vem de dentro do caminho do próprio `unit#45`.

## Deck 2 não reage

Cinco cliques em KEY SYNC no deck 2 (`unit#45` com o ponteiro do deck 2,
`0x2faabfb39c0`) não mudaram nada: o deck 2 ficou em `F ±0` em todas, inclusive
depois de um clique em MASTER no deck 1 (que também não alterou o painel: o deck 1
seguiu em `5,5 %`, ou seja, continuou seguindo o deck 2). `unit#45` é chamado, e o
efeito é nenhum.

- **INFERIDO:** o KEY SYNC só age no deck que está seguindo o outro; no master ele
  não tem referência.
- **DESCONHECIDO:** por que o clique em MASTER no deck 1 não trocou o master nesta
  disposição de tela — precisa ser resolvido antes de usar o deck 2 como seguidor.

## A regra que explica todos os casos

Candidatos que o rekordbox parece considerar: a tonalidade do outro deck, sua
dominante (+7 semitons) e sua subdominante (+5) — as três vizinhas no círculo de
quintas, que é onde a mixagem harmônica costuma tratar tons como compatíveis. O
deslocamento escolhido é o de menor valor absoluto entre os que levam o tom da
faixa a um desses três.

Verificação contra os dados:

| Deck 1 (faixa) | Outro deck | Candidatos | Menor | Observado |
| --- | --- | --- | --- | --- |
| `A` (9) | `F` (5) | `F` −4, `C` +3, `Bb` +1 | **+1** | `Bb +1` (p6-10, três partidas) |
| `Gm` (7) | `Fm` (5) | `Fm` −2, `Cm` +5, `Ebm` +3 | **−2** | `Fm -2` (p6-2) |
| `Bb` (10) | `F` (5) | `F` −5, `C` +2, `Bb` 0 | **0** | nada mudou (p6-7) |

- **INFERIDO:** a regra acima. Ela reproduz os três pares medidos, inclusive os
  dois que pareciam contradizer a regra anterior ("menor distância até o tom do
  outro deck", que daria −4 para `A`/`F`).
- **DESCONHECIDO:** se a escolha no empate de módulos iguais (ex.: +3 contra −3)
  tem desempate; se modo diferente (maior × menor) entra no mesmo conjunto; e a
  grafia das notas alteradas além de `Bb`.
- **Risco declarado:** a regra vem de dois pares distintos (`A`/`F` e `Gm`/`Fm`).
  O terceiro (`Bb`/`F`) só confirma o caso "já está no valor". Um par novo é
  necessário para promover a regra de INFERIDO a OBSERVADO.

## O que a próxima rodada precisa

1. Um par novo de tonalidades, idealmente em que a previsão da regra seja
   diferente da "menor distância até o outro tom" — por exemplo faixa em `C` com
   o outro deck em `A` (regra: `C` +0/±? ; distância direta: +9 → −3).
2. Resolver por que o MASTER não troca nesta disposição, para poder usar os dois
   decks como seguidores e obter o par invertido sem trocar faixa.
3. Grafia das notas alteradas (só `Bb` observada) e desempate de ±6 semitons.

## Rastreabilidade

- Especificação: `spec/rekordbox/30-deck.md` (KEY SYNC e deslocamento manual).
- App: `vilelabox-app/src/sync.rs` (`key_sync_shift`, `transpor`).
- Teste: `sync::tests::key_sync_escolhe_a_tonalidade_compativel_mais_proxima`.
- Dossiês anteriores: `p6-2` (caso `Gm`/`Fm`), `p6-7` (`Bb`/`F`), `p6-8`
  (MT no tom exibido), `p6-9` (chamada confirmada em memória).
