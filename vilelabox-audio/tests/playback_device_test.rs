use std::sync::Arc;
use std::time::Duration;
use vilelabox_audio::{start_audio_engine, AudioBuffer, AudioCommand};

#[test]
fn test_audio_playback_device() {
    let (ctrl, stream) = start_audio_engine().expect("start_audio_engine failed");
    println!("Stream is_some: {}", stream.is_some());

    let buf = Arc::new(AudioBuffer::synthetic_sine(48000, 440.0, 1.0));
    ctrl.send(AudioCommand::LoadTrack {
        deck: 1,
        audio: buf,
        cue_frame: Some(0),
    });

    ctrl.send(AudioCommand::Play { deck: 1 });

    std::thread::sleep(Duration::from_millis(200));

    let snap = ctrl.get_snapshot();
    println!("Deck 1 is_playing: {}", snap.deck1.is_playing);
    println!("Deck 1 pos_frames: {}", snap.deck1.position_frames);
    println!("Deck 1 pos_s: {}", snap.deck1.position_seconds);
    println!("Underruns: {}", snap.underruns);

    assert!(snap.deck1.position_frames > 0);
}
