# G0 — KEY SYNC entre tons distantes: confirmação em nível de motor; par próximo não reproduzido

Rodada de continuação do dossiê `p6-8`. Objetivo: fechar a regra do KEY SYNC
testando a condição nunca variada antes — dois decks com velocidade 0,0 % — e,
se possível, repetir um par de tons a 1–2 semitons (a condição do único caso
positivo, `p6-2`). Resultado: a condição de velocidade zero **já valia** para o
deck 2 no estado herdado da sessão, e o clique em KEY SYNC nesse par distante
foi confirmado em nível de motor (chamada capturada, não só leitura de tela),
reforçando `p6-7`/`p6-8` com uma fonte de evidência diferente. A reprodução de
um par de tons próximos **não foi alcançada** nesta rodada; as tentativas e por
que elas pararam estão registradas abaixo.

## Artefato e ambiente

| Campo | Valor |
| --- | --- |
| Programa | `rekordbox.exe` 7.2.8.0 (SHA-256 `85e3a58524c5cd1e870ad59e23cc8806523175f80fb82d6059c68d57bdf9991a`), pid 15504 |
| Faixas | deck 1 `DJ Chap - 1luv` / `Footwork Frenzy` (163,00 BPM, `A`, em +5,5 %); deck 2 `BOMBABOMBABOMBA` / `fr012en` (172,00 BPM, `F`, em 0,0 %) — estado herdado de `p6-8`, confirmado inalterado no início e no fim desta rodada |
| Instrumentação | `hooks_p6.js` (sha256 `eaa3025567f17f2129c97de3b11be5fc771cb605ebe9e1e1dda7130a2d958bd8`), 134 ganchos, mesmo script das rodadas `p6-6`–`p6-8` |
| Evidência REA | `ev_a2397288fc083b5e3012b76f2d532dbc07ba52083fd697122937dd1e31c9b20d` (instrumentação, 134 ganchos); `ev_a9364704e3791c2f4444c63cb2727eb4fda791a512321758159d42ac4a97a5a1` (sequência com a chamada capturada) |
| Automação | UIA por PowerShell (`rodada.ps1`, cliques por `InvokePattern`), mesma ferramenta das rodadas anteriores |

## Sequência

| Ação | Horário | Chamada capturada | Tela, deck 1 |
| --- | --- | --- | --- |
| estado inicial | — | — | `A`/`Bb +1`, 172,00, `5.5 % ±10`; deck 2 `F`/`F ±0`, 172,00, `0.00 ±6` |
| KEY SYNC deck 1 ligar | 193754 (clique) | nenhuma capturada (ver "Lacuna de captura" abaixo) | `Bb +1`, sem mudança (confirmado por leitura posterior) |
| KEY SYNC deck 1 desligar (restaurar) | 241449 (clique) | `unit#45(0x7ff606feab50, 0x1, 0x2faabfb46e0)` em t=241447 | `Bb +1`, sem mudança |
| estado final | — | — | idêntico ao inicial nos dois decks |

`0x2faabfb46e0` é o ponteiro do deck 1 (terceiro argumento), igual ao registrado
em `p6-8` para a mesma execução — confirma que a chamada partiu do deck 1 e que
o endereço do deck não mudou entre as duas rodadas desta sessão. O argumento do
meio é `0x1` tanto para ligar quanto para desligar, como já registrado em
`p6-2`/`p6-8` para o Q e o próprio KEY SYNC.

### Lacuna de captura no primeiro clique

O primeiro clique (ligar) não aparece nas mensagens do script, só o segundo
(desligar). A leitura de tela logo após o primeiro clique também veio em
branco — rastreado a uma troca de foco acidental para a Alternância de Tarefas
do Windows (`Alt+Tab`) no mesmo intervalo, provavelmente efeito colateral de
uma tecla do automatizador anterior na sessão. Depois de restaurar o foco, a
leitura confirmou que o clique tinha funcionado (o indicador do KEY SYNC
mudou de desligado para ligado). Duas explicações restam, nenhuma verificada:
(i) a mensagem do primeiro clique foi perdida por uma corrida entre o clique e
a primeira consulta de status (consultada sem esperar o intervalo de 1 s do
script); (ii) ligar e desligar não chamam `unit#45` de forma simétrica. A
rodada `p6-8` já tinha capturado `unit#45(1)` tanto ao ligar quanto ao
desligar, o que favorece a explicação (i). Não é um achado novo sobre o KEY
SYNC — é uma lacuna de método, registrada para não ser mal lida como
assimetria real.

## Leitura

- O deck 1 (`A`, `+5,5 %`) e o deck 2 (`F`, `0,0 %`) seguem sendo um par de
  tons a 4 semitons (igual a `p6-8`). O deck 2 já estava com velocidade
  exatamente zero (herdado da sessão), cumprindo pela primeira vez a condição
  "um dos decks em 0,0 %" sem precisar de nenhuma ação nova — e mesmo assim o
  KEY SYNC não deslocou o tom.
- A chamada ao motor (`unit#45`) foi confirmada em memória, não só pela tela:
  o clique chega ao mesmo slot já mapeado em `p6-2`/`p6-8`, com o ponteiro do
  deck 1 como terceiro argumento, e a tela não mudou. Isso fecha, com uma
  segunda fonte de evidência, que a ausência de efeito nos pares distantes
  **não é um problema de leitura de tela** (ex.: arredondamento escondendo um
  deslocamento pequeno) — a chamada acontece e não produz deslocamento visível
  nenhum.

## Tentativas de reproduzir o par próximo (não concluídas)

A condição que falta — dois decks a 1–2 semitons de distância, ambos a 0,0 %
— não foi alcançada nesta rodada. Registrado para a próxima tentativa não
repetir os mesmos becos sem saída:

1. **Painel de pré-preparação (sub-browse).** Alternar `ShowHideSubBrowseButton`
   (50,611) revela quatro grades "blank list" com coluna `Tonalidade`, mas
   vazias de faixas nesta sessão — não é a biblioteca completa, é uma lista de
   preparação por deck, sem conteúdo carregado. Painel fechado de novo ao
   final, estado restaurado.
2. **Biblioteca completa.** A tabela "All Tracks" (3089 faixas) só fica
   visível com o layout padrão da tela de performance expandido — uma
   captura de tela (não a árvore de UIA) foi necessária para localizá-la,
   porque a árvore de UIA não expôs linhas da grade com `DataItem`/`ListItem`
   (a grade de faixas não publica células como texto individual). A coluna
   `Tonalidade` existe no cabeçalho, mas nas primeiras linhas (ordenação
   padrão, por data de adição) ela aparece em branco — as faixas mais
   recentes não têm tom analisado/marcado.
3. **Ordenar por `Tonalidade`.** Clicar no cabeçalho da coluna por coordenada
   de tela (sem elemento de UIA clicável conhecido para `Header`) não é
   confiável: duas tentativas desviaram o clique — uma abriu o relógio da
   barra de tarefas do Windows, outra abriu a capa do álbum em tela cheia e,
   em algum ponto da sequência, a janela do rekordbox foi minimizada. Nenhuma
   biblioteca foi alterada (nenhuma faixa apagada, renomeada ou movida); a
   janela foi restaurada e o estado dos decks conferido idêntico ao inicial.
   Dado o risco de um clique mal calculado mexer em algo que não é só
   visualização, a ordenação por coluna foi abandonada como caminho nesta
   rodada.
4. **Zerar a velocidade do deck 1 por outra via.** Não há um controle
   `Slider` com `RangeValuePattern` de escrita para o fader de andamento por
   deck — o único `Slider` perto da área esperada (900,569) é o crossfader
   central (confirmado por valor e posição), não o fader de cada deck. O
   campo que mostra o BPM central (`725,448`, tipo `Edit`) aceitou foco e
   texto digitado (`163.00` + Enter) sem efeito nenhum na tela — é somente
   leitura pela via tentada. O ajuste fino por `−`/`+` (em torno de `738,496`
   e `764,496`) não foi tentado: exigiria da ordem de 55 cliques de passo
   desconhecido para ir de `+5,5 %` a `0,0 %`, sem confirmação prévia do
   tamanho do passo.

## O que a próxima rodada precisa

1. Uma via confiável para achar duas faixas com `Tonalidade` a 1–2 semitons
   de distância sem depender de ordenar a grade por clique de tela — por
   exemplo, ler a tabela de faixas fora do rekordbox (arquivo de biblioteca)
   para escolher o par antes de abrir o app, ou usar a busca por texto já
   visível no canto da tela de biblioteca (não tentada nesta rodada).
2. Confirmar o tamanho do passo dos botões `−`/`+` de andamento (ou achar o
   controle real do fader por deck) antes de tentar zerar a velocidade do
   deck 1 por clique.
3. Ao consultar `frida_session_status` logo após um clique, esperar ao menos
   a janela do intervalo do script (1 s) antes de concluir que não houve
   chamada — a rodada mostrou uma consulta imediata retornando vazia para uma
   chamada que existia.

## Classificação

- **OBSERVADO:** `unit#45` é chamado com o ponteiro do deck que recebeu o
  clique e argumento `0x1` ao desligar o KEY SYNC entre `A` e `F` (−4
  semitons), com o deck 2 a 0,0 % de velocidade; a tela não mudou. Reforça
  `p6-7` e `p6-8` com leitura de memória em vez de só tela.
- **INFERIDO:** nenhum item novo sobre a regra do KEY SYNC.
- **DESCONHECIDO (continua, sem mudança de classe):** a regra do deslocamento
  do KEY SYNC. A condição do único caso positivo (`p6-2`: dois tons a 1–2
  semitons, ambos a velocidade 0,0 %) segue sem reprodução. Esta rodada só
  descarta, com mais confiança, a hipótese de que a leitura de tela escondia
  um deslocamento real em pares distantes — a chamada ao motor não produz
  deslocamento nenhum nesses casos, verificado em memória.

## Rastreabilidade

- Especificação: `spec/rekordbox/30-deck.md` (KEY SYNC, perguntas abertas).
- Código: `vilelabox-app/src/sync.rs` (`key_sync_shift`) — **sem alteração**
  nesta rodada; a regra inferida de `p6-2` permanece como estava, por não
  haver evidência nova que a confirme ou substitua.
- Dossiês anteriores: `p6-2-sync-slip-quantize.md` (caso positivo único),
  `p6-7-desligar-sync.md`, `p6-8-mt-tom-exibido-keysync.md` (dez tentativas
  em pares distantes, mesma conclusão por leitura de tela).
