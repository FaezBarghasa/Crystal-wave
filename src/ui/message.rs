use crate::timeline::arrangement_view::CatchPlayheadMode;
use crate::ui::telemetry::TelemetryBatch;
use crate::ui_state::{EditorDrawerTab, LcdDisplayMode, ListAreaTab};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AudioBackend {
    #[default]
    PipeWire,
    Alsa,
    PulseAudio,
    Jack,
}

impl AudioBackend {
    pub fn label(&self) -> &'static str {
        match self {
            Self::PipeWire => "PipeWire (Pro Audio Low-Latency)",
            Self::Alsa => "ALSA Direct Hardware",
            Self::PulseAudio => "PulseAudio Standard",
            Self::Jack => "JACK Audio Connection Kit",
        }
    }
}

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
    SetSwing(f32),
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
    SelectMixerTrack(usize),
    SetTrackStereoSeparation {
        track_id: usize,
        separation: f32,
    },
    ToggleTrackPhaseInvert(usize),
    ToggleTrackSwapChannels(usize),
    ToggleInsertSlotBypass {
        track_id: usize,
        slot_id: usize,
    },
    SetInsertSlotMix {
        track_id: usize,
        slot_id: usize,
        mix: f32,
    },
    ToggleMasterSoftClipper,

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
    ToggleFlPluginPicker,
    ToggleFlEdison,
    ToggleFlFpc,
    ToggleFlSlicex,
    SelectFlView(crate::ui::layout::FlActiveView),

    // Edison Operations
    EdisonNormalize,
    EdisonReverse,
    EdisonFadeIn,
    EdisonFadeOut,
    EdisonCut,
    EdisonCopy,
    EdisonPaste,
    EdisonSelectAll,
    EdisonSetSelection {
        start: usize,
        end: usize,
    },
    EdisonSetCursor(usize),

    // FPC Drum Triggers
    FpcTriggerPad(usize),
    FpcSetVolume(f32),
    FpcSelectPad(usize),
    FpcSetPadVolume {
        pad_id: usize,
        volume: f32,
    },
    FpcSetPadPan {
        pad_id: usize,
        pan: f32,
    },
    FpcTogglePadMute(usize),
    FpcTogglePadSolo(usize),

    // Slicex Triggers
    SlicexTriggerSlice(usize),

    // Tap Tempo & Transport
    TapTempo,
    NudgeTempo(f32),

    // Mixer Routing Sends
    MixerToggleSend {
        from_track: usize,
        to_track: usize,
    },
    MixerSetSendLevel {
        from_track: usize,
        to_track: usize,
        level: f32,
    },

    // Fruity Wrapper
    WrapperSetTab(crate::fl_studio::wrapper::WrapperTab),
    WrapperSetAdsr {
        channel_id: usize,
        attack: f32,
        decay: f32,
        sustain: f32,
        release: f32,
    },

    // FL Studio Browser & Sample Audition
    BrowserSetSearchQuery(String),
    BrowserSelectTab(crate::fl_studio::browser::BrowserTab),
    BrowserAuditionSample(usize),
    BrowserStopAudition,
    BrowserDropSampleToPlaylist {
        sample_id: usize,
        track_id: usize,
        start_tick: u64,
    },

    // FL Studio Visual Plugin Picker
    PluginPickerSetSearch(String),
    PluginPickerSelectCategory(Option<crate::plugins::plugin_descriptor::PluginCategory>),
    PluginPickerLoadPlugin {
        plugin_id: u32,
        is_generator: bool,
    },
    OpenPluginPickerForSlot {
        track_id: usize,
        slot_id: usize,
    },

    // FL Studio F10 Audio Settings Modal & Hardware Engine Telemetry
    ToggleAudioSettings,
    SetAudioBackend(AudioBackend),
    SetAudioBufferSize(usize),
    SetAudioSampleRate(u32),
    ResetAudioXruns,

    // FL Studio Channel Rack Step Sequencer & Drag-and-Drop
    ChannelRackToggleStep {
        channel_id: usize,
        step_index: usize,
    },
    ChannelRackSetVolume {
        channel_id: usize,
        volume: f32,
    },
    ChannelRackSetPan {
        channel_id: usize,
        pan: f32,
    },
    ChannelRackSetTargetMixerTrack {
        channel_id: usize,
        track: usize,
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
    PianoRollArpeggiate(crate::fl_studio::piano_roll::ArpDirection),
    PianoRollSetScale(crate::fl_studio::piano_roll::MusicalScale),
    PianoRollSetScaleRoot(u8),
    PianoRollSetEventTarget(crate::fl_studio::piano_roll::EventLaneTarget),
    PianoRollSetNoteProperty {
        note_id: usize,
        property: crate::fl_studio::piano_roll::EventLaneTarget,
        value: f32,
    },

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
    TimelineAddAutomationClip {
        track_id: usize,
        start_tick: u64,
        length_ticks: u64,
        parameter_target: String,
    },
    TimelineSetAutomationTension {
        clip_id: usize,
        segment_index: usize,
        tension: f32,
    },
    TimelineSetAutomationNode {
        clip_id: usize,
        node_index: usize,
        value: f32,
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

    // FL Studio Transport & Pattern/Song Mode
    TogglePatternSongMode,
    SelectPattern(usize),
    ToggleMetronome,
    ToggleTypingKeyboard,
    TogglePrecount,
    ToggleBlendRecording,
    SetMasterVolume(f32),
    SetMasterPitchCents(i32),

    // FL Studio Channel Settings / Fruity Wrapper
    OpenChannelSettings(usize),
    CloseChannelSettings,
    ChannelSettingsSetPitch {
        channel_id: usize,
        pitch_cents: i32,
    },
    ChannelSettingsSetPan {
        channel_id: usize,
        pan: f32,
    },
    ChannelSettingsSetVolume {
        channel_id: usize,
        volume: f32,
    },
    ChannelSettingsSetMixerTrack {
        channel_id: usize,
        track: usize,
    },

    // FL Studio Channel Rack Extended
    ToggleChannelRackGraphEditor,
    SetChannelRackStepCount(usize),

    // FL Studio Tool Strips
    SetPlaylistTool(PlaylistTool),
    SetPianoRollTool(PianoRollTool),

    // Live Hint Bar Parameter Telemetry
    HoverParameter(String),
    ClearHoverParameter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaylistTool {
    #[default]
    Draw, // P
    Paint,  // B
    Delete, // D
    Mute,   // T
    Slip,   // S
    Slice,  // C
    Select, // E
    Zoom,   // Z
}

impl PlaylistTool {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Draw => "Draw (P)",
            Self::Paint => "Paint (B)",
            Self::Delete => "Delete (D)",
            Self::Mute => "Mute (T)",
            Self::Slip => "Slip (S)",
            Self::Slice => "Slice (C)",
            Self::Select => "Select (E)",
            Self::Zoom => "Zoom (Z)",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PianoRollTool {
    #[default]
    Draw, // P
    Paint,  // B
    Delete, // D
    Mute,   // T
    Slice,  // C
    Select, // E
    Zoom,   // Z
}

impl PianoRollTool {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Draw => "Draw (P)",
            Self::Paint => "Paint (B)",
            Self::Delete => "Delete (D)",
            Self::Mute => "Mute (T)",
            Self::Slice => "Slice (C)",
            Self::Select => "Select (E)",
            Self::Zoom => "Zoom (Z)",
        }
    }
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
