pub mod dsp;
pub mod error;
pub mod generative;
pub mod harmony;
pub mod midi;
pub mod timeline;

pub use dsp::compressor::{Compressor, CompressorConfig, CompressorModel, MultibandCompressor};
pub use dsp::equalizer::{BiquadCoeffs, ChannelEq, EqBand, FilterType};
pub use dsp::flex::{FlexMode, FlexPitchNote, FlexProcessor};
pub use dsp::reverb::ChromaVerbEngine;
pub use error::{LogicError, Result};
pub use generative::{GenerativePerformanceParams, SessionPlayer, SessionPlayerType};
pub use harmony::{Chord, ChordQuality, ChordTrack, MarkovProgressionEngine};
pub use midi::{MidiDirection, SequencerStep, StepSequencerLane};
pub use timeline::{PPQN, TakeFolder, Timeline};
