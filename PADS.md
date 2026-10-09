# Pads na skin

Nenhuma controladora foi mapeada. RbxOriginal e RbxFlowBars têm um painel opcional
por deck, aberto pelo botão **PADS** ao lado do contador de compassos. O estado
visual fica salvo pela skin; os modos de execução começam desligados.

- SLICER: habilite o modo e mantenha um dos oito botões pressionado. A janela de
  oito tempos segue pares de compassos relativos ao SET 1; cada botão repete uma
  batida. Ao soltar, retorna à linha temporal do slip. Não toma controle de um
  loop ou slip que já estava ativo. LOOP congela e repete a janela inteira.
- KEYSHIFT: −6 a +6 semitons, sempre com trava de tom ativa.
- KEYBOARD: mesma transposição, com salto ao hot cue escolhido C1–C8. O hot cue
  precisa existir. O modo não transforma maior em menor nem cria um sampler.
- RELEASE / JUMP: ECHO RELEASE desativa as quatro rotas de unidades de efeito
  desse deck e toca um eco próprio de uma batida com decaimento de um compasso.
  STOP APÓS determina se a reprodução para no fim (padrão 0, persistente).
  Os botões BeatJump usam os controles nativos já existentes.

Controles `[ChannelN]`: `slicer_enable`, `slicer_loop`, `slicer_slice_1..8`,
`keyshift_-6..6`, `keyboard_-6..6`, `keyboard_hotcue` (1–8), `release_fx`,
`release_fx_stop`. Slicer exige SET 1. Pads de Slicer são momentâneos.

Release Echo mantém um buffer circular em memória; não carrega uma cadeia de
usuário nem usa modelos. O efeito usa a taxa de áudio vigente ao carregar a
faixa. Recarregue a faixa se mudar a taxa do dispositivo. Não há vinyl brake
nessa versão. Com STOP APÓS desligado, a reprodução normal volta após o eco.
As rotas de efeitos desativadas permanecem desligadas até o usuário religá-las.

Testes de integração: `BarSyncTest.Slicer*`, `BarSyncTest.Keyshift*` e
`BarSyncTest.ReleaseEcho*`. Conferência auditiva e visual no desktop pendente.
Backups das skins: `backups-performance/`.
