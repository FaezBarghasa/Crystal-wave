#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupType {
    Folder,
    SummingStack,
    VcaLink,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackGroup {
    pub id: usize,
    pub name: String,
    pub group_type: GroupType,
    pub track_indices: Vec<usize>,
    pub collapsed: bool,
    pub bus_id: Option<usize>,
}

impl TrackGroup {
    pub fn new(id: usize, name: impl Into<String>, group_type: GroupType) -> Self {
        Self {
            id,
            name: name.into(),
            group_type,
            track_indices: Vec::new(),
            collapsed: false,
            bus_id: None,
        }
    }

    pub fn add_track(&mut self, track_index: usize) {
        if !self.track_indices.contains(&track_index) {
            self.track_indices.push(track_index);
        }
    }

    pub fn remove_track(&mut self, track_index: usize) {
        self.track_indices.retain(|&idx| idx != track_index);
    }
}
