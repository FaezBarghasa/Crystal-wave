#[derive(Debug, Clone, PartialEq)]
pub struct Marker {
    pub id: usize,
    pub name: String,
    pub tick: u64,
    pub length_ticks: Option<u64>,
    pub color_rgb: [u8; 3],
    pub notes: String,
}

impl Marker {
    pub fn new(id: usize, name: impl Into<String>, tick: u64) -> Self {
        Self {
            id,
            name: name.into(),
            tick,
            length_ticks: None,
            color_rgb: [80, 140, 220],
            notes: String::new(),
        }
    }
}
