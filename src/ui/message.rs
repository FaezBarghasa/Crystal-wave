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

    // Timeline & View Navigation
    SelectTrack(usize),
    SetHorizontalZoom(f32),
    SetVerticalZoom(f32),
    SetCatchPlayhead(CatchPlayheadMode),

    // Mixer & Channel Strip Controls
    SetTrackFader { track_id: usize, volume_db: f32 },
    SetTrackPan { track_id: usize, pan: f32 },
    ToggleTrackMute(usize),
    ToggleTrackSolo(usize),
    ResetAutoMute(usize),

    // Real-Time Telemetry & Periodic Ticks
    TelemetryReceived(TelemetryBatch),
    Tick,

    // Project Actions
    NewProject,
    SaveProject,
}
