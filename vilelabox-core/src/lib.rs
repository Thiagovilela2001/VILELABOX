//! `vilelabox-core`: núcleo headless do VILELABOX (P3.3).
//!
//! ADAPTAÇÃO (desenho próprio, `docs/PLANO-CORE-HEADLESS.md`): toda mutação
//! passa por [`Core::executar`]; consultas são leituras sem efeito colateral.
//! Cada comando emite eventos numa fila ordenada, drenada por
//! [`Core::drenar_eventos`]. O núcleo não conhece interface nem motor de
//! áudio e só toca o filesystem nos comandos de importação e verificação.

use std::path::{Path, PathBuf};
use vilelabox_library::{Consulta, ErroLibrary};
use vilelabox_storage::NovaFaixa;
use vilelabox_storage::{Database, ErroStorage, FaixaRow, ListaRow};
use vilelabox_types::{
    AudioIdentity, BeatGrid, Cue, CueKind, CueOrigin, Frame, SampleRate, TrackId, TrackMetadata,
};

/// Deck headless.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Deck {
    Um,
    Dois,
}

impl Deck {
    fn idx(self) -> usize {
        match self {
            Deck::Um => 0,
            Deck::Dois => 1,
        }
    }
}

/// Faixa carregada no deck com a preparação vinda do banco.
#[derive(Debug, Clone, PartialEq)]
pub struct DeckState {
    pub faixa: FaixaRow,
    /// Ordem de posição (a mesma do banco); comandos de cue usam este índice.
    pub cues: Vec<Cue>,
    pub grade: Option<BeatGrid>,
    pub bloqueada: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Comando {
    ImportarDiretorio(PathBuf),
    ImportarCatalogo(PathBuf),
    ImportarXml(PathBuf),
    VerificarLocalizacoes,
    BuscarMovidos(PathBuf),
    SelecionarFaixa(TrackId),
    CriarLista {
        nome: String,
        pai: Option<i64>,
        pasta: bool,
    },
    AdicionarALista {
        lista: i64,
        faixa: TrackId,
    },
    MoverNaLista {
        lista: i64,
        de: usize,
        para: usize,
    },
    CarregarNoDeck {
        deck: Deck,
        faixa: TrackId,
    },
    /// O cue entra sempre com origem `User`.
    CriarCue {
        deck: Deck,
        cue: Cue,
    },
    MoverCue {
        deck: Deck,
        indice: usize,
        posicao: Frame,
    },
    RemoverCue {
        deck: Deck,
        indice: usize,
    },
    SalvarGrade {
        deck: Deck,
        grade: BeatGrid,
        bloqueada: bool,
    },
    /// Cadastra a faixa pelo caminho do áudio, se ainda não existir.
    RegistrarFaixa {
        caminho: String,
        titulo: Option<String>,
        artista: Option<String>,
    },
    /// Grava taxa, canais e duração lidos do áudio decodificado, preservando
    /// o SHA-256 já calculado.
    RegistrarAudio {
        faixa: TrackId,
        taxa: u32,
        canais: u16,
        duracao_frames: i64,
    },
    /// Substitui os cues do usuário (exceto loops) da faixa no deck; os de
    /// outras origens ficam. Todos entram com origem `User`.
    SubstituirCues {
        deck: Deck,
        cues: Vec<Cue>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Evento {
    /// `resumo` descreve o que mudou (relatório do importador, nova lista...).
    BibliotecaAlterada {
        resumo: String,
    },
    SelecaoAlterada(TrackId),
    FaixaCarregada {
        deck: Deck,
        faixa: TrackId,
    },
    PreparacaoAlterada {
        deck: Deck,
        faixa: TrackId,
    },
    /// Faixas ainda sem identidade acústica, à espera da análise (P4).
    TarefaAnaliseAtualizada {
        pendentes: usize,
    },
    /// Taxa, canais e duração da faixa gravados.
    IdentidadeAtualizada(TrackId),
}

#[derive(Debug, thiserror::Error)]
pub enum ErroCore {
    #[error("faixa inexistente: {0}")]
    FaixaInexistente(i64),
    #[error("deck {0:?} vazio")]
    DeckVazio(Deck),
    #[error("cue inexistente no índice {0}")]
    CueInexistente(usize),
    #[error("posição fora da playlist {0}")]
    PosicaoInvalida(i64),
    #[error("erro de E/S: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Storage(#[from] ErroStorage),
    #[error(transparent)]
    Library(#[from] ErroLibrary),
    #[error("erro no catálogo: {0}")]
    Catalogo(#[from] vilelabox_catalog::ErroCatalogo),
    #[error("erro no XML: {0}")]
    Xml(#[from] vilelabox_xml::XmlError),
}

pub struct Core {
    db: Database,
    decks: [Option<DeckState>; 2],
    selecao: Option<TrackId>,
    eventos: Vec<Evento>,
}

impl Core {
    pub fn novo(db: Database) -> Self {
        Self {
            db,
            decks: [None, None],
            selecao: None,
            eventos: Vec::new(),
        }
    }

    pub fn abrir(caminho: &Path) -> Result<Self, ErroCore> {
        Ok(Self::novo(Database::open(caminho)?))
    }

    // --- Consultas ---

    pub fn pesquisar(&self, consulta: &Consulta) -> Result<Vec<FaixaRow>, ErroCore> {
        Ok(vilelabox_library::buscar(&self.db, consulta)?)
    }

    pub fn listas(&self) -> Result<Vec<ListaRow>, ErroCore> {
        Ok(self.db.listas()?)
    }

    pub fn faixas_da_lista(&self, lista: i64) -> Result<Vec<TrackId>, ErroCore> {
        Ok(self.db.faixas_da_lista(lista)?)
    }

    /// Faixa dona do caminho de áudio, se cadastrada.
    pub fn faixa_por_caminho(&self, caminho: &str) -> Result<Option<TrackId>, ErroCore> {
        Ok(self.db.id_por_localizacao(caminho)?)
    }

    /// Taxa de amostragem conhecida da faixa (gravada por `RegistrarAudio`).
    pub fn taxa(&self, faixa: TrackId) -> Result<Option<u32>, ErroCore> {
        Ok(self
            .db
            .faixa(faixa)?
            .and_then(|f| f.identidade)
            .map(|i| i.sample_rate.get()))
    }

    pub fn selecao(&self) -> Option<TrackId> {
        self.selecao
    }

    pub fn deck(&self, deck: Deck) -> Option<&DeckState> {
        self.decks[deck.idx()].as_ref()
    }

    /// Devolve e esvazia a fila, na ordem exata de emissão.
    pub fn drenar_eventos(&mut self) -> Vec<Evento> {
        std::mem::take(&mut self.eventos)
    }

    // --- Mutação ---

    pub fn executar(&mut self, comando: Comando) -> Result<(), ErroCore> {
        match comando {
            Comando::ImportarDiretorio(dir) => {
                let rel = vilelabox_library::varrer_diretorio(&self.db, &dir)?;
                self.importou(format!("{rel:?}"))
            }
            Comando::ImportarCatalogo(arq) => {
                let cat = vilelabox_catalog::abrir(&arq)?;
                let rel = vilelabox_library::importar_catalogo(&self.db, &cat)?;
                self.importou(format!("{rel:?}"))
            }
            Comando::ImportarXml(arq) => {
                let doc = vilelabox_xml::parse(&std::fs::read_to_string(&arq)?)?;
                let rel = vilelabox_library::importar_xml(&self.db, &doc)?;
                self.importou(format!("{rel:?}"))
            }
            Comando::VerificarLocalizacoes => {
                let (presentes, ausentes) = vilelabox_library::verificar_localizacoes(&self.db)?;
                self.biblioteca(format!("presentes={presentes} ausentes={ausentes}"));
                Ok(())
            }
            Comando::BuscarMovidos(dir) => {
                let achados = vilelabox_library::buscar_movidos(&self.db, &dir)?;
                self.biblioteca(format!("movidos={}", achados.len()));
                Ok(())
            }
            Comando::SelecionarFaixa(id) => {
                self.faixa(id)?;
                self.selecao = Some(id);
                self.eventos.push(Evento::SelecaoAlterada(id));
                Ok(())
            }
            Comando::CriarLista { nome, pai, pasta } => {
                let id = self.db.criar_lista(&nome, pai, pasta)?;
                self.biblioteca(format!("lista criada id={id}"));
                Ok(())
            }
            Comando::AdicionarALista { lista, faixa } => {
                self.faixa(faixa)?;
                self.db.adicionar_a_lista(lista, faixa)?;
                self.biblioteca(format!("faixa {} na lista {lista}", faixa.0));
                Ok(())
            }
            Comando::MoverNaLista { lista, de, para } => {
                if !self.db.mover_na_lista(lista, de, para)? {
                    return Err(ErroCore::PosicaoInvalida(lista));
                }
                self.biblioteca(format!("lista {lista}: {de} -> {para}"));
                Ok(())
            }
            Comando::CarregarNoDeck { deck, faixa } => {
                let row = self.faixa(faixa)?;
                let (grade, bloqueada) = match self.db.grade(faixa)? {
                    Some(g) => (Some(g.grade), g.bloqueada),
                    None => (None, false),
                };
                self.decks[deck.idx()] = Some(DeckState {
                    faixa: row,
                    cues: self.db.cues(faixa)?,
                    grade,
                    bloqueada,
                });
                self.eventos.push(Evento::FaixaCarregada { deck, faixa });
                Ok(())
            }
            Comando::CriarCue { deck, cue } => self.editar_cues(deck, |cues| {
                cues.push(Cue {
                    id: None,
                    origin: CueOrigin::User,
                    ..cue
                });
                Ok(())
            }),
            Comando::MoverCue {
                deck,
                indice,
                posicao,
            } => self.editar_cues(deck, |cues| {
                let c = cues
                    .get_mut(indice)
                    .ok_or(ErroCore::CueInexistente(indice))?;
                if let Some(fim) = c.end_position {
                    // Loop anda inteiro: mantém o comprimento.
                    c.end_position = Some(Frame::new(fim.get() - c.position.get() + posicao.get()));
                }
                c.position = posicao;
                Ok(())
            }),
            Comando::RemoverCue { deck, indice } => self.editar_cues(deck, |cues| {
                if indice >= cues.len() {
                    return Err(ErroCore::CueInexistente(indice));
                }
                cues.remove(indice);
                Ok(())
            }),
            Comando::RegistrarFaixa {
                caminho,
                titulo,
                artista,
            } => {
                if self.db.id_por_localizacao(&caminho)?.is_some() {
                    return Ok(());
                }
                let id = self.db.inserir_faixa(&NovaFaixa {
                    metadados: TrackMetadata {
                        title: titulo,
                        artist: artista,
                        ..TrackMetadata::default()
                    },
                    ..NovaFaixa::default()
                })?;
                self.db.adicionar_localizacao(id, &caminho)?;
                self.biblioteca(format!("faixa {} registrada", id.0));
                Ok(())
            }
            Comando::RegistrarAudio {
                faixa,
                taxa,
                canais,
                duracao_frames,
            } => {
                let atual = self.faixa(faixa)?;
                let identidade = AudioIdentity {
                    sha256: atual.identidade.and_then(|i| i.sha256),
                    duration_frames: Frame::new(duracao_frames),
                    sample_rate: SampleRate::new(taxa).map_err(ErroStorage::from)?,
                    channels: canais,
                };
                self.db.atualizar_identidade(faixa, &identidade)?;
                self.eventos.push(Evento::IdentidadeAtualizada(faixa));
                Ok(())
            }
            Comando::SubstituirCues { deck, cues } => {
                let estado = self.decks[deck.idx()]
                    .as_mut()
                    .ok_or(ErroCore::DeckVazio(deck))?;
                let faixa = estado.faixa.id;
                let proprios: Vec<Cue> = cues
                    .into_iter()
                    .filter(|c| c.kind != CueKind::Loop)
                    .collect();
                self.db
                    .substituir_cues_da_origem(faixa, CueOrigin::User, &proprios)?;
                estado.cues = self.db.cues(faixa)?;
                self.eventos
                    .push(Evento::PreparacaoAlterada { deck, faixa });
                Ok(())
            }
            Comando::SalvarGrade {
                deck,
                grade,
                bloqueada,
            } => {
                let estado = self.decks[deck.idx()]
                    .as_mut()
                    .ok_or(ErroCore::DeckVazio(deck))?;
                let faixa = estado.faixa.id;
                self.db.salvar_grade(faixa, &grade, bloqueada)?;
                estado.grade = Some(grade);
                estado.bloqueada = bloqueada;
                self.eventos
                    .push(Evento::PreparacaoAlterada { deck, faixa });
                Ok(())
            }
        }
    }

    fn faixa(&self, id: TrackId) -> Result<FaixaRow, ErroCore> {
        self.db.faixa(id)?.ok_or(ErroCore::FaixaInexistente(id.0))
    }

    fn biblioteca(&mut self, resumo: String) {
        self.eventos.push(Evento::BibliotecaAlterada { resumo });
    }

    fn importou(&mut self, resumo: String) -> Result<(), ErroCore> {
        self.biblioteca(resumo);
        let pendentes = self
            .db
            .listar_faixas(None)?
            .iter()
            .filter(|f| f.identidade.is_none())
            .count();
        self.eventos
            .push(Evento::TarefaAnaliseAtualizada { pendentes });
        Ok(())
    }

    /// Aplica `f` a uma cópia dos cues do deck, persiste no banco e recarrega
    /// (ids e ordem passam a ser os do banco). Erro em `f` não altera nada.
    fn editar_cues(
        &mut self,
        deck: Deck,
        f: impl FnOnce(&mut Vec<Cue>) -> Result<(), ErroCore>,
    ) -> Result<(), ErroCore> {
        let estado = self.decks[deck.idx()]
            .as_mut()
            .ok_or(ErroCore::DeckVazio(deck))?;
        let faixa = estado.faixa.id;
        let mut cues = estado.cues.clone();
        f(&mut cues)?;
        self.db.substituir_cues(faixa, &cues)?;
        estado.cues = self.db.cues(faixa)?;
        self.eventos
            .push(Evento::PreparacaoAlterada { deck, faixa });
        Ok(())
    }
}
