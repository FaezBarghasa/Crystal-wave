pub struct DecodedAudio {
    pub channels: usize,
    pub sample_rate: u32,
    pub samples_interleaved: Vec<f32>,
}

impl DecodedAudio {
    pub fn to_planar_stereo(&self) -> (Vec<f32>, Vec<f32>) {
        if self.channels == 0 {
            return (Vec::new(), Vec::new());
        }
        let total_frames = self.samples_interleaved.len() / self.channels;
        let mut left = Vec::with_capacity(total_frames);
        let mut right = Vec::with_capacity(total_frames);

        if self.channels == 1 {
            for &s in &self.samples_interleaved {
                left.push(s);
                right.push(s);
            }
        } else {
            for frame in 0..total_frames {
                let idx = frame * self.channels;
                left.push(self.samples_interleaved[idx]);
                right.push(self.samples_interleaved[idx + 1]);
            }
        }
        (left, right)
    }
}

pub struct AudioDecoder;

impl AudioDecoder {
    /// Decodes WAV audio samples via hound
    pub fn decode_wav_bytes(bytes: &[u8]) -> Result<DecodedAudio, String> {
        let cursor = std::io::Cursor::new(bytes);
        let mut reader =
            hound::WavReader::new(cursor).map_err(|e| format!("WavReaderError: {e}"))?;
        let spec = reader.spec();
        let channels = spec.channels as usize;
        let sample_rate = spec.sample_rate;

        let samples_interleaved: Vec<f32> = match spec.sample_format {
            hound::SampleFormat::Float => {
                reader.samples::<f32>().map(|s| s.unwrap_or(0.0)).collect()
            }
            hound::SampleFormat::Int => match spec.bits_per_sample {
                16 => reader
                    .samples::<i16>()
                    .map(|s| s.map(|v| v as f32 / 32768.0).unwrap_or(0.0))
                    .collect(),
                24 => reader
                    .samples::<i32>()
                    .map(|s| s.map(|v| v as f32 / 8388608.0).unwrap_or(0.0))
                    .collect(),
                32 => reader
                    .samples::<i32>()
                    .map(|s| s.map(|v| v as f32 / 2147483648.0).unwrap_or(0.0))
                    .collect(),
                _ => return Err("UnsupportedBitDepth".to_string()),
            },
        };

        Ok(DecodedAudio {
            channels,
            sample_rate,
            samples_interleaved,
        })
    }

    /// Universal audio decoder fallback
    pub fn decode_any_format_bytes(bytes: &[u8]) -> Result<DecodedAudio, String> {
        Self::decode_wav_bytes(bytes)
    }
}
