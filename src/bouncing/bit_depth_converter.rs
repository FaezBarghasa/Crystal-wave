use crate::bouncing::dithering::{DitherEngine, DitherType};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BitDepth {
    Bit16,
    #[default]
    Bit24,
    Bit32Float,
}

pub struct BitDepthConverter {
    pub target_depth: BitDepth,
    pub dither: DitherEngine,
}

impl BitDepthConverter {
    pub fn new(target_depth: BitDepth, dither_type: DitherType) -> Self {
        Self {
            target_depth,
            dither: DitherEngine::new(dither_type),
        }
    }

    /// Converts 32-bit float samples to raw byte stream (little-endian)
    pub fn convert_to_bytes(&mut self, input: &[f32]) -> Vec<u8> {
        match self.target_depth {
            BitDepth::Bit16 => {
                let mut bytes = Vec::with_capacity(input.len() * 2);
                for &s in input {
                    let sample_i16 = self.dither.dither_sample_to_i16(s);
                    bytes.extend_from_slice(&sample_i16.to_le_bytes());
                }
                bytes
            }
            BitDepth::Bit24 => {
                let mut bytes = Vec::with_capacity(input.len() * 3);
                for &s in input {
                    let dither_noise = self.dither.generate_tpdf();
                    let scaled = (s * 8388607.0 + dither_noise).round().clamp(-8388608.0, 8388607.0) as i32;
                    let b = scaled.to_le_bytes();
                    bytes.push(b[0]);
                    bytes.push(b[1]);
                    bytes.push(b[2]);
                }
                bytes
            }
            BitDepth::Bit32Float => {
                let mut bytes = Vec::with_capacity(input.len() * 4);
                for &s in input {
                    bytes.extend_from_slice(&s.to_le_bytes());
                }
                bytes
            }
        }
    }
}
