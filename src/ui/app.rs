use crate::audio::hardware_driver::AudioHardwareDriver;
use crate::mixing::MixingEngine;
use crate::timeline::Timeline;
use crate::timeline::arrangement_view::ArrangementView;
use crate::timeline::region::{Region, RegionKind};
use crate::timeline::track::{Track, TrackType};
use crate::ui::message::UiMessage;
use crate::ui_state::{DualChannelInspector, WorkspaceState};
use rtrb::Producer;

pub struct CrystalWaveApp {
    pub workspace: WorkspaceState,
    pub inspector: DualChannelInspector,
    pub arrangement: ArrangementView,
    pub mixer: MixingEngine,
    pub audio_driver: Option<AudioHardwareDriver>,
    pub audio_producer: Option<Producer<f32>>,
    pub is_playing: bool,
    pub is_recording: bool,
    pub is_looping: bool,
    pub playhead_tick: u64,
    pub dirty: bool,
}

impl CrystalWaveApp {
    pub fn new(sample_rate: f32) -> Self {
        let mut timeline = Timeline::new();
        let mut trk1 = Track::new(0, "Audio 1", TrackType::Audio);
        trk1.regions.push(Region {
            id: 1,
            track_id: 0,
            name: "Audio Take 1".to_string(),
            start_tick: 0,
            length_ticks: 3840,
            anchor_offset_ticks: 0,
            muted: false,
            looped: false,
            loop_length_ticks: None,
            gain_db: 0.0,
            transpose_semitones: 0.0,
            reversed: false,
            fade: Default::default(),
            kind: RegionKind::Audio {
                file_path: None,
                sample_offset: 0,
                sample_length: (sample_rate * 4.0) as usize,
            },
        });

        let mut trk2 = Track::new(1, "Inst 1", TrackType::Software);
        trk2.regions.push(Region {
            id: 2,
            track_id: 1,
            name: "MIDI Groove".to_string(),
            start_tick: 1920,
            length_ticks: 3840,
            anchor_offset_ticks: 0,
            muted: false,
            looped: false,
            loop_length_ticks: None,
            gain_db: 0.0,
            transpose_semitones: 0.0,
            reversed: false,
            fade: Default::default(),
            kind: RegionKind::Midi { clip_id: 1 },
        });

        let _ = timeline.add_track(trk1);
        let _ = timeline.add_track(trk2);
        let arrangement = ArrangementView::new(timeline);
        let mut mixer = MixingEngine::new(sample_rate).expect("Mixer init");
        let _ = mixer.add_channel("Audio 1");
        let _ = mixer.add_channel("Inst 1");

        let mut audio_driver = AudioHardwareDriver::new().ok();
        let audio_producer = audio_driver.as_mut().and_then(|d| d.start_stream(4096).ok());

        Self {
            workspace: WorkspaceState::default(),
            inspector: DualChannelInspector::new(0, 0),
            arrangement,
            mixer,
            audio_driver,
            audio_producer,
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
                if let Some(ref mut driver) = self.audio_driver {
                    let _ = driver.resume();
                }
            }
            UiMessage::TransportPause => {
                self.is_playing = false;
                if let Some(ref mut driver) = self.audio_driver {
                    let _ = driver.pause();
                }
            }
            UiMessage::TransportStop => {
                self.is_playing = false;
                self.is_recording = false;
                self.playhead_tick = 0;
                if let Some(ref mut driver) = self.audio_driver {
                    let _ = driver.pause();
                }
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
            UiMessage::SetTrackFader {
                track_id,
                volume_db,
            } => {
                let safe_db = if volume_db.is_finite() {
                    volume_db.clamp(-120.0, 6.0)
                } else {
                    -120.0
                };
                if let Some(strip) = self.mixer.channels.get_mut(track_id) {
                    strip.volume_fader_db = safe_db;
                }
            }
            UiMessage::SetTrackPan { track_id, pan } => {
                let safe_pan = if pan.is_finite() {
                    pan.clamp(-1.0, 1.0)
                } else {
                    0.0
                };
                if let Some(strip) = self.mixer.channels.get_mut(track_id) {
                    strip.pan.balance = safe_pan;
                }
            }
            UiMessage::ToggleTrackMute(track_id) => {
                if let Some(strip) = self.mixer.channels.get_mut(track_id) {
                    strip.muted = !strip.muted;
                }
            }
            UiMessage::ToggleTrackSolo(track_id) => {
                if let Some(strip) = self.mixer.channels.get_mut(track_id) {
                    strip.solo = !strip.solo;
                }
            }
            UiMessage::ResetAutoMute(track_id) => {
                if let Some(strip) = self.mixer.channels.get_mut(track_id) {
                    strip.meter.reset_auto_mute();
                }
            }

            UiMessage::TelemetryReceived(batch) => {
                self.playhead_tick = batch.playhead_tick;
                self.workspace.lcd_diagnostic.cpu_load_percent = batch.cpu_load_percent;
                for meter in batch.meters {
                    if let Some(strip) = meter
                        .auto_mute_tripped
                        .then(|| self.mixer.channels.get_mut(meter.channel_id))
                        .flatten()
                    {
                        strip.meter.auto_mute_tripped = true;
                    }
                }
            }
            UiMessage::KeyboardFocusCommand(cmd) => {
                let cursor_tick = self.playhead_tick;
                let trk_id = self.workspace.selected_track_id;
                if let Some(track) = self.arrangement.timeline.tracks.get_mut(trk_id) {
                    let mut new_region = None;
                    for r in track.regions.iter_mut() {
                        if cursor_tick > r.start_tick && cursor_tick < r.start_tick + r.length_ticks {
                            match cmd {
                                crate::pro_tools::KeyboardFocusCommand::SplitAtCursor => {
                                    let first_len = cursor_tick - r.start_tick;
                                    let second_len = r.length_ticks - first_len;
                                    r.length_ticks = first_len;
                                    let mut split_r = r.clone();
                                    split_r.id = track.regions.len() + 1000;
                                    split_r.name = format!("{}_split", r.name);
                                    split_r.start_tick = cursor_tick;
                                    split_r.length_ticks = second_len;
                                    new_region = Some(split_r);
                                    break;
                                }
                                crate::pro_tools::KeyboardFocusCommand::TrimStartToCursor => {
                                    let delta = cursor_tick - r.start_tick;
                                    r.start_tick = cursor_tick;
                                    r.length_ticks = r.length_ticks.saturating_sub(delta);
                                    break;
                                }
                                crate::pro_tools::KeyboardFocusCommand::TrimEndToCursor => {
                                    r.length_ticks = cursor_tick - r.start_tick;
                                    break;
                                }
                                crate::pro_tools::KeyboardFocusCommand::FadeInToCursor => {
                                    r.fade.in_ticks = cursor_tick - r.start_tick;
                                    break;
                                }
                                crate::pro_tools::KeyboardFocusCommand::FadeOutFromCursor => {
                                    r.fade.out_ticks =
                                        (r.start_tick + r.length_ticks).saturating_sub(cursor_tick);
                                    break;
                                }
                                crate::pro_tools::KeyboardFocusCommand::CrossfadeSelection => break,
                            }
                        }
                    }
                    if let Some(nr) = new_region {
                        track.regions.push(nr);
                    }
                }
                self.dirty = true;
            }
            UiMessage::ToggleVcaSpill(_vca_id) => {
                self.dirty = true;
            }
            UiMessage::SetTrackColorCascade {
                track_id,
                color_rgb,
            } => {
                if let Some(track) = self.arrangement.timeline.tracks.get_mut(track_id) {
                    track.color = crate::timeline::track::Color {
                        r: color_rgb[0],
                        g: color_rgb[1],
                        b: color_rgb[2],
                    };
                }
                self.dirty = true;
            }

            UiMessage::Tick => {
                if self.is_playing {
                    self.playhead_tick = self.playhead_tick.saturating_add(48);
                    self.arrangement.update_playhead_view(self.playhead_tick);

                    let mut left = [0.0f32; 128];
                    let mut right = [0.0f32; 128];
                    self.mixer.sum_buses(&mut left, &mut right);

                    if let Some(ref mut prod) = self.audio_producer {
                        for i in 0..128 {
                            let _ = prod.push(left[i]);
                            let _ = prod.push(right[i]);
                        }
                    }
                }
            }

            UiMessage::NewProject => {
                self.playhead_tick = 0;
                self.is_playing = false;
                self.dirty = false;
            }
            UiMessage::SaveProject => {
                self.dirty = false;
            }
            UiMessage::MenuAction(cmd) => match cmd {
                crate::ui::message::MenuCommand::NewProject => {
                    self.playhead_tick = 0;
                    self.is_playing = false;
                    self.dirty = false;
                }
                crate::ui::message::MenuCommand::SaveSession
                | crate::ui::message::MenuCommand::SaveSessionAs => {
                    self.dirty = false;
                }
                crate::ui::message::MenuCommand::ToggleMixer => {
                    self.workspace.mixer_drawer_open = !self.workspace.mixer_drawer_open;
                }
                crate::ui::message::MenuCommand::TogglePianoRoll => {
                    self.workspace.editor_drawer_open = true;
                    self.workspace.active_editor = crate::ui_state::EditorDrawerTab::PianoRoll;
                }
                crate::ui::message::MenuCommand::ToggleInspector => {
                    self.workspace.inspector_open = !self.workspace.inspector_open;
                }
                crate::ui::message::MenuCommand::AddAudioTrack => {
                    let name = format!("Audio {}", self.mixer.channels.len() + 1);
                    let _ = self.mixer.add_channel(name.clone());
                    let trk_id = self.arrangement.timeline.tracks.len();
                    let _ = self
                        .arrangement
                        .timeline
                        .add_track(Track::new(trk_id, name, TrackType::Audio));
                    self.dirty = true;
                }
                crate::ui::message::MenuCommand::AddMidiTrack => {
                    let name = format!("Inst {}", self.mixer.channels.len() + 1);
                    let _ = self.mixer.add_channel(name.clone());
                    let trk_id = self.arrangement.timeline.tracks.len();
                    let _ = self
                        .arrangement
                        .timeline
                        .add_track(Track::new(trk_id, name, TrackType::Software));
                    self.dirty = true;
                }
                crate::ui::message::MenuCommand::SplitAtCursor => {
                    self.update(UiMessage::KeyboardFocusCommand(
                        crate::pro_tools::KeyboardFocusCommand::SplitAtCursor,
                    ));
                }
                _ => {
                    self.dirty = true;
                }
            },
        }
    }
}
