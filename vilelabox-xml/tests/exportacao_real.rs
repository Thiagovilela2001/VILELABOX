//! Lê a exportação real apontada por `VLBX_XML` e confere ida e volta.
//! Sem a variável (ou sem o arquivo), o teste passa sem fazer nada.

#[test]
fn exportacao_real_ida_e_volta() {
    let Some(path) = std::env::var_os("VLBX_XML") else {
        return;
    };
    let Ok(xml) = std::fs::read_to_string(&path) else {
        return;
    };
    let d = vilelabox_xml::parse(&xml).expect("parse");
    assert!(!d.tracks.is_empty());
    let sem_location = d.tracks.iter().filter(|t| t.location.is_none()).count();
    assert_eq!(sem_location, 0, "Location não decodificada");
    let d2 = vilelabox_xml::parse(&vilelabox_xml::write(&d)).expect("reparse");
    assert_eq!(d, d2);
}
