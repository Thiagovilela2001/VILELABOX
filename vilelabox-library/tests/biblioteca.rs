//! Testes de varredura, busca e identidade de arquivos.

use std::path::PathBuf;
use vilelabox_library::{
    buscar, buscar_movidos, duplicatas, nome_normalizado, sha256_arquivo, varrer_diretorio,
    verificar_localizacoes, Coluna, Consulta,
};
use vilelabox_storage::{Database, NovaFaixa};
use vilelabox_types::{AudioIdentity, Bpm, Frame, SampleRate, TrackMetadata};

fn pasta(nome: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vilelabox-library-{nome}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn faixa(titulo: &str, artista: &str, bpm: f64, estrelas: u8) -> NovaFaixa {
    NovaFaixa {
        metadados: TrackMetadata {
            title: Some(titulo.into()),
            artist: Some(artista.into()),
            ..TrackMetadata::default()
        },
        bpm: Some(Bpm::new(bpm).unwrap()),
        estrelas,
        ..NovaFaixa::default()
    }
}

#[test]
fn varredura_registra_audios_e_ignora_o_resto() {
    let dir = pasta("scan");
    std::fs::write(dir.join("DJ A - Faixa B.mp3"), b"x").unwrap();
    std::fs::write(dir.join("semartista.wav"), b"y").unwrap();
    std::fs::write(dir.join("capa.jpg"), b"z").unwrap();
    std::fs::create_dir(dir.join("sub")).unwrap();
    std::fs::write(dir.join("sub").join("aninhado.flac"), b"w").unwrap();

    let db = Database::open_in_memory().unwrap();
    let rel = varrer_diretorio(&db, &dir).unwrap();
    assert_eq!((rel.novos, rel.ignorados), (3, 1));

    let faixas = db.listar_faixas(None).unwrap();
    assert_eq!(faixas.len(), 3);
    let fb = faixas
        .iter()
        .find(|f| f.metadados.title.as_deref() == Some("Faixa B"))
        .unwrap();
    assert_eq!(fb.metadados.artist.as_deref(), Some("DJ A"));
    let sa = faixas
        .iter()
        .find(|f| f.metadados.title.as_deref() == Some("semartista"))
        .unwrap();
    assert_eq!(sa.metadados.artist, None);
    let an = faixas
        .iter()
        .find(|f| f.metadados.title.as_deref() == Some("aninhado"))
        .unwrap();
    let locs = db.localizacoes(an.id).unwrap();
    assert_eq!(locs.len(), 1);
    assert!(locs[0].caminho.ends_with("sub/aninhado.flac"));

    // Segunda varredura: tudo já conhecido.
    let rel = varrer_diretorio(&db, &dir).unwrap();
    assert_eq!((rel.novos, rel.presentes), (0, 3));
    assert_eq!(db.listar_faixas(None).unwrap().len(), 3);
}

#[test]
fn busca_texto_ordena_e_filtra_por_playlist() {
    let db = Database::open_in_memory().unwrap();
    let a = db
        .inserir_faixa(&faixa("Alpha", "DJ Um", 120.0, 5))
        .unwrap();
    let b = db
        .inserir_faixa(&faixa("beta", "DJ Dois", 140.0, 3))
        .unwrap();
    let c = db
        .inserir_faixa(&faixa("Gamma", "dj tres", 100.0, 4))
        .unwrap();

    // Texto caixa-insensível em título e artista.
    let q = Consulta {
        texto: "ALPHA".into(),
        ..Consulta::default()
    };
    assert_eq!(buscar(&db, &q).unwrap().len(), 1);
    let q = Consulta {
        texto: "dj ".into(),
        ..Consulta::default()
    };
    assert_eq!(buscar(&db, &q).unwrap().len(), 3);

    // Ordenação por BPM desc e por título asc com desempate estável.
    let q = Consulta {
        coluna: Some(Coluna::Bpm),
        ascendente: false,
        ..Consulta::default()
    };
    let ids: Vec<_> = buscar(&db, &q).unwrap().into_iter().map(|f| f.id).collect();
    assert_eq!(ids, [b, a, c]);
    let q = Consulta {
        coluna: Some(Coluna::Titulo),
        ascendente: true,
        ..Consulta::default()
    };
    let ids: Vec<_> = buscar(&db, &q).unwrap().into_iter().map(|f| f.id).collect();
    assert_eq!(ids, [a, b, c]);

    // Universo restrito à playlist.
    let lista = db.criar_lista("Set", None, false).unwrap();
    db.adicionar_a_lista(lista, c).unwrap();
    db.adicionar_a_lista(lista, a).unwrap();
    let q = Consulta {
        lista: Some(lista),
        ..Consulta::default()
    };
    let ids: Vec<_> = buscar(&db, &q).unwrap().into_iter().map(|f| f.id).collect();
    assert_eq!(ids, [c, a], "ordem da playlist preservada sem coluna");
}

#[test]
fn sha256_de_arquivo_confere_com_vetor_conhecido() {
    let dir = pasta("hash");
    let p = dir.join("abc.bin");
    std::fs::write(&p, b"abc").unwrap();
    assert_eq!(
        sha256_arquivo(&p).unwrap(),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert!(sha256_arquivo(&dir.join("nao-existe.bin")).is_err());
}

#[test]
fn duplicatas_agrupa_pelo_mesmo_hash() {
    let db = Database::open_in_memory().unwrap();
    let ident = |sha: &str| AudioIdentity {
        sha256: Some(sha.into()),
        duration_frames: Frame::new(1000),
        sample_rate: SampleRate::CD,
        channels: 2,
    };
    let mut f = faixa("A", "x", 120.0, 0);
    f.identidade = Some(ident("aa"));
    let a = db.inserir_faixa(&f).unwrap();
    let b = db.inserir_faixa(&f).unwrap();
    f.identidade = Some(ident("bb"));
    db.inserir_faixa(&f).unwrap();

    let grupos = duplicatas(&db).unwrap();
    assert_eq!(grupos.len(), 1);
    assert_eq!(grupos[0].0, "aa");
    assert_eq!(grupos[0].1, [a, b]);
}

#[test]
fn verifica_ausentes_e_encontra_movidos() {
    let dir = pasta("movidos");
    let p = dir.join("01 - DJ A - Faixa B.mp3");
    std::fs::write(&p, b"conteudo").unwrap();

    let db = Database::open_in_memory().unwrap();
    let rel = varrer_diretorio(&db, &dir).unwrap();
    assert_eq!(rel.novos, 1);
    let id = db.listar_faixas(None).unwrap()[0].id;

    // Tudo presente.
    assert_eq!(verificar_localizacoes(&db).unwrap(), (1, 0));

    // "Move" o arquivo para uma subpasta: a antiga fica ausente.
    std::fs::create_dir(dir.join("nova_pasta")).unwrap();
    let novo = dir.join("nova_pasta").join("01 - DJ A - Faixa B.mp3");
    std::fs::rename(&p, &novo).unwrap();
    assert_eq!(verificar_localizacoes(&db).unwrap(), (0, 1));

    let achados = buscar_movidos(&db, &dir).unwrap();
    assert_eq!(achados.len(), 1);
    assert_eq!(achados[0].0, id);
    assert!(achados[0].1.ends_with("nova_pasta/01 - DJ A - Faixa B.mp3"));
    // A faixa fica com as duas localizações: antiga ausente, nova presente.
    let locs = db.localizacoes(id).unwrap();
    assert_eq!(locs.len(), 2);
    assert!(locs[0].ausente);
    assert!(!locs[1].ausente);
}

#[test]
fn nome_normalizado_ignora_numeracao_e_extensao() {
    let a = nome_normalizado("01 - FAIXA PRETA ((FUNKHALL)).mp3");
    assert_eq!(a, nome_normalizado("FAIXA PRETA ((FUNKHALL)).wav"));
    assert_eq!(
        nome_normalizado("01. Brutalismus 3000 - Horime.flac").as_deref(),
        Some("brutalismus 3000 - horime")
    );
    assert_eq!(nome_normalizado("capa.jpg"), None);
    assert_eq!(nome_normalizado("123.mp3"), None);
}
