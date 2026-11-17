use crate::ui::app::CrystalWaveApp;
use crate::ui::message::{MenuCommand, UiMessage};
use iced::widget::{button, column, container, row, scrollable, slider, text, text_input};
use iced::{Element, Length};

impl CrystalWaveApp {
    pub fn view(&self) -> Element<'_, UiMessage> {
        let menu_bar = self.view_menu_bar();
        let control_bar = self.view_control_bar();
        let main_workspace = self.view_main_workspace();
        let bottom_drawer = self.view_bottom_drawer();
        let status_bar = self.view_status_bar();

        let mut main_col = column![
            menu_bar,
            control_bar,
            main_workspace,
            bottom_drawer,
            status_bar
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(1);

        if self.command_palette_open {
            let palette = self.view_command_palette();
            main_col = main_col.push(container(palette).padding(8));
        }

        container(main_col)
            .style(|_t| self.theme.header_container_style())
            .width(Length::Fill)
            .height(Length::Fill)
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

        // Theme Switcher & Snap Toggle
        let theme_btn = button(
            text(match self.theme.preset {
                crate::ui::theme::ThemePreset::Graphite => "Theme: Logic Graphite",
                crate::ui::theme::ThemePreset::FlDarkOrange => "Theme: FL Dark Orange",
            })
            .size(11),
        )
        .on_press(UiMessage::SetThemePreset(
            if self.theme.preset == crate::ui::theme::ThemePreset::Graphite {
                crate::ui::theme::ThemePreset::FlDarkOrange
            } else {
                crate::ui::theme::ThemePreset::Graphite
            },
        ));

        let snap_btn = button(text(self.snap_grid.label()).size(11)).on_press(
            UiMessage::SetSnapGrid(match self.snap_grid {
                crate::ui::canvas_timeline::SnapGrid::Sixteenth => {
                    crate::ui::canvas_timeline::SnapGrid::Eighth
                }
                crate::ui::canvas_timeline::SnapGrid::Eighth => {
                    crate::ui::canvas_timeline::SnapGrid::Beat
                }
                crate::ui::canvas_timeline::SnapGrid::Beat => {
                    crate::ui::canvas_timeline::SnapGrid::Bar
                }
                crate::ui::canvas_timeline::SnapGrid::Bar => {
                    crate::ui::canvas_timeline::SnapGrid::Smart
                }
                crate::ui::canvas_timeline::SnapGrid::Smart => {
                    crate::ui::canvas_timeline::SnapGrid::Off
                }
                _ => crate::ui::canvas_timeline::SnapGrid::Sixteenth,
            }),
        );

        let cmd_palette_btn =
            button(text("⌘K / Ctrl+K").size(11)).on_press(UiMessage::ToggleCommandPalette);

        row![
            container(text("CRYSTAL WAVE").size(12)).padding(4),
            file_new,
            file_save,
            file_bounce,
            file_aaf,
            edit_split,
            track_add_audio,
            track_add_midi,
            snap_btn,
            theme_btn,
            cmd_palette_btn,
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
        let play_btn = button(
            text(if self.is_playing {
                "⏸ PAUSE"
            } else {
                "▶ PLAY"
            })
            .size(11),
        )
        .style(self.theme.accent_button_style(if self.is_playing {
            iced::Color::from_rgb8(0xe5, 0xa9, 0x3c)
        } else {
            iced::Color::from_rgb8(0x2e, 0xcc, 0x71)
        }))
        .on_press(if self.is_playing {
            UiMessage::TransportPause
        } else {
            UiMessage::TransportPlay
        });

        let stop_btn = button(text("■ STOP").size(11))
            .style(self.theme.button_style(false))
            .on_press(UiMessage::TransportStop);

        let record_btn = button(
            text(if self.is_recording {
                "● REC [ON]"
            } else {
                "● REC"
            })
            .size(11),
        )
        .style(self.theme.accent_button_style(self.theme.record))
        .on_press(UiMessage::TransportRecord);

        let cycle_btn = button(text("🔁 CYCLE").size(11))
            .style(
                self.theme
                    .button_style(self.arrangement.timeline.loop_region.enabled),
            )
            .on_press(UiMessage::TransportToggleLoop);

        // 1. Time Display Panel (Bar:Beat:Tick & SMPTE Timecode)
        let bar = (self.playhead_tick / 3840) + 1;
        let beat = ((self.playhead_tick % 3840) / 960) + 1;
        let tick = self.playhead_tick % 960;
        let total_seconds = self.playhead_tick as f64 / 1920.0;
        let mins = (total_seconds / 60.0) as u32;
        let secs = (total_seconds % 60.0) as u32;
        let millis = ((total_seconds * 1000.0) % 1000.0) as u32;

        let time_lcd = container(
            column![
                text(format!("{:03} . {:02} . {:03}", bar, beat, tick)).size(12),
                text(format!(
                    "SMPTE {:02}:{:02}:{:02}.{:03}",
                    mins / 60,
                    mins % 60,
                    secs,
                    millis
                ))
                .size(9),
            ]
            .spacing(1),
        )
        .style(|_t| self.theme.oled_vfd_display_style())
        .padding(4);

        // 2. Tempo & Signature Hub
        let tempo_lcd = container(
            column![
                text(format!(
                    "{:.2} BPM  {}/{}",
                    self.workspace.lcd_diagnostic.project_tempo_bpm,
                    self.workspace.lcd_diagnostic.time_signature.0,
                    self.workspace.lcd_diagnostic.time_signature.1
                ))
                .size(11),
                text(format!(
                    "KEY: C min | A4: {:.1} Hz",
                    self.workspace.lcd_diagnostic.master_tuning_hz
                ))
                .size(9),
            ]
            .spacing(1),
        )
        .style(|_t| self.theme.oled_vfd_display_style())
        .padding(4);

        // 3. Audio Engine DSP & System Telemetry
        let dsp_lcd = container(
            column![
                text(format!(
                    "DSP: {:.1}% | 2.6ms",
                    self.workspace.lcd_diagnostic.cpu_load_percent
                ))
                .size(11),
                text("48.0 kHz | 128 spl | 32v").size(9),
            ]
            .spacing(1),
        )
        .style(|_t| self.theme.oled_vfd_display_style())
        .padding(4);

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
            cycle_btn,
            time_lcd,
            tempo_lcd,
            dsp_lcd,
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

            let track_types: Vec<_> = self
                .arrangement
                .timeline
                .tracks
                .iter()
                .map(|t| t.track_type)
                .collect();

            let mut canvas_program = crate::ui::canvas_timeline::ArrangementCanvasProgram::new(
                self.playhead_tick,
                self.arrangement.horizontal_zoom,
                self.arrangement.timeline.tracks.len().max(1),
            )
            .with_track_types(track_types);

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

                let track_accent = self.theme.track_accent_color(trk_idx);

                let mute_btn = button(text("M").size(10))
                    .padding(3)
                    .style(
                        self.theme
                            .toggle_accent_button_style(is_muted, self.theme.mute),
                    )
                    .on_press(UiMessage::ToggleTrackMute(trk_idx));

                let solo_btn = button(text("S").size(10))
                    .padding(3)
                    .style(
                        self.theme
                            .toggle_accent_button_style(is_solo, self.theme.solo),
                    )
                    .on_press(UiMessage::ToggleTrackSolo(trk_idx));

                let rec_btn = button(text(if track.record_armed { "● R" } else { "R" }).size(10))
                    .padding(3)
                    .style(
                        self.theme
                            .toggle_accent_button_style(track.record_armed, self.theme.record),
                    )
                    .on_press(UiMessage::ToggleTrackRecordArm(trk_idx));

                let freeze_btn = button(text(if track.is_frozen { "❄" } else { "·" }).size(10))
                    .padding(3)
                    .style(self.theme.button_style(track.is_frozen))
                    .on_press(UiMessage::ToggleTrackFreeze(trk_idx));

                let pan_txt = if track.pan.abs() < 0.05 {
                    "Pan: C".to_string()
                } else if track.pan < 0.0 {
                    format!("Pan: L{:.0}%", (-track.pan) * 100.0)
                } else {
                    format!("Pan: R{:.0}%", track.pan * 100.0)
                };

                // Vertical color ribbon on left edge
                let accent_strip = container(column![])
                    .width(Length::Fixed(4.0))
                    .height(Length::Fill)
                    .style(move |_t| container::Style {
                        background: Some(iced::Background::Color(track_accent)),
                        ..container::Style::default()
                    });

                let track_inner = column![
                    row![
                        container(text(type_badge).size(9))
                            .padding(2)
                            .style(move |_t| container::Style {
                                background: Some(iced::Background::Color(iced::Color::from_rgba8(
                                    0, 0, 0, 0.35
                                ))),
                                border: iced::Border {
                                    color: track_accent,
                                    width: 1.0,
                                    radius: iced::border::Radius::from(3.0),
                                },
                                ..container::Style::default()
                            }),
                        button(text(&track.name).size(11))
                            .padding(1)
                            .on_press(UiMessage::SelectTrack(trk_idx)),
                    ]
                    .spacing(4),
                    row![mute_btn, solo_btn, rec_btn, freeze_btn].spacing(3),
                    row![
                        text(format!("{:.1} dB | {}", track.gain_db, pan_txt)).size(9),
                        text(if is_muted {
                            "▪▪▫▫"
                        } else {
                            "▪▪▪▪▪▫"
                        })
                        .size(8),
                    ]
                    .spacing(4),
                ]
                .spacing(2)
                .width(Length::Fill);

                let track_card =
                    container(row![accent_strip, container(track_inner).padding(3)].spacing(4))
                        .height(Length::Fixed(crate::ui::canvas_timeline::LANE_HEIGHT))
                        .width(Length::Fill)
                        .style(move |_t| {
                            let mut st = self.theme.panel_container_style();
                            if is_selected {
                                st.border.color = self.theme.border_focus;
                                st.border.width = 1.5;
                            }
                            st
                        });

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
            let is_fl = self.theme.preset == crate::ui::theme::ThemePreset::FlDarkOrange;
            let arrangement_canvas = crate::ui::canvas_timeline::view_canvas_timeline(
                canvas_program
                    .with_snap(self.snap_grid)
                    .with_theme(is_fl)
                    .with_scroll_y(self.timeline_scroll_y)
                    .with_marquee(self.marquee_selection),
            );

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
        let is_open = self.workspace.editor_drawer_open
            || self.fl_windows.mixer_visible
            || self.fl_windows.piano_roll_visible
            || self.fl_windows.channel_rack_visible
            || self.fl_windows.patcher_visible;

        if !is_open {
            return column![].into();
        }

        // Top Drawer Tab Navigation Bar
        let tab_piano = button(text("Piano Roll [F7]").size(11))
            .style(self.theme.button_style(
                self.layout.active_drawer_tab == crate::ui::layout::DrawerTab::PianoRoll,
            ))
            .on_press(UiMessage::SelectDrawerTab(
                crate::ui::layout::DrawerTab::PianoRoll,
            ));

        let tab_step = button(text("Step Sequencer [F6]").size(11))
            .style(self.theme.button_style(
                self.layout.active_drawer_tab == crate::ui::layout::DrawerTab::StepSequencer,
            ))
            .on_press(UiMessage::SelectDrawerTab(
                crate::ui::layout::DrawerTab::StepSequencer,
            ));

        let tab_mixer =
            button(text("Mixer Console [F9]").size(11))
                .style(self.theme.button_style(
                    self.layout.active_drawer_tab == crate::ui::layout::DrawerTab::Mixer,
                ))
                .on_press(UiMessage::SelectDrawerTab(
                    crate::ui::layout::DrawerTab::Mixer,
                ));

        let tab_patcher = button(text("Modular Patcher").size(11))
            .style(self.theme.button_style(false))
            .on_press(UiMessage::ToggleFlPatcher);

        let close_btn = button(text("✕").size(10)).on_press(UiMessage::ToggleEditorDrawer);

        let tab_bar = row![tab_piano, tab_step, tab_mixer, tab_patcher, close_btn,]
            .spacing(6)
            .padding(4);

        // Drawer Content per Active Tab
        let content_element: Element<'_, UiMessage> = match self.layout.active_drawer_tab {
            crate::ui::layout::DrawerTab::StepSequencer => {
                // FL Studio Signature 16-Step Channel Rack Sequencer
                let mut rack_col = column![].spacing(6).padding(4);

                for ch in &self.channel_rack.channels {
                    let mut steps_row = row![].spacing(3);

                    // 16 Step Buttons with 4-beat alternating grouping & tactile hardware styling
                    for (s_idx, step) in ch.steps.iter().enumerate() {
                        let is_active = step.active;
                        let is_even_beat = (s_idx / 4) % 2 == 0;
                        let step_label = if is_active { "■" } else { "·" };

                        let step_btn = button(text(step_label).size(11))
                            .style(self.theme.step_pad_style(is_active, is_even_beat))
                            .padding(2)
                            .on_press(UiMessage::StepSequencerToggleStep {
                                channel_id: ch.id,
                                step_index: s_idx,
                            });

                        steps_row = steps_row.push(step_btn);
                    }

                    let ch_row = row![
                        container(text(&ch.name).size(11))
                            .width(Length::Fixed(120.0))
                            .padding(2),
                        steps_row,
                    ]
                    .spacing(8);

                    rack_col = rack_col.push(ch_row);
                }

                column![
                    text("FL Studio Channel Rack (16-Step Pattern Sequencer)").size(11),
                    scrollable(rack_col),
                ]
                .spacing(4)
                .into()
            }

            crate::ui::layout::DrawerTab::PianoRoll => {
                // Interactive Piano Roll with Alt-Tools & Polyphonic Ghost Notes
                let btn_strum =
                    button(text("Strum").size(10)).on_press(UiMessage::PianoRollQuickStrum);
                let btn_flip_h =
                    button(text("Flip H").size(10)).on_press(UiMessage::PianoRollFlipHorizontal);
                let btn_flip_v =
                    button(text("Flip V").size(10)).on_press(UiMessage::PianoRollFlipVertical);
                let btn_chop =
                    button(text("Chop 1/16").size(10)).on_press(UiMessage::PianoRollQuickChop);
                let btn_add_c4 =
                    button(text("+ Note C4").size(10)).on_press(UiMessage::PianoRollAddNote {
                        channel_id: 0,
                        key: 60,
                        start_tick: self.playhead_tick,
                        length_ticks: 960,
                    });

                let roll_header = row![
                    text("FL Studio Piano Roll (Ghost Polyphony: Active)").size(11),
                    btn_strum,
                    btn_flip_h,
                    btn_flip_v,
                    btn_chop,
                    btn_add_c4,
                    text(format!(
                        "Notes: {} | Scale: Chromatic | Grid: {}",
                        self.piano_roll_notes.len(),
                        self.snap_grid.label()
                    ))
                    .size(10),
                ]
                .spacing(6);

                let roll_canvas = crate::ui::canvas_piano_roll::view_canvas_piano_roll(
                    self.piano_roll_canvas_program(),
                );
                let roll_body = container(roll_canvas)
                    .width(Length::Fill)
                    .height(Length::Fill);

                column![roll_header, roll_body].spacing(4).into()
            }

            crate::ui::layout::DrawerTab::Mixer => {
                // Neve / SSL Studio Console Channel Strip Rack
                let mut channel_strips = row![].spacing(6).padding(4);

                for (idx, ch) in self.mixer.channels.iter().enumerate() {
                    let strip_accent = self.theme.track_accent_color(idx);

                    let mute_btn = button(text("M").size(10))
                        .padding(3)
                        .style(
                            self.theme
                                .toggle_accent_button_style(ch.muted, self.theme.mute),
                        )
                        .on_press(UiMessage::ToggleTrackMute(idx));

                    let solo_btn = button(text("S").size(10))
                        .padding(3)
                        .style(
                            self.theme
                                .toggle_accent_button_style(ch.solo, self.theme.solo),
                        )
                        .on_press(UiMessage::ToggleTrackSolo(idx));

                    let fader_slider = slider(-60.0..=6.0, ch.volume_fader_db, move |vol| {
                        UiMessage::SetTrackFader {
                            track_id: idx,
                            volume_db: vol,
                        }
                    })
                    .step(0.5_f32)
                    .width(Length::Fixed(60.0));

                    let pan_slider = slider(-1.0..=1.0, ch.pan.balance, move |p| {
                        UiMessage::SetTrackPan {
                            track_id: idx,
                            pan: p,
                        }
                    })
                    .step(0.05_f32)
                    .width(Length::Fixed(60.0));

                    let pan_txt = if ch.pan.balance.abs() < 0.05 {
                        "Pan: C".to_string()
                    } else if ch.pan.balance < 0.0 {
                        format!("Pan: L{:.0}%", (-ch.pan.balance) * 100.0)
                    } else {
                        format!("Pan: R{:.0}%", ch.pan.balance * 100.0)
                    };

                    // Insert rack preview slots
                    let fx_slots = row![
                        container(text("EQ").size(8)).padding(1),
                        container(text("DYN").size(8)).padding(1),
                        container(text("REV").size(8)).padding(1),
                        container(text("+").size(8)).padding(1),
                    ]
                    .spacing(2);

                    // Dual L/R 12-segment graduated LED meters
                    let meter_bars = column![
                        text(if ch.volume_fader_db > 0.0 {
                            "CLIP"
                        } else {
                            "■■■■■■■■▫▫▫▫"
                        })
                        .size(7),
                        text(if ch.volume_fader_db > 0.0 {
                            "CLIP"
                        } else {
                            "■■■■■■■■▫▫▫▫"
                        })
                        .size(7),
                    ]
                    .spacing(1);

                    let strip = container(
                        column![
                            // Top strip routing badge
                            row![
                                container(column![])
                                    .width(Length::Fixed(3.0))
                                    .height(Length::Fixed(12.0))
                                    .style(move |_t| container::Style {
                                        background: Some(iced::Background::Color(strip_accent)),
                                        ..container::Style::default()
                                    }),
                                text(format!("CH {:02}", idx + 1)).size(9),
                            ]
                            .spacing(3),
                            fx_slots,
                            text(&ch.name).size(11),
                            text(pan_txt).size(9),
                            pan_slider,
                            row![fader_slider, meter_bars].spacing(4),
                            text(format!("{:.1} dB", ch.volume_fader_db)).size(10),
                            row![mute_btn, solo_btn].spacing(4),
                        ]
                        .spacing(2),
                    )
                    .style(|_t| self.theme.sunken_container_style())
                    .padding(4)
                    .width(Length::Fixed(110.0));

                    channel_strips = channel_strips.push(strip);
                }

                column![
                    text("Logic Pro / FL Studio 256-Bus Topological Mixing Console").size(11),
                    scrollable(channel_strips),
                ]
                .spacing(4)
                .into()
            }

            crate::ui::layout::DrawerTab::Automation => column![
                text("Automation Curve Drawer").size(11),
                text("Select clip or parameter to edit spline curves").size(10),
            ]
            .spacing(4)
            .into(),
        };

        container(column![tab_bar, content_element].spacing(4))
            .style(|_t| self.theme.panel_container_style())
            .height(Length::Fixed(220.0))
            .width(Length::Fill)
            .padding(4)
            .into()
    }

    /// Studio Telemetry Status Bar (Bottom 20px)
    fn view_status_bar(&self) -> Element<'_, UiMessage> {
        let left_telemetry = row![
            text("● ENGINE: ACTIVE").size(9),
            text("|").size(9),
            text(format!("RATE: {:.1} kHz", self.mixer.sample_rate / 1000.0)).size(9),
            text("|").size(9),
            text("BUFFER: 128 spl (2.6 ms)").size(9),
            text("|").size(9),
            text("XRUNS: 0").size(9),
        ]
        .spacing(6);

        let center_tool = row![
            text("TOOL: [ SMART SELECT / TRIM / FADE ]").size(9),
            text("|").size(9),
            text(format!("GRID: [ {} ]", self.snap_grid.label())).size(9),
        ]
        .spacing(6);

        let right_env = row![
            text("BACKEND: PIPEWIRE / ALSA").size(9),
            text("|").size(9),
            text("HOST: LINUX (COSMIC)").size(9),
            text("|").size(9),
            text("PEAK: -0.1 dBFS").size(9),
        ]
        .spacing(6);

        container(
            row![
                left_telemetry,
                container(center_tool)
                    .width(Length::Fill)
                    .align_x(iced::alignment::Horizontal::Center),
                right_env,
            ]
            .width(Length::Fill)
            .padding(2),
        )
        .style(|_t| self.theme.header_container_style())
        .height(Length::Fixed(20.0))
        .width(Length::Fill)
        .into()
    }

    /// Quick Command Palette Modal Overlay (Ctrl+K / ⌘K) with live search filter
    fn view_command_palette(&self) -> Element<'_, UiMessage> {
        let title = text("Crystal Wave Command Palette").size(13);
        let subtitle = text("Fast action launcher — type to search commands").size(10);

        let search_box = text_input(
            "Search actions (play, record, theme, snap, track, save, bounce)...",
            &self.command_query,
        )
        .on_input(UiMessage::CommandPaletteInput)
        .padding(6)
        .size(11);

        let query = self.command_query.to_lowercase();

        let all_commands = [
            ("▶ Transport: Play / Pause (Space)", "play"),
            ("● Transport: Toggle Record Overdub (R)", "record"),
            ("■ Transport: Stop and Rewind (Enter)", "stop"),
            ("[Theme] Switch to FL Studio Dark Orange", "orange"),
            ("[Theme] Switch to Logic Pro Graphite", "logic"),
            ("[Grid] Snap: Sixteenth Beat (1/16)", "snap"),
            ("[Project] Save Session (Ctrl+S)", "save"),
            ("[Export] Bounce Mix to 24-bit WAV", "bounce"),
            ("[Track] Add New Audio Track", "audio"),
            ("[Track] Add New MIDI Instrument Track", "midi"),
            ("[View] Toggle FL Playlist Window (F5)", "playlist"),
            ("[View] Toggle Channel Rack Matrix (F6)", "rack"),
            ("[View] Toggle Piano Roll (F7)", "piano"),
            ("[View] Toggle Mixer Console (F9)", "mixer"),
        ];

        let mut cmd_list = column![].spacing(3);
        for (label, action_key) in all_commands {
            if query.is_empty()
                || label.to_lowercase().contains(&query)
                || action_key.contains(&query)
            {
                let cmd_btn = button(text(label).size(10))
                    .style(self.theme.button_style(false))
                    .on_press(UiMessage::ExecuteCommand(action_key.to_string()))
                    .width(Length::Fill);
                cmd_list = cmd_list.push(cmd_btn);
            }
        }

        let close_btn = button(text("✕ Close").size(10)).on_press(UiMessage::ToggleCommandPalette);

        container(
            column![
                row![
                    title,
                    container(close_btn)
                        .width(Length::Fill)
                        .align_x(iced::alignment::Horizontal::Right),
                ]
                .spacing(4),
                subtitle,
                search_box,
                scrollable(cmd_list).height(Length::Fixed(160.0)),
            ]
            .spacing(6)
            .padding(10),
        )
        .style(|_t| self.theme.panel_container_style())
        .width(Length::Fixed(440.0))
        .into()
    }
}
