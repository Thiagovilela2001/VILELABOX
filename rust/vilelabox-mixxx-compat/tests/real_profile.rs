//! Paridade do decodificador `beats.proto` com as referências exportadas pelo
//! Python (`tools/reference/export_reference.py`) de um perfil real do Mixxx.
//! Só roda localmente: o perfil vem de `VILELABOX_MIXXX_PROFILE` (ou da pasta
//! `bars-profile-win` ao lado do repositório) e as referências de
//! `tests/reference/mixxx/`, ambos fora do git.

use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use vilelabox_mixxx_compat::MixxxDb;

fn profile_dir() -> PathBuf {
    std::env::var_os("VILELABOX_MIXXX_PROFILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../bars-profile-win"))
}

fn read_json(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn beat_grids_match_python_references() {
    let db_path = profile_dir().join("mixxxdb.sqlite");
    let refs = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/reference/mixxx");
    if !db_path.is_file() || !refs.join("locations.json").is_file() {
        eprintln!("sem perfil do Mixxx ou referências locais; teste ignorado");
        return;
    }
    let locations: HashMap<String, String> =
        serde_json::from_value(read_json(&refs.join("locations.json"))).unwrap();

    let db = MixxxDb::open(&db_path).unwrap();
    let (mut checked, mut failures) = (0, Vec::new());
    for track in db.read_tracks().unwrap() {
        let Some(sha) = locations.get(&track.location) else {
            continue; // áudio ausente: o exportador também ignorou
        };
        let reference = read_json(&refs.join(format!("{}.json", sha)));
        let Some(result) = track.beat_grid() else {
            assert!(
                reference.get("first_beat_time_s").is_none(),
                "{}: referência tem grade e o Rust não",
                sha
            );
            continue;
        };
        let grid = result.unwrap_or_else(|e| panic!("{}: {}", sha, e));
        checked += 1;

        let rate = f64::from(grid.sample_rate().get());
        let first = grid.beats()[0].position.get() as f64 / rate;
        let expected_first = reference["first_beat_time_s"].as_f64().unwrap();
        if (first - expected_first).abs() > 1.0 / rate {
            failures.push(format!(
                "{}: primeira batida {} != {}",
                sha, first, expected_first
            ));
        }

        let phase = grid
            .downbeat()
            .map(|d| grid.beats().iter().position(|b| b.position == d).unwrap() as u64);
        if phase != reference["downbeat_index"].as_u64() {
            failures.push(format!(
                "{}: fase {:?} != {}",
                sha, phase, reference["downbeat_index"]
            ));
        }

        let variable = grid.tempo_map().len() > 1;
        let expected_variable = reference["tempo_type"] == "variable";
        if variable && !expected_variable {
            failures.push(format!("{}: grade fixa virou variável", sha));
        }
        if !expected_variable {
            let bpm = grid.primary_bpm().unwrap().get();
            let expected = reference["bpm"].as_f64().unwrap();
            if (bpm - expected).abs() > 0.01 {
                failures.push(format!("{}: bpm {} != {}", sha, bpm, expected));
            }
        }
    }
    eprintln!("{} grades comparadas", checked);
    assert!(
        checked > 0,
        "nenhuma faixa com grade casou com as referências"
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
