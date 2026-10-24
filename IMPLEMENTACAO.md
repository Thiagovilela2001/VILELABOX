# Registro de implementação

## Reorganização Rust independente — 2026-10-24

| Campo | Resultado |
| --- | --- |
| Direção | VILELABOX passa a ser uma aplicação Rust independente |
| Legado | Preservado na branch `legacy-mixxx` |
| Workspace | Promovido de `rust/` para a raiz |
| Removido do `main` | C++/Qt, build CMake, empacotamento e adaptadores específicos do legado |
| Clean-room | `spec/rekordbox`, `xtask`, ferramentas de referência e barreira de terceiros preservados |
| Ativos de terceiros | Movidos para `third_party/rekordbox/ui-workbench`, ignorado pelo Git |
| Próximo gate | Concluir cues, conversão canônica e paridade do leitor ANLZ |

O histórico detalhado da implementação anterior permanece acessível na
branch de legado.

## ANLZ: cues e conversão canônica — 2026-10-24

| Campo | Resultado |
| --- | --- |
| Parser | `PCOB`/`PCPT`, memory cues, hot cues e loops com validação de limites |
| Modelo | `RawCue`, `canonical_cues`, `canonical_phrases` e `track_from_anlz` |
| Segurança | comprimento menor que o cabeçalho e buffers truncados encerram a leitura |
| Diferencial | 40 pares `.DAT`/`.EXT` copiados para a área ignorada e comparados com referências locais |
| Testes | 47 testes Rust, Clippy com `-D warnings`, formatação e `spec-lint` aprovados |
| Correção encontrada | memory cue com slot zero não avalia mais uma subtração que causava overflow |
| Próximo gate | exportador ANLZ em Rust e variantes com label/cor |
