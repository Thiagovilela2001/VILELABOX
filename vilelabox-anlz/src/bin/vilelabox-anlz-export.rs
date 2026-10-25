use std::path::PathBuf;
use std::process::ExitCode;
use vilelabox_anlz::{export_anlz, ExportOptions};

const USAGE: &str = "\
Uso: vilelabox-anlz-export --anlz-dir <cópia USBANLZ> --music-dir <cópia da biblioteca> --output <diretório> [--limit N]";

fn parse_args<I>(args: I) -> Result<ExportOptions, String>
where
    I: IntoIterator<Item = String>,
{
    let mut anlz_dir: Option<PathBuf> = None;
    let mut music_dir: Option<PathBuf> = None;
    let mut output_dir: Option<PathBuf> = None;
    let mut limit: Option<usize> = None;

    let mut iter = args.into_iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--anlz-dir" => {
                let val = iter
                    .next()
                    .ok_or_else(|| "valor ausente para --anlz-dir".to_string())?;
                anlz_dir = Some(PathBuf::from(val));
            }
            "--music-dir" => {
                let val = iter
                    .next()
                    .ok_or_else(|| "valor ausente para --music-dir".to_string())?;
                music_dir = Some(PathBuf::from(val));
            }
            "--output" => {
                let val = iter
                    .next()
                    .ok_or_else(|| "valor ausente para --output".to_string())?;
                output_dir = Some(PathBuf::from(val));
            }
            "--limit" => {
                let val = iter
                    .next()
                    .ok_or_else(|| "valor ausente para --limit".to_string())?;
                let parsed = val
                    .parse::<usize>()
                    .map_err(|_| format!("valor inválido para --limit: {val}"))?;
                limit = Some(parsed);
            }
            "-h" | "--help" => {
                return Err(USAGE.to_string());
            }
            other => {
                return Err(format!("argumento desconhecido: {other}\n{USAGE}"));
            }
        }
    }

    let anlz_dir = anlz_dir.ok_or_else(|| format!("--anlz-dir obrigatório\n{USAGE}"))?;
    let music_dir = music_dir.ok_or_else(|| format!("--music-dir obrigatório\n{USAGE}"))?;
    let output_dir = output_dir.ok_or_else(|| format!("--output obrigatório\n{USAGE}"))?;

    Ok(ExportOptions {
        anlz_dir,
        music_dir,
        output_dir,
        limit,
    })
}

fn main() -> ExitCode {
    let options = match parse_args(std::env::args().skip(1)) {
        Ok(opts) => opts,
        Err(msg) => {
            eprintln!("{msg}");
            return ExitCode::FAILURE;
        }
    };

    match export_anlz(&options) {
        Ok(index) => {
            eprintln!(
                "Exportadas {} faixas ({} com frases, {} com cues, {} sem áudio).",
                index.count, index.with_phrases, index.with_cues, index.missing_audio
            );
            ExitCode::SUCCESS
        }
        Err(err) => {
            eprintln!("erro ao exportar ANLZ: {err}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpreta_argumentos_validos_e_limite() {
        let opts = parse_args([
            "--anlz-dir".into(),
            "a".into(),
            "--music-dir".into(),
            "m".into(),
            "--output".into(),
            "o".into(),
            "--limit".into(),
            "40".into(),
        ])
        .unwrap();
        assert_eq!(opts.anlz_dir, PathBuf::from("a"));
        assert_eq!(opts.music_dir, PathBuf::from("m"));
        assert_eq!(opts.output_dir, PathBuf::from("o"));
        assert_eq!(opts.limit, Some(40));
    }

    #[test]
    fn rejeita_argumentos_incompletos_ou_invalidos() {
        assert!(parse_args(["--anlz-dir".into(), "a".into()]).is_err());
        assert!(parse_args([
            "--anlz-dir".into(),
            "a".into(),
            "--music-dir".into(),
            "m".into(),
            "--output".into(),
            "o".into(),
            "--limit".into(),
            "abc".into(),
        ])
        .is_err());
    }
}
