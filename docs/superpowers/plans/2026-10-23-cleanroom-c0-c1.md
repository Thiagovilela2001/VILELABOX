# Clean-room C0 + C1 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Montar a infraestrutura clean-room (barreira, export do Ghidra, lint de especificações em Rust) e entregar a primeira especificação, `spec/rekordbox/00-mapa.md`, com o mapa geral do rekordbox 7.2.8 e a lista final de especificações das fases C2–C6.

**Architecture:** Lado sujo = projeto Ghidra + `third_party/rekordbox/re/` (ignorados pelo git) + subagents analista/auditor. Ponte = `spec/rekordbox/*.md`, validadas por `cargo run -p xtask -- spec-lint`. O agent principal nunca abre material do lado sujo; só roda ferramentas e move arquivos sem lê-los.

**Tech Stack:** Rust 1.99.1 (workspace `rust/`, só stdlib no `xtask`), Ghidra 12.1.4 headless (`C:\Tools\ghidra_12.1.4_PUBLIC`), JDK 21, PowerShell 5.1.

**Spec:** `docs/superpowers/specs/2026-10-23-rekordbox-cleanroom-rust-design.md`

## Global Constraints

- Português do Brasil em docs, mensagens de CLI e relatórios (`CLAUDE.md`).
- Tudo em Rust, exceto scripts do Ghidra (Java), que são ferramenta do lado sujo.
- Agent principal nunca abre `rekord_clone.gpr`, `rekord_clone.rep/`, `third_party/rekordbox/re/` nem arquivos de evidência/export; verificação só por contagem de linhas, código de saída e cabeçalho de colunas definido por este plano.
- Especificação proibida de conter: pseudocódigo, trechos C/C++ do Ghidra, nomes internos (salvo nomes de formato), endereços, seção "Evidência", tabelas numéricas com mais de 16 valores sem aprovação do usuário.
- Fora do escopo: licença, login, nuvem, streaming, telemetria, contorno de proteção/DRM.
- Nenhum logo/marca AlphaTheta, Pioneer DJ ou rekordbox em artefato do produto.
- O projeto Ghidra é aberto só com `-readOnly`; nada é gravado nele.
- Sem commit, push ou PR sem pedido do usuário; os passos de commit deste plano são "pedir autorização".
- Comando não executado = registrar "não executado".
- `xtask`: `edition = "2021"`, `rust-version = "1.75"`, `license = "GPL-2.0-or-later"`, sem dependências externas.

## Review Focus

- Projeto Ghidra aberto na GUI ao rodar o export: esperado abortar com mensagem clara, sem tocar no projeto (Task 2, teste do lock).
- Artefato do Ghidra escondido em código inline (`` `FUN_1400a3b0` ``) ou em tabela: o lint deve pegar igual (Task 1, teste `pega_nome_ghidra_em_codigo_inline`).
- Especificação salva com CRLF ou BOM pelo Windows: números de linha do lint devem bater com o editor (Task 1, teste `crlf_e_bom_mantem_numero_de_linha`).
- Export ou projeto Ghidra parando no `git add .` por descuido: caminhos têm que estar ignorados (Task 1, verificação `git check-ignore`).
- Função gigante que estoura o timeout do descompilador: o lote continua e registra a falha, sem abortar (Task 2, passo de descompilação de prova).

---

### Task 1: Barreira e `xtask spec-lint`

**Files:**
- Modify: `.gitignore` (após a linha 75, `third_party/rekordbox/`)
- Modify: `rust/Cargo.toml` (members)
- Create: `rust/xtask/Cargo.toml`
- Create: `rust/xtask/src/main.rs`
- Create: `rust/xtask/src/spec_lint.rs` (com `#[cfg(test)] mod tests`)
- Create: `spec/rekordbox/README.md`

**Interfaces:**
- Produces:
  - `pub enum Rule { NomeGhidra, Endereco, TipoDescompilado, BlocoC, SecaoEvidencia, TabelaNumericaLonga }` com `fn code(&self) -> &'static str` devolvendo `"nome-ghidra"`, `"endereco"`, `"tipo-descompilado"`, `"bloco-c"`, `"secao-evidencia"`, `"tabela-numerica-longa"`.
  - `pub struct Violation { pub line: usize, pub rule: Rule }` (`line` começa em 1, `#[derive(Debug, PartialEq, Eq)]`).
  - `pub fn lint_spec(text: &str) -> Vec<Violation>`.
  - CLI: `cargo run -p xtask -- spec-lint <arquivo-ou-diretório>`; imprime `caminho:linha: código` por violação; sai com 0 se limpo, 1 se houver violação, 2 em erro de uso/I/O.

Regras (todas valem também dentro de crases e tabelas):
- `NomeGhidra`: palavra que começa com `FUN_`, `DAT_`, `LAB_`, `PTR_`, `thunk_`, `switchD_`, `caseD_`, ou que é `uVar`/`iVar`/`lVar`/`bVar`/`cVar`/`pcVar`/`pvVar`/`local_`/`param_`/`in_`/`extraout_` seguido de dígito hex.
- `Endereco`: `0x` seguido de 8 ou mais dígitos hex. (Mágicas de formato curtas, como `0x51545A50`, têm 8 dígitos e são pegas; mágica de formato vai escrita como texto `PQTZ`.)
- `TipoDescompilado`: `undefined1|2|4|8`, `undefined ` isolado, `__thiscall`, `__fastcall`, `__cdecl`, `CONCAT`/`SUB`/`ZEXT`/`SEXT` seguidos de dígito.
- `BlocoC`: abertura de bloco cercado com linguagem `c`, `cpp`, `c++`, `h` (case-insensitive). Violação na linha da abertura.
- `SecaoEvidencia`: linha de título Markdown (`#`) contendo `evidência` ou `evidencia` (case-insensitive).
- `TabelaNumericaLonga`: mais de 16 literais numéricos numa linha ou num mesmo bloco cercado, salvo se a linha imediatamente anterior (linha do bloco ou da abertura) for `<!-- tabela-aprovada: AAAA-MM-DD -->`. Tabelas Markdown com `|` contam por linha, não pela tabela inteira. Violação na primeira linha do bloco/da linha.

- [ ] **Step 1: Ignorar o lado sujo**

Acrescentar ao `.gitignore`, logo depois de `third_party/rekordbox/`:

```gitignore
# Projeto Ghidra do rekordbox (lado sujo do clean-room): nunca versionar
rekord_clone.gpr
rekord_clone.rep/
rekord_clone.lock*
```

Run: `git check-ignore rekord_clone.gpr rekord_clone.rep/project.prp third_party/rekordbox/re/export/functions.tsv`
Expected: as três linhas ecoadas, exit 0.

- [ ] **Step 2: Escrever os testes que falham** em `rust/xtask/src/spec_lint.rs`

```rust
fn rules(text: &str) -> Vec<(usize, Rule)> {
    lint_spec(text).into_iter().map(|v| (v.line, v.rule)).collect()
}

#[test]
fn texto_limpo_passa() {
    let t = "# 11 — ANLZ\n\nA tag `PQTZ` guarda o grid.\n\n| Campo | Bytes |\n| --- | --- |\n| bpm | 2 |\n";
    assert!(lint_spec(t).is_empty());
}

#[test]
fn pega_nome_ghidra_em_codigo_inline() {
    assert_eq!(rules("linha\nchama `FUN_1400a3b0` antes\n"), vec![(2, Rule::NomeGhidra)]);
    assert_eq!(rules("| x | uVar3 |\n"), vec![(1, Rule::NomeGhidra)]);
    assert_eq!(rules("param_1 recebe\n"), vec![(1, Rule::NomeGhidra)]);
}

#[test]
fn pega_endereco_longo_mas_nao_curto() {
    assert_eq!(rules("em 0x140001000\n"), vec![(1, Rule::Endereco)]);
    assert!(lint_spec("flag 0x40 e 0xFFFF\n").is_empty());
}

#[test]
fn pega_tipo_descompilado() {
    assert_eq!(rules("um undefined4 aqui\n"), vec![(1, Rule::TipoDescompilado)]);
    assert_eq!(rules("via CONCAT44\n"), vec![(1, Rule::TipoDescompilado)]);
}

#[test]
fn pega_bloco_c_e_aceita_text() {
    assert_eq!(rules("a\n```cpp\nx\n```\n"), vec![(2, Rule::BlocoC)]);
    assert!(lint_spec("```text\nestado A -> B\n```\n").is_empty());
}

#[test]
fn pega_secao_evidencia() {
    assert_eq!(rules("# X\n## Evidência\n"), vec![(2, Rule::SecaoEvidencia)]);
    assert_eq!(rules("### evidencia\n"), vec![(1, Rule::SecaoEvidencia)]);
}

#[test]
fn tabela_numerica_longa_e_aprovacao() {
    let nums: Vec<String> = (0..17).map(|n| n.to_string()).collect();
    let bloco = format!("```text\n{}\n```\n", nums.join("\n"));
    assert_eq!(rules(&bloco), vec![(1, Rule::TabelaNumericaLonga)]);
    let aprovado = format!("<!-- tabela-aprovada: 2026-10-23 -->\n{bloco}");
    assert!(lint_spec(&aprovado).is_empty());
    let dezesseis: Vec<String> = (0..16).map(|n| n.to_string()).collect();
    assert!(lint_spec(&dezesseis.join(", ")).is_empty());
}

#[test]
fn crlf_e_bom_mantem_numero_de_linha() {
    assert_eq!(rules("\u{feff}a\r\nb\r\nFUN_00401000\r\n"), vec![(3, Rule::NomeGhidra)]);
}
```

- [ ] **Step 3: Criar o crate e rodar os testes para ver falhar**

`rust/xtask/Cargo.toml` com `name = "xtask"`, `publish = false` e os campos dos Global Constraints; adicionar `"xtask"` aos `members` de `rust/Cargo.toml`. `main.rs` declara `mod spec_lint;` e `fn main()` vazio; `spec_lint.rs` com os tipos e `lint_spec` devolvendo `Vec::new()`.

Run: `cd rust; cargo test -p xtask`
Expected: FAIL nos testes de detecção (`texto_limpo_passa` passa).

- [ ] **Step 4: Implementar `lint_spec` e a CLI**

Varredura por linha com `str::lines()` (trata CRLF) após remover BOM; tokenização por `char::is_alphanumeric() || '_'`, sem regex. Estado de bloco cercado acumula contagem de números a partir da linha de abertura. Uma violação por (linha, regra). CLI: diretório = todos os `*.md` exceto `README.md`, em ordem alfabética.

- [ ] **Step 5: Rodar os testes e o clippy**

Run: `cd rust; cargo test -p xtask; cargo clippy -p xtask --all-targets -- -D warnings; cargo fmt --all -- --check`
Expected: 8 testes ok, clippy e fmt sem saída de erro.

- [ ] **Step 6: Escrever `spec/rekordbox/README.md`**

Conteúdo: o que é o diretório (ponte clean-room), o modelo de especificação do design (seção "Modelo de especificação", sem a parte de evidência), a lista "Permitido/Proibido" copiada do design, a regra de aprovação `<!-- tabela-aprovada: AAAA-MM-DD -->`, e o comando `cargo run -p xtask -- spec-lint ../spec/rekordbox` (rodado de `rust/`).

Run: `cd rust; cargo run -p xtask -- spec-lint ../spec/rekordbox`
Expected: exit 0 (README é ignorado, não há outras specs).

- [ ] **Step 7: Pedir autorização de commit ao usuário** (`.gitignore`, `rust/Cargo.toml`, `rust/Cargo.lock`, `rust/xtask/`, `spec/rekordbox/README.md`).

---

### Task 2: Export headless do Ghidra

**Files:**
- Create: `tools/re/ExportInventory.java`
- Create: `tools/re/DecompileList.java`
- Create: `tools/re/export.ps1`
- Create: `tools/re/README.md`

**Interfaces:**
- Produces (em `third_party/rekordbox/re/export/`, TSV UTF-8, primeira linha = cabeçalho exato abaixo):
  - `functions.tsv`: `address	name	namespace	size	callers	callees`
  - `classes.tsv`: `class	parent	functions`
  - `strings.tsv`: `address	xrefs	value` (`value` com `\t`, `\n` escapados)
  - `imports.tsv`: `library	symbol`
  - `callgraph.tsv`: `caller	callee` (endereços)
  - `decomp/<endereço>.c` por função pedida; falha/timeout gera o arquivo com a única linha `// falha: <motivo>`
  - `decomp/_status.tsv`: `address	status` (`ok`, `timeout`, `erro`, `nao-encontrada`)
- `export.ps1 -Mode inventory|decompile [-List <arquivo de endereços, um por linha>] [-Ghidra C:\Tools\ghidra_12.1.4_PUBLIC] [-Out third_party\rekordbox\re\export]`

APIs do Ghidra a usar: `getScriptArgs()`; `currentProgram.getFunctionManager().getFunctions(true)`; `Function.getCallingFunctions(monitor)` / `getCalledFunctions(monitor)`; `getBody().getNumAddresses()`; `currentProgram.getSymbolTable().getClassNamespaces()`; `getListing().getDefinedData(true)` com `Data.hasStringValue()`; `getReferenceManager().getReferenceCountTo(addr)`; `getExternalManager().getExternalLibraryNames()` / `getExternalLocations(lib)`; `DecompInterface.openProgram` + `decompileFunction(f, 60, monitor)`.

- [ ] **Step 1: Escrever `export.ps1` com o guarda de lock**

Aborta com `exit 3` e a mensagem `Projeto Ghidra aberto (rekord_clone.lock encontrado). Feche o Ghidra e rode de novo.` se existir `rekord_clone.lock*` na raiz do repo. Senão chama:

```text
<Ghidra>\support\analyzeHeadless.bat <raiz do repo> rekord_clone -process rekordbox.exe -readOnly -noanalysis -scriptPath tools\re -postScript <ExportInventory.java <Out> | DecompileList.java <Out> <List>>
```

e repassa o código de saída.

- [ ] **Step 2: Testar o guarda de lock**

Run: `New-Item rekord_clone.lock.test -ItemType File; powershell -File tools\re\export.ps1 -Mode inventory; "exit=$LASTEXITCODE"; Remove-Item rekord_clone.lock.test`
Expected: a mensagem acima e `exit=3`; nenhum processo Java iniciado.

- [ ] **Step 3: Escrever `ExportInventory.java` e rodar o inventário**

Garantir que o Ghidra GUI está fechado (perguntar ao usuário se o lock existir).

Run: `powershell -File tools\re\export.ps1 -Mode inventory` (pode levar dezenas de minutos; rodar em background)
Expected: exit 0 e os cinco TSV criados.

- [ ] **Step 4: Verificar sem ler o conteúdo**

Run: `Get-ChildItem third_party\rekordbox\re\export\*.tsv | % { "{0} {1} {2}" -f $_.Name, (Get-Content $_ -TotalCount 1), ((Get-Content $_ | Measure-Object -Line).Lines - 1) }`
Expected: cada cabeçalho igual ao da seção Interfaces; `functions.tsv` > 1000 linhas; `imports.tsv`, `callgraph.tsv` e `strings.tsv` > 0. Só cabeçalho e contagem aparecem no terminal.

- [ ] **Step 5: Escrever `DecompileList.java` e rodar a prova de lote**

Lista de prova: as 3 primeiras linhas de dados de `functions.tsv` (coluna `address`, extraída com `Select-Object -Skip 1 -First 3 | % { ($_ -split "\t")[0] }`, sem imprimir) mais o endereço inexistente `ffffffff`.

Run: `powershell -File tools\re\export.ps1 -Mode decompile -List <lista>`
Expected: exit 0; `decomp/_status.tsv` com 4 linhas de dados, a última `ffffffff	nao-encontrada`. Verificar com `Measure-Object` e `Select-String nao-encontrada`, sem abrir os `.c`.

- [ ] **Step 6: Escrever `tools/re/README.md`**

Pré-requisitos (Ghidra 12.1.4, JDK 21, GUI fechado), os dois modos, os arquivos e cabeçalhos produzidos, e o aviso: "Saída é lado sujo: só analistas e auditores leem. O agent principal e implementadores Rust nunca abrem."

- [ ] **Step 7: Pedir autorização de commit ao usuário** (`tools/re/`).

---

### Task 3: Prompts do analista e do auditor

**Files:**
- Create: `tools/re/prompts/analista.md`
- Create: `tools/re/prompts/auditor.md`

**Interfaces:**
- Consumes: export da Task 2; `spec/rekordbox/README.md` da Task 1.
- Produces: prompts com marcadores `{{SPEC_ID}}`, `{{ESCOPO}}`, `{{SCRATCH}}` substituídos pelo principal no despacho. Caminhos de saída:
  - analista: rascunho `{{SCRATCH}}/cleanroom/drafts/{{SPEC_ID}}.md`, evidência `{{SCRATCH}}/cleanroom/evidence/{{SPEC_ID}}.md`
  - auditor: parecer `{{SCRATCH}}/cleanroom/audits/{{SPEC_ID}}.md`, primeira linha exatamente `APROVADO` ou `REPROVADO`, depois uma linha por problema: `linha N: <regra do README> — <motivo sem citar material do lado sujo>`.

- [ ] **Step 1: Escrever `analista.md`**

Deve conter:
- papel: lado sujo;
- o que pode ler: export, instalação `C:\Program Files\rekordbox\rekordbox 7.2.8\` (listagem e metadados de DLL/EXE), análises USBANLZ copiadas;
- proibido: editar o repo, escrever Rust, rodar o rekordbox, abrir `master.db` original;
- regras Permitido/Proibido do `spec/rekordbox/README.md`;
- todo nome no rascunho vem do glossário próprio, e toda afirmação tem linha correspondente na evidência (`afirmação | função/endereço/string | VERIFICADO/INFERIDO`);
- antes de entregar, rodar `cargo run -p xtask -- spec-lint <rascunho>` em `rust/` e corrigir até exit 0;
- resposta final ao principal: só os dois caminhos e "pronto", sem conteúdo.

- [ ] **Step 2: Escrever `auditor.md`**

Deve conter:
- papel: conferir contaminação e lastro;
- lê rascunho, evidência e export;
- reprova quando: um trecho reproduz estrutura de função; nome interno aparece fora de formato; uma afirmação não tem evidência; algo cai fora do escopo (licença/nuvem/streaming/telemetria/DRM); o lint não passa;
- nunca edita o rascunho;
- o parecer não cita código, nomes internos nem endereços.

- [ ] **Step 3: Rodar o lint nos prompts**

Os prompts citam as regras proibidas como exemplo; por isso não passam pelo lint. Verificação: `cargo run -p xtask -- spec-lint ../spec/rekordbox` continua exit 0 (prompts ficam fora de `spec/`).

- [ ] **Step 4: Pedir autorização de commit ao usuário** (`tools/re/prompts/`).

---

### Task 4: Especificação `00-mapa` (C1)

**Files:**
- Create: `spec/rekordbox/00-mapa.md`
- Create (fora do git): `third_party/rekordbox/re/evidence/00-mapa.md`

**Interfaces:**
- Consumes: Tasks 1–3.
- Produces: `spec/rekordbox/00-mapa.md` com, nesta ordem, as seções do modelo e:
  1. **Processos:** papel de `rekordbox.exe`, `rekordboxAgent.exe`, `edb_streamd.exe`, `rbHttpServer.exe`, `rbinit.exe`, `kill_daemon.exe`, e como se comunicam (mecanismo, portas/pipes, quem inicia quem).
  2. **Subsistemas:** tabela `subsistema | responsabilidade | DLLs/arquivos | depende de`, cobrindo no mínimo biblioteca/banco, análise, waveform, motor de áudio, sync/quantize, efeitos/sampler, UI, MIDI/HID, export USB/dispositivos, configuração.
  3. **Threads e tempo real:** threads conhecidas, qual é de áudio, como comandos chegam ao áudio.
  4. **Inicialização e encerramento:** ordem dos passos observáveis.
  5. **Arquivos de dados:** caminhos (com variáveis de ambiente, sem nome de usuário), formato e dono de cada um (`master.db`, `USBANLZ`, configurações, `MidiMappings`, `pad`, `models`).
  6. **Lista de especificações C2–C6:** tabela `id | título | subsistema | depende de | oráculo`, que substitui ou confirma a tabela de fases do design.
  7. **Fora do escopo encontrado:** componentes de licença/nuvem/streaming/telemetria identificados e não especificados (só nome do componente e motivo).
  8. **Perguntas abertas.**

- [ ] **Step 1: Preparar pastas no scratchpad**

Run: `New-Item -ItemType Directory -Force <scratchpad>\cleanroom\drafts, <scratchpad>\cleanroom\evidence, <scratchpad>\cleanroom\audits`

- [ ] **Step 2: Despachar o subagent analista** com `tools/re/prompts/analista.md`, `{{SPEC_ID}} = 00-mapa`, `{{ESCOPO}}` = lista de seções da Interfaces acima.
Expected: resposta só com os dois caminhos.

- [ ] **Step 3: Despachar um subagent auditor novo** com `tools/re/prompts/auditor.md`, mesmo `{{SPEC_ID}}`.
Expected: `audits/00-mapa.md` existe. Ler só esse arquivo.

- [ ] **Step 4: Se `REPROVADO`**, reenviar o parecer ao mesmo analista (`SendMessage`) e voltar ao Step 3 com auditor novo. Após 3 reprovações, parar e levar o parecer ao usuário.

- [ ] **Step 5: Validar e promover o rascunho**

Run: `cd rust; cargo run -p xtask -- spec-lint <scratchpad>\cleanroom\drafts\00-mapa.md`
Expected: exit 0. Então `Copy-Item` do rascunho para `spec/rekordbox/00-mapa.md` e `Copy-Item` da evidência para `third_party/rekordbox/re/evidence/00-mapa.md` (sem abrir a evidência).

- [ ] **Step 6: Conferir as seções**

Run: `Select-String -Path spec\rekordbox\00-mapa.md -Pattern '^## '`
Expected: os 8 títulos acima presentes; leitura do principal confirma que a tabela da seção 6 cobre C2–C6.

- [ ] **Step 7: Mostrar `00-mapa.md` ao usuário e pedir autorização de commit.**

---

### Task 5: Atualizar governança e planos

**Files:**
- Modify: `CLAUDE.md` (bullet "Rust é objetivo de longo prazo" e seção "Agents para análise e documentação")
- Modify: `PLANO-RUST.md` (topo, seção "Estratégia de migração", Fase 5)
- Modify: `docs/superpowers/specs/2026-10-23-rekordbox-cleanroom-rust-design.md` (tabela "Fases")

**Interfaces:**
- Consumes: seção 6 de `spec/rekordbox/00-mapa.md`.

- [ ] **Step 1: `CLAUDE.md`**

Trocar o bullet de Rust pelo estado real:
- crates existentes: `vilelabox-rekordbox`, `vilelabox-types`, `vilelabox-mixxx-compat`, `vilelabox-reference` e `xtask`;
- o rekordbox é a única referência de comportamento, e o Mixxx sai do caminho (o código C++ fica até remoção autorizada).

Acrescentar a seção "Clean-room" com três regras:
- o principal e os implementadores nunca abrem `rekord_clone.*`, `third_party/rekordbox/re/` nem a evidência;
- toda spec passa pelo `spec-lint` e por um auditor;
- para o código Rust, o clean-room prevalece sobre a permissão de copiar material do rekordbox.

- [ ] **Step 2: `PLANO-RUST.md`**

Nota no topo apontando para o design de 2026-10-23, com o oráculo trocado para o rekordbox. A Fase 5 recebe `**Cancelada (2026-10-23):** aplicativo Rust nasce independente; ver design clean-room.`. A estratégia strangler vale só até o aplicativo Rust existir.

- [ ] **Step 3: Design**

Substituir a tabela "Fases" pela lista da seção 6 do `00-mapa`, se houver diferença. Registrar a mudança com data.

- [ ] **Step 4: Verificar**

Run: `Select-String -Path CLAUDE.md,PLANO-RUST.md -Pattern 'clean-room|Cancelada'`
Expected: ao menos uma ocorrência em cada arquivo.

- [ ] **Step 5: Pedir autorização de commit ao usuário** (todos os arquivos desta task).

---

## Depois deste plano

Cada fase C2–C6 ganha plano próprio, escrito a partir das especificações listadas no `00-mapa` e com uma task de implementador limpo por especificação. O prompt do implementador (`tools/re/prompts/implementador.md`) nasce no plano da C2, que é a primeira fase com Rust de produto.
