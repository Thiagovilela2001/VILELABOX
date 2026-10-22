# Plano de layout — skin RbxNative

Objetivo: uma skin do VILELABOX com o layout do modo Performance do rekordbox
7 e a arte original do instalador, sem colocar nenhum arquivo da AlphaTheta no
repositório público.

A RbxNative é uma skin do Mixxx e será descartada na migração para Rust
([PLANO-RUST.md](PLANO-RUST.md), Fase 7). O que fica são as medidas: a tabela de
regiões, a paleta e o mapa de arte das fases 2 e 3 são a especificação de tela
do R7.0. Por isso essas saídas não usam nada específico do XML do Mixxx.

Base: inventário local em
[third_party/rekordbox/REKORDBOX-ASSETS.md](third_party/rekordbox/REKORDBOX-ASSETS.md)
(não versionado) e a skin [RbxOriginal](res/skins/RbxOriginal/README.md), que
já imita esse layout com arte própria.

## Regras deste plano

- Arte, XML de layout e modelos do rekordbox só em `third_party/rekordbox/`
  (no `.gitignore`) e no perfil local. Nunca em `res/skins/`. O hook `pre-push`
  e `tools/check-third-party-assets.sh` continuam ativos como trava.
- `RbxOriginal` e `RbxFlowBars` não mudam. A RbxNative é uma skin nova.
- Sem commit, push ou PR sem pedido do usuário ([CLAUDE.md](CLAUDE.md)).
- Cada fase termina com o critério de pronto cumprido ou com o bloqueio
  registrado em [IMPLEMENTACAO.md](IMPLEMENTACAO.md), seção "Plano de layout".

## Fatos que limitam o plano

| Fato | Evidência | Consequência |
| --- | --- | --- |
| A tela principal do rekordbox é JUCE; posições e tamanhos estão compilados no `rekordbox.exe` | 27 ocorrências de `JUCE`, nenhum Qt/QML; 962 nomes de imagem citados no binário (REKORDBOX-ASSETS.md) | O layout vem de capturas de tela medidas, não de arquivo |
| Os 5 XML de `skins/` do rekordbox cobrem só os modos reduzidos e o PRO DJ LINK | REKORDBOX-ASSETS.md, "Layouts XML" | Servem só de referência para um modo "simple", fora deste plano |
| O instalador não traz knobs, caps de fader, anel do display nem os botões grandes CUE/PLAY como imagem | [res/skins/RbxOriginal/README.md](res/skins/RbxOriginal/README.md) (seção Arquivos) | Esses elementos continuam desenhados por `tools/make_rb.py` |
| O Mixxx lê skins também de `<perfil>/skins` | [src/skin/skinloader.cpp:56-60](src/skin/skinloader.cpp#L56-L60) | A RbxNative é instalada em `~/.mixxx/skins/RbxNative`, fora do repositório |
| O rekordbox está no Windows; o build do mod só existe no Linux | [CLAUDE.md](CLAUDE.md) | A arte precisa ser copiada para a máquina Linux (fase 4) |
| A janela de preferências é Qt nativa | [src/preferences/dialog/dlgpreferences.cpp](src/preferences/dialog/dlgpreferences.cpp) | Os ícones `preferece_*` do rekordbox ficam fora do escopo: a skin não troca os ícones desse diálogo |

## Fase 0 — Decisões do usuário

Parar e perguntar antes de seguir:

1. **Layout alvo:** 2Deck Horizontal (recomendado: é o da RbxOriginal),
   2Deck Vertical ou 4Deck.
2. **Resolução alvo:** 1920×1080 (recomendado: mínimo atual da RbxOriginal é
   1600×860) ou outra.
3. **Onde fica o XML da RbxNative:** (a) tudo local, em
   `third_party/rekordbox/skins/RbxNative/` (recomendado: a skin não aparece
   quebrada para quem clona o repositório); (b) XML versionado em
   `res/skins/RbxNative/` e só a arte local, o que deixa a skin listada e sem
   imagens para os outros.

**Pronto quando:** as três respostas estão registradas em IMPLEMENTACAO.md.

## Fase 1 — Captura de referência

O usuário abre o rekordbox 7.2.8 no modo Performance, no layout escolhido, em
tela cheia na resolução alvo, com uma faixa carregada em cada deck.

Capturas em `third_party/rekordbox/ref/`:

| Arquivo | Estado |
| --- | --- |
| `01-completo.png` | Todos os painéis visíveis (WAVE, MIXER, PADS, GRID, TEMPO, BROWSE) |
| `02-sem-browser.png` | BROWSE desligado (waveforms e decks maiores) |
| `03-pads-hotcue.png`, `04-pads-beatloop.png`, `05-pads-sampler.png` | Cada aba de pads |
| `06-grid.png` | Ferramentas de grid abertas |
| `07-fx.png` | Painel de FX aberto |
| `08-tocando.png` | Deck 1 tocando, com loop ativo e sync ligado (estados acesos) |

**Pronto quando:** as capturas existem na resolução alvo e cada uma tem uma
linha em `third_party/rekordbox/ref/README.md` (estado, data, versão).

## Fase 2 — Medição das regiões

A partir de `01-completo.png`, montar a tabela de regiões em
`third_party/rekordbox/ref/regioes.md`, em pixels da resolução alvo:

| Região | x | y | largura | altura | Arquivo da RbxOriginal que corresponde |
| --- | --- | --- | --- | --- | --- |
| Barra global (modo, layout, painéis, CPU, hora, REC, engrenagem) | | | | | `topbar.xml` |
| Waveforms ampliadas, decks 1 e 2 | | | | | `waveforms_container.xml` |
| Deck 1 / Deck 2 (capa, título, tom, tempo, overview, transporte, BPM) | | | | | `decks/` |
| Mixer (TRIM, EQ, COLOR FX, faders, crossfader, medidores) | | | | | `mixer/` |
| Pads (abas e 8 pads) | | | | | `decks/` (painel PADS) |
| FX | | | | | `fx/` |
| Browser (árvore e lista) | | | | | `library.xml` |

Medir também: margens entre regiões, altura das fileiras de botões, tamanho dos
pads, tamanho das fontes (comparar com a régua da própria captura) e a paleta
real (amostras de cor da captura, para conferir `style.qss`).

**Pronto quando:** toda região da tabela tem coordenadas e arquivo correspondente;
a soma das alturas fecha com a altura da janela (±2 px).

## Fase 3 — Mapa de arte

Criar `third_party/rekordbox/skins/arte.csv` com uma linha por imagem usada:
`arquivo_rekordbox,arquivo_rbxnative,regiao,estado` (estado: `off`, `on`,
`hover`, `disabled`).

Grupos já identificados no inventário (nomes da pasta `skins/` do rekordbox):

| Região | Arte do rekordbox | Observação |
| --- | --- | --- |
| Grid | `Grid_Button_*.svg`, `Grid_Export_Button_*.svg`, `Grid_BackGroundOfBpm.svg` | Mapear para os botões de grid já existentes (SET 1, mover, expandir, tap, metrônomo) |
| Sync | `beatSync_icon_on/off.svg`, `bpmSync_icon_on/off.svg` | BEAT SYNC e MASTER |
| Loop | `Loop_2DeckHorizontal.svg`, `LoopOn_2DeckHorizontal.svg`, `LengthRoll_*.svg`, `loopMinusP.svg`, `loopPlusP.svg` | Aba BEAT LOOP e controles de loop do deck |
| Beat jump | `beatjump_left_*.svg` e equivalentes à direita | Caixa `< >` do deck |
| Sampler | `samplerSlot_btn*_on/off.svg`, `sampler.svg`, `samplerLoop.svg` | Aba SAMPLER (4 slots) |
| Mixer | `eqHigh/Mid/LowOn/Off.svg`, `knob.svg`, `p4352..p4354_chfader*.svg`, `p4355..p4356_crossfader*.svg` | Knobs continuam de `make_rb.py` se `knob.svg` não servir para o widget de rotação do Mixxx |
| Active Censor | `activeCenserOn/Off.svg` | Só se o recurso for implementado; sem isso, fica fora |
| Browser | `brows_*.svg`, `browsTreeMore_*.svg` | Só o que a biblioteca do Mixxx consegue usar via `style.qss` (ícones de árvore, busca, filtro) |
| Topo | `logo.svg`, `p4401_recbtn_*.png`, `p4052_setting_1nrm.png`, `abletonLink*.svg` | REC, engrenagem, Link |

Para cada elemento da RbxOriginal sem arte equivalente no rekordbox, registrar
`sem equivalente` e manter a arte própria de `rb/`.

**Pronto quando:** toda imagem da RbxOriginal (94 arquivos SVG/PNG fora de
`docs/` em 2026-10-22) aparece no CSV como mapeada ou `sem equivalente`. As
referências no XML usam variáveis de template, então a conferência é pelos
arquivos, não por grep no XML:

```bash
find res/skins/RbxOriginal \( -name '*.svg' -o -name '*.png' \) -not -path '*/docs/*' | wc -l
```

## Fase 4 — Montagem e instalação

1. Copiar `res/skins/RbxOriginal/` para `third_party/rekordbox/skins/RbxNative/`
   e trocar o `<title>` em `skin.xml` e o grupo de estado `[RbxOriginal]` por
   `[RbxNative]`, para não misturar as preferências salvas das duas skins.
2. Script local `third_party/rekordbox/install_rbxnative.py`, só biblioteca
   padrão:
   - entrada: pasta do rekordbox instalado, `arte.csv` e o perfil de destino;
   - copia a arte do instalador para `RbxNative/rb-native/`, conforme o CSV;
   - troca os caminhos no XML e no QSS pelos da arte copiada;
   - instala em `<perfil>/skins/RbxNative/`;
   - recusa gravar dentro do repositório fora de `third_party/rekordbox/`.
3. Levar para a máquina Linux: copiar `third_party/rekordbox/` (por exemplo,
   com `rsync`) e rodar o script com o perfil de teste usado por `run-bars.sh`,
   nunca o `~/.mixxx` principal sem pedido do usuário.

**Pronto quando:** `git status --short` não mostra nenhum arquivo novo fora de
`third_party/rekordbox/` e a RbxNative aparece em Preferências → Interface → Skin
no perfil de teste.

## Fase 5 — Geometria

Ajustar as posições e tamanhos da RbxNative pela tabela da fase 2, região por
região, na ordem: barra global, waveforms, decks, mixer, pads, browser, FX.

- Uma região por vez; depois de cada uma, captura com
  `res/skins/RbxOriginal/tools/shoot_nested.sh` na resolução alvo e comparação
  lado a lado com `01-completo.png`.
- Manter funcionando no tamanho mínimo da janela (definir na fase 0 se continua
  1600×860).
- Painéis ligados e desligados pela barra global devem reproduzir
  `02-sem-browser.png` e as capturas de pads.

**Pronto quando:** em cada região, a diferença de posição e tamanho contra a
captura é de no máximo 4 px na resolução alvo, medida nas capturas.

## Fase 6 — Recursos sem equivalente

Elementos do rekordbox que o VILELABOX não tem ([PADS.md](PADS.md),
REKORDBOX-ASSETS.md): PAD FX 1/2, ACTIVE CENSOR, SEQUENCE CALL, TRANSPORT,
SP.SCRATCH, STEMS (adiado), pads USER e Combination.

Na skin, esses espaços ficam **vazios ou escondidos**, nunca com botões que não
fazem nada. Cada recurso que o usuário quiser vira um plano próprio de código,
com teste gtest, fora deste plano de layout.

**Pronto quando:** nenhum botão visível da RbxNative está sem controle ligado
(conferir os `<ConfigKey>` de cada widget novo).

## Fase 7 — Verificação

| Verificação | Comando ou método | Onde roda |
| --- | --- | --- |
| Imagens existem e SVGs no tamanho certo | `python3 <perfil>/skins/RbxNative/tools/check_refs.py` (cópia do verificador da RbxOriginal, que `tools/verify-mod.sh:74` roda para as skins versionadas) | Linux |
| XML da skin carrega | abrir com `run-bars.sh` no perfil de teste e conferir o log do Mixxx sem erros de skin | Linux |
| Nada do rekordbox no git | `tools/check-third-party-assets.sh "<dir rekordbox>" 3ebac449e7..HEAD` e `git status` | Linux ou Windows |
| Comparação visual | capturas da fase 5 lado a lado com as da fase 1 | Linux |
| Uso real | tocar, sincronizar, carregar pads, mexer no mixer e no browser por uma sessão | Desktop do usuário |

**Pronto quando:** as quatro primeiras passam e o uso real fica registrado em
[VALIDACAO.md](VALIDACAO.md). Até lá, a RbxNative é experimental.

## Fora do escopo

- Modos simple player / simple mixer e o painel PRO DJ LINK.
- Iluminação (`lighting/`, 837 imagens), serviços de streaming e nuvem
  (`spotify/`, `tidal/`, `beatport/`, `soundcloud/`, `CloudDirect*`).
- Ícones da janela de preferências.
- Desmontar o `rekordbox.exe` atrás de coordenadas: custo alto, e as capturas
  da fase 1 dão o mesmo resultado.
