# G0 — Integração XML ↔ análises locais

Observações feitas ao ligar o documento XML (12-xml) às análises da pasta
`PIONEER/USBANLZ` do perfil, em 2026-10-24, rekordbox 7.2.8 para Windows.

## Observações

| afirmação | evidência | classificação |
| --- | --- | --- |
| O `PPTH` das análises **locais** guarda só `?/<nome do arquivo>`, sem a pasta | faixa de referência: `?/DJ Cali - ULTRA PIROCADA DE BANDIDO.mp3` | OBSERVADO |
| As análises locais **não** guardam cues: `PCOB`/`PCO2` existem com 0 entradas | 2661 análises varridas, nenhuma com `PCPT`/`PCP2`; a faixa de referência tinha 4 disparos rápidos, 1 memória e 1 loop no programa | OBSERVADO |
| Os cues vivem no catálogo e saem no XML (`POSITION_MARK`) e, presumivelmente, no ANLZ exportado para USB | os mesmos cues aparecem na exportação XML | INFERIDO (USB não verificado) |
| Casar XML ↔ análise pelo nome do arquivo cobre 3042 de 3088 faixas da coleção | contagem no aplicativo | OBSERVADO |

## Consequência para o VILELABOX

- Coleção, tom, metadados, cues e listas vêm do XML (`vilelabox-xml`).
- Grade e formas de onda vêm da análise casada pelo nome do arquivo; com mais de
  uma análise de mesmo nome, desempata a duração (±2 s). Sem análise, a grade
  é montada dos `TEMPO` do XML. Essa política de casamento é `ADAPTAÇÃO`.
- O catálogo (`10-master-db`) seria a fonte direta dos cues sem exportar XML.

## Rastreabilidade

- Leitor/escritor: `vilelabox-xml/src/lib.rs`, `vilelabox-xml/src/location.rs`
- Casamento: `vilelabox-app/src/library.rs` (`load_xml`, `chave_arquivo`)
- Teste com exportação real: `VLBX_XML=<arquivo.xml> cargo test -p vilelabox-xml --test exportacao_real`
