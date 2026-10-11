#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MarkerType {
    #[default]
    Standard,
    Intro,
    Verse,
    Chorus,
    Bridge,
    Drop,
    Outro,
    CyclePoint,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Marker {
    pub id: usize,
    pub name: String,
    pub tick: u64,
    pub length_ticks: Option<u64>,
    pub color_rgb: [u8; 3],
    pub marker_type: MarkerType,
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
            marker_type: MarkerType::Standard,
            notes: String::new(),
        }
    }

    pub fn with_type(mut self, marker_type: MarkerType) -> Self {
        self.marker_type = marker_type;
        self
    }

    pub fn with_length(mut self, length_ticks: u64) -> Self {
        self.length_ticks = Some(length_ticks);
        self
    }

    pub fn with_color(mut self, r: u8, g: u8, b: u8) -> Self {
        self.color_rgb = [r, g, b];
        self
    }
}
