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
