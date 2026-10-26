# Arquitetura

O VILELABOX é uma aplicação Rust independente. O código de produto não
depende de outro aplicativo de DJ.

## Regra de dependências

```text
vilelabox-app
  -> vilelabox-core   (vilelabox-cli usa o mesmo núcleo)
    -> domínio e serviços de biblioteca
      -> vilelabox-storage
      -> vilelabox-types

adaptadores de formato
  -> vilelabox-anlz, vilelabox-xml, vilelabox-catalog (somente leitura)
  -> futuro export-usb
  -> vilelabox-reference (somente validação)
```

- A interface não acessa bancos nem formatos diretamente.
- O núcleo não depende de Slint, SQLite ou APIs de áudio.
- Formatos externos entram por adaptadores e nunca definem o modelo interno.
- Escrita em dados reais é proibida durante pesquisa e testes.
- Implementações clean-room leem apenas especificações aprovadas em
  `spec/rekordbox/`.

## Estado transitório

`vilelabox-core` e a biblioteca própria existem (P3.3). Desde o P5.1 o app
grava cues, grade e identidade do áudio pelo núcleo (`vilelabox-app/src/nucleo.rs`);
a coleção e as playlists da tela ainda vêm do leitor ANLZ e do XML até o P5.2.
