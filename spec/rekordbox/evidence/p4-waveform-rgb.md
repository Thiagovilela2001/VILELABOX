# G0 — Formas de onda no modo RGB (PWV4 e PWV5)

Comparação direta entre os bytes do `.EXT` e a tela do rekordbox em execução,
com a mesma faixa carregada.

## Artefato e ambiente

| Campo | Valor |
| --- | --- |
| Programa | `rekordbox.exe` 7.2.8, modo PERFORMANCE, 2Deck Horizontal, forma de onda RGB |
| Faixa | uma faixa da coleção do usuário (2:29, 130 BPM), deck 1, agulha em 79,2 s |
| Dados | `ANLZ0000.EXT` da faixa: tags `PWV3`, `PWV4` (6 × 1200), `PWV5` (2 × 22390, cabeçalho `00 96 03 05`) |
| Captura | `PrintWindow` da janela, 2026-10-24 (privada, fora do repositório) |

## PWV4 — visão geral do deck

- Cada entrada são **6 bytes independentes**, não três `u16`: os bytes 0, 2, 3, 4 e
  5 variam de 0 a 127 e o byte 1 de 0 a 255 na faixa medida.
- Renderizando cor = (b3, b4, b5) normalizada pelo maior e altura = max(b3, b4, b5),
  as regiões rosa/branco, vermelho e verde coincidem coluna a coluna com a visão
  geral do programa. Alternativas testadas (altura por b0 ou b2, cor invertida)
  não coincidem.

## PWV5 — forma de onda ampliada

- `u16` BE: bits 15–13, 12–10 e 9–7 são **vermelho, verde e azul** (0..7); bits 6–2
  são a altura (0..31), espelhada no centro.
- Uma janela de **24 s** centrada na agulha reproduz a sequência de regiões da linha
  ampliada do programa (ciano, bloco vermelho/rosa, ciano, bloco, amarelo/vermelho,
  ciano) na mesma posição horizontal.

## Classificação

- **OBSERVADO:** bytes 3–5 do PWV4 = cor RGB; campos de 3 bits do PWV5 = R, G, B;
  janela padrão de ~24 s na linha ampliada de 1884 px.
- **INFERIDO:** altura do PWV4 = maior dos três canais (a correlação com a altura na
  tela é 0,58; a tela tem marcadores e agulha sobrepostos).
- **DESCONHECIDO:** significado dos bytes 0–2 do PWV4; escala exata de altura usada
  pelo programa; zoom padrão em outras larguras de janela.

## Rastreabilidade

- Parser: `vilelabox-anlz/src/waveform.rs` (`parse_pwv4`, `parse_pwv5`)
- Renderização: `vilelabox-app/src/waveform.rs`
- Teste com arquivo real: `VLBX_ANLZ_EXT=<.EXT> cargo test -p vilelabox-anlz --test waveform_real`
