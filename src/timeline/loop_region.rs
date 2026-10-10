#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoopRegion {
    pub enabled: bool,
    pub start_tick: u64,
    pub end_tick: u64,
}

impl Default for LoopRegion {
    fn default() -> Self {
        Self {
            enabled: false,
            start_tick: 0,
            end_tick: 3840, // 4 bars in 4/4 at 960 PPQN
        }
    }
}

impl LoopRegion {
    pub fn new(start_tick: u64, end_tick: u64) -> Self {
        Self {
            enabled: true,
            start_tick,
            end_tick: end_tick.max(start_tick + 1),
        }
    }

    pub fn length_ticks(&self) -> u64 {
        self.end_tick.saturating_sub(self.start_tick)
    }

    pub fn wrap_tick(&self, tick: u64) -> u64 {
        if !self.enabled || self.end_tick <= self.start_tick {
            return tick;
        }
        if tick < self.start_tick {
            return tick;
        }
        let len = self.length_ticks();
        self.start_tick + ((tick - self.start_tick) % len)
    }
}
