#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PoeticMeter {
    #[default]
    Iambic,    // unstressed - STRESSED (da-DUM)
    Trochaic,  // STRESSED - unstressed (DUM-da)
    Anapestic, // unstressed - unstressed - STRESSED (da-da-DUM)
    Dactylic,  // STRESSED - unstressed - unstressed (DUM-da-da)
}

#[derive(Debug, Clone, PartialEq)]
pub struct LyricMeterPattern {
    pub meter: PoeticMeter,
    pub feet_per_line: usize, // e.g. 5 for Pentameter, 4 for Tetrameter
}

impl Default for LyricMeterPattern {
    fn default() -> Self {
        Self {
            meter: PoeticMeter::Iambic,
            feet_per_line: 4, // Tetrameter
        }
    }
}

#[derive(Default)]
pub struct AiLyricGeneration {
    pub pattern: LyricMeterPattern,
}


impl AiLyricGeneration {
    pub fn new(pattern: LyricMeterPattern) -> Self {
        Self { pattern }
    }

    pub fn target_syllables_per_line(&self) -> usize {
        match self.pattern.meter {
            PoeticMeter::Iambic | PoeticMeter::Trochaic => self.pattern.feet_per_line * 2,
            PoeticMeter::Anapestic | PoeticMeter::Dactylic => self.pattern.feet_per_line * 3,
        }
    }

    /// Validates whether a line matches expected rhythmic stress/syllable constraint
    pub fn validate_line_meter(&self, syllables: usize) -> bool {
        let target = self.target_syllables_per_line();
        (syllables as i32 - target as i32).abs() <= 1
    }
}
