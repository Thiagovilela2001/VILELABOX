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

`vilelabox-core` e a biblioteca própria existem (P3.3); a interface ainda
conecta direto ao leitor ANLZ até a migração do P5.
