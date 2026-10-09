> Documento gerado por IA como assistência ao usuário.

# blackmesa: mod de compassos para Mixxx 2.5.6

Base: cópia local da tag 2.5.6 em `~/.mixxx/mixxx-source`.
O arquivo fornecido em Documentos é da branch main; sua cópia de referência
foi preservada em `~/.mixxx/mixxx-main-reference`. A implementação usa 2.5.6
para corresponder ao programa instalado.

## Recursos

- Compassos em 4/4. O compasso 1 é marcado automaticamente ao carregar a faixa e pode ser corrigido com SET 1.
- Linha laranja mais larga a cada quatro batidas na waveform.
- Contador `compasso:tempo` ao lado de cada waveform, por exemplo `12:3`.
- Referência persistida no protobuf do grid; não precisa de migração SQL.
- Funciona com grids de BPM constante e variável.
- Deslocamento do grid mantém a referência no mesmo índice de batida.
- Ajuste fino de BPM mantém esse índice. Dobrar/dividir BPM remapeia a
  referência para a batida mais próxima da posição anterior.
- Respeita LOCK e participa do histórico UNDO de ajustes do grid.
- Sem referência, não exibe contagem nem destaca compassos.
- Linha vermelha mais grossa a cada 8 compassos (32 batidas), marcando frases.
- Sync por compasso: ao dar PLAY, o compasso 1 se alinha com o do outro deck.
- Seção BARS: saltos de N compassos inteiros e loop de N compassos a partir do tempo 1.
- Overview (visão geral da faixa) com as marcas de frase em vermelho.

## Compilar e abrir

```bash
bash ~/.mixxx/mixxx-source/build-bars.sh
bash ~/.mixxx/mixxx-source/run-mixxx.sh   # uso normal: perfil principal ~/.mixxx
bash ~/.mixxx/mixxx-source/run-bars.sh    # perfil de teste separado
```

No menu de aplicativos, use **blackmesa** (ou o comando `blackmesa` no terminal, ou o ícone na área de trabalho): ele chama `run-mixxx.sh`.
O perfil principal usa a skin RbxFlowBars. Se abrir o Mixxx instalado no sistema,
a skin carrega, mas os controles de compasso não funcionam. O backup do banco e da
configuração anterior ao mod está em `~/.mixxx/backups-compassos/`.

A compilação usa Qt6, desativa a interface QML experimental e a exportação
Engine Prime para dispensar a dependência libdjinterop. Não instala arquivos
no sistema. Se a compilação já terminou, basta usar `run-bars.sh`.

O perfil de teste fica em `~/.mixxx/bars-profile`. Na primeira preparação,
a biblioteca SQLite é copiada com a API de backup, que inclui alterações
pendentes no WAL. Configurações, efeitos e controladores também são copiados.
O perfil não é sobrescrito nas próximas aberturas. As músicas continuam nos
mesmos caminhos; os arquivos de áudio não são copiados.

## Usar

1. Carregue uma faixa e desligue LOCK.
2. Alinhe o GRID com as batidas audíveis; corrija o BPM se necessário.
3. Ao carregar a faixa, o compasso 1 já vem marcado automaticamente (veja abaixo).
   Se estiver errado, pause no primeiro tempo musical e clique em SET 1.
4. A batida mais próxima vira compasso 1, tempo 1. Confira o contador e as
   linhas laranja em outros trechos e ative LOCK ao terminar.
5. Para remover a referência, desligue LOCK e clique com o botão direito
   em SET 1. UNDO usa as regras existentes de agrupamento de ajustes rápidos.

## SET 1 automático

Ao carregar uma faixa que ainda não tem compasso 1, o blackmesa marca o compasso 1
sozinho, na batida mais próxima do **CUE principal**. O CUE costuma ficar no tempo 1:
no primeiro som da música ou onde você o colocou. Sem CUE, usa a primeira batida do
grid. Se a faixa ainda não foi analisada, a marcação acontece quando a análise
entregar o grid.

- Faixas que já têm compasso 1 não são alteradas.
- Faixas com LOCK aceso não são alteradas; desligue o LOCK e recarregue.
- A marcação é feita uma vez por carregamento: se você remover com o botão
  direito em SET 1, ela não volta até a faixa ser carregada de novo.
- UNDO desfaz a marcação automática.
- Controle `[ChannelN],auto_first_downbeat` (padrão 1, persistente) liga e desliga.

## Sync por compasso

1. Marque SET 1 nas duas faixas.
2. Com um deck tocando, deixe o outro parado no ponto de entrada, com QUANTIZE ou SYNC.
3. Dê PLAY. Além de alinhar a batida, o Mixxx recua de 0 a 3 batidas para que o
   tempo do compasso fique igual ao do outro deck. Recuar, em vez de avançar,
   faz o tempo 1 marcado cair no próximo tempo 1 do outro deck.

O botão `N · BAR`, acima do contador, liga e desliga o recurso por deck. Aceso
significa ligado, o padrão. O alinhamento acontece só ao dar PLAY: hotcues e saltos
com a música tocando continuam alinhando apenas a batida. Ligar SYNC com a faixa
já tocando também não reposiciona o compasso. Se uma das faixas não tiver SET 1,
vale o comportamento normal do Mixxx.

## Saltos e loops em compassos (seção BARS)

Fica à direita de BEAT JUMP, em cada deck.

- `◀◀` / `▶▶`: voltam ou avançam N compassos inteiros (4·N batidas). O tempo
  dentro do compasso não muda: se estava no tempo 3, continua no tempo 3, e a
  mixagem continua alinhada. Com um loop ativo, movem o loop.
- Botão direito em `◀◀` / `▶▶`: diminui pela metade / dobra N (1 a 64). O número
  entre as setas mostra N, que fica salvo entre sessões.
- `BAR LOOP`: cria um loop de N compassos começando no tempo 1 do compasso atual
  e acende enquanto o loop está ativo. Clique de novo para sair. Sem SET 1, o loop
  começa na batida atual.

Antes da referência, a contagem usa compassos 0, −1, −2, etc., mantendo
os tempos entre 1 e 4. A waveform continua recebendo músicas por arrastar.

O mod usa 4/4 fixo. Não detecta automaticamente o primeiro tempo nem trata
mudanças de fórmula de compasso dentro de uma faixa. Uma versão sem o mod
não lê essa referência; reanalisar/regravar o grid nela pode removê-la.

## Controles novos

Em `[ChannelN]`:

| Controle | Função |
| --- | --- |
| `beats_set_first_downbeat` | Define a batida mais próxima como início do compasso 1 |
| `beats_clear_first_downbeat` | Remove a referência |
| `bar_number` | Número inteiro do compasso atual, somente leitura |
| `beat_in_bar` | Tempo atual de 1 a 4, somente leitura |
| `downbeat_valid` | Indica referência disponível, somente leitura |
| `auto_first_downbeat` | Marca o compasso 1 automaticamente ao carregar a faixa (padrão 1, persistente) |
| `bar_size` | N, em compassos, para saltos e BAR LOOP (padrão 4, persistente) |
| `bar_size_halve` / `bar_size_double` | Diminuem pela metade / dobram N |
| `bar_jump_forward` / `bar_jump_backward` | Saltam N compassos inteiros |
| `bar_loop_toggle` | Liga o loop de N compassos a partir do tempo 1 ou desliga o loop |
| `bar_sync` | Liga/desliga o sync por compasso ao dar PLAY (padrão 1, persistente) |

XML da waveform: `ShowBarMarkers` habilita o destaque e `BarMarkerColor`
define sua cor. `ShowPhraseMarkers`, `PhraseBars` (padrão 8) e
`PhraseMarkerColor` controlam as marcas de frase, na waveform e no `<Overview>`;
no overview, `ShowBarMarkers` também é aceito, mas fica desligado por ser denso. Esses elementos existem apenas no executável modificado.
A skin separada `RbxFlowBars` os habilita. A RbxFlow original permanece disponível.

## Validação

Os testes novos estão em `src/test/beatstest.cpp` e
`src/test/beatstranslatetest.cpp`. Cobrem a contagem, o salvamento em grids
constantes/variáveis, a remoção, alterações de BPM/grid, seeks e LOCK.
O script de compilação executa as suítes relevantes ao terminar.
Resultado: compilação completa aprovada; 78 testes de nove suítes passaram (incluindo BarSyncTest, BarJumpLoopTest, GridPanelTest e AutoDownbeatTest), além de 198 testes de sync, loop, cue e engine do Mixxx,
incluindo os novos testes de integração. O carregamento da skin foi confirmado
em modo offscreen/software com saída ALSA nula, sem erros de parsing ou recursos
ausentes. O áudio e a aparência na sessão gráfica precisam ser conferidos pelo usuário.
As ferramentas de referências e dimensões da skin passaram; os XMLs são válidos.

> Fim do documento gerado por IA.
