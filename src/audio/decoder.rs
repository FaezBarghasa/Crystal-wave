use std::io::Cursor;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{CODEC_TYPE_NULL, DecoderOptions};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

#[derive(Debug, Clone, PartialEq)]
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
    /// Decodes an in-memory byte slice (WAV, MP3, FLAC, OGG) into 32-bit float audio samples
    pub fn decode_bytes(
        bytes: &[u8],
        extension_hint: Option<&str>,
    ) -> Result<DecodedAudio, String> {
        let cursor = Cursor::new(bytes.to_vec());
        let mss = MediaSourceStream::new(Box::new(cursor), Default::default());

        let mut hint = Hint::new();
        if let Some(ext) = extension_hint {
            hint.with_extension(ext);
        }

        let meta_opts = MetadataOptions::default();
        let fmt_opts = FormatOptions::default();

        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &fmt_opts, &meta_opts)
            .map_err(|e| format!("ProbeFailed: {e}"))?;

        let mut format = probed.format;
        let track = format
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| "NoAudioTrackFound".to_string())?;

        let track_id = track.id;
        let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
        let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(2);

        let dec_opts = DecoderOptions::default();
        let mut decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &dec_opts)
            .map_err(|e| format!("DecoderInitFailed: {e}"))?;

        let mut samples_interleaved = Vec::new();
        let mut sample_buf = None;

        while let Ok(packet) = format.next_packet() {
            if packet.track_id() != track_id {
                continue;
            }

            match decoder.decode(&packet) {
                Ok(audio_buf) => {
                    if sample_buf.is_none() {
                        let spec = *audio_buf.spec();
                        let duration = audio_buf.capacity() as u64;
                        sample_buf = Some(SampleBuffer::<f32>::new(duration, spec));
                    }

                    if let Some(buf) = sample_buf.as_mut() {
                        buf.copy_interleaved_ref(audio_buf);
                        samples_interleaved.extend_from_slice(buf.samples());
                    }
                }
                Err(SymphoniaError::IoError(e)) => {
                    return Err(format!("DecodeIoError: {e}"));
                }
                Err(SymphoniaError::DecodeError(e)) => {
                    return Err(format!("DecodeError: {e}"));
                }
                Err(_) => break,
            }
        }

        Ok(DecodedAudio {
            channels,
            sample_rate,
            samples_interleaved,
        })
    }
}
