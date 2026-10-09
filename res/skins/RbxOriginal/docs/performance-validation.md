# Validação dos controles de performance

2026-10-08. Build Blackmesa/Mixxx 2.5.6, Qt offscreen, áudio ALSA nulo e perfil isolado em /tmp. A biblioteca foi copiada com SQLite backup; a configuração principal não foi alterada.

- 39 XMLs válidos.
- Checagem de recursos: 47 referências, 37 templates e 52 SVGs aprovados.
- Carregamento da skin confirmado, sem erros de parsing nem controles de engine inexistentes criados pela skin.
- Cliques nos seletores dos quatro modos de pads: hot cues visíveis no primeiro modo e ocultos nos demais.
- Cliques nos seletores RANGE produziram 6%, 10%, 16% e 50% no display nativo.
- Renderização do widget da skin aprovada em 1920x1080 e 1600x860, com sampler e FX visíveis.

O teste não reproduziu faixas nem avaliou áudio de sampler, loops ou efeitos. Os controles usam os nomes nativos confirmados no código do Mixxx. Capturas em modo offscreen, sem faixas carregadas.
