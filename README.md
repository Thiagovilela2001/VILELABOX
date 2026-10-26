# VILELABOX

Aplicativo de DJ independente escrito em Rust, com comportamento e fluxos de
trabalho especificados por um processo clean-room a partir do rekordbox.

O projeto não distribui código, marcas, imagens ou binários da AlphaTheta ou da
Pioneer DJ. `rekordbox` é uma marca da AlphaTheta Corporation e é citado somente
para interoperabilidade e descrição da referência comportamental.

## Estado atual

O projeto ainda é experimental e não é um software de performance pronto.

- `vilelabox-app`: protótipo desktop em Slint. Lê análises ANLZ em modo somente
  leitura e exibe biblioteca, BPM, compassos e frases. Ainda não reproduz áudio.
- `vilelabox-anlz`: leitor e exportador dos contêineres e seções ANLZ.
- `vilelabox-xml` e `vilelabox-catalog`: XML de intercâmbio e leitura do
  catálogo (cópia sem criptografia).
- `vilelabox-storage`: banco SQLite próprio com migrações versionadas.
- `vilelabox-library`: importação, varredura, busca, playlists e identidade.
- `vilelabox-core`: núcleo headless com comandos, eventos e decks.
- `vilelabox-cli`: CLI de desenvolvimento sobre o núcleo.
- `vilelabox-dsp`: primitivas da análise de BPM recuperadas com evidência.
- `vilelabox-audio`: decodificação, transporte e mixagem.
- `vilelabox-types`: tipos e invariantes do domínio musical.
- `vilelabox-reference`: esquema canônico e comparação diferencial.
- `xtask`: ferramentas de validação do processo clean-room.

As especificações ficam em `spec/rekordbox/`. Dados reais, binários e material
de engenharia reversa ficam fora do Git em `third_party/rekordbox/`.

## Desenvolvimento

Requer a toolchain fixada em `rust-toolchain.toml`.

```powershell
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo run -q -p xtask -- spec-lint spec/rekordbox
```

Fluxo de preparação sem interface (cada chamada reabre o mesmo banco):

```powershell
cargo run -q -p vilelabox-cli -- C:\dados\vilelabox.db importar dir C:\caminho\musica
cargo run -q -p vilelabox-cli -- C:\dados\vilelabox.db pesquisar texto
cargo run -q -p vilelabox-cli -- C:\dados\vilelabox.db cue <faixa> <frame>
cargo run -q -p vilelabox-cli -- C:\dados\vilelabox.db deck <faixa>
```

Para abrir o protótipo com uma cópia de uma pasta `USBANLZ`:

```powershell
cargo run -p vilelabox-app -- C:\caminho\para\USBANLZ
```

Nunca execute testes contra a pasta real de dados. Copie os arquivos para uma
pasta temporária e trabalhe somente sobre a cópia.

## Direção

1. concluir os formatos ANLZ, XML, catálogo e exportação USB;
2. implementar análise musical e waveform;
3. construir biblioteca e núcleo de preparação independentes;
4. ligar a interface ao núcleo;
5. implementar transporte, mixer, sync, efeitos e áudio em tempo real;
6. adicionar MIDI, HID, rede, iluminação, vídeo e gravação.

O código legado anterior à aplicação Rust foi preservado na branch
`legacy-mixxx` e não faz parte da arquitetura atual.

## Licença

GPL-2.0-or-later. Consulte `LICENSE`.
