# Tonalidade no VILELABOX

Clique na tonalidade no cabeçalho de cada deck (ou em TOM ▾, se não houver tom detectado).

- Travar tonalidade (MT): mantém o tom ao alterar o BPM. O botão TOM ao lado do jog controla a mesma trava.
- Diminuir/Aumentar 1 semitom: transpõe a música e ativa a trava.
- Restaurar tom original: zera a transposição e mantém a trava ativada.
- Escolher tom: oferece as 12 tônicas no modo maior ou menor original. A escolha ativa a trava e remove desvios de afinação. Se não houver tonalidade analisada, use os semitons ou analise a faixa primeiro.

A transposição preserva o modo maior/menor. Não converte uma música maior em menor.

## Implementação e referência

O motor de áudio continua sendo o do Mixxx, com seu processamento de pitch e tempo. A referência local do rekordbox foi o arquivo `/home/thiago/Downloads/rekordbox_extracted/MidiMappings/DDJ-XP1.midi.csv`, linhas 28–37: MasterTempo, SemitoneDown, SemitoneUp, KeySync e KeyReset. Esses recursos mostram os comandos disponíveis, sem estabelecer o algoritmo proprietário de áudio. As ferramentas nativas de decompilação REA não estavam disponíveis nesta sessão.

Arquivos alterados: `src/widget/wkey.h`, `src/widget/wkey.cpp`, `src/test/enginebuffertest.cpp` e os arquivos `decks/track_info.xml` e `decks/flags.xml` da skin instalada `~/.mixxx/skins/RbxOriginal`.

O seletor é opt-in na skin via `<EditableKey>true</EditableKey>`. Outras skins mantêm o comportamento anterior.

Backup anterior às alterações: `backups-key-controls/20261009-083454/`.

## Verificação

Compilar: `cmake --build build --parallel 4`.

Testes: `QT_QPA_PLATFORM=offscreen build/mixxx-test --gtest_filter='KeyUtilsTest.*:EngineBufferTest.*'`.

O teste KeyMenuSelectsAndHoldsEveryTonic aciona o menu real de Qt para cada uma das 12 tônicas nos modos maior e menor, partindo de uma afinação desviada, e verifica a trava e a manutenção do tom ao alterar a velocidade. Teste auditivo no dispositivo real ainda necessário: o ambiente de execução não teve acesso ao servidor de áudio nem à tela da sessão.

Reabra o VILELABOX pelo atalho existente para carregar o executável recompilado.
