# G0 — Botões restantes do deck: SLIP, MT, BEAT SYNC, MASTER, KEY SYNC, AU/MA, Q e MEMORY

Observação em execução do rekordbox com instrumentação (Frida via REA), mesmo método
de `p6-0-transporte.md` e `p6-1-loops-cues.md`. Nesta rodada os botões foram acionados
pela ferramenta de automação de interface (invocação UIA na janela), não pelo mouse do
usuário, e a atribuição clique → chamada foi feita pela ordem dos cliques e pelo
horário de cada cluster de chamadas.

## Artefato e ambiente

| Campo | Valor |
| --- | --- |
| Programa | `rekordbox.exe` 7.2.8.0 (SHA-256 `85e3a58524c5cd1e870ad59e23cc8806523175f80fb82d6059c68d57bdf9991a`) |
| Execução | Frida via REA, anexado ao processo em execução (pid 17796), 2026-10-25, deck 1, modo performance, dois decks horizontais |
| Faixa no deck 1 | 130,00 BPM; deck 2 em 130,14 BPM |
| Instrumentação | gancho em todos os slots da vtable `rbxfrm::DjPlayerUnit` @ `0x1454066b0` (123 ganchos válidos), contagem por janela de 1 s, mais gancho com argumentos nos slots 9, 14, 15, 19, 22, 31, 32, 45, 47, 53, 54, 56, 57, 58, 61, 65, 73 e 96 |
| Scripts | `cap_p7c.js` (contagem, sha256 `2c6f4eff…412bc3`), `cap_p7d.js` (argumentos, sha256 `99528e92…4c448f`), `cap_p7e.js` (dump de endereços, sha256 `122129b4…f84193`) |
| Evidência REA | instrumentação `ev_bf5dc066a21f60501d16c717a3cb393d23e5f87a7704ae71168d46b48e15caec`, `ev_da36c4381b7ec45ab0b55c9dac0e12856aa32da2f113402726f0b95d1e423719`, `ev_24e31ceddf64d33825655116ab9e5dd5115a3330aa4a4f7d3ec99b5bc6eacec2`; leituras `ev_369dbe1b…`, `ev_9f18e456…`, `ev_ca669e81…`, `ev_08f80035…`, `ev_21e76fb0…`, `ev_d01237d4…`, `ev_8e5ddef7…`, `ev_8ce33df7…`, `ev_d37ac7f5…`, `ev_5b0bd0bd…`, `ev_78a3eb96…` |

Ruído de base, presente em toda janela e descartado: `unit#85`, `unit#78`, `unit#74`,
`unit#29`, `unit#75`. Nenhuma outra função disparou sem clique.

## Slots identificados nesta rodada

Endereços com base de imagem `0x140000000`; slot = índice na vtable `rbxfrm::DjPlayerUnit`.

| Botão na tela | Slot | Endereço | Observado em |
| --- | --- | --- | --- |
| SLIP | `unit#14` | `0x142c93910` | ligar e desligar |
| MT (Master Tempo) | `unit#15` | `0x142c93870` | alternar (a tela passa a mostrar `KEY RESET`) |
| BEAT SYNC | `unit#22` | `0x142c93360` | ligar |
| KEY SYNC | `unit#45` | `0x142c91f60` | ligar |
| Q (Quantize) | `unit#9` | `0x142c93d20` | ligar e desligar |
| Loop manual (ponto de entrada) | `unit#53` | `0x142c915a0` | clique no `IN` do modo MA |
| Tamanho de loop (AU) | `unit#61` | `0x142c909c0` | abrir e fechar |
| Sair do loop | `unit#56` | `0x142c911b0` | fechar |
| Pads (atualização interna) | `unit#65`, `unit#19`, `unit#73` | `0x142c90700`, `0x142c93580`, `0x142c90080` | rajada nos pads 1..15 |

## Sequência capturada

Horários em ms (últimos dígitos do relógio do processo); cada clique produziu exatamente
um cluster, separados por 7–10 s.

| Ação | Horário | Chamadas, em ordem |
| --- | --- | --- |
| SLIP ligar | 353037 | `unit#14`, `reqHotCueSamplerStop(0, 0xffffffff, 0)` |
| SLIP desligar | 634497 | `unit#14` |
| MT | 643500 | `unit#15` |
| BEAT SYNC (deck 1 era master) | 653503 | `unit#22`; nada mais mudou |
| KEY SYNC | 663506 | `unit#45`; tonalidade do deck 1 passou de `Gm ±0` para `Fm -2` |
| AU → MA | 732424 | nenhuma chamada de motor; a caixa de tamanho vira `IN`/`OUT`/`EXIT`; rajada `unit#65`×3, `unit#19`×3, `unit#73`×15, `reqHotCueRec` nos pads 1..15 |
| MEMORY | 832381 | nenhum método do deck; a mesma rajada `reqHotCueRec` 1..15 com o mesmo ponteiro de estado |
| Q ligar | 869381 | `unit#9`, `reqSetBehavior(…, 0x68, deck)` |
| Caixa de loop, Q ligado | 960057 | `unit#61`, `reqAutoBeatLoop(deck, 0x10010, 0)` |
| Fechar loop | 978654 | `unit#56(0, …)`, `unit#61(0, 0x10000, 0x1)` |
| Q desligar | 995132 | `unit#9`, `reqSetBehavior(…, 0x68, deck)` |
| Caixa de loop, Q desligado | 3957 (volta de 1 000 000) | `unit#61`, `reqAutoBeatLoop(deck, 0x10010, 0)` |
| MASTER no deck 2 | — | nenhuma chamada na vtable do deck; o deck 1 passou a `130,14 BPM / +0,1 %` |

Argumentos por slot, para referência:

| Slot | `args[1]` | `args[2]` | `args[3]` | `args[4]` |
| --- | --- | --- | --- | --- |
| `unit#9` | `0x1` (igual nas duas direções) | — | `0x19999999` | `0x0` |
| `unit#61` (abrir) | deck | `0x1` | `0x10010` | `0x1000000010000` |
| `unit#61` (fechar) | deck | `0x0` | `0x10000` | `0x1` |
| `unit#56` (fechar) | deck | `0x0` | — | — |

Frações seguem a codificação já registrada em `p6-1-loops-cues.md`: numerador nos 16 bits
baixos, denominador nos altos — `0x10010` = 16/1 = 16 batidas.

## Comportamento na tela

- **Q:** alterna o indicador; as chamadas de loop têm **os mesmos argumentos** com o Q
  ligado e desligado.
- **KEY SYNC:** com o deck 1 em `Gm` e o deck 2 em outra tonalidade, o deck 1 passou a
  `Fm` com deslocamento `-2` depois da ação.
- **MASTER:** só um deck fica master; o deck não master segue o andamento do master
  (130,00 → 130,14 BPM, `+0,1 %`).
- **AU/MA:** o modo muda a caixa de loop entre um tamanho por batidas (AU) e
  `IN`/`OUT`/`EXIT` (MA). A troca não chama o motor.
- **MEMORY:** a linha de memory cue nova aparece na lista da esquerda (00:51.718); o
  motor não é chamado.

## Classificação

- **OBSERVADO:** o slot acionado por SLIP, MT, BEAT SYNC, KEY SYNC, Q, `IN` do modo
  manual e caixa de loop; os argumentos de `reqAutoBeatLoop` idênticos com Q ligado e
  desligado; KEY SYNC alterando a tonalidade do deck; o deck não master assumindo o
  andamento do master; MEMORY sem chamada de motor; AU/MA trocando a caixa de loop sem
  chamada de motor.
- **INFERIDO:** o Q muda o objeto de comportamento (`reqSetBehavior` com o id `0x68` nos
  dois sentidos), e a quantização do início do loop ou do cue acontece **dentro** desse
  objeto — o argumento da chamada de loop não muda. O `reqHotCueSamplerStop(0, -1, 0)`
  no SLIP é efeito colateral da troca de estado, não a definição do SLIP.
- **DESCONHECIDO:** regra de arredondamento da quantização (nenhuma posição foi lida com
  Q ligado); semântica interna do SLIP (se o deck continua em sombra e volta ao soltar);
  efeito do MT sobre a tonalidade sob ajuste de andamento (o fader não foi movido);
  o que o BEAT SYNC dispara quando o deck já é master (nada mudou na tela e só `unit#22`
  foi chamado); escrita do MEMORY no banco; ordem e o que o `OUT`/`EXIT` do modo manual
  chamam; limite de tamanho de loop.

## Ganchos para a próxima rodada

O caminho mais curto para a quantização é estático, não por clique: o comportamento
quantizado e o normal são objetos distintos e o par de funções equivalentes já está
localizado.

| Papel | Comportamento normal `cueBeh` @ `0x1453ece38` | Comportamento quantizado `qCueBeh` @ `0x1454050b8` |
| --- | --- | --- |
| slot 10 | `0x142ab1780` | `0x142c7dbf0` |
| slot 11 | `0x142ab0740` | `0x142c7d6f0` |

Comparar `0x142c7d6f0` com `0x142ab0740` deve dar a regra de arredondamento; o motor do
VILELABOX recebe o tamanho do loop em batidas e precisa da mesma conversão.

## Comparação estática: comportamento normal contra quantizado

Mesmíssima funcionalidade em duas classes: o objeto quantizado (`qCueBeh`) e o normal
(`cueBeh`) implementam os mesmos slots da vtable. Desmontagem com Capstone 5.0.7 sobre
`rekordbox.exe` (VA → deslocamento de arquivo pelas seções PE), sem depender do Ghidra.

| | Normal | Quantizado |
| --- | --- | --- |
| vtable | `0x1453ece38` | `0x1454050b8` |
| slot 11 (gravar cue) | `0x142ab0740` | `0x142c7d6f0` |
| slot 10 (lançar cue) | `0x142ab1780` | `0x142c7dbf0` |

O achado que fecha a questão de onde mora a quantização: a versão quantizada
**termina chamando a normal**. Em `0x142c7da9d` há `call 0x142ab0740` com os mesmos
argumentos (`rcx` = objeto, `rdx`, `r8d` = índice, mais três argumentos de pilha).
Ou seja, o caminho quantizado é um prefixo: se as precondições dele não valem, ele
delega ao comportamento normal inteiro. Não são duas implementações paralelas.

Precondições do prefixo quantizado (`0x142c7d6f0`): `r9b` (flag) igual a zero, o
ponteiro devolvido por `0x140fbc8e0` não nulo, e `r8d` (índice) ≤ 0. Nesse caminho ele
faz, em ordem:

1. `0x142c275f0(rdx+0x98, índice)` — busca a entrada da grade pelo índice, com
   verificação de limites (devolve uma entrada padrão fora da faixa).
2. `0x142c80840(this)` — devolve um `double`. A implementação lê o campo empacotado em
   `[grade+0x12]` no **mesmo formato do tamanho de loop**: numerador nos 16 bits baixos,
   denominador nos altos (é o mesmo `0x10010` = 16/1 já registrado em
   `p6-1-loops-cues.md`). Ela monta `num / den` em ponto flutuante e subtrai a posição
   lida do player.
3. chamada virtual pelo slot `+0x98` do objeto pego em `[this+0x50]`, com
   `edx = [player+0x14]`, `r8d = [player+0x12]` e o `double` do passo 2 como `xmm3`;
   devolve um índice inteiro.
4. limite do índice: `eax - 2`, negativo vira `0`, e `ebx = min(eax, ebx)` — o índice
   fica em `[0, n-2]`, coerente com escolher um **intervalo entre duas batidas** da grade,
   não uma amostra.
5. `0x140fbccc0` grava a escolha na entrada do cue.

Consequência para o produto: com Quantize ligado, o cue não é gravado em amostras
arredondadas, e sim **referenciado a um índice de batida da grade**. É por isso que
`reqAutoBeatLoop` sai idêntico nas duas direções do Q: o Q troca o objeto de
comportamento, e a conversão para amostras acontece depois, na leitura da grade.

Continua **DESCONHECIDO** só o arredondamento dentro do slot `+0x98` (mais próximo,
para baixo, para cima). A receita para medir: gancho em `0x142c7d8cb` (`mov rcx, rdi`,
imediatamente antes do `call rbx`), lendo `rbx` (endereço real da função), `rdi`, `edx`,
`r8d` e `xmm3` — o par entrada/saída em várias posições dá a regra sem decompilar o helper.

Tentativa de 2026-10-25 que **não** produziu medida, registrada para não ser repetida:
- `Interceptor` não consegue instrumentar o próprio `call rbx` em `0x142c7d8ce`
  ("unable to intercept function"); o gancho precisa ir na instrução anterior,
  `0x142c7d8cb`, onde `rbx`, `rdi`, `edx`, `r8d` e `xmm3` já estão carregados.
- Com esse gancho ativo, gravar hot cue pela **lista** A..H do deck (linhas vazias)
  não disparou `0x142c7d6f0` nem o caminho quantizado: a lista não é o caminho dos pads.
  A captura exige os **pads** visíveis, que esta disposição de tela não mostra —
  é preciso abrir o painel de pads ou trocar de disposição antes da rodada.

### Contradição aberta com o dossiê p6-1

`p6-1-loops-cues.md` registra que apertar pad vazio (com Q **desligado**, segundo o
próprio dossiê) chega a `reqHotCueRec` e daí ao **slot 11 do comportamento quantizado**.
Isso contradiz o rótulo dos dois objetos usados na comparação acima. Três explicações
possíveis, nenhuma verificada: (i) os rótulos `cueBeh`/`qCueBeh` estão trocados;
(ii) o objeto ativo não depende só do Q; (iii) o rótulo do p6-1 está errado.

Verificação iniciada em 2026-10-25, com resultado parcial e uma correção de rota: o
primeiro argumento de `reqSetBehavior` **não é** o objeto de comportamento do cue. A
leitura em execução devolveu `obj 0xffff822bfea52770` com vtable **`0x1453ed088`** — uma
terceira classe, nem `cueBeh` nem `qCueBeh`. O objeto do `unit#9` (o próprio deck)
aponta, como esperado, para `0x1454066b0`. Portanto o pedido de troca passa por um
objeto intermediário, e a pergunta "qual dos dois comportamentos está ativo" continua
aberta por outro caminho.

Caminhos que restam, em ordem de custo:

1. Chamar o motor direto, sem interface: o ponteiro do deck já é conhecido
   (`0xffff822bfe69f368` nesta execução), e `vtable[71](unit, n)` é o aperto de pad
   registrado no `p6-1`. Com ganchos em `0x142c7d6f0` e `0x142ab0740` ligados, o slot
   que disparar responde de que lado está o Q — e o mesmo disparo carrega o `xmm3` do
   gancho em `0x142c7d8cb`, que é a entrada do arredondamento.
2. Ler o ponteiro do comportamento a partir do próprio deck (campo interno), o que
   exige descobrir o deslocamento — não observado até agora.

Enquanto essa leitura não for feita, a conclusão "o quantizado é prefixo do normal"
continua válida como fato de código (a instrução `call` existe e o alvo é a função
normal), mas **qual dos dois objetos é o ativo com o Q ligado** fica em aberto.

### Chamada direta ao motor (2026-10-25): não executada

Script preparado: `tools/re/frida/p6_2_quantize_probe.py`
(SHA-256 `ba181dafa597d402141f403d6bb90499ee7b2e20f11becde332ed9ff2fe434e8`),
frida-python 17.24.3. Ele acha o deck por varredura de memória (vtable `0x1454066b0`),
gancha `0x142c7d6f0`, `0x142ab0740`, `0x142c7d8cb` e `0x142c7d8d0`, troca `0x142c80840`
por um invólucro que registra o `double` devolvido (é o `xmm3` do snap; o contexto de CPU
do Frida não expõe registradores `xmm`), alterna Q pelo slot 9 com `(unit, 0)` e
`(unit, 1)`, aperta `unit#71(unit, 1)` (com `reqHotCueRec(unit, 1, 0)` de reserva) e, com
o alvo do snap capturado, chama o snap direto em `x + 0,2 / 0,5 / 0,8` para `x = 0..3`,
sem gravar cue.

Saída real da tentativa:

```
$ python tools/re/frida/p6_2_quantize_probe.py 17796
frida.NotSupportedError: unexpected error allocating memory in target process (VirtualAllocEx returned 0x00000005)
$ tasklist /FI "PID eq 17796"
INFORMAÇÕES: nenhuma tarefa em execução correspondente aos critérios especificados.
```

`frida.get_local_device().enumerate_processes()` e `tasklist` não listam nenhum processo
`rekordbox`: o programa não estava aberto no momento da rodada. Nenhuma chamada chegou ao
motor; nada foi gravado no rekordbox.

Classificação desta rodada:
- **OBSERVADO:** nada novo do rekordbox.
- **INFERIDO:** nada novo.
- **DESCONHECIDO (continua):** qual objeto (`cueBeh` @ `0x1453ece38` ou `qCueBeh` @
  `0x1454050b8`) está ativo com Q ligado e desligado; regra de arredondamento do slot
  `+0x98`.

O que falta: rekordbox 7.2.8 aberto, faixa com grade carregada no deck 1, deck parado,
pad 1 (A) vazio — o aperto de pad grava hot cue A na faixa, então a rodada altera a
biblioteca; usar uma faixa descartável. Depois rodar o script com o pid novo. A assinatura
do snap usada na chamada direta (`int(obj, edx, r8d, xmm3, 0.0, 0)`) é cópia do que o
chamador em `0x142c7d8b1`..`0x142c7d8ce` monta e ainda não foi exercida.

### Medição em execução (rodada 2): rótulos confirmados e regra encontrada

Segunda tentativa, com o rekordbox reaberto (pid 13788). O script do probe anterior não
achou o deck pela varredura de memória; a descoberta passou a ser por gancho — um slot
chamado continuamente entrega o ponteiro do deck no primeiro argumento (`unit_achado`
via `unit#85`). Script: `probe2.py` (sha256 `c163ee911d8f960f…`, privado), com ganchos em
`0x142c7d6f0`, `0x142ab0740`, `0x142c7d8cb` e nos dois `reqSetBehavior`, mais invólucro em
`0x142c80840` para ler o `double` que vira `xmm3`.

Saída real, com deck parado e um aperto de pad por estado:

| Q | Depois de `reqSetBehavior` | Comportamento que rodou |
| --- | --- | --- |
| desligado | objeto `0x1b39984ec70` | **`0x142ab0740`** (o sem quantização) |
| ligado | objeto `0x1b39984c630` | **`0x142c7d6f0`** (o quantizado) |

**A contradição com o `p6-1` se resolve assim: os rótulos deste dossiê estão corretos e o
rótulo do `p6-1` é que está errado.** Com Q ligado o prefixo quantizado é o que roda; a
captura registrada no `p6-1` como "Q desligado" chegou ao quantizado por outro motivo —
provavelmente porque o Q estava ligado naquele momento, e o dossiê não verificou o estado.

O alvo do snap ficou resolvido no mesmo disparo: `fn_rva = 0x142ab7130`, chamado com
`edx = 6350`, `r8d = 0x10001` (fração 1/1 no formato empacotado) e `xmm3 = 0.5`.

#### A regra de arredondamento

Desmontagem de `0x142ab7130` (Capstone, 107 instruções, deslocamento de arquivo
`0x2ab6530`). As duas saídas que devolvem índice fazem a mesma coisa:

- `0x142ab71f1`: `vroundsd xmm1, xmm1, xmm0, 1` seguido de `vcvttsd2si eax, xmm1`
- `0x142ab72a6`: `vroundsd xmm2, xmm2, xmm1, 1` seguido de `vcvttsd2si eax, xmm2`

O imediato `1` do `vroundsd` é **arredondar para menos infinito**, e o `vcvttsd2si` ainda
trunca por cima disso. Portanto: **a quantização arredonda para baixo**. O cue ou o loop
com Q ligado cai na batida da grade imediatamente **antes** (ou exatamente sobre) a
posição, nunca na seguinte.

Complemento que a mesma função mostra e vale registrar: o passo entre batidas vem da
fração empacotada (`xmm8 = num / den`, calculado de `r8w` e `r8d >> 16`), e antes de
converter há um laço que caminha a posição de batida em batida (`xmm7 ± xmm8`) até cruzar
o alvo — é o que garante cair sobre um nó da grade em vez de um múltiplo arbitrário.

Classificação: **OBSERVADO** — (i) o comportamento quantizado roda com Q ligado e o normal
com Q desligado, medido em execução; (ii) a regra é para baixo, lida nas duas saídas da
função `0x142ab7130`.

## Rastreabilidade

- Especificações a atualizar: `spec/rekordbox/30-deck.md` (transporte, MT, sync),
  `spec/rekordbox/32-loops-cues.md` (Q, AU/MA, MEMORY)
- Método: `spec/rekordbox/evidence/p6-0-transporte.md`, `p6-1-loops-cues.md`
- Scripts de captura (privados, fora do repositório): `cap_p7c.js`, `cap_p7d.js`, `cap_p7e.js`
