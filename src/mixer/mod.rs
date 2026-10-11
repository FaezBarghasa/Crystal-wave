use crate::dsp::compressor::{Compressor, CompressorConfig};
use crate::error::{LogicError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PanMode {
    Balance,
    DualPan,
    Binaural,
    DolbyAtmos3D,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PanPosition {
    pub mode: PanMode,
    /// Balance: -1.0 (Left) to +1.0 (Right)
    pub balance: f32,
    /// Dual Pan: Left (-1.0 to +1.0), Right (-1.0 to +1.0)
    pub dual_left: f32,
    pub dual_right: f32,
    /// Dolby Atmos 3D Object Panner parameters
    pub azimuth_deg: f32,   // -180.0 to +180.0 deg
    pub elevation_deg: f32, // 0.0 to 90.0 deg
    pub spread: f32,        // 0.0 (point source) to 1.0 (diffuse)
    pub size: f32,          // 0.0 to 1.0
}

impl Default for PanPosition {
    fn default() -> Self {
        Self {
            mode: PanMode::Balance,
            balance: 0.0,
            dual_left: -1.0,
            dual_right: 1.0,
            azimuth_deg: 0.0,
            elevation_deg: 0.0,
            spread: 0.0,
            size: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendMode {
    PreFader,
    PostFader,
    PostPan,
}

#[derive(Debug, Clone)]
pub struct SendSlot {
    pub destination_bus: usize,
    pub send_level_db: f32, // -inf to +6 dB
    pub send_mode: SendMode,
    pub muted: bool,
}

pub struct ChannelStrip {
    pub id: usize,
    pub name: String,
    pub volume_fader_db: f32, // -inf to +6.0 dB
    pub pan: PanPosition,
    pub muted: bool,
    pub solo: bool,
    pub phase_invert: bool,
    pub compressor: Option<Compressor>,
    pub sends: Vec<SendSlot>,
    pub vca_group: Option<usize>,
    pub output_bus: usize, // 0 = Stereo Out, 1+ = Aux/Sub bus
    pub auto_mute_tripped: bool,
}

impl ChannelStrip {
    pub fn new(id: usize, name: String, sample_rate: f32) -> Self {
        Self {
            id,
            name,
            volume_fader_db: 0.0,
            pan: PanPosition::default(),
            muted: false,
            solo: false,
            phase_invert: false,
            compressor: Some(Compressor::new(CompressorConfig::default(), sample_rate).unwrap()),
            sends: Vec::new(),
            vca_group: None,
            output_bus: 0,
            auto_mute_tripped: false,
        }
    }

    pub fn reset_auto_mute(&mut self) {
        self.auto_mute_tripped = false;
    }

    /// Linear gain calculated from fader dB
    pub fn linear_gain(&self) -> f32 {
        if self.volume_fader_db <= -96.0 || self.muted || self.auto_mute_tripped {
            0.0
        } else {
            10.0f32.powf(self.volume_fader_db / 20.0)
        }
    }

    /// Process audio block and monitor for > +12 dBFS overload (Auto-Mute protection)
    pub fn process_block(&mut self, left: &mut [f32], right: &mut [f32], vca_gain: f32) -> Result<()> {
        if left.len() != right.len() {
            return Err(LogicError::BufferLengthMismatch {
                expected: left.len(),
                actual: right.len(),
            });
        }

        if self.auto_mute_tripped || self.muted {
            left.fill(0.0);
            right.fill(0.0);
            return Ok(());
        }

        // 1. Phase inversion
        if self.phase_invert {
            for s in left.iter_mut() {
                *s = -*s;
            }
            for s in right.iter_mut() {
                *s = -*s;
            }
        }

        // 2. Insert FX (Compressor)
        if let Some(comp) = &mut self.compressor {
            comp.process(left, right)?;
        }

        // 3. Fader & VCA gain application
        let gain = self.linear_gain() * vca_gain;
        for s in left.iter_mut() {
            *s *= gain;
        }
        for s in right.iter_mut() {
            *s *= gain;
        }

        // 4. Stereo Pan
        match self.pan.mode {
            PanMode::Balance => {
                let pan = self.pan.balance.clamp(-1.0, 1.0);
                let gain_l = ((1.0 - pan) * 0.5 * std::f32::consts::FRAC_PI_2).sin();
                let gain_r = ((1.0 + pan) * 0.5 * std::f32::consts::FRAC_PI_2).sin();
                for s in left.iter_mut() {
                    *s *= gain_l;
                }
                for s in right.iter_mut() {
                    *s *= gain_r;
                }
            }
            PanMode::DualPan => {
                let gain_l = (self.pan.dual_left.clamp(-1.0, 1.0) + 1.0) * 0.5;
                let gain_r = (self.pan.dual_right.clamp(-1.0, 1.0) + 1.0) * 0.5;
                for s in left.iter_mut() {
                    *s *= gain_l;
                }
                for s in right.iter_mut() {
                    *s *= gain_r;
                }
            }
            _ => {}
        }

        // 5. Logic Pro Auto-Mute Safety Protection: Overload check > +12 dBFS (linear ~3.98)
        let max_val = left
            .iter()
            .chain(right.iter())
            .fold(0.0f32, |acc, &s| acc.max(s.abs()));

        if max_val >= 3.98 {
            self.auto_mute_tripped = true;
            left.fill(0.0);
            right.fill(0.0);
        }

        Ok(())
    }
}

/// Dolby Atmos 7.1.4 3D Bed Downmixer & Object Panner
pub struct DolbyAtmosPanner {
    pub azimuth_deg: f32,
    pub elevation_deg: f32,
    pub spread: f32,
}

impl DolbyAtmosPanner {
    pub fn new(azimuth_deg: f32, elevation_deg: f32, spread: f32) -> Self {
        Self {
            azimuth_deg,
            elevation_deg,
            spread,
        }
    }

    /// Calculate 7.1.4 bed gains: [L, R, C, LFE, Ls, Rs, Rls, Rrs, Ltf, Rtf, Ltr, Rtr]
    pub fn calculate_bed_gains(&self) -> [f32; 12] {
        let rad_az = self.azimuth_deg.to_radians();
        let rad_el = self.elevation_deg.to_radians();

        let x = rad_az.sin() * rad_el.cos(); // Right (+) / Left (-)
        let y = rad_az.cos() * rad_el.cos(); // Front (+) / Rear (-)
        let z = rad_el.sin();                // Top (+) / Floor (0)

        let mut gains = [0.0f32; 12];

        // Front Left / Right
        gains[0] = ((1.0 - x) * 0.5 * (1.0 + y) * 0.5 * (1.0 - z)).max(0.0);
        gains[1] = ((1.0 + x) * 0.5 * (1.0 + y) * 0.5 * (1.0 - z)).max(0.0);

        // Center
        gains[2] = ((1.0 - x.abs()) * (1.0 + y).max(0.0) * 0.5 * (1.0 - z)).max(0.0);

        // LFE (Sub)
        gains[3] = 0.0;

        // Surrounds (Ls, Rs)
        gains[4] = ((1.0 - x) * 0.5 * (1.0 - y.abs()) * (1.0 - z)).max(0.0);
        gains[5] = ((1.0 + x) * 0.5 * (1.0 - y.abs()) * (1.0 - z)).max(0.0);

        // Rear Surrounds (Rls, Rrs)
        gains[6] = ((1.0 - x) * 0.5 * (1.0 - y) * 0.5 * (1.0 - z)).max(0.0);
        gains[7] = ((1.0 + x) * 0.5 * (1.0 - y) * 0.5 * (1.0 - z)).max(0.0);

        // Overhead Heights (Ltf, Rtf, Ltr, Rtr)
        gains[8] = ((1.0 - x) * 0.5 * (1.0 + y) * 0.5 * z).max(0.0);
        gains[9] = ((1.0 + x) * 0.5 * (1.0 + y) * 0.5 * z).max(0.0);
        gains[10] = ((1.0 - x) * 0.5 * (1.0 - y) * 0.5 * z).max(0.0);
        gains[11] = ((1.0 + x) * 0.5 * (1.0 - y) * 0.5 * z).max(0.0);

        // Apply spread (diffuse energy across all bed channels)
        if self.spread > 0.0 {
            let diffuse = self.spread / 12.0;
            for g in gains.iter_mut() {
                *g = *g * (1.0 - self.spread) + diffuse;
            }
        }

        gains
    }
}
