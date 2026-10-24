# 00 — Mapa geral

**Escopo.** Este documento delimita os processos, subsistemas, fluxos de ciclo de vida, componentes da instalação e famílias de dados necessárias para planejar a implementação limpa do VILELABOX. Ele é um mapa de dependências, não uma especificação detalhada de algoritmos. Esta é a revisão v2, feita sobre a instalação 7.2.8 para Windows. Toda afirmação marcada "sem confirmar" é inferência; as demais foram observadas diretamente.

**Fora do escopo.** Licença, login, serviços de nuvem, streaming, telemetria, atualização automática e proteção de conteúdo não são especificados. Componentes relacionados são apenas classificados para impedir que entrem nas fases de implementação.

**Glossário.** Neste documento:

- **Núcleo** é o processo que coordena a aplicação;
- **Agente de biblioteca** é o processo auxiliar que expõe uma API HTTP local ao Núcleo;
- **Servidor de mídia** é o auxiliar supervisionado pelo componente de banco do Núcleo e ligado, sem confirmar, ao motor de banco embarcado dos bancos de dispositivo e ao atendimento de pedidos de dados e imagens;
- **Servidor local** é um auxiliar classificado, sem confirmar, como fora do escopo;
- **Ponte de dispositivo** é a camada de comunicação com equipamentos por USB (MIDI e HID);
- **Link de rede** é o conjunto de componentes de rede local do Núcleo, incluindo um servidor de compartilhamento de arquivos; sem confirmar, ele é a parte da Ponte de dispositivo que fala com equipamentos (descoberta, estado, sincronia e compartilhamento de arquivos com eles);
- **Fila de comando de áudio** é a fronteira assíncrona entre controles e processamento sonoro;
- **Trabalhador de tarefas** é uma thread dedicada que consome uma fila de tarefas;
- **Dados de análise** são arquivos derivados de uma faixa;
- **Separação de faixas** é a decomposição de uma faixa em partes, como vocal, bateria e baixo;
- **Pacote de recursos** é um arquivo compactado da instalação com conteúdo de fábrica.

## Processos

| processo observado | papel no mapa | comunicação e iniciação |
| --- | --- | --- |
| `rekordbox.exe` (versão de arquivo 7.2.8.0) | Núcleo: reúne interface, biblioteca, análise, reprodução e integração com dispositivos. | Processo inicial da sessão. Carrega a maior parte dos componentes no próprio processo, inclusive o Link de rede e o compartilhamento de arquivos; o uso desse compartilhamento por equipamentos fica sem confirmar. Sem confirmar: trata uma segunda abertura do programa na instância já em execução. |
| `rekordboxAgent.exe` (versão 3.1.4) | Agente de biblioteca: aplicação de runtime web empacotada. Sem confirmar: oferece leitura e escrita sobre as tabelas do catálogo e serve à sincronização remota. | O Núcleo localiza o executável no subdiretório `rekordboxAgent-win32-x64` e se comunica por HTTP local em `http://127.0.0.1:30001/api/v1`. Quando os arquivos do Agente não são encontrados ou há versões divergentes em execução, o Núcleo avisa o usuário. O Núcleo contém um comando de encerramento forçado do Agente pelo nome da imagem. A chamada que inicia o Agente não foi isolada no export. |
| `edb_streamd.exe` | Servidor de mídia. | O componente de banco do Núcleo registra em log se o Servidor de mídia está ou não em execução e se sua própria thread de comunicação subiu. Sem confirmar: o estado é verificado antes de a thread subir, e o transporte é por socket em `localhost`, com porta de escuta e caminho do executável configuráveis. O número da porta não aparece no export. |
| `rbHttpServer.exe` | Servidor local. | Ligado ao login, sem confirmar; fora do escopo. |
| `rbinit.exe` | Inicializador. | Ligado à licença, sem confirmar; fora do escopo. |
| `kill_daemon.exe` | Sem confirmar, pelo nome: utilitário de encerramento associado ao Servidor de mídia. | O nome é referenciado pelo componente que supervisiona o Servidor de mídia. Argumentos e momento de uso continuam sem confirmar. |
| `Upmgr rekordbox.exe` (versão 3.0.1.2) | Gerenciador de atualização. | Referenciado pelo Núcleo; atualização automática fica fora do escopo. |

O binário principal importa operações de criação, abertura e encerramento de processo, execução pelo shell, pipes anônimos e nomeados e sockets. Pelo export, as classes de conexão entre processos do framework de interface não têm especializações próprias da aplicação. Sem confirmar: os auxiliares conhecidos usam HTTP local (Agente de biblioteca) ou socket local (Servidor de mídia). Não se observou ligação de um pipe nomeado a um auxiliar específico. A leitura de saída de processo filho existe como thread própria; sem confirmar, ela usa pipes anônimos.

Nomes de executáveis do Link de rede e do compartilhamento de arquivos aparecem no binário, mas não estão na instalação 7.2.8. Nesta versão essas funções rodam em threads do Núcleo e usam `NFSDaemon.dll`. Sem confirmar: os nomes ausentes são de versões anteriores.

## Subsistemas

| subsistema | responsabilidade | DLLs/arquivos | depende de |
| --- | --- | --- | --- |
| biblioteca e banco | catálogo, metadados, playlists, histórico, cópias de segurança e mediação de consultas | `sqlite3.dll` (3.45.3), `libcrypto-3-x64.dll` (3.3.1), `zip.dll`, `zlib.dll`, `master.db` da instalação e, sem confirmar, `product.db`; Agente de biblioteca em `rekordboxAgent-win32-x64/` | configuração, sistema de arquivos |
| análise | extrair grade, frases, tonalidade, energia, vocais e outros derivados; pontos sugeridos, sem confirmar | `onnxruntime.dll`, `DirectML.dll`, `tensorflow_cc.dll`, `vd_param.pb` (parâmetros de modelo, sem confirmar), pasta `models/` (`detect_beat/`, `detect_cue_model/`, `embed_model/`) | biblioteca, decodificação de áudio |
| separação de faixas | decompor faixas em partes; uso em reprodução e efeitos sem confirmar | `Demucs3.dll`, `Spleeter.dll`, `pad/Stems.pad.csv`; modelos complementares obtidos sob demanda, sem confirmar | análise, motor de áudio |
| waveform | construir e apresentar formas de onda completas, detalhadas e por bandas | Dados de análise, cache de pontos de onda | análise, interface |
| motor de áudio | decodificar, transportar e mixar áudio por decks e saídas | `libmpg123.dll`, decodificação de mídia do sistema, ASIO e WASAPI; `binary/` (sons de clique; uso pelo metrônomo sem confirmar) | biblioteca, configuração de áudio |
| sync e quantize | alinhar tempo, grade e ações performáticas | grade rítmica, estados dos decks | motor de áudio, análise |
| efeitos e sampler | processar efeitos e reproduzir amostras e sequências | `GROOVE_CIRCUIT_FACTORY_SAMPLES.spp`, `MERGE FX.zip`, `OSC_SAMPLER.zip`, pasta `pad/` | motor de áudio, sync e quantize |
| interface | apresentar navegador, decks, mixer, formas de onda, preferências e guias | `skins/`, `locale/` (arquivos `.lang`), `binary/` (imagens de deck e logotipo), `Introduction/`, `tutorial/`, `DemoTrack.zip`, `WebView2Loader.dll` (sem confirmar) | todos os serviços de domínio |
| MIDI e HID | descobrir equipamentos USB e traduzir mensagens em comandos | `MidiMappings/*.midi.csv`; instaladores em `drivers/` (não usados pelo VILELABOX) | Ponte de dispositivo, motor de áudio |
| Link de rede | compartilhar arquivos; sem confirmar: fazê-lo com equipamentos na rede, descobrir equipamentos, trocar estado e sincronia e compartilhar o catálogo | `NFSDaemon.dll` (5.0.0.2); sem confirmar: `PSvLinkSysMgrINI.xml`, `edb_streamd.exe`, `kill_daemon.exe` | biblioteca, motor de áudio, exportação USB e dispositivos |
| exportação USB e dispositivos | materializar catálogo, análise e ajustes em mídia removível | `PIONEER/rekordbox/export.pdb`, `PIONEER/rekordbox/exportExt.pdb`, `PIONEER/USBANLZ/`, `PIONEER/Artwork/` no dispositivo | biblioteca, análise, sistema de arquivos |
| configuração | persistir preferências de usuário, áudio, controles e recursos habilitados | arquivos do perfil em `%APPDATA%/Pioneer/rekordbox6/`; padrões de fábrica em `pad/` e `MidiMappings/` da instalação | sistema de arquivos |

A coluna "depende de" é uma arquitetura de trabalho inferida, sem confirmar, a partir dos agrupamentos observados. Ela deve ser refinada pelas especificações detalhadas antes de orientar código.

### Inventário de componentes da instalação

Todas as DLLs de primeiro nível de `C:\Program Files\rekordbox\rekordbox 7.2.8\`:

| componente | subsistema | papel |
| --- | --- | --- |
| `sqlite3.dll` | biblioteca e banco | motor SQLite do catálogo; importado diretamente pelo Núcleo |
| `libcrypto-3-x64.dll` | biblioteca e banco | biblioteca de criptografia importada diretamente; sem confirmar, usada pelo catálogo cifrado e por conexões seguras |
| `zip.dll` | biblioteca e banco; efeitos e sampler | leitura e escrita de arquivos ZIP, importada diretamente; sem confirmar, usada em cópias de segurança e Pacotes de recursos |
| `zlib.dll` | biblioteca e banco | descompressão; importado diretamente |
| `onnxruntime.dll` | análise | execução de modelos ONNX de `models/`, com opção de aceleração DirectML; importado diretamente |
| `DirectML.dll` | análise | aceleração por GPU para modelos; sem confirmar, acionada pelo `onnxruntime.dll` |
| `tensorflow_cc.dll` | análise | execução de modelos TensorFlow (`models/detect_beat/`); carregado por nome |
| `Demucs3.dll` | separação de faixas | separador de partes; carregado por nome |
| `Spleeter.dll` | separação de faixas | separador de partes alternativo; carregado por nome |
| `libmpg123.dll` | motor de áudio | decodificação MP3; carregado por nome |
| `NFSDaemon.dll` | Link de rede | servidor de compartilhamento de arquivos com lista de exportação e monitoração de montagem, importado diretamente; o uso para equipamentos em rede fica sem confirmar |
| `WebView2Loader.dll` | interface | carregador do navegador embutido; carregado por nome; sem confirmar, usado por páginas web dentro da aplicação, majoritariamente fora do escopo |
| `lsapiw64.dll` | fora do escopo | licença |
| `clatc.dll` | fora do escopo | componente sem metadados, classificado como proteção, sem confirmar |

Executáveis e pastas restantes:

| componente | subsistema | papel |
| --- | --- | --- |
| `rekordbox.exe`, `rekordboxAgent-win32-x64/` | ver Processos | Núcleo e Agente de biblioteca; a pasta do Agente traz o runtime web e as bibliotecas gráficas próprias dele |
| `edb_streamd.exe`, `kill_daemon.exe` | sem confirmar: Link de rede; biblioteca e banco | Servidor de mídia e, sem confirmar, utilitário de encerramento |
| `rbHttpServer.exe`, `rbinit.exe` | fora do escopo | login e licença, ambos sem confirmar |
| `Upmgr rekordbox.exe` | fora do escopo | atualização |
| `Uninstall rekordbox.exe`, `vc_redist.x64.exe` | instalação | desinstalação e runtime C++ do sistema |
| `residentmanager/`, `firmware/`, `drivers/` | fora do escopo | atualização de firmware e instaladores de drivers de equipamentos |
| `LIGHTING_DB.zip` | iluminação | sem confirmar: base de fixtures de iluminação; ver Perguntas abertas |
| `updatefwrule.bat` | instalação | sem confirmar, pelo nome: script de regras de firewall do instalador; conteúdo não analisado |

## Threads e tempo real

O export confirma, pela hierarquia de classes, threads dedicadas para estes papéis:

- comunicação e laço principal do componente que supervisiona o Servidor de mídia;
- Link de rede: comunicação entre módulos, cliente e servidor TCP, UDP, monitoração, temporizadores de intervalo curto e normal, contador de tempo de batida, conexão de dispositivos e compartilhamento de arquivos;
- áudio: a thread do dispositivo WASAPI, um combinador de dispositivos, Trabalhadores de tarefas (uma variante com trava de rotação e outra não bloqueante), leitura antecipada de áudio paginado, leitores de PCM, esticamento de tempo e tom, gravação e gerenciamento de análise de áudio;
- análise: análise de batida e análise de coleção;
- waveform: preparação de formas de onda no player e no editor, cache de pontos de onda e descarte de formas de onda;
- dispositivos: entrada e saída MIDI, HID e saída HID e MIDI em segundo plano;
- infraestrutura: temporizador do framework, conjunto de trabalhadores, detecção de mídia removível, observação de arquivos e cache de capas.

Com ASIO, nenhuma thread própria do dispositivo aparece na hierarquia. Sem confirmar: o retorno de áudio ASIO roda na thread do driver.

O binário carrega dinamicamente a biblioteca de agendamento multimídia do sistema e referencia o registro de thread na classe `Pro Audio` e o ajuste de prioridade dessa classe. Sem confirmar: é a thread do dispositivo WASAPI que se registra como tarefa multimídia de tempo real. O binário também importa ajuste de prioridade de thread, de prioridade de processo, de afinidade de CPU e de resolução do temporizador do sistema. A gravação tem um ajuste de prioridade próprio, aplicado por tarefa. O único temporizador de alta resolução próprio da aplicação encontrado pelo método de hierarquia está ligado à interface do player; que não haja outros fica sem confirmar.

Há tarefas de conjunto de trabalhadores para processamento de áudio paralelizado e para Separação de faixas. Análise e waveform têm threads próprias e distintas. Sem confirmar: elas não compartilham um conjunto comum de trabalhadores.

O agrupamento de dispatchers e classes de tarefa sugere, sem confirmar, o seguinte fluxo: comandos de interface, MIDI ou HID chegam ao domínio de reprodução e são convertidos em tarefas. Essas tarefas são consumidas por Trabalhadores de tarefas, fora do manipulador da interface, e formam a Fila de comando de áudio deste mapa. A capacidade da fila, a política de perda de comandos, o tamanho de bloco e o prazo máximo continuam sem confirmar.

## Inicialização e encerramento

A ordem abaixo é provisória e está toda sem confirmar. O export não preserva a ordem das chamadas dentro de uma função, então a sequência vem de dependências observáveis:

1. o Núcleo inicia, trata a abertura de uma segunda instância na instância já aberta e registra em log os diretórios temporário, de dados comuns, de dados do usuário e de documentos;
2. resolve o diretório do catálogo, carrega preferências do perfil e prepara o catálogo local; nessa fase instala ou atualiza no perfil a cópia de `product.db` da instalação;
3. inicia ou encontra o Agente de biblioteca e o Servidor de mídia quando esses serviços são necessários; o componente de banco verifica se o Servidor de mídia está em execução antes de subir sua thread de comunicação;
4. constrói os serviços de análise, dispositivos, Link de rede, interface e áudio conforme o modo ativo; nessa fase o Link de rede faz a verificação de portas descrita abaixo;
5. habilita processamento de áudio somente após selecionar e abrir o dispositivo;
6. no pedido de saída, pede confirmação quando há gravação, reprodução ou download em andamento;
7. no encerramento, interrompe novas tarefas, fecha áudio e dispositivos, para o compartilhamento de arquivos, descarrega serviços e aciona a limpeza dos auxiliares, inclusive o encerramento forçado do Agente quando necessário.

Fatos observados que sustentam essa ordem:

- existem mensagens de log dos quatro diretórios do passo 1;
- existem mensagens de instalação e de atualização de `product.db`;
- existem mensagens de confirmação de saída com gravação, reprodução ou download em andamento;
- existe mensagem de erro quando o dispositivo de saída ou de entrada não abre;
- o Link de rede registra uma verificação das portas 2049, 50000, 50001, 50002, 50004 e 50111 e uma falha ao abrir porta UDP. Essas seis são todas as portas dessa verificação no export. Que todas sejam UDP, e o papel de cada uma, ficam sem confirmar.

Além dessas, o export mostra a porta 30001 do Agente de biblioteca e outras portas locais ligadas a componentes fora do escopo. Nenhum outro número de porta foi encontrado nas strings do export, mas a lista pode ser parcial, sem confirmar, porque portas montadas em tempo de execução não aparecem como texto.

Falhas de comunicação com o Agente de biblioteca são apresentadas ao usuário. O Servidor de mídia possui estados observáveis de execução e de ausência. A política de repetição e os tempos limite permanecem perguntas abertas.

## Arquivos de dados

| item | caminho portátil | formato | dono no mapa |
| --- | --- | --- | --- |
| catálogo principal | `%APPDATA%/Pioneer/rekordbox/master.db`; cópias `master.backup.db`, `master.backup2.db` e `master.backup3.db` na mesma pasta | SQLite cifrado em modo WAL; os arquivos `-wal` e `-shm` existem durante o uso, sem confirmar | biblioteca e banco |
| bancos auxiliares do perfil | `%APPDATA%/Pioneer/rekordbox/`: `product.db`, `networkAnalyze6.db`, `networkRecommend.db`, `datafile.edb`, `ExtData.edb` e cópias `.backup.edb` | sem confirmar, pela extensão: SQLite (`.db`) e bancos do motor embarcado de dispositivo (`.edb`) | biblioteca e banco |
| listas do perfil | `%APPDATA%/Pioneer/rekordbox/masterPlaylists6.xml`, `automixPlaylist6.xml` | XML | biblioteca e banco |
| análise local | `%APPDATA%/Pioneer/rekordbox/share/PIONEER/USBANLZ/<grupo>/<faixa>/` com `ANLZ0000.DAT`, `ANLZ0000.EXT` e `ANLZ0000.2EX`; `<grupo>` tem três dígitos hexadecimais | contêiner binário por tags | análise |
| capas locais | `%APPDATA%/Pioneer/rekordbox/share/PIONEER/Artwork/` | imagens, sem confirmar (conteúdo não aberto) | biblioteca e banco |
| análise em dispositivo | `<DISPOSITIVO>/PIONEER/USBANLZ/<grupo>/<faixa>/ANLZnnnn.DAT` e arquivos complementares | contêiner binário por tags | análise e exportação USB |
| preferências | `%APPDATA%/Pioneer/rekordbox6/rekordbox3.settings` e cópia `rekordbox3.backup.settings` | arquivo de configuração da aplicação | configuração |
| preferências por área | `%APPDATA%/Pioneer/rekordbox6/`: `PlaySettings.xml`, `FxUnitSettings7.xml`, `PadFxSettings.xml`, `SamplerSettings1.xml`, `EditSettings.xml`, `MYSETTING.DAT`, `MYSETTING2.DAT`, `DJMMYSETTING.DAT`, `DEVSETTING.DAT` | XML e ajustes binários; que os `.DAT` sejam ajustes de equipamento fica sem confirmar | configuração |
| mapeamentos do usuário | `%APPDATA%/Pioneer/rekordbox6/MidiMappings/`, `KeyMappings/`, `pad/` | arquivos de mapeamento, sem confirmar (conteúdo não aberto) | MIDI e HID; configuração |
| estado do Agente | `%APPDATA%/Pioneer/rekordboxAgent/storage/options.json` | JSON | Agente de biblioteca |
| mapeamentos MIDI de fábrica | `%PROGRAMFILES%/rekordbox/rekordbox 7.2.8/MidiMappings/*.midi.csv` | CSV de comandos e mensagens | MIDI e HID |
| definições de pads | `%PROGRAMFILES%/rekordbox/rekordbox 7.2.8/pad/*.pad.csv` e um arquivo `Version` | CSV de pads; um arquivo por modo, sem confirmar, pelos nomes | efeitos e sampler |
| modelos de análise | `%PROGRAMFILES%/rekordbox/rekordbox 7.2.8/models/` e `vd_param.pb` | modelos ONNX e modelos TensorFlow salvos; `vd_param.pb` como parâmetros fica sem confirmar | análise |

Os nomes `rekordbox` e `rekordbox6` são pastas distintas do perfil: a primeira guarda catálogo e análise, a segunda guarda preferências. O caminho do perfil é apresentado com variável de ambiente e deve ser descoberto pela API do sistema, nunca formado com um nome de usuário. A escrita no catálogo original e em dados reais de dispositivo não faz parte dos testes limpos.

## Lista de especificações C2–C6

| id | título | subsistema | depende de | oráculo |
| --- | --- | --- | --- | --- |
| 10-master-db | catálogo principal | biblioteca e banco | 00-mapa | cópia de catálogo local |
| 11-anlz | dados de análise | análise | 00-mapa | arquivos USBANLZ copiados |
| 12-xml | intercâmbio XML | biblioteca e banco | 00-mapa | arquivos gerados pelo programa real |
| 13-export-usb | exportação para dispositivo | exportação USB | 10-master-db, 11-anlz | mídia exportada pelo programa real |
| 20-bpm-grid | andamento e grade | análise | 11-anlz | USBANLZ real |
| 21-frases | segmentação musical | análise | 11-anlz, 20-bpm-grid | USBANLZ real |
| 22-tonalidade | detecção tonal | análise | 11-anlz | USBANLZ real |
| 23-energia | estimativa de energia | análise | 11-anlz | USBANLZ real |
| 24-waveform | formas de onda | waveform | 11-anlz | USBANLZ real |
| 25-vocal | posição de vocais | análise | 11-anlz | USBANLZ real |
| 26-biblioteca | consultas e coleções | biblioteca e banco | 10-master-db | cópia de catálogo local |
| 27-preparacao | preparação de faixas | biblioteca e banco | 20-bpm-grid, 26-biblioteca | sessão observada e catálogo copiado |
| 40-telas | composição das telas | interface | 26-biblioteca | capturas operadas pelo usuário |
| 41-waveform-render | renderização de waveform | interface | 24-waveform, 40-telas | capturas operadas pelo usuário |
| 42-biblioteca-ui | navegador da biblioteca | interface | 26-biblioteca, 40-telas | capturas operadas pelo usuário |
| 30-deck | transporte por deck | motor de áudio | 20-bpm-grid, 27-preparacao | renderização offline e gravação operada pelo usuário |
| 31-sync | sincronização de decks | sync e quantize | 20-bpm-grid, 30-deck | renderização offline e gravação operada pelo usuário |
| 32-loops-cues | loops e pontos de disparo | motor de áudio | 30-deck, 31-sync | renderização offline e gravação operada pelo usuário |
| 33-mixer-fx | mixer, efeitos e sampler | efeitos e sampler | 30-deck | renderização offline e gravação operada pelo usuário |
| 34-quantize | quantização de ações | sync e quantize | 20-bpm-grid, 30-deck | renderização offline e gravação operada pelo usuário |
| 35-separacao | Separação de faixas | separação de faixas | 30-deck, 33-mixer-fx | renderização offline e gravação operada pelo usuário |
| 50-midi | protocolo MIDI | MIDI e HID | 30-deck, 33-mixer-fx | controladora operada pelo usuário |
| 51-hid | protocolo HID | MIDI e HID | 30-deck, 33-mixer-fx | controladora operada pelo usuário |
| 52-mapeamentos | mapeamentos de controles | MIDI e HID | 50-midi, 51-hid | arquivos de fábrica e controladora operada pelo usuário |
| 53-link-rede | Link de rede | Link de rede | 13-export-usb, 31-sync | captura de rede com equipamento operado pelo usuário |

A ordem de execução permanece C2, C3, C3b, C5, C4 e C6: o motor de áudio começa depois que biblioteca e interface tiverem contratos estáveis. As entradas 35-separacao e 53-link-rede são propostas novas da v2. O encaixe delas em fases fica sem confirmar até decisão pública: a sugestão é C4 para 35-separacao, junto do motor de áudio, e C6 para 53-link-rede, junto de controladoras e dispositivos.

## Fora do escopo encontrado

| componente | motivo da exclusão |
| --- | --- |
| cliente de licença | licença |
| inicializador externo | licença, sem confirmar |
| componente protegido sem metadados | proteção, sem confirmar |
| Servidor local | login, sem confirmar |
| Agente de biblioteca — funções remotas | login, sincronização de nuvem e serviços remotos não serão reproduzidos |
| integrações de catálogo remoto | streaming não faz parte do produto limpo |
| envio de informações de uso | telemetria não será coletada nem reproduzida |
| serviços de compartilhamento remoto | nuvem e publicação externa não serão reproduzidas |
| gerenciador de atualização, atualização de firmware e instaladores de drivers | atualização automática e software de equipamento não fazem parte do produto limpo |

Os nomes acima são categorias próprias do VILELABOX, e não identificadores internos. A presença dos componentes serve somente para definir a fronteira de exclusão.

## Perguntas abertas

- Qual porta o Servidor de mídia usa por padrão e quem a define na inicialização?
- O Agente de biblioteca é necessário para alguma operação local que o VILELABOX precise reproduzir, ou toda operação local é atendida diretamente pelo catálogo?
- O utilitário de encerramento recebe nome de processo, identificador ou canal de controle?
- Qual é a sequência exata de desligamento quando há exportação, análise ou gravação em andamento?
- Além da thread do dispositivo WASAPI, quais threads recebem prioridade elevada e qual afinidade de CPU é aplicada?
- Qual é a capacidade da Fila de comando de áudio e o que acontece quando ela enche?
- Quais campos e versões distinguem cada variante dos Dados de análise, inclusive `.EXT` e `.2EX`?
- Quais arquivos dentro de `pad/` são configuração pública e quais são somente recursos da instalação?
- Quais modelos locais correspondem a cada recurso de análise e quais serão substituídos por algoritmos próprios?
- Iluminação, vídeo e gravação aparecem como subsistemas no binário. Devem ganhar especificações próprias ou ficam fora do produto?
- Na verificação de portas do Link de rede (2049, 50000, 50001, 50002, 50004 e 50111), qual é o protocolo e o papel de cada porta, e há portas abertas em tempo de execução que não aparecem como texto no export?
