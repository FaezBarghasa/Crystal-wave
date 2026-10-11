#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RecordingOverdubMode {
    #[default]
    Replace, // Destructive / tape-style overdub
    Merge,      // Merge into existing region
    NewTrack,   // Create a new track for each take
    TakeFolder, // Quick swipe comping take folder
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoopRegion {
    pub enabled: bool,
    pub start_tick: u64,
    pub end_tick: u64,
    pub punch_enabled: bool,
    pub punch_in_tick: u64,
    pub punch_out_tick: u64,
    pub pre_roll_ticks: u64,
    pub post_roll_ticks: u64,
    pub overdub_mode: RecordingOverdubMode,
}

impl Default for LoopRegion {
    fn default() -> Self {
        Self {
            enabled: false,
            start_tick: 0,
            end_tick: 3840, // 4 bars in 4/4 at 960 PPQN
            punch_enabled: false,
            punch_in_tick: 0,
            punch_out_tick: 3840,
            pre_roll_ticks: 0,
            post_roll_ticks: 0,
            overdub_mode: RecordingOverdubMode::Replace,
        }
    }
}

impl LoopRegion {
    pub fn new(start_tick: u64, end_tick: u64) -> Self {
        Self {
            enabled: true,
            start_tick,
            end_tick: end_tick.max(start_tick + 1),
            ..Default::default()
        }
    }

    pub fn set_punch(&mut self, punch_in: u64, punch_out: u64, enabled: bool) {
        self.punch_in_tick = punch_in;
        self.punch_out_tick = punch_out.max(punch_in + 1);
        self.punch_enabled = enabled;
    }

    pub fn set_pre_post_roll(&mut self, pre_roll_ticks: u64, post_roll_ticks: u64) {
        self.pre_roll_ticks = pre_roll_ticks;
        self.post_roll_ticks = post_roll_ticks;
    }

    pub fn is_in_punch_range(&self, tick: u64) -> bool {
        if !self.punch_enabled {
            true
        } else {
            tick >= self.punch_in_tick && tick < self.punch_out_tick
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
