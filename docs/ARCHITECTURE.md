# Arquitetura

O VILELABOX é uma aplicação Rust independente. O código de produto não
depende de outro aplicativo de DJ.

## Regra de dependências

```text
vilelabox-app
  -> futuro vilelabox-core
    -> domínio e serviços de biblioteca
      -> vilelabox-storage
      -> vilelabox-types

adaptadores de formato
  -> vilelabox-anlz
  -> futuros master-db, xml e export-usb
  -> vilelabox-reference (somente validação)
```

- A interface não acessa bancos nem formatos diretamente.
- O núcleo não depende de Slint, SQLite ou APIs de áudio.
- Formatos externos entram por adaptadores e nunca definem o modelo interno.
- Escrita em dados reais é proibida durante pesquisa e testes.
- Implementações clean-room leem apenas especificações aprovadas em
  `spec/rekordbox/`.

## Estado transitório

O protótipo atual ainda conecta a interface diretamente ao leitor ANLZ. Essa
ligação será substituída quando `vilelabox-core` e a biblioteca própria
existirem.
