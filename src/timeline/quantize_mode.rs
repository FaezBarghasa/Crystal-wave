use crate::timeline::PPQN;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum QuantizeMode {
    #[default]
    Off,
    Bar,
    Beat,
    Division,      // 1/16th note
    Eighth,        // 1/8th note
    Sixteenth,     // 1/16th note
    ThirtySecond,  // 1/32nd note
    TripletEighth,
    TripletSixteenth,
    DottedEighth,
    DottedSixteenth,
    Swing8th(u8),  // 50% - 75%
    Swing16th(u8), // 50% - 75%
}

impl QuantizeMode {
    pub fn quantize_tick(&self, tick: u64, numerator: u8, denominator: u8) -> u64 {
        match self {
            QuantizeMode::Off => tick,
            QuantizeMode::Bar => {
                let bar_ticks = (PPQN * 4 * numerator as u64) / (denominator as u64);
                if bar_ticks == 0 { return tick; }
                ((tick + bar_ticks / 2) / bar_ticks) * bar_ticks
            }
            QuantizeMode::Beat => {
                let beat_ticks = (PPQN * 4) / (denominator as u64);
                if beat_ticks == 0 { return tick; }
                ((tick + beat_ticks / 2) / beat_ticks) * beat_ticks
            }
            QuantizeMode::Division | QuantizeMode::Sixteenth => {
                let step = PPQN / 4; // 240 ticks
                ((tick + step / 2) / step) * step
            }
            QuantizeMode::Eighth => {
                let step = PPQN / 2; // 480 ticks
                ((tick + step / 2) / step) * step
            }
            QuantizeMode::ThirtySecond => {
                let step = PPQN / 8; // 120 ticks
                ((tick + step / 2) / step) * step
            }
            QuantizeMode::TripletEighth => {
                let step = (PPQN * 4) / 12; // 320 ticks
                ((tick + step / 2) / step) * step
            }
            QuantizeMode::TripletSixteenth => {
                let step = (PPQN * 4) / 24; // 160 ticks
                ((tick + step / 2) / step) * step
            }
            QuantizeMode::DottedEighth => {
                let step = (PPQN * 3) / 4; // 720 ticks
                ((tick + step / 2) / step) * step
            }
            QuantizeMode::DottedSixteenth => {
                let step = (PPQN * 3) / 8; // 360 ticks
                ((tick + step / 2) / step) * step
            }
            QuantizeMode::Swing8th(amount) => {
                let pair_ticks = PPQN; // 2 eighth notes
                let base_pair = (tick / pair_ticks) * pair_ticks;
                let offset = tick % pair_ticks;
                let swing_pct = (*amount).clamp(50, 75) as f64 / 100.0;
                let swing_point = (pair_ticks as f64 * swing_pct).round() as u64;
                if offset < (swing_point / 2) {
                    base_pair
                } else if offset < swing_point + ((pair_ticks - swing_point) / 2) {
                    base_pair + swing_point
                } else {
                    base_pair + pair_ticks
                }
            }
            QuantizeMode::Swing16th(amount) => {
                let pair_ticks = PPQN / 2; // 2 sixteenth notes = 480 ticks
                let base_pair = (tick / pair_ticks) * pair_ticks;
                let offset = tick % pair_ticks;
                let swing_pct = (*amount).clamp(50, 75) as f64 / 100.0;
                let swing_point = (pair_ticks as f64 * swing_pct).round() as u64;
                if offset < (swing_point / 2) {
                    base_pair
                } else if offset < swing_point + ((pair_ticks - swing_point) / 2) {
                    base_pair + swing_point
                } else {
                    base_pair + pair_ticks
                }
            }
        }
    }
}
