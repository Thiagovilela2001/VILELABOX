use std::path::PathBuf;
use vilelabox_audio::{decode_audio, decode_wav, AudioError};

#[test]
fn decodifica_wav_sintetico_com_sucesso() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let wav_path = manifest_dir
        .parent()
        .unwrap()
        .join("tests/reference/synthetic/wavs/cues_and_loops_124bpm.wav");

    if wav_path.exists() {
        let buffer = decode_wav(&wav_path).expect("falha ao decodificar WAV sintético");
        assert_eq!(buffer.sample_rate, 44100);
        assert_eq!(buffer.channels, 2);
        assert!(buffer.total_frames > 0);
        assert!((buffer.duration_seconds() - 32.0).abs() < 0.1);
        assert!(!buffer.samples.is_empty());
    }
}

#[test]
fn arquivo_inexistente_retorna_erro_amigavel() {
    let dummy_path = PathBuf::from("arquivo_que_definitivamente_nao_existe_12345.wav");
    let err = decode_audio(&dummy_path).unwrap_err();
    match err {
        AudioError::FileNotFound(p) => assert_eq!(p, dummy_path),
        other => panic!("esperava FileNotFound, recebeu {other:?}"),
    }
}
