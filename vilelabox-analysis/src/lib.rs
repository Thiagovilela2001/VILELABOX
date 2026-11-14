//! `vilelabox-analysis`: pipeline de análise musical (P4.2 / Gate G4).
//!
//! Reúne as etapas A2.1–A2.5 sobre as primitivas de `vilelabox-dsp`, garantindo:
//! - determinismo e invariância ao tamanho do bloco de alimentação (inclusive
//!   tamanhos primos/ímpares como `513` vs `4096`);
//! - tolerâncias explícitas registradas por métrica (`ToleranciasAnalise`);
//! - persistência e cache por `(sha256, VERSAO_PIPELINE, config)` em
//!   `vilelabox-storage`, respeitando grades bloqueadas (`bloqueada = true`);
//! - suporte a cancelamento cooperativo (`TokenCancelamento`) e rejeição limpa
//!   de arquivos ausentes, truncados ou corrompidos sem corromper o banco.

use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use thiserror::Error;
use vilelabox_audio::decode_audio;
use vilelabox_dsp::balib::{AnalisadorDeBatidas, BatidaGrade, Frase};
use vilelabox_dsp::frases::{
    detectar_transicoes_energia, detectar_transicoes_razao, normalizar_e_agrupar_compassos,
};
use vilelabox_dsp::tonalidade::{Tonalidade, BINS};
use vilelabox_storage::{Database, ErroStorage, NovaFaixa};
use vilelabox_types::{
    AudioIdentity, Beat, BeatGrid, Bpm, Frame, KeyScale, MusicalKey, Phrase, PhraseKind,
    SampleRate, Seconds, TempoChange, TrackId,
};

/// Versão do pipeline de análise para chave de cache em `analysis_results`.
pub const VERSAO_PIPELINE: u32 = 1;

/// Nome da etapa persistida em `analysis_results` (`vilelabox-storage`).
pub const ETAPA_PIPELINE: &str = "g4_pipeline";

/// Tolerâncias registradas por métrica para validação diferencial (Gate G4).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ToleranciasAnalise {
    /// Desvio máximo de BPM permitido (ex.: `0.5` BPM).
    pub bpm_max_desvio: f64,
    /// Desvio máximo na posição de cada batida em frames (ex.: `512` frames a 44,1 kHz ≈ 11,6 ms).
    pub batida_max_desvio_frames: u64,
    /// Desvio máximo de fronteira de frase em compassos.
    pub frase_max_desvio_compassos: u32,
}

impl Default for ToleranciasAnalise {
    fn default() -> Self {
        Self {
            bpm_max_desvio: 0.5,
            batida_max_desvio_frames: 512,
            frase_max_desvio_compassos: 1,
        }
    }
}

/// Token de cancelamento cooperativo compartilhável entre threads.
#[derive(Debug, Clone, Default)]
pub struct TokenCancelamento {
    cancelado: Arc<AtomicBool>,
}

impl TokenCancelamento {
    pub fn novo() -> Self {
        Self::default()
    }

    pub fn cancelar(&self) {
        self.cancelado.store(true, Ordering::SeqCst);
    }

    pub fn foi_cancelado(&self) -> bool {
        self.cancelado.load(Ordering::SeqCst)
    }
}

/// Erros do pipeline de análise (Gate G4).
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ErroAnalise {
    #[error("análise cancelada pelo chamador")]
    Cancelado,
    #[error("entrada de áudio inválida: {0}")]
    EntradaInvalida(String),
    #[error("erro de armazenamento: {0}")]
    Armazenamento(String),
}

impl From<ErroStorage> for ErroAnalise {
    fn from(e: ErroStorage) -> Self {
        Self::Armazenamento(e.to_string())
    }
}

/// Configuração de execução do pipeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConfigAnalise {
    /// Modo do estimador de BPM (`0` = modo 0 uniforme, `1` = modo 1 dinâmico).
    pub modo_bpm: u8,
    /// Tamanho do bloco interno de alimentação em frames.
    pub tamanho_bloco: usize,
}

impl Default for ConfigAnalise {
    fn default() -> Self {
        Self {
            modo_bpm: 0,
            tamanho_bloco: 4096,
        }
    }
}

impl ConfigAnalise {
    /// Chave de configuração para cache em `analysis_results` (invariante ao
    /// `tamanho_bloco`).
    #[must_use]
    pub fn chave_cache(self) -> String {
        format!("modo_bpm={}", self.modo_bpm)
    }
}

/// Resultado consolidado do pipeline A2.1–A2.5.
#[derive(Debug, Clone, PartialEq)]
pub struct ResultadoAnalise {
    pub taxa: SampleRate,
    pub duracao: Seconds,
    pub total_frames: u64,
    pub grade: Option<BeatGrid>,
    pub frases: Vec<Phrase>,
    pub tonalidade: Option<MusicalKey>,
    pub energia_media: f32,
    pub envelope_waveform: Vec<u8>,
}

/// Executa o pipeline completo sobre dois canais PCM `i16` (`l` e `r`).
///
/// Garante invariância ao `config.tamanho_bloco`: todos os sub-estágios operam
/// sobre janelas internas próprias (`Alimentador` em blocos de `taxa/75`,
/// `RefinadorDeEnvelope` em blocos de 128 e `AjusteDeBatidas` decimado por 4),
/// produzindo o mesmo `ResultadoAnalise` independentemente do fatiamento de
/// entrada (ex.: blocos de 513 vs 4096 frames).
pub fn analisar_faixa(
    taxa_hz: u32,
    l: &[i16],
    r: &[i16],
    config: ConfigAnalise,
    cancelamento: &TokenCancelamento,
) -> Result<ResultadoAnalise, ErroAnalise> {
    if cancelamento.foi_cancelado() {
        return Err(ErroAnalise::Cancelado);
    }
    let taxa = SampleRate::new(taxa_hz)
        .map_err(|_| ErroAnalise::EntradaInvalida("taxa de amostragem nula".into()))?;
    if l.len() != r.len() {
        return Err(ErroAnalise::EntradaInvalida(
            "canais L e R com comprimentos distintos".into(),
        ));
    }
    if l.is_empty() {
        return Err(ErroAnalise::EntradaInvalida("áudio vazio".into()));
    }
    let n_frames = l.len();
    let duracao = Frame::new(n_frames as i64).to_seconds(taxa);
    let bloco = config.tamanho_bloco.max(1);

    let mut analisador = AnalisadorDeBatidas::novo(taxa_hz, n_frames as i32, Frase::PADRAO)
        .ok_or_else(|| {
            ErroAnalise::EntradaInvalida("taxa incompatível com banco de filtros".into())
        })?;

    for (cl, cr) in l.chunks(bloco).zip(r.chunks(bloco)) {
        if cancelamento.foi_cancelado() {
            return Err(ErroAnalise::Cancelado);
        }
        analisador.alimentar(cl, cr);
    }

    if cancelamento.foi_cancelado() {
        return Err(ErroAnalise::Cancelado);
    }

    let (bpm_val, batidas_raw) = analisador.fechar_modo(config.modo_bpm);
    let grade = construir_beatgrid(taxa, bpm_val, &batidas_raw);
    let frases = if let Some(ref g) = grade {
        construir_frases(l, r, g, n_frames as u64)
    } else {
        Vec::new()
    };

    if cancelamento.foi_cancelado() {
        return Err(ErroAnalise::Cancelado);
    }

    let tonalidade = estimar_tonalidade(taxa_hz, bpm_val, l, r);
    let (energia_media, envelope_waveform) = calcular_energia_e_waveform(l, r);

    Ok(ResultadoAnalise {
        taxa,
        duracao,
        total_frames: n_frames as u64,
        grade,
        frases,
        tonalidade,
        energia_media,
        envelope_waveform,
    })
}

/// Decodifica um arquivo de áudio do disco, calcula o `SHA-256` em streaming,
/// consulta o cache em `analysis_results` por `(sha256, VERSAO_PIPELINE, config)`
/// e, se necessário, executa `analisar_faixa` e persiste identidade, grade,
/// frases, tonalidade e envelope no `Database` (`vilelabox-storage`).
///
/// Segurança de cancelamento e corrupção (Gate G4):
/// - se `cancelamento` for acionado antes do início da persistência, ou se o
///   arquivo estiver ausente/corrompido/truncado, retorna `Err` sem alterar
///   nenhum registro preexistente da faixa no `Database`;
/// - se a faixa já possuir uma `BeatGrid` com `bloqueada == true`, preserva a
///   grade bloqueada intacta no banco.
pub fn analisar_e_persistir_arquivo(
    db: &Database,
    track_id: TrackId,
    caminho: &Path,
    config: ConfigAnalise,
    cancelamento: &TokenCancelamento,
) -> Result<ResultadoAnalise, ErroAnalise> {
    if cancelamento.foi_cancelado() {
        return Err(ErroAnalise::Cancelado);
    }
    let sha256 = calcular_sha256_streaming(caminho, cancelamento)?;
    let chave_cfg = config.chave_cache();

    if let Some(blob) = db.analise(track_id, ETAPA_PIPELINE, VERSAO_PIPELINE, &chave_cfg)? {
        if let Some(cached) = deserializar_cache(&blob, &sha256, db, track_id)? {
            return Ok(cached);
        }
    }

    if cancelamento.foi_cancelado() {
        return Err(ErroAnalise::Cancelado);
    }

    let audio = decode_audio(caminho).map_err(|e| ErroAnalise::EntradaInvalida(e.to_string()))?;
    if audio.total_frames == 0 || audio.channels == 0 {
        return Err(ErroAnalise::EntradaInvalida(
            "áudio decodificado vazio".into(),
        ));
    }

    let n_frames = audio.total_frames as usize;
    let mut l = Vec::with_capacity(n_frames);
    let mut r = Vec::with_capacity(n_frames);
    let ch = usize::from(audio.channels);
    for f in 0..n_frames {
        let base = f * ch;
        let sl = audio.samples.get(base).copied().unwrap_or(0.0);
        let sr = if ch >= 2 {
            audio.samples.get(base + 1).copied().unwrap_or(sl)
        } else {
            sl
        };
        l.push((sl * 32767.0).round().clamp(-32768.0, 32767.0) as i16);
        r.push((sr * 32767.0).round().clamp(-32768.0, 32767.0) as i16);
    }

    let resultado = analisar_faixa(audio.sample_rate, &l, &r, config, cancelamento)?;

    if cancelamento.foi_cancelado() {
        return Err(ErroAnalise::Cancelado);
    }

    let identidade = AudioIdentity {
        sha256: Some(sha256),
        duration_frames: Frame::new(audio.total_frames as i64),
        sample_rate: resultado.taxa,
        channels: audio.channels,
    };

    persistir_resultado_no_banco(db, track_id, &identidade, config, &resultado)?;
    Ok(resultado)
}

/// Persiste um `ResultadoAnalise` e sua `AudioIdentity` no `Database`,
/// respeitando grades bloqueadas (`bloqueada == true`) e gravando o cache em
/// `analysis_results`.
pub fn persistir_resultado_no_banco(
    db: &Database,
    track_id: TrackId,
    identidade: &AudioIdentity,
    config: ConfigAnalise,
    resultado: &ResultadoAnalise,
) -> Result<(), ErroAnalise> {
    db.atualizar_identidade(track_id, identidade)?;

    let grade_bloqueada = db.grade(track_id)?.is_some_and(|row| row.bloqueada);
    if !grade_bloqueada {
        if let Some(ref grade) = resultado.grade {
            db.salvar_grade(track_id, grade, false)?;
        }
    }

    db.substituir_frases(track_id, &resultado.frases)?;

    if let Some(faixa) = db.faixa(track_id)? {
        let mut nova = NovaFaixa {
            metadados: faixa.metadados,
            identidade: Some(identidade.clone()),
            bpm: faixa.bpm,
            tom: faixa.tom,
            estrelas: faixa.estrelas,
        };
        if !grade_bloqueada {
            if let Some(bpm) = resultado.grade.as_ref().and_then(BeatGrid::primary_bpm) {
                nova.bpm = Some(bpm);
            }
        }
        if let Some(ton) = resultado.tonalidade {
            nova.tom = Some(ton.to_camelot().to_string());
        }
        db.atualizar_faixa(track_id, &nova)?;
    }

    let sha_hex = identidade.sha256.as_deref().unwrap_or("");
    let blob = serializar_cache(sha_hex, resultado);
    db.salvar_analise(
        track_id,
        ETAPA_PIPELINE,
        VERSAO_PIPELINE,
        &config.chave_cache(),
        &blob,
    )?;
    Ok(())
}

fn calcular_sha256_streaming(
    caminho: &Path,
    cancelamento: &TokenCancelamento,
) -> Result<String, ErroAnalise> {
    let mut arq = File::open(caminho).map_err(|e| ErroAnalise::EntradaInvalida(e.to_string()))?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        if cancelamento.foi_cancelado() {
            return Err(ErroAnalise::Cancelado);
        }
        let n = arq
            .read(&mut buf)
            .map_err(|e| ErroAnalise::EntradaInvalida(e.to_string()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    let bytes: [u8; 32] = hasher.finalize().into();
    let mut hex = String::with_capacity(64);
    for b in bytes {
        use std::fmt::Write;
        let _ = write!(&mut hex, "{b:02x}");
    }
    Ok(hex)
}

fn serializar_cache(sha256_hex: &str, res: &ResultadoAnalise) -> Vec<u8> {
    let sha_bytes = sha256_hex.as_bytes();
    let mut out = Vec::with_capacity(68 + sha_bytes.len() + res.envelope_waveform.len());
    out.extend_from_slice(&(sha_bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(sha_bytes);
    out.extend_from_slice(&res.taxa.get().to_le_bytes());
    out.extend_from_slice(&res.total_frames.to_le_bytes());
    out.extend_from_slice(&res.energia_media.to_le_bytes());
    match res.tonalidade {
        Some(k) => {
            out.push(1);
            out.push(k.root_pitch_class());
            out.push(match k.scale() {
                KeyScale::Major => 0,
                KeyScale::Minor => 1,
            });
        }
        None => {
            out.push(0);
            out.push(0);
            out.push(0);
        }
    }
    let env_len = res.envelope_waveform.len() as u32;
    out.extend_from_slice(&env_len.to_le_bytes());
    out.extend_from_slice(&res.envelope_waveform);
    out
}

fn deserializar_cache(
    blob: &[u8],
    sha256_esperado: &str,
    db: &Database,
    track_id: TrackId,
) -> Result<Option<ResultadoAnalise>, ErroAnalise> {
    if blob.len() < 4 {
        return Ok(None);
    }
    let sha_len = u32::from_le_bytes(blob[0..4].try_into().unwrap()) as usize;
    let base = 4 + sha_len;
    if blob.len() < base + 4 + 8 + 4 + 3 + 4 {
        return Ok(None);
    }
    if &blob[4..base] != sha256_esperado.as_bytes() {
        return Ok(None);
    }
    let taxa_hz = u32::from_le_bytes(blob[base..base + 4].try_into().unwrap());
    let Ok(taxa) = SampleRate::new(taxa_hz) else {
        return Ok(None);
    };
    let total_frames = u64::from_le_bytes(blob[base + 4..base + 12].try_into().unwrap());
    let energia_media = f32::from_le_bytes(blob[base + 12..base + 16].try_into().unwrap());
    let tonalidade = if blob[base + 16] == 1 {
        let escala = if blob[base + 18] == 1 {
            KeyScale::Minor
        } else {
            KeyScale::Major
        };
        Some(MusicalKey::new(blob[base + 17], escala))
    } else {
        None
    };
    let env_len = u32::from_le_bytes(blob[base + 19..base + 23].try_into().unwrap()) as usize;
    if blob.len() < base + 23 + env_len {
        return Ok(None);
    }
    let envelope_waveform = blob[base + 23..base + 23 + env_len].to_vec();
    let grade = db.grade(track_id)?.map(|row| row.grade);
    let frases = db.frases(track_id)?;
    let duracao = Frame::new(total_frames as i64).to_seconds(taxa);
    Ok(Some(ResultadoAnalise {
        taxa,
        duracao,
        total_frames,
        grade,
        frases,
        tonalidade,
        energia_media,
        envelope_waveform,
    }))
}

fn construir_beatgrid(
    taxa: SampleRate,
    bpm_val: f32,
    batidas_raw: &[BatidaGrade],
) -> Option<BeatGrid> {
    if bpm_val <= 0.0 || batidas_raw.is_empty() {
        return None;
    }
    let bpm_principal = Bpm::new(f64::from(bpm_val)).ok()?;
    let mut beats = Vec::with_capacity(batidas_raw.len());
    let mut tempo_dinamico = Vec::new();
    let mut downbeat = None;
    let mut ultimo_pos: Option<u64> = None;
    let mut ultimo_intervalo: Option<f32> = None;
    let mut variou_intervalo = false;

    for b in batidas_raw {
        if b.pos < 0 {
            continue;
        }
        let pos = b.pos as u64;
        if let Some(ant) = ultimo_pos {
            if pos <= ant {
                continue;
            }
        }
        let beat_num = (b.numero.clamp(1, 4)) as u32;
        let frame = Frame::new(pos as i64);
        if downbeat.is_none() && beat_num == 1 {
            downbeat = Some(frame);
        }
        beats.push(Beat {
            position: frame,
            beat_num,
        });
        if b.intervalo > 0.0 {
            let mudou = match ultimo_intervalo {
                Some(ant) => {
                    if (b.intervalo - ant).abs() > 0.05 {
                        variou_intervalo = true;
                        true
                    } else {
                        false
                    }
                }
                None => true,
            };
            if mudou {
                // Conversão de intervalo (ms) para BPM × 100 conforme 0x142619d60:
                // (int)(6000000.0f / intervalo + 0.5f) / 100.0
                let bpm_x100 = (6_000_000.0f32 / b.intervalo + 0.5) as i32;
                if let Ok(bpm_local) = Bpm::new(f64::from(bpm_x100) / 100.0) {
                    tempo_dinamico.push(TempoChange {
                        position: frame,
                        bpm: bpm_local,
                    });
                    ultimo_intervalo = Some(b.intervalo);
                }
            }
        }
        ultimo_pos = Some(pos);
    }
    if beats.is_empty() {
        return None;
    }
    let tempo = if variou_intervalo && !tempo_dinamico.is_empty() {
        tempo_dinamico
    } else {
        vec![TempoChange {
            position: beats[0].position,
            bpm: bpm_principal,
        }]
    };
    BeatGrid::new(taxa, beats, tempo, downbeat).ok()
}

/// Segmenta e rotula frases a partir da grade de batidas usando as primitivas
/// portadas de `vilelabox_dsp::frases`:
/// - `detectar_transicoes_energia` (`0x1412e7460`, limiar `1.2×`, supressão
///   `<= 750.0` e `<= 0.09`);
/// - `detectar_transicoes_razao` (`0x1412e8de0`, limiares `1.2` na subida e
///   `1.21` na descida com piso `0.003`);
/// - `normalizar_e_agrupar_compassos` (`0x1412d0aa0` / `0x1412e93f0`).
///
/// ADAPTAÇÃO: enquanto o orquestrador de topo `0x1412f7420` (73 chamadas
/// internas, cf. `spec/rekordbox/evidence/p4-1-analise-batida.md`) permanece
/// parcialmente aberto, a fusão dos cortes em múltiplos de 4 compassos e a
/// atribuição dos rótulos `PhraseKind` (`Intro`, `Up`, `Down`, `Chorus`,
/// `Verse`, `Outro`) a partir do perfil normalizado por compasso constituem a
/// camada de colagem do `vilelabox-analysis`.
fn construir_frases(l: &[i16], r: &[i16], grade: &BeatGrid, total_frames: u64) -> Vec<Phrase> {
    let beats = grade.beats();
    if beats.len() < 8 {
        return Vec::new();
    }
    let primeiro_forte = beats.iter().position(|b| b.beat_num == 1).unwrap_or(0);
    let mut energia_batida = Vec::with_capacity(beats.len());
    for i in 0..beats.len() {
        let ini = (beats[i].position.get() as usize).min(l.len());
        let fim = if i + 1 < beats.len() {
            (beats[i + 1].position.get() as usize).min(l.len())
        } else {
            l.len()
        };
        if fim <= ini {
            energia_batida.push(0.0);
        } else {
            let mut soma = 0.0f64;
            for k in ini..fim {
                let m = (f64::from(l[k]) + f64::from(r[k])) * 0.5;
                soma += m.abs();
            }
            energia_batida.push(soma / (fim - ini) as f64);
        }
    }

    let (_, marcas_energia) = detectar_transicoes_energia(&energia_batida, primeiro_forte);
    let (_, marcas_razao) = detectar_transicoes_razao(&energia_batida, primeiro_forte);
    let mut norm = energia_batida;
    let medias_comp = normalizar_e_agrupar_compassos(&mut norm, primeiro_forte);
    if medias_comp.is_empty() {
        return Vec::new();
    }

    let mut cortes_comp = vec![0usize];
    let n_marcas = marcas_energia.len().min(marcas_razao.len());
    for ci in 1..n_marcas {
        let transicao = marcas_energia[ci] != 0 || marcas_razao[ci] != 0;
        if transicao && ci.saturating_sub(*cortes_comp.last().unwrap_or(&0)) >= 4 {
            cortes_comp.push(ci);
        }
    }
    if cortes_comp.len() == 1 && medias_comp.len() >= 8 {
        let mut c = 4usize;
        while c < medias_comp.len() {
            cortes_comp.push(c);
            c += 8;
        }
    }

    let mut frases = Vec::new();
    for (idx, &c_ini) in cortes_comp.iter().enumerate() {
        let c_fim = cortes_comp
            .get(idx + 1)
            .copied()
            .unwrap_or(medias_comp.len());
        let b_ini_idx = primeiro_forte + c_ini * 4;
        let b_fim_idx = primeiro_forte + c_fim * 4;
        if b_ini_idx >= beats.len() {
            break;
        }
        let start = beats[b_ini_idx].position;
        let end = if b_fim_idx < beats.len() {
            beats[b_fim_idx].position
        } else {
            Frame::new(total_frames as i64)
        };
        if start >= end {
            continue;
        }
        let fatia = &medias_comp[c_ini..c_fim];
        let media_seg: f64 = fatia.iter().sum::<f64>() / (fatia.len().max(1) as f64);
        let tendencia = fatia.last().copied().unwrap_or(media_seg)
            - fatia.first().copied().unwrap_or(media_seg);
        let kind = if idx == 0 {
            PhraseKind::Intro
        } else if idx + 1 == cortes_comp.len() && media_seg < 0.35 {
            PhraseKind::Outro
        } else if media_seg >= 0.65 {
            PhraseKind::Chorus
        } else if tendencia > 0.25 {
            PhraseKind::Up
        } else if tendencia < -0.25 {
            PhraseKind::Down
        } else {
            PhraseKind::Verse
        };
        let n_beats = ((c_fim - c_ini) * 4) as u32;
        if let Ok(p) = Phrase::new((idx + 1) as u32, start, end, kind, n_beats) {
            frases.push(p);
        }
    }
    frases
}

/// Estima a tonalidade usando o banco de 72 bins semitonais portado em
/// `vilelabox_dsp::tonalidade::Tonalidade` (`0x14114c4f0` / `0x14114d120`, A0 =
/// 27,5 Hz até G#6) alimentado com o BPM detectado da faixa.
///
/// ADAPTAÇÃO: enquanto a etapa final de decisão de `AnalyzeJobKey`
/// (`0x141133000`–`0x141134400`, registrada como `DESCONHECIDO` em
/// `spec/rekordbox/evidence/p4-1-analise-batida.md`) não é aberta no binário,
/// os 72 bins (6 oitavas × 12 semitons) são dobrados nas 12 classes de altura
/// (cromagrama a partir de A0 = classe 9) e a modalidade maior/menor é decidida
/// pela comparação entre a terça maior (`+4` semitons) e a terça menor (`+3`
/// semitons) da fundamental dominante.
fn estimar_tonalidade(taxa_hz: u32, bpm_val: f32, l: &[i16], r: &[i16]) -> Option<MusicalKey> {
    let passo_dec = 8usize;
    let fs_dec = f64::from(taxa_hz) / passo_dec as f64;
    let mut ton = Tonalidade::novo(fs_dec)?;
    // Limita a janela a ~15 s para manter o custo constante.
    let max_frames = (taxa_hz as usize).saturating_mul(15).min(l.len());
    let mut decimado = Vec::with_capacity(max_frames / passo_dec);
    let mut i = 0usize;
    let escala_bloco = 0.5 / (32768.0 * passo_dec as f64);
    while i + passo_dec <= max_frames {
        let mut soma = 0.0f64;
        for k in 0..passo_dec {
            soma += f64::from(l[i + k]) + f64::from(r[i + k]);
        }
        decimado.push(soma * escala_bloco);
        i += passo_dec;
    }
    let bpm_ref = if bpm_val > 0.0 {
        f64::from(bpm_val)
    } else {
        120.0
    };
    let q = ton.processar(&decimado, bpm_ref, 0, 1, fs_dec, 2)?;
    if q.quadros == 0 {
        return None;
    }
    // Bins 0..72 começam em A0 (27,5 Hz => pitch class 9 = A).
    let mut croma = [0.0f64; 12];
    for bin in 0..BINS {
        let linha = &q.matriz[bin * q.quadros..(bin + 1) * q.quadros];
        let soma: f64 = linha.iter().sum();
        let pc = (bin + 9) % 12;
        croma[pc] += soma;
    }
    let total_croma: f64 = croma.iter().sum();
    if total_croma <= 1e-12 {
        return None;
    }
    let (melhor_pc, _) = croma
        .iter()
        .copied()
        .enumerate()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))?;
    let terca_maior = croma[(melhor_pc + 4) % 12];
    let terca_menor = croma[(melhor_pc + 3) % 12];
    let escala = if terca_menor > terca_maior {
        KeyScale::Minor
    } else {
        KeyScale::Major
    };
    Some(MusicalKey::new(melhor_pc as u8, escala))
}

/// Calcula a energia RMS global e o envelope compacto de waveform (`u8`) para
/// persistência em `analysis_results`.
///
/// ADAPTAÇÃO: resumo de pico por coluna (`<= 400` colunas normalizadas em
/// `0..=255`) e RMS global da faixa para cache rápido da UI.
fn calcular_energia_e_waveform(l: &[i16], r: &[i16]) -> (f32, Vec<u8>) {
    if l.is_empty() {
        return (0.0, Vec::new());
    }
    let mut soma_quad = 0.0f64;
    let passo = (l.len() / 400).max(1);
    let mut env = Vec::with_capacity(l.len() / passo + 1);
    for (cl, cr) in l.chunks(passo).zip(r.chunks(passo)) {
        let mut pico = 0i32;
        for (&sl, &sr) in cl.iter().zip(cr.iter()) {
            let m = (i32::from(sl) + i32::from(sr)) / 2;
            let a = m.abs();
            if a > pico {
                pico = a;
            }
            let mf = f64::from(m) / 32768.0;
            soma_quad += mf * mf;
        }
        env.push(((pico * 255) / 32768).clamp(0, 255) as u8);
    }
    let rms = (soma_quad / l.len() as f64).sqrt() as f32;
    (rms, env)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gerar_sinal_120bpm(taxa: u32, duracao_s: u32) -> (Vec<i16>, Vec<i16>) {
        let total = (taxa * duracao_s) as usize;
        let mut l = vec![0i16; total];
        let mut r = vec![0i16; total];
        let passo = (taxa / 2) as usize;
        for b in 0..(duracao_s * 2) as usize {
            let amp = if b % 4 == 0 { 26_000.0 } else { 14_000.0 };
            let ini = b * passo;
            for k in 0..512usize {
                if ini + k < total {
                    // 440 Hz (A4) para testar também a detecção de tonalidade A.
                    let s = (amp
                        * (2.0 * std::f64::consts::PI * 440.0 * k as f64 / f64::from(taxa)).sin())
                        as i16;
                    l[ini + k] = s;
                    r[ini + k] = s;
                }
            }
        }
        (l, r)
    }

    fn escrever_wav_pcm16_estereo(caminho: &Path, taxa: u32, l: &[i16], r: &[i16]) {
        let n_frames = l.len().min(r.len());
        let data_bytes = (n_frames * 4) as u32;
        let riff_size = 36 + data_bytes;
        let byte_rate = taxa * 4;
        let mut buf = Vec::with_capacity(44 + data_bytes as usize);
        buf.extend_from_slice(b"RIFF");
        buf.extend_from_slice(&riff_size.to_le_bytes());
        buf.extend_from_slice(b"WAVEfmt ");
        buf.extend_from_slice(&16u32.to_le_bytes());
        buf.extend_from_slice(&1u16.to_le_bytes()); // PCM
        buf.extend_from_slice(&2u16.to_le_bytes()); // 2 canais
        buf.extend_from_slice(&taxa.to_le_bytes());
        buf.extend_from_slice(&byte_rate.to_le_bytes());
        buf.extend_from_slice(&4u16.to_le_bytes()); // block align
        buf.extend_from_slice(&16u16.to_le_bytes()); // bits per sample
        buf.extend_from_slice(b"data");
        buf.extend_from_slice(&data_bytes.to_le_bytes());
        for i in 0..n_frames {
            buf.extend_from_slice(&l[i].to_le_bytes());
            buf.extend_from_slice(&r[i].to_le_bytes());
        }
        std::fs::write(caminho, buf).unwrap();
    }

    #[test]
    fn gate_g4_invariancia_de_bloco_e_determinismo() {
        let taxa = 44_100;
        let (l, r) = gerar_sinal_120bpm(taxa, 12);
        let cancel = TokenCancelamento::novo();

        let res_513 = analisar_faixa(
            taxa,
            &l,
            &r,
            ConfigAnalise {
                modo_bpm: 0,
                tamanho_bloco: 513,
            },
            &cancel,
        )
        .unwrap();

        let res_512 = analisar_faixa(
            taxa,
            &l,
            &r,
            ConfigAnalise {
                modo_bpm: 0,
                tamanho_bloco: 512,
            },
            &cancel,
        )
        .unwrap();

        let res_4096 = analisar_faixa(
            taxa,
            &l,
            &r,
            ConfigAnalise {
                modo_bpm: 0,
                tamanho_bloco: 4096,
            },
            &cancel,
        )
        .unwrap();

        assert_eq!(res_513, res_4096);
        assert_eq!(res_512, res_4096);
        let tol = ToleranciasAnalise::default();
        let bpm = res_513.grade.as_ref().unwrap().primary_bpm().unwrap().get();
        assert!((bpm - 120.0).abs() <= tol.bpm_max_desvio, "bpm={bpm}");
        assert!(!res_513.frases.is_empty());
        assert!(res_513.tonalidade.is_some());
        assert!(!res_513.envelope_waveform.is_empty());
    }

    #[test]
    fn gate_g4_persistencia_cache_e_grade_bloqueada() {
        use vilelabox_types::TrackMetadata;

        let taxa = 44_100;
        let (l, r) = gerar_sinal_120bpm(taxa, 12);
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("target")
            .join(format!("vilelabox_g4_test_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let wav_path = dir.join("faixa_120bpm.wav");
        escrever_wav_pcm16_estereo(&wav_path, taxa, &l, &r);

        let db = Database::open_in_memory().unwrap();
        let nova = NovaFaixa {
            metadados: TrackMetadata {
                title: Some("Teste 120 BPM".into()),
                ..Default::default()
            },
            ..Default::default()
        };
        let track_id = db.inserir_faixa(&nova).unwrap();
        db.adicionar_localizacao(track_id, &wav_path.to_string_lossy())
            .unwrap();

        let cancel = TokenCancelamento::novo();
        let cfg = ConfigAnalise::default();
        let res1 = analisar_e_persistir_arquivo(&db, track_id, &wav_path, cfg, &cancel).unwrap();

        // Verifica que identidade, grade, frases, tonalidade e cache foram persistidos.
        let faixa_salva = db.faixa(track_id).unwrap().expect("faixa salva");
        let id_salva = faixa_salva.identidade.expect("identidade salva");
        assert_eq!(id_salva.duration_frames.get(), l.len() as i64);
        assert_eq!(id_salva.sample_rate.get(), taxa);
        assert_eq!(id_salva.channels, 2);

        let grade_row = db.grade(track_id).unwrap().expect("grade salva");
        assert!(!grade_row.bloqueada);
        assert_eq!(Some(grade_row.grade), res1.grade);
        assert_eq!(db.frases(track_id).unwrap(), res1.frases);
        assert!(faixa_salva.bpm.is_some());
        assert!(faixa_salva.tom.is_some());
        assert!(db
            .analise(
                track_id,
                ETAPA_PIPELINE,
                VERSAO_PIPELINE,
                &cfg.chave_cache()
            )
            .unwrap()
            .is_some());

        // Segunda chamada deve acertar o cache e retornar o mesmo ResultadoAnalise.
        let res_cache =
            analisar_e_persistir_arquivo(&db, track_id, &wav_path, cfg, &cancel).unwrap();
        assert_eq!(res1, res_cache);

        // Bloqueia a grade com um BPM customizado e reexecuta (em modo 1 para forçar reanálise):
        // a grade bloqueada no banco deve permanecer intacta.
        let bpm_travado = Bpm::new(128.0).unwrap();
        let grade_travada = BeatGrid::new(
            SampleRate::new(taxa).unwrap(),
            vec![Beat {
                position: Frame::new(0),
                beat_num: 1,
            }],
            vec![TempoChange {
                position: Frame::new(0),
                bpm: bpm_travado,
            }],
            Some(Frame::new(0)),
        )
        .unwrap();
        db.salvar_grade(track_id, &grade_travada, true).unwrap();

        let _ = analisar_e_persistir_arquivo(
            &db,
            track_id,
            &wav_path,
            ConfigAnalise {
                modo_bpm: 1,
                tamanho_bloco: 2048,
            },
            &cancel,
        )
        .unwrap();

        let grade_apos = db.grade(track_id).unwrap().unwrap();
        assert!(grade_apos.bloqueada);
        assert_eq!(grade_apos.grade, grade_travada);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn gate_g4_cancelamento_e_arquivo_corrompido_nao_corrompem_banco() {
        let taxa = 44_100;
        let (l, r) = gerar_sinal_120bpm(taxa, 12);
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("target")
            .join(format!("vilelabox_g4_corrupt_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let wav_ok = dir.join("ok.wav");
        let wav_corrompido = dir.join("corrompido.wav");
        escrever_wav_pcm16_estereo(&wav_ok, taxa, &l, &r);
        std::fs::write(
            &wav_corrompido,
            b"RIFF\x10\x00\x00\x00WAVEfmt lixo_truncado",
        )
        .unwrap();

        let db = Database::open_in_memory().unwrap();
        let track_id = db.inserir_faixa(&NovaFaixa::default()).unwrap();
        db.adicionar_localizacao(track_id, &wav_ok.to_string_lossy())
            .unwrap();

        let ativo = TokenCancelamento::novo();
        let res_ok =
            analisar_e_persistir_arquivo(&db, track_id, &wav_ok, ConfigAnalise::default(), &ativo)
                .unwrap();
        let grade_original = db.grade(track_id).unwrap();
        let frases_originais = db.frases(track_id).unwrap();
        let faixa_original = db.faixa(track_id).unwrap();

        // 1. Execução cancelada não altera dados existentes no banco.
        let cancelado = TokenCancelamento::novo();
        cancelado.cancelar();
        let err_cancel = analisar_e_persistir_arquivo(
            &db,
            track_id,
            &wav_ok,
            ConfigAnalise {
                modo_bpm: 1,
                tamanho_bloco: 1024,
            },
            &cancelado,
        );
        assert_eq!(err_cancel, Err(ErroAnalise::Cancelado));
        assert_eq!(db.grade(track_id).unwrap(), grade_original);
        assert_eq!(db.frases(track_id).unwrap(), frases_originais);
        assert_eq!(db.faixa(track_id).unwrap(), faixa_original);

        // 2. Arquivo corrompido/truncado retorna erro limpo sem corromper o banco.
        let err_corrupt = analisar_e_persistir_arquivo(
            &db,
            track_id,
            &wav_corrompido,
            ConfigAnalise {
                modo_bpm: 1,
                tamanho_bloco: 1024,
            },
            &ativo,
        );
        assert!(matches!(err_corrupt, Err(ErroAnalise::EntradaInvalida(_))));
        assert_eq!(db.grade(track_id).unwrap(), grade_original);
        assert_eq!(db.frases(track_id).unwrap(), frases_originais);
        assert_eq!(db.faixa(track_id).unwrap(), faixa_original);
        assert_eq!(db.grade(track_id).unwrap().map(|r| r.grade), res_ok.grade);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn gate_g4_cancelamento_e_entradas_invalidas() {
        let cancel = TokenCancelamento::novo();
        cancel.cancelar();
        let err = analisar_faixa(
            44_100,
            &[0; 1024],
            &[0; 1024],
            ConfigAnalise::default(),
            &cancel,
        );
        assert_eq!(err, Err(ErroAnalise::Cancelado));

        let ativo = TokenCancelamento::novo();
        assert!(matches!(
            analisar_faixa(0, &[0; 128], &[0; 128], ConfigAnalise::default(), &ativo),
            Err(ErroAnalise::EntradaInvalida(_))
        ));
        assert!(matches!(
            analisar_faixa(44_100, &[], &[], ConfigAnalise::default(), &ativo),
            Err(ErroAnalise::EntradaInvalida(_))
        ));
        assert!(matches!(
            analisar_faixa(44_100, &[0; 10], &[0; 20], ConfigAnalise::default(), &ativo),
            Err(ErroAnalise::EntradaInvalida(_))
        ));
    }
}
