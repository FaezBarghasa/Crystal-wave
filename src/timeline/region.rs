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
pub struct AutomationPoint {
    pub tick_offset: u64,
    pub value: f32,   // Normalized 0.0 to 1.0
    pub tension: f32, // -1.0 to 1.0 (FL Studio tension parameter)
}

impl AutomationPoint {
    pub fn new(tick_offset: u64, value: f32, tension: f32) -> Self {
        Self {
            tick_offset,
            value: value.clamp(0.0, 1.0),
            tension: tension.clamp(-1.0, 1.0),
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
    Automation {
        parameter_target: String,
        min_value: f32,
        max_value: f32,
        points: Vec<AutomationPoint>,
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

    /// Evaluates the automation parameter value at `tick_offset` relative to the region start
    pub fn evaluate_automation(&self, tick_offset: u64) -> Option<f32> {
        if let RegionKind::Automation {
            min_value,
            max_value,
            points,
            ..
        } = &self.kind
        {
            if points.is_empty() {
                return Some(*min_value);
            }
            if tick_offset <= points[0].tick_offset {
                let norm = points[0].value;
                return Some(min_value + (max_value - min_value) * norm);
            }
            if tick_offset >= points[points.len() - 1].tick_offset {
                let norm = points[points.len() - 1].value;
                return Some(min_value + (max_value - min_value) * norm);
            }

            for i in 0..points.len().saturating_sub(1) {
                let p0 = &points[i];
                let p1 = &points[i + 1];
                if tick_offset >= p0.tick_offset && tick_offset <= p1.tick_offset {
                    let seg_len = (p1.tick_offset - p0.tick_offset) as f32;
                    if seg_len <= 0.0 {
                        let norm = p0.value;
                        return Some(min_value + (max_value - min_value) * norm);
                    }
                    let u = ((tick_offset - p0.tick_offset) as f32) / seg_len;
                    let progress = if p0.tension > 0.0 {
                        u.powf(1.0 + p0.tension * 3.0)
                    } else if p0.tension < 0.0 {
                        1.0 - (1.0 - u).powf(1.0 - p0.tension * 3.0)
                    } else {
                        u
                    };
                    let norm = p0.value + (p1.value - p0.value) * progress;
                    return Some(min_value + (max_value - min_value) * norm);
                }
            }
            Some(*min_value)
        } else {
            None
        }
    }
}
