#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwingAmount(pub u8);

impl Default for SwingAmount {
    fn default() -> Self {
        Self(50) // 50% = straight time
    }
}

impl SwingAmount {
    pub fn new(percent: u8) -> Self {
        Self(percent.clamp(50, 75))
    }

    pub fn as_fraction(&self) -> f32 {
        self.0 as f32 / 100.0
    }
}
