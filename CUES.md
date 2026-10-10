# Cues automáticos de frases

Em cada deck, abra PADS e habilite AUTO CUES (`auto_phrase_cues`, persistente,
padrão 0). A preferência de análise oferece o padrão para decks que ainda não
possuem valor salvo; mudanças exigem reabrir o aplicativo para esses defaults.

Uma análise confiável pode criar Intro e Outro se esses tipos ainda não existem,
e hot cues Up (amarelo), Chorus (vermelho) e Down (azul) nos oito primeiros slots
vazios. Cues existentes não são alterados; LOCK impede geração e desfazer.
Os cues são gerados no máximo uma vez por carregamento, quando a primeira análise
confiável fica disponível. Editar o grid recalcula as frases, mas não gera cues de
novo. Ao recarregar a faixa, um hot cue com o mesmo rótulo a até uma batida de
distância conta como já existente, então pequenos ajustes de grid não duplicam
cues. Sem frases confiáveis, nada é criado. A análise em lote guarda frases; a
criação ocorre ao carregar a faixa em um deck com AUTO CUES habilitado ou ao
concluir a análise nesse deck. A criação roda na thread da interface, fora do
processamento de áudio.

DESFAZER CUES (`auto_phrase_cues_undo`) remove somente cues criados no carregamento
atual que conservam posição, duração, rótulo e cor. Cues editados depois são
preservados. O histórico dura até trocar a faixa ou fechar o programa.

Os cues usam a persistência nativa do Mixxx; a análise de frases fica no cache
separado. Não é necessário migrar o esquema do banco.

Teste: `AnalyzerPhrasesTest.AutomaticCuesPreserveUserSlotsLockAndUndoEdits`.
