#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditorDrawerTab {
    PianoRoll,
    AudioTrackEditor,
    AudioFileEditor,
    StepSequencer,
    ScoreEditor,
    LiveLoopsGrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ListAreaTab {
    EventList,
    MarkerList,
    TempoList,
    SignatureList,
    ProjectAudioBin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LcdDisplayMode {
    BeatsAndProject,
    TimecodeSmpte,
    BeatsAndTime,
    CustomDiagnostic,
}

#[derive(Debug, Clone)]
pub struct LcdDiagnosticState {
    pub current_midi_chord: Option<String>,
    pub cpu_load_percent: f32,
    pub disk_load_percent: f32,
    pub varispeed_cents_offset: f32,
    pub project_tempo_bpm: f32,
    pub time_signature: (u8, u8),
    pub master_tuning_hz: f32, // Master reference frequency for A4 (e.g. 440.0, 432.0, 434.5)
}

/// Workspace Window Management (Logic Pro Single-Window Architecture)
#[derive(Debug, Clone)]
pub struct WorkspaceState {
    pub inspector_open: bool,       // Toggle shortcut 'I'
    pub editor_drawer_open: bool,   // Toggle shortcut 'E'
    pub mixer_drawer_open: bool,    // Toggle shortcut 'X'
    pub loop_browser_open: bool,    // Toggle shortcut 'O'
    pub file_browser_open: bool,    // Toggle shortcut 'F'
    pub library_browser_open: bool, // Toggle shortcut 'Y'
    pub active_editor: EditorDrawerTab,
    pub active_list_tab: ListAreaTab,
    pub lcd_mode: LcdDisplayMode,
    pub lcd_diagnostic: LcdDiagnosticState,
    pub selected_track_id: usize,
    pub selected_region_id: Option<usize>,
}

impl Default for WorkspaceState {
    fn default() -> Self {
        Self {
            inspector_open: true,
            editor_drawer_open: false,
            mixer_drawer_open: false,
            loop_browser_open: false,
            file_browser_open: false,
            library_browser_open: false,
            active_editor: EditorDrawerTab::PianoRoll,
            active_list_tab: ListAreaTab::EventList,
            lcd_mode: LcdDisplayMode::BeatsAndProject,
            lcd_diagnostic: LcdDiagnosticState {
                current_midi_chord: None,
                cpu_load_percent: 0.0,
                disk_load_percent: 0.0,
                varispeed_cents_offset: 0.0,
                project_tempo_bpm: 120.0,
                time_signature: (4, 4),
                master_tuning_hz: 440.0,
            },
            selected_track_id: 0,
            selected_region_id: None,
        }
    }
}

/// Dual Channel Strip Inspector Model
#[derive(Debug, Clone)]
pub struct DualChannelInspector {
    pub selected_track_id: usize,
    /// Contextual downstream destination:
    /// 0 = Stereo Master Output, 1+ = Subgroup Aux Bus
    pub downstream_destination_bus: usize,
}

impl DualChannelInspector {
    pub fn new(selected_track_id: usize, downstream_destination_bus: usize) -> Self {
        Self {
            selected_track_id,
            downstream_destination_bus,
        }
    }

    pub fn update_selection(&mut self, track_id: usize, output_bus: usize) {
        self.selected_track_id = track_id;
        self.downstream_destination_bus = output_bus;
    }
}
