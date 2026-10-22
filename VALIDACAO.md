# Validação musical em sessão real (Plano 3, fase 6)

Roteiro para registrar a primeira rodada com faixas reais. Preencha durante a
sessão; erros também são resultado. Nada aqui foi executado ainda.

## Preparação

1. Compile e verifique: `tools/verify-mod.sh`.
2. Preferências → Detecção de BPM: ligue **Analisar frases musicais**,
   **Analisar energia para faixas relacionadas** e confira o
   **Python do detector de vocal**.
3. Escolha 20 faixas variando gênero, duração, presença de vocal e andamento
   (inclua ao menos 2 com andamento variável e 2 instrumentais).
4. Analise as 20 (biblioteca → menu da faixa → análise).
5. Exporte as frases: `python3 tools/phrases/export_csv.py > frases.csv`.

## Configuração usada

| Item | Valor |
| --- | --- |
| Data | |
| Commit (`git log -1 --oneline`) | |
| Interface de áudio / controladora | |
| Taxa de amostragem e tamanho do buffer | |
| Skin | |
| Python do detector | |

## Amostra

| # | Faixa | Gênero | Duração | Vocal? | Andamento variável? |
| --- | --- | --- | --- | --- | --- |
| 1 | | | | | |
| 2 | | | | | |
| 3 | | | | | |
| 4 | | | | | |
| 5 | | | | | |
| 6 | | | | | |
| 7 | | | | | |
| 8 | | | | | |
| 9 | | | | | |
| 10 | | | | | |
| 11 | | | | | |
| 12 | | | | | |
| 13 | | | | | |
| 14 | | | | | |
| 15 | | | | | |
| 16 | | | | | |
| 17 | | | | | |
| 18 | | | | | |
| 19 | | | | | |
| 20 | | | | | |

## Registro por área

Para cada área: faixa, o que era esperado, o que aconteceu e o erro (em ms,
batidas, compassos ou descrição).

| Área | Faixa | Esperado | Observado | Erro |
| --- | --- | --- | --- | --- |
| Grid e compasso 1 | | | | |
| Limites de frase | | | | |
| Rótulos de frase | | | | |
| Vocal (falsos positivos e negativos) | | | | |
| Cues automáticos e edições preservadas | | | | |
| BarSync | | | | |
| PhraseSync | | | | |
| Mix Point Link | | | | |
| Slicer | | | | |
| Release BREAK / ECHO / SPIN | | | | |
| Carregar faixa tocando (estalos?) | | | | |
| Relacionadas durante a reprodução | | | | |

## Frases: como medir

Fluxo de trabalho:
1. **Exportar:** gere o arquivo de anotação com `python3 tools/phrases/export_csv.py > frases.csv`.
2. **Anotar ouvindo:** preencha as colunas `limite_ok` (`sim`/`não`, indicando se o início da frase está a até um compasso do início real) e `tipo_correto` (`INTRO`, `UP`, `CHORUS`, `DOWN`, `VERSE`, `OUTRO`).
3. **Calcular métricas:** execute `python3 tools/phrases/score_csv.py frases.csv`. O script calcula a acurácia por faixa e no total, conferindo a meta de pelo menos 20 faixas anotadas e ≥ 80% de limites corretos.
4. **Registrar:** copie as porcentagens obtidas para a tabela de resultado abaixo.

- **Limites:** `sim` ÷ total de linhas preenchidas. Meta inicial: pelo menos 80%.
- **Rótulos:** linhas em que `tipo_detectado` = `tipo_correto` ÷ total preenchido, contado à parte dos limites.
- Linhas em branco são ignoradas e computadas à parte.
- O resultado vale para a amostra avaliada.

## Resultado

| Critério | Resultado |
| --- | --- |
| 20 faixas registradas, com erros | |
| Limites de frase a até 1 compasso | % |
| Rótulos corretos | % |
| Problemas de áudio (estalos, cortes) | |
| Problemas de interface (travamentos, lentidão) | |
| Itens abertos para correção | |

Uso ao vivo só depois desta validação auditiva e dos critérios combinados para
o seu equipamento.
