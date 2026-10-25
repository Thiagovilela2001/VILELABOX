# G0 — Dossiê de proveniência: Catálogo Principal `master.db` (P2.1)

## Artefato analisado

- **Arquivo:** `rekordbox.exe`
- **Versão:** `7.2.8.0`
- **SHA-256:** `85e3a58524c5cd1e870ad59e23cc8806523175f80fb82d6059c68d57bdf9991a`
- **Registro privado detalhado:** `third_party/rekordbox/re/evidence/10-master-db.md`

## Identificadores de classes, tabelas virtuais e rotinas observadas

| Entidade / Repositório | Símbolo RTTI | Vtable (RVA) | Rotinas analisadas (RVA) |
| --- | --- | --- | --- |
| Banco principal | `rb::middleware::db::rekordbox::MasterDB` / `MasterDbFile` | `vft_3997588` / `vft_398f5d8` | `rva_1603950`, `rva_1ece060` |
| Faixas (`djmdContent`) | `rb::middleware::db::rekordbox::ContentDao` / `SqLiteDao<Content>` | `vft_3991ac8` / `vft_3991b08` | `rva_1ee4c50`, `rva_1ee55f0`, `rva_1ee6a40`, `rva_1f5ebc0`, `rva_1ee4160` |
| Cues (`djmdCue`) | `rb::middleware::db::rekordbox::CueDao` / `SqLiteDao<Cue>` | `vft_39920b0` / `vft_3992040` | `rva_1ee8770`, `rva_1ee8c40`, `rva_1ee9ae0`, `rva_1f636f0`, `rva_1ee8060` |
| Playlists (`djmdPlaylist`) | `rb::middleware::db::rekordbox::PlaylistDao` / `SqLiteDao<Playlist>` | `vft_3992880` / `vft_39928b0` | `rva_1eee7e0`, `rva_1eef180`, `rva_1ef0f50`, `rva_1f6c9d0`, `rva_1eee360` |
| Vínculo playlist-faixa (`djmdSongPlaylist`) | `rb::middleware::db::rekordbox::SongPlaylistDao` / `SqLiteDao<SongPlaylist>` | `vft_3993e80` / `vft_3993eb0` | `rva_1efd690`, `rva_1efdc30`, `rva_1effde0`, `rva_1f75710`, `rva_1efd210` |
| Entidades auxiliares | `SqLiteDao<Artist>`, `Album`, `Genre`, `Key`, `Label`, `Color`, `Property`, `History`, `MyTag` | `vft_39aea78`..`vft_39aeba8` | `rva_2046400`, `rva_2049560`, `rva_20584c0`, `rva_205f6f0`, `rva_206de50`, `rva_2071280` |

## Cadeias de chamadas e consultas SQL observadas

1. **Fluxo de consulta e desserialização (`SqLiteDao<T>::Select` -> `execute` -> desserializador de linha):**
   - `SqLiteDao<Content>::Select` (`rva_1ee55f0`) executa a query via `rva_1ee6a40`, itera sobre os registros retornados e invoca `rva_1f5ebc0` para construir cada instância `Content` (`0x3e0` bytes), destruída por `rva_1ee4160`.
   - `SqLiteDao<Cue>::Select` (`rva_1ee8c40`) invoca `rva_1ee9ae0` e desserializa cada linha via `rva_1f636f0` em uma estrutura `Cue` (`0x170` bytes), destruída por `rva_1ee8060`.
   - `SqLiteDao<Playlist>::Select` (`rva_1eef180`) invoca `rva_1ef0f50` e desserializa cada linha via `rva_1f6c9d0` em uma estrutura `Playlist` (`0x100` bytes), destruída por `rva_1eee360`.
   - `SqLiteDao<SongPlaylist>::Select` (`rva_1efdc30`) invoca `rva_1effde0` e desserializa cada linha via `rva_1f75710`.
2. **Consultas e projeções SQL recuperadas das tabelas de strings (`str_39b5da0`..`str_3a1c600`):**
   - `djmdContent` (`str_39b5da0`): projeta `ID`, `FolderPath`, `FileNameL`, `Title`, `ArtistID`, `AlbumID`, `GenreID`, `BPM`, `Length`, `TrackNo`, `BitRate`, `BitDepth`, `Commnt`, `FileType`, `Rating`, `ReleaseYear`, `RemixerID`, `LabelID`, `OrgArtistID`, `KeyID`, `StockDate`, `ColorID`, `DJPlayCount`, `ImagePath`, `MasterDBID`, `MasterSongID`, `AnalysisDataPath`, `FileSize`, `DiscNo`, `ComposerID`, `Subtitle`, `SampleRate`, `Analysed`, `ReleaseDate`, `DateCreated`, `ContentLink`, `ModifiedByRBM`, `HotCueAutoLoad`, `DeliveryControl`, `DeliveryComment`, `Lyricist`, `ISRC`, `SamplerTrackInfo`, `SamplerPlayOffset`, `SamplerGain`, `VideoAssociate`, `LyricStatus`, `ServiceID`, `OrgFolderPath`, `Reserved1`, `ExtInfo`, `rb_file_id`, `DeviceID`, `created_at`, `UUID` filtrando `where rb_local_deleted = 0`.
   - `djmdCue` (`str_39b6df0`): projeta `ID`, `ContentID`, `InMsec`, `InFrame`, `InMpegFrame`, `InMpegAbs`, `OutMsec`, `OutFrame`, `OutMpegFrame`, `OutMpegAbs`, `Kind`, `Color`, `ColorTableIndex`, `ActiveLoop`, `Comment`, `BeatLoopSize`, `CueMicrosec`, `InPointSeekInfo`, `OutPointSeekInfo` filtrando `where rb_local_deleted = 0`.
   - `djmdPlaylist` (`str_39b75c0`, `str_3a1c5b0`, `str_3a1c600`): projeta `ID`, `UUID`, `Seq`, `Name`, `ImagePath`, `Attribute`, `ParentID`, `SmartList`, `rb_local_deleted`; raiz identificada por `ParentID = "root"`.
   - `djmdSongPlaylist` (`str_39b4490`, `str_3a1c250`..`str_3a1c3e8`): associa `PlaylistID`, `ContentID` e `TrackNo`, ordenando por `djmdSongPlaylist.TrackNo`.
   - Joins de dimensão (`str_3a1acc0`..`str_3a1b120`): `left outer join` de `djmdContent` com `djmdArtist` (`ArtistID`, `RemixerID`, `OrgArtistID`), `djmdAlbum` (`AlbumID`), `djmdGenre` (`GenreID`), `djmdLabel` (`LabelID`), `djmdKey` (`KeyID` -> `ScaleName`) e `djmdColor` (`ColorID` -> `Commnt`, `SortKey`, `ColorCode`).

## Classificação das conclusões

- **OBSERVADO:**
  - Esquema completo de colunas e tipos das tabelas `djmdContent`, `djmdCue`, `djmdPlaylist`, `djmdSongPlaylist`, `djmdArtist`, `djmdAlbum`, `djmdGenre`, `djmdKey`, `djmdLabel`, `djmdColor`, `djmdHistory`, `djmdSongHistory`, `djmdMyTag` e `djmdProperty`.
  - Convenção de exclusão lógica `rb_local_deleted = 0` em todas as leituras e metadados comuns de sincronização (`UUID`, `rb_data_status`, `rb_local_deleted`, `rb_local_usn`, `rb_local_synced`, `created_at`, `updated_at`) processados por `rva_1f692d0`.
- **INFERIDO:**
  - Escala de `djmdContent.BPM` em centésimos de BPM (`BPM / 100.0`) e `djmdContent.Length` em segundos inteiros, consistente com as consultas de filtro por intervalo (`str_3a1a690`, `str_3a1a960`).
- **DESCONHECIDO:**
  - Derivação de chave criptográfica em `MasterDbKey` (`vft_398f688`), deliberadamente fora do escopo por restrição clean-room e barreira contra chaves/DRM.
- **ADAPTAÇÃO:**
  - O futuro crate `vilelabox-catalog` lerá bancos SQLite já descriptografados ou fixtures sintéticas minimizadas em modo estritamente somente leitura (`SQLITE_OPEN_READ_ONLY`), sem embutir chaves SQLCipher.

## Limitações da análise estática e confirmação

- Como `master.db` em disco usa SQLCipher, a estrutura das tabelas e queries foi integralmente recuperada das strings SQL compiladas e dos desserializadores `SqLiteDao<T>` no binário `rekordbox.exe`.

## Rastreabilidade para especificação

- **Especificação:** [`spec/rekordbox/10-master-db.md`](../10-master-db.md)
