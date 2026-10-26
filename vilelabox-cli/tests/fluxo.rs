//! Gate G3: fluxo completo sem interface, com estado preservado entre
//! processos (cada chamada da CLI é um processo novo sobre o mesmo banco).

use std::path::Path;
use std::process::Command;

fn cli(banco: &Path, args: &[&str]) -> String {
    let saida = Command::new(env!("CARGO_BIN_EXE_vilelabox-cli"))
        .arg(banco)
        .args(args)
        .output()
        .unwrap();
    let texto = String::from_utf8(saida.stdout).unwrap();
    assert!(
        saida.status.success(),
        "{args:?}: {}{texto}",
        String::from_utf8_lossy(&saida.stderr)
    );
    texto
}

#[test]
fn importar_pesquisar_selecionar_carregar_editar_cue_persistir() {
    let dir = std::env::temp_dir().join(format!("vilelabox-cli-fluxo-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let musica = dir.join("musica");
    std::fs::create_dir_all(&musica).unwrap();
    std::fs::write(musica.join("DJ A - Alpha.mp3"), b"a").unwrap();
    std::fs::write(musica.join("DJ B - Beta.wav"), b"b").unwrap();
    let banco = dir.join("vilelabox.db");

    let out = cli(&banco, &["importar", "dir", musica.to_str().unwrap()]);
    assert!(
        out.contains("TarefaAnaliseAtualizada { pendentes: 2 }"),
        "{out}"
    );

    let out = cli(&banco, &["pesquisar", "beta"]);
    let linhas: Vec<&str> = out.lines().collect();
    assert_eq!(linhas.len(), 1, "{out}");
    assert!(linhas[0].ends_with("DJ B\tBeta"));
    let id = linhas[0].split('\t').next().unwrap();

    let out = cli(&banco, &["cue", id, "44100"]);
    assert!(out.contains("SelecaoAlterada"), "{out}");
    cli(&banco, &["cue", id, "22050"]);
    cli(&banco, &["cue-mover", id, "1", "88200"]);
    cli(&banco, &["grade", id, "44100", "120", "0", "8", "bloquear"]);
    cli(&banco, &["lista-criar", "Set"]);
    cli(&banco, &["lista-add", "1", id]);

    // Processo novo: tudo vem do banco.
    let out = cli(&banco, &["deck", id]);
    assert!(out.contains("cue 0: MemoryCue 22050"), "{out}");
    assert!(out.contains("cue 1: MemoryCue 88200"), "{out}");
    assert!(out.contains("grade: 8 batidas, bloqueada=true"), "{out}");
    let out = cli(&banco, &["listas"]);
    assert!(out.contains(&format!("Set\t[{id}]")), "{out}");

    let falha = Command::new(env!("CARGO_BIN_EXE_vilelabox-cli"))
        .arg(&banco)
        .args(["deck", "999"])
        .output()
        .unwrap();
    assert!(!falha.status.success());
}

#[test]
fn ajuda_nao_cria_banco() {
    let dir = std::env::temp_dir().join(format!("vilelabox-cli-ajuda-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for args in [&["--help"][..], &["banco.db"][..]] {
        let saida = Command::new(env!("CARGO_BIN_EXE_vilelabox-cli"))
            .current_dir(&dir)
            .args(args)
            .output()
            .unwrap();
        assert!(!saida.status.success());
        assert!(String::from_utf8_lossy(&saida.stderr).contains("uso:"));
    }
    assert_eq!(std::fs::read_dir(&dir).unwrap().count(), 0);
}
