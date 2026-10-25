# G0 — Curvas de ganho do mixer: fader, TRIM, crossfader e EQ

Medição na saída de áudio do rekordbox. Deck 1 tocando um loop de 1 batida
(sinal periódico e estável); cada controle posicionado por automação de
interface (padrão `RangeValue` da UIA, unidades do próprio rekordbox) e a saída
gravada por loopback do Windows.

## Artefato e ambiente

| Campo | Valor |
| --- | --- |
| Programa | `rekordbox.exe` 7.2.8.0 (SHA-256 `85e3a585…9991a`), pid 15504 |
| Captura | loopback da saída padrão (Realtek), 48 kHz, janela de 1,38 s por ponto; scripts privados `rec.py` (Python 3.12 + `soundcard` + `numpy`) e `varrer.ps1` |
| Sinal | faixa do usuário (163 BPM), loop automático de 1 batida no deck 1; deck 2 parado |
| Repetibilidade | três medições do mesmo ponto: −17,84 / −17,87 / −17,87 dBFS (±0,03 dB) |
| Data | 2026-10-25 |

## Faixas dos controles (UIA)

| Controle | Faixa | Observação |
| --- | --- | --- |
| Fader de canal | 0..100 | só valores inteiros (0,75 → 1; 0,25 → 0) |
| TRIM | −10..10 | fracionário |
| HIGH / MID / LOW | −50..50 | fracionário |
| Crossfader | −10..10 | fracionário; deck 1 à esquerda |
| Fone: mix / nível | −50..50 / 0..100 | não medido |

## Fader de canal (dB relativos ao valor 100)

| 100 | 90 | 80 | 70 | 60 | 50 | 40 | 30 | 20 | 10 | 5 | 2 | 1 | 0 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | −2,23 | −4,70 | −7,13 | −9,56 | −12,00 | −16,18 | −24,76 | −39,44 | −61,90 | −76,45 | −88,26 | −95,32 | mudo |

## Crossfader, canal do deck 1 (dB relativos ao crossfader todo à esquerda)

| −10..−2 | −1 | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | −0,14 | −1,06 | −1,93 | −2,91 | −4,07 | −5,40 | −6,99 | −8,94 | −11,53 | −15,02 | −21,00 | mudo |

## TRIM (dB relativos a 0)

| −10 | −8 | −6 | −4 | −2 | 0 | 2 | 4 | 6 | 8 | 10 |
|---|---|---|---|---|---|---|---|---|---|---|
| mudo | −24,17 | −11,87 | −6,06 | −2,55 | 0 | +1,97 | +3,51 | +4,94 | +7,18 | +8,78 |

Os positivos são da segunda medição, com o fader em 50 (−12 dB de folga). Com o
fader em 100 eles saíram achatados (+10 → +5,71 dB), compatível com o limitador
do master agindo nos picos.

## EQ (indicativo)

Medido com música, por banda de energia (20–250, 250–2500, 2500–16000 Hz). Os
cortes das bandas do VILELABOX não coincidem com os do filtro do rekordbox; os
números indicam direção e banda, não a curva do filtro.

| Knob em −50 | grave | médio | agudo |
| --- | --- | --- | --- |
| LOW | −7,2 dB | −1,5 dB | 0 |
| MID | −5,4 dB | −10,9 dB | −10,0 dB |
| HIGH | 0 | −0,3 dB | −8,5 dB |

| Knob em +50 (fader em 50) | grave | médio | agudo |
| --- | --- | --- | --- |
| LOW | +4,0 dB | +1,1 dB | 0 |
| MID | +3,4 dB | +5,1 dB | +5,0 dB |
| HIGH | 0 | +0,2 dB | +3,5 dB |

## Classificação

- **OBSERVADO:** curvas do fader, do crossfader do lado do deck 1 e do TRIM nos
  pontos das tabelas; fader só inteiro; fader 0, TRIM −10 e crossfader +10 mudos;
  direção e banda de cada knob do EQ; mínimo do EQ não é mudo (corte de ~7–11 dB
  na banda medida).
- **INFERIDO:** interpolação linear em dB entre os pontos; lado do deck 2 do
  crossfader como espelho do deck 1; limitador do master como causa do
  achatamento dos positivos.
- **DESCONHECIDO:** crossfader entre +9 e +10 (a medição fina pegou o deck já
  parado; o VILELABOX cai linearmente em ganho de −21 dB ao mudo); frequências de
  corte e formato do filtro do EQ (precisa de tons puros); fone (mix e nível);
  curvas alternativas do crossfader nas preferências; limitador do master.

## Rastreabilidade

- Especificação: `spec/rekordbox/33-mixer-fx.md`
- Motor: `vilelabox-audio/src/mixer.rs` (tabelas e interpolação), `output.rs` (`AudioMixer::ganho`, `SetFader`, `SetTrim`, `SetCrossfader`)
- App: `vilelabox-app/src/main.rs` (tratadores), `presentation.rs` (`render_mixer`), tela em `ui/components/mixer.slint`
- Testes: `vilelabox-audio/src/mixer.rs` (pontos e interpolação), `vilelabox-audio/tests/mixer_tests.rs` (`AudioMixer` real)
