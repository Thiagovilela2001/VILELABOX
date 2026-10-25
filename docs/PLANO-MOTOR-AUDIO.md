# Plano — motor de áudio (P6)

Detalha a fase P6 do [`IMPLEMENTATION-PLAN.md`](IMPLEMENTATION-PLAN.md) e o item U4
do [`PLANO-UI-FUNCOES.md`](PLANO-UI-FUNCOES.md). Objetivo: dois decks que tocam,
pausam, voltam ao CUE, saltam, fazem loop e sincronizam, com a agulha e as
formas de onda acompanhando a reprodução.

O plano registra o que ainda precisa ser recuperado. Nomes de crates, formas de
fila, formatos de PCM e ordem dos estágios abaixo são hipóteses de implementação,
não fatos sobre o programa de referência. Cada comportamento exige o Gate G0;
decisões de infraestrutura Rust ficam marcadas como `ADAPTAÇÃO`.

**Regra de implementação:** não se cria algoritmo, curva, estado, transição,
constante ou processamento de áudio do zero. Primeiro se recupera a lógica do
binário e do comportamento do programa de referência por engenharia reversa,
incluindo decompilação no Ghidra. Depois se traduz a regra documentada para
Rust e se verifica a equivalência com testes diferenciais. Se a lógica ainda
não foi recuperada, a função permanece pendente. O desenho abaixo organiza o
trabalho; não autoriza preencher lacunas com uma solução genérica de DJ.

Nesta fase, o REA é usado junto com o Ghidra para navegar funções, chamadas,
referências cruzadas, dados e pseudocódigo, relacionando os resultados em um
dossiê de evidência. O implementador pode consultar essa análise diretamente
ao traduzir a lógica para Rust. O processo separado de *clean-room* está
reservado para depois que o aplicativo estiver disponível ao público geral.

## Ponto de partida

| já existe | falta |
| --- | --- |
| Faixas carregadas no deck com grade, cues, loops e formas de onda (ANLZ + XML) | decodificar o áudio |
| Posição de agulha na UI (fixa em 0 s) | relógio de reprodução |
| Controles desenhados (CUE, play, loop, sync, pitch, Q, MT, mixer) | qualquer comportamento |
| Fixtures sintéticas em `tests/reference/synthetic/` (WAV com BPM, grade, cues e loops conhecidos) | oráculo de áudio |
| Pistas no mapa (`00-mapa`): `AudioCallback*`, `PioneerWASAPIDevice`, `PioneerASIODevice`, `DjPlayerQuantizePolicy`, `DjSyncSlaveBehaveDispatcher`, `SetThreadPriority`/`AvSetMmThreadPriority` | dossiê G0 do transporte |

O mapa já indica threads distintas para WASAPI, leitura antecipada, PCM,
estiramento de tempo e gravação. Não estabelece ainda frequência, tamanho de
bloco, política de fila nem a ordem exata de processamento. Os controles da UI
que parecem ativos devem ficar desabilitados até que seu comando esteja ligado
ao motor, conforme U0.6. A carga de faixa, os metadados e a persistência vêm
do núcleo headless (P3); P6 recebe uma referência de áudio já resolvida.

## Regras que valem para todo o plano

- **G0 primeiro.** Nenhum comportamento entra sem dossiê de evidência e spec
  (30-deck a 34-quantize). Cada dossiê identifica versão e hash do artefato,
  rotinas e dados, cadeia de chamadas, limites da análise e oráculo reproduzível.
  Usar REA para localizar e correlacionar as evidências do binário e Ghidra para
  inspecionar a decompilação e conferir casos ambíguos; registrar os limites de
  cada ferramenta. O que for escolha nossa é marcado `ADAPTAÇÃO`.
- **Tradução rastreável.** Para cada rotina reconstruída, registrar entradas,
  saídas, unidades, estado anterior e posterior, ordem das operações, constantes
  e condições de erro. A implementação Rust e seus testes apontam para esses
  requisitos. Sem essa correspondência, a rotina não está pronta para codificação.
- **Fase atual de port.** Quem investiga e quem implementa podem ler o export
  do Ghidra e as evidências do REA, rastrear uma rotina até o código Rust e
  conferir a tradução por testes. Pseudocódigo decompilado continua sendo
  aproximação do binário, não fonte original; ramos, tipos e fórmulas duvidosos
  exigem conferência no assembly, nos dados ou no programa em execução.
- **Clean-room posterior.** Os prompts `tools/re/prompts/analista.md` e
  `tools/re/prompts/auditor.md` descrevem uma etapa futura de separação de
  papéis, a iniciar após a disponibilização pública do app. Não bloqueiam o
  acesso direto às evidências durante o port atual.
- **Tempo real.** O callback de áudio não aloca, não bloqueia, não usa mutex e
  não acessa disco. Toda comunicação com ele passa por filas sem trava.
- **Oráculo antes do alto-falante.** Cada comportamento tem primeiro uma
  renderização offline comparável por teste. Quando o programa de referência
  não fornecer saída determinística ou a captura não existir, registrar a lacuna
  e usar somente invariantes verificáveis até obter comparação diferencial.
- **Núcleo sem UI.** `vilelabox-audio` não depende do Slint. A UI envia comandos
  e lê snapshots de posição.
- **Dados separados.** ANLZ/XML oferecem grade, cues e waveform de preparação;
  a reprodução lê o arquivo de áudio associado. A waveform ANLZ não é PCM.
- **Tempo em amostras.** O motor publica posições em frames de áudio e a taxa
  que lhes dá significado. Segundos e pixels são conversões de apresentação.
- **Falha previsível.** Falha de decodificação, dispositivo ausente, underrun e
  comando inválido têm estados explícitos. Nenhum deles derruba a UI.

## Arquitetura a confirmar pelo G0

```text
UI (Slint) ──comandos──▶ core/AppState ──▶ produtor único ──▶ fila RT ──▶ motor
   ▲                                                                │
   └──── snapshot de apresentação ◀── estado publicado ◀──────────────┘

leitor/decodificador ──PCM preparado──▶ buffer por deck ───────────────▶ motor
                                                          │
                             render offline ◀── mesmo processamento ──▶ saída
```

- **Crate `vilelabox-audio`**:
  - `decode`: arquivo → PCM `f32` intercalado, taxa interna fixa (ADAPTAÇÃO até o
    G0 dizer a taxa do programa);
  - `deck`: estado de transporte por deck (parado, tocando, cue, loop), posição em
    amostras, velocidade;
  - `mixer`: soma dos decks, ganho, EQ, crossfader (fase A4);
  - `render`: o mesmo grafo processado offline em blocos, para testes;
  - `output`: dispositivo e callback (WASAPI primeiro).

Esta decomposição em módulos só pode ser fixada depois de mapear com REA e
Ghidra as fronteiras reais entre decodificação, transporte, processamento e
dispositivo.
Se uma biblioteca pronta ocultar ou alterar uma regra recuperada, ela não pode
substituir essa regra sem prova de equivalência.
- **Posição é a fonte de verdade da agulha**: o motor publica um snapshot
  coerente de posição em frames, taxa, geração da faixa, estado e contador de
  underruns. A UI lê no ritmo visual, descarta snapshots de gerações antigas e
  recalcula a janela da forma de onda (`waveform::janela`, `marks::grade`).
  A frequência de atualização da UI não controla o transporte.
- **Comandos têm ordem definida**: carregar, play/pause, CUE, seek, loop e
  parâmetros do mixer atravessam uma fronteira com produtor único. O G0 define
  quais mudanças precisam valer em um frame preciso. Comando recusado ou fila
  cheia deve ser comunicado ao core; não se perde silenciosamente um CUE/stop.
- **Um grafo, dois destinos**: a renderização offline e a saída do dispositivo
  chamam o mesmo processamento por bloco. Conversão para o formato específico
  do dispositivo fica na borda de saída.
- **Arquivo inteiro em memória** na primeira versão (uma faixa de 6 min estéreo em
  `f32` a 44,1 kHz ≈ 127 MB por deck) é apenas uma hipótese para o primeiro
  protótipo offline. Definir limite de memória e comportamento para faixas longas
  no A0; a leitura antecipada paginada indicada no mapa pode exigir streaming
  já na primeira saída em tempo real.

### Contrato mínimo entre core e motor

| dado ou operação | dono | condição de aceite |
| --- | --- | --- |
| `TrackId`, local de áudio, duração e geração da carga | core | troca de faixa invalida comandos e snapshots antigos |
| grade, cues, loops e estado de quantização | core | posições convertidas para frames com arredondamento definido em spec |
| play, pause, CUE, seek, loop e controles contínuos | motor | ordem preservada; processamento de eventos no bloco definido pelo G0 |
| posição, modo, erro e underruns por deck | motor | snapshot coerente e consumível sem bloquear o callback |
| forma de onda, relógio exibido e controles habilitados | UI | derivados do snapshot e do estado do core; sem alterar o relógio de áudio |

Antes de implementar, fixar na spec as unidades, o domínio dos valores, o
comportamento nos limites da faixa e a semântica de repetir comandos. Evitar
`f32` para representar posições persistidas ou para decidir igualdade de frames.

### Dependências candidatas (a decidir após A0)

| crate | para quê | alternativa |
| --- | --- | --- |
| `symphonia` | decodificação fora do callback | backend limitado aos formatos comprovados no A0 |
| `cpal` | saída via API do sistema | WASAPI/ASIO direto se o comportamento recuperado exigir |
| `rtrb` | fila SPSC entre produtor único e motor | `ringbuf` ou estrutura própria mínima |
| `rubato` | resampling, se a fórmula e as condições de borda coincidirem | implementação traduzida da spec |

Registrar versão, licença, plataforma, suporte a formato, alocações no caminho
de áudio e divergência com o oráculo antes de adicionar qualquer dependência.
As crates são meios de portar a lógica recuperada, não fontes da lógica do
produto. Onde houver divergência funcional, traduzir a regra observada para
Rust ou manter a função bloqueada até conseguir verificá-la.
Master tempo exige estiramento temporal; algoritmo e parâmetros aguardam o G0
da fase A5.

## Fases

Cada fase fecha com `fmt`, Clippy, testes e `spec-lint`, como o plano geral.

### A0 — Dossiê G0 do transporte (REA + Ghidra)

Produzir um dossiê por tema em `spec/rekordbox/evidence/`. Usar o REA para
abrir o binário identificado, explorar relações entre funções e dados e
conservar identificadores de evidência e limitações. Usar o Ghidra para
examinar os trechos decompilados e confirmar os pontos que permanecerem
ambíguos. Cruzar as duas leituras antes de escrever a regra funcional:

| tema | perguntas que o dossiê responde |
| --- | --- |
| Dispositivo e callback | taxa de amostragem interna, tamanho de bloco, formato (f32/i16), threads e prioridades, WASAPI exclusivo ou compartilhado |
| Estados do deck | máquina de estados de parado, tocando, cue, pausa no cue, fim da faixa |
| CUE | comportamento do botão CUE parado e tocando, ponto de cue padrão, cue automático (`INT`) |
| Seek e salto | arredondamento da posição, salto por clique na forma de onda, salto por batida |
| Loop | auto loop por batidas (valores de `8`, `‹ ›`), loop de memória, saída e reloop |
| Velocidade | faixa de pitch (±6/10/16/WIDE), passo, resolução do valor exibido |
| Fila e relógio | origem do contador, precisão de comandos, ordem de eventos no bloco, overflow, pausa e troca de faixa |
| Decodificação | formatos aceitos, canais, taxas, conversão, delay/padding de codec e comportamento com arquivo inválido |
| Saídas | roteamento master/fone, formato negociado, hotplug, troca de dispositivo e recuperação de perda |
| Renderização | ordem entre transporte, resampling, EQ, fader, crossfader, master, limitador e efeitos |

Para cada tema, o analista marca `OBSERVADO`, `INFERIDO` e `DESCONHECIDO`.
Capturas de áudio e medições dinâmicas precisam registrar configuração,
dispositivo, taxa, buffer, latência informada e versão exata do programa.
Comparações com saída do sistema consideram a contribuição do driver e do
dispositivo; um desvio não prova sozinho uma regra do motor.

Saída inicial: `p6-0-transporte.md` + specs `30-deck.md` e
`32-loops-cues.md` aprovadas. Dossiês de mixer, sync e quantize são gates
específicos antes de A4 e A5. Sem evidência de decodificação e relógio, A1
não começa. O A0 deve também produzir uma matriz `ação → entrada → estado
anterior → resultado observável → evidência → teste`.

### A1 — Decodificação e renderização offline

- `decode` para MP3/WAV/FLAC; erros de arquivo ausente ou corrompido não derrubam o app.
- Aceitar apenas os formatos confirmados no G0; distinguir formato não
  suportado, arquivo corrompido e falha de I/O.
- Definir PCM interno, conversão de canais, taxa e tratamento de fim de arquivo
  a partir da spec. Decodificação e abertura de arquivo ocorrem fora do callback.
- `render(deck, comandos, n_frames)` determinístico sob a mesma configuração.
- Testes: WAV sintético decodificado bate amostra a amostra; render do trecho
  `[t0, t1]` bate com o recorte do arquivo quando não há processamento; arquivo
  truncado vira erro, não pânico. MP3/AAC exigem tolerância e alinhamento de
  delay/padding definidos antes da comparação.
- Aceite: render em blocos de tamanhos diferentes produz o mesmo estado e a
  mesma sequência de frames, dentro da tolerância numérica documentada.

### A2 — Transporte de um deck com saída real

- Play, pause, CUE e seek conforme `30-deck`.
- Primeiro executar sequências completas somente no render offline; depois
  ligar a saída negociada com WASAPI conforme o G0 e a configuração disponível.
- Publicar estado e posição coerentes; medir discrepância entre posição audível,
  posição do motor e posição mostrada pela UI.
- UI: botões play e CUE ativos; agulha e forma de onda ampliada rolam; tempo
  decorrido e restante correm; clique na visão geral faz seek.
- Testes: play → pause → play, CUE pressionado/solto, seeks antes de zero e
  depois do fim, carga durante reprodução e retorno do dispositivo. Cada
  sequência compara estado e posição com a spec.
- Aceite RT: instrumentar alocação e bloqueio no callback, inclusive ao receber
  comandos; medir underruns, tamanho efetivo do buffer, latência e recuperação
  após perda do dispositivo. Números e equipamento ficam no relatório de teste.

### A3 — Loops, cues e velocidade

- Hot cues A–H e memória: saltar, tocar a partir do ponto.
- Auto loop (8, 4…), metade/dobro (`‹ ›`), loop de memória salvo.
- Pitch com fader sem master tempo, conforme faixa, passo, arredondamento e
  algoritmo de resampling recuperados no G0.
- Testes: fixture `cues_and_loops_124bpm` valida os limites registrados no
  seu JSON; oito batidas a 124 BPM duram aproximadamente 3,871 s antes do
  arredondamento em frames. Também cobrir cue no fim, loop inválido, metade e
  dobro, mudança de velocidade dentro do loop e troca de faixa.
- Aceite: entrada e saída de loop e disparo de cue ocorrem no frame definido
  pela spec. Tratamento da emenda e tolerância de descontinuidade vêm da
  evidência; nenhum crossfade é introduzido por suposição.

### A4 — Dois decks e mixer

- Segundo deck, soma, ganho de canal, TRIM/HIGH/MID/LOW, fader de canal e
  crossfader, CUE de fone (precisa de saída com 4 canais ou segundo dispositivo).
- Spec `33-mixer-fx` com ordem dos estágios, curvas, constantes, roteamento e
  comportamento em saturação antes de implementar.
- Testes offline: impulso, silêncio, canais isolados e senoides de níveis
  conhecidos medem ganho, resposta de EQ, crossfader e separação master/fone.
- Aceite: dois decks mantêm relógios independentes; carregar ou pausar um não
  move o outro. Saída de fone só é habilitada depois de testar roteamento real.

### A5 — Sync, quantize e master tempo

- `31-sync` e `34-quantize` do G0: BEAT SYNC alinha andamento e fase pela grade;
  Q prende ações à batida ou subdivisão recuperada na spec.
- Separar seleção do deck master, ajuste de andamento e correção de fase; cobrir
  grade variável, downbeat ausente e troca de master.
- Master tempo e KEY SYNC só entram após dossiês próprios; não são sinônimos
  de BEAT SYNC. Documentar qualidade, latência e artefatos do estiramento.
- Testes: dois decks com grades diferentes ficam em fase dentro da tolerância
  definida na spec; ações Q são agendadas e executadas no frame esperado,
  inclusive quando o comando chega perto da fronteira do bloco.

### A6 — Desempenho e validação

- Soak test de 1 h com dois decks; zero underrun no equipamento de teste.
- Comparação de render offline com gravação do programa de referência, operada
  pelo usuário, nos casos que o G0 marcar como verificáveis.
- Medir latência de comando até saída, desvio entre decks, CPU, memória e
  efeitos de trocar taxa, buffer e dispositivo; registrar configuração e logs.
- Exercitar suspensão/retomada, remoção de dispositivo, arquivo que desaparece,
  carga repetida e desligamento enquanto o callback está ativo.
- Gate G6 do plano geral: relatório por dispositivo e cenário, com limites
  explícitos e comparação diferencial onde houver oráculo.

## Ordem e dependências

```text
P3 core/carga estáveis ──▶ A0 (G0 de decodificação e transporte) ──▶ A1 ──▶ A2
                                                               │
                              G0 de cues/loops ────────────────┴──▶ A3
                              G0 de mixer/saídas ──────────────────▶ A4
                              G0 de sync/quantize/MT ──────────────▶ A5 ──▶ A6
```

## Próximo pacote executável

1. Verificar que a carga headless do P3 fornece arquivo, identidade, grade e
   cues por deck; registrar qualquer lacuna como pré-requisito concreto.
2. Investigar com REA e Ghidra decodificação, relógio, callback e comandos
   básicos do deck; preencher a matriz de transições e escrever
   `p6-0-transporte.md`.
3. Revisar `30-deck.md` e o recorte inicial de `32-loops-cues.md` contra o
   dossiê e criar oráculos para WAV, play, pause, CUE, seek e fim da faixa.
4. Escolher dependências e modo WASAPI com base nas exigências medidas; registrar
   qualquer diferença como `ADAPTAÇÃO`.
5. Entregar A1 com um deck renderizado offline e relatório de paridade antes de
   ativar os controles da UI.

## Definition of Done de cada pacote

- dossiê e spec rastreiam cada regra de áudio até evidência identificada;
- cada módulo Rust indica quais regras recuperadas traduz e quais adaptações
  técnicas foram necessárias; nenhuma regra funcional deriva apenas de uma
  crate escolhida ou de convenção de mercado;
- testes cobrem transições, limites, erro e tamanho de bloco;
- comparação diferencial indica configuração, tolerância e lacunas;
- callback instrumentado não aloca, bloqueia ou faz I/O quando aplicável;
- `cargo fmt --all -- --check`, Clippy com `-D warnings`, testes do workspace e
  `cargo run -q -p xtask -- spec-lint spec/rekordbox` passam;
- documentação e status do roadmap refletem somente comportamento entregue.
