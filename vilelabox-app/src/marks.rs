//! Marcadores sobre as formas de onda e linhas das listas de cues.
//!
//! Lógica pura (sem Slint): posições relativas 0..1 dentro de uma janela de
//! tempo, para a UI só multiplicar pela largura.
//!
//! Convenção dos cues do ANLZ (`RawCue.hot_cue`): 0 = memória, 1..8 = A..H.

use vilelabox_anlz::{Beat, RawCue};

/// Marca na janela: posição relativa, fim relativo (loops), tipo e cor.
#[derive(Debug, Clone, PartialEq)]
pub struct Marca {
    pub pos: f32,
    pub fim: f32,
    pub tipo: TipoMarca,
    pub rgb: (u8, u8, u8),
    pub letra: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoMarca {
    Batida,
    Compasso,
    HotCue,
    Memoria,
    Loop,
}

/// Linha da lista de cues de um deck.
#[derive(Debug, Clone, PartialEq)]
pub struct LinhaCue {
    pub letra: String,
    pub texto: String,
    pub rgb: (u8, u8, u8),
    pub e_loop: bool,
}

const COR_MEMORIA: (u8, u8, u8) = (0xe0, 0x26, 0x2f);
const COR_HOT_PADRAO: (u8, u8, u8) = (0x2b, 0xb3, 0x4a);

fn letra(hot_cue: u32) -> String {
    char::from_u32(u32::from(b'A') + hot_cue.saturating_sub(1))
        .map(String::from)
        .unwrap_or_default()
}

fn cor(c: &RawCue) -> (u8, u8, u8) {
    match c.color {
        Some(rgb) => ((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8),
        None if c.hot_cue == 0 => COR_MEMORIA,
        None => COR_HOT_PADRAO,
    }
}

fn relativo(t: f64, inicio: f64, dur: f64) -> f32 {
    ((t - inicio) / dur) as f32
}

/// Batidas e compassos dentro de `[inicio, inicio + dur)` segundos.
pub fn grade(beats: &[Beat], inicio: f64, dur: f64) -> Vec<Marca> {
    if dur <= 0.0 {
        return Vec::new();
    }
    beats
        .iter()
        .map(|b| (f64::from(b.time_ms) / 1000.0, b.beat_number))
        .filter(|&(t, _)| t >= inicio && t < inicio + dur)
        .map(|(t, n)| {
            let pos = relativo(t, inicio, dur);
            Marca {
                pos,
                fim: pos,
                tipo: if n == 1 {
                    TipoMarca::Compasso
                } else {
                    TipoMarca::Batida
                },
                rgb: (255, 255, 255),
                letra: String::new(),
            }
        })
        .collect()
}

/// Cues que começam dentro de `[inicio, inicio + dur)`; loops levam o fim.
pub fn cues(cues: &[RawCue], inicio: f64, dur: f64) -> Vec<Marca> {
    if dur <= 0.0 {
        return Vec::new();
    }
    cues.iter()
        .filter(|c| c.start_time_s >= inicio && c.start_time_s < inicio + dur)
        .map(|c| {
            let pos = relativo(c.start_time_s, inicio, dur);
            let fim = c
                .end_time_s
                .filter(|_| c.is_loop)
                .map_or(pos, |e| relativo(e, inicio, dur).min(1.0));
            let tipo = match (c.hot_cue, c.is_loop) {
                (0, true) => TipoMarca::Loop,
                (0, false) => TipoMarca::Memoria,
                _ => TipoMarca::HotCue,
            };
            Marca {
                pos,
                fim,
                tipo,
                rgb: cor(c),
                letra: if c.hot_cue > 0 {
                    letra(c.hot_cue)
                } else {
                    String::new()
                },
            }
        })
        .collect()
}

/// Tempo no formato da lista de cues: `MM:SS:mmm`.
pub fn tempo_cue(s: f64) -> String {
    let ms = (s.max(0.0) * 1000.0).round() as u64;
    format!("{:02}:{:02}:{:03}", ms / 60_000, ms / 1000 % 60, ms % 1000)
}

/// Lista de memória: cues com `hot_cue == 0`, em ordem de tempo.
pub fn lista_memoria(cues: &[RawCue]) -> Vec<LinhaCue> {
    let mut m: Vec<&RawCue> = cues.iter().filter(|c| c.hot_cue == 0).collect();
    m.sort_by(|a, b| a.start_time_s.total_cmp(&b.start_time_s));
    m.into_iter()
        .map(|c| LinhaCue {
            letra: String::new(),
            texto: c
                .label
                .clone()
                .filter(|l| !l.is_empty())
                .unwrap_or_else(|| tempo_cue(c.start_time_s)),
            rgb: cor(c),
            e_loop: c.is_loop,
        })
        .collect()
}

/// Lista de disparos rápidos: sempre 8 linhas, A..H, vazias quando não houver.
pub fn lista_hot(cues: &[RawCue]) -> Vec<LinhaCue> {
    (1..=8)
        .map(|n| {
            let c = cues.iter().find(|c| c.hot_cue == n);
            LinhaCue {
                letra: letra(n),
                texto: c
                    .map(|c| {
                        c.label
                            .clone()
                            .filter(|l| !l.is_empty())
                            .unwrap_or_else(|| tempo_cue(c.start_time_s))
                    })
                    .unwrap_or_default(),
                rgb: c.map_or((0x6b, 0x6b, 0x6b), cor),
                e_loop: c.is_some_and(|c| c.is_loop),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cue(hot: u32, ini: f64, fim: Option<f64>, cor: Option<u32>) -> RawCue {
        RawCue {
            hot_cue: hot,
            status: 0,
            is_loop: fim.is_some(),
            start_time_s: ini,
            end_time_s: fim,
            label: None,
            color: cor,
        }
    }

    fn beat(n: u16, ms: u32) -> Beat {
        Beat {
            beat_number: n,
            bpm: 120.0,
            time_ms: ms,
        }
    }

    #[test]
    fn grade_na_janela_marca_compassos() {
        let b = [beat(4, 500), beat(1, 1000), beat(2, 1500), beat(3, 3000)];
        let g = grade(&b, 1.0, 2.0);
        assert_eq!(g.len(), 2);
        assert_eq!((g[0].pos, g[0].tipo), (0.0, TipoMarca::Compasso));
        assert_eq!((g[1].pos, g[1].tipo), (0.25, TipoMarca::Batida));
    }

    #[test]
    fn cues_tipos_cores_e_fim_de_loop() {
        let c = [
            cue(1, 10.0, None, Some(0xFF3770)),
            cue(0, 20.0, None, None),
            cue(0, 30.0, Some(50.0), None),
            cue(2, 200.0, None, None), // fora da janela
        ];
        let m = cues(&c, 0.0, 40.0);
        assert_eq!(m.len(), 3);
        assert_eq!(
            (m[0].tipo, m[0].rgb, m[0].letra.as_str()),
            (TipoMarca::HotCue, (0xFF, 0x37, 0x70), "A")
        );
        assert_eq!((m[1].tipo, m[1].rgb), (TipoMarca::Memoria, COR_MEMORIA));
        assert_eq!(
            (m[2].tipo, m[2].pos, m[2].fim),
            (TipoMarca::Loop, 0.75, 1.0)
        );
    }

    #[test]
    fn listas_de_memoria_e_hot() {
        let c = [
            cue(0, 5.25, None, None),
            cue(0, 0.05, Some(3.743), None),
            cue(3, 1.0, None, None),
        ];
        let mem = lista_memoria(&c);
        assert_eq!(mem.len(), 2);
        assert_eq!((mem[0].texto.as_str(), mem[0].e_loop), ("00:00:050", true));
        assert_eq!(mem[1].texto, "00:05:250");
        let hot = lista_hot(&c);
        assert_eq!(hot.len(), 8);
        assert_eq!((hot[0].letra.as_str(), hot[0].texto.as_str()), ("A", ""));
        assert_eq!(
            (hot[2].letra.as_str(), hot[2].texto.as_str()),
            ("C", "00:01:000")
        );
    }
}
