# Plano 3 — Consolidação do VILELABOX

Data: 12/10/2026. Base atual: Mixxx 2.5.6.

## Objetivo

Consolidar os recursos existentes antes de ampliar o software: reduzir riscos
de latência, melhorar o desempenho da biblioteca, tornar as análises mais
controláveis e validar o comportamento musical em uso real.

Este documento propõe as próximas entregas. Não registra essas entregas como
implementadas nem altera a decisão anterior de manter a base 2.5.6 e adiar stems.

## Situação (2026-10-12)

Fases 1 a 5 implementadas, com testes e commits registrados em
[IMPLEMENTACAO.md](IMPLEMENTACAO.md). Fase 6 preparada
([VALIDACAO.md](VALIDACAO.md), `tools/phrases/export_csv.py`) e pendente da
sessão com faixas reais. Fase 7 aguarda a decisão de ancoragem. Nenhuma
declaração de uso ao vivo: falta a validação auditiva.

## Estado de partida

Compassos, edição de grid, tonalidade, vocal, frases, cues automáticos,
Mix Point Link, faixas relacionadas e pads já estão implementados. As correções
B1–B5 do plano anterior estão registradas como concluídas.

Na análise que originou este plano, 51 testes direcionados passaram usando o
executável existente em `build/mixxx-test`. Não houve compilação nova, teste de
latência nem validação auditiva. Portanto, esse resultado não comprova que a
versão esteja pronta para uso ao vivo.

## Sequência de entregas

| Fase | Entrega | Prioridade | Dependência |
| --- | --- | --- | --- |
| 1 | Carregamento de frases seguro para o áudio | Alta | Nenhuma |
| 2 | Faixas relacionadas sem trabalho pesado na interface | Alta | Nenhuma |
| 3 | Verificação unificada dos módulos próprios | Alta | Acompanhar as fases 1 e 2 |
| 4 | Análise vocal com cancelamento e diagnóstico | Alta | Nenhuma |
| 5 | Persistência de frases e controle de memória | Média | Preservar o modelo de estado da fase 1 |
| 6 | Validação musical com faixas reais | Alta | Primeira rodada após as fases 1–3 |
| 7 | Correção manual de frases | Média | Resultados da fase 6 |

## Fase 1 — Carregamento de frases seguro para o áudio

### Evidência

`src/engine/controls/phrasecontrol.cpp`, em `trackLoaded()`, ainda cria um estado
com `std::make_shared` e consulta `PhraseAnalysisStore::get()`, que utiliza
`QReadLocker`. Esse caminho é chamado pelo callback de áudio.

### Implementação

- Preparar o estado de frases fora do callback e publicar um estado consistente
  para consumo pelo motor.
- Evitar alocações e aquisição de locks no caminho de carregamento de frases
  executado pelo áudio.
- Verificar também a liberação do estado anterior: sua destruição não deve
  produzir trabalho pesado no callback.
- Garantir que atualizações atrasadas de uma faixa não atinjam outra carregada
  no mesmo deck.
- Revisar a sincronização dos campos compartilhados, incluindo o estado de
  aplicação dos cues automáticos.

### Critérios de conclusão

- Os testes existentes de frases, cues, grid e sincronização continuam passando.
- Há verificação específica de trocas rápidas de faixa e atualizações atrasadas.
- O caminho revisado não aloca nem acessa o lock do cache no callback.
- O carregamento durante reprodução é validado com buffers pequenos, registrando
  configuração, interrupções de áudio e limitações do ambiente.

## Fase 2 — Faixas relacionadas com execução assíncrona

### Evidência

`src/library/related/relatedtracksfeature.cpp`, em `refresh()`, consulta toda a
biblioteca, carrega as energias, calcula as pontuações, ordena os resultados e
cria as células da tabela. A atualização é disparada ao carregar faixas.

`src/analyzer/analyzerenergy.cpp` também carrega o conjunto de energias para
verificar se uma única faixa já foi analisada.

### Implementação

- Executar consulta e pontuação fora da thread da interface, com conexão SQL
  pertencente à thread que a utiliza.
- Aplicar um limite configurável de resultados; proposta inicial: 100 sugestões.
- Descartar resultados antigos quando a referência mudar durante o cálculo.
- Evitar recálculos desnecessários quando a seção estiver oculta.
- Substituir a criação massiva de células por um modelo de tabela, se a medição
  justificar essa mudança.
- Consultar a energia de uma faixa por identidade, evitando carregar toda a
  tabela durante cada inicialização do analisador.

### Decisão de interface

Proposta: oferecer seleção explícita do deck de referência e opção para fixá-la.
Hoje, carregar uma faixa em qualquer deck pode mudar a referência.

### Critérios de conclusão

- A interface permanece responsiva durante consultas em coleções de diferentes
  tamanhos, com tempos e tamanhos registrados.
- Trocas rápidas de referência não exibem resultados obsoletos.
- A pontuação mantém as regras atuais de tom e BPM, incluindo metade e dobro.
- A quantidade de resultados respeita o limite definido.

## Fase 3 — Verificação unificada do mod

### Evidência

O filtro de testes de `build-bars.sh` cobre compassos e grid, mas não inclui todo
o conjunto de testes de frases, vocal e relacionadas.

### Implementação

- Disponibilizar um comando reproduzível para verificar todos os módulos próprios.
- Incluir testes de frases, cues, energia, relacionadas, vocal, Mix Point e pads,
  além dos testes atuais de compassos e grid.
- Incluir a verificação Python da ponte XML e os verificadores existentes das
  skins quando aplicáveis.
- Integrar o comando ao fluxo de CI, respeitando dependências e limitações de
  cada ambiente.
- Registrar falhas conhecidas da base separadamente, com evidência, sem ocultar
  regressões novas.

### Critérios de conclusão

- Um comando documentado executa a verificação do mod e retorna falha quando
  uma verificação obrigatória falha.
- Os relatórios permitem identificar o teste e o módulo afetado.
- Uma compilação nova é validada antes de declarar a entrega concluída.

## Fase 4 — Análise vocal com cancelamento e diagnóstico

### Evidência

`src/analyzer/analyzervocals.cpp`, em `storeResults()`, aguarda o processo Python
por até 120 segundos. A espera ocupa a thread de análise; não se trata de uma
espera no callback de áudio.

### Implementação

- Tornar a execução do detector cancelável durante a espera.
- Conectar cancelamento e encerramento do aplicativo à finalização do processo
  auxiliar e à limpeza dos arquivos temporários.
- Exibir diagnósticos distintos para Python ausente, dependências ausentes,
  timeout, falha do detector e resultado inválido.
- Permitir configurar o interpretador sem depender apenas de variável de ambiente
  ou do caminho padrão `~/.venv/bin/python`.
- Avaliar uma fila própria de processos se as medições mostrarem que o detector
  prejudica a vazão das demais análises.

### Critérios de conclusão

- Cancelar uma análise não exige aguardar o timeout completo.
- Encerrar o aplicativo não deixa processos auxiliares ativos.
- Falhas de dependência têm mensagens acionáveis na interface.
- O reaproveitamento e a invalidação do cache continuam funcionando.

## Fase 5 — Persistência de frases e controle de memória

### Evidência

`src/analyzer/phraseanalysisstore.cpp` já agrupa alterações com atraso de dois
segundos e grava em segundo plano. Entretanto, `save()` ainda serializa todas
as características junto com as frases.

Gravações independentes no `QThreadPool` também merecem revisão: a possibilidade
de uma versão antiga substituir uma nova é um risco de concorrência identificado
na leitura, ainda não reproduzido.

Os stores mantêm resultados em memória; é necessário medir o crescimento durante
sessões longas antes de escolher uma política de descarte.

### Implementação

- Separar características imutáveis dos resultados derivados do grid.
- Serializar ou versionar as escritas por faixa para preservar o resultado mais
  recente.
- Definir o comportamento ao fechar o aplicativo com gravações pendentes.
- Manter invalidação por identidade do arquivo e versão do algoritmo.
- Medir memória ao analisar e carregar muitas faixas; implementar limite ou
  descarte quando necessário.
- Prever leitura compatível ou regeneração segura dos caches antigos.

### Critérios de conclusão

- Ajustes sucessivos de grid não regravam as características imutáveis.
- Uma escrita antiga não sobrescreve o resultado mais recente.
- Fechamento, cache inválido e falha de escrita têm comportamento definido.
- O consumo de memória é medido e sua política documentada.

## Fase 6 — Validação musical em sessão real

### Amostra e registro

Retomar a avaliação de 20 faixas reais prevista no plano original, incluindo
diferenças de gênero, duração, presença de vocal e variação de andamento.
Registrar faixa, configuração, resultado esperado, resultado observado e erro.

Avaliar separadamente:

- Grid e primeiro tempo do compasso.
- Limites de frases e rótulos atribuídos.
- Presença de vocal e falsos positivos.
- Cues automáticos e preservação das edições do usuário.
- Alinhamento de BarSync, PhraseSync e Mix Point.
- Slicer e transições de BREAK, ECHO e SPIN.
- Responsividade ao carregar faixas e consultar relacionadas durante reprodução.

### Critérios de conclusão

- Os resultados das 20 faixas estão registrados, incluindo os erros.
- A meta inicial de limites de frases é pelo menos 80% a até um compasso da
  referência manual; esse resultado vale apenas para a amostra avaliada.
- Qualidade dos rótulos é medida separadamente da qualidade dos limites.
- Problemas de áudio e interface geram itens reproduzíveis para correção.
- A declaração de uso ao vivo fica condicionada à validação auditiva e aos
  critérios acordados para o equipamento utilizado.

## Fase 7 — Correção manual de frases

### Motivação

`src/track/phrases.cpp` classifica segmentos por regras de energia, graves e vocal.
A correção manual permite aproveitar a análise mesmo quando limites ou rótulos
estiverem errados.

### Implementação proposta

- Editar limites e rótulos, com alinhamento ao grid.
- Oferecer desfazer e restauração do resultado automático.
- Persistir as correções separadamente do resultado automático.
- Preservar correções durante reanálise, conforme política explícita.
- Definir o efeito da edição sobre cues automáticos, PhraseSync e Mix Point.

### Decisão necessária antes da implementação

Escolher como ancorar as correções: tempo absoluto, posição musical ou modelo
híbrido. Definir também como ajustes de BPM e SET 1 afetam essas correções.

### Critérios de conclusão

- Correções sobrevivem ao fechamento e à reabertura da faixa.
- Reanalisar não apaga silenciosamente o trabalho manual.
- Desfazer restaura o estado anterior.
- Cues do usuário são preservados e mudanças nos recursos dependentes são
  previsíveis e documentadas.

## Decisões de direção do projeto

| Decisão | Proposta | Quando resolver |
| --- | --- | --- |
| Base do software | Manter 2.5.6 durante a consolidação | Já registrada; revisar quando stems virar prioridade |
| Stems | Fazer avaliação própria de migração para base com suporte nativo | Fora deste ciclo |
| Referência de relacionadas | Seleção de deck e opção de fixação | Fase 2 |
| Novos algoritmos de análise | Medir erros reais antes de substituir os atuais | Após a primeira rodada da fase 6 |
| Correções de frases | Definir ancoragem e preservação em reanálise | Antes da fase 7 |
| Pronto para uso ao vivo | Exigir testes funcionais, medição e sessão auditiva | Antes de declarar estabilidade |

## Documentação e manutenção

- Atualizar `PLANO-2.md` para distinguir registros históricos do estado atual.
  Há referências antigas à publicação e à decisão de B5.
- Registrar os remotos atuais: `origin` aponta para o Mixxx oficial e `vilelabox`
  aponta para o repositório do mod. A existência do remoto não comprova que todas
  as verificações de publicação foram concluídas.
- Registrar entregas e evidências em `IMPLEMENTACAO.md` e manter os documentos
  de cada módulo coerentes com o comportamento final.
- Fazer commits por entrega revisável e executar verificações proporcionais às
  mudanças.

## Primeira entrega recomendada

Concluir as fases 1, 2 e 3: carregamento de frases sem alocações e locks no áudio,
relacionadas com processamento assíncrono e verificação unificada do mod.
Em seguida, realizar a primeira sessão de validação real e usar seus resultados
para ajustar as entregas seguintes.
