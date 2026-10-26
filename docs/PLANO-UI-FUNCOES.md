# Plano — funções do layout

Liga os controles do layout Slint (`vilelabox-app/ui/`) a comportamento real.
Detalha as fases P5.1–P5.3 do [`IMPLEMENTATION-PLAN.md`](IMPLEMENTATION-PLAN.md)
e respeita o Gate G0: nenhum controle ganha comportamento sem spec e dossiê de
evidência.

## Estado atual

O layout está modularizado (P0.1), mas quase tudo é estático.

| região | componente | funciona hoje | estático ou falso |
| --- | --- | --- | --- |
| menu superior | `MenuBar` | — | 7 menus sem ação |
| barra de performance | `PerformanceToolbar` | — | "LINK ATIVO", relógio fixo `10:24` |
| trilho de ícones | `IconRail` | — | botões sem ação |
| árvore | `NavigationSidebar` | — | playlists fixas no código |
| lista de faixas | `LibraryBrowser` | lista do scan USBANLZ, clique seleciona | busca sem efeito, colunas sem ordenação, `TOM` fixo `8A` |
| decks 1 e 2 | `Deck` | título, BPM, compassos e frases da seleção | **os dois decks mostram a mesma seleção**; CUE, play, loop, sync, slip, Q, MT, tempo e hot cues são decorativos; nomes e cores A–H fixos |
| mixer | `MixerStrip`, `MixerBar` | — | todo decorativo |
| status | `StatusBar` | texto | — |

Fonte de dados hoje: `library::scan` lê `.DAT/.EXT` direto da pasta de análises.
Não há catálogo, núcleo (`vilelabox-core`) nem motor de áudio.

## Princípios

1. **A UI só emite comandos e recebe snapshots.** Toda lógica vai para crates
   testáveis sem Slint; `main.rs` e `presentation.rs` só traduzem dados para a
   tela. Nenhuma consulta de dados ou SQL em componentes Slint.
2. **Controle sem backend fica visivelmente desabilitado**, sem fingir que
   funciona. Hoje vários botões parecem ativos e não fazem nada.
3. **Ordem pelas dependências:** primeiro o que só precisa de dados que já
   temos (ANLZ, XML), depois o que precisa do núcleo, por último o áudio.
4. Cada pacote termina com `fmt`, Clippy, testes e `spec-lint`, como manda o
   plano geral.

## Fase U0 — Fundação UI/core (P5.1)

Pré-requisito de todas as demais.

| # | tarefa | entrega | teste |
| --- | --- | --- | --- |
| U0.1 | Estado da aplicação fora do Slint | `struct AppState` (faixas, seleção, decks, filtro) em módulo próprio | unitário de transições |
| U0.2 | Comandos | `enum Command` (`SelectTrack`, `LoadToDeck{deck}`, `Search(String)`, `SortBy(col)`…) e `fn apply(&mut AppState, Command)` | unitário por comando |
| U0.3 | Snapshot → tela | `presentation::render(&AppState, &AppWindow)`, substitui os `set_*` espalhados | teste do mapeamento sem janela |
| U0.4 | Estado por deck | propriedades `deck1_*` e `deck2_*` separadas em `app.slint` | dois decks mostram faixas diferentes |
| U0.5 | Trabalho fora da thread gráfica | scan e carga em thread, resultado via `slint::invoke_from_event_loop` | a janela abre antes de o scan terminar |
| U0.6 | Desabilitar o que não tem backend | propriedade `enabled` nos botões; estilo apagado | revisão visual |

`vilelabox-core` (P3.3) existe desde 2026-10-26; `AppState` e `Command` migram
para ele no P5 sem mudar a UI. U0 é a forma mínima que já nasce com esse formato.

## Fase U1 — Biblioteca (P5.2)

Depende de U0. Usa só dados que já existem.

| # | função | região | origem dos dados | spec |
| --- | --- | --- | --- | --- |
| U1.1 | Busca por texto | campo "Procurar" | `AppState` (filtro em memória) | 42-biblioteca-ui (a escrever) |
| U1.2 | Ordenação por coluna | cabeçalhos | `AppState` | 42-biblioteca-ui |
| U1.3 | Coluna `TOM` real | lista e deck | `Tonality` do XML ou análise; vazio quando não houver | 12-xml |
| U1.4 | Importar XML de coleção | menu Ficheiro | crate `vilelabox-xml` (P2.2) | 12-xml (pronta) |
| U1.5 | Árvore de playlists real | `NavigationSidebar` | `PLAYLISTS` do XML | 12-xml |
| U1.6 | Seleção separada da carga | lista → deck | duplo clique ou arrastar carrega; clique só seleciona | 40-telas |
| U1.7 | Estados vazios e erros | lista e status | — | 40-telas |

**Bloqueio:** U1.4 e U1.5 precisam do crate `vilelabox-xml`. Sugestão: fazer o
P2.2 logo antes desta fase, porque a spec 12-xml já está pronta.

## Fase U2 — Deck sem áudio: preparação (P5.3)

Depende de U1.6. Edita dados; não toca som.

| # | função | dados | spec |
| --- | --- | --- | --- |
| U2.1 | Hot cues A–H reais (nome, cor, posição) | `POSITION_MARK` do XML, `PCOB/PCPT` do ANLZ | 12-xml, 11-anlz |
| U2.2 | Lista de memory cues e loops | idem, `Num=-1` | 12-xml |
| U2.3 | Waveform derivada da análise | seções de waveform do ANLZ | 24-waveform (a escrever) |
| U2.4 | Agulha (cursor) clicável na waveform | posição em segundos | 41-waveform-render (a escrever) |
| U2.5 | Zoom sem alterar posições musicais | — | 41-waveform-render |
| U2.6 | Criar, mover e apagar cue, com undo/redo | comando do core | 27-preparacao (a escrever) |
| U2.7 | Edição de grade | comando do core | 20-bpm-grid (a escrever) |

**Bloqueio:** U2.6 e U2.7 precisam persistir dados, ou seja, do
`vilelabox-storage` e do `vilelabox-core` (P3). O storage existe desde
2026-10-25 e o core desde 2026-10-26; o desbloqueio efetivo vem com a migração do app para o `Core` (P5).

## Fase U3 — Menus, toolbar e status

Baixo risco; pode ocorrer em paralelo a U1 e U2.

| # | função | observação |
| --- | --- | --- |
| U3.1 | Menus com itens reais | só itens com backend; o resto não aparece |
| U3.2 | Relógio real na toolbar | hora do sistema |
| U3.3 | Indicador "LINK" | some até a fase de rede (C6) |
| U3.4 | Status com progresso de tarefas | scan, importação, análise |
| U3.5 | Escala da janela por preferência | hoje é variável de ambiente `VLBX_ESCALA` |

## Fase U4 — Transporte e mixer (P6)

**Bloqueada até existir `vilelabox-audio` e as specs 30–34.** Os controles
ficam desabilitados (U0.6) até lá.

| # | controle | spec |
| --- | --- | --- |
| U4.1 | play/pause, CUE, posição e tempo restante | 30-deck |
| U4.2 | pitch, faixa ±6%, MT (master tempo) | 30-deck |
| U4.3 | BEAT SYNC, KEY SYNC | 31-sync |
| U4.4 | loop, 4 BEAT, ×1, SLIP | 32-loops-cues |
| U4.5 | Q (quantize) | 34-quantize |
| U4.6 | volume, EQ, crossfader e CUE de fone | 33-mixer-fx |

## Ordem recomendada

```text
U0 fundação
  → P2.2 vilelabox-xml          (fora deste plano; desbloqueia U1.4–U1.5)
    → U1 biblioteca             ← U3 em paralelo
      → U2.1–U2.5 deck leitura
        → P3 storage/core       (fora deste plano)
          → U2.6–U2.7 edição
            → P6 áudio → U4
```

## Primeiro pacote

**U0.1 a U0.4**: tira a lógica de `main.rs`, cria `AppState` e `Command` e
separa o estado dos dois decks. É pequeno, não depende de nenhuma spec nova e
corrige o defeito mais visível (os dois decks espelhados).

## Pendências de spec que este plano cria

- `42-biblioteca-ui`: busca, ordenação e colunas;
- `41-waveform-render` e `24-waveform`;
- `27-preparacao` e `20-bpm-grid`;
- decisões abertas de `40-telas`: resolução de referência, modo de edição e
  tema claro.
