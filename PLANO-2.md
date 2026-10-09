> Documento gerado por IA como assistência ao usuário.

# Plano 2 — correções e publicação no GitHub

Ponto de partida: branch `blackmesa` em `4de552c` (fases 0–5 do PLANO.md
commitadas, fase 6 adiada). Este plano cobre três coisas, nesta ordem:

| Fase | Entrega | Bloqueia |
| --- | --- | --- |
| A | Remover material de terceiros do histórico | Qualquer `git push` |
| B | Corrigir os quatro defeitos da revisão, com testes | Uso ao vivo |
| C | Publicar no GitHub | — |
| D | Verificação final e validação na sessão real | Declarar "pronto" |

Regras gerais: um commit por item, todos os testes do módulo passando a cada
commit e nenhum `push` antes de a fase A estar concluída e verificada.

---

## Fase A — Material de terceiros (bloqueia a publicação)

### Problema

39 arquivos em `res/skins/RbxOriginal/rb/` são **cópias byte a byte** de
arquivos do instalador do rekordbox (`~/Downloads/rekordbox_extracted/skins`).
Entre eles estão `rekordboxLogo.svg`, `logo.svg`, botões de play/cue, faders e
o editor de grid. Eles entraram no commit `1de4979` e foram tocados em
`4de552c`. As skins também usam "rekordbox" em textos (10 arquivos da
RbxFlowBars, incluindo o README).

Publicar esses arquivos redistribuiria arte e marca registrada da
AlphaTheta/Pioneer. Apagá-los num commit novo não basta, porque eles continuam no
histórico enviado.

### A1. Inventário reproduzível

Criar `tools/check-third-party-assets.sh`. O script compara o SHA-256 de todos
os arquivos versionados (`git ls-files`) com os arquivos de imagem, áudio e CSV
do diretório extraído, passado como argumento. Sai com código ≠ 0 e lista os
caminhos se encontrar alguma coincidência. Resultado esperado hoje: 39 arquivos.

### A2. Substituir os ícones

- Desenhar SVGs próprios para cada um dos 39 ícones, mantendo nome, tamanho
  (viewBox) e estados on/off, para que os XML da skin não mudem.
- Os logos são removidos, não redesenhados. Usar `res/images/icons/scalable/apps/blackmesa.svg`
  ou só texto "VILELABOX".
- Trocar "rekordbox" nos textos e comentários das skins por descrições neutras
  ("estilo de dois decks", "layout inspirado em controladoras DJ").
- Avaliar renomear `RbxOriginal` / `RbxFlowBars` para `VilelaboxOriginal` /
  `VilelaboxFlow`. Se renomear, atualizar `~/.mixxx/mixxx.cfg` (`ResizableSkin`)
  e os scripts `run-*.sh`.

### A3. Reescrever o histórico local

Ainda não houve push, então reescrever é seguro:

1. `git branch backup/blackmesa-antes-limpeza` (cópia do estado atual).
2. Remover `res/skins/RbxOriginal/rb/` de todos os commits desde `3ebac44`
   (`git filter-repo --path res/skins/RbxOriginal/rb --invert-paths
   --refs 3ebac44..blackmesa` ou rebase equivalente).
3. Um commit novo, "Skins: ícones próprios", com os SVGs da A2.
4. Rodar o A1 em **todos** os commits da branch
   (`git rev-list 3ebac44..blackmesa`). Nenhuma coincidência é aceita.

### A4. Outras verificações antes de publicar

- `tools/rekordbox/analyze_ml_retired.py`: confirmar que não contém pesos,
  hashes de licença nem caminhos com dados pessoais. Senão, remover do histórico também.
- Procurar segredos e dados pessoais: `git grep -nE 'token|password|/home/thiago'`.
  Caminhos absolutos em scripts e documentos devem virar `$HOME` ou relativos.
- GRID.md cita offsets e hashes do `rekordbox.exe`. São só fatos de observação,
  sem conteúdo copiado, mas vale decidir se continuam num repositório público.

Critério de saída: o A1 limpo em toda a branch e o build da skin carregando
sem recursos ausentes (`mixxx --developer` com a skin, sem avisos de arquivo).

---

## Fase B — Correções da revisão

Cada item recebe um teste que **falha antes da correção**.

### B1. Hot cues duplicados ao editar o grid (alta)

Causa: `PhraseControl` chama `AutoPhraseCues::apply` em todo
`PhraseAnalysisStore::updated` (`phrasecontrol.cpp:45`). Isso inclui cada
ajuste de grid, e a deduplicação só aceita posição idêntica
(`autophrasecues.cpp:38`).

Correção:
- O sinal `updated` passa a informar o motivo (`Analyzed` / `Recalculated`).
  Os cues são gerados só em `Analyzed` ou no carregamento, uma vez por faixa carregada.
- Deduplicação por tipo + rótulo com tolerância de 1 batida, não por posição exata.
- Teste: faixa sintética com cues automáticos ligados; deslocar o grid 5 vezes
  com `beats_translate_*` e ajustar o BPM 2 vezes. O número de hot cues não muda.

### B2. Trabalho bloqueante no callback de áudio (alta)

`EngineControl::trackLoaded` roda no callback de áudio (`CachingReader::process`
emite o sinal com `DirectConnection`).

| Onde | Problema | Correção |
| --- | --- | --- |
| `padcontrol.cpp:52` | Aloca cerca de 3 MB a cada faixa | Alocar no construtor para a taxa máxima suportada, ou realocar na thread principal quando `[App],samplerate` mudar, com troca atômica do buffer |
| `phrasecontrol.cpp:49` | Cria cues na faixa | `QMetaObject::invokeMethod(..., Qt::QueuedConnection)` para a thread principal |
| `mixpointcontrol.cpp:97` | `refreshPoints()` trava o mutex da faixa a cada ciclo quando não há pontos | Calcular os pontos na thread principal (carregamento, `cuesUpdated`, frases atualizadas) e publicar nos COs. O `process()` só lê os COs |
| `mixpointcontrol.cpp:101` | `ControlObject::get(ConfigKey)` busca com lock a cada ciclo | Criar `ControlProxy` de `play` para cada `[ChannelN]` no construtor |

Teste: contador de chamadas, só em build de teste, que verifica que
`refreshPoints` e `apply` não são chamados dentro de `process()`/`trackLoaded()`.
Em `PadControl`, verificar que carregar 10 faixas não realoca o buffer
(endereço igual).

### B3. Mix Point entra atrasado (média)

Causa: no disparo, o deck vai para o ponto de entrada sem descontar quanto o
outro deck já passou do ponto de saída, que é até um buffer
(`mixpointcontrol.cpp:118`).

Correção: `entry += (current − m_trigger) × (taxa deste deck / taxa do outro)`,
convertendo frames entre taxas de amostragem diferentes. Com bar_sync ou
quantize ativos, garantir que o PLAY não aplique um segundo reposicionamento
(disparar com a posição já alinhada).

Teste: buffer de 1024 frames com o ponto de saída no meio do buffer. Depois do
disparo, a diferença de fase entre as batidas dos dois decks deve ser < 1 ms.
Repetir com 44,1 / 48 kHz misturados.

### B4. Gravação do cache trava a interface (baixa)

Causa: cada `beatsUpdated` regrava o JSON inteiro (características + frases)
na thread da interface (`phraseanalysisstore.cpp:119`).

Correção: as características são gravadas uma vez, na análise. Nos recálculos,
grava só a parte de frases, com atraso de 2 s (`QTimer` single-shot por faixa)
e escrita fora da thread da interface (`QtConcurrent::run`).

Teste: 20 recálculos seguidos resultam em 1 escrita, e o arquivo continua válido no `load()`.

### B5. Release Echo — decisão do usuário antes de codificar

Comportamento atual, documentado no PADS.md:
- as rotas de efeito do deck são desligadas e não voltam;
- com STOP APÓS desligado, a música volta de repente depois do eco.

Proposta:
- guardar e restaurar as rotas ao terminar o eco;
- com STOP APÓS desligado, fazer o som original voltar num fade de meio tempo
  em vez de corte seco.

Confirmar com o usuário antes de implementar.

---

## Fase C — Publicação no GitHub

Situação nesta máquina: `origin` aponta para `mixxxdj/mixxx` (o projeto
oficial, **não** enviar para lá). Não há chave SSH, credencial do git nem `gh`
instalado. Também não existe `Thiagovilela2001/mixxx`.

Passos, depois das fases A e B:

1. Autenticação (feita pelo usuário, interativa):
   `sudo pacman -S github-cli && gh auth login`.
2. Remotos: `git remote rename origin upstream`.
3. Criar o repositório. Recomendação: **novo repositório privado**
   `Thiagovilela2001/blackmesa`, não um fork. No GitHub, um fork de repositório
   público é sempre público.
   `gh repo create Thiagovilela2001/blackmesa --private --source . --remote origin`
4. Enviar só a branch do mod e a tag base:
   `git push -u origin blackmesa && git push origin 2.5.6`.
5. No repositório: README curto (o que é, base Mixxx 2.5.6, como compilar,
   links para os documentos de cada módulo). A licença continua
   **GPL-2.0-or-later**, herdada do Mixxx (`LICENSE` já está na árvore).
6. Hook `pre-push` local que roda o `tools/check-third-party-assets.sh` e
   bloqueia o envio se encontrar algo.

Se depois quiser tornar público, rodar a A1 de novo e revisar a A4.

---

## Fase D — Verificação final

1. Build limpo do zero: `rm -rf build && bash build-bars.sh`.
2. Suíte completa comparada com a base. Rodar a mesma suíte em `3ebac44` (sem o
   mod) para provar que `ControllerScriptEngineLegacyTimerTest` e o travamento
   em `ControlObjectAliasTest.PlayerManager` já existiam. Registrar os dois
   resultados no IMPLEMENTACAO.md.
3. Checklist para o usuário na sessão real (áudio e tela):
   - [ ] Compassos: contador, linhas laranja e vermelhas e SET 1 automático em 3 faixas.
   - [ ] Sync por compasso e PhraseSync: PLAY no deck parado cai no tempo 1 e no início da frase.
   - [ ] Frases: faixa colorida na waveform e no overview; rótulo e compassos restantes.
   - [ ] Cues automáticos: ligar, ajustar o grid várias vezes, confirmar que não duplicam (B1).
   - [ ] Mix Point Link: armar, ouvir a entrada no compasso certo, cancelar com seek.
   - [ ] Slicer, KeyShift, Keyboard e Release Echo pelos botões da skin.
   - [ ] Faixas relacionadas: a lista muda ao carregar faixas de tons diferentes.
   - [ ] Nenhum estalo ao carregar faixas com os dois decks tocando (B2).
4. Anotação de 20 faixas reais para as frases. Script auxiliar
   `tools/phrases/export_csv.py` que lê os JSON do cache e gera uma planilha
   (faixa, início, fim, tipo detectado, tipo correto). Meta do PLANO.md: ≥ 80%
   dos limites a no máximo 1 compasso de distância, válido só para essa amostra.

---

## Depois deste plano

- Ajustar os limiares de frase (`0.12`, `0.65`, `0.55` em `phrases.cpp`) com
  base nas anotações da D4.
- Fase 6 (stems): reavaliar migração para o Mixxx 2.6 quando houver versão estável.
- Mapeamento de controladora, se uma for adquirida.

> Fim do documento gerado por IA.
