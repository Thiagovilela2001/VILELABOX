//! Ponte entre o app e o núcleo headless (P5.1).
//!
//! O app continua com seu `AppState` para a tela; a preparação (faixa, cues
//! do usuário, grade e bloqueio) passa a morar no banco próprio, gravada e
//! lida pelo `vilelabox-core`. As posições do app são segundos (cues) e
//! milissegundos (grade); no banco são frames, que exigem a taxa de
//! amostragem. A taxa é gravada quando o áudio é decodificado; antes disso a
//! ponte recusa a gravação e o app usa os arquivos JSON de sempre.
//!
//! ADAPTAÇÃO: desenho do VILELABOX (`docs/IMPLEMENTATION-PLAN.md`, P5.1).

use std::path::{Path, PathBuf};
use vilelabox_anlz::{Beat, RawCue};
use vilelabox_core::{Comando, Core, Deck};
use vilelabox_types::{
    Beat as BatidaCore, BeatGrid, Bpm, Cue, CueKind, CueOrigin, Frame, SampleRate, TempoChange,
    TrackId,
};

/// Banco padrão: `%APPDATA%\vilelabox\vilelabox.db`.
pub fn caminho_padrao() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("vilelabox").join("vilelabox.db"))
}

/// Núcleo aberto pelo app.
pub struct Ponte {
    core: Core,
}

impl std::fmt::Debug for Ponte {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Ponte")
    }
}

/// Preparação lida do banco para um deck.
#[derive(Debug, Default, PartialEq)]
pub struct Preparacao {
    /// Cues do usuário, exceto loops (vazio = nada gravado).
    pub cues: Vec<RawCue>,
    /// Grade e bloqueio, quando gravados.
    pub grade: Option<(Vec<Beat>, bool)>,
}

fn deck(i: usize) -> Deck {
    if i == 0 {
        Deck::Um
    } else {
        Deck::Dois
    }
}

fn erro(e: impl std::fmt::Display) -> String {
    e.to_string()
}

/// Segundos para frames, arredondando (como o importador de XML).
fn frames(s: f64, taxa: u32) -> Frame {
    Frame::new((s.max(0.0) * f64::from(taxa)).round() as i64)
}

fn segundos(f: Frame, taxa: u32) -> f64 {
    f.get() as f64 / f64::from(taxa)
}

/// `RawCue` do app para `Cue` do núcleo; loops ficam de fora.
pub fn cue_do_app(c: &RawCue, taxa: u32) -> Option<Cue> {
    if c.is_loop {
        return None;
    }
    let tipo = if c.hot_cue > 0 {
        CueKind::HotCue
    } else {
        CueKind::MemoryCue
    };
    let mut cue = Cue::new(tipo, frames(c.start_time_s, taxa), CueOrigin::User);
    cue.slot = (c.hot_cue > 0).then(|| c.hot_cue.min(255) as u8);
    cue.label = c.label.clone().unwrap_or_default();
    cue.color = c.color;
    Some(cue)
}

/// `Cue` do núcleo para `RawCue` do app (status 1, como `cues_store`).
pub fn cue_para_app(c: &Cue, taxa: u32) -> RawCue {
    RawCue {
        hot_cue: if c.kind == CueKind::HotCue {
            u32::from(c.slot.unwrap_or(0))
        } else {
            0
        },
        status: 1,
        is_loop: false,
        start_time_s: segundos(c.position, taxa),
        end_time_s: None,
        label: (!c.label.is_empty()).then(|| c.label.clone()),
        color: c.color,
    }
}

/// Batidas do app (ms) para a grade do núcleo (frames), com uma mudança de
/// tempo em cada batida cujo BPM difere do anterior.
pub fn grade_do_app(beats: &[Beat], taxa: u32) -> Result<BeatGrid, String> {
    let sr = SampleRate::new(taxa).map_err(erro)?;
    let pos = |b: &Beat| frames(f64::from(b.time_ms) / 1000.0, taxa);
    let batidas = beats
        .iter()
        .map(|b| BatidaCore {
            position: pos(b),
            beat_num: u32::from(b.beat_number),
        })
        .collect();
    let mut tempo: Vec<TempoChange> = Vec::new();
    for b in beats {
        if tempo
            .last()
            .is_none_or(|t| (t.bpm.get() - b.bpm).abs() > 1e-9)
            && b.bpm > 0.0
        {
            tempo.push(TempoChange {
                position: pos(b),
                bpm: Bpm::new(b.bpm).map_err(erro)?,
            });
        }
    }
    let downbeat = beats.iter().find(|b| b.beat_number == 1).map(pos);
    BeatGrid::new(sr, batidas, tempo, downbeat).map_err(erro)
}

/// Grade do núcleo para as batidas do app.
pub fn grade_para_app(g: &BeatGrid) -> Vec<Beat> {
    let taxa = g.sample_rate().get();
    g.beats()
        .iter()
        .map(|b| Beat {
            beat_number: b.beat_num.min(u32::from(u16::MAX)) as u16,
            bpm: g.bpm_at(b.position).map_or(0.0, |x| x.get()),
            time_ms: (segundos(b.position, taxa) * 1000.0).round().max(0.0) as u32,
        })
        .collect()
}

impl Ponte {
    pub fn abrir(caminho: &Path) -> Result<Self, String> {
        if let Some(dir) = caminho.parent() {
            std::fs::create_dir_all(dir).map_err(erro)?;
        }
        Ok(Self {
            core: Core::abrir(caminho).map_err(erro)?,
        })
    }

    pub fn em_memoria() -> Self {
        Self {
            core: Core::novo(vilelabox_storage::Database::open_in_memory().expect("banco")),
        }
    }

    fn faixa(&mut self, caminho: &str, titulo: &str, artista: &str) -> Result<TrackId, String> {
        let vazio = |s: &str| (!s.trim().is_empty()).then(|| s.to_string());
        self.core
            .executar(Comando::RegistrarFaixa {
                caminho: caminho.into(),
                titulo: vazio(titulo),
                artista: vazio(artista),
            })
            .map_err(erro)?;
        self.core
            .faixa_por_caminho(caminho)
            .map_err(erro)?
            .ok_or_else(|| "faixa não registrada".into())
    }

    /// Registra a faixa (se nova), carrega no deck do núcleo e devolve a
    /// preparação gravada. Sem taxa conhecida, devolve vazio.
    pub fn carregar(
        &mut self,
        i: usize,
        caminho: &str,
        titulo: &str,
        artista: &str,
    ) -> Result<Preparacao, String> {
        let id = self.faixa(caminho, titulo, artista)?;
        self.core
            .executar(Comando::CarregarNoDeck {
                deck: deck(i),
                faixa: id,
            })
            .map_err(erro)?;
        let _ = self.core.drenar_eventos();
        let Some(taxa) = self.core.taxa(id).map_err(erro)? else {
            return Ok(Preparacao::default());
        };
        let Some(d) = self.core.deck(deck(i)) else {
            return Ok(Preparacao::default());
        };
        Ok(Preparacao {
            cues: d
                .cues
                .iter()
                .filter(|c| c.origin == CueOrigin::User && c.kind != CueKind::Loop)
                .map(|c| cue_para_app(c, taxa))
                .collect(),
            grade: d.grade.as_ref().map(|g| (grade_para_app(g), d.bloqueada)),
        })
    }

    /// Grava taxa, canais e duração do áudio decodificado.
    pub fn registrar_audio(
        &mut self,
        caminho: &str,
        taxa: u32,
        canais: u16,
        frames: u64,
    ) -> Result<(), String> {
        let id = self.faixa(caminho, "", "")?;
        self.core
            .executar(Comando::RegistrarAudio {
                faixa: id,
                taxa,
                canais,
                duracao_frames: frames.min(i64::MAX as u64) as i64,
            })
            .map_err(erro)?;
        let _ = self.core.drenar_eventos();
        Ok(())
    }

    fn taxa_do_deck(&self, i: usize) -> Result<Option<u32>, String> {
        match self.core.deck(deck(i)) {
            Some(d) => self.core.taxa(d.faixa.id).map_err(erro),
            None => Ok(None),
        }
    }

    /// Grava os cues do deck no banco. `Ok(false)` sem taxa conhecida.
    pub fn gravar_cues(&mut self, i: usize, cues: &[RawCue]) -> Result<bool, String> {
        let Some(taxa) = self.taxa_do_deck(i)? else {
            return Ok(false);
        };
        let cues = cues.iter().filter_map(|c| cue_do_app(c, taxa)).collect();
        self.core
            .executar(Comando::SubstituirCues {
                deck: deck(i),
                cues,
            })
            .map_err(erro)?;
        let _ = self.core.drenar_eventos();
        Ok(true)
    }

    /// Grava grade e bloqueio do deck no banco. `Ok(false)` sem taxa ou sem
    /// batidas.
    pub fn gravar_grade(
        &mut self,
        i: usize,
        beats: &[Beat],
        bloqueada: bool,
    ) -> Result<bool, String> {
        let Some(taxa) = self.taxa_do_deck(i)? else {
            return Ok(false);
        };
        if beats.is_empty() {
            return Ok(false);
        }
        let grade = grade_do_app(beats, taxa)?;
        self.core
            .executar(Comando::SalvarGrade {
                deck: deck(i),
                grade,
                bloqueada,
            })
            .map_err(erro)?;
        let _ = self.core.drenar_eventos();
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hot(slot: u32, s: f64) -> RawCue {
        RawCue {
            hot_cue: slot,
            status: 1,
            is_loop: false,
            start_time_s: s,
            end_time_s: None,
            label: None,
            color: None,
        }
    }

    #[test]
    fn cue_ida_e_volta() {
        let mut c = hot(3, 12.5);
        c.label = Some("drop".into());
        c.color = Some(0xff0000);
        let v = cue_para_app(&cue_do_app(&c, 44100).unwrap(), 44100);
        assert_eq!(v, c);
        let mut lp = hot(0, 1.0);
        lp.is_loop = true;
        assert!(cue_do_app(&lp, 44100).is_none());
    }

    #[test]
    fn grade_ida_e_volta() {
        let beats: Vec<Beat> = (0..8)
            .map(|i| Beat {
                beat_number: (i % 4 + 1) as u16,
                bpm: if i < 4 { 120.0 } else { 128.0 },
                time_ms: 100 + i * 500,
            })
            .collect();
        let g = grade_do_app(&beats, 48000).unwrap();
        assert_eq!(g.tempo_map().len(), 2);
        assert_eq!(grade_para_app(&g), beats);
    }

    #[test]
    fn sem_taxa_recusa_e_com_taxa_persiste() {
        let mut p = Ponte::em_memoria();
        let prep = p.carregar(0, "C:/m/a.mp3", "A", "B").unwrap();
        assert_eq!(prep, Preparacao::default());
        assert!(!p.gravar_cues(0, &[hot(1, 2.0)]).unwrap());
        p.registrar_audio("C:/m/a.mp3", 44100, 2, 441_000).unwrap();
        assert!(p.gravar_cues(0, &[hot(1, 2.0), hot(0, 5.0)]).unwrap());
        let beats = vec![Beat {
            beat_number: 1,
            bpm: 120.0,
            time_ms: 0,
        }];
        assert!(p.gravar_grade(0, &beats, true).unwrap());
        // Recarregar no outro deck devolve o que ficou gravado.
        let prep = p.carregar(1, "C:/m/a.mp3", "A", "B").unwrap();
        assert_eq!(prep.cues, vec![hot(1, 2.0), hot(0, 5.0)]);
        assert_eq!(prep.grade, Some((beats, true)));
    }
}
