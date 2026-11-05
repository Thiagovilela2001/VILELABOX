use crate::marks::{self, TipoMarca};
pub use crate::state::count_bars;
use crate::state::{AppState, DeckState, SortColumn, SortDirection};
use crate::waveform;
use crate::{AppWindow, Bar, LinhaCue, Marca, Segment, TrackRow, WaveTile};
use slint::{Color, Image, ModelRc, SharedString, VecModel};

pub fn phrase_rgb(kind: &str) -> (u8, u8, u8) {
    match kind {
        "INTRO" => (0x2b, 0x5d, 0xa8),
        "UP" | "VERSE" => (0x2e, 0x8b, 0x57),
        "CHORUS" => (0xd9, 0x7a, 0x1e),
        "DOWN" | "BRIDGE" => (0x7b, 0x4f, 0xa8),
        "OUTRO" => (0x55, 0x5a, 0x63),
        _ => (0x3c, 0x40, 0x46),
    }
}

pub fn phrase_color(kind: &str) -> Color {
    let (r, g, b) = phrase_rgb(kind);
    Color::from_rgb_u8(r, g, b)
}

pub fn parse_scale(raw: Option<&str>) -> f32 {
    raw.and_then(|v| v.trim().parse::<f32>().ok())
        .unwrap_or(1.0)
        .clamp(0.35, 1.0)
}

pub fn system_clock_str() -> String {
    chrono::Local::now().format("%H:%M").to_string()
}

pub fn build_track_rows(state: &AppState) -> Vec<TrackRow> {
    state
        .filtered_indices
        .iter()
        .enumerate()
        .filter_map(|(pos, &track_idx)| {
            let track = state.tracks.get(track_idx)?;
            let (artist, title) = track.artist_title();
            let bpm = crate::state::fmt_bpm(track.bpm());
            let duration = crate::state::fmt_dur(track.dur_ms());
            let meta = track.meta.clone().unwrap_or_default();
            let bars_count = count_bars(track.grid.as_ref());

            Some(TrackRow {
                id: (pos + 1) as i32,
                title: SharedString::from(title),
                artist: SharedString::from(artist),
                bpm: SharedString::from(bpm),
                key: SharedString::from(meta.tonality),
                album: SharedString::from(meta.album),
                genre: SharedString::from(meta.genre),
                stars: i32::from(meta.stars),
                duration: SharedString::from(duration),
                bars_count,
                alt: pos % 2 == 1,
            })
        })
        .collect()
}

/// Entrada do cache de waveforms: (índice da faixa, imagem geral, blocos da ampliada).
type WaveCacheEntry = (Option<usize>, Image, Vec<WaveTile>);

/// Imagem geral e blocos da ampliada do deck; vazios quando a faixa não tem `.EXT`.
/// Cacheadas por (deck, faixa): converter ~1 MB por render travava a UI.
fn wave_images(deck: &DeckState, n: usize) -> (Image, ModelRc<WaveTile>) {
    thread_local! {
        static CACHE: std::cell::RefCell<[Option<WaveCacheEntry>; 2]> = Default::default();
    }
    CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if let Some((idx, g, a)) = &c[n] {
            if *idx == deck.loaded_track_idx {
                return (g.clone(), ModelRc::new(VecModel::from(a.clone())));
            }
        }
        let (g, a) = wave_images_raw(deck);
        c[n] = Some((deck.loaded_track_idx, g.clone(), a.clone()));
        (g, ModelRc::new(VecModel::from(a)))
    })
}

fn wave_images_raw(deck: &DeckState) -> (Image, Vec<WaveTile>) {
    let geral = deck
        .preview
        .as_ref()
        .map(|p| Image::from_rgba8(waveform::render_preview(p)))
        .unwrap_or_default();
    let ampliada = deck
        .detail
        .as_ref()
        .map(|d| {
            waveform::detail_blocks(d)
                .into_iter()
                .map(|(inicio, buf)| WaveTile {
                    colunas: buf.width() as i32,
                    img: Image::from_rgba8(buf),
                    inicio: inicio as i32,
                })
                .collect()
        })
        .unwrap_or_default();
    (geral, ampliada)
}

fn rgb((r, g, b): (u8, u8, u8)) -> Color {
    Color::from_rgb_u8(r, g, b)
}

fn modelo_marcas(m: Vec<marks::Marca>) -> ModelRc<Marca> {
    let v: Vec<Marca> = m
        .into_iter()
        .map(|m| Marca {
            pos: m.pos,
            fim: m.fim,
            tipo: match m.tipo {
                TipoMarca::Batida => 0,
                TipoMarca::Compasso => 1,
                TipoMarca::HotCue => 2,
                TipoMarca::Memoria => 3,
                TipoMarca::Loop => 4,
            },
            cor: rgb(m.rgb),
            letra: SharedString::from(m.letra),
        })
        .collect();
    ModelRc::new(VecModel::from(v))
}

/// Marcas (grade + cues) da janela ampliada centrada na posição atual.
fn marcas_detalhe(deck: &DeckState, velocidade: f64) -> ModelRc<Marca> {
    let (ini, dur) = waveform::janela_com_velocidade(deck.pos_s, velocidade);
    let mut detalhe = marks::grade(&deck.beats, ini, dur);
    detalhe.extend(marks::cues(&deck.cues, ini, dur));
    modelo_marcas(detalhe)
}

/// Marcas da faixa inteira (cues) e a lista de cues do deck (memória no
/// deck 1, A..H no deck 2).
fn deck_marks(deck: &DeckState, hot: bool) -> (ModelRc<Marca>, ModelRc<LinhaCue>) {
    let geral = marks::cues(&deck.cues, 0.0, deck.dur_s);
    let linhas = if hot {
        marks::lista_hot(&deck.cues)
    } else {
        marks::lista_memoria(&deck.cues)
    };
    let linhas: Vec<LinhaCue> = linhas
        .into_iter()
        .map(|l| LinhaCue {
            letra: SharedString::from(l.letra),
            texto: SharedString::from(l.texto),
            cor: rgb(l.rgb),
            e_loop: l.e_loop,
        })
        .collect();
    (modelo_marcas(geral), ModelRc::new(VecModel::from(linhas)))
}

fn render_deck1(deck: &DeckState, ui: &AppWindow) {
    let (geral_m, cues) = deck_marks(deck, false);
    ui.set_deck1_marcas_geral(geral_m);
    ui.set_deck1_cues(cues);
    let (geral, ampliada) = wave_images(deck, 0);
    ui.set_deck1_wave_overview(geral);
    ui.set_deck1_wave_tiles(ampliada);
    ui.set_deck1_title(SharedString::from(&deck.title));
    ui.set_deck1_artist(SharedString::from(&deck.artist));
    ui.set_deck1_bpm(SharedString::from(&deck.bpm));
    ui.set_deck1_key(SharedString::from(&deck.key));
    ui.set_deck1_duration(SharedString::from(&deck.duration));
    ui.set_deck1_audio_ready(deck.audio_ready);

    let bars: Vec<Bar> = deck.bars.iter().map(|&pos| Bar { pos }).collect();
    let phrases: Vec<Segment> = deck
        .phrases
        .iter()
        .map(|p| Segment {
            start: p.start,
            end: p.end,
            color: phrase_color(&p.kind),
            label: SharedString::from(&p.kind),
        })
        .collect();
    ui.set_deck1_bars(ModelRc::new(VecModel::from(bars)));
    ui.set_deck1_phrases(ModelRc::new(VecModel::from(phrases)));
}

fn render_deck2(deck: &DeckState, ui: &AppWindow) {
    let (geral_m, cues) = deck_marks(deck, true);
    ui.set_deck2_marcas_geral(geral_m);
    ui.set_deck2_cues(cues);
    let (geral, ampliada) = wave_images(deck, 1);
    ui.set_deck2_wave_overview(geral);
    ui.set_deck2_wave_tiles(ampliada);
    ui.set_deck2_title(SharedString::from(&deck.title));
    ui.set_deck2_artist(SharedString::from(&deck.artist));
    ui.set_deck2_bpm(SharedString::from(&deck.bpm));
    ui.set_deck2_key(SharedString::from(&deck.key));
    ui.set_deck2_duration(SharedString::from(&deck.duration));
    ui.set_deck2_audio_ready(deck.audio_ready);

    let bars: Vec<Bar> = deck.bars.iter().map(|&pos| Bar { pos }).collect();
    let phrases: Vec<Segment> = deck
        .phrases
        .iter()
        .map(|p| Segment {
            start: p.start,
            end: p.end,
            color: phrase_color(&p.kind),
            label: SharedString::from(&p.kind),
        })
        .collect();
    ui.set_deck2_bars(ModelRc::new(VecModel::from(bars)));
    ui.set_deck2_phrases(ModelRc::new(VecModel::from(phrases)));
}

pub fn render(state: &AppState, ui: &AppWindow) {
    let rows = build_track_rows(state);
    ui.set_track_rows(ModelRc::new(VecModel::from(rows)));

    let selected_row = state.selected_filtered_index.map_or(-1, |idx| idx as i32);
    ui.set_selected_row(selected_row);
    ui.set_status(SharedString::from(&state.status_text));
    ui.set_search_text(SharedString::from(&state.filter_text));

    let sort_col_idx = match state.sort_column {
        SortColumn::Id => 0,
        SortColumn::Title => 1,
        SortColumn::Artist => 2,
        SortColumn::Bpm => 3,
        SortColumn::Duration => 4,
        SortColumn::Bars => 5,
    };
    ui.set_sort_col(sort_col_idx);
    ui.set_sort_asc(state.sort_direction == SortDirection::Ascending);

    ui.set_clock_text(SharedString::from(&state.clock_text));
    let listas: Vec<SharedString> = state
        .playlists
        .iter()
        .map(|p| SharedString::from(&p.name))
        .collect();
    ui.set_playlists(ModelRc::new(VecModel::from(listas)));
    ui.set_playlist_sel(state.playlist.map_or(-1, |i| i as i32));

    render_deck1(&state.deck1, ui);
    render_deck2(&state.deck2, ui);
    render_transport(state, ui);
    render_mixer(state, ui);

    ui.set_deck1_loop_tamanho(crate::loops::rotulo(state.loop_batidas[0]).into());
    ui.set_deck2_loop_tamanho(crate::loops::rotulo(state.loop_batidas[1]).into());
    ui.set_deck1_loop_ativo(state.loop_ativo[0]);
    ui.set_deck2_loop_ativo(state.loop_ativo[1]);
    ui.set_deck1_quantize(state.quantize[0]);
    ui.set_deck2_quantize(state.quantize[1]);
    render_botoes(state, ui);
}

/// Fader, TRIM e crossfader (p7-0). Barato: chamado a cada evento de arrasto.
pub fn render_mixer(st: &AppState, ui: &AppWindow) {
    ui.set_mix_fader(ModelRc::new(VecModel::from(st.mix_fader.to_vec())));
    ui.set_mix_trim(ModelRc::new(VecModel::from(st.mix_trim.to_vec())));
    ui.set_mix_cross(st.mix_cross);
}

/// MT, BEAT SYNC, MASTER, KEY SYNC, SLIP e AU/MA (dossiê p6-2).
fn render_botoes(st: &AppState, ui: &AppWindow) {
    use crate::beatgrid::EditRange;
    use crate::sync;
    for i in 0..2 {
        let d = if i == 0 { &st.deck1 } else { &st.deck2 };
        let bpm = if st.velocidade[i] == 1.0 {
            d.bpm.clone()
        } else {
            crate::state::fmt_bpm(sync::bpm_efetivo(st, i))
        };
        let n = sync::shift_exibido(st, i);
        let tom = sync::transpor(&d.key, n).unwrap_or_else(|| d.key.clone());
        let shift = sync::rotulo_shift(n);
        let pitch = sync::rotulo_pitch(st.velocidade[i]);
        if i == 0 {
            ui.set_deck1_bpm(bpm.into());
            ui.set_deck1_key(tom.into());
            ui.set_deck1_key_shift(shift.into());
            ui.set_deck1_pitch(pitch.into());
            ui.set_deck1_master(st.master == 0);
            ui.set_deck1_beat_sync(st.beat_sync[0]);
            ui.set_deck1_key_sync(st.key_sync[0]);
            ui.set_deck1_slip(st.slip[0]);
            ui.set_deck1_mt(st.master_tempo[0]);
            ui.set_deck1_loop_manual(st.loop_manual[0]);
            ui.set_deck1_grid_fine(st.grid_edit[0].fine);
            ui.set_deck1_grid_partial(st.grid_edit[0].range == EditRange::FromPosition);
            ui.set_deck1_grid_locked(st.grid_edit[0].locked);
            ui.set_deck1_grid_can_undo(st.grid_edit[0].can_undo());
            ui.set_deck1_grid_can_redo(st.grid_edit[0].can_redo());
            ui.set_deck1_grid_metronome(i32::from(st.grid_edit[0].metronome));
        } else {
            ui.set_deck2_bpm(bpm.into());
            ui.set_deck2_key(tom.into());
            ui.set_deck2_key_shift(shift.into());
            ui.set_deck2_pitch(pitch.into());
            ui.set_deck2_master(st.master == 1);
            ui.set_deck2_beat_sync(st.beat_sync[1]);
            ui.set_deck2_key_sync(st.key_sync[1]);
            ui.set_deck2_slip(st.slip[1]);
            ui.set_deck2_mt(st.master_tempo[1]);
            ui.set_deck2_loop_manual(st.loop_manual[1]);
            ui.set_deck2_grid_fine(st.grid_edit[1].fine);
            ui.set_deck2_grid_partial(st.grid_edit[1].range == EditRange::FromPosition);
            ui.set_deck2_grid_locked(st.grid_edit[1].locked);
            ui.set_deck2_grid_can_undo(st.grid_edit[1].can_undo());
            ui.set_deck2_grid_can_redo(st.grid_edit[1].can_redo());
            ui.set_deck2_grid_metronome(i32::from(st.grid_edit[1].metronome));
        }
    }
}

fn pos_rel(d: &DeckState) -> f32 {
    if d.dur_s > 0.0 {
        (d.pos_s / d.dur_s).clamp(0.0, 1.0) as f32
    } else {
        0.0
    }
}

/// Só posição/tempo/play/cue dos decks: barato, roda no timer de 25 ms.
pub fn render_transport(state: &AppState, ui: &AppWindow) {
    let (d1, d2) = (&state.deck1, &state.deck2);
    ui.set_deck1_pos_rel(pos_rel(d1));
    ui.set_deck1_tempo_decorrido(SharedString::from(&d1.tempo_decorrido));
    ui.set_deck1_tempo_restante(SharedString::from(&d1.tempo_restante));
    ui.set_deck1_tocando(d1.is_playing);
    ui.set_deck1_cue_aceso(d1.is_cue_standby);
    ui.set_deck1_wave_pos(d1.pos_s as f32);
    ui.set_deck1_wave_rate(state.velocidade[0] as f32);
    ui.set_deck1_marcas_detalhe(marcas_detalhe(d1, state.velocidade[0]));
    ui.set_deck2_pos_rel(pos_rel(d2));
    ui.set_deck2_tempo_decorrido(SharedString::from(&d2.tempo_decorrido));
    ui.set_deck2_tempo_restante(SharedString::from(&d2.tempo_restante));
    ui.set_deck2_tocando(d2.is_playing);
    ui.set_deck2_cue_aceso(d2.is_cue_standby);
    ui.set_deck2_wave_pos(d2.pos_s as f32);
    ui.set_deck2_wave_rate(state.velocidade[1] as f32);
    ui.set_deck2_marcas_detalhe(marcas_detalhe(d2, state.velocidade[1]));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::library::Track;
    use crate::state::Command;
    use crate::state::{duration_ms, fmt_bpm, fmt_dur, split_artist_title};
    use vilelabox_anlz::{Beat, Grid};

    fn sample_grid(beats: &[(u16, u32)], bpm: f64) -> Grid {
        Grid {
            bpm,
            variable: false,
            first_beat_time_s: beats.first().map_or(0.0, |b| b.1 as f64 / 1000.0),
            downbeat_index: 0,
            beats: beats
                .iter()
                .map(|&(beat_number, time_ms)| Beat {
                    beat_number,
                    bpm,
                    time_ms,
                })
                .collect(),
        }
    }

    #[test]
    fn formata_bpm_com_e_sem_grade() {
        assert_eq!(fmt_bpm(0.0), "—");
        let g = sample_grid(&[(1, 0)], 128.0);
        assert_eq!(fmt_bpm(g.bpm), "128.00");
    }

    #[test]
    fn formata_duracao_e_conta_compassos() {
        assert_eq!(fmt_dur(0), "—");
        let empty = sample_grid(&[], 120.0);
        assert_eq!(fmt_dur(duration_ms(Some(&empty))), "—");
        assert_eq!(count_bars(Some(&empty)), 0);

        // 120 BPM (500 ms por batida), última batida em 124_500 ms -> total 125_000 ms (02:05)
        let g = sample_grid(&[(1, 0), (2, 500), (1, 2000), (4, 124_500)], 120.0);
        assert_eq!(fmt_dur(duration_ms(Some(&g))), "02:05");
        assert_eq!(count_bars(Some(&g)), 2);
    }

    #[test]
    fn separa_artista_e_titulo() {
        assert_eq!(
            split_artist_title("01 - DJ Teste - Faixa Nova.mp3"),
            ("DJ Teste".into(), "Faixa Nova".into())
        );
        assert_eq!(
            split_artist_title("Faixa Solta.FLAC"),
            ("Vilelabox / Coleção".into(), "Faixa Solta".into())
        );
    }

    #[test]
    fn cores_de_frase_por_categoria() {
        assert_eq!(phrase_rgb("INTRO"), (0x2b, 0x5d, 0xa8));
        assert_eq!(phrase_rgb("UP"), (0x2e, 0x8b, 0x57));
        assert_eq!(phrase_rgb("VERSE"), (0x2e, 0x8b, 0x57));
        assert_eq!(phrase_rgb("CHORUS"), (0xd9, 0x7a, 0x1e));
        assert_eq!(phrase_rgb("DOWN"), (0x7b, 0x4f, 0xa8));
        assert_eq!(phrase_rgb("BRIDGE"), (0x7b, 0x4f, 0xa8));
        assert_eq!(phrase_rgb("OUTRO"), (0x55, 0x5a, 0x63));
        assert_eq!(phrase_rgb("DESCONHECIDA"), (0x3c, 0x40, 0x46));
    }

    #[test]
    fn escala_respeita_limites_e_padrao() {
        assert_eq!(parse_scale(None), 1.0);
        assert_eq!(parse_scale(Some("invalido")), 1.0);
        assert_eq!(parse_scale(Some("0.62")), 0.62);
        assert_eq!(parse_scale(Some("0.10")), 0.35);
        assert_eq!(parse_scale(Some("2.00")), 1.0);
    }

    #[test]
    fn monta_linhas_da_tabela() {
        let mut state = AppState::new();
        state.apply(Command::ScanFinished {
            tracks: vec![
                Track {
                    title: "Artista A - Uma.mp3".into(),
                    dat: None,
                    path: None,
                    audio_ms: None,
                    meta: None,
                    ext: None,
                    grid: Some(sample_grid(&[(1, 0), (1, 2000)], 120.0)),
                },
                Track {
                    title: "Sem Grade.wav".into(),
                    dat: None,
                    path: None,
                    audio_ms: None,
                    meta: None,
                    ext: None,
                    grid: None,
                },
            ],
            summary: "2 faixas".into(),
        });

        let rows = build_track_rows(&state);
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].id, 1);
        assert_eq!(rows[0].artist.as_str(), "Artista A");
        assert_eq!(rows[0].title.as_str(), "Uma");
        assert_eq!(rows[0].bpm.as_str(), "120.00");
        assert_eq!(rows[0].bars_count, 2);
        assert!(!rows[0].alt);
        assert_eq!(rows[1].id, 2);
        assert_eq!(rows[1].bpm.as_str(), "—");
        assert!(rows[1].alt);
    }
}
