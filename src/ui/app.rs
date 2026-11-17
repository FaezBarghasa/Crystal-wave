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
    pub audio_worker: Option<crate::audio::AudioWorkerHandle>,
    pub is_playing: bool,
    pub is_recording: bool,
    pub is_looping: bool,
    pub playhead_tick: u64,
    pub playhead_frame: u64,
    pub audio_scheduler: crate::audio::TimelineAudioScheduler,
    pub dirty: bool,
    pub plugin_registry: crate::plugins::PluginRegistry,
    pub fl_windows: crate::fl_studio::FlWindowManager,
    pub channel_rack: crate::fl_studio::ChannelRack,
    pub browser: crate::fl_studio::FlBrowser,
    pub patcher: crate::fl_studio::FlPatcher,
    pub piano_roll_notes: Vec<crate::fl_studio::FlPianoRollNote>,
    pub theme: crate::ui::theme::UiTheme,
    pub layout: crate::ui::layout::LayoutState,
    pub snap_grid: crate::ui::canvas_timeline::SnapGrid,
    pub timeline_scroll_y: f32,
    pub timeline_scroll_x: f32,
    pub piano_roll_scroll_y: f32,
    pub piano_roll_selected_channel: usize,
    pub piano_roll_scale: crate::fl_studio::piano_roll::MusicalScale,
    pub piano_roll_scale_root: u8,
    pub piano_roll_event_target: crate::fl_studio::piano_roll::EventLaneTarget,
    pub marquee_selection: Option<(u64, u64, usize, usize)>,
    pub selected_mixer_track: usize,
    pub command_palette_open: bool,
    pub command_query: String,
    pub plugin_picker_search: String,
    pub plugin_picker_category: Option<crate::plugins::plugin_descriptor::PluginCategory>,
    pub plugin_picker_target_slot: Option<(usize, usize)>,
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

        let audio_worker = Some(crate::audio::spawn_audio_worker(
            sample_rate,
            256,
            audio_producer,
        ));

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

        let audio_scheduler = crate::audio::TimelineAudioScheduler::new(sample_rate, 120.0);

        Self {
            workspace: WorkspaceState::default(),
            inspector: DualChannelInspector::new(0, 0),
            arrangement,
            mixer,
            audio_driver,
            audio_producer: None,
            audio_worker,
            is_playing: false,
            is_recording: false,
            is_looping: false,
            playhead_tick: 0,
            playhead_frame: 0,
            audio_scheduler,
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
            theme: crate::ui::theme::UiTheme::default(),
            layout: crate::ui::layout::LayoutState::default(),
            snap_grid: crate::ui::canvas_timeline::SnapGrid::Sixteenth,
            timeline_scroll_y: 0.0,
            timeline_scroll_x: 0.0,
            piano_roll_scroll_y: 872.0, // Center C4 (key 60) in default viewport
            piano_roll_selected_channel: 0,
            piano_roll_scale: crate::fl_studio::piano_roll::MusicalScale::default(),
            piano_roll_scale_root: 0,
            piano_roll_event_target: crate::fl_studio::piano_roll::EventLaneTarget::Velocity,
            marquee_selection: None,
            selected_mixer_track: 0,
            command_palette_open: false,
            command_query: String::new(),
            plugin_picker_search: String::new(),
            plugin_picker_category: None,
            plugin_picker_target_slot: None,
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
                if let Some(ref mut worker) = self.audio_worker {
                    worker.play();
                }
                if let Some(ref mut driver) = self.audio_driver {
                    let _ = driver.resume();
                }
            }
            UiMessage::TransportPause => {
                self.is_playing = false;
                if let Some(ref mut worker) = self.audio_worker {
                    worker.pause();
                }
                if let Some(ref mut driver) = self.audio_driver {
                    let _ = driver.pause();
                }
            }
            UiMessage::TransportStop => {
                self.is_playing = false;
                self.is_recording = false;
                self.playhead_tick = 0;
                self.playhead_frame = 0;
                self.arrangement.update_playhead_view(0);
                if let Some(ref mut worker) = self.audio_worker {
                    worker.stop();
                }
                if let Some(ref mut driver) = self.audio_driver {
                    let _ = driver.pause();
                }
            }
            UiMessage::TransportRecord => {
                self.is_playing = true;
                self.is_recording = true;
                if let Some(ref mut worker) = self.audio_worker {
                    worker.play();
                }
                if let Some(ref mut driver) = self.audio_driver {
                    let _ = driver.resume();
                }
            }
            UiMessage::TransportToggleLoop => {
                self.is_looping = !self.is_looping;
                self.arrangement.timeline.loop_region.enabled = self.is_looping;
            }
            UiMessage::SetPlayhead(tick) => {
                self.playhead_tick = tick;
                self.playhead_frame = self.audio_scheduler.tick_to_frame(tick);
                if let Some(ref mut worker) = self.audio_worker {
                    worker.seek(self.playhead_frame);
                }
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
                self.audio_scheduler.set_tempo(safe_bpm);
                if let Some(ref mut worker) = self.audio_worker {
                    worker.set_tempo(safe_bpm);
                }
            }
            UiMessage::SetSwing(swing) => {
                let safe_swing = if swing.is_finite() {
                    swing.clamp(0.0, 1.0)
                } else {
                    0.0
                };
                self.audio_scheduler.set_swing(safe_swing);
                if let Some(ref mut worker) = self.audio_worker {
                    worker.set_swing(safe_swing);
                }
                self.dirty = true;
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
                if track_id == 9999 {
                    self.mixer.master_strip.volume_fader_db = safe_db;
                    self.mixer.master_fader_db = safe_db;
                } else {
                    if let Some(strip) = self.mixer.channels.get_mut(track_id) {
                        strip.volume_fader_db = safe_db;
                    }
                    if let Some(track) = self.arrangement.timeline.tracks.get_mut(track_id) {
                        track.gain_db = safe_db;
                    }
                }
                self.dirty = true;
            }
            UiMessage::SetTrackPan { track_id, pan } => {
                let safe_pan = if pan.is_finite() {
                    pan.clamp(-1.0, 1.0)
                } else {
                    0.0
                };
                if track_id == 9999 {
                    self.mixer.master_strip.pan.balance = safe_pan;
                } else {
                    if let Some(strip) = self.mixer.channels.get_mut(track_id) {
                        strip.pan.balance = safe_pan;
                    }
                    if let Some(track) = self.arrangement.timeline.tracks.get_mut(track_id) {
                        track.pan = safe_pan;
                    }
                }
                self.dirty = true;
            }
            UiMessage::ToggleTrackMute(track_id) => {
                if track_id == 9999 {
                    self.mixer.master_strip.muted = !self.mixer.master_strip.muted;
                } else {
                    if let Some(strip) = self.mixer.channels.get_mut(track_id) {
                        strip.muted = !strip.muted;
                    }
                    if let Some(track) = self.arrangement.timeline.tracks.get_mut(track_id) {
                        track.muted = !track.muted;
                    }
                }
                self.dirty = true;
            }
            UiMessage::ToggleTrackSolo(track_id) => {
                if let Some(strip) = self.mixer.channels.get_mut(track_id) {
                    strip.solo = !strip.solo;
                }
                if let Some(track) = self.arrangement.timeline.tracks.get_mut(track_id) {
                    track.soloed = !track.soloed;
                }
                self.dirty = true;
            }
            UiMessage::ResetAutoMute(track_id) => {
                if track_id == 9999 {
                    self.mixer.master_strip.meter.reset_auto_mute();
                } else if let Some(strip) = self.mixer.channels.get_mut(track_id) {
                    strip.meter.reset_auto_mute();
                }
                self.dirty = true;
            }
            UiMessage::SelectMixerTrack(track_id) => {
                self.selected_mixer_track = track_id;
            }
            UiMessage::SetTrackStereoSeparation {
                track_id,
                separation,
            } => {
                let safe_sep = if separation.is_finite() {
                    separation.clamp(-1.0, 1.0)
                } else {
                    0.0
                };
                if track_id == 9999 {
                    self.mixer.master_strip.set_stereo_separation(safe_sep);
                } else if let Some(strip) = self.mixer.channels.get_mut(track_id) {
                    strip.set_stereo_separation(safe_sep);
                }
                self.dirty = true;
            }
            UiMessage::ToggleTrackPhaseInvert(track_id) => {
                if track_id == 9999 {
                    self.mixer.master_strip.toggle_phase_invert();
                } else if let Some(strip) = self.mixer.channels.get_mut(track_id) {
                    strip.toggle_phase_invert();
                }
                self.dirty = true;
            }
            UiMessage::ToggleTrackSwapChannels(track_id) => {
                if track_id == 9999 {
                    self.mixer.master_strip.toggle_swap_channels();
                } else if let Some(strip) = self.mixer.channels.get_mut(track_id) {
                    strip.toggle_swap_channels();
                }
                self.dirty = true;
            }
            UiMessage::ToggleInsertSlotBypass { track_id, slot_id } => {
                if slot_id < crate::mixing::NUM_INSERT_SLOTS {
                    if track_id == 9999 {
                        let bypassed = !self.mixer.master_strip.slots[slot_id].bypassed;
                        self.mixer.master_strip.set_slot_bypassed(slot_id, bypassed);
                    } else if let Some(strip) = self.mixer.channels.get_mut(track_id) {
                        let bypassed = !strip.slots[slot_id].bypassed;
                        strip.set_slot_bypassed(slot_id, bypassed);
                    }
                    self.dirty = true;
                }
            }
            UiMessage::SetInsertSlotMix {
                track_id,
                slot_id,
                mix,
            } => {
                let safe_mix = if mix.is_finite() {
                    mix.clamp(0.0, 1.0)
                } else {
                    1.0
                };
                if slot_id < crate::mixing::NUM_INSERT_SLOTS {
                    if track_id == 9999 {
                        self.mixer.master_strip.set_slot_mix(slot_id, safe_mix);
                    } else if let Some(strip) = self.mixer.channels.get_mut(track_id) {
                        strip.set_slot_mix(slot_id, safe_mix);
                    }
                    self.dirty = true;
                }
            }
            UiMessage::ToggleMasterSoftClipper => {
                self.mixer.master_soft_clipper_enabled = !self.mixer.master_soft_clipper_enabled;
                self.dirty = true;
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
                    let bpm = self.workspace.lcd_diagnostic.project_tempo_bpm;
                    self.audio_scheduler.set_tempo(bpm);
                    let block_size = 256; // 256 stereo frames = 512 samples

                    if let Some(ref worker) = self.audio_worker {
                        let frame = worker.playhead_frame();
                        if frame > self.playhead_frame {
                            self.playhead_frame = frame;
                        } else {
                            self.playhead_frame =
                                self.playhead_frame.saturating_add(block_size as u64);
                        }
                        self.workspace.lcd_diagnostic.cpu_load_percent = worker.cpu_load();
                    } else if let Some(ref mut prod) = self.audio_producer {
                        let mut frames_processed = 0;
                        while prod.slots() >= block_size * 2 && frames_processed < 2048 {
                            let mut master_l = [0.0f32; 256];
                            let mut master_r = [0.0f32; 256];

                            self.audio_scheduler.process_block(
                                &self.arrangement.timeline.tracks,
                                &self.piano_roll_notes,
                                &mut self.mixer,
                                self.playhead_frame,
                                block_size,
                                &mut master_l,
                                &mut master_r,
                            );

                            for i in 0..block_size {
                                let _ = prod.push(master_l[i]);
                                let _ = prod.push(master_r[i]);
                            }

                            self.playhead_frame =
                                self.playhead_frame.saturating_add(block_size as u64);
                            frames_processed += block_size;
                        }

                        if frames_processed == 0 {
                            self.playhead_frame =
                                self.playhead_frame.saturating_add(block_size as u64);
                        }
                    } else {
                        self.playhead_frame = self.playhead_frame.saturating_add(block_size as u64);
                    }

                    // Process meter ballistics and monitoring
                    let mut master_l = [0.0f32; 256];
                    let mut master_r = [0.0f32; 256];
                    self.audio_scheduler.process_block(
                        &self.arrangement.timeline.tracks,
                        &self.piano_roll_notes,
                        &mut self.mixer,
                        self.playhead_frame,
                        block_size,
                        &mut master_l,
                        &mut master_r,
                    );

                    // Mathematically project exact hardware sample frames into timeline tick position
                    self.playhead_tick = self.audio_scheduler.frame_to_tick(self.playhead_frame);

                    // Seamless loop cycle boundary wrapping
                    if self.arrangement.timeline.loop_region.enabled {
                        let loop_end = self.arrangement.timeline.loop_region.end_tick;
                        let loop_start = self.arrangement.timeline.loop_region.start_tick;
                        if self.playhead_tick >= loop_end && loop_end > loop_start {
                            self.playhead_tick = loop_start;
                            self.playhead_frame = self.audio_scheduler.tick_to_frame(loop_start);
                            if let Some(ref mut worker) = self.audio_worker {
                                worker.seek(self.playhead_frame);
                            }
                        }
                    }

                    self.arrangement.update_playhead_view(self.playhead_tick);
                }
            }

            UiMessage::NewProject => {
                self.playhead_tick = 0;
                self.playhead_frame = 0;
                self.is_playing = false;
                self.dirty = false;
            }
            UiMessage::SaveProject => {
                self.dirty = false;
            }
            UiMessage::MenuAction(cmd) => match cmd {
                crate::ui::message::MenuCommand::NewProject => {
                    self.playhead_tick = 0;
                    self.playhead_frame = 0;
                    self.is_playing = false;
                    self.dirty = false;
                    self.arrangement.timeline.tracks.clear();
                    self.mixer.channels.clear();
                    let _ = self.mixer.add_channel("Master");
                }
                crate::ui::message::MenuCommand::OpenSession => {
                    if let Ok(json) = std::fs::read_to_string("session_backup.cwave.json")
                        && let Ok(project) = crate::format::ProjectLoader::from_json_str(&json)
                    {
                        self.load_project_session(project);
                    }
                    self.dirty = false;
                }
                crate::ui::message::MenuCommand::SaveSession
                | crate::ui::message::MenuCommand::SaveSessionAs => {
                    let cycle_range = if self.arrangement.timeline.loop_region.enabled {
                        Some((
                            self.arrangement.timeline.loop_region.start_tick,
                            self.arrangement.timeline.loop_region.end_tick,
                        ))
                    } else {
                        None
                    };
                    let project = crate::format::project_format::CrystalWaveProject::from_session(
                        "Crystal Wave Session",
                        self.mixer.sample_rate,
                        self.workspace.lcd_diagnostic.project_tempo_bpm,
                        self.workspace.lcd_diagnostic.time_signature,
                        &self.arrangement.timeline.tracks,
                        &self.mixer.channels,
                        cycle_range,
                    );
                    if let Ok(json) = crate::format::ProjectSerializer::to_json_string(&project) {
                        let _ = std::fs::write("session_backup.cwave.json", json);
                    }
                    self.dirty = false;
                }
                crate::ui::message::MenuCommand::BounceWav => {
                    let export_config = crate::bouncing::export_format::ExportFormatConfig {
                        sample_rate: self.mixer.sample_rate,
                        ..Default::default()
                    };

                    let exporter = crate::bouncing::Exporter::new(
                        export_config,
                        crate::bouncing::bit_depth_converter::BitDepth::Bit24,
                        crate::bouncing::dithering::DitherType::None,
                    );

                    // 1. Determine timeline extent from regions
                    let max_end_tick = self
                        .arrangement
                        .timeline
                        .tracks
                        .iter()
                        .flat_map(|t| t.regions.iter())
                        .map(|r| r.start_tick + r.length_ticks)
                        .max()
                        .unwrap_or(3840 * 4);

                    let (start_tick, end_tick) = if self.arrangement.timeline.loop_region.enabled {
                        (
                            self.arrangement.timeline.loop_region.start_tick,
                            self.arrangement
                                .timeline
                                .loop_region
                                .end_tick
                                .max(self.arrangement.timeline.loop_region.start_tick + 960),
                        )
                    } else {
                        (0, max_end_tick.max(3840))
                    };

                    let start_frame = self.audio_scheduler.tick_to_frame(start_tick);
                    let end_frame = self.audio_scheduler.tick_to_frame(end_tick);
                    let total_frames = (end_frame.saturating_sub(start_frame) as usize).max(512);

                    let mut bounce_l = vec![0.0f32; total_frames];
                    let mut bounce_r = vec![0.0f32; total_frames];

                    // 2. High-speed offline rendering loop in blocks of 512 frames
                    let block_size = 512;
                    let mut cur_offset = 0;
                    while cur_offset < total_frames {
                        let chunk = block_size.min(total_frames - cur_offset);
                        let cur_frame = start_frame + cur_offset as u64;

                        self.audio_scheduler.process_block(
                            &self.arrangement.timeline.tracks,
                            &self.piano_roll_notes,
                            &mut self.mixer,
                            cur_frame,
                            chunk,
                            &mut bounce_l[cur_offset..cur_offset + chunk],
                            &mut bounce_r[cur_offset..cur_offset + chunk],
                        );

                        cur_offset += chunk;
                    }

                    // 3. Export to stereo 24-bit WAV container
                    if let Ok(wav_bytes) =
                        exporter.export_stereo_wav(&bounce_l, &bounce_r, self.mixer.sample_rate)
                    {
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
                crate::ui::message::MenuCommand::DuplicateTrack => {
                    let sel_id = self.workspace.selected_track_id;
                    if let Some(track) = self.arrangement.timeline.tracks.get(sel_id).cloned() {
                        let new_id = self.arrangement.timeline.tracks.len();
                        let mut dup_track = track;
                        dup_track.id = new_id;
                        dup_track.name = format!("{} (Copy)", dup_track.name);
                        for r in &mut dup_track.regions {
                            r.id += 1000;
                            r.track_id = new_id;
                        }
                        let _ = self.mixer.add_channel(dup_track.name.clone());
                        let _ = self.arrangement.timeline.add_track(dup_track);
                    }
                    self.dirty = true;
                }
                crate::ui::message::MenuCommand::FreezeTrack => {
                    let sel_id = self.workspace.selected_track_id;
                    if let Some(track) = self.arrangement.timeline.tracks.get_mut(sel_id) {
                        track.is_frozen = !track.is_frozen;
                    }
                    self.dirty = true;
                }
                crate::ui::message::MenuCommand::DeleteTrack => {
                    let sel_id = self.workspace.selected_track_id;
                    if self.arrangement.timeline.tracks.len() > 1
                        && sel_id < self.arrangement.timeline.tracks.len()
                    {
                        self.arrangement.timeline.tracks.remove(sel_id);
                        if sel_id < self.mixer.channels.len() {
                            self.mixer.channels.remove(sel_id);
                        }
                        // Cascade re-indexing across surviving tracks and their regions
                        for (new_idx, track) in
                            self.arrangement.timeline.tracks.iter_mut().enumerate()
                        {
                            track.id = new_idx;
                            for reg in &mut track.regions {
                                reg.track_id = new_idx;
                            }
                        }
                        self.workspace.selected_track_id = self
                            .workspace
                            .selected_track_id
                            .min(self.arrangement.timeline.tracks.len().saturating_sub(1));
                        self.inspector.selected_track_id = self.workspace.selected_track_id;
                        self.arrangement.selected_track_index =
                            Some(self.workspace.selected_track_id);
                    }
                    self.dirty = true;
                }
                crate::ui::message::MenuCommand::Quantize => {
                    for note in &mut self.piano_roll_notes {
                        note.start_tick = ((note.start_tick + 120) / 240) * 240;
                        note.length_ticks = ((note.length_ticks.max(240) + 120) / 240) * 240;
                    }
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
                if self.fl_windows.channel_rack_visible {
                    self.layout
                        .set_drawer_tab(crate::ui::layout::DrawerTab::StepSequencer);
                }
            }
            UiMessage::ToggleFlPianoRoll => {
                self.fl_windows.toggle_piano_roll();
                if self.fl_windows.piano_roll_visible {
                    self.layout
                        .set_drawer_tab(crate::ui::layout::DrawerTab::PianoRoll);
                }
            }
            UiMessage::ToggleFlPlaylist => {
                self.fl_windows.toggle_playlist();
            }
            UiMessage::ToggleFlMixer => {
                self.fl_windows.toggle_mixer();
                if self.fl_windows.mixer_visible {
                    self.layout
                        .set_drawer_tab(crate::ui::layout::DrawerTab::Mixer);
                }
            }
            UiMessage::ToggleFlBrowser => {
                self.fl_windows.toggle_browser();
                self.layout.browser.visible = self.fl_windows.browser_visible;
            }
            UiMessage::ToggleFlPatcher => {
                self.fl_windows.toggle_patcher();
            }
            UiMessage::ToggleFlPluginPicker => {
                self.fl_windows.toggle_plugin_picker();
            }
            UiMessage::BrowserSetSearchQuery(q) => {
                self.browser.search_query = q;
            }
            UiMessage::BrowserSelectTab(tab) => {
                self.browser.active_tab = tab;
            }
            UiMessage::BrowserAuditionSample(id) => {
                self.browser.start_preview(id);
            }
            UiMessage::BrowserStopAudition => {
                self.browser.stop_preview();
            }
            UiMessage::BrowserDropSampleToPlaylist {
                sample_id,
                track_id,
                start_tick,
            } => {
                let _ = self.browser.drop_to_playlist(
                    &mut self.arrangement.timeline.tracks,
                    sample_id,
                    track_id,
                    start_tick,
                    1920,
                );
                self.dirty = true;
            }
            UiMessage::PluginPickerSetSearch(q) => {
                self.plugin_picker_search = q;
            }
            UiMessage::PluginPickerSelectCategory(cat) => {
                self.plugin_picker_category = cat;
            }
            UiMessage::OpenPluginPickerForSlot { track_id, slot_id } => {
                self.plugin_picker_target_slot = Some((track_id, slot_id));
                self.fl_windows.plugin_picker_visible = true;
            }
            UiMessage::PluginPickerLoadPlugin {
                plugin_id,
                is_generator,
            } => {
                if let Some(item) = self.browser.plugins.iter().find(|p| p.id == plugin_id) {
                    if is_generator {
                        let _ = self.channel_rack.add_channel(item.name.clone());
                    } else if let Some((trk_id, slot_id)) = self.plugin_picker_target_slot {
                        let _ = self.mixer.ensure_channel_exists(trk_id);
                        if let Some(strip) = self.mixer.channels.get_mut(trk_id) {
                            strip.assign_plugin_to_slot(slot_id, plugin_id, item.name.clone());
                        }
                    }
                }
                self.fl_windows.plugin_picker_visible = false;
                self.dirty = true;
            }

            UiMessage::ChannelRackToggleStep {
                channel_id,
                step_index,
            } => {
                self.channel_rack.toggle_step(channel_id, step_index);
                self.dirty = true;
            }
            UiMessage::ChannelRackSetVolume { channel_id, volume } => {
                self.channel_rack.set_channel_volume(channel_id, volume);
                self.dirty = true;
            }
            UiMessage::ChannelRackSetPan { channel_id, pan } => {
                self.channel_rack.set_channel_pan(channel_id, pan);
                self.dirty = true;
            }
            UiMessage::ChannelRackSetTargetMixerTrack { channel_id, track } => {
                if track > 0 {
                    let _ = self.mixer.ensure_channel_exists(track - 1);
                }
                self.channel_rack
                    .set_channel_target_mixer_track(channel_id, track);
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
                let mut source_track_idx = 0;
                for (trk_idx, track) in self.arrangement.timeline.tracks.iter_mut().enumerate() {
                    if let Some(pos) = track.regions.iter().position(|r| r.id == clip_id) {
                        let reg = track.regions.remove(pos);
                        source_track_idx = trk_idx;
                        found_region = Some(reg);
                        break;
                    }
                }

                if let Some(mut reg) = found_region {
                    let dest_type = self
                        .arrangement
                        .timeline
                        .tracks
                        .get(new_track_id)
                        .map(|t| t.track_type);

                    let is_fl = matches!(
                        self.theme.preset,
                        crate::ui::theme::ThemePreset::FlDarkOrange
                            | crate::ui::theme::ThemePreset::FlStudioModern
                    );

                    // Track Type Compatibility Validation:
                    // In FL Studio mode or for Automation clips, free-form coexistence allows any clip on any lane!
                    // In Logic Pro mode, strict track typing is enforced.
                    let is_compatible = is_fl
                        || matches!(
                            reg.kind,
                            crate::timeline::region::RegionKind::Automation { .. }
                        )
                        || matches!(
                            (&reg.kind, dest_type),
                            (
                                crate::timeline::region::RegionKind::Audio { .. },
                                Some(
                                    crate::timeline::track::TrackType::Audio
                                        | crate::timeline::track::TrackType::Aux,
                                ),
                            ) | (
                                crate::timeline::region::RegionKind::Midi { .. }
                                    | crate::timeline::region::RegionKind::Pattern { .. },
                                Some(
                                    crate::timeline::track::TrackType::Software
                                        | crate::timeline::track::TrackType::Midi
                                        | crate::timeline::track::TrackType::Drum,
                                ),
                            )
                        );

                    let final_track_id = if is_compatible {
                        new_track_id
                    } else {
                        source_track_idx
                    };
                    reg.start_tick = new_start_tick;
                    reg.track_id = final_track_id;

                    if let Some(dest_track) =
                        self.arrangement.timeline.tracks.get_mut(final_track_id)
                    {
                        dest_track.regions.push(reg);
                    }
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

            UiMessage::SelectRegion(region_id) => {
                for (trk_idx, track) in self.arrangement.timeline.tracks.iter().enumerate() {
                    if track.regions.iter().any(|r| r.id == region_id) {
                        self.workspace.selected_track_id = trk_idx;
                        break;
                    }
                }
            }

            UiMessage::SetCycleRange {
                start_tick,
                end_tick,
            } => {
                self.arrangement.timeline.loop_region.enabled = true;
                self.arrangement.timeline.loop_region.start_tick = start_tick;
                self.arrangement.timeline.loop_region.end_tick = end_tick.max(start_tick + 240);
                self.dirty = true;
            }

            UiMessage::ToggleTrackRecordArm(track_id) => {
                if let Some(track) = self.arrangement.timeline.tracks.get_mut(track_id) {
                    track.record_armed = !track.record_armed;
                }
            }

            UiMessage::ToggleTrackFreeze(track_id) => {
                if let Some(track) = self.arrangement.timeline.tracks.get_mut(track_id) {
                    track.is_frozen = !track.is_frozen;
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
            UiMessage::PianoRollArpeggiate(direction) => {
                if !self.piano_roll_notes.is_empty() {
                    let total_ticks = self
                        .piano_roll_notes
                        .iter()
                        .map(|n| n.start_tick + n.length_ticks)
                        .max()
                        .unwrap_or(960);
                    let arped = crate::fl_studio::FlPianoRollTools::arpeggiate(
                        &self.piano_roll_notes,
                        direction,
                        240, // 16th note step
                        total_ticks,
                    );
                    if !arped.is_empty() {
                        self.piano_roll_notes = arped;
                    }
                }
                self.dirty = true;
            }
            UiMessage::PianoRollSetScale(scale) => {
                self.piano_roll_scale = scale;
                self.dirty = true;
            }
            UiMessage::PianoRollSetScaleRoot(root) => {
                self.piano_roll_scale_root = root % 12;
                self.dirty = true;
            }
            UiMessage::PianoRollSetEventTarget(target) => {
                self.piano_roll_event_target = target;
                self.dirty = true;
            }
            UiMessage::PianoRollSetNoteProperty {
                note_id,
                property,
                value,
            } => {
                if let Some(note) = self.piano_roll_notes.iter_mut().find(|n| n.id == note_id) {
                    match property {
                        crate::fl_studio::piano_roll::EventLaneTarget::Velocity => {
                            note.velocity = value.clamp(0.0, 1.0);
                        }
                        crate::fl_studio::piano_roll::EventLaneTarget::Pan => {
                            note.pan = value.clamp(-1.0, 1.0);
                        }
                        crate::fl_studio::piano_roll::EventLaneTarget::Pitch => {
                            note.fine_pitch_cents = value.clamp(-100.0, 100.0);
                        }
                    }
                    self.dirty = true;
                }
            }

            UiMessage::SetThemePreset(preset) => {
                self.theme = crate::ui::theme::UiTheme::from_preset(preset);
            }
            UiMessage::SetSnapGrid(snap) => {
                self.snap_grid = snap;
            }
            UiMessage::TimelineMarqueeSelect {
                start_tick,
                end_tick,
                start_track,
                end_track,
            } => {
                self.marquee_selection = Some((start_tick, end_tick, start_track, end_track));
                for trk in &mut self.arrangement.timeline.tracks {
                    if trk.id >= start_track && trk.id <= end_track {
                        for reg in &mut trk.regions {
                            let reg_end = reg.start_tick + reg.length_ticks;
                            if reg.start_tick < end_tick && reg_end > start_tick {
                                self.workspace.selected_region_id = Some(reg.id);
                            }
                        }
                    }
                }
            }
            UiMessage::TimelineDuplicateClip(clip_id) => {
                let max_id = self
                    .arrangement
                    .timeline
                    .tracks
                    .iter()
                    .flat_map(|t| t.regions.iter())
                    .map(|r| r.id)
                    .max()
                    .unwrap_or(0);
                for trk in &mut self.arrangement.timeline.tracks {
                    if let Some(pos) = trk.regions.iter().position(|r| r.id == clip_id) {
                        let reg = &trk.regions[pos];
                        let mut duplicated = reg.clone();
                        duplicated.id = max_id + 1;
                        duplicated.start_tick = reg.start_tick + reg.length_ticks;
                        trk.regions.push(duplicated);
                        self.dirty = true;
                        break;
                    }
                }
            }
            UiMessage::TimelineAddPatternClip {
                track_id,
                start_tick,
                length_ticks,
                name,
            } => {
                let max_id = self
                    .arrangement
                    .timeline
                    .tracks
                    .iter()
                    .flat_map(|t| t.regions.iter())
                    .map(|r| r.id)
                    .max()
                    .unwrap_or(0);
                let new_id = max_id + 1;
                if let Some(trk) = self.arrangement.timeline.tracks.get_mut(track_id) {
                    trk.regions.push(Region {
                        id: new_id,
                        track_id,
                        name,
                        start_tick,
                        length_ticks,
                        anchor_offset_ticks: 0,
                        muted: false,
                        looped: false,
                        loop_length_ticks: None,
                        gain_db: 0.0,
                        transpose_semitones: 0.0,
                        reversed: false,
                        fade: Default::default(),
                        kind: RegionKind::Pattern { pattern_id: new_id },
                    });
                    self.dirty = true;
                }
            }
            UiMessage::TimelineAddAutomationClip {
                track_id,
                start_tick,
                length_ticks,
                parameter_target,
            } => {
                let max_id = self
                    .arrangement
                    .timeline
                    .tracks
                    .iter()
                    .flat_map(|t| t.regions.iter())
                    .map(|r| r.id)
                    .max()
                    .unwrap_or(0);
                let new_id = max_id + 1;
                if let Some(trk) = self.arrangement.timeline.tracks.get_mut(track_id) {
                    let initial_points = vec![
                        crate::timeline::region::AutomationPoint::new(0, 0.2, 0.0),
                        crate::timeline::region::AutomationPoint::new(length_ticks / 2, 0.8, 0.5),
                        crate::timeline::region::AutomationPoint::new(length_ticks, 0.5, -0.5),
                    ];
                    trk.regions.push(Region {
                        id: new_id,
                        track_id,
                        name: format!("Auto: {}", parameter_target),
                        start_tick,
                        length_ticks,
                        anchor_offset_ticks: 0,
                        muted: false,
                        looped: false,
                        loop_length_ticks: None,
                        gain_db: 0.0,
                        transpose_semitones: 0.0,
                        reversed: false,
                        fade: Default::default(),
                        kind: RegionKind::Automation {
                            parameter_target,
                            min_value: 0.0,
                            max_value: 1.0,
                            points: initial_points,
                        },
                    });
                    self.dirty = true;
                }
            }
            UiMessage::TimelineSetAutomationTension {
                clip_id,
                segment_index,
                tension,
            } => {
                for trk in &mut self.arrangement.timeline.tracks {
                    if let Some(reg) = trk.regions.iter_mut().find(|r| r.id == clip_id) {
                        if let RegionKind::Automation { points, .. } = &mut reg.kind
                            && let Some(pt) = points.get_mut(segment_index)
                        {
                            pt.tension = tension.clamp(-1.0, 1.0);
                            self.dirty = true;
                        }
                        break;
                    }
                }
            }
            UiMessage::TimelineSetAutomationNode {
                clip_id,
                node_index,
                value,
            } => {
                for trk in &mut self.arrangement.timeline.tracks {
                    if let Some(reg) = trk.regions.iter_mut().find(|r| r.id == clip_id) {
                        if let RegionKind::Automation { points, .. } = &mut reg.kind
                            && let Some(pt) = points.get_mut(node_index)
                        {
                            pt.value = value.clamp(0.0, 1.0);
                            self.dirty = true;
                        }
                        break;
                    }
                }
            }
            UiMessage::SelectDrawerTab(tab) => {
                self.layout.set_drawer_tab(tab);
                self.workspace.editor_drawer_open = true;
            }
            UiMessage::TogglePanelDock(panel_id) => {
                self.layout.toggle_panel(panel_id);
            }
            UiMessage::DetachPanelToWindow(panel_id) => {
                self.layout.detach_to_window(panel_id);
            }
            UiMessage::RedockPanel(panel_id, dock) => {
                self.layout.redock_to_main(panel_id, dock);
            }
            UiMessage::ToggleCommandPalette => {
                self.command_palette_open = !self.command_palette_open;
                self.command_query.clear();
            }
            UiMessage::CommandPaletteInput(query) => {
                self.command_query = query;
            }
            UiMessage::ExecuteCommand(cmd) => {
                let lower = cmd.to_lowercase();
                if lower.contains("play") {
                    self.is_playing = true;
                } else if lower.contains("stop") {
                    self.is_playing = false;
                    self.playhead_tick = 0;
                } else if lower.contains("record") {
                    self.is_recording = !self.is_recording;
                } else if lower.contains("save") {
                    self.dirty = false;
                } else if lower.contains("modern") {
                    self.theme = crate::ui::theme::UiTheme::fl_studio_modern();
                } else if lower.contains("orange") || lower.contains("fl") {
                    self.theme = crate::ui::theme::UiTheme::fl_dark_orange();
                } else if lower.contains("graphite") || lower.contains("logic") {
                    self.theme = crate::ui::theme::UiTheme::graphite();
                } else if lower.contains("snap") {
                    self.snap_grid = crate::ui::canvas_timeline::SnapGrid::Sixteenth;
                }
                self.command_palette_open = false;
            }
            UiMessage::SetTimelineScrollY(y) => {
                self.timeline_scroll_y = y.max(0.0);
            }
            UiMessage::PianoRollAddNote {
                channel_id,
                key,
                start_tick,
                length_ticks,
            } => {
                let id = self.piano_roll_notes.len() + 1;
                self.piano_roll_notes
                    .push(crate::fl_studio::FlPianoRollNote::new(
                        id,
                        channel_id,
                        key,
                        start_tick,
                        length_ticks,
                    ));
                self.dirty = true;
            }
            UiMessage::PianoRollDeleteNote(id) => {
                self.piano_roll_notes.retain(|n| n.id != id);
                self.dirty = true;
            }
            UiMessage::PianoRollMoveNote {
                id,
                delta_ticks,
                delta_key,
            } => {
                if let Some(note) = self.piano_roll_notes.iter_mut().find(|n| n.id == id) {
                    let new_key = ((note.key as i16) + (delta_key as i16)).clamp(0, 127) as u8;
                    let new_start = ((note.start_tick as i64) + delta_ticks).max(0) as u64;
                    note.key = new_key;
                    note.start_tick = new_start;
                    self.dirty = true;
                }
            }
            UiMessage::PianoRollResizeNote { id, delta_ticks } => {
                if let Some(note) = self.piano_roll_notes.iter_mut().find(|n| n.id == id) {
                    let new_len = ((note.length_ticks as i64) + delta_ticks).max(60) as u64;
                    note.length_ticks = new_len;
                    self.dirty = true;
                }
            }
            UiMessage::PianoRollToggleSlide(id) => {
                if let Some(note) = self.piano_roll_notes.iter_mut().find(|n| n.id == id) {
                    note.is_slide = !note.is_slide;
                    self.dirty = true;
                }
            }
            UiMessage::PianoRollSelectKey(key) => {
                use crate::instruments::VirtualInstrument;
                self.audio_scheduler
                    .synth
                    .note_on(key, (0.8 * 65535.0) as u16);
            }
            UiMessage::PianoRollSetScrollY(y) => {
                self.piano_roll_scroll_y = y.max(0.0);
            }
            UiMessage::StepSequencerToggleStep {
                channel_id,
                step_index,
            } => {
                self.channel_rack.toggle_step(channel_id, step_index);
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

    /// Load and restore an entire project session into active timeline tracks and mixer
    pub fn load_project_session(
        &mut self,
        project: crate::format::project_format::CrystalWaveProject,
    ) {
        self.workspace.lcd_diagnostic.project_tempo_bpm = project.metadata.bpm;
        self.workspace.lcd_diagnostic.time_signature = project.metadata.time_signature;
        self.arrangement.timeline.loop_region.enabled = project.cycle_loop_enabled;
        self.arrangement.timeline.loop_region.start_tick = project.cycle_loop_start_tick;
        self.arrangement.timeline.loop_region.end_tick = project.cycle_loop_end_tick;

        self.arrangement.timeline.tracks.clear();
        self.mixer.channels.clear();

        for (idx, strk) in project.tracks.into_iter().enumerate() {
            let trk_type = match strk.track_type.as_str() {
                "Software" => TrackType::Software,
                "Drum" => TrackType::Drum,
                "Midi" => TrackType::Midi,
                _ => TrackType::Audio,
            };

            let mut track = Track::new(idx, strk.name.clone(), trk_type);
            track.gain_db = strk.volume_fader_db;
            track.pan = strk.pan;
            track.muted = strk.muted;
            track.soloed = strk.solo;

            for r in strk.regions {
                let mut reg = Region::new(
                    r.id,
                    idx,
                    r.name,
                    r.start_tick,
                    r.length_ticks,
                    RegionKind::Audio {
                        file_path: None,
                        sample_offset: 0,
                        sample_length: (r.length_ticks as usize) * 441,
                    },
                );
                reg.muted = r.muted;
                reg.gain_db = r.gain_db;
                track.regions.push(reg);
            }

            let _ = self.mixer.add_channel(strk.name);
            if let Some(ch) = self.mixer.channels.get_mut(idx) {
                ch.volume_fader_db = strk.volume_fader_db;
                ch.pan.balance = strk.pan;
                ch.muted = strk.muted;
                ch.solo = strk.solo;
            }

            let _ = self.arrangement.timeline.add_track(track);
        }

        if self.arrangement.timeline.tracks.is_empty() {
            let trk = Track::new(0, "Audio 1", TrackType::Audio);
            let _ = self.mixer.add_channel("Audio 1");
            let _ = self.arrangement.timeline.add_track(trk);
        }
        self.workspace.selected_track_id = 0;
        self.dirty = false;
    }

    /// Constructs the 2D Piano Roll Canvas Program with active notes and polyphonic ghost notes
    pub fn piano_roll_canvas_program(
        &self,
    ) -> crate::ui::canvas_piano_roll::PianoRollCanvasProgram {
        let active_notes: Vec<crate::ui::canvas_piano_roll::PianoRollNoteDisplay> = self
            .piano_roll_notes
            .iter()
            .filter(|n| n.channel_id == self.piano_roll_selected_channel)
            .map(|n| crate::ui::canvas_piano_roll::PianoRollNoteDisplay {
                id: n.id,
                channel_id: n.channel_id,
                key: n.key,
                start_tick: n.start_tick,
                length_ticks: n.length_ticks,
                velocity: n.velocity,
                pan: n.pan,
                fine_pitch_cents: n.fine_pitch_cents,
                is_slide: n.is_slide,
            })
            .collect();

        let mut ghost_notes = Vec::new();

        // 1. Ghost notes from other piano roll channels
        for n in self
            .piano_roll_notes
            .iter()
            .filter(|n| n.channel_id != self.piano_roll_selected_channel)
        {
            let ch_name = self
                .channel_rack
                .channels
                .get(n.channel_id)
                .map_or_else(|| format!("Ch {}", n.channel_id), |c| c.name.clone());
            ghost_notes.push(crate::ui::canvas_piano_roll::GhostNoteDisplay {
                key: n.key,
                start_tick: n.start_tick,
                length_ticks: n.length_ticks,
                channel_name: ch_name,
            });
        }

        // 2. Ghost notes from background channel rack step patterns
        for (ch_idx, ch) in self.channel_rack.channels.iter().enumerate() {
            if ch_idx != self.piano_roll_selected_channel {
                for (step_idx, step) in ch.steps.iter().enumerate() {
                    if step.active {
                        let step_ticks = 240u64; // 16th note subdivision
                        let pitch = (60i16 + step.pitch_semitones as i16).clamp(0, 127) as u8;
                        ghost_notes.push(crate::ui::canvas_piano_roll::GhostNoteDisplay {
                            key: pitch,
                            start_tick: (step_idx as u64) * step_ticks,
                            length_ticks: step_ticks,
                            channel_name: ch.name.clone(),
                        });
                    }
                }
            }
        }

        crate::ui::canvas_piano_roll::PianoRollCanvasProgram::new(
            self.playhead_tick,
            self.arrangement.horizontal_zoom,
            self.piano_roll_scroll_y,
        )
        .with_notes(active_notes)
        .with_ghost_notes(ghost_notes)
        .with_snap(self.snap_grid)
        .with_theme(matches!(
            self.theme.preset,
            crate::ui::theme::ThemePreset::FlDarkOrange
                | crate::ui::theme::ThemePreset::FlStudioModern
        ))
        .with_channel_id(self.piano_roll_selected_channel)
        .with_scale(self.piano_roll_scale, self.piano_roll_scale_root)
        .with_event_target(self.piano_roll_event_target)
    }
}
