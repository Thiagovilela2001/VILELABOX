# Detector experimental de vocal

O blackmesa/VILELABOX analisa a presença provável de voz em segundo plano ao
carregar uma faixa. A skin RbxOriginal mostra uma faixa rosa na waveform completa
e na waveform que acompanha a reprodução. O indicador abaixo da tonalidade mostra
`VOZ…` durante a análise, `VOCAL` em um trecho detectado, `SEM VOZ` fora dele e
`VOZ ?` se o analisador estiver indisponível (passe o mouse para ver o motivo e
como resolver) e `VOZ –` se a análise foi cancelada. Silêncio não é marcado como vocal.
Reabra o aplicativo depois da atualização para carregar o novo executável e a skin.

É uma implementação própria inspirada na função do rekordbox. O modelo
`vd_param.pb` da instalação extraída não foi reutilizado: não foi possível
estabelecer um formato executável compatível com o ambiente Linux disponível.
Não há alegação de equivalência com a precisão ou o algoritmo do rekordbox.

## Funcionamento

O analisador nativo aproveita a decodificação de áudio existente, mistura os dois
canais para mono e escreve um arquivo temporário, que um processo Python local
analisa. Não usa o microfone nem envia áudio para serviços externos. O
temporário é removido ao concluir ou cancelar a análise.

### Motor Demucs (preferido)

Com o pacote `demucs` instalado no Python do detector, a voz é **separada** do
resto da música pelo modelo aberto Demucs (`htdemucs`, licença MIT, roda local na
CPU) e a marcação vem da energia da voz isolada, em quadros de 50 ms: é voz
quando a voz isolada passa de −42 dBFS e não está mais de 24 dB abaixo da
mistura. Uma votação de 250 ms tira oscilações, pausas de até 0,8 s entre
palavras ficam no mesmo trecho e trechos menores que 0,4 s são descartados. A
faixa é separada em blocos de 30 s (com 2 s de contexto) para limitar a memória.
Usa metade dos núcleos da CPU (`VILELABOX_VOCAL_THREADS` muda) e leva cerca de
um terço da duração da faixa num desktop; por isso o tempo-limite padrão é 900 s.

Medido nas faixas do usuário (2026-10-12):

| Faixa | Classificador antigo | Demucs |
| --- | --- | --- |
| `pulse_na_base_beat` (instrumental) | 93% marcado como voz | 0% |
| `pulse_na_base` (mesmo beat com voz) | — | 51%, em 9 trechos nos versos |
| `sophierefix` | 59% | 17%, um trecho no início |

Instalação usada: `~/.venv/bin/python -m pip install --index-url
https://download.pytorch.org/whl/cpu torch torchaudio` e depois `pip install
demucs`. Os pesos do modelo são baixados uma vez e ficam no cache local.

### Classificador (reserva)

Sem o Demucs, roda a rede pequena de `res/vocals/model.json` sobre
características espectrais em janelas de cerca de um segundo. Ela erra muito em
música mixada (marca beats instrumentais densos como voz) e fica só como reserva.
O motor usado entra na identidade do cache: ao instalar o Demucs, as faixas
analisadas pelo classificador são reanalisadas na próxima carga.

Resultados ficam em `~/.mixxx/analysis/vocals/*.json`. O identificador incorpora
caminho, tamanho, data do áudio, modelo e código Python; alterações invalidam o
resultado antigo. Na próxima carga de uma faixa já analisada, o cache é usado.
A análise não modifica os arquivos de música. Falha do helper produz estado de
indisponibilidade, separado de um resultado instrumental.

### Interpretador, diagnóstico e cancelamento

O Python do detector vem, nesta ordem: do campo **Python do detector de vocal**
em Preferências → Detecção de BPM (`[Vilelabox],VocalPython`), da variável
`VILELABOX_VOCAL_PYTHON`, de `~/.venv/bin/python` ou do `python3` do sistema.
Ele precisa de NumPy e SciPy.

Cada falha tem um diagnóstico próprio, mostrado no tooltip do indicador:
Python não encontrado, arquivos do detector ausentes, dependência ausente
(com o nome do módulo e o comando de instalação), tempo esgotado, falha do
detector (última linha do erro), resultado inválido e análise cancelada.

O detector tem até 120 s por faixa (`[Vilelabox],VocalTimeoutSeconds`). A espera
é feita em fatias de 100 ms e consulta o pedido de parada da thread de análise:
cancelar a análise ou fechar o aplicativo encerra o processo em cerca de
100 ms, sem esperar o tempo-limite, e apaga o áudio temporário. No Linux, o
processo auxiliar também é encerrado se o Mixxx cair (`PR_SET_PDEATHSIG`).
O detector roda na fila de análise, depois dos outros analisadores da faixa;
uma fila própria não foi necessária até aqui e não foi medida.
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
