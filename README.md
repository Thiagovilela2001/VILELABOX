<p align="center">
  <img src="res/images/icons/scalable/apps/blackmesa.svg" width="96" alt="">
</p>

# VILELABOX

Mod pessoal do [Mixxx](https://mixxx.org) 2.5.6 com foco em compassos,
frases e edição de grid, com uma interface e um conjunto de recursos inspirados
no rekordbox. O nome interno do build é **blackmesa**.

> Projeto experimental e não oficial. Não tem relação com a equipe do Mixxx nem
> com a AlphaTheta (rekordbox, Pioneer DJ). rekordbox é marca da AlphaTheta
> Corporation.

---

## O que muda em relação ao Mixxx 2.5.6

Cada recurso tem um documento próprio, com controles, uso, limites e testes.

| Recurso | O que faz | Documento |
| --- | --- | --- |
| **Compassos** | Contagem `compasso:tempo` em 4/4, linhas de compasso e de frase (8 compassos) na waveform e no overview, compasso 1 automático, SET 1 manual, sync por compasso ao dar PLAY, saltos e loops de N compassos | [COMPASSOS.md](COMPASSOS.md) |
| **Grid** | Painel de edição por deck (GRID, deslocar, SET 1, LOCK, BPM ±0,01, TAP, ½×, 2×, UNDO) e o detector "VILELABOX — Grid preciso e primeiro tempo", com modos de BPM fixo e variável | [GRID.md](GRID.md) |
| **Tonalidade** | Menu no cabeçalho do deck: trava de tom, ±1 semitom, voltar ao tom original e escolher a tônica diretamente | [TONALIDADE.md](TONALIDADE.md) |
| **Vocal** | Detector próprio de presença de voz, mostrado como faixa rosa na waveform e indicador VOCAL / SEM VOZ | [VOCALS.md](VOCALS.md) |
| **Frases** | Segmentação em Intro, Up, Chorus, Down, Verse e Outro, alinhada aos compassos; PhraseSync opcional | [FRASES.md](FRASES.md) |
| **Cues automáticos** | Intro/Outro e hot cues de Up, Chorus e Down gerados a partir das frases, só em slots vazios, com desfazer | [CUES.md](CUES.md) |
| **Mix Point Link** | Arma o deck parado para começar sozinho quando o outro deck chega ao ponto de saída | [MIXPOINT.md](MIXPOINT.md) |
| **Faixas relacionadas** | Seção na biblioteca que sugere faixas por tom compatível (Camelot), BPM (±6%, com metade e dobro) e uma nota de energia própria | [RELACIONADAS.md](RELACIONADAS.md) |
| **Pads na tela** | Slicer, KeyShift, Keyboard e Release FX (BREAK, ECHO, SPIN), em painel opcional na skin | [PADS.md](PADS.md) |
| **Skins** | `RbxOriginal` (layout de dois decks no estilo do modo Performance) e `RbxFlowBars` (gráficos gerados por scripts próprios) com todos os controles do mod | [res/skins/](res/skins/) |
| **Ponte XML** | Exporta e importa faixas, hot cues, loops, grid e playlists em `rekordbox.xml` | [tools/rekordbox/sync.py](tools/rekordbox/sync.py) |

### Compatibilidade com o Mixxx original

- **O banco de dados não muda de esquema.** O `mixxxdb.sqlite` continua abrindo
  num Mixxx sem o mod.
- **A referência de compasso fica no protobuf do grid**, num campo novo. Um
  Mixxx sem o mod ignora esse campo; reanalisar ou regravar o grid nele pode
  apagá-lo.
- **Frases, vocal e energia ficam em caches separados**, dentro do perfil
  (`analysis/phrases/`, `analysis/vocals/`, `analysis/blackmesa.sqlite`).
- **Os controles novos só existem neste executável.** Abrir as skins no Mixxx
  oficial carrega o layout, mas os botões do mod ficam sem efeito.
- **Todos os recursos novos são opt-in** ou vêm com padrão conservador
  (análises de frases e energia, cues automáticos e PhraseSync começam desligados).
- O build desliga a interface QML experimental e a exportação Engine Prime
  (dispensa a dependência libdjinterop).

---

## Como o VILELABOX usa o rekordbox

O rekordbox entra no projeto de três formas. **Nenhum arquivo do rekordbox é
distribuído aqui e nenhum código ou modelo dele é executado.**

### 1. Como catálogo de recursos

Um instalador do rekordbox foi examinado localmente, sem executar o programa,
para saber **quais** funções existem e como estão organizadas. As fontes foram:

- os **mapeamentos MIDI** (`MidiMappings/*.midi.csv`, 69 controladoras), que
  listam pelo nome todas as funções que uma controladora pode acionar;
- a **lista de modos de pad** (`pad/*.pad.csv`);
- os **nomes dos ícones** da interface, que revelam painéis e botões.

A partir disso, cada recurso foi implementado do zero sobre o motor do Mixxx:

| Função no rekordbox | Implementação no VILELABOX |
| --- | --- |
| `GridAdjust`, `GridSlide`, `GridAdjustHalf/Double`, ícones `Grid_Button_*` | Painel de grid (GRID.md) |
| `MasterTempo`, `SemitoneUp/Down`, `KeyReset`, `KeySync` | Menu de tonalidade (TONALIDADE.md) |
| `PhraseSync` | Análise de frases e PhraseSync (FRASES.md) |
| `MixPointLink`, `MixPointLinkSilent`, `MixPointSelectNext/Prev` | Mix Point Link (MIXPOINT.md) |
| `RelatedTracks`, ícones de sugestão de faixas | Faixas relacionadas (RELACIONADAS.md) |
| Modos de pad `Slicers`, `KeyShift`, `Keyboard`, `*ReleaseFX*` | Pads na tela (PADS.md) |
| Detecção de vocal da waveform | Detector de vocal (VOCALS.md) |

Os algoritmos são próprios ou vêm de projetos abertos (Queen Mary, para o grid).
Os resultados **não são equivalentes** aos do rekordbox e não há essa alegação.

### 2. Como formato de troca

- `tools/rekordbox/sync.py` lê e escreve o **XML de coleção do rekordbox**
  (`COLLECTION`, `DJ_PLAYLISTS`). Isso permite levar faixas, hot cues, loops,
  grid e playlists entre os dois programas.
- A importação recusa rodar com o Mixxx aberto, faz backup consistente do
  banco antes de gravar e desfaz tudo se falhar.
- A leitura de pendrives exportados pelo rekordbox (`lib/rekordbox-metadata/`)
  é recurso do Mixxx original, mantido sem alterações.

### 3. Como referência visual

A skin `RbxOriginal` reproduz a **disposição** do modo Performance
(2 decks horizontais, jog, mixer central e biblioteca embaixo). Os ícones dela
(`res/skins/RbxOriginal/rb/`) são desenhos próprios, simples, feitos para
este repositório.

### O que o projeto não faz

- **Não usa os modelos do rekordbox.** O instalador traz redes neurais (grid e
  downbeat em TensorFlow, cues e embeddings em ONNX, separação de stems com
  Demucs/Spleeter, `vd_param.pb` para vocal). Nenhum é incluído, carregado ou
  convertido. Um script que fazia isso foi aposentado
  (`tools/rekordbox/analyze_ml_retired.py`).
- **Não copia arte, samples, mapeamentos nem binários do instalador.**
  `tools/check-third-party-assets.sh` compara os arquivos de cada commit com os
  do instalador e falha se encontrar uma cópia. Ele roda antes de cada publicação.
- **Não faz engenharia reversa de algoritmos.**

---

## Compilar e abrir

É preciso ter as dependências de compilação do Mixxx 2.5
([guia oficial](https://github.com/mixxxdj/mixxx/wiki/Compiling-on-Linux)).
Os scripts supõem o código em `~/.mixxx/mixxx-source`.

```bash
bash build-bars.sh      # compila em build/ e roda os testes de compassos e grid
bash run-mixxx.sh       # abre com o perfil principal (~/.mixxx)
bash run-bars.sh        # abre com um perfil de teste separado (~/.mixxx/bars-profile)
```

- **Faça backup de `~/.mixxx` antes do primeiro uso com o perfil principal.**
  O `run-mixxx.sh` usa os mesmos biblioteca e configuração do Mixxx instalado.
- O `run-bars.sh` copia a biblioteca e as configurações para um perfil separado
  na primeira execução e não toca no principal.
- Nada é instalado no sistema.
- Escolha a skin em **Preferências → Interface**: `RbxOriginal` ou `RbxFlowBars`.

Testes dos módulos novos:

```bash
QT_QPA_PLATFORM=offscreen build/mixxx-test --resource-path "$PWD/res" \
  --gtest_filter='Phrase*:*Phrase*:BarSync*:BarJumpLoop*:*GridPanel*:AutoDownbeat*:RelatedTracks*:VocalAnalysis*:AnalyzerVocals*'
```

O detector de vocal usa um Python com NumPy e SciPy (padrão
`~/.venv/bin/python`, ou a variável `VILELABOX_VOCAL_PYTHON`).

---

## Estado atual

Experimental, em uso pessoal. Antes de tocar ao vivo, considere:

- **Validação auditiva e visual em uso real ainda está pendente** para a maioria
  dos módulos. Os testes automatizados usam áudio sintético.
- **Stems ainda não existem.** Dependem de migrar para o Mixxx 2.6.
- **Nenhuma controladora foi mapeada.** Os recursos novos são acionados pela skin.

Histórico do desenvolvimento: [PLANO.md](PLANO.md) (plano original) e
[IMPLEMENTACAO.md](IMPLEMENTACAO.md) (registro da execução).

---

## Licença e créditos

GPL-2.0-or-later, a mesma do Mixxx ([LICENSE](LICENSE)). Este projeto é
construído sobre o trabalho da equipe e da comunidade do Mixxx. O README
original do projeto está em [README.mixxx.md](README.mixxx.md).
