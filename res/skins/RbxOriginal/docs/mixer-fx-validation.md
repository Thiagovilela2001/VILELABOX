# FX no mixer — validação

2026-10-08. Mixxx/Blackmesa 2.5.6, perfil temporário, áudio nulo e Qt offscreen.

- Painel horizontal removido; duas unidades integradas abaixo dos COLOR FX, ao lado dos VUs master.
- 40 XMLs válidos; referências a recursos, 38 templates e 52 SVGs aprovados.
- Carregamento final sem erros de parsing, tooltip ou controles de efeito inexistentes.
- Cada seletor apresentou 25 opções; seleção de Echo confirmada nas duas unidades.
- ON: estado 0 → 1 confirmado nas duas unidades. O botão escreve no enabled do primeiro efeito e da unidade.
- Roteamento independente: FX1/Canal1 1 → 0 → 1; FX1/Canal2 0 → 1 → 0; FX2/Canal1 0 → 1 → 0; FX2/Canal2 1 → 0 → 1.
- Capturas em 1920×1080 e 1600×860; tamanho mínimo preservado.

A configuração e a biblioteca principais não foram alteradas. Se FX estiver desmarcado na barra superior, marque para mostrar as unidades. Os testes verificaram interface e controles; não avaliaram o som produzido. O arquivo MERGE FX.zip do rekordbox foi inspecionado e contém samples WAV; não foi importado como plugin. Os seletores usam efeitos nativos do Mixxx.
