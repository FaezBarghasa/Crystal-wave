use thiserror::Error;

#[derive(Error, Debug, Clone, PartialEq)]
pub enum LogicError {
    #[error("Invalid sample rate: {0} Hz (must be positive between 22050 and 384000)")]
    InvalidSampleRate(f32),

    #[error("Buffer length mismatch: expected {expected}, got {actual}")]
    BufferLengthMismatch { expected: usize, actual: usize },

    #[error("Invalid channel count: {0} (supported: 1 to 16)")]
    InvalidChannelCount(usize),

    #[error("Harmonic key or scale not found in Markov model: {0}")]
    MarkovKeyNotFound(String),

    #[error("Invalid Flex Time/Pitch parameter: {0}")]
    InvalidFlexParameter(String),

    #[error("Incompatible take comp boundary: slice at tick {slice_tick} is outside take bounds [{start}, {end}]")]
    IncompatibleTakeBoundary { slice_tick: u64, start: u64, end: u64 },

    #[error("DSP processing error: {0}")]
    DspError(String),
}

pub type Result<T> = std::result::Result<T, LogicError>;
