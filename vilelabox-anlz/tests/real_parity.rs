use std::fs;
use std::path::{Path, PathBuf};
use vilelabox_anlz::track_from_anlz;
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
fn copied_anlz_matches_local_references() {
    let Some(anlz_dir) = std::env::var_os("VILELABOX_ANLZ_COPY").map(PathBuf::from) else {
        eprintln!("VILELABOX_ANLZ_COPY ausente; paridade real ignorada");
        return;
    };
    let reference_dir = std::env::var_os("VILELABOX_REFERENCE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("tests/reference/rekordbox"));

    assert!(
        anlz_dir.is_dir(),
        "cópia ANLZ ausente: {}",
        anlz_dir.display()
    );
    assert!(
        reference_dir.is_dir(),
        "referências ausentes: {}",
        reference_dir.display()
    );

    let comparator = DifferentialComparator::new();
    let mut checked = 0usize;

    for reference_path in reference_files(&reference_dir) {
        let expected = ReferenceTrack::from_file(&reference_path).unwrap();
        let source_ref = expected.source_ref.as_deref().expect("source_ref ausente");
        let dat_path = anlz_dir.join(source_ref);
        let ext_path = dat_path.with_extension("EXT");
        let dat = fs::read(&dat_path).unwrap_or_else(|error| {
            panic!("{}: {error}", dat_path.display());
        });
        let ext = fs::read(&ext_path).ok();
        let actual = track_from_anlz(&dat, ext.as_deref(), &expected.sha256, source_ref).unwrap();
        let report = comparator.compare(&expected, &actual);
        assert!(
            report.passed,
            "{}: {:?}",
            dat_path.display(),
            report.metrics
        );
        checked += 1;
    }

    assert!(checked > 0, "nenhuma referência real encontrada");
    eprintln!("{checked} faixas ANLZ comparadas");
}
