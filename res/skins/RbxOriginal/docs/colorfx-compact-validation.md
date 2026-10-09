# COLOR FX compacto — validação

2026-10-08. Um único seletor central substitui os seis botões de COLOR FX. Cada canal mantém seu knob CFX. Não há fixação automática do preset FILTER.

O widget nativo recebeu o elemento opcional TargetEffectUnitGroup, restrito a uma segunda cadeia QuickEffect válida. Widgets sem esse elemento mantêm o comportamento anterior. Selecionar um preset aplica o mesmo preset aos dois canais; selecionar --- remove o efeito dos dois. Os nomes dos presets são usados, sem índices fixos.

- Compilação do executável Blackmesa aprovada.
- XMLs, referências, templates e SVGs válidos.
- Testes em perfil isolado: FILTER → FILTER/FILTER; DUB ECHO → DUB ECHO/DUB ECHO; --- → ---/---.
- Skin carregada sem erros de parsing ou controles inexistentes.
- Layout conferido em 1920×1080 e 1600×860.

Capturas offscreen; o áudio não foi avaliado. Os presets e a configuração principais não foram alterados. O executável anterior e os dois arquivos de código foram preservados em mixxx-source/backups-colorfx/20261008-213421.
