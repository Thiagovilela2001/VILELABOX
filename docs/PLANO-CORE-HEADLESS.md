# Plano — núcleo headless e CLI (P3.3–P3.4)

Detalha as fases P3.3 e P3.4 do [`IMPLEMENTATION-PLAN.md`](IMPLEMENTATION-PLAN.md).
Objetivo: um núcleo headless com comandos e eventos (`vilelabox-core`) e uma
CLI de desenvolvimento que prova o fluxo completo de preparação sem interface
gráfica (Gate G3).

O modelo de comandos e eventos é desenho do VILELABOX (`ADAPTAÇÃO`), ensaiado
pelo `AppState`/`Command` do app (fase U0). Os dados persistidos seguem o
banco próprio (P3.1); os comportamentos vindos do programa de referência
continuam exigindo spec e dossiê de evidência (Gate G0).

## Ponto de partida

| já existe | falta |
| --- | --- |
| `vilelabox-storage`: faixas, localizações, cues, grades, frases, playlists e cache de análise (P3.1) | ponto único de mutação do estado |
| `vilelabox-library`: importadores de catálogo e XML, varredura, busca, ordenação, identidade, movidos e duplicatas (P3.2) | comandos e eventos do núcleo |
| App com `AppState`/`Command` fora do Slint (U0), formato já compatível com a migração para o core | decks headless com preparação carregada do banco |
| Regra de arquitetura: a interface depende do núcleo, nunca o contrário | CLI do fluxo ponta a ponta |

## Princípios

1. Toda mutação passa por `Core::executar(Comando)`; consultas são métodos de
   leitura sem efeito colateral.
2. O núcleo não conhece Slint nem o motor de áudio; toca o filesystem somente
   nos comandos de importação e verificação.
3. Cada comando emite os eventos previstos, numa fila drenável e ordenada.
4. Erros são tipados (`ErroCore`); nenhum comando entra em pânico com entrada
   inválida, faixa inexistente ou deck vazio.
5. Cada pacote termina com `fmt`, Clippy, testes e `spec-lint`, como manda o
   plano geral.

## Fase C1 — `vilelabox-core` (P3.3)

| # | tarefa | entrega | teste |
| --- | --- | --- | --- |
| C1.1 | Estrutura do crate | `Core { db, decks, selecao, eventos }`, `Deck { Um, Dois }`, `DeckState { faixa, cues, grade, bloqueada }` | unitário de construção |
| C1.2 | Comandos de importação | `ImportarDiretorio`, `ImportarCatalogo`, `ImportarXml`, `VerificarLocalizacoes`, `BuscarMovidos` → `BibliotecaAlterada` + `TarefaAnaliseAtualizada` | integração em pasta temporária |
| C1.3 | Seleção e consulta | `SelecionarFaixa` → `SelecaoAlterada`; `pesquisar(Consulta)` e `listas()` como leitura | unitário |
| C1.4 | Playlists | `CriarLista`, `AdicionarALista`, `MoverNaLista` (reordenação transacional no storage) | unitário de ordem estável |
| C1.5 | Decks | `CarregarNoDeck`: faixa + cues + grade do banco → `FaixaCarregada`; deck vazio dá erro tipado | integração |
| C1.6 | Cues do usuário | `CriarCue`, `MoverCue`, `RemoverCue` (origem `User`, persistidos no banco) → `PreparacaoAlterada` | unitário + ida-e-volta |
| C1.7 | Grade | `SalvarGrade { grade, bloqueada }` → `PreparacaoAlterada` | unitário |
| C1.8 | Eventos | `drenar_eventos()` devolve a fila na ordem exata de emissão | sequência esperada no fluxo |

**Aceite:** nenhuma mutação fora do `Core`; cada comando da tabela tem teste;
os eventos do fluxo `importar → carregar → editar cue` saem na ordem prevista.

## Fase C2 — CLI headless (P3.4)

| # | tarefa | entrega | teste |
| --- | --- | --- | --- |
| C2.1 | Binário | `vilelabox-cli` com subcomandos mínimos (importar, pesquisar, listas, carregar, cue, grade, deck), sem dependência nova de parsing | roda e imprime eventos |
| C2.2 | Fluxo de prova | `importar → pesquisar → selecionar → carregar → editar cue → persistir` | integração `tests/fluxo.rs` |
| C2.3 | Persistência | reabrir o processo preserva faixas, cues, grades e playlists | Gate G3 |

### Gate G3 — Preparação independente

- fluxo completo funciona sem interface gráfica;
- reiniciar o processo preserva todo o estado de preparação;
- testes não dependem de diretórios pessoais.

## Ligação com o que vem depois

- P4 ([`PLANO-ANALISE.md`](PLANO-ANALISE.md)) escreve identidade, grade e
  frases pelo `Core`; `TarefaAnaliseAtualizada` já prevê o acompanhamento.
- P5 troca `AppState`/`Command` do app pelo `Core` sem mudar a UI (U0 já
  nasceu nesse formato).
- P6 recebe do `Core` a faixa carregada com cues e grade já resolvidos.
