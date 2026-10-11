use crate::timeline::Timeline;
use crate::timeline::marker::Marker;
use crate::timeline::track::Track;
use crate::timeline::track_group::TrackGroup;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CatchPlayheadMode {
    #[default]
    Off,
    ContinuousScroll,
    PageByPage,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BeatMappingPoint {
    pub tick: u64,
    pub sample_position: usize,
    pub transient_confidence: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MovieTrackInfo {
    pub movie_path: String,
    pub fps: f32,
    pub start_smpte_tc: String,
    pub sample_rate: f32,
}

pub struct ArrangementView {
    pub timeline: Timeline,
    pub track_pool: Vec<Track>,
    pub groups: Vec<TrackGroup>,
    pub selected_track_index: Option<usize>,
    pub visible_start_tick: u64,
    pub visible_end_tick: u64,
    pub vertical_zoom: f32,
    pub horizontal_zoom: f32,
    pub waveform_vertical_zoom: f32,
    pub catch_playhead: CatchPlayheadMode,
    pub arrangement_markers: Vec<Marker>,
    pub beat_mappings: Vec<BeatMappingPoint>,
    pub movie_track: Option<MovieTrackInfo>,
}

impl ArrangementView {
    pub fn new(timeline: Timeline) -> Self {
        Self {
            timeline,
            track_pool: Vec::new(),
            groups: Vec::new(),
            selected_track_index: None,
            visible_start_tick: 0,
            visible_end_tick: 3840 * 8,
            vertical_zoom: 1.0,
            horizontal_zoom: 1.0,
            waveform_vertical_zoom: 1.0,
            catch_playhead: CatchPlayheadMode::ContinuousScroll,
            arrangement_markers: Vec::new(),
            beat_mappings: Vec::new(),
            movie_track: None,
        }
    }

    pub fn select_track(&mut self, index: usize) {
        if index < self.track_pool.len() {
            self.selected_track_index = Some(index);
        }
    }

    pub fn set_waveform_zoom(&mut self, zoom: f32) {
        self.waveform_vertical_zoom = zoom.clamp(0.1, 10.0);
    }

    pub fn set_catch_playhead(&mut self, mode: CatchPlayheadMode) {
        self.catch_playhead = mode;
    }

    pub fn update_playhead_view(&mut self, playhead_tick: u64) {
        let view_width = self
            .visible_end_tick
            .saturating_sub(self.visible_start_tick);
        match self.catch_playhead {
            CatchPlayheadMode::Off => {}
            CatchPlayheadMode::ContinuousScroll => {
                if playhead_tick > self.visible_end_tick || playhead_tick < self.visible_start_tick
                {
                    let half = view_width / 2;
                    self.visible_start_tick = playhead_tick.saturating_sub(half);
                    self.visible_end_tick = self.visible_start_tick + view_width;
                }
            }
            CatchPlayheadMode::PageByPage => {
                if playhead_tick >= self.visible_end_tick {
                    self.visible_start_tick = self.visible_end_tick;
                    self.visible_end_tick = self.visible_start_tick + view_width;
                } else if playhead_tick < self.visible_start_tick {
                    self.visible_start_tick = (playhead_tick / view_width) * view_width;
                    self.visible_end_tick = self.visible_start_tick + view_width;
                }
            }
        }
    }
}
