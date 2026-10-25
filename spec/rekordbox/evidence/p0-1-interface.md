# G0 — Dossiê de proveniência: Hierarquia e Estados da Interface (P0.1)

## Artefato analisado

- **Arquivo:** `rekordbox.exe`
- **Versão:** `7.2.8.0`
- **SHA-256:** `85e3a58524c5cd1e870ad59e23cc8806523175f80fb82d6059c68d57bdf9991a`
- **Registro privado detalhado:** `third_party/rekordbox/re/evidence/40-telas.md`

## Identificadores de classes, tabelas virtuais e rotinas observadas

| Região arquitetural | Símbolo RTTI / Componente | Vtable (RVA) | Rotinas analisadas (RVA) |
| --- | --- | --- | --- |
| Barra global | `juce::MenuBarComponent` | `vft_53ce9a8` | `rva_f30234`, `rva_f3b744` |
| Deck de reprodução | `browse::PlayerDeckComponent` | `vft_35d4ee8` | `rva_13abbb0`, `rva_13ad570`, `rva_13acbc0` |
| Subbloco visual de deck | `browse::PlayerDeckComponent::DeckComponent` | `vft_35d4ce0` | `rva_13abb50` |
| Forma de onda geral | `DJSystemOverviewWave` | `vft_385c618` | `rva_1b8d8b0`, `rva_1b8d460` |
| Faixa de frases na onda | `ViewLayoutPhrasesComponent` | `vft_388cb80` | `rva_1bfe870` |
| Rótulo de frase na onda | `ViewLayoutPhraseNameComponent` | `vft_388c458` | `rva_1bfe150` |
| Abstração de mixer | `AbstractMixerComponent` | `vft_5411578` | `rva_1061fa0` |
| Mixer do sistema DJ | `DJSystemMixerComponent` | `vft_385df60` | `rva_1b91150`, `rva_1b90de0` |
| Navegador primário | `ViewBrowse1` | `vft_388e140` | `rva_1c02aa0`, `rva_1c02790` |
| Navegador secundário | `ViewBrowse2` | `vft_388e438` | `rva_1c03100` |
| Barra de estado | `common::StatusBar` / `common::StatusBarComponent` | `vft_373a5f0` / `vft_373a630` | `rva_186d760`, `rva_186d8b0`, `rva_1869fa0` |

## Cadeias de chamadas e estados observados

1. **Composição da janela principal em cinco regiões:**
   - Barra global (`juce::MenuBarComponent`, `rva_f30234`) gerencia seleção de modo, disposição, painéis ativos e escala visual.
   - Área de reprodução (`browse::PlayerDeckComponent`, `rva_13ad570`) encapsula o subcomponente `DeckComponent` (`vft_35d4ce0`), a forma de onda geral (`DJSystemOverviewWave`, `rva_1b8d460`) e a trilha de frases musicais (`ViewLayoutPhrasesComponent`, `rva_1bfe870` + `ViewLayoutPhraseNameComponent`, `rva_1bfe150`).
   - Área de mixagem (`DJSystemMixerComponent`, `rva_1b90de0`) herda de `AbstractMixerComponent` (`rva_1061fa0`) e agrega cinco subcomponentes filhos independentes nos offsets de ponteiro `0x1c`, `0x1d`, `0x20`, `0x21` e `0x22`, além de lista de observadores em `+0x110..0x120`.
   - Navegador de coleção (`ViewBrowse1`, `rva_1c02790`, e `ViewBrowse2`, `rva_1c03100`) isola árvore lateral, barra de busca/filtro e grade tabular do estado de reprodução dos decks.
   - Rodapé (`common::StatusBarComponent`, `rva_1869fa0`) apresenta informações contextuais da sessão.
2. **Separação entre seleção e carga:**
   - O estado de seleção na tabela pertence a `ViewBrowse1` / `ViewBrowse2`, enquanto a faixa carregada pertence a cada instância de `browse::PlayerDeckComponent`. Alterar a seleção no navegador não substitui a faixa em reprodução sem comando explícito de carga.

## Constantes, layouts e regras recuperadas

- Limites de escala de interface: fator contínuo limitado ao intervalo `[0.80, 1.35]` com passo de `0.05` e restauração para `1.00`.
- Rótulos de frases em `ViewLayoutPhraseNameComponent`: `INTRO`, `UP`, `DOWN`, `VERSE`, `BRIDGE`, `CHORUS`, `OUTRO`.

## Classificação das conclusões

- **OBSERVADO:**
  - Hierarquia de componentes em cinco regiões (`MenuBarComponent`, `PlayerDeckComponent`, `DJSystemMixerComponent`, `ViewBrowse1`/`ViewBrowse2`, `StatusBarComponent`).
  - Desacoplamento entre `DJSystemOverviewWave`, `ViewLayoutPhrasesComponent` e `DeckComponent`.
  - Agregação de cinco subblocos filhos dentro de `DJSystemMixerComponent` (`rva_1b90de0`).
- **INFERIDO:**
  - Mapeamento funcional dos cinco subblocos de `DJSystemMixerComponent` para controles de canal A, canal B, equalização/ganho, medidores/master e crossfader/monitoração.
- **DESCONHECIDO:**
  - Coordenadas absolutas em pixels para monitor `1920x1080` em tempo de execução (a tela local não suporta essa resolução nativa).
- **ADAPTAÇÃO:**
  - Modularização da interface Slint em `vilelabox-app/ui/theme.slint`, `vilelabox-app/ui/app.slint` e `vilelabox-app/ui/components/{navigation,deck,mixer,library,status}.slint`, com formatação e estado puros em `vilelabox-app/src/presentation.rs`.

## Limitações da análise estática e confirmação

- A análise estática via RTTI e destrutores/construtores confirma a árvore de propriedade e ciclo de vida dos componentes, sem depender de captura de pixels.
- A paridade estrutural e de formatação é validada pelos testes unitários em `vilelabox-app/src/presentation.rs` e pela compilação de `vilelabox-app/build.rs`.

## Rastreabilidade para código e testes

- **Especificação:** [`spec/rekordbox/40-telas.md`](../40-telas.md)
- **Implementação Rust/Slint:** `vilelabox-app/ui/app.slint`, `vilelabox-app/ui/theme.slint`, `vilelabox-app/ui/components/*.slint`, `vilelabox-app/src/presentation.rs`, `vilelabox-app/src/main.rs`
- **Testes:** `cargo test -p vilelabox-app` (6 testes unitários de apresentação e estado)
