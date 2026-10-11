#[derive(Debug, Clone, PartialEq)]
pub struct GroupedTake {
    pub id: usize,
    pub name: String,
    pub rating: u8,
    pub data_per_track: Vec<Vec<f32>>,
}

impl GroupedTake {
    pub fn new(id: usize, name: &str, rating: u8, data_per_track: Vec<Vec<f32>>) -> Self {
        Self {
            id,
            name: name.to_string(),
            rating: rating.clamp(1, 5),
            data_per_track,
        }
    }
}

/// Multi-track grouped playlist manager for tracking multi-mic sources (e.g., 10-piece drum kit).
/// Ensures take switching, rating, and comping happen synchronously across all grouped channels.
#[derive(Debug, Clone, PartialEq)]
pub struct MultiTrackPlaylistGroup {
    pub group_id: usize,
    pub group_name: String,
    pub track_ids: Vec<usize>,
    pub active_take_idx: usize,
    pub takes: Vec<GroupedTake>,
    /// Consolidated master comp audio buffer per track
    pub main_comp_buffers: Vec<Vec<f32>>,
}

impl MultiTrackPlaylistGroup {
    pub fn new(group_id: usize, group_name: &str, track_ids: Vec<usize>) -> Self {
        let num_tracks = track_ids.len();
        Self {
            group_id,
            group_name: group_name.to_string(),
            track_ids,
            active_take_idx: 0,
            takes: Vec::new(),
            main_comp_buffers: vec![Vec::new(); num_tracks],
        }
    }

    pub fn num_tracks(&self) -> usize {
        self.track_ids.len()
    }

    pub fn add_take(
        &mut self,
        name: &str,
        data_per_track: Vec<Vec<f32>>,
    ) -> Result<usize, &'static str> {
        if data_per_track.len() != self.num_tracks() {
            return Err("GroupMismatch: Buffer count does not match grouped track count");
        }

        let id = self.takes.len();
        let take = GroupedTake::new(id, name, 3, data_per_track);
        self.takes.push(take);
        Ok(id)
    }

    /// Synchronously switches the active listening playlist across all grouped tracks.
    pub fn gang_switch_take(&mut self, take_idx: usize) -> Result<(), &'static str> {
        if take_idx >= self.takes.len() {
            return Err("OutOfBoundsTake: Requested take index does not exist");
        }
        self.active_take_idx = take_idx;
        Ok(())
    }

    /// Rates a take (1 to 5 stars) across all channels in the group.
    pub fn gang_rate_take(&mut self, take_idx: usize, rating: u8) -> Result<(), &'static str> {
        if take_idx >= self.takes.len() {
            return Err("OutOfBoundsTake: Requested take index does not exist");
        }
        self.takes[take_idx].rating = rating.clamp(1, 5);
        Ok(())
    }

    /// Promotes a selected time range [start_sample..start_sample + len_samples] from the active take
    /// into the main target comp playlist across ALL tracks in the group synchronously.
    pub fn promote_selection_to_main(
        &mut self,
        start_sample: usize,
        len_samples: usize,
    ) -> Result<(), &'static str> {
        if self.active_take_idx >= self.takes.len() {
            return Err("NoActiveTake: No active take selected to promote");
        }

        let active_take = &self.takes[self.active_take_idx];

        for (ch_idx, track_data) in active_take.data_per_track.iter().enumerate() {
            if start_sample + len_samples > track_data.len() {
                return Err("RangeOutOfBounds: Promotion range exceeds take buffer length");
            }

            // Ensure destination buffer has sufficient size
            let required_len = start_sample + len_samples;
            if self.main_comp_buffers[ch_idx].len() < required_len {
                self.main_comp_buffers[ch_idx].resize(required_len, 0.0);
            }

            // Copy slice into main comp
            let slice = &track_data[start_sample..start_sample + len_samples];
            self.main_comp_buffers[ch_idx][start_sample..start_sample + len_samples]
                .copy_from_slice(slice);
        }

        Ok(())
    }
}
