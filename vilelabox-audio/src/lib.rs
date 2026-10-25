//! `vilelabox-audio`: Motor de áudio, decodificação e transporte para o VILELABOX.
//!
//! Implementa as regras funcionais recuperadas do programa de referência
//! documentadas no Gate G0 (`spec/rekordbox/evidence/p6-0-transporte.md`)
//! e especificadas em `spec/rekordbox/30-deck.md` e `32-loops-cues.md`.

pub mod deck;
pub mod decode;
pub mod error;
pub mod output;
pub mod render;

pub use deck::{Deck, DeckSnapshot, TransportState};
pub use decode::{decode_audio, decode_wav, AudioBuffer};
pub use error::AudioError;
pub use output::{start_audio_engine, AudioCommand, AudioController, AudioMixer, EngineSnapshot};
pub use render::render_offline;
