use std::f32::consts::PI;

pub const BED_CHANNELS_7_1_4: usize = 12;

/// Channel mapping indices for 7.1.4 Dolby Atmos:
/// 0: Left, 1: Right, 2: Center, 3: LFE,
/// 4: Left Side Surround (Lss), 5: Right Side Surround (Rss),
/// 6: Left Rear Surround (Lsr), 7: Right Rear Surround (Rsr),
/// 8: Left Top Front (Ltf), 9: Right Top Front (Rtf),
/// 10: Left Top Rear (Ltr), 11: Right Top Rear (Rtr)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpatialObjectPanner {
    pub azimuth_deg: f32,   // -180.0 to +180.0
    pub elevation_deg: f32, // 0.0 to 90.0
    pub spread: f32,        // 0.0 to 1.0
    pub size: f32,          // 0.0 to 1.0
    pub height: f32,        // 0.0 to 1.0
}

impl Default for SpatialObjectPanner {
    fn default() -> Self {
        Self {
            azimuth_deg: 0.0,
            elevation_deg: 0.0,
            spread: 0.0,
            size: 0.0,
            height: 0.0,
        }
    }
}

impl SpatialObjectPanner {
    pub fn new(azimuth_deg: f32, elevation_deg: f32, spread: f32) -> Self {
        Self {
            azimuth_deg: azimuth_deg.clamp(-180.0, 180.0),
            elevation_deg: elevation_deg.clamp(0.0, 90.0),
            spread: spread.clamp(0.0, 1.0),
            size: 0.0,
            height: elevation_deg / 90.0,
        }
    }

    /// Calculate 12-channel bed gains according to VBAP / Dolby Atmos panning rules
    pub fn calculate_bed_gains(&self) -> [f32; BED_CHANNELS_7_1_4] {
        let mut gains = [0.0f32; BED_CHANNELS_7_1_4];

        let elev_rad = (self.elevation_deg * (PI / 180.0)).clamp(0.0, PI / 2.0);
        let height_gain = elev_rad.sin();
        let bed_plane_gain = elev_rad.cos();

        // Convert azimuth to 0..2*PI (0 = Center front, PI/2 = Right, PI = Rear, 3PI/2 = Left)
        let az_rad = ((self.azimuth_deg + 360.0) % 360.0) * (PI / 180.0);

        // Bed horizontal panning (Front L/C/R, Sides, Rears)
        let center_weight = (1.0 - (self.azimuth_deg / 45.0).abs()).max(0.0);
        let left_right = (az_rad.sin() + 1.0) * 0.5; // 0 = full Left, 1 = full Right
        let front_back = (az_rad.cos() + 1.0) * 0.5; // 1 = full Front, 0 = full Rear

        // 0: Left, 1: Right, 2: Center
        gains[0] = (1.0 - left_right) * front_back * bed_plane_gain * (1.0 - center_weight * 0.5);
        gains[1] = left_right * front_back * bed_plane_gain * (1.0 - center_weight * 0.5);
        gains[2] = center_weight * front_back * bed_plane_gain;

        // 3: LFE (subwoofer) gets diffuse spread if enabled
        gains[3] = self.spread * 0.2;

        // 4: Lss, 5: Rss (side surround: when front_back is near 0.5)
        let side_weight = 1.0 - (2.0 * front_back - 1.0).abs();
        gains[4] = (1.0 - left_right) * side_weight * bed_plane_gain;
        gains[5] = left_right * side_weight * bed_plane_gain;

        // 6: Lsr, 7: Rsr (rear surround)
        let rear_weight = (1.0 - front_back).max(0.0);
        gains[6] = (1.0 - left_right) * rear_weight * bed_plane_gain;
        gains[7] = left_right * rear_weight * bed_plane_gain;

        // Heights: 8: Ltf, 9: Rtf, 10: Ltr, 11: Rtr
        gains[8] = (1.0 - left_right) * front_back * height_gain;
        gains[9] = left_right * front_back * height_gain;
        gains[10] = (1.0 - left_right) * rear_weight * height_gain;
        gains[11] = left_right * rear_weight * height_gain;

        // Apply spread diffusion: blend toward equal distribution across bed
        if self.spread > 0.0 {
            let diffuse = (1.0 / (BED_CHANNELS_7_1_4 as f32).sqrt()) * self.spread;
            for g in &mut gains {
                *g = *g * (1.0 - self.spread) + diffuse;
            }
        }

        // Normalize sum of squares (energy conservation)
        let sum_sq: f32 = gains.iter().map(|&g| g * g).sum();
        if sum_sq > 1e-6 {
            let norm = 1.0 / sum_sq.sqrt();
            for g in &mut gains {
                *g *= norm;
            }
        }

        gains
    }

    /// Downmix 7.1.4 bed to binaural stereo for headphone preview
    pub fn binaural_downmix(&self, bed: &[f32; BED_CHANNELS_7_1_4]) -> (f32, f32) {
        // Binaural HRTF ITD (Interaural Time Difference) and ILD (Interaural Level Difference)
        let left = bed[0] * 1.0
            + bed[2] * 0.707
            + bed[4] * 0.8
            + bed[6] * 0.6
            + bed[8] * 0.7
            + bed[10] * 0.5
            + bed[1] * 0.2;
        let right = bed[1] * 1.0
            + bed[2] * 0.707
            + bed[5] * 0.8
            + bed[7] * 0.6
            + bed[9] * 0.7
            + bed[11] * 0.5
            + bed[0] * 0.2;
        (left * 0.6, right * 0.6)
    }

    /// Positions object on 3D spherical dome
    pub fn move_to(&mut self, azimuth_deg: f32, elevation_deg: f32) {
        self.azimuth_deg = azimuth_deg.clamp(-180.0, 180.0);
        self.elevation_deg = elevation_deg.clamp(0.0, 90.0);
        self.height = self.elevation_deg / 90.0;
    }

    /// Renders a mono input audio stream into a 12-channel 7.1.4 bed block
    pub fn render_3d_block(&self, input: &[f32], output_12ch: &mut [&mut [f32]]) {
        if output_12ch.len() < BED_CHANNELS_7_1_4 {
            return;
        }
        let bed_gains = self.calculate_bed_gains();
        let n = input.len();

        for ch in 0..BED_CHANNELS_7_1_4 {
            let gain = bed_gains[ch];
            let out_slice = &mut output_12ch[ch];
            let len = n.min(out_slice.len());
            for i in 0..len {
                out_slice[i] = input[i] * gain;
            }
        }
    }
}
