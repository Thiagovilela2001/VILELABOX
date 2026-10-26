# BPM e beatgrid

## Contrato observado

Referência funcional: rekordbox 7.2.8 para Windows e manual oficial 7.2.8,
seção GRID/PHRASE EDIT (pp. 134–135 do PDF). Artefato inspecionado:
`rekordbox.exe`, SHA-256
`85e3a58524c5cd1e870ad59e23cc8806523175f80fb82d6059c68d57bdf9991a`.

O editor deve oferecer, por deck:

- definir a batida mais próxima como 1.1;
- ir à primeira batida do compasso;
- deslocar a grade para esquerda/direita em 3 ms, ou 1 ms no modo fino;
- desfazer e refazer;
- reproduzir/pausar enquanto se edita;
- informar BPM ou medi-lo por TAP;
- estreitar/alargar o intervalo da grade em 3 ms, ou 1 ms no modo fino;
- dividir/multiplicar o BPM por dois;
- aplicar à faixa inteira ou somente da posição atual em diante;
- reconstruir a grade, ancorada na posição atual, com o BPM informado;
- metrônomo sincronizado à grade, com três volumes e acento no 1;
- bloquear/desbloquear a edição;
- persistir a grade editada e o bloqueio por faixa.

## Manipulação da waveform

Ao pressionar e arrastar a waveform ampliada, o áudio deve acompanhar
continuamente o gesto como uma mídia física. Arrastar para a direita recua a
posição e lê o buffer em ordem reversa, produzindo backspin; arrastar para a
esquerda avança. Soltar restaura o estado de transporte anterior. `Shift`
reduz a sensibilidade para 15%, e a roda continua disponível para jog discreto.

O caráter audível de scratch/backspin foi observado pelo usuário. A resposta
foi implementada como leitura bidirecional com interpolação linear e perseguição
do alvo a cada bloco de áudio; essa modelagem física é uma escolha do VILELABOX,
não uma reconstrução comprovada do algoritmo DSP proprietário.

## Mapeamento VILELABOX

- `beatgrid.rs`: transformações, modos inteiro/parcial, TAP e histórico de 64 estados.
- `grid_store.rs`: sidecar JSON gravado atomicamente por faixa.
- `grid_edit.slint`: painel operável com mouse.
- `deck.rs`: clique de metrônomo gerado no motor de áudio.
- `sync.rs`: sincronização de BPM, fase e número da batida.

## Análise automática

Evidência: `evidence/p4-1-analise-batida.md` (estática, endereços do
`rekordbox.exe` 7.2.8).

- O BPM da análise é entregue como inteiro BPM × 100, arredondado com
  `(int)(bpm * 100 + 0,5)`.
- Cada batida da análise carrega posição, número no compasso e BPM × 100
  calculado de `6000000 / intervalo_ms`, arredondado.
- O resultado é dobrado ou dividido por dois até cair na faixa de BPM da
  preferência; as faixas são 70–180 (padrão), 48–95, 58–115, 68–135, 78–155,
  88–175, 98–195, 108–215, 118–235 e 128–255.
- O motor avalia candidatos somando um sinal `i16` em janelas de uma batida
  (60/BPM segundos) ou meia batida (30/BPM), começando em um quarto da
  posição pedida.

## Perguntas abertas

As operações visíveis acima são documentadas e foram reproduzidas. Os detalhes
internos proprietários do cálculo de âncora parcial e da filtragem do TAP não
foram recuperados: a indexação Ghidra do executável de 573 MB excedeu o tempo
da sessão. Nesses dois pontos, a implementação usa regras determinísticas
compatíveis com o comportamento documentado. A persistência é um sidecar do
VILELABOX; ela não altera `master.db` nem arquivos ANLZ do rekordbox.
