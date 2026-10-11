use crate::bouncing::bit_depth_converter::{BitDepth, BitDepthConverter};
use crate::bouncing::bounce_engine::BounceEngine;
use crate::bouncing::dithering::DitherType;
use crate::bouncing::export_format::ExportFormatConfig;
use crate::bouncing::sample_rate_converter::SampleRateConverter;

pub struct RenderedStem {
    pub name: String,
    pub raw_bytes: Vec<u8>,
}

pub struct Exporter {
    pub config: ExportFormatConfig,
    pub bit_depth: BitDepth,
    pub dither: DitherType,
    pub bounce_engine: BounceEngine,
}

impl Exporter {
    pub fn new(config: ExportFormatConfig, bit_depth: BitDepth, dither: DitherType) -> Self {
        Self {
            config,
            bit_depth,
            dither,
            bounce_engine: BounceEngine::default(),
        }
    }

    /// Exports stereo audio stream to formatted byte buffer
    pub fn export_stereo_buffer(&self, left: &[f32], right: &[f32], source_rate: f32) -> Vec<u8> {
        let mut work_l = left.to_vec();
        let mut work_r = right.to_vec();

        // 1. Normalization
        self.bounce_engine
            .apply_normalization(&mut work_l, &mut work_r);

        // 2. Sample Rate Conversion if required
        let (resampled_l, resampled_r) = if (source_rate - self.config.sample_rate).abs() > 1e-3 {
            let src = SampleRateConverter::new(source_rate, self.config.sample_rate);
            (src.resample(&work_l), src.resample(&work_r))
        } else {
            (work_l, work_r)
        };

        // 3. Interleave stereo channels: [L0, R0, L1, R1, ...]
        let len = resampled_l.len().min(resampled_r.len());
        let mut interleaved = Vec::with_capacity(len * 2);
        for i in 0..len {
            interleaved.push(resampled_l[i]);
            interleaved.push(resampled_r[i]);
        }

        // 4. Bit depth quantization and dithering
        let mut bit_conv = BitDepthConverter::new(self.bit_depth, self.dither);
        bit_conv.convert_to_bytes(&interleaved)
    }

    /// Multi-track stems export
    pub fn export_stems(
        &self,
        stems: &[(&str, &[f32], &[f32])],
        source_rate: f32,
    ) -> Vec<RenderedStem> {
        let mut out = Vec::with_capacity(stems.len());
        for &(name, l, r) in stems {
            let bytes = self.export_stereo_buffer(l, r, source_rate);
            out.push(RenderedStem {
                name: name.to_string(),
                raw_bytes: bytes,
            });
        }
        out
    }
}
