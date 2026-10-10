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
- RELEASE / JUMP: Release FX nos três tipos que o rekordbox define nos
  mapeamentos das controladoras (`FX1ReleaseFXType`, "BREAK-ECHO-SPIN"):
  - BREAK: freio de vinil; a velocidade cai em linha reta até parar.
  - ECHO: o som original é cortado e a última batida se repete, decaindo.
  - SPIN: backspin rápido para trás, que vai perdendo velocidade.
  
  `½` e `2×` ajustam a duração de 1 a 16 batidas (o Adjust Time do rekordbox;
  padrão 4). RELEASE dispara e fica laranja enquanto o efeito roda. Como no
  rekordbox, o Release "solta" os efeitos de batida: as unidades de FX
  enviadas a esse deck são desligadas, mas o envio (rotas 1/2) continua como
  estava, então basta apertar ON para religar. STOP APÓS (padrão ligado,
  persistente) para o deck no fim, já que os três tipos são efeitos de saída;
  desligado, o som do deck volta com fade de meio tempo. Os arquivos do
  rekordbox não definem o que acontece depois do efeito; parar é a escolha
  deste mod. Os botões BeatJump usam os controles nativos já existentes.
- O painel e a aba escolhida ficam salvos entre sessões.

Controles `[ChannelN]`: `slicer_enable`, `slicer_loop`, `slicer_slice_1..8`,
`keyshift_-6..6`, `keyboard_-6..6`, `keyboard_hotcue` (1–8), `release_fx`,
`release_fx_stop`, `release_fx_type` (0 BREAK, 1 ECHO, 2 SPIN; persistente),
`release_fx_break` / `release_fx_echo` / `release_fx_spin`, `release_fx_beats`
(persistente), `release_fx_beats_halve` / `release_fx_beats_double` e
`release_fx_active` (somente leitura). Slicer exige SET 1. Pads de Slicer são momentâneos.

Release Echo mantém um buffer circular em memória; não carrega uma cadeia de
usuário nem usa modelos. O buffer é alocado uma vez por deck, ao abrir o
programa (cerca de 3 MB), e nunca durante o áudio; o efeito acompanha a taxa
atual do dispositivo. BREAK e SPIN usam o controle nativo `scratch2` do Mixxx
para mudar a velocidade, então a altura cai junto, como num toca-discos.

Testes de integração: `BarSyncTest.Slicer*`, `BarSyncTest.Keyshift*` e
`BarSyncTest.ReleaseEcho*`. Conferência auditiva e visual no desktop pendente.
Backups das skins: `backups-performance/`.
