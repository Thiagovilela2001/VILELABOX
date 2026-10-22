//! CLI da ponte rekordbox.xml <-> mixxxdb.sqlite.

use std::env;
use std::path::{Path, PathBuf};
use std::process;
use vilelabox_rekordbox::{
    export_mixxx_to_rekordbox_xml, get_status, import_rekordbox_xml_to_mixxx,
};

fn default_mixxx_db() -> PathBuf {
    if let Ok(home) = env::var("HOME").or_else(|_| env::var("USERPROFILE")) {
        Path::new(&home).join(".mixxx").join("mixxxdb.sqlite")
    } else {
        PathBuf::from("mixxxdb.sqlite")
    }
}

fn default_xml_path() -> PathBuf {
    if let Ok(home) = env::var("HOME").or_else(|_| env::var("USERPROFILE")) {
        Path::new(&home)
            .join("mixxx-rekordbox")
            .join("rekordbox.xml")
    } else {
        PathBuf::from("rekordbox.xml")
    }
}

fn print_usage() {
    println!(
        "Mixxx <-> Rekordbox Realtime XML Synchronization Bridge (Rust)
=======================================================
Permite compartilhar faixas, Hot Cues, Loops, Beatgrid e Playlists
entre o Mixxx e o Rekordbox sem quebrar a integridade dos bancos de dados.

Uso:
  vilelabox-rekordbox export [caminho/rekordbox.xml]
  vilelabox-rekordbox import <caminho/rekordbox.xml>
  vilelabox-rekordbox status [caminho/rekordbox.xml]"
    );
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 || args[1] == "-h" || args[1] == "--help" {
        print_usage();
        process::exit(0);
    }

    let cmd = args[1].to_lowercase();
    let db_path = default_mixxx_db();
    let target_xml = if args.len() > 2 {
        PathBuf::from(&args[2])
    } else {
        default_xml_path()
    };

    match cmd.as_str() {
        "export" => {
            println!("[*] Conectando ao banco do Mixxx: {}", db_path.display());
            match export_mixxx_to_rekordbox_xml(&db_path, &target_xml) {
                Ok((tracks, pls)) => {
                    println!(
                        "[+] Sucesso! Exportadas {} faixas e {} playlists para:",
                        tracks, pls
                    );
                    println!("    {}", target_xml.display());
                }
                Err(e) => {
                    eprintln!("[!] Erro na exportação: {}", e);
                    process::exit(1);
                }
            }
        }
        "import" => {
            println!("[*] Lendo XML do Rekordbox: {}", target_xml.display());
            match import_rekordbox_xml_to_mixxx(&target_xml, &db_path) {
                Ok((cues, pls)) => {
                    println!(
                        "[+] Sucesso! Sincronizados {} Hot Cues e {} Playlists no Mixxx.",
                        cues, pls
                    );
                }
                Err(e) => {
                    eprintln!("[!] Erro na importação: {}", e);
                    process::exit(1);
                }
            }
        }
        "status" => {
            let status = get_status(&db_path, &target_xml);
            println!("==================================================");
            println!("      MIXXX <-> REKORDBOX SYNC BRIDGE STATUS      ");
            println!("==================================================");
            if status.db_exists {
                println!("[Mixxx DB] Local: {}", status.db_path.display());
                println!(
                    "           Faixas: {} | Playlists: {} | Hot Cues: {}",
                    status.track_count, status.playlist_count, status.hotcue_count
                );
            } else {
                println!("[Mixxx DB] Não encontrado em: {}", status.db_path.display());
            }
            println!("--------------------------------------------------");
            if status.xml_exists {
                println!("[Rekordbox XML] Local: {}", status.xml_path.display());
                println!(
                    "                Tamanho: {:.1} KB | Modificado: {}",
                    status.xml_size_kb, status.xml_mtime
                );
                println!("                Faixas no XML: {}", status.xml_entries);
            } else {
                println!(
                    "[Rekordbox XML] Ainda não gerado em: {}",
                    status.xml_path.display()
                );
                println!("                Execute: vilelabox-rekordbox export");
            }
            println!("==================================================");
        }
        _ => {
            eprintln!("[!] Comando desconhecido: '{}'", cmd);
            print_usage();
            process::exit(1);
        }
    }
}
