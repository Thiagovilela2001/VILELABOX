use std::path::Path;
use vilelabox_dsp::balib::{AnalisadorDeBatidas, Frase};

fn ler_wav_i16(caminho: &Path) -> (u32, Vec<i16>, Vec<i16>) {
    let bytes = std::fs::read(caminho).expect("falha ao ler WAV");
    assert!(&bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE");
    let mut pos = 12usize;
    let mut taxa = 44_100u32;
    let mut canais = 2u16;
    let mut pcm: &[u8] = &[];
    while pos + 8 <= bytes.len() {
        let id = &bytes[pos..pos + 4];
        let sz = u32::from_le_bytes(bytes[pos + 4..pos + 8].try_into().unwrap()) as usize;
        let dados = &bytes[pos + 8..(pos + 8 + sz).min(bytes.len())];
        if id == b"fmt " {
            canais = u16::from_le_bytes(dados[2..4].try_into().unwrap());
            taxa = u32::from_le_bytes(dados[4..8].try_into().unwrap());
        } else if id == b"data" {
            pcm = dados;
            break;
        }
        pos += 8 + sz + (sz & 1);
    }
    let mut l = Vec::new();
    let mut r = Vec::new();
    let step = (canais as usize) * 2;
    for frame in pcm.chunks_exact(step) {
        let sl = i16::from_le_bytes([frame[0], frame[1]]);
        let sr = if canais >= 2 {
            i16::from_le_bytes([frame[2], frame[3]])
        } else {
            sl
        };
        l.push(sl);
        r.push(sr);
    }
    (taxa, l, r)
}

fn sintetizar_cliques(duracao_s: f64, tempos_batida: &[f64]) -> (u32, Vec<i16>, Vec<i16>) {
    let taxa = 44_100u32;
    let total = (duracao_s * f64::from(taxa)) as usize;
    let mut buf = vec![0.0f64; total];
    let click_len = (f64::from(taxa) * 0.03) as isize;
    for &bt in tempos_batida {
        let bf = (bt * f64::from(taxa)) as isize;
        let j_ini = (-bf).max(0);
        let j_fim = click_len.min(total as isize - bf);
        for j in j_ini..j_fim {
            let t_local = j as f64 / f64::from(taxa);
            let sample =
                (-t_local * 250.0).exp() * (t_local * 2.0 * std::f64::consts::PI * 180.0).cos();
            buf[(bf + j) as usize] += sample;
        }
    }
    let mut l = Vec::with_capacity(total);
    let mut r = Vec::with_capacity(total);
    for s in buf {
        let v = (s.clamp(-1.0, 1.0) * 32_767.0) as i16;
        l.push(v);
        r.push(v);
    }
    (taxa, l, r)
}

fn beat_times(first: f64, bpm: f64, until: f64) -> Vec<f64> {
    let mut out = Vec::new();
    let mut t = first;
    while t < until - 0.05 {
        out.push(t);
        t += 60.0 / bpm;
    }
    out
}

fn carregar_ou_sintetizar(nome: &str) -> (u32, Vec<i16>, Vec<i16>) {
    let caminho = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("tests")
        .join("reference")
        .join("synthetic")
        .join("wavs")
        .join(nome);
    if caminho.exists() {
        return ler_wav_i16(&caminho);
    }
    match nome {
        "fixed_120bpm.wav" => sintetizar_cliques(24.0, &beat_times(0.317, 120.0, 24.0)),
        "silent_intro_125bpm.wav" => sintetizar_cliques(24.0, &beat_times(5.0, 125.0, 24.0)),
        "instrumental_126bpm.wav" => sintetizar_cliques(16.0, &beat_times(0.2, 126.0, 16.0)),
        "silence_10s.wav" => sintetizar_cliques(10.0, &[]),
        "short_audio_1s.wav" => sintetizar_cliques(1.0, &beat_times(0.1, 120.0, 1.0)),
        "tempo_change_120_to_140bpm.wav" => {
            let split = 16.0;
            let mut beats = beat_times(0.5, 120.0, split + 0.05);
            beats.extend(beat_times(split + 60.0 / 140.0, 140.0, 32.0));
            sintetizar_cliques(32.0, &beats)
        }
        other => panic!("fixture sintética desconhecida: {other}"),
    }
}

fn analisar_wav(nome: &str) -> (f32, usize) {
    let (taxa, l, r) = carregar_ou_sintetizar(nome);
    let mut analisador = AnalisadorDeBatidas::novo(taxa, l.len() as i32, Frase::PADRAO).unwrap();
    for (cl, cr) in l.chunks(4096).zip(r.chunks(4096)) {
        analisador.alimentar(cl, cr);
    }
    let (bpm, grade) = analisador.fechar();
    (bpm, grade.len())
}

#[test]
fn fixtures_sinteticas_estimam_bpm_e_silencio() {
    let (bpm_120, n_120) = analisar_wav("fixed_120bpm.wav");
    assert!((bpm_120 - 120.0).abs() < 1.0, "fixed_120bpm: bpm={bpm_120}");
    assert!(n_120 > 30, "fixed_120bpm: batidas={n_120}");

    let (bpm_125, n_125) = analisar_wav("silent_intro_125bpm.wav");
    assert!(
        (bpm_125 - 125.0).abs() < 1.5,
        "silent_intro_125bpm: bpm={bpm_125}"
    );
    assert!(n_125 > 20, "silent_intro_125bpm: batidas={n_125}");

    let (bpm_126, _) = analisar_wav("instrumental_126bpm.wav");
    assert!(
        (bpm_126 - 126.0).abs() < 1.5,
        "instrumental_126bpm: bpm={bpm_126}"
    );

    let (bpm_sil, n_sil) = analisar_wav("silence_10s.wav");
    assert_eq!(bpm_sil, 0.0, "silence_10s: bpm={bpm_sil}");
    assert_eq!(n_sil, 0, "silence_10s: batidas={n_sil}");

    let (bpm_short, n_short) = analisar_wav("short_audio_1s.wav");
    assert_eq!(bpm_short, 0.0, "short_audio_1s: bpm={bpm_short}");
    assert_eq!(n_short, 0, "short_audio_1s: batidas={n_short}");

    // Verifica também a fixture de mudança de andamento em Modo 1.
    let (taxa, l, r) = carregar_ou_sintetizar("tempo_change_120_to_140bpm.wav");
    let mut analisador_m1 = AnalisadorDeBatidas::novo(taxa, l.len() as i32, Frase::PADRAO).unwrap();
    for (cl, cr) in l.chunks(4096).zip(r.chunks(4096)) {
        analisador_m1.alimentar(cl, cr);
    }
    let (bpm_m1, grade_m1) = analisador_m1.fechar_modo(1);
    assert!(bpm_m1 > 115.0 && bpm_m1 < 145.0, "bpm_m1={bpm_m1}");
    assert!(grade_m1.len() > 40, "grade_m1 len={}", grade_m1.len());
}
