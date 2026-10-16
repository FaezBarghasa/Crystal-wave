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
    pub plugin_registry: crate::plugins::PluginRegistry,
    pub fl_windows: crate::fl_studio::FlWindowManager,
    pub channel_rack: crate::fl_studio::ChannelRack,
    pub browser: crate::fl_studio::FlBrowser,
    pub patcher: crate::fl_studio::FlPatcher,
    pub piano_roll_notes: Vec<crate::fl_studio::FlPianoRollNote>,
}

impl Default for CrystalWaveApp {
    fn default() -> Self {
        Self::new(44100.0)
    }
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
        let audio_producer = audio_driver
            .as_mut()
            .and_then(|d| d.start_stream(4096).ok());

        let mut channel_rack = crate::fl_studio::ChannelRack::new(16);
        let kick_idx = channel_rack.add_channel("808 Kick");
        let snare_idx = channel_rack.add_channel("Clap / Snare");
        let hat_idx = channel_rack.add_channel("Hi-Hat");
        let bass_idx = channel_rack.add_channel("Spinz Bass");

        // Pre-activate standard 4-on-the-floor and trap rhythm
        channel_rack.toggle_step(kick_idx, 0);
        channel_rack.toggle_step(kick_idx, 8);
        channel_rack.toggle_step(snare_idx, 4);
        channel_rack.toggle_step(snare_idx, 12);
        for i in (0..16).step_by(2) {
            channel_rack.toggle_step(hat_idx, i);
        }
        channel_rack.toggle_step(bass_idx, 0);
        channel_rack.toggle_step(bass_idx, 6);

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
            plugin_registry: crate::plugins::PluginRegistry::new(),
            fl_windows: crate::fl_studio::FlWindowManager::new(),
            channel_rack,
            browser: crate::fl_studio::FlBrowser::new(),
            patcher: crate::fl_studio::FlPatcher::default(),
            piano_roll_notes: vec![
                crate::fl_studio::FlPianoRollNote::new(1, 0, 60, 0, 960), // C4
                crate::fl_studio::FlPianoRollNote::new(2, 0, 64, 0, 960), // E4
                crate::fl_studio::FlPianoRollNote::new(3, 0, 67, 0, 960), // G4
                crate::fl_studio::FlPianoRollNote::new(4, 0, 71, 0, 960), // B4
                crate::fl_studio::FlPianoRollNote::new(5, 0, 72, 960, 960).with_slide(true), // C5 Slide
            ],
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
            UiMessage::SetMasterTuning(hz) => {
                let safe_hz = if hz.is_finite() {
                    hz.clamp(400.0, 480.0)
                } else {
                    440.0
                };
                self.workspace.lcd_diagnostic.master_tuning_hz = safe_hz;
                self.dirty = true;
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
                    let current_regions_len = track.regions.len();
                    for r in track.regions.iter_mut() {
                        if cursor_tick > r.start_tick && cursor_tick < r.start_tick + r.length_ticks
                        {
                            match cmd {
                                crate::pro_tools::KeyboardFocusCommand::SplitAtCursor => {
                                    let first_len = cursor_tick - r.start_tick;
                                    let second_len = r.length_ticks - first_len;
                                    r.length_ticks = first_len;
                                    let mut split_r = r.clone();
                                    split_r.id = current_regions_len + 1000;
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

                    if let Some(ref mut prod) = self.audio_producer {
                        let mut frames_pushed = 0;
                        while prod.slots() >= 256 && frames_pushed < 2048 {
                            let mut left = [0.0f32; 128];
                            let mut right = [0.0f32; 128];
                            self.mixer.sum_buses(&mut left, &mut right);
                            for i in 0..128 {
                                let _ = prod.push(left[i]);
                                let _ = prod.push(right[i]);
                            }
                            frames_pushed += 128;
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
                    self.arrangement.timeline.tracks.clear();
                    self.mixer.channels.clear();
                    let _ = self.mixer.add_channel("Master");
                }
                crate::ui::message::MenuCommand::SaveSession
                | crate::ui::message::MenuCommand::SaveSessionAs => {
                    let project = crate::format::project_format::CrystalWaveProject::default();
                    if let Ok(json) = crate::format::ProjectSerializer::to_json_string(&project) {
                        let _ = std::fs::write("session_backup.cwave.json", json);
                    }
                    self.dirty = false;
                }
                crate::ui::message::MenuCommand::BounceWav => {
                    let exporter = crate::bouncing::Exporter::new(
                        crate::bouncing::export_format::ExportFormatConfig::default(),
                        crate::bouncing::bit_depth_converter::BitDepth::Bit24,
                        crate::bouncing::dithering::DitherType::None,
                    );
                    let mut dummy_l = vec![0.0f32; 44100 * 2];
                    let mut dummy_r = vec![0.0f32; 44100 * 2];
                    self.mixer.sum_buses(&mut dummy_l, &mut dummy_r);
                    if let Ok(wav_bytes) = exporter.export_stereo_wav(&dummy_l, &dummy_r, 44100.0) {
                        let _ = std::fs::write("bounce_output.wav", wav_bytes);
                    }
                    self.dirty = false;
                }
                crate::ui::message::MenuCommand::ExportAaf => {
                    let session = crate::format::AafSession::new(
                        "ExportedSession",
                        44100.0,
                        crate::format::SmpteFrameRate::Fps24,
                    );
                    let _ = std::fs::write("session_export.edl", format!("{:?}", session));
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
                    let _ = self.arrangement.timeline.add_track(Track::new(
                        trk_id,
                        name,
                        TrackType::Audio,
                    ));
                    self.dirty = true;
                }
                crate::ui::message::MenuCommand::AddMidiTrack => {
                    let name = format!("Inst {}", self.mixer.channels.len() + 1);
                    let _ = self.mixer.add_channel(name.clone());
                    let trk_id = self.arrangement.timeline.tracks.len();
                    let _ = self.arrangement.timeline.add_track(Track::new(
                        trk_id,
                        name,
                        TrackType::Software,
                    ));
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

            UiMessage::RemovePlugin(id) => {
                self.plugin_registry.remove_plugin(id);
                self.dirty = true;
            }
            UiMessage::EnablePlugin(id) => {
                self.plugin_registry.enable_plugin(id);
            }
            UiMessage::DisablePlugin(id) => {
                self.plugin_registry.disable_plugin(id);
            }

            UiMessage::ToggleFlChannelRack => {
                self.fl_windows.toggle_channel_rack();
            }
            UiMessage::ToggleFlPianoRoll => {
                self.fl_windows.toggle_piano_roll();
            }
            UiMessage::ToggleFlPlaylist => {
                self.fl_windows.toggle_playlist();
            }
            UiMessage::ToggleFlMixer => {
                self.fl_windows.toggle_mixer();
            }
            UiMessage::ToggleFlBrowser => {
                self.fl_windows.toggle_browser();
            }
            UiMessage::ToggleFlPatcher => {
                self.fl_windows.toggle_patcher();
            }

            UiMessage::ChannelRackToggleStep {
                channel_id,
                step_index,
            } => {
                self.channel_rack.toggle_step(channel_id, step_index);
                self.dirty = true;
            }

            UiMessage::ChannelRackDropSample { sample_id } => {
                let _ = self
                    .browser
                    .drop_to_channel_rack(&mut self.channel_rack, sample_id);
                self.dirty = true;
            }

            UiMessage::TimelineMoveClip {
                clip_id,
                new_track_id,
                new_start_tick,
            } => {
                // Find and extract region
                let mut found_region = None;
                for track in &mut self.arrangement.timeline.tracks {
                    if let Some(pos) = track.regions.iter().position(|r| r.id == clip_id) {
                        let mut reg = track.regions.remove(pos);
                        reg.start_tick = new_start_tick;
                        reg.track_id = new_track_id;
                        found_region = Some(reg);
                        break;
                    }
                }
                // Insert into new destination track
                if let Some(reg) = found_region
                    && let Some(dest_track) = self.arrangement.timeline.tracks.get_mut(new_track_id)
                {
                    dest_track.regions.push(reg);
                }
                self.dirty = true;
            }

            UiMessage::TimelineTrimClipStart {
                clip_id,
                new_start_tick,
            } => {
                for track in &mut self.arrangement.timeline.tracks {
                    if let Some(reg) = track.regions.iter_mut().find(|r| r.id == clip_id) {
                        let delta = new_start_tick.saturating_sub(reg.start_tick);
                        reg.start_tick = new_start_tick;
                        reg.length_ticks = reg.length_ticks.saturating_sub(delta).max(240);
                        break;
                    }
                }
                self.dirty = true;
            }

            UiMessage::TimelineTrimClipEnd {
                clip_id,
                new_length_ticks,
            } => {
                for track in &mut self.arrangement.timeline.tracks {
                    if let Some(reg) = track.regions.iter_mut().find(|r| r.id == clip_id) {
                        reg.length_ticks = new_length_ticks.max(240);
                        break;
                    }
                }
                self.dirty = true;
            }

            UiMessage::TimelineFadeClip {
                clip_id,
                fade_in_ticks,
                fade_out_ticks,
            } => {
                for track in &mut self.arrangement.timeline.tracks {
                    if let Some(reg) = track.regions.iter_mut().find(|r| r.id == clip_id) {
                        if let Some(fi) = fade_in_ticks {
                            reg.fade.in_ticks = fi;
                        }
                        if let Some(fo) = fade_out_ticks {
                            reg.fade.out_ticks = fo;
                        }
                        break;
                    }
                }
                self.dirty = true;
            }

            UiMessage::PianoRollQuickStrum => {
                crate::fl_studio::FlPianoRollTools::quick_strum(
                    &mut self.piano_roll_notes,
                    15, // 15 ticks human strumming offset
                    true,
                );
                self.dirty = true;
            }
            UiMessage::PianoRollFlipHorizontal => {
                let pivot = self
                    .piano_roll_notes
                    .iter()
                    .map(|n| n.start_tick)
                    .max()
                    .unwrap_or(960)
                    / 2;
                crate::fl_studio::FlPianoRollTools::flip_horizontal(
                    &mut self.piano_roll_notes,
                    pivot,
                );
                self.dirty = true;
            }
            UiMessage::PianoRollFlipVertical => {
                crate::fl_studio::FlPianoRollTools::flip_vertical(&mut self.piano_roll_notes, 64); // E4 harmonic pivot
                self.dirty = true;
            }
            UiMessage::PianoRollQuickChop => {
                let mut chopped_notes = Vec::new();
                for note in &self.piano_roll_notes {
                    let mut subs = crate::fl_studio::FlPianoRollTools::quick_chop(note, 2);
                    chopped_notes.append(&mut subs);
                }
                self.piano_roll_notes = chopped_notes;
                self.dirty = true;
            }
        }
    }

    /// Event subscription listening for hardware keyboard shortcuts and smooth GUI clock
    pub fn subscription(&self) -> iced::Subscription<UiMessage> {
        let clock = iced::window::frames().map(|_| UiMessage::Tick);
        let keys = iced::event::listen_with(|event, _status, _window| {
            if let iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, .. }) = event {
                match key {
                    iced::keyboard::Key::Named(iced::keyboard::key::Named::Space) => {
                        Some(UiMessage::TransportPlay)
                    }
                    iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter) => {
                        Some(UiMessage::SetPlayhead(0))
                    }
                    iced::keyboard::Key::Named(iced::keyboard::key::Named::F5) => {
                        Some(UiMessage::ToggleFlPlaylist)
                    }
                    iced::keyboard::Key::Named(iced::keyboard::key::Named::F6) => {
                        Some(UiMessage::ToggleFlChannelRack)
                    }
                    iced::keyboard::Key::Named(iced::keyboard::key::Named::F7) => {
                        Some(UiMessage::ToggleFlPianoRoll)
                    }
                    iced::keyboard::Key::Named(iced::keyboard::key::Named::F8) => {
                        Some(UiMessage::ToggleFlBrowser)
                    }
                    iced::keyboard::Key::Named(iced::keyboard::key::Named::F9) => {
                        Some(UiMessage::ToggleFlMixer)
                    }
                    iced::keyboard::Key::Character(ref c) if c.eq_ignore_ascii_case("i") => {
                        Some(UiMessage::ToggleInspector)
                    }
                    iced::keyboard::Key::Character(ref c) if c.eq_ignore_ascii_case("l") => {
                        Some(UiMessage::ToggleLibraryBrowser)
                    }
                    iced::keyboard::Key::Character(ref c) if c.eq_ignore_ascii_case("r") => {
                        Some(UiMessage::TransportRecord)
                    }
                    iced::keyboard::Key::Character(ref c) if c.eq_ignore_ascii_case("b") => Some(
                        UiMessage::MenuAction(crate::ui::message::MenuCommand::SplitAtCursor),
                    ),
                    iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) => {
                        Some(UiMessage::TransportStop)
                    }
                    _ => None,
                }
            } else {
                None
            }
        });

        iced::Subscription::batch([clock, keys])
    }
}
