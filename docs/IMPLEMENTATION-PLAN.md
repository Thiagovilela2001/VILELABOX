# Plano de implementação do VILELABOX

Status de referência: 2026-10-24.

## Objetivo

Reconstruir em Rust o comportamento tecnicamente observado no programa de
referência, usando como fonte primária a engenharia reversa, a decompilação no
Ghidra, a análise assistida pelo REA e artefatos reais autorizados. O resultado
deve ser um aplicativo de DJ independente, sem dependência de outro aplicativo
em tempo de execução.

O plano avança por unidades verticais verificáveis. Cada pacote precisa entregar
código, testes, documentação e evidência antes que o pacote dependente comece.

## Regras permanentes

- Nenhuma funcionalidade é projetada do zero quando existe comportamento no
  programa de referência: a implementação começa pela evidência correspondente.
- Toda tarefa de código deve apontar para uma especificação em
  `spec/rekordbox/`; toda afirmação dessa especificação deve apontar para
  evidência de decompilação, estrutura binária ou observação reproduzível.
- REA e Ghidra são usados em conjunto para recuperar fluxo, estruturas,
  constantes, tabelas, estados e relações entre funções; o implementador pode
  consultar diretamente as evidências nesta fase do port. Testes diferenciais
  confirmam o comportamento que a análise estática não consegue provar sozinha.
- Pseudocódigo decompilado é evidência, não código-fonte original. A tradução
  para Rust preserva comportamento e invariantes, sem copiar nomes ou estruturas
  sem confirmação.
- Toda decisão inevitavelmente nova — integração entre crates, segurança de
  memória, persistência própria ou adaptação de API — é registrada como
  `ADAPTAÇÃO`, com justificativa e limite claramente separados do que foi
  observado.
- A separação de papéis *clean-room* é uma etapa futura, após a disponibilização
  pública do aplicativo. Ela não é pré-requisito para o port atual; as regras
  de proveniência, verificação e proteção dos artefatos reais continuam valendo.
- Dados reais são copiados para uma pasta ignorada antes de qualquer teste.
- Nenhum teste escreve em catálogos, dispositivos ou perfis reais.
- Formatos externos entram por adaptadores e não definem o modelo interno.
- A interface depende do núcleo; o núcleo nunca depende da interface.
- Operações de áudio em tempo real não alocam, bloqueiam ou acessam disco.
- Marcas, logos, ícones e outros ativos proprietários não entram no produto.
- Todo pacote termina com:

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo run -q -p xtask -- spec-lint spec/rekordbox
```

## Fluxo obrigatório de reconstrução

Nenhum pacote entra em implementação antes de completar este ciclo:

```text
artefato identificado e hash registrado
  → função/recurso localizado com REA e Ghidra
    → pseudocódigo, assembly, chamadas, dados e constantes documentados
      → observações, inferências e lacunas separadas
        → especificação aprovada em spec/rekordbox
          → fixtures e oráculos diferenciais preparados
            → tradução para Rust
              → comparação com o programa de referência
```

Cada investigação produz um registro em `evidence/` ou no local equivalente
ignorado pelo Git, e um resumo versionável sem conteúdo proprietário em
`spec/rekordbox/evidence/`. O resumo precisa conter:

- versão e SHA-256 do artefato analisado;
- endereços ou identificadores das funções e dados relevantes;
- cadeia de chamadas e estados observados;
- constantes, layouts e fórmulas recuperados;
- classificação de cada conclusão como `OBSERVADO`, `INFERIDO` ou `DESCONHECIDO`;
- limitações da análise estática e resultado da confirmação dinâmica;
- ligação para requisito, fixture e teste Rust correspondentes.

### Gate G0 — Proveniência da implementação

Antes de iniciar qualquer item das fases P0–P7:

- existe um dossiê de evidência para o comportamento em questão;
- a especificação cita esse dossiê e delimita as lacunas;
- há um oráculo ou teste diferencial definido;
- adaptações necessárias ao Rust estão identificadas, sem serem apresentadas
  como comportamento recuperado;
- se faltar evidência, a tarefa retorna à engenharia reversa e não avança por
  suposição de produto.

## Estado inicial

Já concluído:

- workspace Rust na raiz;
- tipos musicais e esquema canônico;
- leitor ANLZ de contêiner, caminho, grade, frases e cues básicos;
- conversão ANLZ para `ReferenceTrack`;
- comparação diferencial aprovada em 40 cópias de faixas reais;
- protótipo Slint que lista ANLZ e exibe BPM, compassos e frases;
- CI Rust, fixtures sintéticas e `spec-lint`.

## Ordem de execução

```text
P0 estabilização
  → P1 formatos ANLZ
    → P2 demais formatos
      → P3 storage, biblioteca e core
        → P4 análise musical
          → P5 interface funcional
            → P6 motor de áudio
              → P7 dispositivos e recursos avançados
```

---

## P0 — Encerrar a estabilização

### P0.1 — Modularizar a interface Slint

**Objetivo:** retirar a definição visual de `main.rs` e separar componentes de
acordo com a hierarquia, os estados e as responsabilidades recuperados da
interface de referência, sem inventar comportamento.

**Arquivos previstos:**

```text
vilelabox-app/
  build.rs
  ui/
    app.slint
    theme.slint
    components/
      deck.slint
      library.slint
      mixer.slint
      navigation.slint
      status.slint
  src/
    main.rs
    library.rs
    presentation.rs
```

**Tarefas:**

1. localizar com REA e Ghidra a composição da tela, modelos, eventos e estados
   usados;
2. registrar a correspondência entre controles recuperados e componentes Slint;
3. adicionar `slint-build` como dependência de build;
4. mover structs e componentes visuais para arquivos `.slint` seguindo a
   decomposição recuperada;
5. manter carregamento e transformação de dados em Rust;
6. mover formatação de BPM, duração e cores para `presentation.rs`, usando as
   regras e constantes identificadas;
7. criar testes unitários e comparações visuais para a camada de apresentação;
8. verificar que a janela compila e abre com uma pasta vazia.

**Aceite:**

- nenhum arquivo de UI excede 500 linhas;
- `main.rs` somente compõe dependências e inicia a janela;
- nenhuma mudança visual ou funcional deliberada;
- CI completo aprovado.

---

## P1 — Concluir o formato ANLZ

### P1.1 — Exportador ANLZ em Rust

**Objetivo:** produzir referências canônicas diretamente pelo leitor Rust.

**Origem obrigatória:** rotinas de enumeração, associação DAT/EXT, resolução de
caminhos e serialização recuperadas com REA e Ghidra e confrontadas com o exportador
já observado. O formato JSON canônico é uma `ADAPTAÇÃO` interna documentada.

**Arquivos previstos:**

```text
vilelabox-anlz/src/export.rs
vilelabox-anlz/src/bin/vilelabox-anlz-export.rs
vilelabox-anlz/tests/export.rs
vilelabox-anlz/tests/export_parity.rs
```

**Interface inicial:**

```text
vilelabox-anlz-export
  --anlz-dir <cópia USBANLZ>
  --music-dir <cópia da biblioteca>
  --output <diretório>
  [--limit N]
```

**Tarefas:**

1. localizar pares `.DAT`/`.EXT` recursivamente;
2. resolver o áudio somente dentro de `--music-dir`;
3. calcular SHA-256 por streaming;
4. converter com `track_from_anlz`;
5. gravar JSONs de forma atômica;
6. gerar `index.json` determinístico;
7. recusar entrada e saída sobrepostas;
8. comparar os 40 resultados reais com as referências existentes.

**Aceite:**

- nenhuma leitura fora das raízes fornecidas;
- execuções repetidas produzem bytes idênticos;
- falha parcial não deixa um `index.json` válido apontando para arquivos ausentes;
- paridade das 40 faixas aprovada.

### P1.2 — Variantes de cues

**Pré-requisito:** especificação aprovada descrevendo labels, cores e a variante
de cue ainda não coberta pelo parser atual.

**Tarefas:**

1. criar fixtures binárias mínimas para cada variante;
2. estender `RawCue` sem expor offsets fora do módulo de formato;
3. preencher `label` e `color` no modelo canônico;
4. preservar entradas desconhecidas como aviso, sem panic;
5. validar contra pelo menos uma cópia real que contenha cues.

**Aceite:** memory cue, hot cue e loop preservam posição, slot, duração,
label e cor.

### Gate G1 — ANLZ concluído

- parser tolerante a truncamento e tags desconhecidas;
- exportador determinístico;
- paridade real documentada;
- nenhuma dependência da interface.

---

## P2 — Completar os formatos de interoperabilidade

### P2.1 — Catálogo principal

1. localizar com REA e Ghidra abertura, consultas, transformação de registros e
   relações do catálogo;
2. concluir e aprovar `spec/rekordbox/10-master-db.md` com essa evidência;
3. decidir como fixtures descriptografadas e minimizadas serão produzidas sem
   incorporar chave, DRM ou mecanismo de licença;
4. criar `vilelabox-catalog` somente leitura;
5. importar faixas, metadados, playlists, cues e relacionamentos;
6. testar versões conhecidas do esquema e erros de versão.

### P2.2 — XML

1. usar `spec/rekordbox/12-xml.md` como contrato;
2. criar `vilelabox-xml`, independente de qualquer banco interno;
3. implementar leitura e escrita com preservação de campos conhecidos;
4. cobrir caracteres especiais, URLs, grades variáveis, loops e playlists
   aninhadas;
5. validar round-trip e comparar com XMLs gerados pelo programa de referência.

### P2.3 — Exportação USB

1. concluir `spec/rekordbox/13-export-usb.md`;
2. criar escritor inicialmente direcionado a uma imagem de diretório temporária;
3. gerar manifesto e validar todo arquivo antes da publicação;
4. implementar publicação atômica no destino;
5. testar apenas em mídia descartável ou imagem de filesystem.

### Gate G2 — Formatos concluídos

- ANLZ, catálogo, XML e exportação USB possuem crates independentes;
- fixtures cobrem arquivos válidos, truncados e versões desconhecidas;
- nenhum formato externo aparece nos tipos do núcleo.

---

## P3 — Banco próprio, biblioteca e núcleo headless

### P3.1 — `vilelabox-storage`

- recuperar com REA e Ghidra entidades, relações, índices, transações e ciclo de vida
  dos dados antes de definir o esquema;
- banco SQLite próprio com migrations versionadas, tratado como `ADAPTAÇÃO`
  quando o armazenamento de referência não puder ser reproduzido diretamente;
- conexões, transações e integridade referencial;
- tabelas para faixas, localizações, metadados, playlists, cues, grids, frases
  e resultados de análise;
- backup antes de migration;
- testes de upgrade, rollback e recuperação de interrupção.

### P3.2 — `vilelabox-library`

- importadores chamam adaptadores de formato e escrevem no storage;
- busca, ordenação e filtros;
- árvore de playlists com ordem estável;
- identidade por conteúdo e localizações independentes;
- detecção de arquivos movidos, ausentes e duplicados.

### P3.3 — `vilelabox-core`

**Comandos iniciais:**

- importar diretório ou formato externo;
- selecionar faixa;
- preparar e carregar faixa em um deck;
- criar, mover e remover cue;
- editar grid;
- criar e reorganizar playlists.

**Eventos iniciais:**

- biblioteca alterada;
- seleção alterada;
- faixa carregada;
- preparação alterada;
- tarefa de análise atualizada.

### P3.4 — Aplicativo headless

Criar uma CLI de desenvolvimento para provar o fluxo:

```text
importar → pesquisar → selecionar → carregar → editar cue → persistir
```

### Gate G3 — Preparação independente

- fluxo completo funciona sem interface gráfica;
- reiniciar o processo preserva todo o estado;
- testes não dependem de diretórios pessoais.

---

## P4 — Análise musical

### P4.1 — Infraestrutura DSP

- recuperar com REA e Ghidra tipos numéricos, tamanhos de bloco, janelas, constantes,
  encadeamento e tratamento de bordas;
- criar `vilelabox-dsp` com contrato PCM explícito;
- leitura em blocos produz o mesmo resultado independentemente do tamanho do
  bloco;
- janelas, FFT e estatísticas sem I/O;
- benchmarks reproduzíveis.

### P4.2 — Pipeline de análise

- reconstruir cada etapa somente após localizar suas entradas, saídas, chamadas,
  parâmetros e tabelas no binário;
- criar `vilelabox-analysis`;
- implementar BPM/grid, downbeat, tonalidade, energia, waveform, frases e vocal
  como etapas canceláveis;
- cache por hash do áudio, versão do algoritmo e configuração;
- comparar cada etapa com as referências reais e sintéticas.

### Gate G4 — Análise reproduzível

- resultados determinísticos no mesmo backend;
- tolerâncias registradas por métrica;
- cancelamento e arquivos corrompidos não comprometem o banco.

---

## P5 — Interface funcional

### P5.1 — Modelo UI/core

- canal de comandos da UI para o core;
- snapshots imutáveis do core para a UI;
- tarefas longas fora da thread gráfica;
- nenhuma consulta SQL dentro de componentes Slint.

### P5.2 — Biblioteca

- seletor de pasta e importação;
- busca, filtros, playlists e estados vazios;
- progresso, cancelamento e erros acionáveis;
- seleção separada de carga no deck.

### P5.3 — Preparação e waveform

- waveform derivada do áudio;
- zoom e redimensionamento sem alterar posições musicais;
- edição real de cues, loops, grid e frases;
- undo/redo por comandos do core.

### Gate G5 — Interface de preparação

- fluxo headless do G3 está totalmente acessível pela interface;
- testes de interação cobrem busca, carga e edição;
- layouts de referência passam em resoluções definidas.

---

## P6 — Motor de áudio

### P6.1 — Decodificação e transporte

- recuperar com REA e Ghidra o grafo do transporte, estados, relógios, buffers e regras
  de seek antes de escolher APIs Rust;
- criar `vilelabox-audio`;
- decodificação para PCM interno;
- play, pause, seek, ganho e transporte por deck;
- renderização offline como primeiro oráculo.

### P6.2 — Mixer e sincronização

- recuperar equações, curvas, estados e constantes antes de implementá-los;
- mixer de canais e crossfader;
- resampling e alteração de tempo;
- sync por batida e compasso;
- quantização, loops e cues.

### P6.3 — Efeitos e desempenho RT

- cadeia de efeitos e sampler;
- filas lock-free entre core e callback;
- medição de underruns, latência e uso de CPU;
- soak test e validação auditiva em dispositivo real.

### Gate G6 — Performance básica

- dois decks reproduzem e sincronizam sem underruns no equipamento de teste;
- loops e cues permanecem dentro da tolerância definida;
- render offline e saída em tempo real são comparáveis.

---

## P7 — Dispositivos e recursos avançados

Executar em pacotes independentes, nesta ordem:

1. MIDI e formato de mapeamentos;
2. HID e primeiro dispositivo real;
3. gravação;
4. separação de faixas;
5. vídeo sincronizado;
6. rede;
7. iluminação.

Cada pacote precisa de simulador ou fixture automatizada antes do teste com
equipamento real.

---

## Próximos pacotes imediatos

Ordem recomendada para as próximas entregas:

1. **G0/P0.1:** documentar com REA e Ghidra a hierarquia e os estados da interface;
2. **P0.1:** modularizar a interface conforme a evidência recuperada;
3. **G0/P1.1:** documentar o fluxo de exportação e associação ANLZ;
4. **P1.1:** traduzir esse fluxo para o exportador ANLZ em Rust;
5. **G0/P1.2:** recuperar labels, cores e variantes de cues;
6. **P1.2:** implementar as variantes conforme as evidências;
7. **G0/P2.1:** mapear com REA e Ghidra o catálogo principal e suas consultas.

## Definition of Done

Uma tarefa só está concluída quando:

- possui dossiê G0 com identidade do artefato e rastreabilidade até REA/Ghidra;
- separa explicitamente `OBSERVADO`, `INFERIDO`, `DESCONHECIDO` e `ADAPTAÇÃO`;
- a especificação necessária está aprovada;
- entradas válidas, limites, truncamento e erros possuem testes;
- `fmt`, Clippy, testes e `spec-lint` passam;
- dados reais foram usados somente por cópia, quando aplicável;
- a comparação diferencial está registrada;
- o roadmap e `IMPLEMENTACAO.md` foram atualizados;
- não existem ativos proprietários ou dados pessoais no diff;
- o commit é pequeno o suficiente para ser revertido isoladamente.
