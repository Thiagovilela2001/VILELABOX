# VILELABOX — Módulos Rust

Espaço de trabalho (workspace) para a reescrita gradual dos componentes do VILELABOX em Rust, iniciando pelas partes desacopladas do motor de áudio.

## Crates

- `vilelabox-rekordbox`: biblioteca e CLI para leitura, escrita e sincronização de arquivos `rekordbox.xml` com o banco `mixxxdb.sqlite`, escrita para reproduzir `tools/rekordbox/sync.py`.

**Estado: experimental.** Compila e passa em `cargo test`, `clippy -D warnings`
e `fmt --check` no Windows (Rust 1.99.1, 2026-10-22). A exportação gera o mesmo
XML que o `sync.py` num banco de teste (`tests/equivalence.py`). Falta testar com
um XML e um banco reais; `sync.py` continua sendo a ponte em uso.

## Justificativa de Dependências

- `rusqlite = { version = "0.31", features = ["bundled", "backup"] }`: acesso ao SQLite com driver embutido estático e suporte nativo à API de backup do SQLite (incluindo WAL).
- `quick-xml = "0.36"`: leitura e escrita de XML por eventos, com escape e unescape de entidades.
- `percent-encoding = "2.3"`: codificação e decodificação determinística de URLs `file://localhost/...` conforme padrão Pioneer.
- `chrono = "0.4"`: formatação consistente de datas e geração de carimbos de data/hora para os arquivos de backup.
- `tempfile = "3.10"` (dev-dependency): isolamento de fixtures e bancos de teste temporários na suíte de testes.

## Compilação e Testes

Requer Rust 1.75+ instalado via [rustup](https://rustup.rs):

```bash
cd rust/vilelabox-rekordbox
cargo build
cargo test
```

Equivalência com o Python (cria um banco de teste, exporta com os dois e compara):

```bash
python3 rust/vilelabox-rekordbox/tests/equivalence.py
```
