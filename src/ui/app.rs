use crate::mixing::MixingEngine;
use crate::timeline::arrangement_view::ArrangementView;
use crate::timeline::Timeline;
use crate::ui::message::UiMessage;
use crate::ui_state::{DualChannelInspector, WorkspaceState};

pub struct CrystalWaveApp {
    pub workspace: WorkspaceState,
    pub inspector: DualChannelInspector,
    pub arrangement: ArrangementView,
    pub mixer: MixingEngine,
    pub is_playing: bool,
    pub is_recording: bool,
    pub is_looping: bool,
    pub playhead_tick: u64,
    pub dirty: bool,
}

impl CrystalWaveApp {
    pub fn new(sample_rate: f32) -> Self {
        let timeline = Timeline::new();
        let arrangement = ArrangementView::new(timeline);
        let mixer = MixingEngine::new(sample_rate).expect("Mixer init");
        Self {
            workspace: WorkspaceState::default(),
            inspector: DualChannelInspector::new(0, 0),
            arrangement,
            mixer,
            is_playing: false,
            is_recording: false,
            is_looping: false,
            playhead_tick: 0,
            dirty: false,
        }
    }

    /// Pure Elm Architecture State Reducer with Input Hardening & Sanitization
    pub fn update(&mut self, message: UiMessage) {
        match message {
            UiMessage::ToggleInspector => {
                self.workspace.inspector_open = !self.workspace.inspector_open;
            }
            UiMessage::ToggleEditorDrawer => {
                self.workspace.editor_drawer_open = !self.workspace.editor_drawer_open;
            }
            UiMessage::ToggleMixerDrawer => {
                self.workspace.mixer_drawer_open = !self.workspace.mixer_drawer_open;
            }
            UiMessage::ToggleLoopBrowser => {
                self.workspace.loop_browser_open = !self.workspace.loop_browser_open;
            }
            UiMessage::ToggleFileBrowser => {
                self.workspace.file_browser_open = !self.workspace.file_browser_open;
            }
            UiMessage::ToggleLibraryBrowser => {
                self.workspace.library_browser_open = !self.workspace.library_browser_open;
            }
            UiMessage::SelectEditorTab(tab) => {
                self.workspace.active_editor = tab;
                self.workspace.editor_drawer_open = true;
            }
            UiMessage::SelectListTab(tab) => {
                self.workspace.active_list_tab = tab;
            }
            UiMessage::SetLcdMode(mode) => {
                self.workspace.lcd_mode = mode;
            }

            UiMessage::TransportPlay => {
                self.is_playing = true;
                self.is_recording = false;
            }
            UiMessage::TransportPause => {
                self.is_playing = false;
            }
            UiMessage::TransportStop => {
                self.is_playing = false;
                self.is_recording = false;
                self.playhead_tick = 0;
            }
            UiMessage::TransportRecord => {
                self.is_playing = true;
                self.is_recording = true;
            }
            UiMessage::TransportToggleLoop => {
                self.is_looping = !self.is_looping;
            }
            UiMessage::SetPlayhead(tick) => {
                self.playhead_tick = tick;
                self.arrangement.update_playhead_view(tick);
            }

            // Hardened Tempo Input: sanitize NaNs & clamp to [10.0, 999.0] BPM
            UiMessage::SetTempoBpm(bpm) => {
                let safe_bpm = if bpm.is_finite() {
                    bpm.clamp(10.0, 999.0)
                } else {
                    120.0
                };
                self.workspace.lcd_diagnostic.project_tempo_bpm = safe_bpm;
            }
            UiMessage::SetTimeSignature(num, den) => {
                let safe_num = num.clamp(1, 32);
                let safe_den = if [1, 2, 4, 8, 16, 32].contains(&den) {
                    den
                } else {
                    4
                };
                self.workspace.lcd_diagnostic.time_signature = (safe_num, safe_den);
            }

            // Hardened Track Selection: safe indexing
            UiMessage::SelectTrack(idx) => {
                self.workspace.selected_track_id = idx;
                self.inspector.update_selection(idx, 0);
                self.arrangement.select_track(idx);
            }

            // Hardened Zoom: clamp to [0.1, 50.0]
            UiMessage::SetHorizontalZoom(zoom) => {
                let safe_zoom = if zoom.is_finite() {
                    zoom.clamp(0.1, 50.0)
                } else {
                    1.0
                };
                self.arrangement.horizontal_zoom = safe_zoom;
            }
            UiMessage::SetVerticalZoom(zoom) => {
                let safe_zoom = if zoom.is_finite() {
                    zoom.clamp(0.1, 50.0)
                } else {
                    1.0
                };
                self.arrangement.vertical_zoom = safe_zoom;
            }
            UiMessage::SetCatchPlayhead(mode) => {
                self.arrangement.set_catch_playhead(mode);
            }

            // Hardened Fader & Pan: sanitize NaNs, Infs, and clamp
            UiMessage::SetTrackFader { track_id, volume_db } => {
                let safe_db = if volume_db.is_finite() {
                    volume_db.clamp(-120.0, 6.0)
                } else {
                    -120.0
                };
                if let Some(strip) = self.mixer.channel_strips.get_mut(track_id) {
                    strip.volume_fader_db = safe_db;
                }
            }
            UiMessage::SetTrackPan { track_id, pan } => {
                let safe_pan = if pan.is_finite() {
                    pan.clamp(-1.0, 1.0)
                } else {
                    0.0
                };
                if let Some(strip) = self.mixer.channel_strips.get_mut(track_id) {
                    strip.pan.balance = safe_pan;
                }
            }
            UiMessage::ToggleTrackMute(track_id) => {
                if let Some(strip) = self.mixer.channel_strips.get_mut(track_id) {
                    strip.muted = !strip.muted;
                }
            }
            UiMessage::ToggleTrackSolo(track_id) => {
                if let Some(strip) = self.mixer.channel_strips.get_mut(track_id) {
                    strip.solo = !strip.solo;
                }
            }
            UiMessage::ResetAutoMute(track_id) => {
                if let Some(strip) = self.mixer.channel_strips.get_mut(track_id) {
                    strip.reset_auto_mute();
                }
            }

            UiMessage::TelemetryReceived(batch) => {
                self.playhead_tick = batch.playhead_tick;
                self.workspace.lcd_diagnostic.cpu_load_percent = batch.cpu_load_percent;
                for meter in batch.meters {
                    if meter.auto_mute_tripped {
                        if let Some(strip) = self.mixer.channel_strips.get_mut(meter.channel_id) {
                            strip.auto_mute_tripped = true;
                        }
                    }
                }
            }
            UiMessage::Tick => {}
            UiMessage::NewProject => {
                self.playhead_tick = 0;
                self.is_playing = false;
                self.dirty = false;
            }
            UiMessage::SaveProject => {
                self.dirty = false;
            }
        }
    }
}
