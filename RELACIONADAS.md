# Faixas relacionadas

A seção **Relacionadas** da barra lateral compara a biblioteca com a última
faixa carregada em um deck. Abra a seção novamente para atualizar os resultados
após análise de energia ou edição de metadados. Duplo clique carrega uma faixa
pelo fluxo normal da biblioteca. A tabela mostra pontuação, motivo, artista,
título, BPM, tom Camelot e energia; a busca geral não se aplica a essa tabela.

O filtro exige tom compatível pela roda de quintas do `KeyUtils` (mesmo tom,
vizinhos e relativo maior/menor) e BPM a até 6%, considerando metade e dobro.
Até 85 pontos vêm desses critérios; até 15 vêm da proximidade de energia.
Energia ausente é identificada como pendente e não é tratada como zero.
Gênero e artista não influem na pontuação nesta versão.

Habilite **Analisar energia para faixas relacionadas** nas preferências de BPM
e analise/carregue as faixas. O analisador usa loudness integrada do libebur128,
densidade de ataques em blocos de 50 ms e razão de graves/energia total. A nota
própria de 1 a 10 combina 60%, 25% e 15% dessas métricas normalizadas. Não é um
padrão de energia musical nem uma medida de danceability validada.

As notas ficam em `analysis/blackmesa.sqlite` do perfil, separadas de
`mixxxdb.sqlite`, com identidade por caminho, tamanho, data e versão do algoritmo.
Não altera músicas nem injeta dados no banco principal. A interface lê os
metadados nativos da biblioteca. Análises ficam desligadas por padrão.

Testes: `RelatedTracksTest.*`.
