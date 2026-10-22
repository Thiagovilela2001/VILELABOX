# RbxOriginal: modo Performance do rekordbox no Mixxx (blackmesa)

Layout inspirado no modo Performance do rekordbox. Os ícones de `rb/` são
desenhos próprios e simples; nenhum arquivo do rekordbox é distribuído aqui.
rekordbox é marca da AlphaTheta Corporation; este projeto não tem relação com ela.

## Layout (rekordbox 6/7 Performance, 2Deck Horizontal)

```
[hex] PERFORMANCE | 2Deck Horizontal | [x]WAVE [x]MIXER ...   CPU  MASTER ▬▬ ━●━  hora  (REC) (engrenagem)
waveforms grandes, deck 1 e deck 2 (contador de compasso e SET 1 à esquerda)
DECK 1                                   |  MIXER   | DECK 2
 capa TÍTULO/artista  TOM  tempo  MASTER/BEAT SYNC   | TRIM|TRIM|
 overview + ferramentas de grid          | HIGH|HIGH|
 [1][<>][4]  CUE  (display redondo)  SLIP Q MT REV   | MID | MID|
             PLAY   BPM ±%           LOOP / BARS     | LOW | LOW|
 HOT CUE 1-8                             | CFX | CFX|
                                         |[COLOR FX]|
                                         | CUE | CUE|
                                         | fader    |
                                         | fone MIX LEVEL |
                                         | crossfader     |
BROWSER (árvore + lista)
```

- O display redondo mostra BPM, tempo % e faixa (±). O anel cinza enche
  conforme a faixa toca e a marca vermelha mostra a posição.
- A caixa do número do deck tem `< >`, o beat jump (com o botão direito,
  divide ou dobra o tamanho), e embaixo o tamanho do salto.
- MASTER é o sync leader do Mixxx e BEAT SYNC é o sync. MT é o key lock
  (master tempo).
- CFX é o knob de QuickEffect do canal (equivalente ao COLOR FX).
- As caixas de seleção da barra de cima mostram e escondem os painéis
  (MIXER inclusive). O estado é guardado em `[RbxOriginal]`, separado da
  RbxFlowBars.

## Mixer

Desenhado a partir do mixer que aparece nas capturas de tela do próprio
instalador do rekordbox (`tutorial/12_17.png`, `tutorial/DDJ200/mix_10.png`):
dois canais estreitos lado a lado, sem faixa master no meio.

- Knob sem corpo: só o anel cinza aberto embaixo (o arco de valor fica azul)
  e um traço branco; o nome fica embaixo do knob.
- HIGH, MID e LOW são botões: clique corta a banda (kill), como no rekordbox
  (borda azul com um traço diagonal quando cortada). O botão direito no knob
  também corta.
- COLOR FX: um seletor para os dois canais, entre os knobs CFX e os CUE.
- CUE (fone) fica azul quando ligado.
- Fader sem capa: trilho fino, linha azul do fundo até o nível e um traço como
  alça; ele ocupa toda a altura que sobra no deck. Ao lado, um medidor fino.
- Embaixo: fone (MIX, LEVEL, SPLIT) e o crossfader fino com três marcas.
- O volume geral (MASTER) fica na barra superior, com medidor L/R e slider,
  como no rekordbox.
- Saíram da skin, porque o mixer do rekordbox não os mostra: BAL (balanço),
  BOOTH e o seletor de crossfader A/THRU/B. Os controles continuam no Mixxx
  (`[Master],balance`, `[Master],booth_gain`, `[ChannelN],orientation`) para
  controladoras e outras skins.
- Os desenhos novos (knob, botões de kill, alças, crossfader, ícone do fone,
  medidores em PNG) são gerados por `tools/make_rb.py`.
- Os controles de compasso (BARS, SET 1, BAR LOOP, contador) só funcionam no
  build blackmesa (`run-mixxx.sh`).

## Arquivos

- `rb/`: ícones próprios (CUE, PLAY, PAUSE, REC, engrenagem, logo...) com os mesmos nomes e tamanhos usados pelo XML
- `tools/make_rb.py` (rodar depois de `make_assets.py`/`make_look.py`, que sobrescrevem os knobs): desenha os knobs, os caps e trilhos de fader e o anel do
  display no estilo rekordbox (o instalador não traz essas peças)
- `style.qss`: paleta tirada das SVGs do rekordbox (#1e1e1e, #282828,
  azul #1473eb, CUE #ffb937, PLAY #2dd728)
- `tools/shoot_nested.sh out.png 1920 1080 [faixaA faixaB]`: captura de tela
  sem abrir janela no desktop

Tamanho mínimo da janela: 1600x860 (feita para o monitor de 1920x1080).

## Voltar para a RbxFlowBars

Escolha em Preferências > Interface > Skin, ou restaure o backup
`~/.mixxx/mixxx.cfg.bak-rbxoriginal-*`.

## Controles de performance adicionados

Cada deck tem abas HOT CUE, BEAT LOOP (1–128 batidas), BEAT JUMP (±1, ±2, ±4, ±8) e SAMPLER. Botão direito nos pads BEAT LOOP faz roll enquanto pressionado.

SAMPLER mostra quatro slots compartilhados entre os decks. Arraste uma faixa da biblioteca sobre o título do slot para carregar; ▶ dispara a partir do cue, STOP volta ao cue e para, LOOP alterna repetição.

Os seletores de RANGE definem ±6%, ±10%, ±16% ou ±50%; o disco continua mostrando a faixa atual.

FX na barra superior mostra as duas unidades numa faixa acima dos decks. Cada unidade tem seletor do primeiro efeito, ON (ativa o efeito e a unidade), destinos 1/2, MIX e META. Os destinos são independentes: pode enviar uma unidade para um ou ambos os canais. O estado existente de roteamento é preservado. Outros slots já carregados na unidade continuam conforme a configuração existente.

Os seletores usam efeitos nativos do Mixxx. Os arquivos extraídos do rekordbox incluem imagens, mapeamentos e samples de MERGE FX, mas não fornecem plugins de efeitos compatíveis com o Mixxx.

## COLOR FX compacto

Um único seletor central substitui a grade de seis botões. A seleção aplica o mesmo preset aos canais 1/2, e cada knob CFX continua independente. O seletor usa os nomes dos presets, sem depender de posições na lista. O build Blackmesa adiciona TargetEffectUnitGroup ao widget nativo EffectChainPresetSelector para permitir essa seleção compartilhada. O seletor fica no mixer, entre os knobs CFX e os botões CUE.
