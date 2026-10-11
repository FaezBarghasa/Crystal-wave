use std::collections::BTreeMap;

/// Pro Tools Memory Location saving exact cursor position, zoom factor,
/// track show/hide visibility, and pre/post-roll settings.
#[derive(Debug, Clone, PartialEq)]
pub struct MemoryLocation {
    pub number: usize,
    pub name: String,
    pub tick: u64,
    pub zoom_factor: f32,
    pub track_visibility_mask: Vec<bool>,
    pub pre_roll_ticks: u64,
    pub post_roll_ticks: u64,
    pub color_rgb: [u8; 3],
}

impl MemoryLocation {
    pub fn new(number: usize, name: &str, tick: u64) -> Self {
        Self {
            number,
            name: name.to_string(),
            tick,
            zoom_factor: 1.0,
            track_visibility_mask: Vec::new(),
            pre_roll_ticks: 0,
            post_roll_ticks: 0,
            color_rgb: [255, 204, 0], // Pro Tools marker yellow
        }
    }

    pub fn with_zoom(mut self, zoom: f32) -> Self {
        self.zoom_factor = zoom;
        self
    }

    pub fn with_track_visibility(mut self, mask: Vec<bool>) -> Self {
        self.track_visibility_mask = mask;
        self
    }

    pub fn with_pre_post_roll(mut self, pre: u64, post: u64) -> Self {
        self.pre_roll_ticks = pre;
        self.post_roll_ticks = post;
        self
    }
}

/// Bank storing and recalling memory locations by numeric index (e.g. keypad . N . in Pro Tools)
#[derive(Debug, Clone, Default, PartialEq)]
pub struct MemoryLocationBank {
    pub locations: BTreeMap<usize, MemoryLocation>,
}

impl MemoryLocationBank {
    pub fn new() -> Self {
        Self {
            locations: BTreeMap::new(),
        }
    }

    /// Stores a memory location at its designated number (1..=999).
    pub fn store(&mut self, location: MemoryLocation) -> Result<(), &'static str> {
        if location.number == 0 || location.number > 999 {
            return Err("InvalidNumber: Memory location numbers must be between 1 and 999");
        }
        self.locations.insert(location.number, location);
        Ok(())
    }

    /// Recalls a memory location by slot number.
    pub fn recall(&self, number: usize) -> Option<&MemoryLocation> {
        self.locations.get(&number)
    }

    /// Removes a memory location by slot number.
    pub fn delete(&mut self, number: usize) -> Option<MemoryLocation> {
        self.locations.remove(&number)
    }

    /// Returns a list of all stored memory locations sorted by slot number.
    pub fn list_all(&self) -> Vec<&MemoryLocation> {
        self.locations.values().collect()
    }
}
