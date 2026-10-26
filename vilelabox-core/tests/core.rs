//! Comandos e eventos do núcleo headless (P3.3, fase C1).

use std::path::PathBuf;
use vilelabox_core::{Comando, Core, Deck, ErroCore, Evento};
use vilelabox_library::Consulta;
use vilelabox_storage::Database;
use vilelabox_types::{
    Beat, BeatGrid, Bpm, Cue, CueKind, CueOrigin, Frame, SampleRate, TempoChange, TrackId,
};

fn pasta(nome: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vilelabox-core-{nome}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Core em memória com duas faixas importadas de uma pasta temporária.
fn core_com_faixas(nome: &str) -> (Core, Vec<TrackId>) {
    let dir = pasta(nome);
    std::fs::write(dir.join("DJ A - Primeira.mp3"), b"a").unwrap();
    std::fs::write(dir.join("DJ B - Segunda.wav"), b"b").unwrap();
    let mut core = Core::novo(Database::open_in_memory().unwrap());
    core.executar(Comando::ImportarDiretorio(dir)).unwrap();
    let ids = core
        .pesquisar(&Consulta::default())
        .unwrap()
        .iter()
        .map(|f| f.id)
        .collect();
    (core, ids)
}

fn memory(frame: i64) -> Cue {
    Cue::new(CueKind::MemoryCue, Frame::new(frame), CueOrigin::Imported)
}

#[test]
fn importacao_emite_biblioteca_e_tarefa_de_analise() {
    let (mut core, ids) = core_com_faixas("importa");
    assert_eq!(ids.len(), 2);
    let ev = core.drenar_eventos();
    assert!(matches!(ev[0], Evento::BibliotecaAlterada { .. }));
    assert_eq!(ev[1], Evento::TarefaAnaliseAtualizada { pendentes: 2 });
    assert!(core.drenar_eventos().is_empty());
}

#[test]
fn selecao_valida_e_invalida() {
    let (mut core, ids) = core_com_faixas("selecao");
    core.drenar_eventos();
    core.executar(Comando::SelecionarFaixa(ids[1])).unwrap();
    assert_eq!(core.selecao(), Some(ids[1]));
    assert_eq!(core.drenar_eventos(), vec![Evento::SelecaoAlterada(ids[1])]);
    let erro = core.executar(Comando::SelecionarFaixa(TrackId(999)));
    assert!(matches!(erro, Err(ErroCore::FaixaInexistente(999))));
    assert_eq!(core.selecao(), Some(ids[1]));
}

#[test]
fn playlist_reordena_com_ordem_estavel() {
    let (mut core, ids) = core_com_faixas("lista");
    core.executar(Comando::CriarLista {
        nome: "Set".into(),
        pai: None,
        pasta: false,
    })
    .unwrap();
    let lista = core.listas().unwrap()[0].id;
    for f in [ids[0], ids[1], ids[0]] {
        core.executar(Comando::AdicionarALista { lista, faixa: f })
            .unwrap();
    }
    core.executar(Comando::MoverNaLista {
        lista,
        de: 2,
        para: 0,
    })
    .unwrap();
    assert_eq!(
        core.faixas_da_lista(lista).unwrap(),
        vec![ids[0], ids[0], ids[1]]
    );
    core.executar(Comando::MoverNaLista {
        lista,
        de: 2,
        para: 1,
    })
    .unwrap();
    assert_eq!(
        core.faixas_da_lista(lista).unwrap(),
        vec![ids[0], ids[1], ids[0]]
    );
    let erro = core.executar(Comando::MoverNaLista {
        lista,
        de: 3,
        para: 0,
    });
    assert!(matches!(erro, Err(ErroCore::PosicaoInvalida(_))));
    assert_eq!(core.faixas_da_lista(lista).unwrap().len(), 3);
}

#[test]
fn deck_vazio_e_faixa_inexistente_dao_erro_tipado() {
    let (mut core, _) = core_com_faixas("vazio");
    let erro = core.executar(Comando::CriarCue {
        deck: Deck::Dois,
        cue: memory(10),
    });
    assert!(matches!(erro, Err(ErroCore::DeckVazio(Deck::Dois))));
    let erro = core.executar(Comando::CarregarNoDeck {
        deck: Deck::Um,
        faixa: TrackId(42),
    });
    assert!(matches!(erro, Err(ErroCore::FaixaInexistente(42))));
    assert!(core.deck(Deck::Um).is_none());
}

#[test]
fn fluxo_importar_carregar_editar_cue_em_ordem() {
    let (mut core, ids) = core_com_faixas("fluxo");
    core.executar(Comando::CarregarNoDeck {
        deck: Deck::Um,
        faixa: ids[0],
    })
    .unwrap();
    core.executar(Comando::CriarCue {
        deck: Deck::Um,
        cue: memory(4800),
    })
    .unwrap();
    core.executar(Comando::CriarCue {
        deck: Deck::Um,
        cue: memory(960),
    })
    .unwrap();
    let ev = core.drenar_eventos();
    let tipos: Vec<_> = ev
        .iter()
        .map(|e| match e {
            Evento::BibliotecaAlterada { .. } => "biblioteca",
            Evento::TarefaAnaliseAtualizada { .. } => "analise",
            Evento::FaixaCarregada { .. } => "carregada",
            Evento::PreparacaoAlterada { .. } => "preparacao",
            Evento::SelecaoAlterada(_) => "selecao",
            Evento::IdentidadeAtualizada(_) => "identidade",
        })
        .collect();
    assert_eq!(
        tipos,
        [
            "biblioteca",
            "analise",
            "carregada",
            "preparacao",
            "preparacao"
        ]
    );

    // Ordem por posição; origem forçada para User.
    let d = core.deck(Deck::Um).unwrap();
    let pos: Vec<i64> = d.cues.iter().map(|c| c.position.get()).collect();
    assert_eq!(pos, [960, 4800]);
    assert!(d.cues.iter().all(|c| c.origin == CueOrigin::User));

    core.executar(Comando::MoverCue {
        deck: Deck::Um,
        indice: 0,
        posicao: Frame::new(9600),
    })
    .unwrap();
    let pos: Vec<i64> = core
        .deck(Deck::Um)
        .unwrap()
        .cues
        .iter()
        .map(|c| c.position.get())
        .collect();
    assert_eq!(pos, [4800, 9600]);

    let erro = core.executar(Comando::RemoverCue {
        deck: Deck::Um,
        indice: 5,
    });
    assert!(matches!(erro, Err(ErroCore::CueInexistente(5))));
    core.executar(Comando::RemoverCue {
        deck: Deck::Um,
        indice: 0,
    })
    .unwrap();

    // Recarregar do banco devolve o que ficou persistido.
    core.executar(Comando::CarregarNoDeck {
        deck: Deck::Dois,
        faixa: ids[0],
    })
    .unwrap();
    let d2 = core.deck(Deck::Dois).unwrap();
    assert_eq!(d2.cues.len(), 1);
    assert_eq!(d2.cues[0].position.get(), 9600);
}

#[test]
fn mover_loop_preserva_comprimento() {
    let (mut core, ids) = core_com_faixas("loop");
    core.executar(Comando::CarregarNoDeck {
        deck: Deck::Um,
        faixa: ids[0],
    })
    .unwrap();
    let mut lp = Cue::new(CueKind::Loop, Frame::new(100), CueOrigin::User);
    lp.end_position = Some(Frame::new(300));
    core.executar(Comando::CriarCue {
        deck: Deck::Um,
        cue: lp,
    })
    .unwrap();
    core.executar(Comando::MoverCue {
        deck: Deck::Um,
        indice: 0,
        posicao: Frame::new(1000),
    })
    .unwrap();
    let c = &core.deck(Deck::Um).unwrap().cues[0];
    assert_eq!(c.end_position, Some(Frame::new(1200)));
}

#[test]
fn grade_salva_e_recarrega_com_bloqueio() {
    let (mut core, ids) = core_com_faixas("grade");
    core.executar(Comando::CarregarNoDeck {
        deck: Deck::Um,
        faixa: ids[1],
    })
    .unwrap();
    let taxa = SampleRate::new(44100).unwrap();
    let grade = BeatGrid::new(
        taxa,
        (0..4)
            .map(|i| Beat {
                position: Frame::new(i * 22050),
                beat_num: (i % 4 + 1) as u32,
            })
            .collect(),
        vec![TempoChange {
            position: Frame::ZERO,
            bpm: Bpm::new(120.0).unwrap(),
        }],
        Some(Frame::ZERO),
    )
    .unwrap();
    core.executar(Comando::SalvarGrade {
        deck: Deck::Um,
        grade: grade.clone(),
        bloqueada: true,
    })
    .unwrap();
    core.executar(Comando::CarregarNoDeck {
        deck: Deck::Dois,
        faixa: ids[1],
    })
    .unwrap();
    let d = core.deck(Deck::Dois).unwrap();
    assert_eq!(d.grade.as_ref(), Some(&grade));
    assert!(d.bloqueada);
}

#[test]
fn registrar_faixa_audio_e_substituir_cues() {
    let mut core = Core::novo(Database::open_in_memory().unwrap());
    let caminho = "C:/musica/a.mp3".to_string();
    for _ in 0..2 {
        core.executar(Comando::RegistrarFaixa {
            caminho: caminho.clone(),
            titulo: Some("A".into()),
            artista: None,
        })
        .unwrap();
    }
    // Segunda chamada não duplica.
    assert_eq!(core.pesquisar(&Consulta::default()).unwrap().len(), 1);
    let id = core.faixa_por_caminho(&caminho).unwrap().unwrap();
    assert_eq!(core.taxa(id).unwrap(), None);
    core.executar(Comando::RegistrarAudio {
        faixa: id,
        taxa: 44100,
        canais: 2,
        duracao_frames: 441_000,
    })
    .unwrap();
    assert_eq!(core.taxa(id).unwrap(), Some(44100));
    core.executar(Comando::CarregarNoDeck {
        deck: Deck::Um,
        faixa: id,
    })
    .unwrap();
    let mut hot = Cue::new(CueKind::HotCue, Frame::new(100), CueOrigin::Imported);
    hot.slot = Some(2);
    let lp = Cue::new_loop(Frame::new(10), Frame::new(20), CueOrigin::User).unwrap();
    core.executar(Comando::SubstituirCues {
        deck: Deck::Um,
        cues: vec![hot, lp, memory(50)],
    })
    .unwrap();
    // Loop fica de fora; o resto entra como User.
    let cues = &core.deck(Deck::Um).unwrap().cues;
    assert_eq!(cues.len(), 2);
    assert!(cues
        .iter()
        .all(|c| c.origin == CueOrigin::User && c.kind != CueKind::Loop));
    assert_eq!(cues[1].slot, Some(2));
    let ev = core.drenar_eventos();
    assert!(ev.contains(&Evento::IdentidadeAtualizada(id)));
}
