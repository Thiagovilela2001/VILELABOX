# Referências canônicas

Este diretório contém resultados esperados usados na validação diferencial
do VILELABOX.

- `synthetic/`: matriz determinística, sem gravações externas.
- `synthetic/wavs/`: WAVs recriados localmente e ignorados pelo Git.
- `rekordbox/`: exportações locais de cópias de arquivos ANLZ, ignoradas pelo
  Git porque podem identificar a biblioteca do usuário.

Os JSONs seguem `ReferenceTrack`, definido em
`vilelabox-reference/src/schema.rs`, com `schema_version = 1`. Campo ausente
significa que a fonte não fornece aquela informação.

## Recriar a matriz sintética

```powershell
python tools/reference/make_synthetic.py
cargo test -p vilelabox-reference --test fixtures
```

## Exportar referências reais

Trabalhe apenas sobre cópias de `USBANLZ` e dos arquivos de áudio:

```powershell
python tools/reference/export_rekordbox_anlz.py `
  --anlz-dir C:\copia\USBANLZ `
  --music-dir C:\copia\Musicas `
  --output tests\reference\rekordbox

cargo test -p vilelabox-reference --test fixtures
```

Os arquivos de áudio não são versionados. A identidade da faixa é o SHA-256
do conteúdo de áudio; o índice não guarda artista nem título.
