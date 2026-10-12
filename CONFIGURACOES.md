# Configurações do VILELABOX

A engrenagem da RbxOriginal e o botão SETTINGS da RbxFlowBars abrem a janela
completa de configurações. Também é possível usar Preferências no menu ou
Ctrl+P no Linux e Windows.

## Organização

A janela usa visual escuro, categorias na lateral, abas para as seções da
categoria e busca por nomes e textos das opções.

| Categoria | Configurações |
| --- | --- |
| Áudio | Dispositivos, entradas, saídas e decodificação disponível no build |
| Biblioteca | Coleção, diretórios e opções da biblioteca |
| Controladoras | Dispositivos, mapeamentos e controle por vinil quando disponível |
| Visualização | Interface, waveforms, cores e painéis das skins |
| Performance | Decks, mixer, efeitos e Auto DJ |
| Gravação e transmissão | Gravação e transmissão quando disponível |
| Análise | BPM/grid, frases, cues automáticos, energia, tonalidade e normalização |

Algumas seções dependem das opções de compilação do Mixxx.

## Painéis e opções do mod

Em **Visualização → Painéis e opções do mod**, as abas RbxOriginal e RbxFlowBars
substituem o antigo painel simples da skin. Elas configuram waveforms, mixer,
biblioteca, pads, loops, beat jump, grid, cues de intro/outro, tempo, EQ e crossfader.
Cada skin mantém suas próprias opções.

Na aba **Opções do mod**, é possível configurar:

- Modo RGB da waveform (graves vermelhos, médios verdes e agudos azuis).
- Máximo de faixas relacionadas: de 1 a 1.000, padrão 100.
- Tempo limite do detector de vocal: de 1 a 3.600 segundos, padrão 900 (o Demucs leva cerca de um terço da duração da faixa).

O limite de relacionadas é usado na próxima busca; o timeout de vocal é usado
na próxima execução do detector. O modo RGB depende do renderer RGB compatível.

## Aplicar alterações

**Aplicar** salva as alterações sem fechar a janela. **OK** aplica e fecha.
**Cancelar** descarta as alterações pendentes conforme o comportamento de cada
página existente. Algumas páginas do Mixxx oferecem ajustes imediatos ou pedem
recarregamento da interface.

Na nova página de painéis do mod, as edições só alteram os controles ao aplicar.
Restaurar padrões deixa os valores pendentes para revisão antes de aplicar.

Buscar sem encontrar resultados não apaga os valores editados. Limpe a busca
para voltar às seções.

## Referência visual

A organização usa a janela de preferências do rekordbox como referência:
categorias, seções e controles agrupados. O conteúdo disponível corresponde
às funcionalidades implementadas no VILELABOX e no Mixxx.

Referência: [manual oficial de preparação do rekordbox 6.8](https://cdn.rekordbox.com/files/20231204094619/rekordbox6.8.0_introduction_EN.pdf).
