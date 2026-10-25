//! Identidade por conteúdo, ausentes, movidos e duplicados.

use crate::ErroLibrary;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::io::Read;
use std::path::Path;
use vilelabox_storage::Database;
use vilelabox_types::TrackId;

/// SHA-256 do arquivo em hex minúsculas, lido em blocos de 64 KiB (mesmo
/// fluxo do exportador ANLZ).
pub fn sha256_arquivo(caminho: &Path) -> Result<String, ErroLibrary> {
    let mut f = std::fs::File::open(caminho)?;
    let mut h = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(format!("{:x}", h.finalize()))
}

/// Nome normalizado para casar arquivos movidos: exige extensão de áudio;
/// minúsculas, sem extensão e sem numeração inicial ("01 - ", "01. ", "01 ").
/// `None` para nomes sem conteúdo útil (ex.: imagens, só dígitos).
pub fn nome_normalizado(nome: &str) -> Option<String> {
    let p = Path::new(nome);
    let ext = p.extension()?.to_str()?.to_lowercase();
    if !crate::scan::EXTENSOES.contains(&ext.as_str()) {
        return None;
    }
    let stem = p.file_stem()?.to_str()?.trim().to_lowercase();
    let limpo = stem
        .trim_start_matches(|c: char| c.is_ascii_digit())
        .trim_start_matches(['.', ' ', '-'])
        .trim_start();
    (!limpo.is_empty()).then(|| limpo.to_string())
}

/// Confere cada localização no disco e atualiza `missing`.
/// Retorna (presentes, ausentes).
pub fn verificar_localizacoes(db: &Database) -> Result<(usize, usize), ErroLibrary> {
    let mut presentes = 0;
    let mut ausentes = 0;
    for loc in db.listar_localizacoes()? {
        let existe = Path::new(&loc.caminho).is_file();
        db.marcar_localizacao(loc.id, !existe)?;
        if existe {
            presentes += 1;
        } else {
            ausentes += 1;
        }
    }
    Ok((presentes, ausentes))
}

/// Faixas com o mesmo SHA-256 (identidade já calculada), agrupadas pelo hash.
pub fn duplicatas(db: &Database) -> Result<Vec<(String, Vec<TrackId>)>, ErroLibrary> {
    let mut por_hash: HashMap<String, Vec<TrackId>> = HashMap::new();
    for f in db.listar_faixas(None)? {
        if let Some(sha) = f.identidade.and_then(|i| i.sha256) {
            por_hash.entry(sha).or_default().push(f.id);
        }
    }
    let mut grupos: Vec<_> = por_hash
        .into_iter()
        .filter(|(_, ids)| ids.len() > 1)
        .collect();
    grupos.sort();
    Ok(grupos)
}

/// Procura sob `dir`, pelo nome normalizado, arquivos que casem com
/// localizações marcadas como ausentes. Caso único e ainda não cadastrado, a
/// nova localização é gravada (a antiga continua marcada como ausente).
/// Casos ambíguos ficam de fora, para decisão do usuário.
pub fn buscar_movidos(db: &Database, dir: &Path) -> Result<Vec<(TrackId, String)>, ErroLibrary> {
    let mut indice: HashMap<String, Vec<String>> = HashMap::new();
    indexar(dir, &mut indice)?;
    let mut achados = Vec::new();
    for loc in db.listar_localizacoes()? {
        if !loc.ausente {
            continue;
        }
        let Some(nome) = Path::new(&loc.caminho)
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(nome_normalizado)
        else {
            continue;
        };
        let Some(candidatos) = indice.get(&nome) else {
            continue;
        };
        if candidatos.len() != 1 {
            continue;
        }
        let novo = &candidatos[0];
        if db.id_por_localizacao(novo)?.is_some() {
            continue;
        }
        db.adicionar_localizacao(loc.faixa, novo)?;
        achados.push((loc.faixa, novo.clone()));
    }
    Ok(achados)
}

fn indexar(dir: &Path, indice: &mut HashMap<String, Vec<String>>) -> Result<(), ErroLibrary> {
    for e in std::fs::read_dir(dir)? {
        let e = e?;
        let caminho = e.path();
        if e.file_type()?.is_dir() {
            indexar(&caminho, indice)?;
        } else if let Some(nome) = e.file_name().to_str().and_then(nome_normalizado) {
            indice
                .entry(nome)
                .or_default()
                .push(caminho.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}
