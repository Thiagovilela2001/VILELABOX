# Detector experimental de vocal

O blackmesa/VILELABOX analisa a presença provável de voz em segundo plano ao
carregar uma faixa. A skin RbxOriginal mostra uma faixa rosa na waveform completa
e na waveform que acompanha a reprodução. O indicador abaixo da tonalidade mostra
`VOZ…` durante a análise, `VOCAL` em um trecho detectado, `SEM VOZ` fora dele e
`VOZ ?` se o analisador estiver indisponível. Silêncio não é marcado como vocal.
Reabra o aplicativo depois da atualização para carregar o novo executável e a skin.

É uma implementação própria inspirada na função do rekordbox. O modelo
`vd_param.pb` da instalação extraída não foi reutilizado: não foi possível
estabelecer um formato executável compatível com o ambiente Linux disponível.
Não há alegação de equivalência com a precisão ou o algoritmo do rekordbox.

## Funcionamento

O analisador nativo aproveita a decodificação de áudio existente, mistura os dois
canais para mono e escreve um arquivo temporário. Um processo Python local extrai
características espectrais em janelas de aproximadamente um segundo, com passos
de 256 ms, e executa uma rede neural pequena. Suavização, histerese e união de
intervalos próximos produzem as marcações. Não usa o microfone nem envia áudio
para serviços externos. O temporário é removido ao concluir ou cancelar a análise.

Resultados ficam em `~/.mixxx/analysis/vocals/*.json`. O identificador incorpora
caminho, tamanho, data do áudio, modelo e código Python; alterações invalidam o
resultado antigo. Na próxima carga de uma faixa já analisada, o cache é usado.
A análise não modifica os arquivos de música. Falha do helper produz estado de
indisponibilidade, separado de um resultado instrumental.

O ambiente atual usa `/home/thiago/.venv/bin/python` com NumPy e SciPy. É possível
selecionar outro Python compatível pela variável `VILELABOX_VOCAL_PYTHON`.
Recursos: `res/vocals/vocal_detector.py` e `res/vocals/model.json`.

## Limites e validação

O modelo foi treinado localmente com samples de vocais, instrumentos e misturas
sintéticas disponíveis em `~/Documentos/drumkit`. Os rótulos foram inferidos das
pastas dos packs; não constituem anotação humana dos instantes cantados.
Dois packs inteiros foram reservados para validação, sem seus arquivos no treino.
Treino: 6.188 janelas. Validação: 1.690 janelas. Com limiar 0,90: precisão de vocal
86,7%, recall de vocal 88,9% e especificidade instrumental 77,8%. Esses números
medem a classificação das janelas dessa amostra, não precisão em músicas completas
nem dos limites de entrada/saída de voz. Sons de sintetizador e vocais processados
podem produzir erros. Vocais discretos ou muito curtos podem não aparecer.

`tools/vocals/train.py` reproduz o treino com os samples locais; o manifesto registra
os arquivos usados. `validation.json` registra os resultados por classe.
`validate_sequence.py` exercita a inferência por blocos com três sequências de
instrumental → mistura com vocal → instrumental, usando arquivos reservados.
Não é um benchmark representativo de faixas comerciais.

Verificação automatizada:

```
~/.venv/bin/python tools/vocals/test_detector.py
QT_QPA_PLATFORM=offscreen build/mixxx-test --resource-path "$PWD/res" \
  --gtest_filter='VocalAnalysisTest.*:AnalyzerVocalsTest.*'
```

Os testes verificam silêncio em 44,1/48 kHz, limites de segmentos, equivalência
numérica do modelo exportado, rejeição de cache inválido e execução real do helper
pelo analisador C++ seguida de recarga pelo cache, invalidação após mudança do áudio
e transições do indicador no widget do deck. Reprodução no desktop com saída de
áudio não pôde ser verificada no ambiente de execução.

Backup anterior às alterações em `backups-vocals/20261009-092929`.

Validação em 09/10/2026: compilação concluída, 3 testes Python e 8 testes C++
aprovados, incluindo regressões de grid e menu de tonalidade. Relatório C++ em
`/tmp/vilelabox-vocal-tests.xml`, log em `/tmp/vilelabox-vocal-tests.log`.
