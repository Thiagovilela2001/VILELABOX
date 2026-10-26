# Armazenamento de cues

> **Atualização (P5.1, 2026-10-26):** o destino principal passou a ser o banco
> próprio, via `vilelabox-core` (`vilelabox-app/src/nucleo.rs`): cues do
> usuário (exceto loops) com origem `User`, em frames, com rótulo e cor. O
> JSON descrito abaixo continua como reserva enquanto a taxa de amostragem da
> faixa não é conhecida (antes de o áudio ser decodificado) e é migrado para o
> banco na primeira decodificação. ADAPTAÇÃO: o banco devolve os cues em ordem
> de posição, não na ordem de gravação do JSON.

Decisão de produto. Cue é comportamento de referência; o armazenamento é nosso.

## Por que arquivo próprio e não `djmdCue`

O rekordbox grava memory cue e hot cue em `djmdCue` do `master.db` (`Kind 0` memory, `Kind 1`
hot, posição em `InMsec`) e não toca o ANLZ (`evidence/p6-4-memory-persistencia.md`). O banco é
SQLCipher, 20 MB, centenas de tabelas. Copiar isso traria criptografia, esquema alheio e risco de
corromper a coleção do usuário, sem ganho para o comportamento. O VILELABOX não lê nem escreve
`master.db`.

## Formato

Um JSON por faixa (`vilelabox-app/src/cues_store.rs`):

```json
{
  "versao": 1,
  "faixa": "C:\\Musica\\Artista - Faixa (Mix).mp3",
  "cues": [
    { "tipo": "hot", "slot": 3, "pos_s": 12.5 },
    { "tipo": "memory", "pos_s": 51.718 }
  ]
}
```

- `tipo`: `memory` ou `hot`; `slot` (1..8 = A..H) só em hot cue; `pos_s` em segundos.
- Ordem da lista = ordem em memória (`DeckState.cues`).
- Hot cue no mesmo slot substitui o anterior (`sync::gravar_hot_cue`).

## Chave e local

- Pasta: `%APPDATA%\vilelabox\cues\` (criada na primeira gravação). A base é parâmetro do módulo.
- Nome: `<nome base sanitizado>-<FNV-1a 64 bits do caminho em bytes, 16 hex>.json`. O nome base
  (sem extensão, só `[A-Za-z0-9_-]`, até 40 caracteres) serve para reconhecer a olho; a unicidade vem
  do hash. Mesmo caminho, mesmo arquivo, em qualquer máquina.

## Atomicidade e tolerância

- Escrita em `<nome>.json.tmp`, `sync_all`, `rename` por cima. Erro remove o `.tmp` e vira aviso na
  barra de status; o app segue.
- Leitura: arquivo ausente, JSON inválido, versão desconhecida ou campo faltando dão lista vazia.

## Ligação no app

- Carregar faixa (`Command::LoadToDeck` em `state.rs`): se o arquivo da faixa tem cues, eles
  substituem os cues vindos do XML/ANLZ; loops da origem são mantidos.
- MEMORY e hot cue (`main.rs`, após `sync::gravar_*`): `AppState::salvar_cues` grava na hora.

## Limites conhecidos

- Faixa sem caminho (sem `PPTH` nem `Location`) não tem cues salvos: a chave sai do
  caminho, então sem ele não há arquivo.
- Lista vazia não persiste: a leitura só aplica os cues salvos quando a lista é não vazia,
  então um deck sem cue volta a valer o XML/ANLZ. Enquanto não houver comando de apagar cue,
  isso não incomoda; quando houver, precisa ser revisto.

## Fora do escopo

- Loops, cor, comentário e rótulo do cue (não são gravados).
- Apagar cue; migrar arquivo quando a faixa muda de caminho.
- Exportar para o rekordbox (XML/ANLZ) ou importar do `master.db`.
