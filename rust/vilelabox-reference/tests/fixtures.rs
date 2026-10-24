//! Lê toda a matriz de `tests/reference/` pelo esquema canônico e prova que o
//! comparador está ativo em cada fixture. As referências do rekordbox não são
//! versionadas (biblioteca pessoal): quando o diretório existe, entram também.

use serde_json::Value;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use vilelabox_reference::{
    DifferentialComparator, ReferencePlaylistSet, ReferenceTrack, Tolerances,
};

/// Matriz mínima de R1.3 (PLANO-RUST.md).
const REQUIRED_SYNTHETIC: &[&str] = &[
    "fixed_120bpm",
    "tempo_change_120_to_140bpm",
    "silent_intro_125bpm",
    "first_beat_before_zero_128bpm",
    "no_downbeat_120bpm",
    "short_audio_1s",
    "silence_10s",
    "cues_and_loops_124bpm",
    "phrases_8_16_32_bars_128bpm",
    "instrumental_126bpm",
    "vocal_segments_126bpm",
];

fn reference_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/reference")
}

/// Faixas do diretório, conferidas contra o `index.json`.
fn load_tracks(dir: &Path) -> Vec<ReferenceTrack> {
    let index: Value =
        serde_json::from_str(&fs::read_to_string(dir.join("index.json")).unwrap()).unwrap();
    let skip: HashSet<String> = index["playlists"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(String::from))
        .chain(["index.json".to_string(), "locations.json".to_string()])
        .collect();

    let mut tracks = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        let file = path.file_name().unwrap().to_string_lossy().to_string();
        if path.extension().is_some_and(|e| e == "json") && !skip.contains(&file) {
            let track = ReferenceTrack::from_file(&path)
                .unwrap_or_else(|e| panic!("{}: {}", path.display(), e));
            tracks.push(track);
        }
    }

    let indexed: HashSet<&str> = index["tracks"]
        .as_array()
        .expect("index.json sem lista tracks")
        .iter()
        .map(|t| t["sha256"].as_str().unwrap())
        .collect();
    let on_disk: HashSet<&str> = tracks.iter().map(|t| t.sha256.as_str()).collect();
    assert_eq!(
        indexed,
        on_disk,
        "{}: index.json e arquivos divergem",
        dir.display()
    );
    assert_eq!(index["count"].as_u64(), Some(tracks.len() as u64));
    tracks
}

/// A referência passa contra si mesma e falha com o BPM deslocado.
fn assert_comparator_active(track: &ReferenceTrack) {
    let comparator =
        DifferentialComparator::with_tolerances(Tolerances::for_source(track.source.as_deref()));
    let label = track.name.as_deref().unwrap_or(&track.sha256);
    let report = comparator.compare(track, track);
    assert!(report.passed, "{}: {:?}", label, report.metrics);

    if let Some(bpm) = track.bpm {
        let mut shifted = track.clone();
        shifted.bpm = Some(bpm + 0.05);
        assert!(!comparator.compare(track, &shifted).passed, "{}", label);
    }
}

#[test]
fn synthetic_matrix_is_complete_and_valid() {
    let dir = reference_dir().join("synthetic");
    let tracks = load_tracks(&dir);
    let names: HashSet<&str> = tracks.iter().filter_map(|t| t.name.as_deref()).collect();
    for required in REQUIRED_SYNTHETIC {
        assert!(names.contains(required), "fixture ausente: {}", required);
    }
    for track in &tracks {
        assert_eq!(track.source.as_deref(), Some("synthetic"));
        assert_comparator_active(track);
    }

    let by_name = |n: &str| {
        tracks
            .iter()
            .find(|t| t.name.as_deref() == Some(n))
            .unwrap()
    };
    assert!(
        by_name("first_beat_before_zero_128bpm")
            .first_beat_time_s
            .unwrap()
            < 0.0
    );
    assert!(by_name("no_downbeat_120bpm").downbeat_index.is_none());
    assert!(by_name("silence_10s").bpm.is_none());

    let cues = &by_name("cues_and_loops_124bpm").cues;
    let count = |kind: &str| cues.iter().filter(|c| c.type_name == kind).count();
    assert_eq!((count("hotcue"), count("memory"), count("loop")), (8, 2, 2));
    let rate = 44100.0;
    assert!(cues
        .iter()
        .any(|c| (c.position_s * rate).fract().abs() > 1e-6));

    let bars: Vec<_> = by_name("phrases_8_16_32_bars_128bpm")
        .phrases
        .iter()
        .map(|p| p.bars)
        .collect();
    assert_eq!(bars, [Some(8), Some(16), Some(32)]);

    let vocal = by_name("vocal_segments_126bpm");
    assert_eq!(vocal.has_vocal, Some(true));
    assert_eq!(vocal.vocal_segments.as_ref().unwrap().len(), 2);
    assert_eq!(by_name("instrumental_126bpm").has_vocal, Some(false));
}

#[test]
fn synthetic_playlists_cover_empty_nested_and_homonyms() {
    let dir = reference_dir().join("synthetic");
    let set = ReferencePlaylistSet::from_file(dir.join("playlists_tree.json")).unwrap();
    let flat = set.flatten();

    assert!(flat.iter().any(|(_, tracks)| tracks.is_empty()));
    assert!(flat.iter().any(|(path, _)| path.matches('/').count() >= 2));
    let favoritas: Vec<_> = flat
        .iter()
        .filter(|(path, _)| path.rsplit('/').next().unwrap().starts_with("Favoritas["))
        .collect();
    assert!(favoritas.len() >= 3, "homônimas: {:?}", favoritas);

    // Toda faixa citada existe na matriz.
    let known: HashSet<String> = load_tracks(&dir).into_iter().map(|t| t.sha256).collect();
    for (path, tracks) in &flat {
        for sha in tracks {
            assert!(known.contains(sha), "{}: faixa desconhecida {}", path, sha);
        }
    }
    assert!(
        DifferentialComparator::new()
            .compare_playlists(&set, &set)
            .passed
    );
}

/// Exportações locais (`export_rekordbox_anlz.py`, `export_reference.py`).
fn real_references_parse_when_present(subdir: &str, source: &str) {
    let dir = reference_dir().join(subdir);
    if !dir.join("index.json").is_file() {
        eprintln!(
            "sem referências locais em {}; teste ignorado",
            dir.display()
        );
        return;
    }
    for track in load_tracks(&dir) {
        assert_eq!(track.source.as_deref(), Some(source));
        assert_comparator_active(&track);
    }
}

#[test]
fn rekordbox_references_parse_when_present() {
    real_references_parse_when_present("rekordbox", "rekordbox_anlz");
}

#[test]
fn mixxx_references_parse_when_present() {
    real_references_parse_when_present("mixxx", "mixxx");
}
