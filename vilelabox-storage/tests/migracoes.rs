//! Testes de upgrade, rollback, backup e recuperação de interrupção das
//! migrações do banco próprio (P3.1). Tudo em pasta temporária.

use rusqlite::Connection;
use std::path::PathBuf;
use vilelabox_storage::migrations::{migrar, versao_atual, VERSAO_ESQUEMA};
use vilelabox_storage::{Database, ErroStorage};

fn pasta(nome: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("vilelabox-storage-{nome}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

const V1_A: &str = "CREATE TABLE a (x INTEGER); INSERT INTO a VALUES (7);";
const V2_A: &str = "ALTER TABLE a ADD COLUMN y INTEGER;";

#[test]
fn banco_novo_cria_esquema_atual() {
    let dir = pasta("novo");
    let db = Database::open(&dir.join("vilelabox.db")).unwrap();
    assert_eq!(db.versao().unwrap(), VERSAO_ESQUEMA);
}

#[test]
fn reabrir_e_idempotente_e_preserva_dados() {
    let dir = pasta("reabrir");
    let p = dir.join("vilelabox.db");
    {
        let db = Database::open(&p).unwrap();
        assert_eq!(db.versao().unwrap(), VERSAO_ESQUEMA);
    }
    // Segunda abertura: nenhuma migração pendente, nenhum backup criado.
    let db = Database::open(&p).unwrap();
    assert_eq!(db.versao().unwrap(), VERSAO_ESQUEMA);
    assert!(!dir.join("vilelabox.db.backup-v1").exists());
}

#[test]
fn upgrade_aplica_versoes_em_ordem() {
    let con = Connection::open_in_memory().unwrap();
    assert_eq!(migrar(&con, &[(1, V1_A)]).unwrap(), 1);
    assert_eq!(migrar(&con, &[(1, V1_A), (2, V2_A)]).unwrap(), 2);
    // Coluna da v2 existe e o dado da v1 sobreviveu.
    con.execute("INSERT INTO a (x, y) VALUES (8, 9)", [])
        .unwrap();
    let soma: i64 = con
        .query_row("SELECT sum(x) FROM a", [], |r| r.get(0))
        .unwrap();
    assert_eq!(soma, 15);
}

#[test]
fn migracao_com_erro_faz_rollback_completo() {
    let con = Connection::open_in_memory().unwrap();
    migrar(&con, &[(1, V1_A)]).unwrap();
    // v2 quebra no meio: a tabela `a` já existe.
    let v2_ruim = "ALTER TABLE a ADD COLUMN y INTEGER; CREATE TABLE a (z INTEGER);";
    assert!(migrar(&con, &[(1, V1_A), (2, v2_ruim)]).is_err());
    // Banco permanece na v1 e o ALTER parcial foi desfeito.
    assert_eq!(versao_atual(&con).unwrap(), 1);
    let colunas: Vec<String> = con
        .prepare("SELECT name FROM pragma_table_info('a')")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(colunas, ["x"]);
}

#[test]
fn lacuna_de_versao_e_rejeitada() {
    let con = Connection::open_in_memory().unwrap();
    match migrar(&con, &[(2, V1_A)]) {
        Err(ErroStorage::Migracao(msg)) => assert!(msg.contains("lacuna")),
        outro => panic!("esperava ErroStorage::Migracao, veio {outro:?}"),
    }
}

#[test]
fn abertura_de_banco_antigo_faz_backup_e_migra() {
    let dir = pasta("backup");
    let p = dir.join("vilelabox.db");
    {
        let _ = Database::abrir_com(&p, &[(1, V1_A)]).unwrap();
    }
    // Reabre com a v2 disponível: backup da v1 antes de migrar.
    let db = Database::abrir_com(&p, &[(1, V1_A), (2, V2_A)]).unwrap();
    assert_eq!(db.versao().unwrap(), 2);
    let bkp = dir.join("vilelabox.db.backup-v1");
    assert!(bkp.exists(), "backup não criado em {}", bkp.display());
    // O backup contém o estado anterior, abrível e íntegro.
    let con = Connection::open(&bkp).unwrap();
    assert_eq!(versao_atual(&con).unwrap(), 1);
    let x: i64 = con.query_row("SELECT x FROM a", [], |r| r.get(0)).unwrap();
    assert_eq!(x, 7);
}

#[test]
fn interrompida_continua_da_versao_parcial() {
    let dir = pasta("interrompida");
    let p = dir.join("vilelabox.db");
    {
        // Processo "morre" com só a v1 aplicada.
        let _ = Database::abrir_com(&p, &[(1, V1_A)]).unwrap();
    }
    let db = Database::abrir_com(&p, &[(1, V1_A), (2, V2_A)]).unwrap();
    assert_eq!(db.versao().unwrap(), 2);
    let x: i64 = db
        .con()
        .query_row("SELECT x FROM a", [], |r| r.get(0))
        .unwrap();
    assert_eq!(x, 7);
}

#[test]
fn arquivo_invalido_da_erro_tipado() {
    let dir = pasta("invalido");
    let p = dir.join("vilelabox.db");
    std::fs::write(&p, b"nao e um banco sqlite").unwrap();
    assert!(matches!(Database::open(&p), Err(ErroStorage::Sqlite(_))));
}

#[test]
fn abertura_em_caminho_sem_permissao_da_erro() {
    let dir = pasta("semcaminho");
    let p = dir.join("nao-existe").join("vilelabox.db");
    assert!(Database::open(&p).is_err());
}
