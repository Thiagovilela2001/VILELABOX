//! CLI headless de desenvolvimento (P3.4): prova o fluxo de preparação sem
//! interface gráfica. Cada invocação abre o banco, executa um comando pelo
//! `Core`, imprime os eventos e encerra; o estado vive só no banco.
//!
//! Uso: `vilelabox-cli <banco.db> <subcomando> [args]`

use std::path::PathBuf;
use std::process::ExitCode;
use vilelabox_core::{Comando, Core, Deck};
use vilelabox_library::Consulta;
use vilelabox_types::{
    Beat, BeatGrid, Bpm, Cue, CueKind, CueOrigin, Frame, SampleRate, TempoChange, TrackId,
};

const USO: &str = "uso: vilelabox-cli <banco.db> <subcomando>
  importar dir|catalogo|xml <caminho>
  verificar
  movidos <dir>
  pesquisar [texto]
  listas
  lista-criar <nome>
  lista-add <lista> <faixa>
  lista-mover <lista> <de> <para>
  deck <faixa>
  cue <faixa> <frame>
  cue-mover <faixa> <indice> <frame>
  cue-remover <faixa> <indice>
  grade <faixa> <taxa> <bpm> <primeiro_frame> <batidas> [bloquear]";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match rodar(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("erro: {e}");
            ExitCode::FAILURE
        }
    }
}

fn num<T: std::str::FromStr>(args: &[String], i: usize) -> Result<T, String> {
    args.get(i)
        .ok_or_else(|| USO.to_string())?
        .parse()
        .map_err(|_| format!("argumento inválido: {}", args[i]))
}

fn texto(args: &[String], i: usize) -> Result<&str, String> {
    args.get(i)
        .map(String::as_str)
        .ok_or_else(|| USO.to_string())
}

fn rodar(args: &[String]) -> Result<(), String> {
    // Sem subcomando (inclui `--help`) não abre banco: abrir criaria o arquivo.
    if args.len() < 2 || args[0].starts_with('-') {
        return Err(USO.into());
    }
    let banco = PathBuf::from(texto(args, 0)?);
    let mut core = Core::abrir(&banco).map_err(|e| e.to_string())?;
    let faixa = |i| num::<i64>(args, i).map(TrackId);
    let carregar = |i| -> Result<Comando, String> {
        Ok(Comando::CarregarNoDeck {
            deck: Deck::Um,
            faixa: faixa(i)?,
        })
    };

    let comandos = match texto(args, 1)? {
        "importar" => {
            let caminho = PathBuf::from(texto(args, 3)?);
            vec![match texto(args, 2)? {
                "dir" => Comando::ImportarDiretorio(caminho),
                "catalogo" => Comando::ImportarCatalogo(caminho),
                "xml" => Comando::ImportarXml(caminho),
                _ => return Err(USO.into()),
            }]
        }
        "verificar" => vec![Comando::VerificarLocalizacoes],
        "movidos" => vec![Comando::BuscarMovidos(texto(args, 2)?.into())],
        "pesquisar" => {
            let consulta = Consulta {
                texto: args.get(2).cloned().unwrap_or_default(),
                ..Consulta::default()
            };
            for f in core.pesquisar(&consulta).map_err(|e| e.to_string())? {
                let m = &f.metadados;
                println!(
                    "{}\t{}\t{}",
                    f.id.0,
                    m.artist.as_deref().unwrap_or(""),
                    m.title.as_deref().unwrap_or("")
                );
            }
            vec![]
        }
        "listas" => {
            for l in core.listas().map_err(|e| e.to_string())? {
                let faixas = core.faixas_da_lista(l.id).map_err(|e| e.to_string())?;
                let ids: Vec<i64> = faixas.iter().map(|t| t.0).collect();
                println!("{}\t{}\t{ids:?}", l.id, l.nome);
            }
            vec![]
        }
        "lista-criar" => vec![Comando::CriarLista {
            nome: texto(args, 2)?.into(),
            pai: None,
            pasta: false,
        }],
        "lista-add" => vec![Comando::AdicionarALista {
            lista: num(args, 2)?,
            faixa: faixa(3)?,
        }],
        "lista-mover" => vec![Comando::MoverNaLista {
            lista: num(args, 2)?,
            de: num(args, 3)?,
            para: num(args, 4)?,
        }],
        "deck" => vec![carregar(2)?],
        "cue" => vec![
            Comando::SelecionarFaixa(faixa(2)?),
            carregar(2)?,
            Comando::CriarCue {
                deck: Deck::Um,
                cue: Cue::new(
                    CueKind::MemoryCue,
                    Frame::new(num(args, 3)?),
                    CueOrigin::User,
                ),
            },
        ],
        "cue-mover" => vec![
            carregar(2)?,
            Comando::MoverCue {
                deck: Deck::Um,
                indice: num(args, 3)?,
                posicao: Frame::new(num(args, 4)?),
            },
        ],
        "cue-remover" => vec![
            carregar(2)?,
            Comando::RemoverCue {
                deck: Deck::Um,
                indice: num(args, 3)?,
            },
        ],
        "grade" => vec![carregar(2)?, salvar_grade(args)?],
        _ => return Err(USO.into()),
    };

    for c in comandos {
        core.executar(c).map_err(|e| e.to_string())?;
    }
    for e in core.drenar_eventos() {
        println!("evento: {e:?}");
    }
    if let Some(d) = core.deck(Deck::Um) {
        for (i, c) in d.cues.iter().enumerate() {
            println!("cue {i}: {:?} {}", c.kind, c.position.get());
        }
        if let Some(g) = &d.grade {
            println!(
                "grade: {} batidas, bloqueada={}",
                g.beats().len(),
                d.bloqueada
            );
        }
    }
    Ok(())
}

/// Grade de tempo constante (compasso 4/4) para provar a persistência.
fn salvar_grade(args: &[String]) -> Result<Comando, String> {
    let taxa = SampleRate::new(num(args, 3)?).map_err(|e| e.to_string())?;
    let bpm = Bpm::new(num(args, 4)?).map_err(|e| e.to_string())?;
    let inicio: i64 = num(args, 5)?;
    let n: u32 = num(args, 6)?;
    let passo = 60.0 * taxa.get() as f64 / bpm.get();
    let beats = (0..n)
        .map(|i| Beat {
            position: Frame::new(inicio + (i as f64 * passo).round() as i64),
            beat_num: i % 4 + 1,
        })
        .collect();
    let tempo = vec![TempoChange {
        position: Frame::new(inicio),
        bpm,
    }];
    let grade =
        BeatGrid::new(taxa, beats, tempo, Some(Frame::new(inicio))).map_err(|e| e.to_string())?;
    Ok(Comando::SalvarGrade {
        deck: Deck::Um,
        grade,
        bloqueada: args.get(7).is_some_and(|a| a == "bloquear"),
    })
}
