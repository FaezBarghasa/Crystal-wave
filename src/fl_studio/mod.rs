pub mod automation_clip;
pub mod browser;
pub mod channel_rack;
pub mod gross_beat;
pub mod history;
pub mod patcher;
pub mod piano_roll;
pub mod playlist;
pub mod soft_clipper;

pub use automation_clip::{AutomationClip, SplineNode};
pub use browser::{FlBrowser, SampleEntry};
pub use channel_rack::{ChannelRack, RackChannel, StepProperties};
pub use gross_beat::{GrossBeatEngine, GrossBeatTimePreset, GrossBeatVolumePreset};
pub use history::{FlHistoryManager, HistoryAction, MAX_HISTORY_STEPS};
pub use patcher::{FlPatcher, PatcherCable, PatcherMacroKnob, PatcherNode, PatcherNodeKind};
pub use piano_roll::{
    ArpDirection, EventLaneTarget, FlPianoRollNote, FlPianoRollTools, GhostNote, MusicalScale,
};
pub use playlist::{FlPattern, FlPlaylist, PlaylistItem, PlaylistTrackLane};
pub use soft_clipper::FruitySoftClipper;

/// FL Studio Free Windows Architecture (Detachable, independent windows)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlWindowManager {
    pub channel_rack_visible: bool, // F6 shortcut in FL Studio
    pub piano_roll_visible: bool,   // F7 shortcut in FL Studio
    pub playlist_visible: bool,     // F5 shortcut in FL Studio
    pub mixer_visible: bool,        // F9 shortcut in FL Studio
    pub browser_visible: bool,      // Alt+F8 shortcut in FL Studio
    pub patcher_visible: bool,
    pub history_visible: bool,
}

impl Default for FlWindowManager {
    fn default() -> Self {
        Self {
            channel_rack_visible: true,
            piano_roll_visible: false,
            playlist_visible: true,
            mixer_visible: false,
            browser_visible: true,
            patcher_visible: false,
            history_visible: false,
        }
    }
}

impl FlWindowManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn toggle_channel_rack(&mut self) {
        self.channel_rack_visible = !self.channel_rack_visible;
    }

    pub fn toggle_piano_roll(&mut self) {
        self.piano_roll_visible = !self.piano_roll_visible;
    }

    pub fn toggle_playlist(&mut self) {
        self.playlist_visible = !self.playlist_visible;
    }

    pub fn toggle_mixer(&mut self) {
        self.mixer_visible = !self.mixer_visible;
    }

    pub fn toggle_browser(&mut self) {
        self.browser_visible = !self.browser_visible;
    }

    pub fn toggle_patcher(&mut self) {
        self.patcher_visible = !self.patcher_visible;
    }

    pub fn toggle_history(&mut self) {
        self.history_visible = !self.history_visible;
    }
}
