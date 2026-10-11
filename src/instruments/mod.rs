pub mod built_in_instruments;
pub mod instrument;
pub mod virtual_instrument;

pub use built_in_instruments::BuiltInInstrumentType;
pub use instrument::{VirtualInstrument, VoiceStealingMode};
pub use virtual_instrument::{AdsrEnvelope, OscillatorWaveform, PolyphonicSynth, SynthVoiceState};
