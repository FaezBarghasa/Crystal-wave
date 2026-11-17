use crate::timeline::arrangement_view::CatchPlayheadMode;
use crate::ui::telemetry::TelemetryBatch;
use crate::ui_state::{EditorDrawerTab, LcdDisplayMode, ListAreaTab};

#[derive(Debug, Clone, PartialEq)]
pub enum UiMessage {
    // Window & Drawer Toggles (Logic Pro shortcuts: I, E, X, O, F, Y)
    ToggleInspector,
    ToggleEditorDrawer,
    ToggleMixerDrawer,
    ToggleLoopBrowser,
    ToggleFileBrowser,
    ToggleLibraryBrowser,
    SelectEditorTab(EditorDrawerTab),
    SelectListTab(ListAreaTab),
    SetLcdMode(LcdDisplayMode),

    // Transport & Playhead (Shortcuts: Space, R, Enter)
    TransportPlay,
    TransportPause,
    TransportStop,
    TransportRecord,
    TransportToggleLoop,
    SetPlayhead(u64),
    SetTempoBpm(f32),
    SetTimeSignature(u8, u8),
    SetMasterTuning(f32),

    // Timeline & View Navigation
    SelectTrack(usize),
    SetHorizontalZoom(f32),
    SetVerticalZoom(f32),
    SetCatchPlayhead(CatchPlayheadMode),

    // Mixer & Channel Strip Controls
    SetTrackFader {
        track_id: usize,
        volume_db: f32,
    },
    SetTrackPan {
        track_id: usize,
        pan: f32,
    },
    ToggleTrackMute(usize),
    ToggleTrackSolo(usize),
    ResetAutoMute(usize),

    // Real-Time Telemetry & Periodic Ticks
    TelemetryReceived(TelemetryBatch),
    Tick,

    // Keyboard Focus Single-Key Editing ('B', 'A', 'S', 'D', 'G', 'F')
    KeyboardFocusCommand(crate::pro_tools::KeyboardFocusCommand),
    ToggleVcaSpill(usize),
    SetTrackColorCascade {
        track_id: usize,
        color_rgb: [u8; 3],
    },

    // Project Actions
    NewProject,
    SaveProject,
    MenuAction(MenuCommand),

    // Plugin Lifecycle Management
    RemovePlugin(u32),
    EnablePlugin(u32),
    DisablePlugin(u32),

    // FL Studio Multi-Window & Free Floating View Actions
    ToggleFlChannelRack,
    ToggleFlPianoRoll,
    ToggleFlPlaylist,
    ToggleFlMixer,
    ToggleFlBrowser,
    ToggleFlPatcher,

    // FL Studio Channel Rack Step Sequencer & Drag-and-Drop
    ChannelRackToggleStep {
        channel_id: usize,
        step_index: usize,
    },
    ChannelRackDropSample {
        sample_id: usize,
    },
    TimelineMoveClip {
        clip_id: usize,
        new_track_id: usize,
        new_start_tick: u64,
    },
    TimelineTrimClipStart {
        clip_id: usize,
        new_start_tick: u64,
    },
    TimelineTrimClipEnd {
        clip_id: usize,
        new_length_ticks: u64,
    },
    TimelineFadeClip {
        clip_id: usize,
        fade_in_ticks: Option<u64>,
        fade_out_ticks: Option<u64>,
    },
    SelectRegion(usize),
    SetCycleRange {
        start_tick: u64,
        end_tick: u64,
    },
    ToggleTrackRecordArm(usize),
    ToggleTrackFreeze(usize),

    // FL Studio Piano Roll Articulations & Tools
    PianoRollQuickStrum,
    PianoRollFlipHorizontal,
    PianoRollFlipVertical,
    PianoRollQuickChop,

    // UI/UX Fusion Upgrades
    SetThemePreset(crate::ui::theme::ThemePreset),
    SetSnapGrid(crate::ui::canvas_timeline::SnapGrid),
    TimelineMarqueeSelect {
        start_tick: u64,
        end_tick: u64,
        start_track: usize,
        end_track: usize,
    },
    TimelineDuplicateClip(usize),
    TimelineAddPatternClip {
        track_id: usize,
        start_tick: u64,
        length_ticks: u64,
        name: String,
    },
    SelectDrawerTab(crate::ui::layout::DrawerTab),
    TogglePanelDock(crate::ui::layout::PanelId),
    DetachPanelToWindow(crate::ui::layout::PanelId),
    RedockPanel(crate::ui::layout::PanelId, crate::ui::layout::DockPosition),
    ToggleCommandPalette,
    CommandPaletteInput(String),
    ExecuteCommand(String),
    SetTimelineScrollY(f32),
    PianoRollAddNote {
        channel_id: usize,
        key: u8,
        start_tick: u64,
        length_ticks: u64,
    },
    PianoRollDeleteNote(usize),
    PianoRollMoveNote {
        id: usize,
        delta_ticks: i64,
        delta_key: i8,
    },
    PianoRollResizeNote {
        id: usize,
        delta_ticks: i64,
    },
    PianoRollToggleSlide(usize),
    PianoRollSelectKey(u8),
    PianoRollSetScrollY(f32),
    StepSequencerToggleStep {
        channel_id: usize,
        step_index: usize,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuCommand {
    NewProject,
    OpenSession,
    SaveSession,
    SaveSessionAs,
    BounceWav,
    ExportAaf,
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SplitAtCursor,
    Quantize,
    AddAudioTrack,
    AddMidiTrack,
    DuplicateTrack,
    FreezeTrack,
    DeleteTrack,
    AudioSettings,
    MidiSettings,
    ToggleMixer,
    TogglePianoRoll,
    TogglePatcher,
    ToggleInspector,
}
