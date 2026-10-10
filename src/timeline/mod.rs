pub mod arrangement_view;
pub mod loop_region;
pub mod marker;
pub mod quantize_mode;
pub mod region;
pub mod snapping_mode;
pub mod swing;
pub mod track;
pub mod track_group;

pub use arrangement_view::ArrangementView;
pub use loop_region::LoopRegion;
pub use marker::Marker;
pub use quantize_mode::QuantizeMode;
pub use region::{Region, RegionKind};
pub use snapping_mode::SnappingMode;
pub use swing::SwingAmount;
pub use track::{Color, ReturnNode, SendNode, Track, TrackType};
pub use track_group::{GroupType, TrackGroup};

use crate::error::{LogicError, Result};

pub const PPQN: u64 = 960;
pub const MAX_TRACKS: usize = 1000;

#[derive(Debug, Clone)]
pub struct TempoEvent {
    pub tick: u64,
    pub bpm: f32,
}

#[derive(Debug, Clone)]
pub struct TimeSignatureEvent {
    pub tick: u64,
    pub numerator: u8,
    pub denominator: u8,
}

#[derive(Debug, Clone)]
pub struct Timeline {
    pub tempo_map: Vec<TempoEvent>,
    pub signature_map: Vec<TimeSignatureEvent>,
    pub tracks: Vec<Track>,
    pub loop_region: LoopRegion,
    pub swing: SwingAmount,
    pub snapping_mode: SnappingMode,
}

impl Default for Timeline {
    fn default() -> Self {
        Self {
            tempo_map: vec![TempoEvent { tick: 0, bpm: 120.0 }],
            signature_map: vec![TimeSignatureEvent { tick: 0, numerator: 4, denominator: 4 }],
            tracks: Vec::new(),
            loop_region: LoopRegion::default(),
            swing: SwingAmount::default(),
            snapping_mode: SnappingMode::default(),
        }
    }
}

impl Timeline {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_track(&mut self, track: Track) -> Result<usize> {
        if self.tracks.len() >= MAX_TRACKS {
            return Err(LogicError::DspError("Maximum tracks (1000) reached".into()));
        }
        let id = self.tracks.len();
        self.tracks.push(track);
        Ok(id)
    }

    pub fn set_loop_region(&mut self, start_tick: u64, end_tick: u64) {
        self.loop_region = LoopRegion::new(start_tick, end_tick);
    }

    pub fn set_swing(&mut self, swing: SwingAmount) {
        self.swing = swing;
    }

    pub fn snap_to(&self, tick: u64, fps: f32) -> u64 {
        let (num, denom) = self.current_signature(tick);
        let bpm = self.current_bpm(tick);
        self.snapping_mode.snap(tick, num, denom, fps, bpm)
    }

    pub fn quantize(&self, region: &mut Region, mode: QuantizeMode) {
        let (num, denom) = self.current_signature(region.start_tick);
        region.start_tick = mode.quantize_tick(region.start_tick, num, denom);
    }

    pub fn current_bpm(&self, tick: u64) -> f32 {
        let mut bpm = 120.0f32;
        for ev in &self.tempo_map {
            if ev.tick <= tick {
                bpm = ev.bpm;
            } else {
                break;
            }
        }
        bpm
    }

    pub fn current_signature(&self, tick: u64) -> (u8, u8) {
        let mut sig = (4u8, 4u8);
        for ev in &self.signature_map {
            if ev.tick <= tick {
                sig = (ev.numerator, ev.denominator);
            } else {
                break;
            }
        }
        sig
    }

    pub fn tick_to_seconds(&self, target_tick: u64) -> f64 {
        let mut total_seconds = 0.0;
        let mut prev_tick = 0u64;
        let mut current_bpm = 120.0f32;

        for event in &self.tempo_map {
            if event.tick > target_tick {
                break;
            }
            let delta_ticks = event.tick - prev_tick;
            total_seconds += (delta_ticks as f64) / (PPQN as f64) * (60.0 / current_bpm as f64);
            prev_tick = event.tick;
            current_bpm = event.bpm;
        }

        let rem_ticks = target_tick - prev_tick;
        total_seconds += (rem_ticks as f64) / (PPQN as f64) * (60.0 / current_bpm as f64);
        total_seconds
    }
}

#[derive(Debug, Clone)]
pub struct Take {
    pub id: usize,
    pub name: String,
    pub audio_samples: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct CompSlice {
    pub take_id: usize,
    pub start_sample: usize,
    pub end_sample: usize,
}

/// Quick Swipe Comping Solver (Logic Pro Take Folder model)
pub struct TakeFolder {
    pub takes: Vec<Take>,
    pub comp_slices: Vec<CompSlice>,
    pub crossfade_samples: usize,
}

impl TakeFolder {
    pub fn new(crossfade_samples: usize) -> Self {
        Self {
            takes: Vec::new(),
            comp_slices: Vec::new(),
            crossfade_samples,
        }
    }

    pub fn add_take(&mut self, name: String, samples: Vec<f32>) -> usize {
        let id = self.takes.len();
        self.takes.push(Take { id, name, audio_samples: samples });
        id
    }

    pub fn set_comp_slice(&mut self, take_id: usize, start_sample: usize, end_sample: usize) -> Result<()> {
        if take_id >= self.takes.len() {
            return Err(LogicError::IncompatibleTakeBoundary {
                slice_tick: start_sample as u64,
                start: 0,
                end: self.takes.len() as u64,
            });
        }
        self.comp_slices.push(CompSlice { take_id, start_sample, end_sample });
        self.comp_slices.sort_by_key(|s| s.start_sample);
        Ok(())
    }

    pub fn render_comp(&self, total_samples: usize) -> Vec<f32> {
        let mut output = vec![0.0; total_samples];
        let xfade_len = self.crossfade_samples;

        for slice in &self.comp_slices {
            if let Some(take) = self.takes.iter().find(|t| t.id == slice.take_id) {
                let start = slice.start_sample.min(output.len());
                let end = slice.end_sample.min(output.len()).min(take.audio_samples.len());

                for i in start..end {
                    let mut sample = take.audio_samples[i];

                    if i < start + xfade_len && start > 0 {
                        let phase = (i - start) as f32 / xfade_len as f32;
                        let fade_in = (phase * std::f32::consts::FRAC_PI_2).sin();
                        sample *= fade_in;
                    }

                    if i + xfade_len > end && end < output.len() {
                        let phase = (end - i) as f32 / xfade_len as f32;
                        let fade_out = (phase * std::f32::consts::FRAC_PI_2).sin();
                        sample *= fade_out;
                    }

                    output[i] += sample;
                }
            }
        }

        output
    }
}
