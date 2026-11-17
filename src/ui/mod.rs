pub mod app;
pub mod canvas_piano_roll;
pub mod canvas_timeline;
pub mod layout;
pub mod message;
pub mod session_driver;
pub mod telemetry;
pub mod theme;
pub mod view;

pub use app::CrystalWaveApp;
pub use canvas_piano_roll::{
    GhostNoteDisplay, PianoRollCanvasProgram, PianoRollNoteDisplay, view_canvas_piano_roll,
};
pub use canvas_timeline::{ArrangementCanvasProgram, SnapGrid, WaveformPeak, view_canvas_timeline};
pub use layout::{DockPosition, DrawerTab, LayoutState, PanelId, PanelState};
pub use message::UiMessage;
pub use session_driver::UiSessionDriver;
pub use telemetry::{ChannelMeterFrame, TelemetryBatch, TelemetryChannel};
pub use theme::{ThemePreset, UiTheme};
