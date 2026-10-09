# Edição do grid — Mixxx 2.5.6

O painel ao lado da visão geral de cada deck mantém a altura de 60 px.
A largura passa de 66 para 102 px; o restante do deck usa o mesmo layout.

- GRID: alinha a batida mais próxima à posição de reprodução. Botão direito: alinha o grid com outro deck em reprodução.
- ◀ / ▶: deslocam o grid para antes/depois, sem alterar seu BPM.
- BPM− / BPM+: ajustam o BPM original em passos de 0,01.
- TAP: ajusta o BPM original pelo ritmo dos cliques, sem mover o pitch.
- UNDO: desfaz os ajustes do grid (até dez estados; ajustes rápidos podem ser agrupados).
- Número inferior: BPM original da música, sem o pitch do deck.
- LOCK: liga/desliga o bloqueio do BPM; o botão aceso indica bloqueio.

## Uso

1. Carregue uma faixa e desative LOCK caso esteja aceso.
2. Pause em uma batida clara, usando a waveform ampliada, e pressione GRID.
3. Confira outras partes da música. Use as setas para corrigir deslocamento;
   se o erro aumentar ao longo da faixa, ajuste BPM− / BPM+.
4. Use UNDO se necessário e ative LOCK quando terminar.

## Compassos no executável modificado

A skin RbxFlowBars acompanha o mod de Mixxx 2.5.6. O botão SET 1 define a
batida mais próxima como compasso 1, tempo 1. As linhas laranja destacam cada
quarta batida e o contador indica compasso:tempo. Use o executável modificado
via `~/.mixxx/mixxx-source/run-bars.sh`. Veja COMPASSOS.md.

Referências:
- https://manual.mixxx.org/2.5/en/chapters/appendix/mixxx_controls.html
- https://github.com/mixxxdj/mixxx/issues/15778

Backup anterior: backups/beatgrid-20261007/beatgrid_controls.xml.
Validação: XMLs analisados e ferramentas check_refs.py e check_sizes.py aprovadas.
A tentativa de carregamento isolado em /tmp não chegou à interface: o ambiente
restringe o acesso ao áudio. A aparência e os cliques ainda precisam ser conferidos
no Mixxx da sessão gráfica. A biblioteca do usuário não foi modificada.

## Correção (07/10/2026)

Os botões deste painel não funcionavam: os `ConfigKey` estavam quebrados em várias
linhas e o Mixxx não remove esses espaços. Ele criava em silêncio controles
falsos com nomes como `"\n [Channel\n 1\n ],beats_translate_curpos"`. Agora cada
`ConfigKey` fica em uma linha só. Para conferir, abra o Mixxx com `--developer`
e procure no log por `Creating skin control object`: só devem aparecer
`[RbxFlow],show_*` e `[App],leftdeck/rightdeck`. Backup em `backups/gridfix-*`.
