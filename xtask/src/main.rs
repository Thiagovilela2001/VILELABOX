mod spec_lint;

use std::env;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn arquivos_markdown(path: &Path) -> Result<Vec<PathBuf>, String> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    if !path.is_dir() {
        return Err(format!("caminho não encontrado: {}", path.display()));
    }

    let mut pendentes = vec![path.to_path_buf()];
    let mut arquivos = Vec::new();
    while let Some(diretorio) = pendentes.pop() {
        let entradas = std::fs::read_dir(&diretorio)
            .map_err(|erro| format!("não foi possível ler {}: {erro}", diretorio.display()))?;
        for entrada in entradas {
            let entrada = entrada
                .map_err(|erro| format!("não foi possível ler {}: {erro}", diretorio.display()))?;
            let caminho = entrada.path();
            let tipo = entrada.file_type().map_err(|erro| {
                format!("não foi possível examinar {}: {erro}", caminho.display())
            })?;
            if tipo.is_dir() {
                pendentes.push(caminho);
            } else if tipo.is_file()
                && caminho.extension().is_some_and(|extensao| extensao == "md")
                && caminho.file_name().is_some_and(|nome| nome != "README.md")
            {
                arquivos.push(caminho);
            }
        }
    }
    arquivos.sort();
    Ok(arquivos)
}

fn executar_spec_lint(path: &Path) -> Result<bool, String> {
    let arquivos = arquivos_markdown(path)?;
    let mut limpo = true;
    for arquivo in arquivos {
        let texto = std::fs::read_to_string(&arquivo)
            .map_err(|erro| format!("não foi possível ler {}: {erro}", arquivo.display()))?;
        for violacao in spec_lint::lint_spec(&texto) {
            println!(
                "{}:{}: {}",
                arquivo.display(),
                violacao.line,
                violacao.rule.code()
            );
            limpo = false;
        }
    }
    Ok(limpo)
}

fn main() -> ExitCode {
    let mut argumentos = env::args_os().skip(1);
    let Some(comando) = argumentos.next() else {
        eprintln!("uso: cargo run -p xtask -- spec-lint <arquivo-ou-diretório>");
        return ExitCode::from(2);
    };
    let Some(caminho) = argumentos.next() else {
        eprintln!("uso: cargo run -p xtask -- spec-lint <arquivo-ou-diretório>");
        return ExitCode::from(2);
    };
    if comando != "spec-lint" || argumentos.next().is_some() {
        eprintln!("uso: cargo run -p xtask -- spec-lint <arquivo-ou-diretório>");
        return ExitCode::from(2);
    }

    match executar_spec_lint(Path::new(&caminho)) {
        Ok(true) => ExitCode::SUCCESS,
        Ok(false) => ExitCode::from(1),
        Err(erro) => {
            eprintln!("erro: {erro}");
            ExitCode::from(2)
        }
    }
}
