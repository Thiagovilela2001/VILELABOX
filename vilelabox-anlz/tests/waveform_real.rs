//! Confere as waveforms num `.EXT` real apontado por `VLBX_ANLZ_EXT`.
//! Sem a variável (ou sem o arquivo), o teste passa sem fazer nada.

use vilelabox_anlz::waveform_from_ext;

#[test]
fn waveform_de_ext_real() {
    let Some(path) = std::env::var_os("VLBX_ANLZ_EXT") else {
        return;
    };
    let Ok(ext) = std::fs::read(&path) else {
        return;
    };
    let (preview, detail) = waveform_from_ext(&ext);
    let preview = preview.expect("PWV4 ausente");
    assert_eq!(preview.red.len(), 1200);
    let detail = detail.expect("PWV5/PWV3 ausente");
    assert!(!detail.height.is_empty());
    assert!(detail.height.iter().all(|&h| h <= 31));
}
