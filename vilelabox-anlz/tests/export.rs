use std::fs;
use std::path::Path;
use tempfile::tempdir;
use vilelabox_anlz::{compute_file_sha256, export_anlz, AnlzError, ExportIndex, ExportOptions};
use vilelabox_reference::ReferenceTrack;

fn build_section(tag: &[u8; 4], header_len: u32, payload: &[u8]) -> Vec<u8> {
    let total_len = header_len + payload.len() as u32;
    let mut out = Vec::with_capacity(total_len as usize);
    out.extend_from_slice(tag);
    out.extend_from_slice(&header_len.to_be_bytes());
    out.extend_from_slice(&total_len.to_be_bytes());
    out.resize(header_len as usize, 0);
    out.extend_from_slice(payload);
    out
}

fn build_pmai(sections: &[Vec<u8>]) -> Vec<u8> {
    let body_len: usize = sections.iter().map(Vec::len).sum();
    let total = 16 + body_len as u32;
    let mut out = Vec::with_capacity(total as usize);
    out.extend_from_slice(b"PMAI");
    out.extend_from_slice(&16u32.to_be_bytes());
    out.extend_from_slice(&total.to_be_bytes());
    out.extend_from_slice(&0u32.to_be_bytes());
    for sec in sections {
        out.extend_from_slice(sec);
    }
    out
}

fn build_ppth(path: &str) -> Vec<u8> {
    let mut utf16 = Vec::new();
    for unit in path.encode_utf16() {
        utf16.extend_from_slice(&unit.to_be_bytes());
    }
    utf16.extend_from_slice(&[0, 0]);
    let mut payload = (utf16.len() as u32).to_be_bytes().to_vec();
    payload.extend_from_slice(&utf16);
    build_section(b"PPTH", 12, &payload)
}

fn build_pqtz(beats: &[(u16, u16, u32)]) -> Vec<u8> {
    let mut payload = Vec::new();
    payload.extend_from_slice(&0u32.to_be_bytes());
    payload.extend_from_slice(&0x80000u32.to_be_bytes());
    payload.extend_from_slice(&(beats.len() as u32).to_be_bytes());
    for &(beat_num, tempo_x100, time_ms) in beats {
        payload.extend_from_slice(&beat_num.to_be_bytes());
        payload.extend_from_slice(&tempo_x100.to_be_bytes());
        payload.extend_from_slice(&time_ms.to_be_bytes());
    }
    build_section(b"PQTZ", 12, &payload)
}

fn build_pssi(entries: &[(u16, u16, u16)]) -> Vec<u8> {
    let len_entry = 24u32;
    let len_entries = entries.len() as u16;
    let mut payload = Vec::new();
    payload.extend_from_slice(&len_entry.to_be_bytes());
    payload.extend_from_slice(&len_entries.to_be_bytes());
    // Body sem máscara: mood = 1 (high), 6 bytes pad, end_beat = 32, 2 bytes pad, bank = 1, 1 pad
    payload.extend_from_slice(&1u16.to_be_bytes());
    payload.extend_from_slice(&[0u8; 6]);
    payload.extend_from_slice(&32u16.to_be_bytes());
    payload.extend_from_slice(&[0u8; 2]);
    payload.push(1);
    payload.push(0);

    for &(idx, beat, kind) in entries {
        let mut entry = vec![0u8; len_entry as usize];
        entry[0..2].copy_from_slice(&idx.to_be_bytes());
        entry[2..4].copy_from_slice(&beat.to_be_bytes());
        entry[4..6].copy_from_slice(&kind.to_be_bytes());
        payload.extend_from_slice(&entry);
    }
    build_section(b"PSSI", 12, &payload)
}

fn write_sample_anlz_pair(dir: &Path, ppth: &str, phrases: &[(u16, u16, u16)]) {
    fs::create_dir_all(dir).unwrap();
    let beats: Vec<(u16, u16, u32)> = (0..16)
        .map(|i| (((i % 4) + 1) as u16, 12800, i * 468))
        .collect();
    let dat = build_pmai(&[build_ppth(ppth), build_pqtz(&beats)]);
    let ext = build_pmai(&[build_ppth(ppth), build_pssi(phrases)]);
    fs::write(dir.join("ANLZ0000.DAT"), dat).unwrap();
    fs::write(dir.join("ANLZ0000.EXT"), ext).unwrap();
}

#[test]
fn exporta_deterministico_e_calcula_sha256_por_streaming() {
    let tmp = tempdir().unwrap();
    let anlz_dir = tmp.path().join("anlz");
    let music_dir = tmp.path().join("music");
    let out1 = tmp.path().join("out1");
    let out2 = tmp.path().join("out2");

    fs::create_dir_all(&music_dir).unwrap();
    let audio_a = music_dir.join("faixa_a.mp3");
    let audio_b = music_dir.join("faixa_b.flac");
    fs::write(&audio_a, vec![0x41u8; 150_000]).unwrap();
    fs::write(&audio_b, vec![0x42u8; 90_000]).unwrap();

    write_sample_anlz_pair(
        &anlz_dir.join("001/sub_a"),
        "C:/Colecao/faixa_a.mp3",
        &[(1, 1, 1), (2, 5, 2)],
    );
    write_sample_anlz_pair(
        &anlz_dir.join("002/sub_b"),
        "/Users/dj/faixa_b.flac",
        &[(1, 1, 1)],
    );

    let idx1 = export_anlz(&ExportOptions {
        anlz_dir: anlz_dir.clone(),
        music_dir: music_dir.clone(),
        output_dir: out1.clone(),
        limit: None,
    })
    .unwrap();

    let idx2 = export_anlz(&ExportOptions {
        anlz_dir,
        music_dir,
        output_dir: out2.clone(),
        limit: None,
    })
    .unwrap();

    assert_eq!(idx1, idx2);
    assert_eq!(idx1.count, 2);
    assert_eq!(idx1.with_phrases, 2);
    assert_eq!(idx1.missing_audio, 0);

    let index_bytes_1 = fs::read(out1.join("index.json")).unwrap();
    let index_bytes_2 = fs::read(out2.join("index.json")).unwrap();
    assert_eq!(index_bytes_1, index_bytes_2);

    let sha_a = compute_file_sha256(&audio_a).unwrap();
    let track_bytes_1 = fs::read(out1.join(format!("{sha_a}.json"))).unwrap();
    let track_bytes_2 = fs::read(out2.join(format!("{sha_a}.json"))).unwrap();
    assert_eq!(track_bytes_1, track_bytes_2);

    // Republicar no mesmo destino também precisa ser determinístico e não
    // pode depender de o diretório estar vazio.
    let idx_repetido = export_anlz(&ExportOptions {
        anlz_dir: tmp.path().join("anlz"),
        music_dir: tmp.path().join("music"),
        output_dir: out1.clone(),
        limit: None,
    })
    .unwrap();
    assert_eq!(idx_repetido, idx1);
    assert_eq!(fs::read(out1.join("index.json")).unwrap(), index_bytes_1);

    let parsed = ReferenceTrack::from_file(out1.join(format!("{sha_a}.json"))).unwrap();
    assert_eq!(parsed.source_ref.as_deref(), Some("001/sub_a/ANLZ0000.DAT"));
}

#[test]
fn nao_le_audio_fora_de_music_dir() {
    let tmp = tempdir().unwrap();
    let anlz_dir = tmp.path().join("anlz");
    let music_dir = tmp.path().join("music");
    let secret_dir = tmp.path().join("outside");
    let output_dir = tmp.path().join("out");

    fs::create_dir_all(&music_dir).unwrap();
    fs::create_dir_all(&secret_dir).unwrap();
    let outside_audio = secret_dir.join("secreto.mp3");
    fs::write(&outside_audio, b"conteudo fora da raiz").unwrap();

    write_sample_anlz_pair(
        &anlz_dir.join("001/faixa"),
        &outside_audio.to_string_lossy(),
        &[(1, 1, 1)],
    );

    let index = export_anlz(&ExportOptions {
        anlz_dir,
        music_dir,
        output_dir: output_dir.clone(),
        limit: None,
    })
    .unwrap();

    assert_eq!(index.count, 0);
    assert_eq!(index.missing_audio, 1);
    let parsed_index: ExportIndex =
        serde_json::from_str(&fs::read_to_string(output_dir.join("index.json")).unwrap()).unwrap();
    assert!(parsed_index.tracks.is_empty());
}

#[test]
fn recusa_entrada_e_saida_sobrepostas() {
    let tmp = tempdir().unwrap();
    let anlz_dir = tmp.path().join("anlz");
    let music_dir = tmp.path().join("music");
    fs::create_dir_all(&anlz_dir).unwrap();
    fs::create_dir_all(&music_dir).unwrap();

    for bad_out in [
        anlz_dir.clone(),
        anlz_dir.join("sub"),
        music_dir.clone(),
        music_dir.join("sub"),
        tmp.path().to_path_buf(),
    ] {
        let err = export_anlz(&ExportOptions {
            anlz_dir: anlz_dir.clone(),
            music_dir: music_dir.clone(),
            output_dir: bad_out,
            limit: None,
        })
        .unwrap_err();
        assert!(matches!(err, AnlzError::CaminhoInvalido(_)));
    }
}

#[test]
fn falha_parcial_nao_deixa_index_valido() {
    let tmp = tempdir().unwrap();
    let anlz_dir = tmp.path().join("anlz");
    let music_dir = tmp.path().join("music");
    let output_dir = tmp.path().join("out");

    fs::create_dir_all(&music_dir).unwrap();
    fs::write(music_dir.join("ok.mp3"), b"audio ok").unwrap();
    fs::write(music_dir.join("bad.mp3"), b"audio ruim").unwrap();

    write_sample_anlz_pair(&anlz_dir.join("001/ok"), "/music/ok.mp3", &[(1, 1, 1)]);

    // Primeira execução válida cria index.json
    export_anlz(&ExportOptions {
        anlz_dir: anlz_dir.clone(),
        music_dir: music_dir.clone(),
        output_dir: output_dir.clone(),
        limit: None,
    })
    .unwrap();
    assert!(output_dir.join("index.json").is_file());
    let index_anterior = fs::read(output_dir.join("index.json")).unwrap();

    // Segunda faixa tem batida decrescente na PSSI, causando falha parcial
    write_sample_anlz_pair(
        &anlz_dir.join("002/bad"),
        "/music/bad.mp3",
        &[(1, 9, 1), (2, 1, 2)],
    );

    let res = export_anlz(&ExportOptions {
        anlz_dir,
        music_dir,
        output_dir: output_dir.clone(),
        limit: None,
    });
    assert!(res.is_err());
    assert_eq!(
        fs::read(output_dir.join("index.json")).unwrap(),
        index_anterior
    );
}

#[test]
fn nome_de_audio_ambiguo_nao_escolhe_arquivo_arbitrariamente() {
    let tmp = tempdir().unwrap();
    let anlz_dir = tmp.path().join("anlz");
    let music_dir = tmp.path().join("music");
    let output_dir = tmp.path().join("out");

    fs::create_dir_all(music_dir.join("a")).unwrap();
    fs::create_dir_all(music_dir.join("b")).unwrap();
    fs::write(music_dir.join("a/igual.mp3"), b"primeiro").unwrap();
    fs::write(music_dir.join("b/igual.mp3"), b"segundo").unwrap();
    write_sample_anlz_pair(
        &anlz_dir.join("001/faixa"),
        "C:/biblioteca/igual.mp3",
        &[(1, 1, 1)],
    );

    let index = export_anlz(&ExportOptions {
        anlz_dir,
        music_dir,
        output_dir,
        limit: None,
    })
    .unwrap();

    assert_eq!(index.count, 0);
    assert_eq!(index.missing_audio, 1);
}
