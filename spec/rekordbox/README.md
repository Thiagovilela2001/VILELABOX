# Especificações do rekordbox

Este diretório é a ponte clean-room entre a análise de comportamento do rekordbox e a implementação limpa do VILELABOX. Somente especificações aprovadas entram aqui; material de análise e evidência permanece fora do repositório.

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
- endereços ou referências ao Ghidra no corpo; ficam somente num apêndice "Evidência", lido pelo auditor e removido antes da entrega ao lado limpo;
- assuntos de licença, login, nuvem, streaming, telemetria ou contorno de proteção e DRM.

Para aprovar uma tabela numérica longa, coloque imediatamente antes da linha ou do bloco:

```text
<!-- tabela-aprovada: AAAA-MM-DD -->
```

Em `rust/`, valide todas as especificações com:

```text
cargo run -p xtask -- spec-lint ../spec/rekordbox
```
