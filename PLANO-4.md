# Plano 4 — Próximos passos (etapas 0 a 3)

Plano de implementação para execução por um agente de código (Gemini). Cobre as
etapas 0 a 3 do [README.md](README.md#plano-de-desenvolvimento): preparação da
validação real, saneamento de CI e docs, inventário técnico, conjunto de
referência e a primeira crate Rust (ponte `rekordbox.xml`).

## Antes de começar

1. Leia [CLAUDE.md](CLAUDE.md). As regras de lá valem integralmente para este
   plano: português do Brasil, evidência antes de afirmar, separar implementado
   de planejado, não alterar `lib/`, `src/controllers/` e `res/controllers/`,
   nunca copiar nada do instalador do rekordbox, preservar a GPL, **sem commit,
   push ou PR sem pedido do usuário**.
2. Leia [README.md](README.md) (estado atual) e [IMPLEMENTACAO.md](IMPLEMENTACAO.md).
3. Ambiente: a máquina do usuário é **Windows**, sem `cargo` instalado e sem
   build do mod (os scripts são bash/Linux). Consequências:
   - Não rode `build-bars.sh`, `verify-mod.sh` nem `cmake` aqui. Mudanças em
     C++ ou em scripts bash ficam marcadas como **"não executado"** no relatório.
   - Python 3 está disponível (`python`). Testes Python podem rodar.
   - Para a etapa 3, peça ao usuário para instalar o Rust via
     [rustup](https://rustup.rs) antes de começar; não instale nada sozinho.

## Como trabalhar

- Execute as tarefas **na ordem**. Cada tarefa tem um critério de pronto; só
  avance quando ele for cumprido ou quando o bloqueio estiver registrado.
- Uma tarefa = um conjunto pequeno de arquivos. Não refatore fora do escopo.
- Ao fim de cada tarefa, acrescente uma linha em
  [IMPLEMENTACAO.md](IMPLEMENTACAO.md), na seção "Plano 4" (crie a seção), com:
  tarefa, arquivos alterados, comando de verificação e resultado real
  (passou / falhou / não executado).
- Se algo no plano contradizer o código, **o código vence**: registre a
  divergência e siga o código.
- Decisões marcadas como **[DECISÃO DO USUÁRIO]** não são suas. Pare nelas e
  pergunte.

---

## Etapa A — Saneamento (pré-requisito de tudo)

### A1. Corrigir a referência `2.5.6` inexistente

O clone não tem tag `2.5.6`. A base upstream é o commit `3ebac449e7`.

- `tools/verify-mod.sh:83`: trocar `2.5.6..HEAD` por `3ebac449e7..HEAD`.
- `tools/check-third-party-assets.sh:8` (comentário de uso): mesma troca.
- `tools/verify-mod.sh:31`: o filtro gtest repete `VilelaboxPreferencesTest.*`;
  remover a duplicata.

**Pronto quando:** `grep -n "2\.5\.6\.\." tools/` não retorna nada. Marcar
"não executado" para o script em si.

### A2. Docs desatualizados

| Arquivo | Problema | Correção |
| --- | --- | --- |
| `VOCALS.md` | Seção "Interpretador" diz timeout de 120 s; o padrão é 900 s (`src/analyzer/analyzervocals.cpp:153`) | Corrigir para 900 s, citando a opção em Configurações |
| `IMPLEMENTACAO.md` | Cita remotos `origin` = Mixxx oficial e `vilelabox`, branches `blackmesa` e `vilelabox-publico` | Acrescentar nota datada: hoje há só `origin` = `Thiagovilela2001/VILELABOX` e branch `main`. Não apagar o histórico |
| `tools/rekordbox/sync.py` (docstring, linhas 7-12) | Usa o nome antigo `mixxx_rekordbox_bridge.py` | Trocar por `sync.py`. Sem mudar lógica |
| `CUES.md`, `PLANO.md` | Verificar se algum ainda cita hot cues "Up/Chorus/Down" como estado atual | Só corrigir `CUES.md` se divergir de `src/track/autophrasecues.cpp:53-57`; `PLANO.md` é histórico, não mexer |

**Pronto quando:** `python tools/rekordbox/test_sync.py` passa (4 testes) e cada
correção cita a linha de código que a sustenta.

### A3. CI herdado do Mixxx

Os workflows `build.yml`, `changelog.yml`, `checks.yml`, `develop.yml`,
`download_cleanup.yml`, `flatpak.yml`, `git.yml`, `labeler.yml`,
`pr-command.yml`, `pre-commit.yml`, `release.yml`, `stale.yml` e
`sync_branches.yml` são do Mixxx e estão ativos no fork sem adaptação
(`develop.yml` consulta PRs de `mixxxdj/mixxx`; `build.yml` roda macOS e
Windows, sem suporte no mod).

**[DECISÃO DO USUÁRIO]**, apresente as opções e espere a resposta:

- (a) mover os herdados para `.github/workflows-mixxx/` (o GitHub ignora) e
  manter só `vilelabox.yml` e, se o usuário quiser, `pre-commit.yml`;
- (b) manter os arquivos e adicionar `if: github.repository == 'mixxxdj/mixxx'`
  nos jobs;
- (c) não mexer.

Depois de aplicado: pedir ao usuário para rodar `vilelabox.yml` manualmente
(`workflow_dispatch`) e registrar o resultado. **Pronto quando:** existe um
registro da primeira execução do `vilelabox.yml` (passou ou falhou, com link).

### A4. Canais de contribuição

`CONTRIBUTING.md`, `.github/ISSUE_TEMPLATE/config.yml` e os templates apontam
para Zulip e para o fórum do Mixxx.

- Renomear `CONTRIBUTING.md` para `CONTRIBUTING.mixxx.md` (mesmo padrão de
  `README.mixxx.md`) e criar um `CONTRIBUTING.md` curto em pt-BR: issues no
  GitHub do VILELABOX, mudanças pequenas, `tools/verify-mod.sh` obrigatório,
  regra de terceiros do rekordbox.
- `ISSUE_TEMPLATE/config.yml`: trocar o link do fórum pelas issues do repo.
- `README.md`, seção "Contribuições": atualizar a frase que diz que esses
  arquivos são os do Mixxx.

**Pronto quando:** `grep -rn "zulip\|mixxx.org/forums" CONTRIBUTING.md .github/ISSUE_TEMPLATE`
não retorna nada.

---

## Etapa B — Preparar a validação real (etapa 0 do README)

A validação de ouvido é do usuário. Seu papel é deixar a medição automática.

### B1. Medidor da anotação de frases

`tools/phrases/export_csv.py` gera o CSV com as colunas `limite_ok`
(`sim`/`não`), `tipo_correto` e `observacao`, preenchidas à mão.

Criar `tools/phrases/score_csv.py`:

- entrada: o CSV anotado (argumento);
- saída: por faixa e no total, a porcentagem de `limite_ok = sim` entre as
  linhas preenchidas, e a acurácia de tipo (`tipo_detectado == tipo_correto`);
- linhas sem anotação são ignoradas e contadas à parte;
- código de saída 0 se o total de limites for ≥ 80% com pelo menos 20 faixas
  anotadas, 1 caso contrário (meta de [FRASES.md](FRASES.md) e
  [VALIDACAO.md](VALIDACAO.md));
- só biblioteca padrão.

Teste: `tools/phrases/test_score_csv.py` (unittest) com um CSV pequeno criado no
próprio teste, cobrindo: tudo certo, abaixo da meta, menos de 20 faixas e
linhas vazias. Incluir no `tools/verify-mod.sh`, ao lado dos outros testes
Python.

**Pronto quando:** `python tools/phrases/test_score_csv.py` passa.

### B2. Roteiro

Em [VALIDACAO.md](VALIDACAO.md), acrescentar o fluxo de uso:
`export_csv.py` → anotar ouvindo → `score_csv.py` → copiar o resultado para a
tabela. Não preencha nenhuma tabela de resultado.

---

## Etapa C — Inventário técnico (etapa 1 do README)

Criar `INVENTARIO.md` na raiz, gerado por leitura do código (sem build).

Para cada recurso (compassos, grid, faixa de BPM, tonalidade, vocal, frases,
cues automáticos, Mix Point, relacionadas, energia, pads, configurações,
waveform RGB, skins, ponte XML), uma tabela com:

| Campo | Conteúdo |
| --- | --- |
| Arquivos próprios | criados pelo mod (`git diff --name-status 3ebac449e7 HEAD`, status `A`) |
| Arquivos do Mixxx alterados | status `M`, com o trecho alterado resumido |
| Testes | suíte gtest ou script Python e onde fica |
| Dependências do Mixxx | classes e sistemas usados (ex.: `ControlObject`, `TrackPointer`, `mixxx::Beats`, `UserSettings`, `AnalyzerTrack`) |
| Dados persistidos | onde ficam (`mixxxdb.sqlite`, `beats.proto`, `analysis/...`, chaves `[Vilelabox]`) |
| Acoplamento | **baixo** (lógica pura, portável), **médio** (depende de dados do Mixxx, não do motor) ou **alto** (depende do motor de áudio ou da UI) |
| Candidato a Rust | sim / depois / não, com justificativa de uma linha |

No fim, uma seção "Ordem de extração sugerida", do menor para o maior
acoplamento, citando a evidência de cada classificação.

**Pronto quando:** todo recurso da tabela do README aparece no inventário e
toda afirmação cita `arquivo:linha` ou commit.

---

## Etapa D — Conjunto de referência (etapa 2 do README)

Objetivo: resultados esperados que qualquer reimplementação (Rust ou não) possa
comparar, **sem versionar áudio com direitos autorais**.

### D1. Exportador de resultados

Criar `tools/reference/export_reference.py`:

- lê um perfil do Mixxx (padrão `~/.mixxx`, argumento opcional) **somente
  leitura** (abrir SQLite com `mode=ro` via URI);
- para cada faixa analisada, gera um JSON com: hash SHA-256 do arquivo de áudio
  (identificador, nunca o caminho absoluto como chave), duração, BPM, primeira
  batida, referência de compasso (do protobuf do grid em `src/proto/beats.proto`),
  tom, frases (de `analysis/phrases/`), cues e hot cues (tabela `cues`) e
  presença de vocal (de `analysis/vocals/`);
- grava em `tests/reference/<hash>.json` (diretório novo) e um `index.json`.

Decodificar o protobuf exige o pacote `protobuf` ou parser manual. **Prefira
parser manual mínimo** dos campos usados, para manter só a biblioteca padrão;
se não for viável, registre e pergunte ao usuário antes de adicionar
dependência.

### D2. Casos sintéticos

Os testes atuais já geram áudio sintético em C++. Criar
`tools/reference/make_synthetic.py` que gera WAVs determinísticos (cliques em
BPM fixo, mudança de BPM, intro sem batida) com a biblioteca padrão (`wave`,
`struct`, `math`) e o JSON esperado para cada um. Os WAVs **não** são
versionados; o script os recria. Os JSONs são versionados em
`tests/reference/synthetic/`.

### D3. Tolerâncias

Documentar em `tests/reference/README.md` as tolerâncias de comparação: grid em
ms, compasso 1 exato, limites de frase em compassos, BPM em ±0,01. Partir dos
números de [GRID.md](GRID.md) e [FRASES.md](FRASES.md); citar a fonte de cada um.

**Pronto quando:** `make_synthetic.py` roda no Windows e gera os JSONs; o
exportador tem um teste unittest com um SQLite mínimo criado no teste (mesmo
padrão de `tools/rekordbox/test_sync.py`). Exportar faixas reais fica para o
usuário rodar no Linux; registrar como pendente.

---

## Etapa E — Primeira crate Rust: ponte rekordbox (etapa 3 do README)

**[DECISÃO DO USUÁRIO] antes de começar:** o README registra que migrar a base
para o Mixxx 2.6 (stems) compete com a reescrita em Rust. Confirme com o
usuário que a etapa E deve seguir. Se não, pare aqui.

Escopo: reimplementar em Rust o que `tools/rekordbox/sync.py` faz, como
biblioteca + CLI, **sem substituir o script** até a equivalência ser provada.

### E1. Workspace

```text
rust/
├── Cargo.toml                 # workspace
└── vilelabox-rekordbox/
    ├── Cargo.toml
    ├── src/lib.rs             # modelo + leitura/escrita
    ├── src/main.rs            # CLI: export / import / status
    └── tests/
```

- Licença no `Cargo.toml`: `GPL-2.0-or-later`.
- Dependências: escolher crates maduras para XML e SQLite (sugestão:
  `quick-xml` e `rusqlite` com `bundled`). Justificar cada uma em uma linha no
  `rust/README.md`. Nada de dependência que não seja usada.
- Adicionar `rust/target/` ao `.gitignore`.

### E2. Comportamento a reproduzir

Ler `tools/rekordbox/sync.py` inteiro antes. Reproduzir:

- `export`: tabelas `library`, `track_locations`, `cues`, `Playlists`,
  `PlaylistTracks` → `DJ_PLAYLISTS` / `COLLECTION` / `TEMPO` / `POSITION_MARK` /
  `PLAYLISTS`, com conversão de cores (`int_to_rgb`/`rgb_to_int`) e caminhos
  (`path_to_file_url`/`file_url_to_path`);
- `import`: as mesmas garantias de segurança do Python: recusar com o Mixxx
  aberto, bloquear escritor concorrente, backup consistente incluindo WAL em
  `backups-rekordbox/` antes de gravar, transação única com rollback;
- `status`: as mesmas contagens.

Caminhos padrão iguais aos do script. **Nunca** apontar testes para o
`~/.mixxx` real.

### E3. Testes de equivalência

- Portar os 4 testes de `test_sync.py` para Rust.
- Teste de ida e volta: SQLite mínimo de fixture → `export` → `import` em banco
  limpo → mesmas faixas, cues e playlists.
- Teste de equivalência com o Python: para a mesma fixture, o XML do Rust e o
  do `sync.py` devem ser iguais depois de normalizar (ordem de atributos e
  espaços). Script de comparação em `rust/vilelabox-rekordbox/tests/`.

**Pronto quando:** `cargo test` passa no Windows e o teste de equivalência
confirma XML igual ao do `sync.py`. Registrar em IMPLEMENTACAO.md. Não
adicionar ao CI nem trocar o script Python sem pedido do usuário.

---

## Fora do escopo deste plano

- Validação de ouvido e preenchimento de VALIDACAO.md (cabe ao usuário).
- Migração para o Mixxx 2.6 e stems.
- Etapas 4 a 7 do README (análise, interface, motor de áudio e controladoras
  em Rust).
- Qualquer mudança em `lib/`, controladoras ou esquema do `mixxxdb.sqlite`.

## Entrega final

Ao terminar (ou ao parar num bloqueio), entregue ao usuário:

1. Lista de tarefas com status: feito / bloqueado (motivo) / decisão pendente.
2. Arquivos criados e alterados.
3. Comandos de verificação e resultado real de cada um; o que não pôde rodar
   no Windows, marcado como "não executado".
4. Divergências entre este plano e o código encontradas no caminho.
