# Design: funcionamento do rekordbox em Rust por clean-room

Data: 2026-10-23. Status: decisões do usuário registradas; aguardando
aprovação final do documento.

## Intenção

O usuário quer reproduzir em Rust a estrutura de funcionamento inteira do
rekordbox 7.2.8: subsistemas, formatos, análise, motor de áudio, interface e
controladoras. O resultado é código próprio do VILELABOX, não um clone. A
fonte de comportamento é a análise do `rekordbox.exe` no Ghidra
(`rekord_clone.gpr` / `rekord_clone.rep`, programa `rekordbox.exe`).

O método é **clean-room**, por decisão do usuário: quem lê o binário nunca
escreve o código, e quem escreve o código nunca vê o binário. A ponte entre os
dois lados é uma especificação escrita em prosa.

Este documento complementa o `PLANO-RUST.md`. As crates, gates (G0–G8),
Definition of Done e regras de agents de lá continuam valendo. Muda a fonte de
referência: onde houver especificação do rekordbox, ela substitui o Mixxx como
oráculo de comportamento. O Mixxx continua sendo o produto executável até G8.

## Objetivos

- Produzir especificações de comportamento de cada subsistema do rekordbox.
- Implementar essas especificações em Rust, com testes e comparação
  diferencial contra o rekordbox real (arquivos que ele gera, ou o programa
  rodando).
- Manter rastreabilidade: cada crate/módulo Rust aponta para a especificação
  que implementa; cada especificação aponta para a evidência do lado sujo
  (endereço/função no Ghidra) sem reproduzir o código.

## Não-objetivos

- Marca: nenhum logo, nome de produto ou marca da AlphaTheta/Pioneer
  DJ/rekordbox na interface, no binário, nos ícones ou nos textos do
  aplicativo. A interface em si segue a do rekordbox (ver decisões).
- Verificação de licença, assinatura, login, nuvem (Cloud Library Sync),
  serviços de streaming e telemetria: não são especificados nem implementados.
- Contornar proteção, DRM ou licenciamento do rekordbox.
- Substituir agora o Mixxx ou alterar `src/`, `lib/`, `src/controllers/`.

## Anti-objetivos (falha mesmo se funcionar)

- Pseudocódigo, código descompilado ou estrutura de função traduzida para Rust,
  ainda que renomeada.
- Agent do lado limpo que leu, mesmo uma vez, material do lado sujo.
- Crate que compila mas não tem teste diferencial ou justificativa registrada.
- Qualquer escrita no `master.db`, na pasta `Pioneer` ou no perfil real do
  Mixxx durante testes.

## Modelo clean-room

### Papéis

| Papel | Lê | Escreve | Nunca |
| --- | --- | --- | --- |
| Exportador (ferramenta) | `rekord_clone.rep` via Ghidra headless | `third_party/rekordbox/re/export/` | — |
| Analista (lado sujo) | export do Ghidra, binários e DLLs do rekordbox, especificações existentes | rascunho de especificação no scratchpad | código Rust |
| Auditor de contaminação | rascunho da especificação + export (para comparar) | parecer aprovado/reprovado | editar a especificação |
| Implementador (lado limpo) | `spec/rekordbox/`, `rust/`, `tests/reference/`, docs públicos | patch Rust + testes (handoff) | abrir `third_party/rekordbox/re/`, `rekord_clone.*`, Ghidra ou DLLs |
| Agent principal | `spec/rekordbox/`, `rust/`, docs públicos, pareceres do auditor | aplica especificação aprovada e patches limpos | abrir lado sujo; alterar lógica de patch limpo (correções voltam ao implementador) |

O agent principal atua como integrador neutro: aplica, formata e roda testes.
Se um patch limpo precisar de correção de lógica, a correção volta ao
implementador com a falha do teste, não é feita pelo principal.

### Barreira física

- Lado sujo: `third_party/rekordbox/re/` (já ignorado por `third_party/rekordbox/`
  no `.gitignore`; confirmar) e o projeto Ghidra. Nunca versionados.
- Ponte: `spec/rekordbox/NN-nome.md`, versionado.
- Lado limpo: `rust/`, `tests/reference/`.
- Prompt de todo implementador lista os caminhos proibidos e exige declarar no
  handoff "não li material do lado sujo".

### O que uma especificação pode conter

Permitido:

- comportamento observável: entradas, saídas, estados, transições, erros;
- formatos binários e de banco: layout de campos, tags, tipos, versões
  (necessários para interoperabilidade);
- constantes funcionais: limites, tolerâncias, valores mágicos de formato,
  tamanhos de janela, valores padrão de configuração;
- ordem de operações e regras de decisão descritas em prosa ou tabela;
- casos de teste: entrada concreta e saída esperada, de preferência obtida
  rodando o rekordbox real.

Proibido:

- pseudocódigo, trechos C/C++ do Ghidra, estrutura de laços/ramos espelhando
  a função;
- nomes internos de funções, classes e variáveis, salvo quando são parte de um
  formato (nome de tabela, tag `PQTZ`, chave XML);
- tabelas numéricas longas copiadas do binário (mais de 16 valores) sem
  aprovação explícita do usuário, registrada na própria especificação;
- endereços ou referências ao Ghidra no corpo. Ficam só num apêndice
  "Evidência", lido pelo auditor e removido antes da entrega ao lado limpo.

### Modelo de especificação

```text
# NN — Subsistema
Escopo / fora do escopo
Glossário (nomes próprios do VILELABOX)
Comportamento (estados, fluxos, regras)
Formatos (se houver)
Constantes funcionais (valor, unidade, função)
Casos de teste (entrada -> saída esperada, origem do esperado)
Perguntas abertas
--- (removido antes do handoff limpo)
Evidência (função/endereço Ghidra, string, DLL)
```

### Ciclo por subsistema

1. Exportar ou atualizar o export do Ghidra para as funções do subsistema.
2. Analista escreve o rascunho da especificação.
3. Auditor compara com o export e aprova ou reprova (motivo por linha).
4. Principal grava `spec/rekordbox/NN-*.md` sem o apêndice de evidência; a
   evidência fica em `third_party/rekordbox/re/evidence/NN.md`.
5. Implementador limpo recebe só o caminho da especificação e o pacote de
   tarefa do `PLANO-RUST.md`, e entrega patch + testes.
6. Revisor confere o patch contra a especificação (não contra o Ghidra).
7. Principal aplica, roda `cargo fmt/clippy/test` e o diferencial contra
   saídas reais do rekordbox.
8. Registro em `IMPLEMENTACAO.md` no formato do `PLANO-RUST.md`.

## Ferramental do lado sujo

- Ghidra 12.1.4 em `C:\Tools\ghidra_12.1.4_PUBLIC`, JDK 21 (Adoptium) no PATH.
- Script headless (`tools/re/`, código próprio, versionável) que abre
  `rekord_clone.gpr` em modo leitura e exporta para
  `third_party/rekordbox/re/export/`: lista de funções, árvore de classes
  (RTTI MSVC), strings com referências, imports por DLL, grafo de chamadas e
  descompilação por função sob demanda (não o binário inteiro de uma vez).
- O projeto Ghidra não é modificado pelo script. Renomeações e comentários
  feitos pelo usuário na GUI são preservados.

## Fases

Cada fase usa a lista final aprovada em `spec/rekordbox/00-mapa.md`.

| fase | ids de especificação | Rust | oráculo | gate |
| --- | --- | --- | --- | --- |
| C0 Infraestrutura | modelo de spec, checklist do auditor | `xtask spec-lint` (export do Ghidra em Java, lado sujo) | — | — |
| C1 Mapa geral | `00-mapa` | nenhum | — | — |
| C2 Formatos | `10-master-db`, `11-anlz`, `12-xml`, `13-export-usb` | leitores e escritores em crates de formato | cópia de catálogo, USBANLZ, XML e mídia gerados pelo programa real | G1/G2 |
| C3 Análise | `20-bpm-grid`, `21-frases`, `22-tonalidade`, `23-energia`, `24-waveform`, `25-vocal` | `vilelabox-dsp`, `vilelabox-analysis` | USBANLZ real | G3/G4 |
| C3b Biblioteca e núcleo | `26-biblioteca`, `27-preparacao` | `vilelabox-storage`, `vilelabox-library`, `vilelabox-core` | catálogo copiado e sessão observada | G5 |
| C5 Interface | `40-telas`, `41-waveform-render`, `42-biblioteca-ui` | `vilelabox-ui` (layout do rekordbox, sem logos) | capturas operadas pelo usuário | G6 |
| C4 Motor de áudio | `30-deck`, `31-sync`, `32-loops-cues`, `33-mixer-fx`, `34-quantize`, `35-separacao` | `vilelabox-audio` | renderização offline e gravação operada pelo usuário | G7 |
| C6 Controladoras e rede | `50-midi`, `51-hid`, `52-mapeamentos`, `53-link-rede` | `vilelabox-midi` (rede: crate a definir no plano da C6) | arquivos de fábrica, controladora e captura de rede com equipamento operados pelo usuário | G8 |

Alteração registrada em 2026-10-24: o `00-mapa` v2 (aprovado na 4ª rodada de
auditoria) propôs `35-separacao` e `53-link-rede`; o agent encaixou-as em C4
e C6, como o próprio mapa sugere. Iluminação, vídeo e gravação aparecem como
subsistemas no binário e aguardam decisão do usuário (dentro ou fora do
produto).

Alteração registrada em 2026-10-23: C1 confirmou os ids e as dependências da
lista acima; os títulos, subsistemas, dependências e oráculos normativos ficam
na seção “Lista de especificações C2–C6” de `spec/rekordbox/00-mapa.md`.

C1 é a primeira entrega útil: um documento que mostra como o rekordbox inteiro
funciona e define a lista final de especificações. A tabela acima pode mudar
depois do C1. A mudança é registrada neste documento.

## Restrições

- Todas as regras do `CLAUDE.md`: português do Brasil, evidência com
  `arquivo:linha`, separar implementado/planejado, sem commit/push sem pedido,
  agents de análise não editam o repo.
- Leitura de `master.db`, `USBANLZ` e da instalação só por cópia em diretório
  temporário. `master.db` é SQLCipher; a forma de abri-lo é decisão listada
  abaixo.
- Nenhuma crate exige material do rekordbox para compilar ou rodar o CI.
  Testes que usam dados reais são opcionais e pulam quando o dado não existe.
- Dependências Rust novas seguem a auditoria de licença do `PLANO-RUST.md`.

## Decidido pelo usuário

- Código em Rust, por clean-room, sem clone (2026-10-23).
- Ordem das fases conforme proposta do agent (2026-10-23).
- O rekordbox é a única referência de comportamento, e o Mixxx sai do caminho
  (2026-10-23). Consequências:
  - quando o rekordbox e um recurso atual do VILELABOX discordam, vence o
    rekordbox; gates G3/G4 são medidos de novo contra ele;
  - a Fase 5 do `PLANO-RUST.md` (modo sombra dentro do Mixxx, ABI C/FFI) é
    cancelada; o aplicativo Rust nasce independente;
  - `vilelabox-mixxx-compat` vira só importador opcional de bibliotecas Mixxx;
  - o fork C++ em `src/` não é apagado agora. A remoção continua sendo tarefa
    própria com autorização (`PLANO-RUST.md` R10.3).

## Decidido pelo agent (pode ser revisto)

- Barreira por diretório + regras de prompt, com auditor de contaminação.
- Especificações versionadas em `spec/rekordbox/`; evidência fora do git.
- Principal como integrador neutro, sem corrigir lógica de patch limpo.
- Limite de 16 valores para tabelas numéricas sem aprovação.
- Exclusão de licença, nuvem, streaming e telemetria.
- Ordem de execução C0, C1, C2, C3, C3b, C5, C4, C6, seguindo a regra do
  `PLANO-RUST.md` de que áudio RT só começa com G5 e G6 aprovados.
- Arte na C5: a versão distribuída usa ícones e imagens redesenhados pelo
  projeto no mesmo estilo. A arte original pode ser carregada como tema local a
  partir da instalação do usuário (permitido pelo `CLAUDE.md`), desligada por
  padrão e sem logos. A fonte principal da especificação de layout é a
  observação do programa rodando (capturas de tela, medição de regiões como no
  R7.0), não o Ghidra.
- Escritores da C2 provam round-trip interno e leitura pelos leitores
  independentes já existentes (Kaitai `rekordbox_anlz.ksy`). A aceitação por
  rekordbox ou CDJ real é validação manual do usuário, registrada em
  `VALIDACAO.md`, e não bloqueia o gate.
- Oráculos que exigem o programa rodando ou hardware (gravações da C4, revisão
  visual da C5, controladora da C6) são operados pelo usuário. A tarefa do
  agent é preparar o roteiro de captura. Até a captura existir, o item fica
  "bloqueado: aguardando usuário".

## Deixado ao construtor

- Linguagem do script Ghidra (Java ou PyGhidra).
- Divisão fina de crates dentro da arquitetura do `PLANO-RUST.md`.
- Granularidade das especificações dentro de cada fase.
- Formato dos testes diferenciais, reaproveitando `vilelabox-reference`.

## Decididas pelo usuário após a verificação do construtor (2026-10-23)

- **Tudo em Rust.** Todo código de produto, ferramentas, testes diferenciais e
  `xtask` são Rust. Os scripts Python atuais (`tools/rekordbox/sync.py`,
  `tools/reference/*.py`) são substituídos por equivalentes Rust ao longo das
  fases. Única exceção: o script de export do Ghidra (C0), porque o Ghidra só
  executa scripts Java/Python. Ele é ferramenta do lado sujo e não faz parte do
  produto.
- **Separação clean-room real.** O agent principal nunca abre o export do
  Ghidra, a evidência nem o projeto `rekord_clone.*`. Só subagents analistas
  leem o lado sujo; um subagent auditor separado revisa cada especificação;
  implementadores e principal veem apenas `spec/rekordbox/`. Para o código
  Rust, esta regra prevalece sobre a permissão do `CLAUDE.md` de copiar
  material do rekordbox; essa permissão continua valendo para material local
  (arte, modelos, mapeamentos) fora do código. Isso substitui a linha
  "Agent principal" da tabela de papéis: o principal não lê o lado sujo em
  momento algum.
- **`master.db`:** leitura de uma cópia, com a chave SQLCipher já publicada
  pela comunidade (usada pelo pyrekordbox). A chave não é extraída do
  binário. Nunca há escrita no banco original.
- **Interface do rekordbox:** a UI Rust reproduz layout, proporções, cores,
  fluxo de telas e comportamento da interface do rekordbox. Especificações da
  C5 podem conter isso. Proibido: logos e marcas da AlphaTheta (ver
  não-objetivos).
- **Modelos de ML do rekordbox:** algoritmos próprios em Rust no produto. Os
  modelos Keras/ONNX do rekordbox rodam só localmente, como oráculo opcional
  de comparação, nunca no produto nem no CI.
