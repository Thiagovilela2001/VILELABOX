# 10 — Catálogo Principal (`master.db`)

Especificação limpa do esquema relacional, consultas de leitura, relacionamentos
e invariantes do catálogo principal (`master.db`) para interoperabilidade
somente leitura.

Proveniência G0: [`evidence/p2-1-master-db.md`](evidence/p2-1-master-db.md).

## Escopo e fora do escopo

**Escopo.** Tabelas principais do catálogo (`djmdContent`, `djmdCue`,
`djmdPlaylist`, `djmdSongPlaylist`), tabelas de dimensão (`djmdArtist`,
`djmdAlbum`, `djmdGenre`, `djmdKey`, `djmdLabel`, `djmdColor`, `djmdProperty`,
`djmdHistory`, `djmdSongHistory`, `djmdMyTag`, `djmdSongMyTag`), convenção de
exclusão lógica (`rb_local_deleted`), unidades de medida e contrato do futuro
adaptador `vilelabox-catalog`.

**Fora do escopo.** Derivação de chaves SQLCipher, mecanismos de licença,
autenticação em nuvem e escrita no catálogo proprietário. O VILELABOX nunca
embute chaves criptográficas de terceiros nem modifica o banco real do usuário.

## Convenções globais do esquema

Todas as tabelas gerenciadas pelos repositórios `SqLiteDao<T>` compartilham os
campos de controle de sincronização e ciclo de vida:

| Coluna | Tipo lógico | Papel |
| --- | --- | --- |
| `ID` | texto / inteiro | Identificador primário do registro |
| `UUID` | texto | Identificador único universal da entidade |
| `rb_data_status` | inteiro | Estado local de modificação do registro |
| `rb_local_deleted` | inteiro (`0` ou `1`) | Exclusão lógica (`0` = ativo, `1` = removido) |
| `rb_local_usn` | inteiro | Número de sequência de atualização local |
| `rb_local_synced` | inteiro | Indicador de sincronização concluída |
| `created_at` | texto (timestamp) | Data/hora de criação do registro |
| `updated_at` | texto (timestamp) | Data/hora da última atualização |

**Regra normativa.** Toda leitura de catálogo deve filtrar `rb_local_deleted = 0`
tanto na tabela principal quanto nas tabelas associadas por `join`.

## Tabela de faixas (`djmdContent`)

Armazena os metadados musicais, técnicos e de localização de cada faixa da
coleção:

| Coluna | Tipo lógico | Descrição e unidade |
| --- | --- | --- |
| `ID` | chave primária | Identificador único da faixa |
| `FolderPath` | texto | Caminho completo ou diretório de origem do arquivo de áudio |
| `FileNameL` | texto | Nome longo do arquivo de áudio |
| `Title` | texto | Título da faixa |
| `Subtitle` | texto | Subtítulo ou versão da faixa |
| `ArtistID` | chave estrangeira | Referência a `djmdArtist.ID` (artista principal) |
| `OrgArtistID` | chave estrangeira | Referência a `djmdArtist.ID` (artista original) |
| `RemixerID` | chave estrangeira | Referência a `djmdArtist.ID` (remixer) |
| `ComposerID` | chave estrangeira | Referência a `djmdArtist.ID` (compositor) |
| `AlbumID` | chave estrangeira | Referência a `djmdAlbum.ID` |
| `GenreID` | chave estrangeira | Referência a `djmdGenre.ID` |
| `LabelID` | chave estrangeira | Referência a `djmdLabel.ID` |
| `KeyID` | chave estrangeira | Referência a `djmdKey.ID` (tonalidade musical) |
| `ColorID` | chave estrangeira | Referência a `djmdColor.ID` (cor atribuída à faixa) |
| `BPM` | inteiro | Andamento multiplicado por `100` (ex.: `12800` = `128.00` BPM) |
| `Length` | inteiro | Duração da faixa em segundos |
| `SampleRate` | inteiro | Taxa de amostragem em Hz (ex.: `44100`, `48000`) |
| `BitRate` | inteiro | Taxa de bits em kbps |
| `BitDepth` | inteiro | Profundidade de bits por amostra |
| `FileSize` | inteiro | Tamanho do arquivo de áudio em bytes |
| `FileType` | inteiro | Código do formato de arquivo de áudio |
| `TrackNo` | inteiro | Número da faixa no álbum |
| `DiscNo` | inteiro | Número do disco |
| `ReleaseYear` | inteiro | Ano de lançamento |
| `ReleaseDate` | texto | Data de lançamento |
| `DateCreated` | texto | Data de criação/importação na coleção |
| `StockDate` | texto | Data de inclusão no acervo |
| `Rating` | inteiro | Classificação de `0` a `5` |
| `DJPlayCount` | inteiro | Contagem de reproduções |
| `Commnt` | texto | Comentário do usuário (grafia literal da coluna: `Commnt`) |
| `AnalysisDataPath` | texto | Caminho relativo sob `PIONEER/USBANLZ` para o par `.DAT`/`.EXT` |
| `ImagePath` | texto | Caminho relativo da arte de capa |
| `Analysed` | inteiro | Flags indicando etapas de análise concluídas |
| `HotCueAutoLoad` | texto / inteiro | Estado de carregamento automático de hot cues |
| `ISRC` | texto | Código ISRC da gravação |

## Tabela de marcadores e loops (`djmdCue`)

Armazena memory cues, hot cues e loops vinculados a cada faixa (`ContentID`):

| Coluna | Tipo lógico | Descrição e unidade |
| --- | --- | --- |
| `ID` | chave primária | Identificador do marcador |
| `ContentID` | chave estrangeira | Referência a `djmdContent.ID` |
| `Kind` | inteiro | `0` = memory cue/loop; `1..=8` = hot cue A..H |
| `InMsec` | inteiro | Posição inicial em milissegundos |
| `OutMsec` | inteiro | Posição final do loop em milissegundos (`-1` ou `0` quando não é loop) |
| `CueMicrosec` | inteiro | Ajuste fino de posição em microssegundos |
| `Comment` | texto | Rótulo (`label`) associado ao marcador |
| `Color` | inteiro | Código de cor atribuído ao marcador |
| `ColorTableIndex` | inteiro | Índice na tabela de cores |
| `ActiveLoop` | inteiro | Indicador de loop ativo (`1` = ativo, `0` = inativo) |
| `BeatLoopSize` | inteiro | Tamanho de loop automático em batidas |
| `InFrame` / `OutFrame` | inteiro | Posição inicial/final em quadros |
| `InMpegFrame` / `OutMpegFrame` | inteiro | Quadro MPEG inicial/final |
| `InMpegAbs` / `OutMpegAbs` | inteiro | Offset absoluto MPEG inicial/final |
| `InPointSeekInfo` / `OutPointSeekInfo` | blob / texto | Dados auxiliares de busca nos pontos de entrada e saída |

Índice nativo observado para consulta por faixa: `(ContentID, rb_local_deleted)`.

## Tabelas de playlists (`djmdPlaylist` e `djmdSongPlaylist`)

### Hierarquia de playlists (`djmdPlaylist`)

| Coluna | Tipo lógico | Descrição |
| --- | --- | --- |
| `ID` | chave primária | Identificador da playlist ou pasta |
| `ParentID` | texto | `"root"` para itens de nível superior ou `ID` da pasta pai |
| `Seq` | inteiro | Ordem de exibição entre irmãos do mesmo `ParentID` |
| `Name` | texto | Nome da playlist ou pasta |
| `Attribute` | inteiro | Tipo do nó (`0` = playlist normal, `1` = pasta/diretório, `4` = lista inteligente) |
| `SmartList` | texto (XML/JSON) | Condições de filtro quando `Attribute` representa lista inteligente |
| `ImagePath` | texto | Caminho de imagem associada à playlist |

### Associação faixa-playlist (`djmdSongPlaylist`)

| Coluna | Tipo lógico | Descrição |
| --- | --- | --- |
| `ID` | chave primária | Identificador do vínculo |
| `PlaylistID` | chave estrangeira | Referência a `djmdPlaylist.ID` |
| `ContentID` | chave estrangeira | Referência a `djmdContent.ID` |
| `TrackNo` | inteiro | Posição ordenada (`1`-based) da faixa dentro da playlist |

Consulta canônica de faixas de uma playlist:

- `inner join` entre `djmdContent` e `djmdSongPlaylist` por
  `djmdContent.ID = djmdSongPlaylist.ContentID`;
- filtro `djmdContent.rb_local_deleted = 0 and djmdSongPlaylist.rb_local_deleted = 0 and djmdSongPlaylist.PlaylistID = <id>`;
- ordenação `order by djmdSongPlaylist.TrackNo`.

## Tabelas de dimensão e histórico

| Tabela | Colunas principais | Relação com `djmdContent` |
| --- | --- | --- |
| `djmdArtist` | `ID`, `Name`, `SearchStr` | `ArtistID`, `RemixerID`, `OrgArtistID`, `ComposerID` |
| `djmdAlbum` | `ID`, `Name`, `AlbumArtistID`, `ImagePath`, `Compilation` | `AlbumID` (e `AlbumArtistID -> djmdArtist.ID`) |
| `djmdGenre` | `ID`, `Name` | `GenreID` |
| `djmdLabel` | `ID`, `Name` | `LabelID` |
| `djmdKey` | `ID`, `ScaleName`, `Seq` | `KeyID` (`ScaleName` fornece a tonalidade textual) |
| `djmdColor` | `ID`, `ColorCode`, `SortKey`, `Commnt` | `ColorID` (`Commnt` fornece o rótulo da cor) |
| `djmdMyTag` / `djmdSongMyTag` | `ID`, `Name`, `ParentID`, `Seq` / `MyTagID`, `ContentID`, `TrackNo` | Tags hierárquicas definidas pelo usuário |
| `djmdHistory` / `djmdSongHistory` | `ID`, `Name`, `Attribute`, `ParentID`, `Seq` / `HistoryID`, `ContentID`, `TrackNo` | Histórico de sessões reproduzidas |
| `djmdProperty` | `DBID`, `DBVersion`, `BaseDBDrive`, `CurrentDBDrive`, `DeviceID` | Metadados globais e versão do banco |

## Contrato para `vilelabox-catalog` (somente leitura)

1. Abrir o arquivo SQLite sempre com `OpenFlags::SQLITE_OPEN_READ_ONLY` e sem
   criar arquivos auxiliares de escrita;
2. Operar sobre bancos SQLite descriptografados pelo próprio usuário ou fixtures
   sintéticas criadas em testes automatizados, sem incluir chaves SQLCipher no
   repositório;
3. Verificar a presença das tabelas essenciais (`djmdContent`, `djmdPlaylist`,
   `djmdSongPlaylist`, `djmdCue`) e retornar erro tipado para esquemas ou
   versões incompatíveis;
4. Converter registros de faixa, cues (`InMsec`, `OutMsec`, `Kind`, `Comment`,
   `Color`) e árvore de playlists para os tipos do domínio sem vazar nomes de
   tabelas `djmd*` para `vilelabox-core`.

## Implementação (2026-10-25)

- Crate `vilelabox-catalog`: abre a cópia com `immutable=1` e
  `SQLITE_OPEN_READ_ONLY` (nenhum `-wal`/`-shm` criado), confere as quatro
  tabelas essenciais (`ErroCatalogo::Esquema`) e devolve faixas, cues e
  playlists normais com o caminho das pastas no nome (`"Pasta / Lista"`). Listas
  inteligentes (`Attribute = 4`) ficam de fora.
- A cópia sem criptografia é gerada pelo usuário com
  `tools/catalog/exportar_catalogo.py`, que copia `master.db` para uma pasta
  temporária e exporta pela biblioteca externa `pyrekordbox` para
  `%LOCALAPPDATA%\vilelabox\catalogo.db`. O original nunca é aberto e nenhuma
  chave entra no repositório.
- No app: `cargo run -p vilelabox-app -- <catalogo.db>`. Grade e formas de onda
  vêm do `.DAT`/`.EXT` apontado por `AnalysisDataPath`, relativo à pasta `share`.
- Observado na coleção do usuário: 3091 faixas, 258 cues, 10 playlists.
  `FolderPath` é o caminho completo do arquivo (com o nome). 2239 caminhos
  existem como estão; os demais foram movidos de pasta depois da importação, e a
  busca por nome do app encontra 764 deles (3003 no total).
- `djmdCue.Color` é um código, não RGB; sem mapeamento observado, os cues do
  catálogo entram sem cor própria.
