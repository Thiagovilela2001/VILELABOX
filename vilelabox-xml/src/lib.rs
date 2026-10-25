//! Documento de intercâmbio XML (`DJ_PLAYLISTS`), conforme
//! `spec/rekordbox/12-xml.md` (v2, conferida contra exportações reais).
//!
//! Independente de qualquer banco: lê para um modelo próprio e escreve de
//! volta na forma observada (ordem de atributos, casas decimais, recuo de 2).

pub mod location;

use std::fmt::Write as _;

#[derive(Debug, thiserror::Error)]
pub enum XmlError {
    #[error("XML malformado: {0}")]
    Sintaxe(#[from] roxmltree::Error),
    #[error("raiz não é DJ_PLAYLISTS")]
    Raiz,
}

/// `PRODUCT`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Product {
    pub name: String,
    pub version: String,
    pub company: String,
}

/// Trecho de grade (`TEMPO`).
#[derive(Debug, Clone, PartialEq)]
pub struct Tempo {
    pub inizio: f64,
    pub bpm: f64,
    pub metro: String,
    pub battito: u8,
}

/// Marca (`POSITION_MARK`). `num` -1 = memória; 0.. = disparo rápido A..
#[derive(Debug, Clone, PartialEq)]
pub struct Mark {
    pub name: String,
    pub kind: u8,
    pub start: f64,
    pub end: Option<f64>,
    pub num: i32,
    pub rgb: Option<(u8, u8, u8)>,
}

/// Registro de faixa (`TRACK` em `COLLECTION`). `location` já decodificada.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Track {
    pub track_id: String,
    pub name: String,
    pub artist: String,
    pub composer: String,
    pub album: String,
    pub grouping: String,
    pub genre: String,
    pub kind: String,
    pub size: u64,
    pub total_time: u32,
    pub disc_number: u32,
    pub track_number: u32,
    pub year: u32,
    pub average_bpm: f64,
    pub date_added: String,
    pub bit_rate: u32,
    pub sample_rate: u32,
    pub comments: String,
    pub play_count: u32,
    pub rating: u8,
    /// Caminho com `/`; `None` se o `Location` não pôde ser decodificado.
    pub location: Option<String>,
    pub remixer: String,
    pub tonality: String,
    pub label: String,
    pub mix: String,
    pub colour: Option<String>,
    pub tempos: Vec<Tempo>,
    pub marks: Vec<Mark>,
}

/// Nó da árvore de playlists.
#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Folder {
        name: String,
        children: Vec<Node>,
    },
    /// `keys`: `TrackID` (key_type 0) ou `Location` (key_type 1), na ordem.
    Playlist {
        name: String,
        key_type: u8,
        keys: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub version: String,
    pub product: Product,
    pub tracks: Vec<Track>,
    /// Pasta raiz (`ROOT`).
    pub playlists: Node,
}

impl Default for Document {
    fn default() -> Self {
        Self {
            version: "1.0.0".into(),
            product: Product::default(),
            tracks: Vec::new(),
            playlists: Node::Folder {
                name: "ROOT".into(),
                children: Vec::new(),
            },
        }
    }
}

// ---------------------------------------------------------------- leitura

fn attr<T: std::str::FromStr + Default>(n: roxmltree::Node, nome: &str) -> T {
    n.attribute(nome)
        .and_then(|v| v.trim().parse().ok())
        .unwrap_or_default()
}

fn texto(n: roxmltree::Node, nome: &str) -> String {
    n.attribute(nome).unwrap_or_default().to_string()
}

fn ler_track(n: roxmltree::Node) -> Track {
    let filhos = |tag: &'static str| n.children().filter(move |c| c.has_tag_name(tag));
    let rgb = |m: roxmltree::Node| -> Option<(u8, u8, u8)> {
        Some((
            m.attribute("Red")?.parse().ok()?,
            m.attribute("Green")?.parse().ok()?,
            m.attribute("Blue")?.parse().ok()?,
        ))
    };
    Track {
        track_id: texto(n, "TrackID"),
        name: texto(n, "Name"),
        artist: texto(n, "Artist"),
        composer: texto(n, "Composer"),
        album: texto(n, "Album"),
        grouping: texto(n, "Grouping"),
        genre: texto(n, "Genre"),
        kind: texto(n, "Kind"),
        size: attr(n, "Size"),
        total_time: attr(n, "TotalTime"),
        disc_number: attr(n, "DiscNumber"),
        track_number: attr(n, "TrackNumber"),
        year: attr(n, "Year"),
        average_bpm: attr(n, "AverageBpm"),
        date_added: texto(n, "DateAdded"),
        bit_rate: attr(n, "BitRate"),
        sample_rate: attr(n, "SampleRate"),
        comments: texto(n, "Comments"),
        play_count: attr(n, "PlayCount"),
        rating: attr(n, "Rating"),
        location: n.attribute("Location").and_then(location::decode),
        remixer: texto(n, "Remixer"),
        tonality: texto(n, "Tonality"),
        label: texto(n, "Label"),
        mix: texto(n, "Mix"),
        colour: n.attribute("Colour").map(str::to_string),
        tempos: filhos("TEMPO")
            .map(|t| Tempo {
                inizio: attr(t, "Inizio"),
                bpm: attr(t, "Bpm"),
                metro: n_or(t.attribute("Metro"), "4/4"),
                battito: attr(t, "Battito"),
            })
            .collect(),
        marks: filhos("POSITION_MARK")
            .map(|m| Mark {
                name: texto(m, "Name"),
                kind: attr(m, "Type"),
                start: attr(m, "Start"),
                end: m.attribute("End").and_then(|v| v.parse().ok()),
                num: m
                    .attribute("Num")
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(-1),
                rgb: rgb(m),
            })
            .collect(),
    }
}

fn n_or(v: Option<&str>, padrao: &str) -> String {
    v.unwrap_or(padrao).to_string()
}

fn ler_node(n: roxmltree::Node) -> Node {
    let name = texto(n, "Name");
    if attr::<u8>(n, "Type") == 1 {
        Node::Playlist {
            name,
            key_type: attr(n, "KeyType"),
            keys: n
                .children()
                .filter(|c| c.has_tag_name("TRACK"))
                .filter_map(|c| c.attribute("Key").map(str::to_string))
                .collect(),
        }
    } else {
        Node::Folder {
            name,
            children: n
                .children()
                .filter(|c| c.has_tag_name("NODE"))
                .map(ler_node)
                .collect(),
        }
    }
}

/// Lê um documento. Atributos ausentes ficam com valor padrão; atributos e
/// elementos desconhecidos são ignorados.
pub fn parse(xml: &str) -> Result<Document, XmlError> {
    let doc = roxmltree::Document::parse(xml)?;
    let raiz = doc.root_element();
    if !raiz.has_tag_name("DJ_PLAYLISTS") {
        return Err(XmlError::Raiz);
    }
    let filho = |tag: &str| raiz.children().find(|c| c.has_tag_name(tag));
    let product = filho("PRODUCT").map_or_else(Product::default, |p| Product {
        name: texto(p, "Name"),
        version: texto(p, "Version"),
        company: texto(p, "Company"),
    });
    let tracks = filho("COLLECTION")
        .map(|c| {
            c.children()
                .filter(|t| t.has_tag_name("TRACK"))
                .map(ler_track)
                .collect()
        })
        .unwrap_or_default();
    let playlists = filho("PLAYLISTS")
        .and_then(|p| p.children().find(|c| c.has_tag_name("NODE")))
        .map_or_else(|| Document::default().playlists, ler_node);
    Ok(Document {
        version: n_or(raiz.attribute("Version"), "1.0.0"),
        product,
        tracks,
        playlists,
    })
}

// ---------------------------------------------------------------- escrita

fn esc(v: &str) -> String {
    let mut o = String::with_capacity(v.len());
    for c in v.chars() {
        match c {
            '&' => o.push_str("&amp;"),
            '<' => o.push_str("&lt;"),
            '>' => o.push_str("&gt;"),
            '"' => o.push_str("&quot;"),
            '\'' => o.push_str("&apos;"),
            _ => o.push(c),
        }
    }
    o
}

fn escrever_node(o: &mut String, n: &Node, nivel: usize) {
    let pad = "  ".repeat(nivel);
    match n {
        Node::Folder { name, children } => {
            let _ = write!(
                o,
                "{pad}<NODE Type=\"0\" Name=\"{}\" Count=\"{}\"",
                esc(name),
                children.len()
            );
            if children.is_empty() {
                o.push_str("/>\n");
            } else {
                o.push_str(">\n");
                for c in children {
                    escrever_node(o, c, nivel + 1);
                }
                let _ = writeln!(o, "{pad}</NODE>");
            }
        }
        Node::Playlist {
            name,
            key_type,
            keys,
        } => {
            let _ = write!(
                o,
                "{pad}<NODE Name=\"{}\" Type=\"1\" KeyType=\"{key_type}\" Entries=\"{}\"",
                esc(name),
                keys.len()
            );
            if keys.is_empty() {
                o.push_str("/>\n");
            } else {
                o.push_str(">\n");
                for k in keys {
                    let _ = writeln!(o, "{pad}  <TRACK Key=\"{}\"/>", esc(k));
                }
                let _ = writeln!(o, "{pad}</NODE>");
            }
        }
    }
}

/// Escreve o documento na ordem e no formato observados no programa de
/// referência. Sem a quebra de linha de atributos longos (sem significado).
pub fn write(d: &Document) -> String {
    let mut o = String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\n");
    let _ = writeln!(o, "<DJ_PLAYLISTS Version=\"{}\">", esc(&d.version));
    let p = &d.product;
    let _ = writeln!(
        o,
        "  <PRODUCT Name=\"{}\" Version=\"{}\" Company=\"{}\"/>",
        esc(&p.name),
        esc(&p.version),
        esc(&p.company)
    );
    let _ = writeln!(o, "  <COLLECTION Entries=\"{}\">", d.tracks.len());
    for t in &d.tracks {
        let loc = t
            .location
            .as_deref()
            .map(location::encode)
            .unwrap_or_default();
        let _ = write!(
            o,
            "    <TRACK TrackID=\"{}\" Name=\"{}\" Artist=\"{}\" Composer=\"{}\" Album=\"{}\" Grouping=\"{}\" Genre=\"{}\" Kind=\"{}\" Size=\"{}\" TotalTime=\"{}\" DiscNumber=\"{}\" TrackNumber=\"{}\" Year=\"{}\" AverageBpm=\"{:.2}\" DateAdded=\"{}\" BitRate=\"{}\" SampleRate=\"{}\" Comments=\"{}\" PlayCount=\"{}\" Rating=\"{}\" Location=\"{}\" Remixer=\"{}\" Tonality=\"{}\" Label=\"{}\" Mix=\"{}\"",
            esc(&t.track_id), esc(&t.name), esc(&t.artist), esc(&t.composer), esc(&t.album),
            esc(&t.grouping), esc(&t.genre), esc(&t.kind), t.size, t.total_time, t.disc_number,
            t.track_number, t.year, t.average_bpm, esc(&t.date_added), t.bit_rate, t.sample_rate,
            esc(&t.comments), t.play_count, t.rating, esc(&loc), esc(&t.remixer),
            esc(&t.tonality), esc(&t.label), esc(&t.mix)
        );
        if let Some(c) = &t.colour {
            let _ = write!(o, " Colour=\"{}\"", esc(c));
        }
        if t.tempos.is_empty() && t.marks.is_empty() {
            o.push_str("/>\n");
            continue;
        }
        o.push_str(">\n");
        for g in &t.tempos {
            let _ = writeln!(
                o,
                "      <TEMPO Inizio=\"{:.3}\" Bpm=\"{:.2}\" Metro=\"{}\" Battito=\"{}\"/>",
                g.inizio,
                g.bpm,
                esc(&g.metro),
                g.battito
            );
        }
        for m in &t.marks {
            let _ = write!(
                o,
                "      <POSITION_MARK Name=\"{}\" Type=\"{}\" Start=\"{:.3}\"",
                esc(&m.name),
                m.kind,
                m.start
            );
            if let Some(e) = m.end {
                let _ = write!(o, " End=\"{e:.3}\"");
            }
            let _ = write!(o, " Num=\"{}\"", m.num);
            if let Some((r, g, b)) = m.rgb {
                let _ = write!(o, " Red=\"{r}\" Green=\"{g}\" Blue=\"{b}\"");
            }
            o.push_str("/>\n");
        }
        o.push_str("    </TRACK>\n");
    }
    o.push_str("  </COLLECTION>\n  <PLAYLISTS>\n");
    escrever_node(&mut o, &d.playlists, 2);
    o.push_str("  </PLAYLISTS>\n</DJ_PLAYLISTS>\n");
    o
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOC: &str = r#"<?xml version="1.0" encoding="UTF-8"?>

<DJ_PLAYLISTS Version="1.0.0">
  <PRODUCT Name="rekordbox" Version="7.2.8" Company="AlphaTheta"/>
  <COLLECTION Entries="2">
    <TRACK TrackID="1" Name="Rock &amp; Roll" Artist="X" Kind="Ficheiro MP3" TotalTime="300"
           AverageBpm="125.00" Rating="51" Tonality="Abm"
           Location="file://localhost/C:/M%c3%basicas/Rock%20%26%20Roll/faixa%20#1.mp3">
      <TEMPO Inizio="0.050" Bpm="120.00" Metro="4/4" Battito="1"/>
      <TEMPO Inizio="60.050" Bpm="130.00" Metro="4/4" Battito="3"/>
      <POSITION_MARK Name="" Type="0" Start="36.974" Num="2" Red="125" Green="193" Blue="61"/>
      <POSITION_MARK Name="" Type="4" Start="0.050" End="3.743" Num="-1"/>
      <POSITION_MARK Name="" Type="0" Start="0.050" Num="-1"/>
      <DESCONHECIDO A="1"/>
    </TRACK>
    <TRACK TrackID="2" Location="C:/invalida.mp3"/>
  </COLLECTION>
  <PLAYLISTS>
    <NODE Type="0" Name="ROOT" Count="1">
      <NODE Type="0" Name="Festas" Count="1">
        <NODE Name="Abertura" Type="1" KeyType="0" Entries="2">
          <TRACK Key="1"/>
          <TRACK Key="2"/>
        </NODE>
      </NODE>
    </NODE>
  </PLAYLISTS>
</DJ_PLAYLISTS>
"#;

    #[test]
    fn le_faixa_grade_marcas_e_playlists() {
        let d = parse(DOC).unwrap();
        assert_eq!(
            (d.product.name.as_str(), d.product.version.as_str()),
            ("rekordbox", "7.2.8")
        );
        let t = &d.tracks[0];
        assert_eq!(t.name, "Rock & Roll");
        assert_eq!(
            t.location.as_deref(),
            Some("C:/Músicas/Rock & Roll/faixa #1.mp3")
        );
        assert_eq!(
            (t.average_bpm, t.rating, t.tonality.as_str()),
            (125.0, 51, "Abm")
        );
        assert_eq!(t.tempos.len(), 2);
        assert_eq!((t.tempos[1].inizio, t.tempos[1].battito), (60.05, 3));
        assert_eq!(t.marks[0].rgb, Some((125, 193, 61)));
        assert_eq!(
            (
                t.marks[1].kind,
                t.marks[1].end,
                t.marks[1].num,
                t.marks[1].rgb
            ),
            (4, Some(3.743), -1, None)
        );
        assert_eq!(d.tracks[1].location, None);
        let Node::Folder { children, .. } = &d.playlists else {
            panic!()
        };
        let Node::Folder { name, children } = &children[0] else {
            panic!()
        };
        assert_eq!(name, "Festas");
        assert_eq!(
            children[0],
            Node::Playlist {
                name: "Abertura".into(),
                key_type: 0,
                keys: vec!["1".into(), "2".into()]
            }
        );
    }

    #[test]
    fn ida_e_volta_preserva_o_modelo() {
        let d = parse(DOC).unwrap();
        let escrito = write(&d);
        let d2 = parse(&escrito).unwrap();
        // a faixa com Location inválida volta sem Location
        assert_eq!(d.tracks[0], d2.tracks[0]);
        assert_eq!(d.playlists, d2.playlists);
        assert!(escrito.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\n<DJ_PLAYLISTS"));
        assert!(escrito.contains("Name=\"Rock &amp; Roll\""));
        assert!(escrito.contains(
            "<POSITION_MARK Name=\"\" Type=\"4\" Start=\"0.050\" End=\"3.743\" Num=\"-1\"/>"
        ));
        assert!(escrito.contains("faixa%20#1.mp3"));
    }

    #[test]
    fn documento_minimo_e_erros() {
        let d = parse(r#"<DJ_PLAYLISTS Version="1.0.0"><PRODUCT Name="x" Version="1" Company="y"/><COLLECTION Entries="0"/><PLAYLISTS><NODE Type="0" Name="ROOT" Count="0"/></PLAYLISTS></DJ_PLAYLISTS>"#).unwrap();
        assert!(d.tracks.is_empty());
        assert_eq!(
            d.playlists,
            Node::Folder {
                name: "ROOT".into(),
                children: vec![]
            }
        );
        assert!(matches!(parse("<OUTRA/>"), Err(XmlError::Raiz)));
        assert!(matches!(parse("<DJ_PLAYLISTS>"), Err(XmlError::Sintaxe(_))));
    }
}
