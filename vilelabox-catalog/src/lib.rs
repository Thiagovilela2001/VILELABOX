//! `vilelabox-catalog`: leitura do catálogo do rekordbox (spec/rekordbox/10-master-db.md).
//!
//! Lê só uma cópia SQLite **sem criptografia** gerada pelo usuário
//! (`tools/catalog/exportar_catalogo.py`); nenhuma chave entra aqui. O arquivo é
//! aberto imutável e somente leitura: nada é escrito nem criado ao lado dele.
//! Toda consulta filtra `rb_local_deleted = 0` na tabela principal e nas
//! associadas. Os nomes `djmd*` não saem deste crate.

use rusqlite::types::Value;
use rusqlite::{Connection, OpenFlags};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, thiserror::Error)]
pub enum ErroCatalogo {
    #[error("não foi possível abrir o catálogo: {0}")]
    Abrir(String),
    #[error("catálogo incompatível: falta a tabela {0}")]
    Esquema(&'static str),
    #[error("consulta ao catálogo falhou: {0}")]
    Consulta(#[from] rusqlite::Error),
}

/// Marcador de uma faixa: `tipo` 0 = memory cue/loop, 1..=8 = hot cue A..H.
#[derive(Debug, Clone, PartialEq)]
pub struct Cue {
    pub tipo: u32,
    pub entrada_ms: i64,
    /// Saída do loop; `None` quando não é loop (`OutMsec` ≤ 0).
    pub saida_ms: Option<i64>,
    pub rotulo: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Faixa {
    pub id: String,
    pub titulo: String,
    pub artista: String,
    pub album: String,
    pub genero: String,
    pub tom: String,
    /// 0..5.
    pub estrelas: u8,
    pub bpm: f64,
    pub duracao_s: u32,
    /// Caminho completo do arquivo de áudio (`FolderPath`).
    pub caminho: String,
    /// Caminho do `.DAT` relativo à pasta `share` do rekordbox (`AnalysisDataPath`).
    pub analise: Option<String>,
    pub cues: Vec<Cue>,
}

/// Playlist normal com o caminho das pastas no nome ("Pasta / Lista") e as
/// faixas na ordem (`TrackNo`), por `Faixa::id`.
#[derive(Debug, Clone, PartialEq)]
pub struct Lista {
    pub nome: String,
    pub faixas: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Catalogo {
    pub faixas: Vec<Faixa>,
    pub listas: Vec<Lista>,
}

const ESSENCIAIS: [&str; 4] = ["djmdContent", "djmdCue", "djmdPlaylist", "djmdSongPlaylist"];

/// `ID` é texto ou inteiro conforme a versão; normaliza para texto.
fn texto(v: Value) -> String {
    match v {
        Value::Text(s) => s,
        Value::Integer(i) => i.to_string(),
        Value::Real(f) => f.to_string(),
        Value::Null | Value::Blob(_) => String::new(),
    }
}

fn inteiro(v: Value) -> i64 {
    match v {
        Value::Integer(i) => i,
        Value::Real(f) => f as i64,
        Value::Text(s) => s.trim().parse().unwrap_or(0),
        Value::Null | Value::Blob(_) => 0,
    }
}

pub fn abrir(caminho: &Path) -> Result<Catalogo, ErroCatalogo> {
    let uri = format!(
        "file:{}?immutable=1",
        caminho.display().to_string().replace('\\', "/")
    );
    let flags = OpenFlags::SQLITE_OPEN_READ_ONLY
        | OpenFlags::SQLITE_OPEN_URI
        | OpenFlags::SQLITE_OPEN_NO_MUTEX;
    let con =
        Connection::open_with_flags(uri, flags).map_err(|e| ErroCatalogo::Abrir(e.to_string()))?;
    for t in ESSENCIAIS {
        let existe: bool = con
            .query_row(
                "select count(*) from sqlite_master where type = 'table' and name = ?1",
                [t],
                |r| r.get::<_, i64>(0),
            )
            .map(|n| n > 0)
            .map_err(|e| ErroCatalogo::Abrir(e.to_string()))?;
        if !existe {
            return Err(ErroCatalogo::Esquema(t));
        }
    }
    let mut cues = ler_cues(&con)?;
    let mut faixas = ler_faixas(&con)?;
    for f in &mut faixas {
        f.cues = cues.remove(&f.id).unwrap_or_default();
    }
    let listas = ler_listas(&con)?;
    Ok(Catalogo { faixas, listas })
}

fn ler_faixas(con: &Connection) -> Result<Vec<Faixa>, ErroCatalogo> {
    let mut st = con.prepare(
        "select c.ID, c.Title, a.Name, al.Name, g.Name, k.ScaleName, c.Rating, c.BPM, c.Length,
                c.FolderPath, c.AnalysisDataPath
         from djmdContent c
         left join djmdArtist a on a.ID = c.ArtistID and a.rb_local_deleted = 0
         left join djmdAlbum al on al.ID = c.AlbumID and al.rb_local_deleted = 0
         left join djmdGenre g on g.ID = c.GenreID and g.rb_local_deleted = 0
         left join djmdKey k on k.ID = c.KeyID and k.rb_local_deleted = 0
         where c.rb_local_deleted = 0
         order by c.created_at",
    )?;
    let linhas = st.query_map([], |r| {
        let g = |i: usize| r.get::<_, Value>(i);
        Ok(Faixa {
            id: texto(g(0)?),
            titulo: texto(g(1)?),
            artista: texto(g(2)?),
            album: texto(g(3)?),
            genero: texto(g(4)?),
            tom: texto(g(5)?),
            estrelas: inteiro(g(6)?).clamp(0, 5) as u8,
            // BPM inteiro escalado por 100 (12800 = 128,00).
            bpm: inteiro(g(7)?) as f64 / 100.0,
            duracao_s: inteiro(g(8)?).max(0) as u32,
            caminho: texto(g(9)?),
            analise: Some(texto(g(10)?)).filter(|s| !s.is_empty()),
            cues: Vec::new(),
        })
    })?;
    Ok(linhas.collect::<Result<_, _>>()?)
}

fn ler_cues(con: &Connection) -> Result<HashMap<String, Vec<Cue>>, ErroCatalogo> {
    let mut st = con.prepare(
        "select ContentID, Kind, InMsec, OutMsec, Comment from djmdCue
         where rb_local_deleted = 0 order by ContentID, InMsec",
    )?;
    let mut por_faixa: HashMap<String, Vec<Cue>> = HashMap::new();
    let linhas = st.query_map([], |r| {
        let g = |i: usize| r.get::<_, Value>(i);
        let saida = inteiro(g(3)?);
        Ok((
            texto(g(0)?),
            Cue {
                tipo: inteiro(g(1)?).clamp(0, 8) as u32,
                entrada_ms: inteiro(g(2)?),
                saida_ms: (saida > 0).then_some(saida),
                rotulo: Some(texto(g(4)?)).filter(|s| !s.is_empty()),
            },
        ))
    })?;
    for l in linhas {
        let (id, cue) = l?;
        por_faixa.entry(id).or_default().push(cue);
    }
    Ok(por_faixa)
}

struct No {
    id: String,
    pai: String,
    nome: String,
    atributo: i64,
}

fn ler_listas(con: &Connection) -> Result<Vec<Lista>, ErroCatalogo> {
    let mut st = con.prepare(
        "select ID, ParentID, Name, Attribute from djmdPlaylist
         where rb_local_deleted = 0 order by Seq",
    )?;
    let nos: Vec<No> = st
        .query_map([], |r| {
            let g = |i: usize| r.get::<_, Value>(i);
            Ok(No {
                id: texto(g(0)?),
                pai: texto(g(1)?),
                nome: texto(g(2)?),
                atributo: inteiro(g(3)?),
            })
        })?
        .collect::<Result<_, _>>()?;
    let mut st = con.prepare(
        "select PlaylistID, ContentID from djmdSongPlaylist
         where rb_local_deleted = 0 order by PlaylistID, TrackNo",
    )?;
    let mut faixas: HashMap<String, Vec<String>> = HashMap::new();
    for l in st.query_map([], |r| Ok((texto(r.get(0)?), texto(r.get(1)?))))? {
        let (lista, faixa) = l?;
        faixas.entry(lista).or_default().push(faixa);
    }
    let mut out = Vec::new();
    achatar(&nos, "root", "", &mut faixas, &mut out);
    Ok(out)
}

/// Pastas (`Attribute` 1) viram prefixo; playlists normais (0) entram com as
/// faixas. Listas inteligentes (4) ficam de fora: o conteúdo delas é uma
/// condição, não está em `djmdSongPlaylist`.
fn achatar(
    nos: &[No],
    pai: &str,
    prefixo: &str,
    faixas: &mut HashMap<String, Vec<String>>,
    out: &mut Vec<Lista>,
) {
    for n in nos.iter().filter(|n| n.pai == pai) {
        match n.atributo {
            1 => achatar(nos, &n.id, &format!("{prefixo}{} / ", n.nome), faixas, out),
            0 => out.push(Lista {
                nome: format!("{prefixo}{}", n.nome),
                faixas: faixas.remove(&n.id).unwrap_or_default(),
            }),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Catálogo sintético mínimo com as colunas lidas.
    fn fixture(dir: &Path) -> std::path::PathBuf {
        let p = dir.join("catalogo.db");
        let _ = std::fs::remove_file(&p);
        let c = Connection::open(&p).unwrap();
        c.execute_batch(
            "create table djmdContent (ID text, Title text, ArtistID text, AlbumID text, GenreID text,
               KeyID text, Rating int, BPM int, Length int, FolderPath text, AnalysisDataPath text,
               created_at text, rb_local_deleted int);
             create table djmdArtist (ID text, Name text, rb_local_deleted int);
             create table djmdAlbum (ID text, Name text, rb_local_deleted int);
             create table djmdGenre (ID text, Name text, rb_local_deleted int);
             create table djmdKey (ID text, ScaleName text, rb_local_deleted int);
             create table djmdCue (ContentID text, Kind int, InMsec int, OutMsec int, Comment text, rb_local_deleted int);
             create table djmdPlaylist (ID text, ParentID text, Name text, Attribute int, Seq int, rb_local_deleted int);
             create table djmdSongPlaylist (PlaylistID text, ContentID text, TrackNo int, rb_local_deleted int);
             insert into djmdArtist values ('a1', 'Artista', 0);
             insert into djmdKey values ('k1', 'Gm', 0);
             insert into djmdContent values ('1', 'Faixa', 'a1', null, null, 'k1', 4, 12800, 300,
               'C:/m/f.mp3', '/PIONEER/USBANLZ/x/ANLZ0000.DAT', '2026-01-01', 0);
             insert into djmdContent values ('2', 'Apagada', null, null, null, null, 0, 0, 0, 'C:/m/g.mp3', '', '2026-01-02', 1);
             insert into djmdCue values ('1', 0, 25, -1, '', 0);
             insert into djmdCue values ('1', 1, 37410, 39256, 'drop', 0);
             insert into djmdCue values ('1', 2, 1000, -1, '', 1);
             insert into djmdPlaylist values ('p', 'root', 'Pasta', 1, 1, 0);
             insert into djmdPlaylist values ('l', 'p', 'Set', 0, 1, 0);
             insert into djmdPlaylist values ('s', 'root', 'Inteligente', 4, 2, 0);
             insert into djmdSongPlaylist values ('l', '1', 1, 0);
             insert into djmdSongPlaylist values ('l', '2', 2, 1);",
        )
        .unwrap();
        p
    }

    #[test]
    fn le_faixas_cues_e_listas_ignorando_apagados() {
        let dir = std::env::temp_dir().join(format!("vilelabox-catalog-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let cat = abrir(&fixture(&dir)).unwrap();
        assert_eq!(cat.faixas.len(), 1);
        let f = &cat.faixas[0];
        assert_eq!(
            (f.titulo.as_str(), f.artista.as_str(), f.tom.as_str()),
            ("Faixa", "Artista", "Gm")
        );
        assert_eq!((f.bpm, f.duracao_s, f.estrelas), (128.0, 300, 4));
        assert_eq!(f.caminho, "C:/m/f.mp3");
        assert_eq!(
            f.analise.as_deref(),
            Some("/PIONEER/USBANLZ/x/ANLZ0000.DAT")
        );
        assert_eq!(f.cues.len(), 2);
        assert_eq!(
            f.cues[0],
            Cue {
                tipo: 0,
                entrada_ms: 25,
                saida_ms: None,
                rotulo: None
            }
        );
        assert_eq!(f.cues[1].saida_ms, Some(39256));
        assert_eq!(
            cat.listas,
            [Lista {
                nome: "Pasta / Set".into(),
                faixas: vec!["1".into()]
            }]
        );
        // imutável: nada criado ao lado do banco
        assert!(!dir.join("catalogo.db-wal").exists());
    }

    #[test]
    fn esquema_incompleto_da_erro_tipado() {
        let dir =
            std::env::temp_dir().join(format!("vilelabox-catalog-vazio-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("vazio.db");
        let _ = std::fs::remove_file(&p);
        Connection::open(&p)
            .unwrap()
            .execute_batch("create table x (a int);")
            .unwrap();
        assert!(matches!(
            abrir(&p),
            Err(ErroCatalogo::Esquema("djmdContent"))
        ));
    }
}
