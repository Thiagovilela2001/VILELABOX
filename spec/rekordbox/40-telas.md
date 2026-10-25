# 40 — Telas

Especificação da interface do Núcleo. Revisão v2: estrutura derivada do manual oficial
do rekordbox 7 e da visão geral oficial da interface; geometria medida na UI em execução
em 2026-10-24, registrada em [`evidence/p3-1-ui-medido.md`](evidence/p3-1-ui-medido.md).
O que ainda não foi medido continua marcado.

Fontes:

- manual oficial do rekordbox 7 (PDF publicado pela AlphaTheta);
- página oficial de visão geral da interface do rekordbox 7;
- comportamento observável descrito na documentação de usuário dos painéis;
- medição da UI em execução nesta máquina (2026-10-24): dossiê de proveniência
  [`evidence/p3-1-ui-medido.md`](evidence/p3-1-ui-medido.md);
- dossiê de proveniência G0: [`evidence/p0-1-interface.md`](evidence/p0-1-interface.md).

## Escopo e fora do escopo

**Escopo.** Estrutura da janela principal, regiões, painéis, controles, estados
visuais, hierarquia de informação e regras de reorganização. Serve para separar a
apresentação das funções de biblioteca, análise e áudio.

**Fora do escopo.** Coordenadas em pixels, famílias tipográficas, valores hexadecimais
de cor e ícones de uma versão específica. Nenhum desses itens foi medido. Também
ficam fora: licença, conta, nuvem, streaming, telemetria e atualização.

## Glossário

- **Modo** é a área de trabalho ativa da janela: exportação ou performance.
- **Região** é um dos blocos maiores da janela: barra global, reprodução, mixagem,
  navegador e barra de estado.
- **Deck** é o conjunto de identificação, transporte, formas de onda e pads de uma
  faixa carregada.
- **Canal** é a faixa de mixer associada a um deck.
- **Painel** é uma área auxiliar que pode ser mostrada ou oculta.
- **Marcador** é um ponto temporal assinalado: disparo rápido, memória ou loop.
- **Fonte** é uma origem de conteúdo do navegador: coleção, playlist, dispositivo.
- **Seleção** é a linha destacada no navegador; **carga** é a faixa presente no deck.

## Regiões da janela

A janela se divide em cinco regiões com papéis distintos. A divisão é conceitual: na
tela real, painéis compartilhados podem conter elementos de regiões diferentes, e
regiões podem sumir conforme o modo e os painéis ativos.

| região | papel | muda com o modo |
| --- | --- | --- |
| Barra global | modo, disposição, painéis ativos, indicadores e ferramentas da janela | não |
| Reprodução | decks: identificação, transporte, formas de onda, pads | sim, em espaço |
| Mixagem | canais, equalização, níveis, crossfader, monitoração, efeitos | sim, em espaço |
| Navegador | fontes, playlists, filtros, busca e tabela de faixas | sim, em espaço |
| Barra de estado | informação contextual, quando presente | sim, em conteúdo |

Reprodução e navegador são os dois espaços de trabalho. No modo de performance, a
mixagem e os pads ganham espaço; no modo de exportação, a preparação e a organização
da coleção passam ao centro. Medido no 7.2.8: o seletor de modo oferece exactamente
`EXPORT`, `PERFORMANCE` e `EDIT`, e não há modo de iluminação nesta instalação. O EDIT
não é o performance com menos painéis: é outro espaço de trabalho, com barra de menus
própria, barra de projecto, painel de paleta à direita e abas de ferramentas na base
([medição](evidence/p3-2-ui-outros-estados.md)).

## Barra global

Contém o que afeta a janela ou o programa inteiro.

- **Seletor de modo.** Alterna a área de trabalho. O estado ativo precisa ser
  inequívoco, e o conteúdo central da janela muda com a seleção.
- **Seletor de disposição.** Altera a composição da área de reprodução: um, dois ou
  quatro decks; orientação horizontal ou vertical; navegador ampliado. Medido no 7.2.8:
  as opções são `2Deck Horizontal`, `2Deck Vertical`, `4Deck Horizontal`,
  `4Deck Vertical` e `Browser`. Em quatro decks a coluna de mixer fica entre as duas
  colunas de decks, com um canal por deck ([medição](evidence/p3-2-ui-outros-estados.md)).
- **Seleção de painéis.** Mostra ou oculta painéis auxiliares: efeitos, sampler,
  mixer, gravação, pontos de mixagem e outros habilitados nas preferências.
- **Indicadores globais.** Nível de áudio, uso de processador, informação de programa
  e de estado, quando o modo os apresenta.

Controle e leitura precisam ser distinguíveis pela aparência e pelo comportamento: o
nível de áudio e o uso de processador são leitura; o volume principal é controle.

**Regra.** Abrir ou fechar um painel redistribui o espaço restante sem retirar da tela
os controles essenciais de reprodução. Fechar um painel não pode deslocar o conteúdo
de outro de forma que perca a correspondência entre deck e canal.

## Deck

Identificação, controle, posição e manipulação temporal de uma faixa carregada. Cada
deck tem identidade associada ao número do player e ao canal correspondente.

### Identificação

Título, artista, capa, andamento, tonalidade, tempo decorrido, tempo restante, posição
atual e indicadores de sincronização e reprodução.

Título e artista identificam o conteúdo. Andamento e tonalidade orientam a escolha de
faixas compatíveis. Os tempos informam quanto falta para um ponto de transição.

**Regra.** Sem capa, o espaço de identificação é preservado: a ausência de arte não
pode deslocar os demais elementos do deck.

**Regra.** O valor detectado e o valor ajustado por controle precisam ser distinguíveis
quando o programa oferece essa distinção.

### Transporte

Reproduzir e pausar, retornar ao ponto de referência, percorrer a faixa, saltar por
batidas em quantidade selecionável e sincronizar o andamento.

Estados que a aparência precisa comunicar: reprodução ativa, reprodução pausada,
sincronização ligada e função indisponível.

### Controles de andamento

Ajuste de velocidade, valor do andamento, faixa percentual de ajuste quando exibida,
sincronização e preservação da tonalidade percebida sob ajuste de velocidade.

### Roda de reprodução

Controle circular com superfície, anel externo, indicador de posição e área central,
apresentado conforme a disposição escolhida. Não é obrigatória em toda disposição.

## Formas de onda

São dois componentes distintos, e não a mesma apresentação em escalas diferentes.

| componente | representa | conteúdo |
| --- | --- | --- |
| Forma de onda geral | a faixa inteira | amplitude ao longo da duração |
| Forma de onda ampliada | a região em reprodução | posição, grade, marcadores, cores de análise |

A forma de onda geral serve para reconhecer regiões: introdução, quebra, refrão,
trecho denso, transição e final. A ampliada é o espaço de trabalho de precisão.

### Grade de batidas

Representa a organização rítmica estimada: linhas regulares, marcas de batida e
referências de compasso. Serve para conferir se a análise está alinhada ao áudio. A
interface precisa permitir visualizar e corrigir o alinhamento onde o programa
permite.

### Marcadores

Disparo rápido identifica pontos de acesso durante a reprodução. Memória registra
posições de referência para preparação e navegação. Loop delimita um intervalo.

Qualquer marcador tem posição temporal e identificação visual; cores distinguem
categorias e letras ou nomes identificam cada ponto.

**Regra.** Marcadores permanecem associados à posição correta depois de redimensionar
a forma de onda ou alterar a escala. A posição é do áudio, não da tela.

**Regra.** A linha de reprodução é distinguível da grade, dos marcadores e da própria
forma de onda. Loop ativo tem estado visual próprio, com limites identificáveis.

### Cores

A aparência depende da preferência de visualização. As categorias que precisam ser
distinguíveis entre si: fundo, forma de onda, posição atual, grade, disparo rápido,
loop ativo, seleção e foco, e regiões diferenciadas pela análise.

Não se deve presumir valor hexadecimal sem medir a tela. As cores acima são funções
visuais, não uma paleta extraída.

## Mixer

Reúne os controles que alteram nível e características do áudio de cada canal. A
disposição acompanha o número de decks.

### Canal

Ordem de cima para baixo, conforme a documentação do painel: ganho de entrada, faixa
alta, faixa média, faixa baixa, medidor de nível, envio para fone, fader do canal e,
quando habilitados, controles adicionais.

**Regra.** Botões de equalização indicam a posição do ajuste; faders indicam o nível;
medidores reagem ao áudio, e não ao movimento do ponteiro.

### Crossfader

Trilho horizontal com controle deslizante, posição central e limites. A contribuição
relativa dos lados depende da posição, da curva configurada e da atribuição dos
canais.

### Saída principal

Nível e medidores da mixagem. O volume principal precisa ser distinguível dos ajustes
individuais de canal.

### Monitoração

Seleção de canal para fone, nível de saída e mistura entre saída principal e
pré-escuta. A pré-escuta permite preparar uma faixa sem enviá-la à saída principal.

## Pads de performance

Grade de células associadas ao deck, uma por função. Cada célula tem função,
identificação e estado.

Modos citados: disparo rápido, efeitos de pad, salto por batidas, sampler e teclado,
conforme versão e equipamento.

**Regra.** Célula vazia é distinguível de célula configurada; célula acionada apresenta
retorno visual; a cor pode identificar a função.

## Painéis auxiliares

| painel | conteúdo |
| --- | --- |
| Efeitos | seleção de efeito, intensidade, parâmetros, ativação, origem, referência rítmica |
| Sampler | slots com nome, identificação visual, reprodução, estado, volume e repetição |
| Pontos de mixagem | pontos de entrada e saída de faixas diferentes, ordem temporal e associação |
| Gravação | origem, nível, medidor, início e fim, tempo decorrido e espaço |

**Regra.** Selecionar um efeito não é ativá-lo: o efeito carregado sem aplicação precisa
ser distinguível do efeito em uso.

**Regra.** Slot vazio é distinguível de slot ocupado, e o slot em reprodução tem estado
ativo.

**Regra.** Ponto de mixagem não é marcador de disparo rápido: um identifica posição
para acesso, o outro descreve uma combinação entre duas faixas.

**Regra.** O botão de início de gravação não pode parecer igual ao de interrupção
durante uma gravação ativa.

## Navegador

Fonte, filtro, busca e tabela. A proporção entre navegação lateral e tabela depende da
disposição e dos divisores.

### Navegação lateral

Hierarquia de fontes: coleção, playlists, pastas, histórico, preparação, dispositivos e
serviços habilitados. Pastas se expandem; o item selecionado fica destacado e determina
o conteúdo da tabela.

### Cabeçalho do navegador

Busca, filtros, controles de visualização, ordenação e acesso a recursos de análise e
organização. A busca reduz o conjunto visível; o filtro restringe por atributo sem
alterar metadados; a ordenação altera a sequência das linhas.

### Tabela

Cada linha é uma faixa; cada coluna é um metadado. Colunas citadas: capa, título,
artista, álbum, andamento, tonalidade, duração, classificação, gênero, comentários,
data de inclusão e contagem de reprodução. A ordem e a presença são configuráveis.

### Seleção e carga

**Regra.** Seleção e carga são estados distintos. Faixa selecionada não está
necessariamente tocando; faixa tocando não precisa estar selecionada. A faixa carregada
no deck e a linha destacada precisam ser distinguíveis ao mesmo tempo.

### Navegador secundário e visão em colunas

Um segundo navegador, quando a disposição permite, é independente no contexto de
seleção e coerente com a biblioteca compartilhada. A visão em colunas percorre a
hierarquia mantendo visível o caminho da seleção; não substitui a tabela.

## Diferença entre os modos

| modo | centro da experiência | espaço da biblioteca |
| --- | --- | --- |
| Exportação | organização, metadados, análise e preparação | amplo |
| Performance | decks, formas de onda, mixer, pads, efeitos e monitoração | dividido com os controles |

## Regras de reorganização

- Divisores entre reprodução e navegador, entre navegação lateral e tabela, e entre
  painéis auxiliares.
- Limites mínimos de largura e altura por região.
- Rolagem vertical da tabela e horizontal quando há colunas além da largura.
- Redimensionamento da forma de onda acompanha a região, sem perder a correspondência
  entre posição desenhada e posição de áudio.
- Ao diminuir uma região, os controles prioritários são preservados e os secundários
  podem ser ocultados ou reorganizados.
- O navegador não desaparece por completo quando a reprodução cresce; abrir um painel
  não torna os controles essenciais inacessíveis.

## Estados visuais

| estado | comportamento esperado |
| --- | --- |
| Faixa selecionada | destaque na linha |
| Faixa carregada | identificação no deck |
| Reprodução ativa | indicador e posição em movimento |
| Reprodução pausada | posição estável |
| Sincronização ligada | indicador próprio |
| Envio para fone ligado | estado de monitoração identificável |
| Loop ativo | região destacada na forma de onda |
| Marcador configurado | marca na posição correspondente |
| Pad acionado | retorno visual de ativação |
| Efeito aplicado | indicação de aplicação |
| Gravação ativa | estado visual próprio |
| Busca sem resultado | estado vazio informativo |
| Biblioteca carregando | indicador de processamento |
| Arquivo ausente | identificação de indisponibilidade |

**Regra.** Foco de teclado, seleção por ponteiro e estado de reprodução não compartilham
o mesmo indicador: são condições diferentes.

## Números e alinhamento

Andamento, tempos e níveis precisam de alinhamento consistente entre decks e canais,
para permitir comparação por leitura rápida em área densa. Números permanecem legíveis
no tamanho compacto usado nos painéis.

## O que falta para a reprodução exata

Esta especificação descreve estrutura e comportamento; não descreve geometria. Para
passar de estrutura a pixels é preciso fixar, antes de qualquer medição:

| item | estado |
| --- | --- |
| Versão de referência | 7.2.8 (a instalada) |
| Sistema e escala de tela | Windows 11, 1920×1080, DPI 120 (escala 1,25) |
| Resolução de referência | 1920×1080; área de cliente medida 1920×1051 px físicos = 1536×841 lógicos |
| Modo e disposição | performance, dois decks, horizontais |
| Painéis abertos | nenhum painel auxiliar; quatro slots de lista vazios |
| Capturas de referência | `captura.png` em `third_party/rekordbox/ui-workbench/medicoes/` (privado); números em [`evidence/p3-1-ui-medido.md`](evidence/p3-1-ui-medido.md) |

A medição de 2026-10-24 cobre geometria de regiões, painéis, colunas e faixas da tela
no estado acima. Continuam sem medição: paleta completa, famílias tipográficas, modos
de exportação e quatro decks, e a tela com faixa carregada.

## Casos de teste

| caso | entrada | saída esperada |
| --- | --- | --- |
| Disposição muda | alternar de dois para quatro decks | quatro decks, um canal por deck, nenhum controle essencial fora da tela |
| Painel auxiliar | abrir efeitos com reprodução ativa | reprodução intacta, espaço redistribuído |
| Redimensionar | arrastar o divisor da biblioteca até o mínimo | biblioteca visível no mínimo, sem sobreposição |
| Marcador sob redimensionamento | redimensionar a forma de onda ampliada | marcador continua na mesma posição musical |
| Seleção e carga | selecionar outra faixa com um deck tocando | linha destacada muda, deck não muda |
| Busca vazia | buscar termo sem correspondência | estado vazio informativo, sem linhas |

Origem dos resultados: derivados da descrição publicada do comportamento, não de
execução do programa real.

## Perguntas abertas

- A geometria vem da medição de capturas reais ou da proporção dos recursos de
  interface da instalação? (decidido: medição direta, ver
  [`evidence/p3-1-ui-medido.md`](evidence/p3-1-ui-medido.md))
- Os modos de edição e de iluminação entram no produto ou ficam fora?
- Existe tema claro nesta versão, e ele é requisito?
