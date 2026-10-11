#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum FadeCurve {
    #[default]
    Linear,
    EqualPower,
    Exponential,
    SCurve,
    SpeedUp,  // Tape start acceleration
    SlowDown, // Tape stop deceleration
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RegionFade {
    pub in_ticks: u64,
    pub in_curve: FadeCurve,
    pub out_ticks: u64,
    pub out_curve: FadeCurve,
}

impl Default for RegionFade {
    fn default() -> Self {
        Self {
            in_ticks: 0,
            in_curve: FadeCurve::Linear,
            out_ticks: 0,
            out_curve: FadeCurve::Linear,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RegionKind {
    Audio {
        file_path: Option<String>,
        sample_offset: usize,
        sample_length: usize,
    },
    Midi {
        clip_id: usize,
    },
    Pattern {
        pattern_id: usize,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Region {
    pub id: usize,
    pub track_id: usize,
    pub name: String,
    pub start_tick: u64,
    pub length_ticks: u64,
    pub anchor_offset_ticks: i64, // pickup note offset
    pub muted: bool,
    pub looped: bool,
    pub loop_length_ticks: Option<u64>,
    pub gain_db: f32,
    pub transpose_semitones: f32,
    pub reversed: bool,
    pub fade: RegionFade,
    pub kind: RegionKind,
}

impl Region {
    pub fn new(
        id: usize,
        track_id: usize,
        name: impl Into<String>,
        start_tick: u64,
        length_ticks: u64,
        kind: RegionKind,
    ) -> Self {
        Self {
            id,
            track_id,
            name: name.into(),
            start_tick,
            length_ticks,
            anchor_offset_ticks: 0,
            muted: false,
            looped: false,
            loop_length_ticks: None,
            gain_db: 0.0,
            transpose_semitones: 0.0,
            reversed: false,
            fade: RegionFade::default(),
            kind,
        }
    }

    pub fn effective_start_tick(&self) -> u64 {
        if self.anchor_offset_ticks < 0 {
            self.start_tick
                .saturating_sub((-self.anchor_offset_ticks) as u64)
        } else {
            self.start_tick + self.anchor_offset_ticks as u64
        }
    }

    pub fn set_anchor_offset(&mut self, offset_ticks: i64) {
        self.anchor_offset_ticks = offset_ticks;
    }

    pub fn set_fades(
        &mut self,
        in_ticks: u64,
        in_curve: FadeCurve,
        out_ticks: u64,
        out_curve: FadeCurve,
    ) {
        self.fade = RegionFade {
            in_ticks,
            in_curve,
            out_ticks,
            out_curve,
        };
    }

    pub fn set_reversed(&mut self, reversed: bool) {
        self.reversed = reversed;
    }

    pub fn split(&self, split_tick: u64, new_id: usize) -> Option<(Region, Region)> {
        if split_tick <= self.start_tick || split_tick >= self.start_tick + self.length_ticks {
            return None;
        }
        let first_len = split_tick - self.start_tick;
        let second_len = self.length_ticks - first_len;

        let mut first = self.clone();
        first.length_ticks = first_len;

        let mut second = self.clone();
        second.id = new_id;
        second.start_tick = split_tick;
        second.length_ticks = second_len;

        Some((first, second))
    }

    pub fn set_loop(&mut self, looped: bool, loop_length_ticks: Option<u64>) {
        self.looped = looped;
        self.loop_length_ticks = loop_length_ticks;
    }
}
