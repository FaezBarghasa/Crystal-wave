use crate::timeline::PPQN;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SnappingMode {
    #[default]
    Smart,
    Bar,
    Beat,
    Division,
    Tick,
    Frame,
    Quart,
}

impl SnappingMode {
    pub fn grid_ticks(&self, numerator: u8, denominator: u8, fps: f32, bpm: f32) -> u64 {
        match self {
            SnappingMode::Bar => (PPQN * 4 * numerator as u64) / (denominator as u64),
            SnappingMode::Beat => (PPQN * 4) / (denominator as u64),
            SnappingMode::Division => (PPQN / 4).max(1),
            SnappingMode::Tick => 1,
            SnappingMode::Frame => {
                let ticks_per_sec = (PPQN as f32) * (bpm / 60.0);
                ((ticks_per_sec / fps.max(1.0)).round() as u64).max(1)
            }
            SnappingMode::Quart => {
                let frame_ticks = SnappingMode::Frame.grid_ticks(numerator, denominator, fps, bpm);
                (frame_ticks / 4).max(1)
            }
            SnappingMode::Smart => (PPQN / 4).max(1),
        }
    }

    pub fn snap(&self, tick: u64, numerator: u8, denominator: u8, fps: f32, bpm: f32) -> u64 {
        let grid = self.grid_ticks(numerator, denominator, fps, bpm);
        if grid <= 1 {
            return tick;
        }
        let half = grid / 2;
        ((tick + half) / grid) * grid
    }
}
