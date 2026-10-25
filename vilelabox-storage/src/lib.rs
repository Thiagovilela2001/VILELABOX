//! `vilelabox-storage`: banco SQLite próprio do VILELABOX (P3.1).
//!
//! ADAPTAÇÃO documentada em `spec/rekordbox/10-master-db.md`: o esquema é
//! próprio — sem a máquina de sincronização `rb_*` do catálogo de referência —
//! e guarda os mesmos conceitos de domínio: faixas, localizações, metadados,
//! playlists, cues, grade, frases e resultados de análise. Migrações são
//! versionadas por `PRAGMA user_version`, aplicadas uma por transação e
//! precedidas de cópia de backup do arquivo.
//!
//! Nada aqui escreve em catálogos de terceiros: o único arquivo tocado é o
//! informado em [`Database::open`] (e seu backup).

mod analysis;
mod cues;
mod grids;
mod phrases;
mod playlists;
mod tracks;

pub mod error;
pub mod migrations;

pub use error::ErroStorage;
pub use grids::GradeRow;
pub use playlists::ListaRow;
pub use tracks::{FaixaRow, LocalizacaoCompleta, LocalizacaoRow, NovaFaixa};

use rusqlite::Connection;
use std::path::{Path, PathBuf};
use vilelabox_types::{AudioIdentity, BeatGrid, Cue, CueOrigin, Phrase, TrackId};

/// Banco próprio aberto, já migrado e com chaves estrangeiras ligadas.
pub struct Database {
    con: Connection,
}

impl Database {
    /// Abre (ou cria) o banco em `caminho` e aplica as migrações pendentes.
    /// Um arquivo preexistente é copiado para `<nome>.backup-v<N>.db` antes de
    /// qualquer migração, onde `<N>` é a versão encontrada.
    pub fn open(caminho: &Path) -> Result<Self, ErroStorage> {
        Self::abrir_com(caminho, migrations::MIGRACOES)
    }

    /// Banco volátil em memória (desenvolvimento e testes).
    pub fn open_in_memory() -> Result<Self, ErroStorage> {
        let con = Connection::open_in_memory()?;
        con.pragma_update(None, "foreign_keys", "ON")?;
        migrations::migrar(&con, migrations::MIGRACOES)?;
        Ok(Self { con })
    }

    /// Versão do esquema gravada no banco.
    pub fn versao(&self) -> Result<u32, ErroStorage> {
        migrations::versao_atual(&self.con)
    }

    /// Variante de [`Database::open`] com lista explícita de migrações;
    /// existe para testar upgrade, rollback, backup e interrupção.
    #[doc(hidden)]
    pub fn abrir_com(caminho: &Path, migracoes: &[(u32, &str)]) -> Result<Self, ErroStorage> {
        let preexistente = std::fs::metadata(caminho)
            .map(|m| m.len() > 0)
            .unwrap_or(false);
        let mut con = Connection::open(caminho)?;
        con.pragma_update(None, "foreign_keys", "ON")?;
        let versao = migrations::versao_atual(&con)?;
        let pendente = migracoes.iter().any(|(v, _)| *v > versao);
        if preexistente && pendente {
            con.close().map_err(|(_, e)| ErroStorage::Sqlite(e))?;
            backup(caminho, versao)?;
            con = Connection::open(caminho)?;
            con.pragma_update(None, "foreign_keys", "ON")?;
        }
        migrations::migrar(&con, migracoes)?;
        Ok(Self { con })
    }

    /// Acesso à conexão para administração e testes de corrupção.
    #[doc(hidden)]
    pub fn con(&self) -> &Connection {
        &self.con
    }

    // --- Faixas e localizações ---

    pub fn inserir_faixa(&self, faixa: &NovaFaixa) -> Result<TrackId, ErroStorage> {
        tracks::inserir(&self.con, faixa)
    }

    pub fn faixa(&self, id: TrackId) -> Result<Option<FaixaRow>, ErroStorage> {
        tracks::obter(&self.con, id)
    }

    /// Remove a faixa e, em cascata, cues, grade, frases, localizações e
    /// vínculos de playlist. Retorna se a faixa existia.
    pub fn remover_faixa(&self, id: TrackId) -> Result<bool, ErroStorage> {
        tracks::remover(&self.con, id)
    }

    pub fn adicionar_localizacao(&self, faixa: TrackId, caminho: &str) -> Result<i64, ErroStorage> {
        tracks::adicionar_localizacao(&self.con, faixa, caminho)
    }

    pub fn localizacoes(&self, faixa: TrackId) -> Result<Vec<LocalizacaoRow>, ErroStorage> {
        tracks::localizacoes(&self.con, faixa)
    }

    /// Todas as faixas do banco (`None`) ou as de uma playlist na ordem dela.
    pub fn listar_faixas(&self, lista: Option<i64>) -> Result<Vec<FaixaRow>, ErroStorage> {
        tracks::listar(&self.con, lista)
    }

    /// Atualiza metadados, BPM, tom e estrelas (não toca a identidade).
    pub fn atualizar_faixa(&self, id: TrackId, faixa: &NovaFaixa) -> Result<bool, ErroStorage> {
        tracks::atualizar(&self.con, id, faixa)
    }

    /// Grava a identidade acústica (hash, duração, taxa, canais).
    pub fn atualizar_identidade(
        &self,
        id: TrackId,
        identidade: &AudioIdentity,
    ) -> Result<bool, ErroStorage> {
        tracks::atualizar_identidade(&self.con, id, identidade)
    }

    /// Faixa dona de uma localização pelo caminho exato.
    pub fn id_por_localizacao(&self, caminho: &str) -> Result<Option<TrackId>, ErroStorage> {
        tracks::id_por_localizacao(&self.con, caminho)
    }

    /// Todas as localizações com as faixas donas.
    pub fn listar_localizacoes(&self) -> Result<Vec<LocalizacaoCompleta>, ErroStorage> {
        tracks::listar_localizacoes(&self.con)
    }

    /// Marca uma localização como ausente ou presente.
    pub fn marcar_localizacao(&self, id: i64, ausente: bool) -> Result<(), ErroStorage> {
        tracks::marcar_localizacao(&self.con, id, ausente)
    }

    // --- Cues ---

    /// Substitui atomicamente o conjunto de cues da faixa.
    pub fn substituir_cues(&self, faixa: TrackId, cues: &[Cue]) -> Result<(), ErroStorage> {
        cues::substituir(&self.con, faixa, cues)
    }

    pub fn cues(&self, faixa: TrackId) -> Result<Vec<Cue>, ErroStorage> {
        cues::listar(&self.con, faixa)
    }

    /// Substitui só os cues da origem dada, preservando os das demais.
    pub fn substituir_cues_da_origem(
        &self,
        faixa: TrackId,
        origem: CueOrigin,
        cues: &[Cue],
    ) -> Result<(), ErroStorage> {
        cues::substituir_por_origem(&self.con, faixa, origem, cues)
    }

    // --- Grade de batidas ---

    pub fn salvar_grade(
        &self,
        faixa: TrackId,
        grade: &BeatGrid,
        bloqueada: bool,
    ) -> Result<(), ErroStorage> {
        grids::salvar(&self.con, faixa, grade, bloqueada)
    }

    pub fn grade(&self, faixa: TrackId) -> Result<Option<GradeRow>, ErroStorage> {
        grids::obter(&self.con, faixa)
    }

    // --- Frases ---

    pub fn substituir_frases(&self, faixa: TrackId, frases: &[Phrase]) -> Result<(), ErroStorage> {
        phrases::substituir(&self.con, faixa, frases)
    }

    pub fn frases(&self, faixa: TrackId) -> Result<Vec<Phrase>, ErroStorage> {
        phrases::listar(&self.con, faixa)
    }

    // --- Playlists ---

    /// Cria pasta ou playlist como filha de `pai` (`None` = raiz).
    pub fn criar_lista(
        &self,
        nome: &str,
        pai: Option<i64>,
        pasta: bool,
    ) -> Result<i64, ErroStorage> {
        playlists::criar(&self.con, nome, pai, pasta)
    }

    pub fn adicionar_a_lista(&self, lista: i64, faixa: TrackId) -> Result<(), ErroStorage> {
        playlists::adicionar_faixa(&self.con, lista, faixa)
    }

    pub fn faixas_da_lista(&self, lista: i64) -> Result<Vec<TrackId>, ErroStorage> {
        playlists::faixas(&self.con, lista)
    }

    pub fn listas(&self) -> Result<Vec<ListaRow>, ErroStorage> {
        playlists::listar(&self.con)
    }

    // --- Resultados de análise ---

    pub fn salvar_analise(
        &self,
        faixa: TrackId,
        etapa: &str,
        versao: u32,
        config: &str,
        dados: &[u8],
    ) -> Result<(), ErroStorage> {
        analysis::salvar(&self.con, faixa, etapa, versao, config, dados)
    }

    pub fn analise(
        &self,
        faixa: TrackId,
        etapa: &str,
        versao: u32,
        config: &str,
    ) -> Result<Option<Vec<u8>>, ErroStorage> {
        analysis::obter(&self.con, faixa, etapa, versao, config)
    }
}

/// Copia o arquivo do banco para `<nome>.backup-v<versao>.db` ao lado dele.
fn backup(caminho: &Path, versao: u32) -> Result<PathBuf, ErroStorage> {
    let nome = caminho
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("vilelabox.db");
    let destino = caminho.with_file_name(format!("{nome}.backup-v{versao}"));
    std::fs::copy(caminho, &destino)?;
    Ok(destino)
}
