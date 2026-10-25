//! Testes dos importadores: catálogo e XML → banco próprio.

use vilelabox_catalog::{Catalogo, Cue as CueCatalogo, Faixa, Lista};
use vilelabox_library::{importar_catalogo, importar_xml};
use vilelabox_storage::Database;
use vilelabox_types::{Cue, CueKind, CueOrigin, Frame};
use vilelabox_xml::{Document, Mark, Node, Tempo, Track};

fn catalogo() -> Catalogo {
    Catalogo {
        faixas: vec![Faixa {
            id: "1".into(),
            titulo: "Faixa Um".into(),
            artista: "DJ A".into(),
            album: "Coletânea".into(),
            genero: "House".into(),
            tom: "Gm".into(),
            estrelas: 4,
            bpm: 128.0,
            duracao_s: 300,
            taxa_amostragem: 44100,
            caminho: "C:/musicas/a.mp3".into(),
            analise: None,
            cues: vec![
                CueCatalogo {
                    tipo: 0,
                    entrada_ms: 25,
                    saida_ms: None,
                    rotulo: None,
                },
                CueCatalogo {
                    tipo: 2,
                    entrada_ms: 1000,
                    saida_ms: Some(2000),
                    rotulo: Some("loop b".into()),
                },
            ],
        }],
        listas: vec![Lista {
            nome: "Pasta / Set".into(),
            faixas: vec!["1".into()],
        }],
    }
}

#[test]
fn catalogo_importa_faixas_cues_e_playlists() {
    let db = Database::open_in_memory().unwrap();
    let rel = importar_catalogo(&db, &catalogo()).unwrap();
    assert_eq!((rel.faixas_novas, rel.listas_criadas), (1, 2));
    assert!(rel.erros.is_empty());

    let faixas = db.listar_faixas(None).unwrap();
    assert_eq!(faixas.len(), 1);
    let f = &faixas[0];
    assert_eq!(f.metadados.artist.as_deref(), Some("DJ A"));
    assert_eq!(f.bpm.unwrap().get(), 128.0);
    assert_eq!(f.estrelas, 4);
    assert!(f.identidade.is_none(), "catálogo não carrega hash do áudio");

    // Cues: memória em 25 ms e loop quente (slot 1) de 1 s a 2 s em 44,1 kHz.
    let cues = db.cues(f.id).unwrap();
    assert_eq!(cues.len(), 2);
    assert_eq!(cues[0].kind, CueKind::MemoryCue);
    assert_eq!(cues[0].position, Frame::new(1102));
    assert_eq!(cues[1].kind, CueKind::Loop);
    assert_eq!(cues[1].slot, Some(1));
    assert_eq!(cues[1].end_position, Some(Frame::new(88200)));
    assert_eq!(cues[1].origin, CueOrigin::Imported);
    assert_eq!(cues[1].label, "loop b");

    // Árvore: pasta "Pasta" com a playlist "Set" contendo a faixa.
    let arvore = db.listas().unwrap();
    assert_eq!(arvore.len(), 2);
    assert!(arvore[0].pasta);
    let set = arvore.iter().find(|l| !l.pasta).unwrap();
    assert_eq!(set.pai, Some(arvore[0].id));
    assert_eq!(db.faixas_da_lista(set.id).unwrap(), [f.id]);
}

#[test]
fn reimportar_catalogo_atualiza_sem_duplicar_e_preserva_cues_do_usuario() {
    let db = Database::open_in_memory().unwrap();
    importar_catalogo(&db, &catalogo()).unwrap();
    let id = db.listar_faixas(None).unwrap()[0].id;
    // Cue do usuário convive com os importados.
    db.substituir_cues(
        id,
        &[Cue::new(CueKind::HotCue, Frame::new(44100), CueOrigin::User).with_slot(0)],
    )
    .unwrap();

    let rel = importar_catalogo(&db, &catalogo()).unwrap();
    assert_eq!((rel.faixas_novas, rel.faixas_atualizadas), (0, 1));
    assert_eq!(db.listar_faixas(None).unwrap().len(), 1);

    let cues = db.cues(id).unwrap();
    let usuario = cues.iter().filter(|c| c.origin == CueOrigin::User).count();
    let importados = cues
        .iter()
        .filter(|c| c.origin == CueOrigin::Imported)
        .count();
    assert_eq!((usuario, importados), (1, 2));
}

fn documento() -> Document {
    Document {
        tracks: vec![Track {
            track_id: "10".into(),
            name: "Faixa XML".into(),
            artist: "DJ X".into(),
            year: 2024,
            average_bpm: 128.0,
            sample_rate: 44100,
            rating: 204,
            tonality: "Am".into(),
            location: Some("C:/musicas/x.mp3".into()),
            marks: vec![
                Mark {
                    name: "mem".into(),
                    kind: 0,
                    start: 1.0,
                    end: None,
                    num: -1,
                    rgb: None,
                },
                Mark {
                    name: "hot".into(),
                    kind: 0,
                    start: 2.0,
                    end: None,
                    num: 2,
                    rgb: Some((255, 0, 0)),
                },
                Mark {
                    name: "loop".into(),
                    kind: 0,
                    start: 4.0,
                    end: Some(8.0),
                    num: 0,
                    rgb: None,
                },
            ],
            tempos: vec![Tempo {
                inizio: 0.0,
                bpm: 128.0,
                metro: "4/4".into(),
                battito: 1,
            }],
            ..Track::default()
        }],
        playlists: Node::Folder {
            name: "ROOT".into(),
            children: vec![
                Node::Folder {
                    name: "Pasta".into(),
                    children: vec![Node::Playlist {
                        name: "Set".into(),
                        key_type: 0,
                        keys: vec!["10".into()],
                    }],
                },
                Node::Playlist {
                    name: "PorLocal".into(),
                    key_type: 1,
                    keys: vec![vilelabox_xml::location::encode("C:/musicas/x.mp3")],
                },
            ],
        },
        ..Document::default()
    }
}

#[test]
fn xml_importa_faixas_marcas_grade_e_arvore() {
    let db = Database::open_in_memory().unwrap();
    let rel = importar_xml(&db, &documento()).unwrap();
    assert_eq!((rel.faixas_novas, rel.listas_criadas), (1, 3));
    assert!(rel.erros.is_empty(), "erros: {:?}", rel.erros);

    let f = &db.listar_faixas(None).unwrap()[0];
    assert_eq!(f.metadados.title.as_deref(), Some("Faixa XML"));
    assert_eq!(f.metadados.year, Some(2024));
    // Rating 204 → 4 estrelas (tabela 0/51/102/153/204/255).
    assert_eq!(f.estrelas, 4);
    assert_eq!(f.tom.as_deref(), Some("Am"));

    let cues = db.cues(f.id).unwrap();
    assert_eq!(cues.len(), 3);
    assert_eq!(cues[0].kind, CueKind::MemoryCue);
    assert_eq!(cues[0].position, Frame::new(44100));
    assert_eq!(cues[1].kind, CueKind::HotCue);
    assert_eq!(cues[1].slot, Some(2));
    assert_eq!(cues[1].color, Some(0xFF0000));
    assert_eq!(cues[2].kind, CueKind::Loop);
    assert_eq!(cues[2].slot, Some(0));
    assert_eq!(cues[2].end_position, Some(Frame::new(352800)));

    // Grade só com o mapa de tempo (o XML não carrega batidas).
    let g = db.grade(f.id).unwrap().unwrap();
    assert!(!g.bloqueada);
    assert!(g.grade.beats().is_empty());
    assert_eq!(g.grade.tempo_map().len(), 1);
    assert_eq!(g.grade.tempo_map()[0].bpm.get(), 128.0);

    // Playlist por TrackID (em "Pasta") e por Location codificada (na raiz).
    let arvore = db.listas().unwrap();
    let set = arvore.iter().find(|l| l.nome == "Set").unwrap();
    let pasta = arvore.iter().find(|l| l.nome == "Pasta").unwrap();
    assert_eq!(set.pai, Some(pasta.id));
    assert_eq!(db.faixas_da_lista(set.id).unwrap(), [f.id]);
    let por_local = arvore.iter().find(|l| l.nome == "PorLocal").unwrap();
    assert_eq!(por_local.pai, None);
    assert_eq!(db.faixas_da_lista(por_local.id).unwrap(), [f.id]);
}

#[test]
fn reimportar_xml_e_idempotente() {
    let db = Database::open_in_memory().unwrap();
    importar_xml(&db, &documento()).unwrap();
    let rel = importar_xml(&db, &documento()).unwrap();
    assert_eq!((rel.faixas_novas, rel.faixas_atualizadas), (0, 1));
    assert_eq!(db.listar_faixas(None).unwrap().len(), 1);
    let id = db.listar_faixas(None).unwrap()[0].id;
    assert_eq!(db.cues(id).unwrap().len(), 3);
    // A reimportação não apaga nem duplica a grade.
    assert_eq!(db.grade(id).unwrap().unwrap().grade.tempo_map().len(), 1);
}

#[test]
fn xml_sem_taxa_de_amostragem_conta_erro_e_importa_o_restolho() {
    let db = Database::open_in_memory().unwrap();
    let mut doc = documento();
    doc.tracks[0].sample_rate = 0;
    let rel = importar_xml(&db, &doc).unwrap();
    assert_eq!(rel.faixas_novas, 1);
    assert_eq!(rel.erros.len(), 3, "uma marca ignorada por linha");
    let f = &db.listar_faixas(None).unwrap()[0];
    assert!(db.cues(f.id).unwrap().is_empty());
    assert!(db.grade(f.id).unwrap().is_none());
}
