#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum VoiceStealingMode {
    #[default]
    Oldest,
    LowestVelocity,
    Quietest,
}

pub trait VirtualInstrument: Send + Sync {
    fn name(&self) -> &str;
    fn note_on(&mut self, note: u8, velocity: u16);
    fn note_off(&mut self, note: u8);
    fn pitch_bend(&mut self, semitones: f32);
    fn aftertouch(&mut self, pressure: u16);
    fn render_block(&mut self, left: &mut [f32], right: &mut [f32]);
    fn active_voices(&self) -> usize;
    fn reset(&mut self);
    fn set_master_tuning(&mut self, _tuning_hz: f32) {}
}
