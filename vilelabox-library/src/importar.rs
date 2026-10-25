//! Importadores: adaptadores de formato → banco próprio.
//!
//! Catálogo (`spec/rekordbox/10-master-db.md`) e XML (`spec/rekordbox/12-xml.md`)
//! entram como faixas, cues, grades e playlists. A correspondência é pelo
//! caminho do arquivo: reimportar atualiza a faixa em vez de duplicá-la.
//! Cues entram com origem `Imported`, substituindo só os dessa origem — cues
//! do usuário são preservados.

use crate::ErroLibrary;
use std::collections::HashMap;
use vilelabox_catalog::{Catalogo, Cue as CueCatalogo};
use vilelabox_storage::{Database, NovaFaixa};
use vilelabox_types::{
    BeatGrid, Bpm, Cue, CueKind, CueOrigin, Frame, SampleRate, TempoChange, TrackId, TrackMetadata,
};
use vilelabox_xml::{Document, Mark, Node};

/// Resultado agregado de uma importação; falhas por faixa são coletadas em
/// `erros` sem abortar o lote.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Relatorio {
    pub faixas_novas: usize,
    pub faixas_atualizadas: usize,
    pub cues_importados: usize,
    pub listas_criadas: usize,
    pub erros: Vec<String>,
}

fn vazio(s: &str) -> Option<String> {
    let t = s.trim();
    (!t.is_empty()).then(|| t.to_string())
}

/// Insere ou atualiza a faixa conforme o caminho. Sem caminho, insere sempre
/// (a correspondência por conteúdo é responsabilidade de `identidade`).
fn upsert(
    db: &Database,
    caminho: &str,
    nova: &NovaFaixa,
    rel: &mut Relatorio,
) -> Result<TrackId, ErroLibrary> {
    if !caminho.is_empty() {
        if let Some(id) = db.id_por_localizacao(caminho)? {
            db.atualizar_faixa(id, nova)?;
            rel.faixas_atualizadas += 1;
            return Ok(id);
        }
    }
    let id = db.inserir_faixa(nova)?;
    if !caminho.is_empty() {
        db.adicionar_localizacao(id, caminho)?;
    }
    rel.faixas_novas += 1;
    Ok(id)
}

fn frames_de_ms(ms: i64, taxa: u32) -> Frame {
    Frame::new(((ms.max(0) as u64 * u64::from(taxa)) / 1000) as i64)
}

/// Cue do catálogo: `tipo` 0 = memória, 1..=8 = hot A..H; com saída vira loop.
/// `None` quando a faixa não tem taxa de amostragem.
fn cue_do_catalogo(c: &CueCatalogo, taxa: u32) -> Option<Cue> {
    if taxa == 0 {
        return None;
    }
    let pos = frames_de_ms(c.entrada_ms, taxa);
    let mut cue = match c.saida_ms {
        Some(fim) => Cue::new_loop(pos, frames_de_ms(fim, taxa), CueOrigin::Imported).ok()?,
        None if c.tipo == 0 => Cue::new(CueKind::MemoryCue, pos, CueOrigin::Imported),
        None => Cue::new(CueKind::HotCue, pos, CueOrigin::Imported),
    };
    if (1..=8).contains(&c.tipo) {
        cue = cue.with_slot((c.tipo - 1) as u8);
    }
    Some(cue.with_label(c.rotulo.clone().unwrap_or_default()))
}

/// Importa a leitura do catálogo (`vilelabox_catalog::abrir`).
pub fn importar_catalogo(db: &Database, cat: &Catalogo) -> Result<Relatorio, ErroLibrary> {
    let mut rel = Relatorio::default();
    let mut ids: HashMap<&str, TrackId> = HashMap::new();
    for f in &cat.faixas {
        let nova = NovaFaixa {
            metadados: TrackMetadata {
                title: vazio(&f.titulo),
                artist: vazio(&f.artista),
                album: vazio(&f.album),
                genre: vazio(&f.genero),
                year: None,
                comment: None,
            },
            identidade: None,
            bpm: if f.bpm > 0.0 {
                Bpm::new(f.bpm).ok()
            } else {
                None
            },
            tom: vazio(&f.tom),
            estrelas: f.estrelas.min(5),
        };
        let id = upsert(db, &f.caminho, &nova, &mut rel)?;
        ids.insert(f.id.as_str(), id);
        let mut cues = Vec::new();
        for c in &f.cues {
            match cue_do_catalogo(c, f.taxa_amostragem) {
                Some(cue) => cues.push(cue),
                None => rel.erros.push(format!(
                    "{}: cue ignorado (sem taxa de amostragem)",
                    f.titulo
                )),
            }
        }
        rel.cues_importados += cues.len();
        db.substituir_cues_da_origem(id, CueOrigin::Imported, &cues)?;
    }
    for l in &cat.listas {
        let lista = criar_caminho(db, &l.nome, &mut rel)?;
        for ext in &l.faixas {
            match ids.get(ext.as_str()) {
                Some(id) => db.adicionar_a_lista(lista, *id)?,
                None => rel
                    .erros
                    .push(format!("{}: faixa {ext} da playlist não importada", l.nome)),
            }
        }
    }
    Ok(rel)
}

/// Cadeia de pastas a partir do nome achatado do catálogo ("Pasta / Set").
/// Nomes com " / " no original são divididos — limitação da leitura achatada.
fn criar_caminho(db: &Database, nome: &str, rel: &mut Relatorio) -> Result<i64, ErroLibrary> {
    let partes: Vec<&str> = nome.split(" / ").collect();
    let mut pai = None;
    for p in &partes[..partes.len().saturating_sub(1)] {
        pai = Some(db.criar_lista(p, pai, true)?);
        rel.listas_criadas += 1;
    }
    let id = db.criar_lista(partes[partes.len() - 1], pai, false)?;
    rel.listas_criadas += 1;
    Ok(id)
}

fn frames_de_s(s: f64, taxa: u32) -> Frame {
    Frame::new((s.max(0.0) * f64::from(taxa)).round() as i64)
}

/// Marca do XML (`POSITION_MARK`): `num` -1 = memória; 0.. = hot A..; com
/// `end` vira loop. `None` quando a faixa não tem taxa de amostragem.
fn cue_do_xml(m: &Mark, taxa: u32) -> Option<Cue> {
    if taxa == 0 {
        return None;
    }
    let pos = frames_de_s(m.start, taxa);
    let mut cue = match m.end {
        Some(fim) => Cue::new_loop(pos, frames_de_s(fim, taxa), CueOrigin::Imported).ok()?,
        None if m.num < 0 => Cue::new(CueKind::MemoryCue, pos, CueOrigin::Imported),
        None => Cue::new(CueKind::HotCue, pos, CueOrigin::Imported),
    };
    if m.num >= 0 {
        cue = cue.with_slot(m.num as u8);
    }
    if let Some((r, g, b)) = m.rgb {
        cue = cue.with_color((u32::from(r) << 16) | (u32::from(g) << 8) | u32::from(b));
    }
    Some(cue.with_label(m.name.clone()))
}

/// Importa o documento de intercâmbio (`vilelabox_xml::parse`). A grade vem
/// dos `TEMPO` (o XML não carrega posições de batida); o bloqueio da grade
/// existente é preservado.
pub fn importar_xml(db: &Database, doc: &Document) -> Result<Relatorio, ErroLibrary> {
    let mut rel = Relatorio::default();
    let mut por_trackid: HashMap<&str, TrackId> = HashMap::new();
    let mut por_location: HashMap<String, TrackId> = HashMap::new();
    for t in &doc.tracks {
        // Rating 0..255 em degraus de 51 (12-xml, tabela Rating).
        let estrelas = ((u32::from(t.rating) + 25) / 51).min(5) as u8;
        let nova = NovaFaixa {
            metadados: TrackMetadata {
                title: vazio(&t.name),
                artist: vazio(&t.artist),
                album: vazio(&t.album),
                genre: vazio(&t.genre),
                year: (t.year > 0).then_some(t.year),
                comment: vazio(&t.comments),
            },
            identidade: None,
            bpm: if t.average_bpm > 0.0 {
                Bpm::new(t.average_bpm).ok()
            } else {
                None
            },
            tom: vazio(&t.tonality),
            estrelas,
        };
        let caminho = t.location.clone().unwrap_or_default();
        let id = upsert(db, &caminho, &nova, &mut rel)?;
        if !t.track_id.is_empty() {
            por_trackid.insert(t.track_id.as_str(), id);
        }
        if !caminho.is_empty() {
            por_location.insert(caminho, id);
        }
        let mut cues = Vec::new();
        for m in &t.marks {
            match cue_do_xml(m, t.sample_rate) {
                Some(cue) => cues.push(cue),
                None => rel.erros.push(format!(
                    "{}: marca ignorada (sem taxa de amostragem)",
                    t.name
                )),
            }
        }
        rel.cues_importados += cues.len();
        db.substituir_cues_da_origem(id, CueOrigin::Imported, &cues)?;
        if !t.tempos.is_empty() && t.sample_rate > 0 {
            let mut tempo = Vec::new();
            for tp in &t.tempos {
                if let Ok(bpm) = Bpm::new(tp.bpm) {
                    tempo.push(TempoChange {
                        position: frames_de_s(tp.inizio, t.sample_rate),
                        bpm,
                    });
                }
            }
            let bloqueada = db.grade(id)?.map(|g| g.bloqueada).unwrap_or(false);
            if !tempo.is_empty() {
                let grid = BeatGrid::new(SampleRate::new(t.sample_rate)?, Vec::new(), tempo, None)?;
                db.salvar_grade(id, &grid, bloqueada)?;
            }
        }
    }
    importar_no(
        db,
        &doc.playlists,
        None,
        &por_trackid,
        &por_location,
        &mut rel,
    )?;
    Ok(rel)
}

fn importar_no(
    db: &Database,
    no: &Node,
    pai: Option<i64>,
    por_trackid: &HashMap<&str, TrackId>,
    por_location: &HashMap<String, TrackId>,
    rel: &mut Relatorio,
) -> Result<(), ErroLibrary> {
    match no {
        Node::Folder { name, children } => {
            // A raiz ("ROOT") não vira pasta.
            let pai = if name == "ROOT" {
                pai
            } else {
                rel.listas_criadas += 1;
                Some(db.criar_lista(name, pai, true)?)
            };
            for c in children {
                importar_no(db, c, pai, por_trackid, por_location, rel)?;
            }
        }
        Node::Playlist {
            name,
            key_type,
            keys,
        } => {
            let lista = db.criar_lista(name, pai, false)?;
            rel.listas_criadas += 1;
            for k in keys {
                let achada = if *key_type == 1 {
                    vilelabox_xml::location::decode(k).and_then(|c| por_location.get(&c))
                } else {
                    por_trackid.get(k.as_str())
                };
                match achada {
                    Some(id) => db.adicionar_a_lista(lista, *id)?,
                    None => rel
                        .erros
                        .push(format!("{name}: faixa da playlist não encontrada ({k})")),
                }
            }
        }
    }
    Ok(())
}
