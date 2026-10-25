//! Ações de loop e cue do deck (spec/rekordbox/32-loops-cues.md), fora dos
//! closures da tela: cada uma muda o `AppState` e devolve o comando que o
//! chamador envia ao motor (`None` = nenhuma chamada de motor). `i` é o deck
//! (0/1) e `pos` a posição do motor em segundos.

use crate::state::{AppState, Command, DeckId, DeckState};
use crate::{loops, sync};
use std::sync::Arc;
use vilelabox_audio::{AudioBuffer, AudioCommand};

fn deck(st: &mut AppState, i: usize) -> &mut DeckState {
    if i == 0 {
        &mut st.deck1
    } else {
        &mut st.deck2
    }
}

/// Áudio decodificado chegou ao deck: carrega no motor (o que limpa o loop
/// dele, `Deck::load_audio`) e marca o deck pronto.
pub fn carregar_audio(st: &mut AppState, id: DeckId, audio_buf: Arc<AudioBuffer>) -> AudioCommand {
    let i = if id == DeckId::Deck1 { 0 } else { 1 };
    st.apply(Command::SetDeckAudioReady { deck: id, ready: true });
    st.loop_ativo[i] = false;
    st.loop_in[i] = None;
    st.loop_pontos[i] = None;
    let d = deck(st, i);
    if d.dur_s <= 0.0 {
        let dur = audio_buf.duration_seconds();
        d.dur_s = dur;
        d.duration = crate::state::fmt_dur((dur * 1000.0) as u64);
        d.tempo_restante = format!("-{}", crate::state::fmt_tenths(dur));
    }
    d.audio_buf = Some(Arc::clone(&audio_buf));
    d.audio_ready = true;
    AudioCommand::LoadTrack { deck: i as u8 + 1, audio: audio_buf, cue_frame: Some(0) }
}

/// Clique no tamanho: sem loop, abre `loop_batidas` batidas na posição
/// (quantizada pela grade com Q ligado); com loop, encerra.
pub fn loop_alternar(st: &mut AppState, i: usize, pos: f64) -> Option<AudioCommand> {
    let n = i as u8 + 1;
    if st.loop_ativo[i] {
        st.loop_ativo[i] = false;
        return Some(AudioCommand::ClearLoop { deck: n });
    }
    let (batidas, q) = (st.loop_batidas[i], st.quantize[i]);
    let d = deck(st, i);
    let (Some(beat), Some(sr)) = (
        loops::batida_s(&d.beats, pos),
        d.audio_buf.as_ref().map(|b| f64::from(b.sample_rate)),
    ) else {
        st.status_text = "Loop indisponível: faixa sem grade de batidas".into();
        return None;
    };
    let len_frames = (batidas * beat * sr).round() as u64;
    // Q ligado: o app quantiza o início pela grade (para baixo, p6-2);
    // desligado, o motor usa a posição exata dele.
    let start_frame = q.then(|| (loops::quantizar_s(&d.beats, pos, true) * sr).round() as u64);
    st.loop_ativo[i] = true;
    Some(AudioCommand::AutoLoop { deck: n, len_frames, start_frame })
}

/// ½ (`dobro = false`) e ×2: observado só com loop ativo; sem loop, nada.
pub fn loop_cut(st: &mut AppState, i: usize, dobro: bool) -> Option<AudioCommand> {
    if !st.loop_ativo[i] {
        return None;
    }
    let (num, den) = if dobro { (2, 1) } else { (1, 2) };
    st.loop_batidas[i] = st.loop_batidas[i] * num as f64 / den as f64;
    Some(AudioCommand::LoopCut { deck: i as u8 + 1, num, den })
}

/// Q (unit#9, p6-2): estado; o arredondamento para baixo é aplicado pelo app
/// ao gravar loop automático e hot cue (`loops::quantizar_s`).
pub fn quantize(st: &mut AppState, i: usize) -> AudioCommand {
    st.quantize[i] = !st.quantize[i];
    AudioCommand::SetQuantize { deck: i as u8 + 1, on: st.quantize[i] }
}

/// AU/MA: troca a caixa de loop, sem chamada de motor.
pub fn loop_modo(st: &mut AppState, i: usize) {
    st.loop_manual[i] = !st.loop_manual[i];
}

fn frame(st: &mut AppState, i: usize, s: f64) -> Option<u64> {
    let sr = f64::from(deck(st, i).audio_buf.as_ref()?.sample_rate);
    Some((s * sr).round() as u64)
}

/// IN (unit#52, p6-5): guarda a entrada e faz dela o ponto de cue; o
/// transporte segue.
pub fn loop_in(st: &mut AppState, i: usize, pos: f64) -> Option<AudioCommand> {
    let f = frame(st, i, pos)?;
    st.loop_in[i] = Some(pos);
    Some(AudioCommand::SetCuePoint { deck: i as u8 + 1, frame: f })
}

/// OUT (unit#53, p6-5): fecha o loop IN..posição atual e passa a repetir.
/// Sem IN anterior, nada (não observado).
pub fn loop_out(st: &mut AppState, i: usize, pos: f64) -> Option<AudioCommand> {
    let entrada = st.loop_in[i].filter(|&e| pos > e)?;
    let (a, b) = (frame(st, i, entrada)?, frame(st, i, pos)?);
    st.loop_pontos[i] = Some((entrada, pos));
    st.loop_ativo[i] = true;
    Some(AudioCommand::SetLoop { deck: i as u8 + 1, start_frame: a, end_frame: b })
}

/// RELOOP/EXIT, o mesmo botão (p6-5). Com loop: EXIT encerra e a reprodução
/// segue. Sem loop: RELOOP volta ao IN e repete o último IN/OUT.
pub fn loop_exit_reloop(st: &mut AppState, i: usize) -> Vec<AudioCommand> {
    let n = i as u8 + 1;
    if st.loop_ativo[i] {
        st.loop_ativo[i] = false;
        return vec![AudioCommand::ClearLoop { deck: n }];
    }
    let Some((e, s)) = st.loop_pontos[i] else { return Vec::new() };
    let (Some(a), Some(b)) = (frame(st, i, e), frame(st, i, s)) else { return Vec::new() };
    st.loop_ativo[i] = true;
    vec![
        AudioCommand::SetLoop { deck: n, start_frame: a, end_frame: b },
        AudioCommand::SeekFrame { deck: n, frame: a },
    ]
}

/// MEMORY: grava na lista do app e persiste, sem chamada de motor.
pub fn memory(st: &mut AppState, i: usize, pos: f64) {
    let d = deck(st, i);
    if d.audio_ready {
        sync::gravar_memory_cue(&mut d.cues, pos);
        st.salvar_cues(i);
    }
}

/// Linha da lista de cues. Deck 2 lista A..H: com cue gravado, pula e toca;
/// vazio, grava na posição (quantizada com Q). Deck 1 lista memory cues em
/// ordem de tempo: define o cue e para nele.
pub fn cue_linha(st: &mut AppState, i: usize, linha: usize, pos: f64) -> Option<AudioCommand> {
    let q = st.quantize[i];
    let d = deck(st, i);
    let sr = f64::from(d.audio_buf.as_ref()?.sample_rate);
    let n = i as u8 + 1;
    if i == 1 {
        let slot = linha as u32 + 1;
        if let Some(c) = d.cues.iter().find(|c| c.hot_cue == slot) {
            let frame = (c.start_time_s * sr).round() as u64;
            return Some(AudioCommand::JumpAndPlay { deck: n, frame });
        }
        let pos = loops::quantizar_s(&d.beats, pos, q);
        sync::gravar_hot_cue(&mut d.cues, slot, pos);
        st.salvar_cues(i);
        None
    } else {
        let mut mem: Vec<f64> =
            d.cues.iter().filter(|c| c.hot_cue == 0).map(|c| c.start_time_s).collect();
        mem.sort_by(f64::total_cmp);
        let frame = (mem.get(linha)? * sr).round() as u64;
        Some(AudioCommand::SetCueAndStandby { deck: n, frame })
    }
}
