# Especificações do rekordbox

Este diretório reúne requisitos verificáveis para o port do rekordbox em Rust.
Na fase atual, REA e Ghidra apoiam diretamente a investigação e quem implementa
pode consultar a decompilação e a evidência. A separação de papéis *clean-room*
é uma etapa futura, após a disponibilização pública do aplicativo.

As especificações continuam sendo documentos concisos de comportamento e
formato, com proveniência identificada. Material bruto de análise permanece
na área privada ignorada pelo Git; os dossiês resumidos e revisáveis podem
ficar em `spec/rekordbox/evidence/`.

## Modelo de especificação

Cada especificação segue esta ordem:

- título no formato `# NN — Subsistema`;
- escopo e fora do escopo;
- glossário com nomes próprios do VILELABOX;
- comportamento, incluindo estados, fluxos e regras;
- formatos, quando houver;
- constantes funcionais, com valor, unidade e função;
- casos de teste, com entrada, saída esperada e origem do resultado;
- perguntas abertas.

## Permitido

- comportamento observável: entradas, saídas, estados, transições e erros;
- formatos binários e de banco: layout de campos, tags, tipos e versões necessários para interoperabilidade;
- constantes funcionais: limites, tolerâncias, valores mágicos de formato, tamanhos de janela e valores padrão de configuração;
- ordem de operações e regras de decisão descritas em prosa ou tabela;
- casos de teste com entrada concreta e saída esperada, de preferência obtida executando o programa real.

## Proibido

- pseudocódigo, trechos C ou C++ do Ghidra e estrutura de laços ou ramos que espelhe uma função;
- nomes internos de funções, classes e variáveis, salvo quando fazem parte de um formato, como nome de tabela, tag `PQTZ` ou chave XML;
- tabelas numéricas com mais de 16 valores copiadas do binário sem aprovação explícita do usuário;
- endereços ou referências internas ao REA/Ghidra no corpo da especificação;
  ficam no dossiê de evidência ligado à regra funcional;
- assuntos de licença, login, nuvem, streaming, telemetria ou contorno de proteção e DRM.

Para aprovar uma tabela numérica longa, coloque imediatamente antes da linha ou do bloco:

```text
<!-- tabela-aprovada: AAAA-MM-DD -->
```

Na raiz do repositório, valide todas as especificações com:

```text
cargo run -p xtask -- spec-lint spec/rekordbox
```
