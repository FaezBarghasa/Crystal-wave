use crate::timeline::PPQN;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MidiQuantizeGrid {
    #[default]
    Sixteenth,       // 240 ticks
    Eighth,          // 480 ticks
    Quarter,         // 960 ticks
    ThirtySecond,    // 120 ticks
    TripletEighth,   // 320 ticks
    TripletSixteenth,// 160 ticks
}

impl MidiQuantizeGrid {
    pub fn step_ticks(&self) -> u64 {
        match self {
            MidiQuantizeGrid::Quarter => PPQN,
            MidiQuantizeGrid::Eighth => PPQN / 2,
            MidiQuantizeGrid::Sixteenth => PPQN / 4,
            MidiQuantizeGrid::ThirtySecond => PPQN / 8,
            MidiQuantizeGrid::TripletEighth => (PPQN * 4) / 12,
            MidiQuantizeGrid::TripletSixteenth => (PPQN * 4) / 24,
        }
    }

    pub fn quantize(&self, tick: u64) -> u64 {
        let step = self.step_ticks();
        let half = step / 2;
        ((tick + half) / step) * step
    }
}
