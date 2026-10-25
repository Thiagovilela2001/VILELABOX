# 33 — Mixer

## Escopo

Fader de canal, TRIM e crossfader dos dois canais, operados pela tela.

Fora do escopo, por falta de evidência: forma do filtro do EQ (HIGH/MID/LOW),
fone (CUE de canal, mix e nível), curvas alternativas do crossfader, limitador do
master, efeitos e sampler.

## Glossário

- **Valor:** posição do controle nas unidades que o rekordbox expõe (fader
  0..100, TRIM −10..10, crossfader −10..10).
- **Ganho do canal:** produto dos ganhos de fader, TRIM e crossfader.

## Comportamento

| Controle | Regra | Classe |
| --- | --- | --- |
| fader | ganho pela tabela abaixo; só valores inteiros; 0 é mudo | OBSERVADO |
| TRIM | ganho pela tabela abaixo; −10 é mudo | OBSERVADO |
| crossfader | lado do canal (deck 1 à esquerda) com ganho cheio até −2; cai até o mudo no lado oposto (+10) | OBSERVADO para o deck 1; deck 2 é o espelho (INFERIDO) |
| entre pontos | interpolação linear em dB | INFERIDO |
| EQ | atua na banda do nome; mínimo não é mudo | OBSERVADO; curva DESCONHECIDA, sem efeito no VILELABOX |

## Constantes funcionais

Ganho em dB; tabelas completas no dossiê `evidence/p7-0-mixer-curvas.md`.

| Nome | Valor | Unidade | Função |
| --- | --- | --- | --- |
| fader em 50 | −12,00 | dB | metade do curso |
| fader em 1 | −95,32 | dB | último degrau antes do mudo |
| TRIM em +10 | +8,78 | dB | ganho máximo |
| TRIM em −8 | −24,17 | dB | último ponto antes do mudo |
| crossfader no centro | −1,06 | dB | cada canal no centro |
| crossfader em +9, canal oposto | −21,00 | dB | último ponto medido antes do mudo |

## Casos de teste

Origem: rekordbox 7.2.8, saída gravada por loopback (dossiê `evidence/p7-0-mixer-curvas.md`).

1. Fader 100 → 50: saída −12,0 dB. Teste: `mixer_tests::fader_trim_e_crossfader_nos_pontos_medidos`.
2. TRIM 0 → −2: saída −2,55 dB.
3. Crossfader centro → todo no deck 1: saída do deck 1 +1,06 dB.
4. Fader 0, TRIM −10 ou crossfader +10: mudo.

## Perguntas abertas

- Crossfader entre +9 e +10 (o VILELABOX cai linearmente em ganho de −21 dB ao mudo).
- Frequências de corte e formato do filtro do EQ (medir com tons puros).
- Fone; curvas do crossfader nas preferências; limitador do master.
