# CLAUDE.md — VILELABOX

## O que é

Fork do Mixxx 2.5.6 (C++20, Qt 6, CMake), nome de código **blackmesa**
(executável continua `build/mixxx`), com
recursos de compassos, grid, frases, vocal, cues automáticos, Mix Point,
relacionadas, pads e skins `RbxOriginal`/`RbxFlowBars`. Inspirado no rekordbox.

- Commit-base upstream: `3ebac449e7` (Release 2.5.6). Commits próprios:
  `git log 3ebac449e7..HEAD`. Não há tag `2.5.6` no clone.
- **Rust é o destino do produto independente.** O workspace já contém
  `vilelabox-rekordbox`, `vilelabox-types`, `vilelabox-mixxx-compat`,
  `vilelabox-reference` e `xtask`. O rekordbox é a única referência de
  comportamento; o Mixxx sai do caminho do produto Rust. O código C++
  permanece no repositório até sua remoção ser autorizada.
- Visão geral e estado: [README.md](README.md). Um doc por recurso:
  COMPASSOS, GRID, TONALIDADE, VOCALS, FRASES, CUES, MIXPOINT, RELACIONADAS,
  PADS, CONFIGURACOES. Histórico: PLANO*.md, IMPLEMENTACAO.md. Validação real:
  VALIDACAO.md (ainda vazio).

## Comandos (Linux)

```bash
tools/debian_buildenv.sh setup     # dependências (apt, Qt 6)
bash build-bars.sh                 # configura build/, compila, roda verify-mod --no-build
tools/verify-mod.sh [--no-build|--full]   # testes do mod (gtest + Python + skins)
bash run-bars.sh                   # abre com perfil de teste separado
```

- Scripts supõem o repo em `~/.mixxx/mixxx-source`.
- `run-mixxx.sh` usa e edita o perfil principal `~/.mixxx`. Não rodar sem o
  usuário pedir.
- Falhas preexistentes do Mixxx: `tools/verify/known-failures.txt`.
- Windows: `build-windows.bat` compila em `build-win/` (MSVC 2022, CMake/Ninja
  via `pip --user`, dependências do Mixxx em `buildenv/`), sem gtest nem
  benchmarks; `run-windows.bat` abre com perfil de teste `../bars-profile-win`.
  Testes do mod continuam só no Linux (`tools/verify-mod.sh`). Não rodar build
  aqui sem avisar; marcar comandos não executados como "não executado".

## Regras

- Português do Brasil em docs, commits e respostas.
- Evidência antes de afirmar: toda afirmação sobre recurso, teste ou resultado
  cita `arquivo:linha` ou commit. Sem fonte = "não verificado".
- Separar sempre **implementado** / **em desenvolvimento** / **planejado**.
  Testes automatizados usam áudio sintético; validação real está pendente em
  todos os módulos. Nunca declarar "pronto para uso ao vivo".
- Mudanças de código: pequenas, com teste gtest no módulo e passando em
  `tools/verify-mod.sh`. Não alterar `lib/`, `src/controllers/`, `res/controllers/`
  sem pedido explícito.
- Recursos novos são opt-in ou com padrão conservador. Não mudar esquema do
  `mixxxdb.sqlite`; dados próprios vão em caches de `analysis/`.
- rekordbox: por decisão do usuário em 2026-10-22, é permitido copiar arte
  (ícones, imagens, elementos visuais), modelos, mapeamentos e binários do
  instalador e executar código dele. Nada disso é GPL: identificar como de
  terceiros (AlphaTheta) e não declarar licença própria para esse material.
- Licença GPL-2.0-or-later herdada: preservar avisos de copyright do Mixxx e de
  `lib/`. Não declarar licença nova.
- Sem commit, push ou PR sem pedido do usuário. Remoto: `origin` =
  `Thiagovilela2001/VILELABOX` (IMPLEMENTACAO.md cita remotos antigos).

## Agents para análise e documentação

Para tarefas grandes de leitura (README, inventário, auditoria, planos):

1. **Onda 1, até 3 agents em paralelo, só leitura.** Eixos sem sobreposição,
   por exemplo: (a) inventário de recursos e status; (b) build, testes e CI;
   (c) licença, dependências, arquitetura e Rust. Cada um grava evidências no
   scratchpad, uma linha por afirmação: `afirmação | arquivo:linha |
   VERIFICADO / INFERIDO / NÃO ENCONTRADO`, mais "Inconsistências" e
   "Pendências".
2. **Onda 2:** o agent principal é o único que escreve no repo, a partir dos
   arquivos de evidência (handoff pelo caminho do arquivo, não por resumo).
3. **Onda 3:** um agent revisor só leitura confere cada afirmação do resultado
   contra o repo; o principal corrige e entrega relatório com: verificado, a
   confirmar, planejado e inconsistências.

Agents nunca editam o repo, nunca rodam build e nunca fazem commit.

## Clean-room

- O agent principal e os implementadores nunca abrem `rekord_clone.*`,
  `third_party/rekordbox/re/` nem arquivos de evidência.
- Toda especificação em `spec/rekordbox/` passa pelo `spec-lint` e por um
  auditor de contaminação e lastro antes de ser promovida.
- Para código Rust, a separação clean-room prevalece sobre a permissão de
  copiar material do rekordbox; o código é escrito somente a partir das
  especificações públicas aprovadas.
