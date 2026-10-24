# Plano de implementação do VILELABOX

Status de referência: 2026-10-24.

## Objetivo

Construir um aplicativo de DJ independente em Rust, seguindo as especificações
clean-room em `spec/rekordbox/`, sem dependência de outro aplicativo de DJ.

O plano avança por unidades verticais verificáveis. Cada pacote precisa entregar
código, testes, documentação e evidência antes que o pacote dependente comece.

## Regras permanentes

- O código limpo lê somente `spec/rekordbox/` e fontes públicas autorizadas.
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

**Objetivo:** retirar a definição visual de `main.rs` e separar componentes por
responsabilidade, sem alterar o comportamento.

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

1. adicionar `slint-build` como dependência de build;
2. mover structs e componentes visuais para arquivos `.slint`;
3. manter carregamento e transformação de dados em Rust;
4. mover formatação de BPM, duração e cores para `presentation.rs`;
5. criar testes unitários para a camada de apresentação;
6. verificar que a janela compila e abre com uma pasta vazia.

**Aceite:**

- nenhum arquivo de UI excede 500 linhas;
- `main.rs` somente compõe dependências e inicia a janela;
- nenhuma mudança visual ou funcional deliberada;
- CI completo aprovado.

---

## P1 — Concluir o formato ANLZ

### P1.1 — Exportador ANLZ em Rust

**Objetivo:** produzir referências canônicas diretamente pelo leitor Rust.

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

1. concluir e aprovar `spec/rekordbox/10-master-db.md`;
2. decidir como fixtures descriptografadas e minimizadas serão produzidas sem
   incorporar chave, DRM ou mecanismo de licença;
3. criar `vilelabox-catalog` somente leitura;
4. importar faixas, metadados, playlists, cues e relacionamentos;
5. testar versões conhecidas do esquema e erros de versão.

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

- banco SQLite próprio com migrations versionadas;
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

- criar `vilelabox-dsp` com contrato PCM explícito;
- leitura em blocos produz o mesmo resultado independentemente do tamanho do
  bloco;
- janelas, FFT e estatísticas sem I/O;
- benchmarks reproduzíveis.

### P4.2 — Pipeline de análise

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

- criar `vilelabox-audio`;
- decodificação para PCM interno;
- play, pause, seek, ganho e transporte por deck;
- renderização offline como primeiro oráculo.

### P6.2 — Mixer e sincronização

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

1. **P0.1:** modularizar a interface sem mudar comportamento;
2. **P1.1:** exportador ANLZ em Rust;
3. **P1.2:** especificar e implementar labels e cores de cues;
4. **P2.1:** iniciar a especificação do catálogo principal;
5. **P2.2:** criar o parser XML independente.

## Definition of Done

Uma tarefa só está concluída quando:

- a especificação necessária está aprovada;
- entradas válidas, limites, truncamento e erros possuem testes;
- `fmt`, Clippy, testes e `spec-lint` passam;
- dados reais foram usados somente por cópia, quando aplicável;
- a comparação diferencial está registrada;
- o roadmap e `IMPLEMENTACAO.md` foram atualizados;
- não existem ativos proprietários ou dados pessoais no diff;
- o commit é pequeno o suficiente para ser revertido isoladamente.
