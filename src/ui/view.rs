use crate::ui::app::CrystalWaveApp;
use crate::ui::message::{MenuCommand, UiMessage};
use iced::widget::{button, column, container, row, scrollable, slider, text};
use iced::{Element, Length};

impl CrystalWaveApp {
    pub fn view(&self) -> Element<'_, UiMessage> {
        let menu_bar = self.view_menu_bar();
        let control_bar = self.view_control_bar();
        let main_workspace = self.view_main_workspace();
        let bottom_drawer = self.view_bottom_drawer();

        column![menu_bar, control_bar, main_workspace, bottom_drawer]
            .width(Length::Fill)
            .height(Length::Fill)
            .spacing(2)
            .into()
    }

    fn view_menu_bar(&self) -> Element<'_, UiMessage> {
        let file_new =
            button(text("New").size(11)).on_press(UiMessage::MenuAction(MenuCommand::NewProject));
        let file_save =
            button(text("Save").size(11)).on_press(UiMessage::MenuAction(MenuCommand::SaveSession));
        let file_bounce = button(text("Export WAV").size(11))
            .on_press(UiMessage::MenuAction(MenuCommand::BounceWav));
        let file_aaf =
            button(text("AAF").size(11)).on_press(UiMessage::MenuAction(MenuCommand::ExportAaf));

        let edit_split = button(text("Cut (B)").size(11))
            .on_press(UiMessage::MenuAction(MenuCommand::SplitAtCursor));
        let track_add_audio = button(text("+ Audio").size(11))
            .on_press(UiMessage::MenuAction(MenuCommand::AddAudioTrack));
        let track_add_midi = button(text("+ MIDI").size(11))
            .on_press(UiMessage::MenuAction(MenuCommand::AddMidiTrack));

        // FL Studio Multi-Window Toolbars (F5, F6, F7, F8, F9)
        let win_playlist = button(
            text(if self.fl_windows.playlist_visible {
                "Playlist [F5]*"
            } else {
                "Playlist [F5]"
            })
            .size(11),
        )
        .on_press(UiMessage::ToggleFlPlaylist);
        let win_channel_rack = button(
            text(if self.fl_windows.channel_rack_visible {
                "Rack [F6]*"
            } else {
                "Rack [F6]"
            })
            .size(11),
        )
        .on_press(UiMessage::ToggleFlChannelRack);
        let win_piano_roll = button(
            text(if self.fl_windows.piano_roll_visible {
                "Piano Roll [F7]*"
            } else {
                "Piano Roll [F7]"
            })
            .size(11),
        )
        .on_press(UiMessage::ToggleFlPianoRoll);
        let win_browser = button(
            text(if self.fl_windows.browser_visible {
                "Browser [F8]*"
            } else {
                "Browser [F8]"
            })
            .size(11),
        )
        .on_press(UiMessage::ToggleFlBrowser);
        let win_mixer = button(
            text(if self.fl_windows.mixer_visible {
                "Mixer [F9]*"
            } else {
                "Mixer [F9]"
            })
            .size(11),
        )
        .on_press(UiMessage::ToggleFlMixer);
        let win_patcher = button(
            text(if self.fl_windows.patcher_visible {
                "Patcher*"
            } else {
                "Patcher"
            })
            .size(11),
        )
        .on_press(UiMessage::ToggleFlPatcher);
        let win_inspector = button(
            text(if self.workspace.inspector_open {
                "Inspect [I]*"
            } else {
                "Inspect [I]"
            })
            .size(11),
        )
        .on_press(UiMessage::ToggleInspector);
        let win_plugins =
            button(text("Plugins [L]").size(11)).on_press(UiMessage::ToggleLibraryBrowser);

        row![
            container(text("CRYSTAL WAVE").size(12)).padding(4),
            file_new,
            file_save,
            file_bounce,
            file_aaf,
            edit_split,
            track_add_audio,
            track_add_midi,
            win_playlist,
            win_channel_rack,
            win_piano_roll,
            win_browser,
            win_mixer,
            win_patcher,
            win_inspector,
            win_plugins
        ]
        .spacing(4)
        .padding(4)
        .into()
    }

    fn view_control_bar(&self) -> Element<'_, UiMessage> {
        let play_btn = button(text(if self.is_playing { "Pause" } else { "Play" }).size(12))
            .on_press(if self.is_playing {
                UiMessage::TransportPause
            } else {
                UiMessage::TransportPlay
            });
        let stop_btn = button(text("Stop").size(12)).on_press(UiMessage::TransportStop);
        let record_btn = button(text("Record").size(12)).on_press(UiMessage::TransportRecord);

        let lcd = container(
            text(format!(
                "BPM: {:.1} | Sig: {}/{} | Pos: {} | A4: {:.1}Hz | CPU: {:.1}%",
                self.workspace.lcd_diagnostic.project_tempo_bpm,
                self.workspace.lcd_diagnostic.time_signature.0,
                self.workspace.lcd_diagnostic.time_signature.1,
                self.playhead_tick,
                self.workspace.lcd_diagnostic.master_tuning_hz,
                self.workspace.lcd_diagnostic.cpu_load_percent,
            ))
            .size(11),
        )
        .padding(5);

        let tune_440_btn =
            button(text("440 Hz").size(10)).on_press(UiMessage::SetMasterTuning(440.0));
        let tune_432_btn =
            button(text("432 Hz").size(10)).on_press(UiMessage::SetMasterTuning(432.0));
        let tune_434_btn =
            button(text("434.5 Hz").size(10)).on_press(UiMessage::SetMasterTuning(434.5));

        row![
            play_btn,
            stop_btn,
            record_btn,
            lcd,
            tune_440_btn,
            tune_432_btn,
            tune_434_btn
        ]
        .spacing(6)
        .padding(4)
        .into()
    }

    fn view_main_workspace(&self) -> Element<'_, UiMessage> {
        let mut content = row![].spacing(4).width(Length::Fill).height(Length::Fill);

        // 1. FL Studio Browser (Alt+F8 / F8) - Drag and Drop sample library
        if self.fl_windows.browser_visible {
            let mut sample_list = column![
                row![
                    text("Browser / Samples").size(12),
                    button(text("x").size(10)).on_press(UiMessage::ToggleFlBrowser),
                ]
                .spacing(8),
                text("Click sample to load into Rack").size(10),
            ]
            .spacing(4);

            for sample in &self.browser.samples {
                let sample_id = sample.id;
                let sample_card = button(
                    column![
                        text(&sample.name).size(11),
                        text(format!(
                            "{} | {:.1}s",
                            sample.category, sample.duration_seconds
                        ))
                        .size(9),
                    ]
                    .spacing(2),
                )
                .on_press(UiMessage::ChannelRackDropSample { sample_id })
                .width(Length::Fill);

                sample_list = sample_list.push(sample_card);
            }

            let browser_panel = container(scrollable(sample_list))
                .width(Length::Fixed(190.0))
                .padding(6);

            content = content.push(browser_panel);
        }

        // 2. Logic/Pro Tools Inspector (I)
        if self.workspace.inspector_open {
            let trk_id = self.workspace.selected_track_id;
            let channel_name = self
                .mixer
                .channels
                .get(trk_id)
                .map(|c| c.name.as_str())
                .unwrap_or("No Track");

            let fader_val = self
                .mixer
                .channels
                .get(trk_id)
                .map(|c| c.volume_fader_db)
                .unwrap_or(0.0);

            let pan_val = self
                .mixer
                .channels
                .get(trk_id)
                .map(|c| c.pan.balance)
                .unwrap_or(0.0);

            let is_muted = self
                .mixer
                .channels
                .get(trk_id)
                .map(|c| c.muted)
                .unwrap_or(false);

            let is_solo = self
                .mixer
                .channels
                .get(trk_id)
                .map(|c| c.solo)
                .unwrap_or(false);

            let mute_btn = button(text(if is_muted { "Muted" } else { "Mute" }).size(10))
                .on_press(UiMessage::ToggleTrackMute(trk_id));

            let solo_btn = button(text(if is_solo { "Soloed" } else { "Solo" }).size(10))
                .on_press(UiMessage::ToggleTrackSolo(trk_id));

            let inspector_box = container(
                column![
                    row![
                        text(format!("Track {}", trk_id)).size(12),
                        button(text("x").size(10)).on_press(UiMessage::ToggleInspector),
                    ]
                    .spacing(8),
                    text(format!("Name: {}", channel_name)).size(11),
                    text(format!("Level: {:.1} dB", fader_val)).size(11),
                    text(format!("Pan: {:.2}", pan_val)).size(11),
                    row![mute_btn, solo_btn].spacing(4),
                ]
                .spacing(4),
            )
            .width(Length::Fixed(160.0))
            .padding(6);

            content = content.push(inspector_box);
        }

        // 3. FL Studio Channel Rack Window (F6) - Step Sequencer Matrix
        if self.fl_windows.channel_rack_visible {
            let mut rack_col = column![
                row![
                    text("Channel Rack / Step Sequencer [F6]").size(12),
                    button(text("x").size(10)).on_press(UiMessage::ToggleFlChannelRack),
                ]
                .spacing(8),
            ]
            .spacing(4);

            for chan in &self.channel_rack.channels {
                let chan_id = chan.id;
                let mut steps_row =
                    row![container(text(&chan.name).size(11)).width(Length::Fixed(90.0)),]
                        .spacing(2);

                for (s_idx, step) in chan.steps.iter().enumerate() {
                    let step_lbl = if step.active { "■" } else { "·" };
                    let step_btn = button(text(step_lbl).size(11)).padding(2).on_press(
                        UiMessage::ChannelRackToggleStep {
                            channel_id: chan_id,
                            step_index: s_idx,
                        },
                    );
                    steps_row = steps_row.push(step_btn);
                }

                rack_col = rack_col.push(steps_row);
            }

            let rack_panel = container(scrollable(rack_col))
                .width(Length::Fixed(360.0))
                .padding(6);

            content = content.push(rack_panel);
        }

        // 4. Apple Logic Pro Professional Arrangement Timeline & Track Headers
        if self.fl_windows.playlist_visible {
            let mut track_headers = column![
                // Top Ruler spacer bar matching RULER_HEIGHT = 30.0
                container(
                    row![
                        text("TRACKS").size(10),
                        button(text("+ Aud").size(9))
                            .padding(2)
                            .on_press(UiMessage::MenuAction(MenuCommand::AddAudioTrack)),
                        button(text("+ Inst").size(9))
                            .padding(2)
                            .on_press(UiMessage::MenuAction(MenuCommand::AddMidiTrack)),
                    ]
                    .spacing(4)
                )
                .height(Length::Fixed(crate::ui::canvas_timeline::RULER_HEIGHT))
                .width(Length::Fill)
                .padding(4),
            ];

            let mut canvas_program = crate::ui::canvas_timeline::ArrangementCanvasProgram::new(
                self.playhead_tick,
                self.arrangement.horizontal_zoom,
                self.arrangement.timeline.tracks.len().max(1),
            );

            canvas_program.selected_track_id = self.workspace.selected_track_id;
            canvas_program.loop_range = if self.arrangement.timeline.loop_region.enabled {
                Some((
                    self.arrangement.timeline.loop_region.start_tick,
                    self.arrangement.timeline.loop_region.end_tick,
                ))
            } else {
                None
            };

            for (trk_idx, track) in self.arrangement.timeline.tracks.iter().enumerate() {
                let is_selected = trk_idx == self.workspace.selected_track_id;
                let ch_muted = self
                    .mixer
                    .channels
                    .get(trk_idx)
                    .map(|c| c.muted)
                    .unwrap_or(false);
                let ch_solo = self
                    .mixer
                    .channels
                    .get(trk_idx)
                    .map(|c| c.solo)
                    .unwrap_or(false);
                let is_muted = track.muted || ch_muted;
                let is_solo = track.soloed || ch_solo;

                let type_badge = match track.track_type {
                    crate::timeline::track::TrackType::Audio => "AUD",
                    crate::timeline::track::TrackType::Software => "INST",
                    crate::timeline::track::TrackType::Drum => "DRUM",
                    crate::timeline::track::TrackType::Midi => "MIDI",
                    crate::timeline::track::TrackType::Aux => "AUX",
                    crate::timeline::track::TrackType::Master => "MST",
                };

                let mute_btn = button(text(if is_muted { "M" } else { "m" }).size(10))
                    .padding(3)
                    .on_press(UiMessage::ToggleTrackMute(trk_idx));

                let solo_btn = button(text(if is_solo { "S" } else { "s" }).size(10))
                    .padding(3)
                    .on_press(UiMessage::ToggleTrackSolo(trk_idx));

                let rec_btn = button(text(if track.record_armed { "R" } else { "r" }).size(10))
                    .padding(3)
                    .on_press(UiMessage::ToggleTrackRecordArm(trk_idx));

                let freeze_btn = button(text(if track.is_frozen { "❄" } else { "·" }).size(10))
                    .padding(3)
                    .on_press(UiMessage::ToggleTrackFreeze(trk_idx));

                let track_card = container(
                    column![
                        row![
                            text(type_badge).size(9),
                            button(text(&track.name).size(11))
                                .padding(1)
                                .on_press(UiMessage::SelectTrack(trk_idx)),
                        ]
                        .spacing(4),
                        row![mute_btn, solo_btn, rec_btn, freeze_btn].spacing(3),
                        text(format!("{:.1} dB | Pan {:.2}", track.gain_db, track.pan)).size(9),
                    ]
                    .spacing(2),
                )
                .height(Length::Fixed(crate::ui::canvas_timeline::LANE_HEIGHT))
                .width(Length::Fill)
                .padding(4);

                track_headers = track_headers.push(track_card);

                for region in &track.regions {
                    let visual_kind = match &region.kind {
                        crate::timeline::region::RegionKind::Audio { .. } => {
                            crate::ui::canvas_timeline::ClipVisualKind::Audio {
                                waveform_peaks: Vec::new(),
                            }
                        }
                        crate::timeline::region::RegionKind::Midi { .. } => {
                            crate::ui::canvas_timeline::ClipVisualKind::Midi { notes: Vec::new() }
                        }
                        crate::timeline::region::RegionKind::Pattern { .. } => {
                            crate::ui::canvas_timeline::ClipVisualKind::Pattern
                        }
                    };

                    canvas_program
                        .clips
                        .push(crate::ui::canvas_timeline::TimelineClipDisplay {
                            id: region.id,
                            track_index: trk_idx,
                            start_tick: region.start_tick,
                            length_ticks: region.length_ticks,
                            name: region.name.clone(),
                            color_rgb: [track.color.r, track.color.g, track.color.b],
                            muted: region.muted || is_muted,
                            looped: region.looped,
                            fade_in_ticks: region.fade.in_ticks,
                            fade_out_ticks: region.fade.out_ticks,
                            kind: visual_kind,
                            is_selected,
                        });
                }
            }

            let track_header_column = container(track_headers).width(Length::Fixed(200.0));
            let arrangement_canvas =
                crate::ui::canvas_timeline::view_canvas_timeline(canvas_program);

            let logic_timeline = row![track_header_column, arrangement_canvas]
                .width(Length::Fill)
                .height(Length::Fill)
                .spacing(1);

            content = content.push(logic_timeline);
        }

        // 5. Plugin Manager Sidebar
        if self.workspace.library_browser_open {
            let mut plugins_list = column![
                row![
                    text("Plugin Manager").size(12),
                    button(text("x").size(10)).on_press(UiMessage::ToggleLibraryBrowser),
                ]
                .spacing(8),
                text(format!("Installed: {}", self.plugin_registry.len())).size(10),
            ]
            .spacing(4);

            for entry in self.plugin_registry.list_entries() {
                let meta_id = entry.descriptor.unique_id;
                let is_enabled = entry.is_enabled;
                let toggle_btn = if is_enabled {
                    button(text("Disable").size(10)).on_press(UiMessage::DisablePlugin(meta_id))
                } else {
                    button(text("Enable").size(10)).on_press(UiMessage::EnablePlugin(meta_id))
                };
                let remove_btn =
                    button(text("Uninstall").size(10)).on_press(UiMessage::RemovePlugin(meta_id));

                let plugin_card = container(
                    column![
                        text(format!(
                            "{} v{}",
                            entry.descriptor.name, entry.descriptor.version
                        ))
                        .size(11),
                        text(format!(
                            "{:?} | {}",
                            entry.descriptor.category, entry.descriptor.vendor
                        ))
                        .size(9),
                        row![toggle_btn, remove_btn].spacing(4),
                    ]
                    .spacing(2),
                )
                .padding(4);

                plugins_list = plugins_list.push(plugin_card);
            }

            let plugin_panel = container(scrollable(plugins_list))
                .width(Length::Fixed(210.0))
                .padding(6);

            content = content.push(plugin_panel);
        }

        content.into()
    }

    fn view_bottom_drawer(&self) -> Element<'_, UiMessage> {
        // Multi-Window Bottom Area: Either FL Mixer (F9) or Piano Roll (F7)
        if self.fl_windows.mixer_visible || self.workspace.mixer_drawer_open {
            let mut channel_strips = row![].spacing(6).padding(4);

            for (idx, ch) in self.mixer.channels.iter().enumerate() {
                let mute_btn = button(text(if ch.muted { "M" } else { "m" }).size(10))
                    .on_press(UiMessage::ToggleTrackMute(idx));

                let solo_btn = button(text(if ch.solo { "S" } else { "s" }).size(10))
                    .on_press(UiMessage::ToggleTrackSolo(idx));

                let fader_slider = slider(-60.0..=6.0, ch.volume_fader_db, move |vol| {
                    UiMessage::SetTrackFader {
                        track_id: idx,
                        volume_db: vol,
                    }
                })
                .step(0.5_f32)
                .width(Length::Fixed(70.0));

                let pan_slider = slider(-1.0..=1.0, ch.pan.balance, move |p| {
                    UiMessage::SetTrackPan {
                        track_id: idx,
                        pan: p,
                    }
                })
                .step(0.05_f32)
                .width(Length::Fixed(70.0));

                let strip = container(
                    column![
                        text(&ch.name).size(11),
                        text(format!("{:.1} dB", ch.volume_fader_db)).size(10),
                        fader_slider,
                        text(format!("Pan: {:.2}", ch.pan.balance)).size(10),
                        pan_slider,
                        row![mute_btn, solo_btn].spacing(4),
                    ]
                    .spacing(3),
                )
                .padding(4)
                .width(Length::Fixed(90.0));

                channel_strips = channel_strips.push(strip);
            }

            return container(
                column![
                    row![
                        text("FL Studio Mixer Console [F9]").size(12),
                        button(text("x").size(10)).on_press(UiMessage::ToggleFlMixer),
                    ]
                    .spacing(8),
                    scrollable(channel_strips),
                ]
                .spacing(4),
            )
            .height(Length::Fixed(190.0))
            .width(Length::Fill)
            .padding(4)
            .into();
        }

        if self.fl_windows.piano_roll_visible || self.workspace.editor_drawer_open {
            let btn_strum = button(text("Strum").size(10)).on_press(UiMessage::PianoRollQuickStrum);
            let btn_flip_h =
                button(text("Flip H").size(10)).on_press(UiMessage::PianoRollFlipHorizontal);
            let btn_flip_v =
                button(text("Flip V").size(10)).on_press(UiMessage::PianoRollFlipVertical);
            let btn_chop =
                button(text("Chop 1/16").size(10)).on_press(UiMessage::PianoRollQuickChop);

            let roll_header = row![
                text("FL Piano Roll [F7] (Ghost Polyphony: Active)").size(12),
                btn_strum,
                btn_flip_h,
                btn_flip_v,
                btn_chop,
                text("Scale: Chromatic | Grid: 1/16 | Slide Pitch: Active").size(10),
                button(text("x").size(10)).on_press(UiMessage::ToggleFlPianoRoll),
            ]
            .spacing(6);

            let keys_col = column![
                text("C5 - [ Lead ]").size(10),
                text("B4 - [ Ghost ]").size(10),
                text("A4 - [ Bass ]").size(10),
                text("G4 - [ Chord ]").size(10),
                text("F4 - [ Ghost ]").size(10),
                text("E4 - [ Pad ]").size(10),
                text("D4 - [ Note ]").size(10),
                text("C4 - [ Root ]").size(10),
            ]
            .spacing(2)
            .width(Length::Fixed(110.0));

            let mut notes_col = column![].spacing(3);
            for note in &self.piano_roll_notes {
                let slide_tag = if note.is_slide { " [Slide]" } else { "" };
                notes_col = notes_col.push(
                    text(format!(
                        "Note {}: Key {} ({}..{}) Vel: {:.0}%{}",
                        note.id,
                        note.key,
                        note.start_tick,
                        note.start_tick + note.length_ticks,
                        note.velocity * 100.0,
                        slide_tag
                    ))
                    .size(10),
                );
            }
            let grid_content = container(scrollable(notes_col)).padding(4);

            let roll_body = row![keys_col, grid_content].spacing(8);

            return container(column![roll_header, roll_body].spacing(4))
                .height(Length::Fixed(190.0))
                .width(Length::Fill)
                .padding(4)
                .into();
        }

        if self.fl_windows.patcher_visible {
            let mut nodes_row = row![].spacing(12).padding(8);
            for node in &self.patcher.nodes {
                let node_box = container(
                    column![
                        text(&node.name).size(11),
                        text(format!("{:?}", node.kind)).size(9),
                        text(format!("({:.0}, {:.0})", node.canvas_x, node.canvas_y)).size(8),
                    ]
                    .spacing(2),
                )
                .padding(6);
                nodes_row = nodes_row.push(node_box);
            }

            let patcher_header = row![
                text("FL Patcher Modular Routing Graph").size(12),
                text(format!(
                    "Nodes: {} | Cables: {}",
                    self.patcher.nodes.len(),
                    self.patcher.cables.len()
                ))
                .size(10),
                button(text("x").size(10)).on_press(UiMessage::ToggleFlPatcher),
            ]
            .spacing(8);

            return container(column![patcher_header, scrollable(nodes_row)].spacing(4))
                .height(Length::Fixed(190.0))
                .width(Length::Fill)
                .padding(4)
                .into();
        }

        column![].into()
    }
}
