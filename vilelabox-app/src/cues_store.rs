//! Cues gravados pelo usuário, um arquivo JSON por faixa. Decisão de produto
//! (docs/armazenamento-de-cues.md): o rekordbox grava em `djmdCue` do
//! `master.db` (evidence/p6-4-memory-persistencia.md); aqui o armazenamento é
//! próprio. Não conhece Slint nem o motor: recebe e devolve dados.

use serde::{Deserialize, Serialize};
use std::io::Write;
use std::path::{Path, PathBuf};
use vilelabox_anlz::RawCue;

pub const VERSAO: u32 = 1;

#[derive(Serialize, Deserialize, PartialEq, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum Tipo {
    Memory,
    Hot,
}

#[derive(Serialize, Deserialize)]
struct Cue {
    tipo: Tipo,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    slot: Option<u32>,
    pos_s: f64,
}

#[derive(Serialize, Deserialize)]
struct Arquivo {
    versao: u32,
    faixa: String,
    cues: Vec<Cue>,
}

/// `%APPDATA%\vilelabox\cues` (None se a variável faltar).
pub fn dir_padrao() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("vilelabox").join("cues"))
}

/// FNV-1a 64 bits dos bytes do caminho.
fn fnv1a(s: &str) -> u64 {
    s.bytes().fold(0xcbf2_9ce4_8422_2325, |h, b| {
        (h ^ u64::from(b)).wrapping_mul(0x0100_0000_01b3)
    })
}

/// `<nome base sanitizado>-<fnv1a hex>.json`: mesmo caminho, mesmo arquivo.
pub fn arquivo(base: &Path, faixa: &str) -> PathBuf {
    let nome = faixa.rsplit(['/', '\\']).next().unwrap_or("");
    let nome = nome.rsplit_once('.').map_or(nome, |(n, _)| n);
    let nome: String = nome
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .take(40)
        .collect();
    base.join(format!("{nome}-{:016x}.json", fnv1a(faixa)))
}

/// Cues da faixa; ausente, inválido ou de versão desconhecida = vazio.
pub fn ler(base: &Path, faixa: &str) -> Vec<RawCue> {
    let Ok(txt) = std::fs::read_to_string(arquivo(base, faixa)) else {
        return Vec::new();
    };
    match serde_json::from_str::<Arquivo>(&txt) {
        Ok(a) if a.versao == VERSAO => a
            .cues
            .into_iter()
            .map(|c| RawCue {
                hot_cue: if c.tipo == Tipo::Hot {
                    c.slot.unwrap_or(0)
                } else {
                    0
                },
                status: 1,
                is_loop: false,
                start_time_s: c.pos_s,
                end_time_s: None,
                label: None,
                color: None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Grava os cues (loops ficam de fora) em `.tmp` e renomeia por cima.
pub fn gravar(base: &Path, faixa: &str, cues: &[RawCue]) -> std::io::Result<()> {
    let a = Arquivo {
        versao: VERSAO,
        faixa: faixa.into(),
        cues: cues
            .iter()
            .filter(|c| !c.is_loop)
            .map(|c| Cue {
                tipo: if c.hot_cue == 0 {
                    Tipo::Memory
                } else {
                    Tipo::Hot
                },
                slot: (c.hot_cue != 0).then_some(c.hot_cue),
                pos_s: c.start_time_s,
            })
            .collect(),
    };
    std::fs::create_dir_all(base)?;
    let destino = arquivo(base, faixa);
    let tmp = destino.with_extension("json.tmp");
    let r = (|| {
        let mut f = std::fs::File::create(&tmp)?;
        f.write_all(serde_json::to_string_pretty(&a)?.as_bytes())?;
        f.sync_all()?;
        drop(f);
        std::fs::rename(&tmp, &destino)
    })();
    if r.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::{gravar_hot_cue, gravar_memory_cue};

    fn dir(nome: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("vilelabox-cues-{nome}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    const FAIXA: &str = r"C:\Musica\Artista - Faixa (Mix).mp3";

    #[test]
    fn ida_e_volta_com_ordem_e_sem_tmp() {
        let d = dir("ida");
        let mut cues = Vec::new();
        gravar_hot_cue(&mut cues, 3, 12.5);
        gravar_memory_cue(&mut cues, 51.718);
        gravar(&d, FAIXA, &cues).unwrap();
        let lidos = ler(&d, FAIXA);
        let v: Vec<_> = lidos.iter().map(|c| (c.hot_cue, c.start_time_s)).collect();
        assert_eq!(v, [(3, 12.5), (0, 51.718)]);
        let nomes: Vec<_> = std::fs::read_dir(&d)
            .unwrap()
            .map(|e| e.unwrap().file_name())
            .collect();
        assert_eq!(nomes.len(), 1);
        assert!(!nomes[0].to_string_lossy().ends_with(".tmp"));
        println!("{}", std::fs::read_to_string(arquivo(&d, FAIXA)).unwrap());
    }

    #[test]
    fn ausente_invalido_e_versao_desconhecida_dao_vazio() {
        let d = dir("ruim");
        assert!(ler(&d, FAIXA).is_empty());
        std::fs::create_dir_all(&d).unwrap();
        let f = arquivo(&d, FAIXA);
        for txt in [
            "{nao e json",
            r#"{"versao":99,"faixa":"x","cues":[]}"#,
            r#"{"versao":1,"faixa":"x","cues":[{"tipo":"hot"}]}"#,
            r#"{"versao":1,"cues":[]}"#,
        ] {
            std::fs::write(&f, txt).unwrap();
            assert!(ler(&d, FAIXA).is_empty(), "{txt}");
        }
    }

    #[test]
    fn chave_estavel() {
        let b = Path::new("base");
        assert_eq!(arquivo(b, FAIXA), arquivo(b, FAIXA));
        assert_ne!(arquivo(b, FAIXA), arquivo(b, r"C:\Musica\Outra.mp3"));
        assert_ne!(arquivo(b, "/a/x.mp3"), arquivo(b, "/b/x.mp3"));
        // FNV-1a de referência: vazio = offset basis, "a" = af63dc4c8601ec8c.
        assert_eq!(fnv1a(""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(fnv1a("a"), 0xaf63_dc4c_8601_ec8c);
        let n = arquivo(b, FAIXA)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert!(n.starts_with("Artista_-_Faixa__Mix_-"), "{n}");
    }

    #[test]
    fn hot_cue_mesmo_slot_sobrescreve() {
        let d = dir("slot");
        let mut cues = Vec::new();
        gravar_hot_cue(&mut cues, 1, 1.0);
        gravar_hot_cue(&mut cues, 1, 2.0);
        gravar(&d, FAIXA, &cues).unwrap();
        let v: Vec<_> = ler(&d, FAIXA)
            .iter()
            .map(|c| (c.hot_cue, c.start_time_s))
            .collect();
        assert_eq!(v, [(1, 2.0)]);
    }
}
