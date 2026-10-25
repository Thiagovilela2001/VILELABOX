use std::fs;
use std::path::{Path, PathBuf};
use tempfile::tempdir;
use vilelabox_anlz::{export_anlz, ExportIndex, ExportOptions};
use vilelabox_reference::{DifferentialComparator, ReferenceTrack};

fn reference_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<_> = fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension().is_some_and(|ext| ext == "json")
                && path.file_name().is_some_and(|name| name != "index.json")
        })
        .collect();
    files.sort();
    files
}

#[test]
fn export_matches_reference_collection() {
    let Some(anlz_dir) = std::env::var_os("VILELABOX_ANLZ_COPY").map(PathBuf::from) else {
        eprintln!("VILELABOX_ANLZ_COPY ausente; paridade do exportador ignorada");
        return;
    };
    let Some(music_dir) = std::env::var_os("VILELABOX_MUSIC_COPY").map(PathBuf::from) else {
        eprintln!("VILELABOX_MUSIC_COPY ausente; paridade do exportador ignorada");
        return;
    };
    let reference_dir = std::env::var_os("VILELABOX_REFERENCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("../tests/reference/rekordbox"));
    let reference_dir = if reference_dir.is_dir() {
        reference_dir
    } else {
        PathBuf::from("tests/reference/rekordbox")
    };

    assert!(
        anlz_dir.is_dir(),
        "cópia ANLZ ausente: {}",
        anlz_dir.display()
    );
    assert!(
        music_dir.is_dir(),
        "cópia de música ausente: {}",
        music_dir.display()
    );
    assert!(
        reference_dir.is_dir(),
        "referências ausentes: {}",
        reference_dir.display()
    );

    let out_tmp = tempdir().unwrap();
    let output_dir = out_tmp.path().join("exported");

    let actual_index = export_anlz(&ExportOptions {
        anlz_dir,
        music_dir,
        output_dir: output_dir.clone(),
        limit: Some(40),
    })
    .unwrap();

    let expected_index: ExportIndex =
        serde_json::from_str(&fs::read_to_string(reference_dir.join("index.json")).unwrap())
            .unwrap();
    assert_eq!(actual_index, expected_index);

    let comparator = DifferentialComparator::new();
    let expected_files = reference_files(&reference_dir);
    assert_eq!(expected_files.len(), 40);

    for expected_path in expected_files {
        let file_name = expected_path.file_name().unwrap();
        let actual_path = output_dir.join(file_name);
        assert!(
            actual_path.is_file(),
            "arquivo exportado ausente: {}",
            actual_path.display()
        );
        let expected = ReferenceTrack::from_file(&expected_path).unwrap();
        let actual = ReferenceTrack::from_file(&actual_path).unwrap();
        let report = comparator.compare(&expected, &actual);
        assert!(
            report.passed,
            "{}: {:?}",
            actual_path.display(),
            report.metrics
        );
        assert_eq!(expected, actual);
    }
}
