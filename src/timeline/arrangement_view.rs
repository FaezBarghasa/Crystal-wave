use crate::timeline::track::Track;
use crate::timeline::track_group::TrackGroup;
use crate::timeline::Timeline;

pub struct ArrangementView {
    pub timeline: Timeline,
    pub track_pool: Vec<Track>,
    pub groups: Vec<TrackGroup>,
    pub selected_track_index: Option<usize>,
    pub visible_start_tick: u64,
    pub visible_end_tick: u64,
    pub vertical_zoom: f32,
    pub horizontal_zoom: f32,
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
        }
    }

    pub fn select_track(&mut self, index: usize) {
        if index < self.track_pool.len() {
            self.selected_track_index = Some(index);
        }
    }
}
