//! Testes de integridade referencial, cascata e ida-e-volta dos registros do
//! banco próprio (P3.1).

use vilelabox_storage::{Database, ErroStorage, NovaFaixa};
use vilelabox_types::{
    AudioIdentity, Beat, BeatGrid, Bpm, Cue, CueKind, CueOrigin, Frame, Phrase, PhraseKind,
    SampleRate, TempoChange, TrackId, TrackMetadata,
};

fn faixa(nome: &str) -> NovaFaixa {
    NovaFaixa {
        metadados: TrackMetadata {
            title: Some(nome.into()),
            artist: Some("Artista".into()),
            ..TrackMetadata::default()
        },
        identidade: Some(AudioIdentity {
            sha256: Some("ab12".into()),
            duration_frames: Frame::new(44100 * 300),
            sample_rate: SampleRate::CD,
            channels: 2,
        }),
        bpm: Some(Bpm::new(128.0).unwrap()),
        tom: Some("Gm".into()),
        estrelas: 4,
    }
}

fn grade() -> BeatGrid {
    BeatGrid::new(
        SampleRate::CD,
        vec![
            Beat {
                position: Frame::new(0),
                beat_num: 1,
            },
            Beat {
                position: Frame::new(22050),
                beat_num: 2,
            },
        ],
        vec![
            TempoChange {
                position: Frame::new(0),
                bpm: Bpm::new(128.0).unwrap(),
            },
            TempoChange {
                position: Frame::new(22050),
                bpm: Bpm::new(130.0).unwrap(),
            },
        ],
        Some(Frame::new(0)),
    )
    .unwrap()
}

#[test]
fn ciclo_completo_de_faixa() {
    let db = Database::open_in_memory().unwrap();
    let id = db.inserir_faixa(&faixa("Faixa A")).unwrap();

    // Faixa e identidade.
    let f = db.faixa(id).unwrap().unwrap();
    assert_eq!(f.metadados.artist.as_deref(), Some("Artista"));
    let ident = f.identidade.unwrap();
    assert_eq!(ident.sample_rate, SampleRate::CD);
    assert_eq!(ident.duration_frames, Frame::new(44100 * 300));
    assert_eq!(f.estrelas, 4);

    // Localizações.
    db.adicionar_localizacao(id, "C:/musicas/a.mp3").unwrap();
    db.adicionar_localizacao(id, "D:/backup/a.mp3").unwrap();
    let locs = db.localizacoes(id).unwrap();
    assert_eq!(locs.len(), 2);
    assert!(!locs[0].ausente);

    // Cues de todos os tipos.
    let cues = vec![
        Cue::new(CueKind::MemoryCue, Frame::new(100), CueOrigin::Imported),
        Cue::new(CueKind::HotCue, Frame::new(200), CueOrigin::User)
            .with_slot(1)
            .with_label("drop")
            .with_color(0xFF0000),
        Cue::new_loop(Frame::new(300), Frame::new(600), CueOrigin::User).unwrap(),
    ];
    db.substituir_cues(id, &cues).unwrap();
    let lidos = db.cues(id).unwrap();
    assert_eq!(lidos.len(), 3);
    assert_eq!(lidos[1].kind, CueKind::HotCue);
    assert_eq!(lidos[1].slot, Some(1));
    assert_eq!(lidos[1].label, "drop");
    assert_eq!(lidos[1].color, Some(0xFF0000));
    assert!(lidos[0].id.is_some());
    assert_eq!(lidos[2].end_position, Some(Frame::new(600)));

    // Grade com mapa de tempo e bloqueio.
    db.salvar_grade(id, &grade(), true).unwrap();
    let g = db.grade(id).unwrap().unwrap();
    assert!(g.bloqueada);
    assert_eq!(g.grade.beats().len(), 2);
    assert_eq!(g.grade.tempo_map().len(), 2);
    assert_eq!(g.grade.downbeat(), Some(Frame::new(0)));
    assert_eq!(g.grade.bpm_at(Frame::new(30000)).unwrap().get(), 130.0);
    // Regravar substitui sem duplicar.
    db.salvar_grade(id, &grade(), false).unwrap();
    let g = db.grade(id).unwrap().unwrap();
    assert!(!g.bloqueada);
    assert_eq!(g.grade.beats().len(), 2);

    // Frases.
    let frases = vec![
        Phrase::new(
            1,
            Frame::new(0),
            Frame::new(44100 * 8),
            PhraseKind::Intro,
            32,
        )
        .unwrap(),
        Phrase::new(
            2,
            Frame::new(44100 * 8),
            Frame::new(44100 * 16),
            PhraseKind::Chorus,
            32,
        )
        .unwrap(),
    ];
    db.substituir_frases(id, &frases).unwrap();
    assert_eq!(db.frases(id).unwrap(), frases);

    // Playlist com ordem estável.
    let outra = db.inserir_faixa(&faixa("Faixa B")).unwrap();
    let pasta = db.criar_lista("Pasta", None, true).unwrap();
    let lista = db.criar_lista("Set", Some(pasta), false).unwrap();
    db.adicionar_a_lista(lista, id).unwrap();
    db.adicionar_a_lista(lista, outra).unwrap();
    assert_eq!(db.faixas_da_lista(lista).unwrap(), [id, outra]);
    let arvore = db.listas().unwrap();
    assert_eq!(arvore.len(), 2);
    assert!(arvore[0].pasta);
    assert_eq!(arvore[1].pai, Some(pasta));

    // Cache de análise: versões coexistem, upsert substitui.
    db.salvar_analise(id, "bpm", 1, "", &[1, 2, 3]).unwrap();
    db.salvar_analise(id, "bpm", 2, "", &[4]).unwrap();
    db.salvar_analise(id, "bpm", 1, "fino", &[5, 6]).unwrap();
    db.salvar_analise(id, "bpm", 1, "", &[9]).unwrap();
    assert_eq!(db.analise(id, "bpm", 1, "").unwrap().unwrap(), [9]);
    assert_eq!(db.analise(id, "bpm", 2, "").unwrap().unwrap(), [4]);
    assert_eq!(db.analise(id, "bpm", 1, "fino").unwrap().unwrap(), [5, 6]);
    assert!(db.analise(id, "key", 1, "").unwrap().is_none());
}

#[test]
fn cue_sem_faixa_da_erro_de_referencia() {
    let db = Database::open_in_memory().unwrap();
    let cues = vec![Cue::new(CueKind::MemoryCue, Frame::new(1), CueOrigin::User)];
    match db.substituir_cues(TrackId(999), &cues) {
        Err(ErroStorage::Validacao(msg)) => assert!(msg.contains("inexistente")),
        outro => panic!("esperava Validacao, veio {outro:?}"),
    }
}

#[test]
fn remover_faixa_apaga_filhos_em_cascata() {
    let db = Database::open_in_memory().unwrap();
    let id = db.inserir_faixa(&faixa("Faixa A")).unwrap();
    db.adicionar_localizacao(id, "C:/musicas/a.mp3").unwrap();
    db.substituir_cues(
        id,
        &[Cue::new(CueKind::HotCue, Frame::new(1), CueOrigin::User)],
    )
    .unwrap();
    db.salvar_grade(id, &grade(), false).unwrap();
    db.substituir_frases(
        id,
        &[Phrase::new(1, Frame::new(0), Frame::new(100), PhraseKind::Intro, 4).unwrap()],
    )
    .unwrap();
    db.salvar_analise(id, "bpm", 1, "", &[1]).unwrap();
    let lista = db.criar_lista("Set", None, false).unwrap();
    db.adicionar_a_lista(lista, id).unwrap();

    assert!(db.remover_faixa(id).unwrap());
    assert!(!db.remover_faixa(id).unwrap());
    let tabelas = [
        "cues",
        "beatgrids",
        "grid_beats",
        "grid_tempo",
        "phrases",
        "track_locations",
        "playlist_tracks",
        "analysis_results",
    ];
    for tabela in tabelas {
        let n: i64 = db
            .con()
            .query_row(&format!("SELECT count(*) FROM {tabela}"), [], |r| r.get(0))
            .unwrap();
        assert_eq!(n, 0, "restou linha em {tabela}");
    }
    assert!(db.faixa(id).unwrap().is_none());
}

#[test]
fn localizacao_duplicada_da_erro() {
    let db = Database::open_in_memory().unwrap();
    let id = db.inserir_faixa(&faixa("Faixa A")).unwrap();
    db.adicionar_localizacao(id, "C:/musicas/a.mp3").unwrap();
    assert!(matches!(
        db.adicionar_localizacao(id, "C:/musicas/a.mp3"),
        Err(ErroStorage::Validacao(_))
    ));
}

#[test]
fn estrelas_acima_de_cinco_da_erro() {
    let db = Database::open_in_memory().unwrap();
    let mut f = faixa("Faixa A");
    f.estrelas = 6;
    assert!(matches!(
        db.inserir_faixa(&f),
        Err(ErroStorage::Validacao(_))
    ));
}

#[test]
fn grade_corrompida_da_erro_tipado() {
    let db = Database::open_in_memory().unwrap();
    let id = db.inserir_faixa(&faixa("Faixa A")).unwrap();
    db.salvar_grade(id, &grade(), false).unwrap();
    // Corrompe as posições diretamente: idx 1 passa a vir antes do idx 0.
    db.con()
        .execute(
            "UPDATE grid_beats SET position_frames = 50 WHERE track_id = ?1 AND idx = 0",
            [id.0],
        )
        .unwrap();
    db.con()
        .execute(
            "UPDATE grid_beats SET position_frames = 10 WHERE track_id = ?1 AND idx = 1",
            [id.0],
        )
        .unwrap();
    assert!(matches!(db.grade(id), Err(ErroStorage::Tipo(_))));
}

#[test]
fn adicionar_a_lista_inexistente_da_erro() {
    let db = Database::open_in_memory().unwrap();
    let id = db.inserir_faixa(&faixa("Faixa A")).unwrap();
    assert!(matches!(
        db.adicionar_a_lista(999, id),
        Err(ErroStorage::Validacao(_))
    ));
}
