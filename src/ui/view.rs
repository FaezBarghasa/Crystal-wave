use crate::ui::app::CrystalWaveApp;
use crate::ui::message::{MenuCommand, PianoRollTool, PlaylistTool, UiMessage};
use iced::widget::{
    Space, button, column, container, row, scrollable, slider, stack, text, text_input,
    vertical_slider,
};
use iced::{Color, Element, Length};

impl CrystalWaveApp {
    /// Render standard FL Studio sub-window titlebar chrome (28px) with minimize, maximize/tile, and close controls
    pub fn view_window_chrome<'a>(
        &'a self,
        title: &'static str,
        icon: &'static str,
        close_msg: UiMessage,
    ) -> Element<'a, UiMessage> {
        let icon_w = container(text(icon).size(11).color(self.theme.accent)).padding([0, 6]);
        let title_w = text(title).size(11).color(self.theme.text_secondary);
        let spacer = Space::new().width(Length::Fill);

        let min_btn = button(text("─").size(9))
            .padding([2, 5])
            .style(self.theme.button_style(false))
            .on_press(close_msg.clone());
        let is_dual = self.layout.secondary_view.is_some();
        let max_btn = button(text(if is_dual { "❐" } else { "◫" }).size(9))
            .padding([2, 5])
            .style(self.theme.button_style(is_dual))
            .on_press(UiMessage::ToggleDualWorkspace(None));
        let close_btn = button(text("×").size(11))
            .padding([2, 5])
            .style(self.theme.button_style(false))
            .on_press(if is_dual {
                UiMessage::CloseDualWorkspace
            } else {
                close_msg
            });

        container(
            row![icon_w, title_w, spacer, min_btn, max_btn, close_btn]
                .align_y(iced::Alignment::Center)
                .spacing(3),
        )
        .height(Length::Fixed(28.0))
        .width(Length::Fill)
        .padding([2, 6])
        .style(|_t| self.theme.window_titlebar_style())
        .into()
    }

    pub fn view(&self) -> Element<'_, UiMessage> {
        let menu_bar = self.view_menu_bar();
        let control_bar = self.view_control_bar();
        let main_workspace = self.view_main_workspace();

        // Suppress bottom drawer when the workspace already displays that view
        let is_redundant_drawer = matches!(
            self.layout.active_view,
            crate::ui::layout::FlActiveView::PianoRoll
                | crate::ui::layout::FlActiveView::Mixer
                | crate::ui::layout::FlActiveView::ChannelRack
                | crate::ui::layout::FlActiveView::Edison
                | crate::ui::layout::FlActiveView::Fpc
                | crate::ui::layout::FlActiveView::Slicex
        );

        let bottom_drawer: Element<'_, UiMessage> = if is_redundant_drawer {
            column![].into()
        } else {
            self.view_bottom_drawer()
        };

        let status_bar = self.view_status_bar();

        let main_col = column![
            menu_bar,
            control_bar,
            main_workspace,
            bottom_drawer,
            status_bar
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(1);

        let base_content = container(main_col)
            .style(|_t| self.theme.header_container_style())
            .width(Length::Fill)
            .height(Length::Fill);

        // Centered Floating Modal Overlays with Dimmed Backdrop (FL Studio / Modern 2026 UX)
        let mut root_stack = stack![base_content];

        if let Some(chan_id) = self.active_channel_settings {
            let settings_modal = self.view_channel_settings_modal(chan_id);
            let overlay = container(settings_modal)
                .width(Length::Fill)
                .height(Length::Fill)
                .center(Length::Fill)
                .style(|_t| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        0.0, 0.0, 0.0, 0.70,
                    ))),
                    ..Default::default()
                });
            root_stack = root_stack.push(overlay);
        } else if self.fl_windows.plugin_picker_visible {
            let picker = self.view_plugin_picker();
            let overlay = container(picker)
                .width(Length::Fill)
                .height(Length::Fill)
                .center(Length::Fill)
                .style(|_t| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        0.0, 0.0, 0.0, 0.70,
                    ))),
                    ..Default::default()
                });
            root_stack = root_stack.push(overlay);
        } else if self.audio_settings_open {
            let audio_modal = self.view_audio_settings_modal();
            let overlay = container(audio_modal)
                .width(Length::Fill)
                .height(Length::Fill)
                .center(Length::Fill)
                .style(|_t| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        0.0, 0.0, 0.0, 0.70,
                    ))),
                    ..Default::default()
                });
            root_stack = root_stack.push(overlay);
        } else if self.command_palette_open {
            let palette = self.view_command_palette();
            let overlay = container(palette)
                .width(Length::Fill)
                .height(Length::Fill)
                .center(Length::Fill)
                .style(|_t| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgba(
                        0.0, 0.0, 0.0, 0.70,
                    ))),
                    ..Default::default()
                });
            root_stack = root_stack.push(overlay);
        }

        root_stack.into()
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

        // FL Studio Multi-Window Toolbars (F5, F6, F7, F8, F9, F10) with LED jewel indicators
        let is_pl = self.layout.active_view == crate::ui::layout::FlActiveView::Playlist;
        let is_cr = self.layout.active_view == crate::ui::layout::FlActiveView::ChannelRack;
        let is_pr = self.layout.active_view == crate::ui::layout::FlActiveView::PianoRoll;
        let is_mx = self.layout.active_view == crate::ui::layout::FlActiveView::Mixer;
        let is_ed = self.layout.active_view == crate::ui::layout::FlActiveView::Edison;
        let is_fpc = self.layout.active_view == crate::ui::layout::FlActiveView::Fpc;
        let is_sl = self.layout.active_view == crate::ui::layout::FlActiveView::Slicex;
        let is_pat = self.layout.active_view == crate::ui::layout::FlActiveView::Patcher;

        let win_playlist = button(
            text(if is_pl {
                "● Playlist [F5]"
            } else {
                "○ Playlist [F5]"
            })
            .size(11),
        )
        .style(self.theme.button_style(is_pl))
        .on_press(UiMessage::SelectFlView(
            crate::ui::layout::FlActiveView::Playlist,
        ));

        let win_channel_rack = button(
            text(if is_cr {
                "● Rack [F6]"
            } else {
                "○ Rack [F6]"
            })
            .size(11),
        )
        .style(self.theme.button_style(is_cr))
        .on_press(UiMessage::SelectFlView(
            crate::ui::layout::FlActiveView::ChannelRack,
        ));

        let win_piano_roll = button(
            text(if is_pr {
                "● Piano Roll [F7]"
            } else {
                "○ Piano Roll [F7]"
            })
            .size(11),
        )
        .style(self.theme.button_style(is_pr))
        .on_press(UiMessage::SelectFlView(
            crate::ui::layout::FlActiveView::PianoRoll,
        ));

        let win_browser = button(
            text(if self.fl_windows.browser_visible {
                "● Browser [F8]"
            } else {
                "○ Browser [F8]"
            })
            .size(11),
        )
        .style(self.theme.button_style(self.fl_windows.browser_visible))
        .on_press(UiMessage::ToggleFlBrowser);

        let win_mixer = button(
            text(if is_mx {
                "● Mixer [F9]"
            } else {
                "○ Mixer [F9]"
            })
            .size(11),
        )
        .style(self.theme.button_style(is_mx))
        .on_press(UiMessage::SelectFlView(
            crate::ui::layout::FlActiveView::Mixer,
        ));

        let win_audio = button(
            text(if self.audio_settings_open {
                "● Audio [F10]"
            } else {
                "○ Audio [F10]"
            })
            .size(11),
        )
        .style(self.theme.button_style(self.audio_settings_open))
        .on_press(UiMessage::ToggleAudioSettings);

        let win_edison = button(text(if is_ed { "● Edison" } else { "○ Edison" }).size(11))
            .style(self.theme.button_style(is_ed))
            .on_press(UiMessage::SelectFlView(
                crate::ui::layout::FlActiveView::Edison,
            ));

        let win_fpc = button(text(if is_fpc { "● FPC" } else { "○ FPC" }).size(11))
            .style(self.theme.button_style(is_fpc))
            .on_press(UiMessage::SelectFlView(
                crate::ui::layout::FlActiveView::Fpc,
            ));

        let win_slicex = button(text(if is_sl { "● Slicex" } else { "○ Slicex" }).size(11))
            .style(self.theme.button_style(is_sl))
            .on_press(UiMessage::SelectFlView(
                crate::ui::layout::FlActiveView::Slicex,
            ));

        let win_patcher = button(text(if is_pat { "● Patcher" } else { "○ Patcher" }).size(11))
            .style(self.theme.button_style(is_pat))
            .on_press(UiMessage::SelectFlView(
                crate::ui::layout::FlActiveView::Patcher,
            ));

        let win_inspector = button(
            text(if self.workspace.inspector_open {
                "● Inspect [I]"
            } else {
                "○ Inspect [I]"
            })
            .size(11),
        )
        .style(self.theme.button_style(self.workspace.inspector_open))
        .on_press(UiMessage::ToggleInspector);

        let win_plugins = button(
            text(if self.fl_windows.plugin_picker_visible {
                "● Plugins"
            } else {
                "○ Plugins"
            })
            .size(11),
        )
        .style(
            self.theme
                .button_style(self.fl_windows.plugin_picker_visible),
        )
        .on_press(UiMessage::ToggleFlPluginPicker);

        let win_dual = {
            let is_dual = self.layout.secondary_view.is_some();
            button(
                text(if is_dual {
                    "◫ Dual [Split]"
                } else {
                    "❐ Single [Full]"
                })
                .size(11),
            )
            .style(self.theme.button_style(is_dual))
            .on_press(UiMessage::ToggleDualWorkspace(None))
        };

        // Theme Switcher & Snap Toggle
        let theme_btn = button(
            text(match self.theme.preset {
                crate::ui::theme::ThemePreset::Graphite => "Theme: Logic Graphite",
                crate::ui::theme::ThemePreset::FlDarkOrange => "Theme: FL Dark Orange",
                crate::ui::theme::ThemePreset::FlStudioModern => "Theme: FL 2026 Modern",
            })
            .size(11),
        )
        .on_press(UiMessage::SetThemePreset(match self.theme.preset {
            crate::ui::theme::ThemePreset::Graphite => crate::ui::theme::ThemePreset::FlDarkOrange,
            crate::ui::theme::ThemePreset::FlDarkOrange => {
                crate::ui::theme::ThemePreset::FlStudioModern
            }
            crate::ui::theme::ThemePreset::FlStudioModern => {
                crate::ui::theme::ThemePreset::Graphite
            }
        }));

        let snap_btn = button(text(format!("⊞ {}", self.snap_grid.label())).size(11)).on_press(
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

        // Modular machined floating capsules
        let brand_logo = container(
            text("◆ CRYSTAL WAVE 24")
                .size(12)
                .color(Color::from_rgb8(0xff, 0x88, 0x00)),
        )
        .padding(4);

        let capsule_file = container(
            row![brand_logo, file_new, file_save, file_bounce, file_aaf,]
                .spacing(4)
                .align_y(iced::Alignment::Center),
        )
        .style(|_t| self.theme.machined_capsule_style())
        .padding(3);

        let capsule_tools =
            container(row![edit_split, track_add_audio, track_add_midi, snap_btn,].spacing(4))
                .style(|_t| self.theme.machined_capsule_style())
                .padding(3);

        let capsule_windows = container(
            row![
                win_playlist,
                win_channel_rack,
                win_piano_roll,
                win_browser,
                win_mixer,
                win_audio,
                win_edison,
                win_fpc,
                win_slicex,
                win_patcher,
                win_inspector,
                win_plugins,
                win_dual,
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center),
        )
        .style(|_t| self.theme.machined_capsule_style())
        .padding(3);

        let capsule_system = container(row![theme_btn, cmd_palette_btn,].spacing(4))
            .style(|_t| self.theme.machined_capsule_style())
            .padding(3);

        // Row 1: Top Titlebar Row (22px) with system and project telemetry
        let title_controls = row![
            button(text("─").size(9))
                .padding([1, 4])
                .style(self.theme.button_style(false)),
            button(text("□").size(9))
                .padding([1, 4])
                .style(self.theme.button_style(false)),
            button(text("×").size(10))
                .padding([1, 4])
                .style(self.theme.button_style(false)),
        ]
        .spacing(2)
        .align_y(iced::Alignment::Center);

        let title_project = text("Crystal-wave — Untitled.flp")
            .size(10)
            .color(self.theme.text_secondary);

        let (cpu_load, ram_mb) = (self.workspace.lcd_diagnostic.cpu_load_percent, 526);
        let title_stats = text(format!("RAM: {}MB | CPU: {:.0}%", ram_mb, cpu_load))
            .size(9)
            .color(self.theme.text_muted);

        let row_titlebar = container(
            row![
                title_controls,
                Space::new().width(Length::Fixed(8.0)),
                title_project,
                Space::new().width(Length::Fill),
                title_stats,
            ]
            .align_y(iced::Alignment::Center)
            .spacing(4),
        )
        .height(Length::Fixed(22.0))
        .padding([2, 6])
        .style(|_t| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb8(0x0f, 0x0f, 0x12))),
            text_color: Some(Color::from_rgb8(0x88, 0x8c, 0x99)),
            border: iced::Border {
                color: Color::from_rgb8(0x20, 0x22, 0x28),
                width: 0.5,
                ..Default::default()
            },
            ..Default::default()
        });

        // Row 2: Standard FL Studio Menu Ribbon + Window Capsules
        let row_menubar = row![capsule_file, capsule_tools, capsule_windows, capsule_system,]
            .spacing(6)
            .padding(2)
            .align_y(iced::Alignment::Center);

        // Row 3: Hint Bar (18px, FL Studio LCD Telemetry readout)
        let hint_text = self.active_hover_hint.as_deref().unwrap_or(
            "Hint: Click to select, drag to adjust, double-click or right-click to reset",
        );
        let hint_color = if self.active_hover_hint.is_some() {
            self.theme.accent
        } else {
            self.theme.text_muted
        };

        let row_hintbar = container(
            row![
                text(hint_text).size(9).color(hint_color),
                Space::new().width(Length::Fill),
                text("SYNC ✓").size(9).color(self.theme.accent),
            ]
            .align_y(iced::Alignment::Center)
            .padding([1, 6]),
        )
        .height(Length::Fixed(18.0))
        .style(|_t| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb8(0x18, 0x1a, 0x20))),
            border: iced::Border {
                color: Color::from_rgb8(0x28, 0x2a, 0x32),
                width: 0.5,
                ..Default::default()
            },
            ..Default::default()
        });

        column![row_titlebar, row_menubar, row_hintbar]
            .spacing(1)
            .into()
    }

    fn view_control_bar(&self) -> Element<'_, UiMessage> {
        // Module 1: Transport Pill (PAT / SONG mode toggle, Play/Pause, Stop, Rec, Loop)
        let pill_accent = if self.is_song_mode {
            Color::from_rgb8(0x2e, 0xcc, 0x71)
        } else {
            Color::from_rgb8(0xff, 0x88, 0x00)
        };

        // Module 1: Circular Transport Controls (Loop, Play/Pause, Stop, Rec)
        let pat_song_pill = button(
            row![
                container(text("PAT").size(9).color(if !self.is_song_mode {
                    Color::from_rgb8(0xff, 0x88, 0x00)
                } else {
                    Color::from_rgb8(0x60, 0x66, 0x72)
                }),)
                .padding([1, 3]),
                container(text("SONG").size(9).color(if self.is_song_mode {
                    Color::from_rgb8(0x2e, 0xcc, 0x71)
                } else {
                    Color::from_rgb8(0x60, 0x66, 0x72)
                }),)
                .padding([1, 3]),
            ]
            .spacing(2)
            .align_y(iced::Alignment::Center),
        )
        .style(self.theme.transport_pill_style(true, pill_accent))
        .padding(2)
        .on_press(UiMessage::TogglePatternSongMode);

        let cycle_btn = button(
            container(text("↺").size(12))
                .width(Length::Fixed(24.0))
                .height(Length::Fixed(24.0))
                .center(Length::Fill),
        )
        .padding(1)
        .style(self.theme.circular_transport_btn_style(
            Color::from_rgb8(0xff, 0x85, 0x00),
            self.arrangement.timeline.loop_region.enabled,
        ))
        .on_press(UiMessage::TransportToggleLoop);

        let play_btn = button(
            container(text(if self.is_playing { "⏸" } else { "▶" }).size(12))
                .width(Length::Fixed(26.0))
                .height(Length::Fixed(26.0))
                .center(Length::Fill),
        )
        .padding(1)
        .style(self.theme.circular_transport_btn_style(
            if self.is_playing {
                Color::from_rgb8(0xe5, 0xa9, 0x3c)
            } else {
                Color::from_rgb8(0x2e, 0xcc, 0x71)
            },
            self.is_playing,
        ))
        .on_press(if self.is_playing {
            UiMessage::TransportPause
        } else {
            UiMessage::TransportPlay
        });

        let stop_btn = button(
            container(text("■").size(11))
                .width(Length::Fixed(24.0))
                .height(Length::Fixed(24.0))
                .center(Length::Fill),
        )
        .padding(1)
        .style(
            self.theme
                .circular_transport_btn_style(Color::from_rgb8(0x40, 0x45, 0x52), false),
        )
        .on_press(UiMessage::TransportStop);

        let record_btn = button(
            container(text("●").size(12))
                .width(Length::Fixed(24.0))
                .height(Length::Fixed(24.0))
                .center(Length::Fill),
        )
        .padding(1)
        .style(
            self.theme
                .circular_transport_btn_style(self.theme.record, self.is_recording),
        )
        .on_press(UiMessage::TransportRecord);

        // Module 2: Time Display Panel (Bar:Beat:Tick & SMPTE Timecode in VFD amber/cyan)
        let bar = (self.playhead_tick / 3840) + 1;
        let beat = ((self.playhead_tick % 3840) / 960) + 1;
        let tick = self.playhead_tick % 960;
        let total_seconds = self.playhead_tick as f64 / 1920.0;
        let mins = (total_seconds / 60.0) as u32;
        let secs = (total_seconds % 60.0) as u32;
        let millis = ((total_seconds * 1000.0) % 1000.0) as u32;

        let time_lcd = container(
            column![
                text(format!("{:03} : {:02} : {:03}", bar, beat, tick)).size(12),
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
        .style(|_t| self.theme.vfd_amber_display_style())
        .padding(4);

        // Module 3: Master Volume & Pitch Rotary Dials (FL Studio 2026 compact dials)
        let master_vol_knob = crate::ui::canvas_knob::rotary_knob(
            22.0,
            0.0,
            1.25,
            self.master_volume,
            false,
            Color::from_rgb8(0x2e, 0xcc, 0x71),
            UiMessage::SetMasterVolume,
        );
        let master_vol_txt = text(format!("{:.0}%", self.master_volume * 100.0)).size(8);

        let master_pitch_knob = crate::ui::canvas_knob::rotary_knob(
            22.0,
            -1200.0,
            1200.0,
            self.master_pitch_cents as f32,
            true,
            self.theme.accent,
            |c| UiMessage::SetMasterPitchCents(c.round() as i32),
        );
        let master_pitch_txt = text(if self.master_pitch_cents == 0 {
            "0 ct".to_string()
        } else {
            format!("{:+} ct", self.master_pitch_cents)
        })
        .size(8);

        let master_knobs_capsule = container(
            row![
                column![
                    text("VOL").size(7).color(self.theme.text_muted),
                    master_vol_knob,
                    master_vol_txt
                ]
                .align_x(iced::Alignment::Center)
                .spacing(1),
                column![
                    text("PITCH").size(7).color(self.theme.text_muted),
                    master_pitch_knob,
                    master_pitch_txt
                ]
                .align_x(iced::Alignment::Center)
                .spacing(1),
            ]
            .spacing(6)
            .align_y(iced::Alignment::Center),
        )
        .style(|_t| self.theme.knob_well_style())
        .padding(3);

        // Module 4: Tempo & Signature Hub + Performance LEDs (MET, 3.2.1, KEY, BLEND)
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
        .style(|_t| self.theme.vfd_cyan_display_style())
        .padding(4);

        let btn_tap = button(text("TAP").size(9))
            .padding(3)
            .style(self.theme.button_style(false))
            .on_press(UiMessage::TapTempo);

        let btn_tempo_minus = button(text("▼").size(8))
            .padding(2)
            .style(self.theme.button_style(false))
            .on_press(UiMessage::NudgeTempo(-1.0));

        let btn_tempo_plus = button(text("▲").size(8))
            .padding(2)
            .style(self.theme.button_style(false))
            .on_press(UiMessage::NudgeTempo(1.0));

        let met_btn =
            button(
                text(if self.metronome_enabled {
                    "● MET"
                } else {
                    "○ MET"
                })
                .size(9),
            )
            .padding(2)
            .style(self.theme.toggle_accent_button_style(
                self.metronome_enabled,
                Color::from_rgb8(0x2e, 0xcc, 0x71),
            ))
            .on_press(UiMessage::ToggleMetronome);

        let precount_btn =
            button(
                text(if self.precount_enabled {
                    "● 3.2.1"
                } else {
                    "○ 3.2.1"
                })
                .size(9),
            )
            .padding(2)
            .style(self.theme.toggle_accent_button_style(
                self.precount_enabled,
                Color::from_rgb8(0xf3, 0x9c, 0x12),
            ))
            .on_press(UiMessage::TogglePrecount);

        let typing_btn = button(
            text(if self.typing_keyboard_to_piano {
                "● KEY"
            } else {
                "○ KEY"
            })
            .size(9),
        )
        .padding(2)
        .style(self.theme.toggle_accent_button_style(
            self.typing_keyboard_to_piano,
            Color::from_rgb8(0x00, 0xf2, 0xfe),
        ))
        .on_press(UiMessage::ToggleTypingKeyboard);

        let blend_btn = button(
            text(if self.blend_recording_enabled {
                "● BLEND"
            } else {
                "○ BLEND"
            })
            .size(9),
        )
        .padding(2)
        .style(self.theme.toggle_accent_button_style(
            self.blend_recording_enabled,
            Color::from_rgb8(0xff, 0x88, 0x00),
        ))
        .on_press(UiMessage::ToggleBlendRecording);

        let leds_row = row![met_btn, precount_btn, typing_btn, blend_btn].spacing(3);

        // Module 5: Audio Engine DSP & System Telemetry
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
        .style(|_t| self.theme.vfd_cyan_display_style())
        .padding(4);

        let capsule_transport = container(
            row![pat_song_pill, cycle_btn, play_btn, stop_btn, record_btn,]
                .spacing(4)
                .align_y(iced::Alignment::Center),
        )
        .style(|_t| self.theme.machined_capsule_style())
        .padding(3);

        let capsule_time = container(time_lcd)
            .style(|_t| self.theme.machined_capsule_style())
            .padding(3);

        let capsule_master = container(master_knobs_capsule)
            .style(|_t| self.theme.machined_capsule_style())
            .padding(3);

        let capsule_tempo = container(
            row![
                tempo_lcd,
                row![btn_tempo_minus, btn_tempo_plus].spacing(1),
                btn_tap,
                leds_row,
            ]
            .spacing(3)
            .align_y(iced::Alignment::Center),
        )
        .style(|_t| self.theme.machined_capsule_style())
        .padding(3);

        let capsule_dsp = container(dsp_lcd)
            .style(|_t| self.theme.machined_capsule_style())
            .padding(3);

        row![
            capsule_transport,
            capsule_time,
            capsule_master,
            capsule_tempo,
            capsule_dsp,
        ]
        .spacing(6)
        .padding(4)
        .align_y(iced::Alignment::Center)
        .into()
    }

    /// FL Studio Browser / Sample Library Panel (Left docked 260px)
    pub fn view_browser_panel(&self) -> Element<'_, UiMessage> {
        let tabs = [
            (crate::fl_studio::browser::BrowserTab::All, "All"),
            (crate::fl_studio::browser::BrowserTab::Drums, "Drums"),
            (crate::fl_studio::browser::BrowserTab::Bass, "808"),
            (crate::fl_studio::browser::BrowserTab::Instruments, "Inst"),
            (crate::fl_studio::browser::BrowserTab::Fx, "FX"),
            (crate::fl_studio::browser::BrowserTab::Plugins, "Plugins"),
            (crate::fl_studio::browser::BrowserTab::Presets, "Presets"),
        ];

        let mut tab_row = row![].spacing(2);
        for (tab_variant, tab_lbl) in tabs {
            let is_active = self.browser.active_tab == tab_variant;
            let tab_btn = button(text(tab_lbl).size(9))
                .padding(3)
                .style(self.theme.button_style(is_active))
                .on_press(UiMessage::BrowserSelectTab(tab_variant));
            tab_row = tab_row.push(tab_btn);
        }

        let search_input = text_input("⌕ Search library & tags...", &self.browser.search_query)
            .on_input(UiMessage::BrowserSetSearchQuery)
            .padding(4)
            .size(10);

        let mut items_col = column![].spacing(3).width(Length::Fill);

        match self.browser.active_tab {
            crate::fl_studio::browser::BrowserTab::Presets => {
                for preset in self.browser.filtered_presets() {
                    let card = container(
                        row![
                            text("◈").size(10).color(self.theme.accent),
                            column![
                                text(&preset.name).size(10),
                                text(format!("{} • {}", preset.category, preset.plugin_name))
                                    .size(8)
                                    .color(Color::from_rgb8(0x8a, 0x90, 0x9a)),
                            ]
                            .spacing(1)
                            .width(Length::Fill),
                        ]
                        .spacing(4)
                        .align_y(iced::Alignment::Center),
                    )
                    .padding(4)
                    .style(|_t| self.theme.inner_well_style());
                    items_col = items_col.push(card);
                }
            }
            crate::fl_studio::browser::BrowserTab::Plugins => {
                for plugin in self
                    .browser
                    .filtered_plugins(&self.browser.search_query, None)
                {
                    let pid = plugin.id;
                    let is_gen = plugin.is_generator;
                    let icon_glyph = if is_gen { "⬡" } else { "FX" };
                    let card = container(
                        row![
                            text(icon_glyph)
                                .size(10)
                                .color(Color::from_rgb8(0x00, 0xf2, 0xfe)),
                            column![
                                text(&plugin.name).size(10),
                                text(format!("{:?} • {}", plugin.category, plugin.vendor))
                                    .size(8)
                                    .color(Color::from_rgb8(0x8a, 0x90, 0x9a)),
                            ]
                            .spacing(1)
                            .width(Length::Fill),
                            button(text("+ Load").size(9))
                                .padding(3)
                                .style(self.theme.accent_button_style(self.theme.accent))
                                .on_press(UiMessage::PluginPickerLoadPlugin {
                                    plugin_id: pid,
                                    is_generator: is_gen,
                                }),
                        ]
                        .spacing(4)
                        .align_y(iced::Alignment::Center),
                    )
                    .padding(4)
                    .style(|_t| self.theme.inner_well_style());
                    items_col = items_col.push(card);
                }
            }
            _ => {
                for sample in self.browser.filtered_samples() {
                    let sample_id = sample.id;
                    let is_previewing = self.browser.previewing_sample_id == Some(sample_id);

                    let audition_btn = button(text(if is_previewing { "■" } else { "▶" }).size(9))
                        .padding(3)
                        .style(
                            self.theme
                                .toggle_accent_button_style(is_previewing, self.theme.record),
                        )
                        .on_press(if is_previewing {
                            UiMessage::BrowserStopAudition
                        } else {
                            UiMessage::BrowserAuditionSample(sample_id)
                        });

                    let rack_btn = button(text("+Rack").size(8))
                        .padding(2)
                        .on_press(UiMessage::ChannelRackDropSample { sample_id });

                    let lane_btn = button(text("+Lane").size(8)).padding(2).on_press(
                        UiMessage::BrowserDropSampleToPlaylist {
                            sample_id,
                            track_id: self.workspace.selected_track_id,
                            start_tick: self.playhead_tick,
                        },
                    );

                    let card = container(
                        row![
                            text("⬟").size(9).color(Color::from_rgb8(0x2e, 0xcc, 0x71)),
                            audition_btn,
                            column![
                                text(&sample.name).size(10),
                                text(format!(
                                    "{} | {:.1}s",
                                    sample.category, sample.duration_seconds
                                ))
                                .size(8)
                                .color(Color::from_rgb8(0x8a, 0x90, 0x9a)),
                            ]
                            .spacing(1)
                            .width(Length::Fill),
                            rack_btn,
                            lane_btn,
                        ]
                        .spacing(3)
                        .align_y(iced::Alignment::Center),
                    )
                    .padding(3)
                    .style(|_t| self.theme.inner_well_style());

                    items_col = items_col.push(card);
                }
            }
        }

        let browser_header = row![
            text("BROWSER / LIBRARY").size(11),
            Space::new().width(Length::Fill),
            button(text("✕").size(9))
                .padding(2)
                .on_press(UiMessage::ToggleFlBrowser),
        ]
        .align_y(iced::Alignment::Center);

        container(
            column![
                browser_header,
                tab_row,
                scrollable(items_col).height(Length::Fill),
                search_input,
            ]
            .spacing(4),
        )
        .width(Length::Fixed(260.0))
        .padding(6)
        .style(|_t| self.theme.sunken_container_style())
        .into()
    }

    /// FL Studio 24 Professional Playlist Free-Form Timeline & Track Headers
    pub fn view_playlist_window(&self) -> Element<'_, UiMessage> {
        let tools = [
            PlaylistTool::Draw,
            PlaylistTool::Paint,
            PlaylistTool::Delete,
            PlaylistTool::Mute,
            PlaylistTool::Slip,
            PlaylistTool::Slice,
            PlaylistTool::Select,
            PlaylistTool::Zoom,
        ];
        let mut tool_row = row![].spacing(2);
        for t in tools {
            let is_sel = self.active_playlist_tool == t;
            let btn = button(text(t.label()).size(9))
                .padding(2)
                .style(self.theme.button_style(is_sel))
                .on_press(UiMessage::SetPlaylistTool(t));
            tool_row = tool_row.push(btn);
        }

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
                    tool_row,
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center)
            )
            .height(Length::Fixed(crate::ui::canvas_timeline::RULER_HEIGHT))
            .width(Length::Fill)
            .padding(3),
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

            let accent_strip = container(column![])
                .width(Length::Fixed(4.0))
                .height(Length::Fill)
                .style(move |_t| container::Style {
                    background: Some(iced::Background::Color(track_accent)),
                    ..container::Style::default()
                });

            let fader_mini = slider(-60.0..=6.0, track.gain_db, move |v| {
                UiMessage::SetTrackFader {
                    track_id: trk_idx,
                    volume_db: v,
                }
            })
            .step(0.5_f32)
            .width(Length::Fixed(50.0));

            let pan_mini = slider(-1.0..=1.0, track.pan, move |p| UiMessage::SetTrackPan {
                track_id: trk_idx,
                pan: p,
            })
            .step(0.05_f32)
            .width(Length::Fixed(40.0));

            let track_inner = column![
                row![
                    container(text(type_badge).size(9))
                        .padding(2)
                        .style(move |_t| container::Style {
                            background: Some(iced::Background::Color(iced::Color::from_rgba8(
                                0, 0, 0, 0.35,
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
                    fader_mini,
                    pan_mini,
                    text(format!("{:.1} dB", track.gain_db)).size(8),
                ]
                .spacing(3),
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
                    crate::timeline::region::RegionKind::Automation {
                        parameter_target,
                        points,
                        ..
                    } => {
                        let pts = points
                            .iter()
                            .map(|p| (p.tick_offset, p.value, p.tension))
                            .collect();
                        crate::ui::canvas_timeline::ClipVisualKind::Automation {
                            parameter_target: parameter_target.clone(),
                            points: pts,
                        }
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

        let track_header_column = container(track_headers).width(Length::Fixed(220.0));
        let is_fl = matches!(
            self.theme.preset,
            crate::ui::theme::ThemePreset::FlDarkOrange
                | crate::ui::theme::ThemePreset::FlStudioModern
        );
        let arrangement_canvas = crate::ui::canvas_timeline::view_canvas_timeline(
            canvas_program
                .with_snap(self.snap_grid)
                .with_theme(is_fl)
                .with_scroll_y(self.timeline_scroll_y)
                .with_marquee(self.marquee_selection),
        );

        let chrome = self.view_window_chrome(
            "Playlist — Arrangement",
            "◈",
            UiMessage::SelectFlView(crate::ui::layout::FlActiveView::Playlist),
        );

        column![
            chrome,
            row![track_header_column, arrangement_canvas]
                .width(Length::Fill)
                .height(Length::Fill)
                .spacing(1)
        ]
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(0)
        .into()
    }

    /// Master FL Studio Workspace: Routes to Active Center View with Optional Docked Panels
    fn view_main_workspace(&self) -> Element<'_, UiMessage> {
        let mut content = row![].spacing(2).width(Length::Fill).height(Length::Fill);

        // 1. FL Studio Browser (Alt+F8 / F8) - Samples, Presets & Plugins Library
        if self.fl_windows.browser_visible {
            content = content.push(self.view_browser_panel());
        }

        // 2. Active Center Workspace View with Dual MDI Compositor Support
        let render_view = |v: crate::ui::layout::FlActiveView| -> Element<'_, UiMessage> {
            match v {
                crate::ui::layout::FlActiveView::Playlist => self.view_playlist_window(),
                crate::ui::layout::FlActiveView::ChannelRack => self.view_channel_rack(false),
                crate::ui::layout::FlActiveView::PianoRoll => self.view_piano_roll_window(),
                crate::ui::layout::FlActiveView::Mixer => self.view_mixer_window(),
                crate::ui::layout::FlActiveView::Edison => self.view_edison_window(),
                crate::ui::layout::FlActiveView::Fpc => self.view_fpc_window(),
                crate::ui::layout::FlActiveView::Slicex => self.view_slicex_window(),
                crate::ui::layout::FlActiveView::Patcher => self.view_patcher_window(),
                crate::ui::layout::FlActiveView::Browser => self.view_playlist_window(),
            }
        };

        if let Some(sec_view) = self.layout.secondary_view {
            // Dual Workspace Mode: Render primary and secondary view side-by-side or stacked
            let primary = container(render_view(self.layout.active_view))
                .width(Length::FillPortion(1))
                .height(Length::Fill);
            let secondary = container(render_view(sec_view))
                .width(Length::FillPortion(1))
                .height(Length::Fill);

            content = content.push(
                row![primary, secondary]
                    .spacing(3)
                    .width(Length::Fill)
                    .height(Length::Fill),
            );
        } else {
            let center_view = render_view(self.layout.active_view);
            content = content.push(
                container(center_view)
                    .width(Length::Fill)
                    .height(Length::Fill),
            );
        }
        content.into()
    }

    pub fn view_piano_roll_window(&self) -> Element<'_, UiMessage> {
        // Interactive Piano Roll with Alt-Tools, Event Editor & Polyphonic Ghost Notes
        let tools = [
            PianoRollTool::Draw,
            PianoRollTool::Paint,
            PianoRollTool::Delete,
            PianoRollTool::Mute,
            PianoRollTool::Slice,
            PianoRollTool::Select,
            PianoRollTool::Zoom,
        ];
        let mut tool_row = row![].spacing(2);
        for t in tools {
            let is_sel = self.active_pianoroll_tool == t;
            let btn = button(text(t.label()).size(9))
                .padding(2)
                .style(self.theme.button_style(is_sel))
                .on_press(UiMessage::SetPianoRollTool(t));
            tool_row = tool_row.push(btn);
        }

        let btn_strum = button(text("Strum").size(9)).on_press(UiMessage::PianoRollQuickStrum);
        let btn_flip_h =
            button(text("Flip H").size(9)).on_press(UiMessage::PianoRollFlipHorizontal);
        let btn_flip_v = button(text("Flip V").size(9)).on_press(UiMessage::PianoRollFlipVertical);
        let btn_chop = button(text("Chop 1/16").size(9)).on_press(UiMessage::PianoRollQuickChop);
        let btn_arp_up = button(text("Arp ↑").size(9)).on_press(UiMessage::PianoRollArpeggiate(
            crate::fl_studio::piano_roll::ArpDirection::Up,
        ));
        let btn_arp_down = button(text("Arp ↓").size(9)).on_press(UiMessage::PianoRollArpeggiate(
            crate::fl_studio::piano_roll::ArpDirection::Down,
        ));
        let btn_add_c4 = button(text("+ Note C4").size(9)).on_press(UiMessage::PianoRollAddNote {
            channel_id: 0,
            key: 60,
            start_tick: self.playhead_tick,
            length_ticks: 960,
        });

        let next_scale = match self.piano_roll_scale {
            crate::fl_studio::piano_roll::MusicalScale::Chromatic => {
                crate::fl_studio::piano_roll::MusicalScale::Major
            }
            crate::fl_studio::piano_roll::MusicalScale::Major => {
                crate::fl_studio::piano_roll::MusicalScale::Minor
            }
            crate::fl_studio::piano_roll::MusicalScale::Minor => {
                crate::fl_studio::piano_roll::MusicalScale::Dorian
            }
            crate::fl_studio::piano_roll::MusicalScale::Dorian => {
                crate::fl_studio::piano_roll::MusicalScale::Phrygian
            }
            crate::fl_studio::piano_roll::MusicalScale::Phrygian => {
                crate::fl_studio::piano_roll::MusicalScale::HarmonicMinor
            }
            crate::fl_studio::piano_roll::MusicalScale::HarmonicMinor => {
                crate::fl_studio::piano_roll::MusicalScale::PentatonicMinor
            }
            _ => crate::fl_studio::piano_roll::MusicalScale::Chromatic,
        };

        let btn_scale = button(text(format!("Scale: {}", self.piano_roll_scale.label())).size(9))
            .style(self.theme.button_style(
                self.piano_roll_scale != crate::fl_studio::piano_roll::MusicalScale::Chromatic,
            ))
            .on_press(UiMessage::PianoRollSetScale(next_scale));

        let btn_vel = button(text("VEL").size(9))
            .style(self.theme.button_style(
                self.piano_roll_event_target
                    == crate::fl_studio::piano_roll::EventLaneTarget::Velocity,
            ))
            .on_press(UiMessage::PianoRollSetEventTarget(
                crate::fl_studio::piano_roll::EventLaneTarget::Velocity,
            ));
        let btn_pan = button(text("PAN").size(9))
            .style(self.theme.button_style(
                self.piano_roll_event_target == crate::fl_studio::piano_roll::EventLaneTarget::Pan,
            ))
            .on_press(UiMessage::PianoRollSetEventTarget(
                crate::fl_studio::piano_roll::EventLaneTarget::Pan,
            ));
        let btn_pitch = button(text("PITCH").size(9))
            .style(self.theme.button_style(
                self.piano_roll_event_target
                    == crate::fl_studio::piano_roll::EventLaneTarget::Pitch,
            ))
            .on_press(UiMessage::PianoRollSetEventTarget(
                crate::fl_studio::piano_roll::EventLaneTarget::Pitch,
            ));

        let roll_header = row![
            text("FL Studio Piano Roll").size(11),
            tool_row,
            btn_strum,
            btn_flip_h,
            btn_flip_v,
            btn_chop,
            btn_arp_up,
            btn_arp_down,
            btn_add_c4,
            btn_scale,
            container(row![text("EVENT:").size(9), btn_vel, btn_pan, btn_pitch].spacing(2))
                .padding(2),
            text(format!(
                "Notes: {} | Grid: {}",
                self.piano_roll_notes.len(),
                self.snap_grid.label()
            ))
            .size(10),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center);

        let roll_canvas =
            crate::ui::canvas_piano_roll::view_canvas_piano_roll(self.piano_roll_canvas_program());
        let roll_body = container(roll_canvas)
            .width(Length::Fill)
            .height(Length::Fill);

        let chrome = self.view_window_chrome(
            "Piano Roll",
            "◈",
            UiMessage::SelectFlView(crate::ui::layout::FlActiveView::PianoRoll),
        );

        column![chrome, roll_header, roll_body].spacing(2).into()
    }

    pub fn view_mixer_window(&self) -> Element<'_, UiMessage> {
        let selected_id = self.selected_mixer_track;
        let is_master_selected = selected_id == 9999;
        let active_strip = if is_master_selected {
            &self.mixer.master_strip
        } else {
            self.mixer
                .channels
                .get(selected_id)
                .unwrap_or(&self.mixer.master_strip)
        };

        // 1. Pinned Master Output Strip (Left)
        let m_ch = &self.mixer.master_strip;
        let m_selected = is_master_selected;

        let m_header_btn = button(
            row![
                container(column![])
                    .width(Length::Fixed(3.0))
                    .height(Length::Fixed(12.0))
                    .style(|_t| container::Style {
                        background: Some(iced::Background::Color(Color::from_rgb8(
                            0xe7, 0x4c, 0x3c,
                        ))),
                        ..container::Style::default()
                    }),
                text("MASTER").size(10),
            ]
            .spacing(4),
        )
        .padding(2)
        .style(self.theme.button_style(m_selected))
        .on_press(UiMessage::SelectMixerTrack(9999));

        let m_phase_btn =
            button(text("Ø").size(9))
                .padding(2)
                .style(self.theme.toggle_accent_button_style(
                    m_ch.phase_invert,
                    Color::from_rgb8(0x38, 0xef, 0x7d),
                ))
                .on_press(UiMessage::ToggleTrackPhaseInvert(9999));

        let m_swap_btn =
            button(text("⇄").size(9))
                .padding(2)
                .style(self.theme.toggle_accent_button_style(
                    m_ch.swap_channels,
                    Color::from_rgb8(0x00, 0xf2, 0xfe),
                ))
                .on_press(UiMessage::ToggleTrackSwapChannels(9999));

        let m_clipper_btn = button(text("CLIP").size(9))
            .padding(2)
            .style(self.theme.toggle_accent_button_style(
                self.mixer.master_soft_clipper_enabled,
                Color::from_rgb8(0x38, 0xef, 0x7d),
            ))
            .on_press(UiMessage::ToggleMasterSoftClipper);

        let m_mute_btn = button(
            container(column![])
                .width(Length::Fixed(10.0))
                .height(Length::Fixed(10.0))
                .style(move |_t| container::Style {
                    background: Some(iced::Background::Color(if m_ch.muted {
                        Color::from_rgb8(0x4a, 0x36, 0x28)
                    } else {
                        Color::from_rgb8(0x2e, 0xcc, 0x71)
                    })),
                    border: iced::Border {
                        radius: 5.0.into(),
                        ..Default::default()
                    },
                    ..Default::default()
                }),
        )
        .padding(2)
        .style(self.theme.button_style(false))
        .on_press(UiMessage::ToggleTrackMute(9999));

        let m_fader = vertical_slider(-60.0..=6.0, m_ch.volume_fader_db, |v| {
            UiMessage::SetTrackFader {
                track_id: 9999,
                volume_db: v,
            }
        })
        .step(0.5_f32)
        .width(22.0)
        .height(Length::Fixed(240.0));

        let m_meters = row![
            render_vertical_led_meter(
                m_ch.meter.peak_db,
                m_ch.meter.rms_db,
                m_ch.meter.peak_hold_db,
                m_ch.meter.auto_mute_tripped,
            ),
            render_vertical_led_meter(
                m_ch.meter.rms_db,
                m_ch.meter.rms_db,
                m_ch.meter.peak_hold_db,
                m_ch.meter.auto_mute_tripped,
            ),
        ]
        .spacing(1);

        let m_pan_val = m_ch.pan.balance;
        let m_pan_knob = crate::ui::canvas_knob::rotary_knob(
            22.0,
            -1.0,
            1.0,
            m_pan_val,
            true,
            self.theme.accent,
            move |p| UiMessage::SetTrackPan {
                track_id: 9999,
                pan: p,
            },
        );

        let m_strip = container(
            column![
                m_header_btn,
                row![m_phase_btn, m_swap_btn, m_clipper_btn].spacing(2),
                text("Master Out").size(10),
                m_pan_knob,
                row![m_fader, m_meters]
                    .spacing(4)
                    .align_y(iced::Alignment::Center),
                text(format!("{:.1} dB", m_ch.volume_fader_db)).size(10),
                m_mute_btn,
                text("OUT: DAC 1/2")
                    .size(8)
                    .color(Color::from_rgb8(0x7a, 0x80, 0x8c)),
            ]
            .spacing(3)
            .align_x(iced::Alignment::Center),
        )
        .style(move |_t| self.theme.mixer_strip_style(true, m_selected))
        .padding(4)
        .width(Length::Fixed(96.0));

        // 2. Scrollable Insert Channel Strips (125 Channels)
        let mut channel_strips = row![].spacing(4).padding(4);

        let total_inserts = 125.max(self.mixer.channels.len());
        for idx in 0..total_inserts {
            let is_selected = selected_id == idx;
            let strip_accent = self.theme.track_accent_color(idx);

            let ch = self.mixer.channels.get(idx);
            let ch_name = ch
                .map(|c| c.name.clone())
                .unwrap_or_else(|| format!("Insert {}", idx + 1));
            let vol_db = ch.map(|c| c.volume_fader_db).unwrap_or(0.0);
            let pan_val = ch.map(|c| c.pan.balance).unwrap_or(0.0);
            let is_muted = ch.map(|c| c.muted).unwrap_or(false);
            let is_solo = ch.map(|c| c.solo).unwrap_or(false);
            let is_phase_inv = ch.map(|c| c.phase_invert).unwrap_or(false);
            let is_swap = ch.map(|c| c.swap_channels).unwrap_or(false);
            let peak_db = ch.map(|c| c.meter.peak_db).unwrap_or(-60.0);
            let rms_db = ch.map(|c| c.meter.rms_db).unwrap_or(-60.0);
            let peak_hold_db = ch.map(|c| c.meter.peak_hold_db).unwrap_or(-60.0);
            let auto_mute_tripped = ch.map(|c| c.meter.auto_mute_tripped).unwrap_or(false);

            let header_btn = button(
                row![
                    container(column![])
                        .width(Length::Fixed(3.0))
                        .height(Length::Fixed(12.0))
                        .style(move |_t| container::Style {
                            background: Some(iced::Background::Color(strip_accent)),
                            ..container::Style::default()
                        }),
                    text(format!("{:02}", idx + 1)).size(9),
                ]
                .spacing(3),
            )
            .padding(2)
            .style(self.theme.button_style(is_selected))
            .on_press(UiMessage::SelectMixerTrack(idx));

            let phase_btn =
                button(text("Ø").size(9))
                    .padding(2)
                    .style(self.theme.toggle_accent_button_style(
                        is_phase_inv,
                        Color::from_rgb8(0x38, 0xef, 0x7d),
                    ))
                    .on_press(UiMessage::ToggleTrackPhaseInvert(idx));

            let swap_btn = button(text("⇄").size(9))
                .padding(2)
                .style(
                    self.theme
                        .toggle_accent_button_style(is_swap, Color::from_rgb8(0x00, 0xf2, 0xfe)),
                )
                .on_press(UiMessage::ToggleTrackSwapChannels(idx));

            let mute_btn = button(
                container(column![])
                    .width(Length::Fixed(8.0))
                    .height(Length::Fixed(8.0))
                    .style(move |_t| container::Style {
                        background: Some(iced::Background::Color(if is_muted {
                            Color::from_rgb8(0x4a, 0x36, 0x28)
                        } else {
                            Color::from_rgb8(0x2e, 0xcc, 0x71)
                        })),
                        border: iced::Border {
                            radius: 4.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
            )
            .padding(2)
            .style(self.theme.button_style(false))
            .on_press(UiMessage::ToggleTrackMute(idx));

            let solo_btn = button(
                container(column![])
                    .width(Length::Fixed(8.0))
                    .height(Length::Fixed(8.0))
                    .style(move |_t| container::Style {
                        background: Some(iced::Background::Color(if is_solo {
                            Color::from_rgb8(0xf3, 0x9c, 0x12)
                        } else {
                            Color::from_rgb8(0x35, 0x38, 0x40)
                        })),
                        border: iced::Border {
                            radius: 4.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
            )
            .padding(2)
            .style(self.theme.button_style(false))
            .on_press(UiMessage::ToggleTrackSolo(idx));

            let fader_slider =
                vertical_slider(-60.0..=6.0, vol_db, move |vol| UiMessage::SetTrackFader {
                    track_id: idx,
                    volume_db: vol,
                })
                .step(0.5_f32)
                .width(20.0)
                .height(Length::Fixed(240.0));

            let pan_knob = crate::ui::canvas_knob::rotary_knob(
                22.0,
                -1.0,
                1.0,
                pan_val,
                true,
                self.theme.accent,
                move |p| UiMessage::SetTrackPan {
                    track_id: idx,
                    pan: p,
                },
            );

            let pan_txt = if pan_val.abs() < 0.05 {
                "C".to_string()
            } else if pan_val < 0.0 {
                format!("L{:.0}%", (-pan_val) * 100.0)
            } else {
                format!("R{:.0}%", pan_val * 100.0)
            };

            let meter_bars = row![
                render_vertical_led_meter(peak_db, rms_db, peak_hold_db, auto_mute_tripped),
                render_vertical_led_meter(rms_db, rms_db, peak_hold_db, auto_mute_tripped),
            ]
            .spacing(1);

            let is_sending_master = self
                .mixer_sends
                .get(idx)
                .and_then(|s| s.first())
                .copied()
                .unwrap_or(1.0)
                > 0.0;
            let send_btn =
                button(
                    text(if is_sending_master {
                        "▲ MST"
                    } else {
                        "△ MST"
                    })
                    .size(8),
                )
                .padding(2)
                .style(self.theme.toggle_accent_button_style(
                    is_sending_master,
                    Color::from_rgb8(0xff, 0x85, 0x00),
                ))
                .on_press(UiMessage::MixerToggleSend {
                    from_track: idx,
                    to_track: 0,
                });

            let strip = container(
                column![
                    header_btn,
                    row![phase_btn, swap_btn].spacing(2),
                    container(text(ch_name).size(10)).height(Length::Fixed(14.0)),
                    row![text("Pan:").size(8), text(pan_txt).size(8)].spacing(2),
                    pan_knob,
                    row![fader_slider, meter_bars]
                        .spacing(3)
                        .align_y(iced::Alignment::Center),
                    text(format!("{:.1} dB", vol_db)).size(9),
                    row![mute_btn, solo_btn, send_btn].spacing(2),
                    text(if is_sending_master {
                        "→ Master"
                    } else {
                        "→ Unrouted"
                    })
                    .size(8)
                    .color(Color::from_rgb8(0x7a, 0x80, 0x8c)),
                ]
                .spacing(2)
                .align_x(iced::Alignment::Center),
            )
            .style(move |_t| self.theme.mixer_strip_style(false, is_selected))
            .padding(4)
            .width(Length::Fixed(82.0));

            channel_strips = channel_strips.push(strip);
        }

        // 3. Dedicated 10-Slot Insert FX Rack Inspector
        let target_track_id = if is_master_selected {
            9999
        } else {
            selected_id
        };
        let target_title = if is_master_selected {
            "MASTER".to_string()
        } else {
            format!("CH {:02}: {}", selected_id + 1, active_strip.name)
        };

        let mut slots_col = column![].spacing(3);
        for (s_idx, slot) in active_strip.slots.iter().enumerate() {
            let is_active = !slot.bypassed;
            let led_color = if is_active {
                Color::from_rgb8(0x38, 0xef, 0x7d) // Neon Green active LED
            } else {
                Color::from_rgb8(0x55, 0x55, 0x55) // Dim bypassed LED
            };

            let led_btn = button(text(if is_active { "●" } else { "○" }).size(11))
                .padding(1)
                .style(move |_t, _status| button::Style {
                    background: None,
                    text_color: led_color,
                    ..button::Style::default()
                })
                .on_press(UiMessage::ToggleInsertSlotBypass {
                    track_id: target_track_id,
                    slot_id: s_idx,
                });

            let slot_label = text(format!("{}. {}", s_idx + 1, slot.name)).size(10);
            let mix_slider = slider(0.0..=1.0, slot.mix, move |m| UiMessage::SetInsertSlotMix {
                track_id: target_track_id,
                slot_id: s_idx,
                mix: m,
            })
            .step(0.01_f32)
            .width(Length::Fixed(55.0));

            let mix_pct = text(format!("{:.0}%", slot.mix * 100.0)).size(8);

            let slot_row = container(
                row![
                    led_btn,
                    container(slot_label).width(Length::Fixed(100.0)),
                    mix_slider,
                    mix_pct,
                ]
                .spacing(3)
                .align_y(iced::Alignment::Center),
            )
            .padding(2)
            .style(|_t| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb8(0x18, 0x1a, 0x22))),
                border: iced::Border {
                    color: Color::from_rgb8(0x28, 0x2c, 0x37),
                    width: 1.0,
                    radius: 3.0.into(),
                },
                text_color: Some(Color::from_rgb8(0xd0, 0xd4, 0xdc)),
                ..container::Style::default()
            });

            slots_col = slots_col.push(slot_row);
        }

        // Stereo separation readout
        let sep_val = active_strip.stereo_separation;
        let sep_txt = if sep_val < -0.05 {
            format!("Sep: {:.0}% Wide", (-sep_val) * 100.0)
        } else if sep_val > 0.05 {
            format!("Mono: {:.0}% Merged", sep_val * 100.0)
        } else {
            "100% Normal Stereo".to_string()
        };

        let sep_slider = slider(-1.0..=1.0, sep_val, move |s| {
            UiMessage::SetTrackStereoSeparation {
                track_id: target_track_id,
                separation: s,
            }
        })
        .step(0.05_f32)
        .width(Length::Fixed(110.0));

        let insp_phase_btn = button(
            text(if active_strip.phase_invert {
                "Ø [INV]"
            } else {
                "Ø [NORM]"
            })
            .size(9),
        )
        .padding(3)
        .style(self.theme.toggle_accent_button_style(
            active_strip.phase_invert,
            Color::from_rgb8(0x38, 0xef, 0x7d),
        ))
        .on_press(UiMessage::ToggleTrackPhaseInvert(target_track_id));

        let insp_swap_btn = button(
            text(if active_strip.swap_channels {
                "⇄ [SWAP]"
            } else {
                "⇄ [NORM]"
            })
            .size(9),
        )
        .padding(3)
        .style(self.theme.toggle_accent_button_style(
            active_strip.swap_channels,
            Color::from_rgb8(0x00, 0xf2, 0xfe),
        ))
        .on_press(UiMessage::ToggleTrackSwapChannels(target_track_id));

        let mut inspector_footer = column![
            row![
                text("STEREO SEPARATION").size(9),
                sep_slider,
                text(sep_txt).size(9)
            ]
            .spacing(4),
            row![insp_phase_btn, insp_swap_btn].spacing(4),
        ]
        .spacing(3);

        if is_master_selected {
            let clipper_btn = button(
                text(if self.mixer.master_soft_clipper_enabled {
                    "SOFT CLIPPER (-0.1 dBFS): [ACTIVE]"
                } else {
                    "SOFT CLIPPER: [BYPASSED]"
                })
                .size(9),
            )
            .padding(3)
            .style(self.theme.toggle_accent_button_style(
                self.mixer.master_soft_clipper_enabled,
                Color::from_rgb8(0x38, 0xef, 0x7d),
            ))
            .on_press(UiMessage::ToggleMasterSoftClipper);

            inspector_footer = inspector_footer.push(clipper_btn);
        }

        let fx_inspector = container(
            column![
                container(
                    row![
                        text("INSERT FX RACK").size(10),
                        Space::new().width(Length::Fill),
                        text(target_title).size(10),
                    ]
                    .spacing(4),
                )
                .padding(3)
                .style(|_t| self.theme.machined_capsule_style()),
                slots_col,
                inspector_footer,
            ]
            .spacing(4),
        )
        .style(|_t| container::Style {
            background: Some(iced::Background::Color(Color::from_rgb8(0x11, 0x12, 0x16))),
            border: iced::Border {
                color: Color::from_rgb8(0x2d, 0x31, 0x3d),
                width: 1.0,
                radius: 4.0.into(),
            },
            text_color: Some(Color::from_rgb8(0xec, 0xf0, 0xf1)),
            ..container::Style::default()
        })
        .padding(4)
        .width(Length::Fixed(240.0));

        let content = row![
            m_strip,
            scrollable(channel_strips)
                .width(Length::Fill)
                .height(Length::Fill)
                .direction(scrollable::Direction::Horizontal(
                    scrollable::Scrollbar::default(),
                )),
            fx_inspector,
        ]
        .spacing(6)
        .height(Length::Fill);

        let chrome = self.view_window_chrome(
            "Mixer — Master & 125 Inserts",
            "◈",
            UiMessage::SelectFlView(crate::ui::layout::FlActiveView::Mixer),
        );

        container(
            column![
                chrome,
                row![
                    text("FL Studio 24 Studio Mixer Console (125 Inserts + Master Strip)").size(11),
                    Space::new().width(Length::Fill),
                    text(format!(
                        "Target: [{}]",
                        if is_master_selected {
                            "Master".to_string()
                        } else {
                            format!("CH {:02}", selected_id + 1)
                        }
                    ))
                    .size(10)
                    .color(Color::from_rgb8(0xff, 0x85, 0x00)),
                ]
                .spacing(6),
                content,
            ]
            .spacing(4),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(4)
        .style(|_t| self.theme.panel_container_style())
        .into()
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
            crate::ui::layout::DrawerTab::StepSequencer => self.view_channel_rack(false),
            crate::ui::layout::DrawerTab::PianoRoll => self.view_piano_roll_window(),
            crate::ui::layout::DrawerTab::Mixer => self.view_mixer_window(),
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
        let (xruns, host_name, buffer_str) = if let Some(drv) = &self.audio_driver {
            let xruns = drv
                .underrun_count
                .load(std::sync::atomic::Ordering::Relaxed);
            let host = if drv.host_name.is_empty() {
                "ALSA / PIPEWIRE".to_string()
            } else {
                drv.host_name.to_uppercase()
            };
            let buffer_ms = (128.0 / (drv.sample_rate as f32).max(1.0)) * 1000.0;
            (
                xruns,
                host,
                format!("BUFFER: 128 spl ({:.1} ms)", buffer_ms),
            )
        } else {
            (
                0,
                "ALSA / PIPEWIRE".to_string(),
                format!(
                    "BUFFER: 128 spl ({:.1} ms)",
                    (128.0 / self.mixer.sample_rate.max(1.0)) * 1000.0
                ),
            )
        };

        let master_peak_str = if self.mixer.master_strip.meter.auto_mute_tripped {
            "PEAK: AUTO-MUTE TRIPPED (+12 dBFS)".to_string()
        } else if self.mixer.master_strip.meter.peak_db > 0.0 {
            format!(
                "PEAK: +{:.1} dBFS (CLIP)",
                self.mixer.master_strip.meter.peak_db
            )
        } else if self.mixer.master_strip.meter.peak_db > -90.0 {
            format!("PEAK: {:.1} dBFS", self.mixer.master_strip.meter.peak_db)
        } else {
            "PEAK: -inf dBFS".to_string()
        };

        let left_telemetry = row![
            text("● ENGINE: ACTIVE").size(9),
            text("|").size(9),
            text(format!("RATE: {:.1} kHz", self.mixer.sample_rate / 1000.0)).size(9),
            text("|").size(9),
            text(buffer_str).size(9),
            text("|").size(9),
            text(format!("XRUNS: {}", xruns)).size(9),
        ]
        .spacing(6);

        let center_tool = row![
            text("TOOL: [ SMART SELECT / TRIM / FADE ]").size(9),
            text("|").size(9),
            text(format!("GRID: [ {} ]", self.snap_grid.label())).size(9),
        ]
        .spacing(6);

        let right_env = row![
            text(format!("BACKEND: {}", host_name)).size(9),
            text("|").size(9),
            text("HOST: LINUX (COSMIC)").size(9),
            text("|").size(9),
            text(master_peak_str).size(9),
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

    fn view_plugin_picker(&self) -> Element<'_, UiMessage> {
        let title_bar = row![
            text("VISUAL PLUGIN PICKER [F8]").size(13),
            Space::new().width(Length::Fill),
            button(text("✕").size(11))
                .padding(3)
                .on_press(UiMessage::ToggleFlPluginPicker),
        ]
        .align_y(iced::Alignment::Center);

        let search_box = text_input(
            "Search plugins by name, vendor, or category...",
            &self.plugin_picker_search,
        )
        .on_input(UiMessage::PluginPickerSetSearch)
        .padding(6)
        .size(11);

        let categories = [
            (None, "All"),
            (
                Some(crate::plugins::plugin_descriptor::PluginCategory::Synth),
                "Synths",
            ),
            (
                Some(crate::plugins::plugin_descriptor::PluginCategory::Sampler),
                "Samplers",
            ),
            (
                Some(crate::plugins::plugin_descriptor::PluginCategory::Dynamics),
                "Dynamics",
            ),
            (
                Some(crate::plugins::plugin_descriptor::PluginCategory::Equalizer),
                "EQ",
            ),
            (
                Some(crate::plugins::plugin_descriptor::PluginCategory::Reverb),
                "Reverbs",
            ),
            (
                Some(crate::plugins::plugin_descriptor::PluginCategory::Delay),
                "Delays",
            ),
            (
                Some(crate::plugins::plugin_descriptor::PluginCategory::Utility),
                "Utility",
            ),
        ];

        let mut cat_row = row![].spacing(4);
        for (cat_val, cat_label) in categories {
            let is_sel = self.plugin_picker_category == cat_val;
            let cat_btn = button(text(cat_label).size(10))
                .padding(4)
                .style(self.theme.button_style(is_sel))
                .on_press(UiMessage::PluginPickerSelectCategory(cat_val));
            cat_row = cat_row.push(cat_btn);
        }

        let filtered = self
            .browser
            .filtered_plugins(&self.plugin_picker_search, self.plugin_picker_category);

        let mut cards_col = column![].spacing(6).width(Length::Fill);

        for plugin in filtered {
            let pid = plugin.id;
            let is_gen = plugin.is_generator;

            let cat_badge_color = if is_gen {
                Color::from_rgb8(0xff, 0x88, 0x00) // Neon orange for generators/synths
            } else {
                Color::from_rgb8(0x00, 0xa8, 0xff) // Neon cyan for effects
            };

            let type_text = if is_gen { "GENERATOR" } else { "EFFECT" };

            let target_hint = if let Some((trk, slot)) = self.plugin_picker_target_slot {
                if !is_gen {
                    format!("Assign to Track {} Slot {}", trk, slot + 1)
                } else {
                    "Load as Generator Channel".into()
                }
            } else if is_gen {
                "Add to Channel Rack".into()
            } else {
                "Add to Selected Insert".into()
            };

            let card = container(
                row![
                    container(
                        column![
                            text(type_text).size(8).color(cat_badge_color),
                            text(format!("{:?}", plugin.category))
                                .size(8)
                                .color(Color::from_rgb8(0x8a, 0x90, 0x9a)),
                        ]
                        .spacing(2),
                    )
                    .width(Length::Fixed(80.0))
                    .padding(4)
                    .style(|_t| self.theme.inner_well_style()),
                    column![
                        row![
                            text(&plugin.name).size(12),
                            Space::new().width(Length::Fixed(6.0)),
                            text(&plugin.vendor)
                                .size(9)
                                .color(Color::from_rgb8(0x70, 0x75, 0x80)),
                        ]
                        .align_y(iced::Alignment::Center),
                        text(&plugin.description)
                            .size(9)
                            .color(Color::from_rgb8(0xa0, 0xa5, 0xb0)),
                    ]
                    .spacing(2)
                    .width(Length::Fill),
                    column![
                        button(text("+ Load").size(10))
                            .padding(4)
                            .style(self.theme.accent_button_style(cat_badge_color))
                            .on_press(UiMessage::PluginPickerLoadPlugin {
                                plugin_id: pid,
                                is_generator: is_gen,
                            }),
                        text(target_hint)
                            .size(7)
                            .color(Color::from_rgb8(0x70, 0x75, 0x80)),
                    ]
                    .spacing(2)
                    .align_x(iced::Alignment::End),
                ]
                .spacing(8)
                .align_y(iced::Alignment::Center),
            )
            .padding(6)
            .style(|_t| self.theme.outer_bezel_style());

            cards_col = cards_col.push(card);
        }

        container(
            column![
                title_bar,
                search_box,
                cat_row,
                scrollable(cards_col).height(Length::Fixed(240.0)),
            ]
            .spacing(8),
        )
        .style(|_t| self.theme.oled_vfd_display_style())
        .width(Length::Fixed(680.0))
        .padding(10)
        .into()
    }

    fn view_audio_settings_modal(&self) -> Element<'_, UiMessage> {
        let title_bar = row![
            text("AUDIO HARDWARE & ENGINE SETTINGS [F10]").size(13),
            Space::new().width(Length::Fill),
            button(text("✕").size(11))
                .padding(3)
                .on_press(UiMessage::ToggleAudioSettings),
        ]
        .align_y(iced::Alignment::Center);

        // 1. Audio Backend
        let backends = [
            (
                crate::ui::message::AudioBackend::PipeWire,
                "PipeWire (Pro Audio)",
            ),
            (
                crate::ui::message::AudioBackend::Alsa,
                "ALSA (Direct Kernel)",
            ),
            (crate::ui::message::AudioBackend::PulseAudio, "PulseAudio"),
            (
                crate::ui::message::AudioBackend::Jack,
                "JACK Audio Connection Kit",
            ),
        ];
        let mut backend_row = row![].spacing(6);
        for (b_val, b_label) in backends {
            let is_sel = self.audio_backend == b_val;
            let b_btn = button(text(b_label).size(10))
                .padding(5)
                .style(self.theme.button_style(is_sel))
                .on_press(UiMessage::SetAudioBackend(b_val));
            backend_row = backend_row.push(b_btn);
        }

        // 2. Buffer Size & Latency Math: \tau = (buffer / f_s) * 1000 ms
        let buffer_sizes = [64, 128, 256, 512, 1024, 2048];
        let mut buf_row = row![].spacing(4);
        for size in buffer_sizes {
            let is_sel = self.audio_buffer_size == size;
            let btn = button(text(format!("{}", size)).size(10))
                .padding(4)
                .style(self.theme.button_style(is_sel))
                .on_press(UiMessage::SetAudioBufferSize(size));
            buf_row = buf_row.push(btn);
        }

        let latency_ms = (self.audio_buffer_size as f32
            / self.audio_sample_rate_selection.max(1) as f32)
            * 1000.0;
        let latency_str = format!(
            "Computed Roundtrip Buffer Latency: {:.2} ms ({} frames @ {} Hz)",
            latency_ms, self.audio_buffer_size, self.audio_sample_rate_selection
        );

        // 3. Sample Rate
        let sample_rates = [44100, 48000, 96000];
        let mut sr_row = row![].spacing(6);
        for sr in sample_rates {
            let is_sel = self.audio_sample_rate_selection == sr;
            let btn = button(text(format!("{} Hz", sr)).size(10))
                .padding(4)
                .style(self.theme.button_style(is_sel))
                .on_press(UiMessage::SetAudioSampleRate(sr));
            sr_row = sr_row.push(btn);
        }

        // 4. Hardware Telemetry & Real-Time Underrun Protection
        let cpu_pct = if let Some(ref worker) = self.audio_worker {
            worker
                .state
                .cpu_load_percent
                .load(std::sync::atomic::Ordering::Relaxed) as f32
                / 10.0
        } else {
            0.4
        };

        let worker_xruns = self
            .audio_worker
            .as_ref()
            .map(|w| w.state.underruns.load(std::sync::atomic::Ordering::Relaxed))
            .unwrap_or(0);
        let driver_xruns = self
            .audio_driver
            .as_ref()
            .map(|d| d.underrun_count.load(std::sync::atomic::Ordering::Relaxed))
            .unwrap_or(0);
        let total_xruns = worker_xruns + driver_xruns;

        let xruns_color = if total_xruns > 0 {
            Color::from_rgb8(0xe7, 0x4c, 0x3c) // Warning red
        } else {
            Color::from_rgb8(0x2e, 0xcc, 0x71) // Safe green
        };

        let telemetry_panel = container(
            column![
                row![
                    text("Real-Time DSP Worker CPU Load:").size(10),
                    text(format!("{:.1}%", cpu_pct))
                        .size(10)
                        .color(if cpu_pct > 75.0 {
                            Color::from_rgb8(0xe7, 0x4c, 0x3c)
                        } else {
                            Color::from_rgb8(0x5a, 0xe0, 0xff)
                        }),
                    Space::new().width(Length::Fixed(16.0)),
                    text("Buffer Underruns (XRUNs):").size(10),
                    text(format!("{}", total_xruns)).size(10).color(xruns_color),
                    button(text("Reset Counter").size(9))
                        .padding(2)
                        .on_press(UiMessage::ResetAudioXruns),
                ]
                .spacing(6)
                .align_y(iced::Alignment::Center),
                row![
                    text("Audio Engine Architecture:")
                        .size(9)
                        .color(Color::from_rgb8(0x8a, 0x90, 0x9a)),
                    text("FTZ/DAZ Enabled (Denormals-Are-Zero: ACTIVE)")
                        .size(9)
                        .color(Color::from_rgb8(0x2e, 0xcc, 0x71)),
                    Space::new().width(Length::Fixed(12.0)),
                    text("Priority: Real-Time FIFO / Cosmic Wayland")
                        .size(9)
                        .color(Color::from_rgb8(0x8a, 0x90, 0x9a)),
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center),
            ]
            .spacing(6),
        )
        .padding(8)
        .style(|_t| self.theme.inner_well_style());

        container(
            column![
                title_bar,
                text("Audio Output Driver Backend:").size(11),
                backend_row,
                text("Hardware Buffer Size (Samples):").size(11),
                buf_row,
                text(latency_str)
                    .size(10)
                    .color(Color::from_rgb8(0x5a, 0xe0, 0xff)),
                text("Master Sample Rate:").size(11),
                sr_row,
                text("Engine Telemetry & Hardware Status:").size(11),
                telemetry_panel,
            ]
            .spacing(8),
        )
        .style(|_t| self.theme.oled_vfd_display_style())
        .width(Length::Fixed(640.0))
        .padding(12)
        .into()
    }

    /// Dedicated Edison Sample Editor & Waveform Manipulation Window
    pub fn view_edison_window(&self) -> Element<'_, UiMessage> {
        let ed = &self.edison;

        // 1. Authentic FL Studio Edison Transport & Recording Strip (Top)
        let transport_strip = row![
            button(text("🔗 SLAVE").size(9))
                .padding([2, 5])
                .style(self.theme.button_style(true)),
            button(text("∞ LOOP").size(9))
                .padding([2, 5])
                .style(self.theme.button_style(false)),
            button(text("▶").size(10)).padding([2, 6]).style(
                self.theme
                    .accent_button_style(Color::from_rgb8(0x2e, 0xcc, 0x71))
            ),
            button(text("■").size(10))
                .padding([2, 6])
                .style(self.theme.button_style(false)),
            button(text("● REC").size(9))
                .padding([2, 6])
                .style(self.theme.accent_button_style(self.theme.record)),
            button(text("ON INPUT ▾").size(9))
                .padding([2, 5])
                .style(self.theme.button_style(false)),
            button(text("FOR 5' ▾").size(9))
                .padding([2, 5])
                .style(self.theme.button_style(false)),
            Space::new().width(Length::Fill),
            button(text("Export WAV").size(10))
                .style(self.theme.accent_button_style(self.theme.accent))
                .padding([2, 6])
                .on_press(UiMessage::MenuAction(MenuCommand::BounceWav)),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center);

        // 2. Sample Properties Bar (Sample Rate, Bit Depth, Tempo Sync, Title & Timecode)
        let total_samples = ed.len();
        let total_seconds = ed.duration_seconds();
        let mins = (total_seconds / 60.0) as u32;
        let secs = (total_seconds % 60.0) as u32;
        let millis = ((total_seconds * 1000.0) % 1000.0) as u32;
        let timecode_str = format!("{:02}:{:02}.{:03}", mins, secs, millis);

        let properties_bar = container(
            row![
                text(format!("SAMPLERATE: {:.0} Hz", ed.sample_rate))
                    .size(9)
                    .color(Color::from_rgb8(0x5a, 0xe0, 0xff)),
                text("FORMAT: 24-bit PCM")
                    .size(9)
                    .color(Color::from_rgb8(0xc0, 0xc6, 0xd4)),
                text("TEMPO: 120.000 BPM")
                    .size(9)
                    .color(Color::from_rgb8(0xff, 0xb8, 0x5a)),
                container(text("SYNCED").size(8).color(Color::WHITE))
                    .padding([1, 4])
                    .style(|_t| container::Style {
                        background: Some(iced::Background::Color(Color::from_rgb8(
                            0x2e, 0xcc, 0x71
                        ))),
                        border: iced::Border {
                            radius: 2.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
                text("TITLE: Sample_Audio_Master.wav")
                    .size(9)
                    .color(self.theme.text_secondary),
                Space::new().width(Length::Fill),
                text(format!("LENGTH: {} spl | {}", total_samples, timecode_str))
                    .size(9)
                    .color(Color::from_rgb8(0x5a, 0xe0, 0xff)),
            ]
            .spacing(8)
            .align_y(iced::Alignment::Center),
        )
        .height(Length::Fixed(22.0))
        .padding([2, 6])
        .style(|_t| self.theme.inner_well_style());

        // 3. Audio Processing Actions Row
        let actions_row = row![
            button(text("Normalize").size(10))
                .padding(3)
                .on_press(UiMessage::EdisonNormalize),
            button(text("Reverse").size(10))
                .padding(3)
                .on_press(UiMessage::EdisonReverse),
            button(text("Fade In").size(10))
                .padding(3)
                .on_press(UiMessage::EdisonFadeIn),
            button(text("Fade Out").size(10))
                .padding(3)
                .on_press(UiMessage::EdisonFadeOut),
            button(text("Cut").size(10))
                .padding(3)
                .on_press(UiMessage::EdisonCut),
            button(text("Copy").size(10))
                .padding(3)
                .on_press(UiMessage::EdisonCopy),
            button(text("Paste").size(10))
                .padding(3)
                .on_press(UiMessage::EdisonPaste),
            button(text("Select All").size(10))
                .padding(3)
                .on_press(UiMessage::EdisonSelectAll),
        ]
        .spacing(4)
        .align_y(iced::Alignment::Center);

        // 4. Waveform & Spectral Visualization
        let (sel_start, sel_end) = ed.active_range();
        let waveform_canvas = crate::ui::canvas_edison::view_canvas_edison(
            &ed.samples_l,
            &ed.samples_r,
            ed.sample_rate,
            ed.selection_start,
            ed.selection_end,
            ed.cursor_pos,
            &ed.markers,
        );

        let waveform_container = container(waveform_canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(4)
            .style(|_t| self.theme.inner_well_style());

        let telemetry_row = row![
            text(format!("Markers: {} points", ed.markers.len()))
                .size(9)
                .color(Color::from_rgb8(0x8a, 0x90, 0x9c)),
            Space::new().width(Length::Fill),
            text(format!(
                "Selection: {} .. {} ({} samples)",
                sel_start,
                sel_end,
                sel_end.saturating_sub(sel_start)
            ))
            .size(9),
        ]
        .spacing(6);

        let chrome = self.view_window_chrome(
            "Edison Waveform Editor",
            "◈",
            UiMessage::SelectFlView(crate::ui::layout::FlActiveView::Edison),
        );

        container(
            column![
                chrome,
                transport_strip,
                properties_bar,
                actions_row,
                waveform_container,
                telemetry_row,
            ]
            .spacing(4),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(4)
        .style(|_t| self.theme.panel_container_style())
        .into()
    }

    /// Dedicated FPC (Fruity Pad Controller) 16-Pad MPC Drum Sampler Window
    pub fn view_fpc_window(&self) -> Element<'_, UiMessage> {
        let fpc = &self.fpc;
        let selected_pad_id = self.selected_fpc_pad.min(fpc.pads.len().saturating_sub(1));
        let selected_pad = fpc.pads.get(selected_pad_id);

        let header = row![
            container(
                text("FPC — 16-PAD DRUM SAMPLER")
                    .size(11)
                    .color(Color::from_rgb8(0xff, 0x88, 0x00)),
            )
            .padding(3),
            Space::new().width(Length::Fixed(12.0)),
            text(format!("Kit: {}", fpc.active_kit_name)).size(10),
            Space::new().width(Length::Fill),
            row![
                text("Master Vol:").size(9),
                slider(0.0..=1.25, fpc.master_volume, UiMessage::FpcSetVolume)
                    .step(0.02_f32)
                    .width(Length::Fixed(80.0)),
                text(format!("{:.0}%", fpc.master_volume * 100.0)).size(9),
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center);

        // 1. Left Section: 4x4 MPC Drum Pad Matrix
        let mut pad_grid_col = column![].spacing(6);
        for row_idx in (0..4).rev() {
            let mut pad_row = row![].spacing(6);
            for col_idx in 0..4 {
                let pad_id = row_idx * 4 + col_idx;
                if let Some(pad) = fpc.pads.get(pad_id) {
                    let p_col =
                        Color::from_rgb8(pad.color_rgb[0], pad.color_rgb[1], pad.color_rgb[2]);
                    let is_trig = pad.is_triggered;
                    let is_selected = pad_id == selected_pad_id;

                    let base_style = self.theme.fl_pad_style(p_col, is_trig);
                    let pad_glyph = match pad_id % 4 {
                        0 => "⬢", // Kick/Low
                        1 => "◎", // Snare/Perc
                        2 => "⌒", // Hat
                        _ => "◇", // Cymbal/Clap
                    };
                    let pad_card = button(
                        column![
                            row![
                                text(format!("{:02}", pad.id + 1))
                                    .size(8)
                                    .color(Color::from_rgba(1.0, 1.0, 1.0, 0.7)),
                                text(pad_glyph)
                                    .size(8)
                                    .color(Color::from_rgba(1.0, 1.0, 1.0, 0.6)),
                                Space::new().width(Length::Fill),
                                text(format!("MIDI {}", pad.note_number))
                                    .size(8)
                                    .color(Color::from_rgba(1.0, 1.0, 1.0, 0.5)),
                            ]
                            .spacing(2)
                            .align_y(iced::Alignment::Center),
                            Space::new().height(Length::Fill),
                            text(&pad.name).size(10),
                            text(&pad.sample_name)
                                .size(8)
                                .color(Color::from_rgba(1.0, 1.0, 1.0, 0.8)),
                        ]
                        .padding(4),
                    )
                    .width(Length::Fixed(130.0))
                    .height(Length::Fixed(80.0))
                    .style(move |theme, status| {
                        let mut st = base_style(theme, status);
                        if is_selected {
                            st.border.color = Color::from_rgb8(0x00, 0xf2, 0xfe);
                            st.border.width = 2.0;
                        }
                        st
                    })
                    .on_press(UiMessage::FpcTriggerPad(pad_id));

                    pad_row = pad_row.push(pad_card);
                }
            }
            pad_grid_col = pad_grid_col.push(pad_row);
        }

        // Bank A / Bank B selector bar (bottom of pad matrix)
        let bank_selector = row![
            button(
                row![
                    container(column![])
                        .width(Length::Fixed(6.0))
                        .height(Length::Fixed(6.0))
                        .style(|_t| container::Style {
                            background: Some(iced::Background::Color(Color::from_rgb8(
                                0xff, 0x88, 0x00
                            ))),
                            border: iced::Border {
                                radius: 3.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    text("BANK A (Pads 1-16)").size(9),
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center),
            )
            .padding([2, 8])
            .style(self.theme.button_style(true)),
            Space::new().width(Length::Fill),
            button(
                row![
                    container(column![])
                        .width(Length::Fixed(6.0))
                        .height(Length::Fixed(6.0))
                        .style(|_t| container::Style {
                            background: Some(iced::Background::Color(Color::from_rgb8(
                                0x55, 0x5a, 0x66
                            ))),
                            border: iced::Border {
                                radius: 3.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    text("BANK B (Pads 17-32)").size(9),
                ]
                .spacing(4)
                .align_y(iced::Alignment::Center),
            )
            .padding([2, 8])
            .style(self.theme.button_style(false)),
        ]
        .align_y(iced::Alignment::Center);

        let pad_matrix_container = container(column![pad_grid_col, bank_selector].spacing(6))
            .padding(6)
            .style(|_t| self.theme.inner_well_style());

        // 2. Right Section: Selected Pad Inspector & Sample Layer Controls
        let pad_inspector: Element<'_, UiMessage> = if let Some(pad) = selected_pad {
            let p_col = Color::from_rgb8(pad.color_rgb[0], pad.color_rgb[1], pad.color_rgb[2]);

            let pad_header = container(
                row![
                    container(column![])
                        .width(Length::Fixed(4.0))
                        .height(Length::Fixed(16.0))
                        .style(move |_t| container::Style {
                            background: Some(iced::Background::Color(p_col)),
                            ..container::Style::default()
                        }),
                    text(format!(
                        "PAD {:02}: {} [MIDI {}]",
                        pad.id + 1,
                        pad.name,
                        pad.note_number
                    ))
                    .size(11),
                    Space::new().width(Length::Fill),
                    button(text("▶ PREVIEW").size(9))
                        .padding(3)
                        .style(self.theme.accent_button_style(self.theme.accent))
                        .on_press(UiMessage::FpcTriggerPad(pad.id)),
                ]
                .spacing(6)
                .align_y(iced::Alignment::Center),
            )
            .padding(4)
            .style(|_t| self.theme.machined_capsule_style());

            let vol_slider = slider(0.0..=1.25, pad.volume, move |v| {
                UiMessage::FpcSetPadVolume {
                    pad_id: pad.id,
                    volume: v,
                }
            })
            .step(0.01_f32)
            .width(Length::Fixed(120.0));

            let pan_slider = slider(-1.0..=1.0, pad.pan, move |p| UiMessage::FpcSetPadPan {
                pad_id: pad.id,
                pan: p,
            })
            .step(0.05_f32)
            .width(Length::Fixed(120.0));

            let pan_txt = if pad.pan.abs() < 0.05 {
                "Center".to_string()
            } else if pad.pan < 0.0 {
                format!("Left {:.0}%", (-pad.pan) * 100.0)
            } else {
                format!("Right {:.0}%", pad.pan * 100.0)
            };

            let mute_btn = button(text("MUTE").size(9))
                .padding(4)
                .style(
                    self.theme
                        .toggle_accent_button_style(pad.muted, self.theme.mute),
                )
                .on_press(UiMessage::FpcTogglePadMute(pad.id));

            let solo_btn = button(text("SOLO").size(9))
                .padding(4)
                .style(
                    self.theme
                        .toggle_accent_button_style(pad.solo, self.theme.solo),
                )
                .on_press(UiMessage::FpcTogglePadSolo(pad.id));

            let sample_info = container(
                column![
                    text("SAMPLE LAYER DETAILS")
                        .size(9)
                        .color(Color::from_rgb8(0x9a, 0xa0, 0xaa)),
                    text(format!("File: {}", pad.sample_name)).size(10),
                    text("Format: 24-bit 44.1 kHz PCM WAV")
                        .size(9)
                        .color(Color::from_rgb8(0x7a, 0x80, 0x8c)),
                    text(format!("Target Mixer Insert: CH {:02}", pad.id + 1))
                        .size(9)
                        .color(Color::from_rgb8(0x00, 0xf2, 0xfe)),
                ]
                .spacing(3),
            )
            .padding(6)
            .style(|_t| self.theme.inner_well_style());

            container(
                column![
                    pad_header,
                    sample_info,
                    row![
                        text("Pad Level:").size(10),
                        vol_slider,
                        text(format!("{:.0}%", pad.volume * 100.0)).size(9),
                    ]
                    .spacing(4)
                    .align_y(iced::Alignment::Center),
                    row![text("Pad Pan:").size(10), pan_slider, text(pan_txt).size(9),]
                        .spacing(4)
                        .align_y(iced::Alignment::Center),
                    row![mute_btn, solo_btn].spacing(6),
                ]
                .spacing(6),
            )
            .width(Length::Fill)
            .padding(6)
            .style(|_t| container::Style {
                background: Some(iced::Background::Color(Color::from_rgb8(0x13, 0x15, 0x1a))),
                border: iced::Border {
                    color: Color::from_rgb8(0x28, 0x2c, 0x38),
                    width: 1.0,
                    radius: 4.0.into(),
                },
                text_color: Some(Color::from_rgb8(0xec, 0xf0, 0xf1)),
                ..container::Style::default()
            })
            .into()
        } else {
            column![].into()
        };

        let body = row![pad_matrix_container, pad_inspector,]
            .spacing(10)
            .height(Length::Fill);

        let chrome = self.view_window_chrome(
            "FPC (Fruity Pad Controller)",
            "◈",
            UiMessage::SelectFlView(crate::ui::layout::FlActiveView::Fpc),
        );

        container(column![chrome, header, body].spacing(6))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(4)
            .style(|_t| self.theme.panel_container_style())
            .into()
    }

    /// Dedicated Slicex Beat & Transient Slicer Window
    pub fn view_slicex_window(&self) -> Element<'_, UiMessage> {
        let sl = &self.slicex;
        let header = row![
            container(
                text("SLICEX — BEAT & TRANSIENT SLICER")
                    .size(11)
                    .color(Color::from_rgb8(0xff, 0x88, 0x00)),
            )
            .padding(3),
            Space::new().width(Length::Fixed(12.0)),
            text(format!("Loop: {}", sl.sample_name)).size(10),
            Space::new().width(Length::Fill),
            text(format!("Sens: {:.0}%", sl.sensitivity * 100.0)).size(9),
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center);

        let mut slice_wave_row = row![].spacing(3).align_y(iced::Alignment::Center);
        for marker in &sl.markers {
            let is_act = sl.active_slice_id == Some(marker.id);
            let col = if is_act {
                Color::from_rgb8(0xff, 0x88, 0x00)
            } else {
                Color::from_rgb8(0x00, 0xf2, 0xfe)
            };
            let slice_box = container(
                column![
                    text(&marker.name).size(9).color(col),
                    text(format!("Note: {}", marker.trigger_key)).size(8),
                    Space::new().height(Length::Fixed(30.0)),
                ]
                .padding(3),
            )
            .width(Length::Fixed(75.0))
            .height(Length::Fixed(90.0))
            .style(|_t| self.theme.inner_well_style());

            slice_wave_row = slice_wave_row.push(slice_box);
        }

        let mut trigger_row = row![].spacing(4);
        for marker in &sl.markers {
            let mid = marker.id;
            let is_act = sl.active_slice_id == Some(mid);
            let btn = button(text(format!("Slice {}", mid + 1)).size(9))
                .padding(4)
                .style(self.theme.button_style(is_act))
                .on_press(UiMessage::SlicexTriggerSlice(mid));
            trigger_row = trigger_row.push(btn);
        }

        let chrome = self.view_window_chrome(
            "Slicex — Beat & Transient Slicer",
            "◈",
            UiMessage::SelectFlView(crate::ui::layout::FlActiveView::Slicex),
        );

        container(
            column![
                chrome,
                header,
                container(slice_wave_row)
                    .padding(6)
                    .style(|_t| self.theme.inner_well_style()),
                row![text("Trigger:").size(10), trigger_row]
                    .spacing(6)
                    .align_y(iced::Alignment::Center),
            ]
            .spacing(6),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(4)
        .style(|_t| self.theme.panel_container_style())
        .into()
    }

    /// Dedicated Modular Patcher Routing Graph Window
    pub fn view_patcher_window(&self) -> Element<'_, UiMessage> {
        let p = &self.patcher;
        let header = row![
            container(
                text("PATCHER — MODULAR DSP ROUTING")
                    .size(11)
                    .color(Color::from_rgb8(0xff, 0x88, 0x00)),
            )
            .padding(3),
            Space::new().width(Length::Fill),
            text(format!(
                "{} Nodes • {} Cables",
                p.nodes.len(),
                p.cables.len()
            ))
            .size(9)
            .color(Color::from_rgb8(0x9a, 0xa0, 0xaa)),
        ]
        .align_y(iced::Alignment::Center);

        let mut nodes_row = row![].spacing(12).align_y(iced::Alignment::Center);
        for node in &p.nodes {
            let (k_label, k_color) = match node.kind {
                crate::fl_studio::PatcherNodeKind::FromFL => {
                    ("HOST IN", Color::from_rgb8(0x38, 0xef, 0x7d))
                }
                crate::fl_studio::PatcherNodeKind::ToFL => {
                    ("HOST OUT", Color::from_rgb8(0xe7, 0x4c, 0x3c))
                }
                crate::fl_studio::PatcherNodeKind::GeneratorPlugin => {
                    ("SYNTH", Color::from_rgb8(0x00, 0xf2, 0xfe))
                }
                crate::fl_studio::PatcherNodeKind::EffectPlugin => {
                    ("FX", Color::from_rgb8(0xff, 0x88, 0x00))
                }
                crate::fl_studio::PatcherNodeKind::Splitter => {
                    ("SPLIT", Color::from_rgb8(0x9b, 0x59, 0xb6))
                }
            };

            let node_box = container(
                column![
                    container(text(k_label).size(8).color(Color::WHITE))
                        .padding(2)
                        .style(move |_t| container::Style {
                            background: Some(iced::Background::Color(k_color)),
                            border: iced::Border {
                                radius: 2.0.into(),
                                ..Default::default()
                            },
                            ..container::Style::default()
                        }),
                    text(&node.name).size(10),
                    text("In: Audio/Midi")
                        .size(8)
                        .color(Color::from_rgb8(0x7a, 0x80, 0x8c)),
                    text("Out: Audio/Midi")
                        .size(8)
                        .color(Color::from_rgb8(0x7a, 0x80, 0x8c)),
                ]
                .spacing(3),
            )
            .padding(6)
            .width(Length::Fixed(120.0))
            .style(|_t| self.theme.inner_well_style());

            nodes_row = nodes_row.push(node_box);
        }

        let chrome = self.view_window_chrome(
            "Patcher — Modular Routing",
            "◈",
            UiMessage::SelectFlView(crate::ui::layout::FlActiveView::Patcher),
        );

        container(
            column![
                chrome,
                header,
                container(nodes_row)
                    .padding(12)
                    .style(|_t| self.theme.sunken_container_style()),
            ]
            .spacing(6),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(8)
        .style(|_t| self.theme.panel_container_style())
        .into()
    }

    pub fn view_channel_rack(&self, is_window: bool) -> Element<'_, UiMessage> {
        let pattern_val = self.current_pattern_index + 1;
        let btn_prev_pat = button(text("◀").size(9))
            .padding(2)
            .on_press(UiMessage::SelectPattern(
                self.current_pattern_index.saturating_sub(1),
            ));
        let pat_label = container(text(format!("Pattern {}", pattern_val)).size(10))
            .padding(2)
            .style(|_t| self.theme.vfd_amber_display_style());
        let btn_next_pat = button(text("▶").size(9))
            .padding(2)
            .on_press(UiMessage::SelectPattern(self.current_pattern_index + 1));
        let btn_add_pat = button(text("+").size(9))
            .padding(2)
            .on_press(UiMessage::SelectPattern(self.current_pattern_index + 2));

        let pat_capsule = row![btn_prev_pat, pat_label, btn_next_pat, btn_add_pat]
            .spacing(2)
            .align_y(iced::Alignment::Center);

        let swing_slider = slider(
            0.0..=1.0,
            self.channel_rack.global_swing,
            UiMessage::SetSwing,
        )
        .step(0.01_f32)
        .width(Length::Fixed(55.0));
        let swing_txt = text(format!("{:.0}%", self.channel_rack.global_swing * 100.0)).size(8);

        let graph_btn = button(text("GRAPH").size(9))
            .padding(3)
            .style(self.theme.button_style(self.graph_editor_open))
            .on_press(UiMessage::ToggleChannelRackGraphEditor);

        let step16_btn = button(text("16").size(9))
            .padding(2)
            .style(self.theme.button_style(self.channel_rack_step_count == 16))
            .on_press(UiMessage::SetChannelRackStepCount(16));
        let step32_btn = button(text("32").size(9))
            .padding(2)
            .style(self.theme.button_style(self.channel_rack_step_count == 32))
            .on_press(UiMessage::SetChannelRackStepCount(32));
        let step64_btn = button(text("64").size(9))
            .padding(2)
            .style(self.theme.button_style(self.channel_rack_step_count == 64))
            .on_press(UiMessage::SetChannelRackStepCount(64));
        let steps_row = row![step16_btn, step32_btn, step64_btn].spacing(1);

        let mut header_row = row![
            text("CHANNEL RACK [F6]").size(11),
            pat_capsule,
            row![text("Swing:").size(9), swing_slider, swing_txt]
                .spacing(2)
                .align_y(iced::Alignment::Center),
            graph_btn,
            steps_row,
        ]
        .spacing(6)
        .align_y(iced::Alignment::Center);

        if is_window {
            header_row = header_row.push(Space::new().width(Length::Fill));
            header_row = header_row.push(
                button(text("✕").size(9))
                    .padding(2)
                    .on_press(UiMessage::ToggleFlChannelRack),
            );
        }

        let num_beats = (self.channel_rack_step_count / 4).max(4);
        let mut rack_col = column![].spacing(3).padding(2);

        for chan in &self.channel_rack.channels {
            let chan_id = chan.id;
            let is_sel = self.active_channel_settings == Some(chan_id);

            // Mute / Solo LED dot
            let mute_btn = button(
                container(column![])
                    .width(Length::Fixed(8.0))
                    .height(Length::Fixed(8.0))
                    .style(move |_t| container::Style {
                        background: Some(iced::Background::Color(Color::from_rgb8(
                            0x2e, 0xcc, 0x71,
                        ))),
                        border: iced::Border {
                            radius: 4.0.into(),
                            ..Default::default()
                        },
                        ..Default::default()
                    }),
            )
            .padding(2)
            .style(self.theme.button_style(false))
            .on_press(UiMessage::ToggleTrackMute(
                chan.target_mixer_track.saturating_sub(1),
            ));

            // Pan mini rotary knob (FL style bipolar dial)
            let pan_knob = crate::ui::canvas_knob::rotary_knob(
                20.0,
                -1.0,
                1.0,
                chan.pan,
                true,
                self.theme.accent,
                move |p| UiMessage::ChannelRackSetPan {
                    channel_id: chan_id,
                    pan: p,
                },
            );

            // Vol mini rotary knob (FL style unipolar dial)
            let vol_knob = crate::ui::canvas_knob::rotary_knob(
                20.0,
                0.0,
                1.25,
                chan.volume,
                false,
                Color::from_rgb8(0x2e, 0xcc, 0x71),
                move |v| UiMessage::ChannelRackSetVolume {
                    channel_id: chan_id,
                    volume: v,
                },
            );

            // Target Mixer Track spinbox
            let track_num = chan.target_mixer_track;
            let track_dec = button(text("◀").size(7)).padding(1).on_press(
                UiMessage::ChannelRackSetTargetMixerTrack {
                    channel_id: chan_id,
                    track: track_num.saturating_sub(1).max(1),
                },
            );
            let track_txt = text(format!("{:02}", track_num)).size(9);
            let track_inc = button(text("▶").size(7)).padding(1).on_press(
                UiMessage::ChannelRackSetTargetMixerTrack {
                    channel_id: chan_id,
                    track: (track_num + 1).min(125),
                },
            );
            let track_spinbox = row![track_dec, track_txt, track_inc]
                .spacing(1)
                .align_y(iced::Alignment::Center);

            // Channel button (authentic FL rounded pill with left LED dot)
            let chan_btn = button(
                row![
                    container(column![])
                        .width(Length::Fixed(6.0))
                        .height(Length::Fixed(6.0))
                        .style(move |_t| container::Style {
                            background: Some(iced::Background::Color(if is_sel {
                                Color::from_rgb8(0x00, 0xf2, 0xfe)
                            } else {
                                Color::from_rgb8(0x2e, 0xcc, 0x71)
                            })),
                            border: iced::Border {
                                radius: 3.0.into(),
                                ..Default::default()
                            },
                            ..Default::default()
                        }),
                    text(&chan.name).size(10),
                ]
                .spacing(6)
                .align_y(iced::Alignment::Center),
            )
            .padding([3, 8])
            .style(self.theme.channel_pill_style(is_sel))
            .on_press(UiMessage::OpenChannelSettings(chan_id))
            .width(Length::Fixed(120.0));

            // Step buttons in 4-beat groupings with 3D rubber pad styling
            let mut beat_groups = row![].spacing(5);
            for b_idx in 0..num_beats {
                let mut beat_col = row![].spacing(2);
                for step_in_beat in 0..4 {
                    let s_idx = b_idx * 4 + step_in_beat;
                    let is_active = chan.steps.get(s_idx).map(|s| s.active).unwrap_or(false);
                    let is_even_beat = b_idx % 2 != 0;
                    let step_lbl = if is_active { "●" } else { "•" };
                    let step_btn = button(text(step_lbl).size(10))
                        .style(self.theme.step_pad_style(is_active, is_even_beat))
                        .padding([2, 5])
                        .on_press(UiMessage::ChannelRackToggleStep {
                            channel_id: chan_id,
                            step_index: s_idx,
                        });
                    beat_col = beat_col.push(step_btn);
                }
                beat_groups = beat_groups.push(beat_col);
            }

            let ch_row = row![
                mute_btn,
                pan_knob,
                vol_knob,
                track_spinbox,
                chan_btn,
                beat_groups,
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center);

            rack_col = rack_col.push(ch_row);
        }

        let add_chan_btn = button(text("+ Add One More Channel (Plugin Picker)").size(10))
            .padding(3)
            .style(self.theme.button_style(false))
            .on_press(UiMessage::ToggleFlPluginPicker);

        let mut main_rack = column![
            header_row,
            scrollable(rack_col).height(Length::Shrink),
            add_chan_btn,
        ]
        .spacing(4);

        if self.graph_editor_open {
            let active_cid = self.active_channel_settings.unwrap_or(0);
            let mut graph_bars = row![].spacing(2).align_y(iced::Alignment::End);
            if let Some(ch) = self.channel_rack.channels.get(active_cid) {
                for s in ch.steps.iter().take(self.channel_rack_step_count) {
                    let h = (s.velocity * 36.0).max(4.0);
                    let bar = container(column![])
                        .width(Length::Fixed(12.0))
                        .height(Length::Fixed(h))
                        .style(|_t| container::Style {
                            background: Some(iced::Background::Color(Color::from_rgb8(
                                0x00, 0xf2, 0xfe,
                            ))),
                            border: iced::Border {
                                radius: 1.0.into(),
                                ..Default::default()
                            },
                            ..container::Style::default()
                        });
                    graph_bars = graph_bars.push(bar);
                }
            }
            let graph_drawer = container(
                column![
                    row![
                        text(format!(
                            "GRAPH EDITOR — Velocity Stems (Channel {})",
                            active_cid + 1
                        ))
                        .size(9),
                        Space::new().width(Length::Fill),
                        text("0..100%").size(8),
                    ]
                    .spacing(4),
                    container(graph_bars).height(Length::Fixed(40.0)).padding(2),
                ]
                .spacing(2),
            )
            .padding(4)
            .style(|_t| self.theme.sunken_container_style());

            main_rack = main_rack.push(graph_drawer);
        }

        let chrome = self.view_window_chrome(
            "Channel Rack",
            "◈",
            UiMessage::SelectFlView(crate::ui::layout::FlActiveView::ChannelRack),
        );

        let rack_column = column![chrome, main_rack].spacing(4);

        if is_window {
            container(rack_column)
                .style(|_t| self.theme.panel_container_style())
                .padding(4)
                .into()
        } else {
            container(
                container(rack_column)
                    .style(|_t| self.theme.panel_container_style())
                    .padding(4),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(4)
            .into()
        }
    }

    pub fn view_channel_settings_modal(&self, chan_id: usize) -> Element<'_, UiMessage> {
        let chan = self.channel_rack.channels.get(chan_id);
        let name = chan.map(|c| c.name.as_str()).unwrap_or("Channel");
        let vol = chan.map(|c| c.volume).unwrap_or(0.8);
        let pan = chan.map(|c| c.pan).unwrap_or(0.0);
        let pitch_st = chan.map(|c| c.pitch_semitones).unwrap_or(0);
        let target_track = chan.map(|c| c.target_mixer_track).unwrap_or(1);

        let title_bar = row![
            text(format!("CHANNEL SETTINGS — {} (Fruity Wrapper)", name))
                .size(12)
                .color(Color::from_rgb8(0xff, 0x88, 0x00)),
            Space::new().width(Length::Fill),
            button(text("✕").size(10))
                .padding(2)
                .on_press(UiMessage::CloseChannelSettings),
        ]
        .align_y(iced::Alignment::Center);

        // Routing Section: Target Mixer Track
        let track_dec = button(text("◀").size(8)).padding(2).on_press(
            UiMessage::ChannelSettingsSetMixerTrack {
                channel_id: chan_id,
                track: target_track.saturating_sub(1).max(1),
            },
        );
        let track_txt = text(format!("Mixer Track: {:02}", target_track)).size(10);
        let track_inc = button(text("▶").size(8)).padding(2).on_press(
            UiMessage::ChannelSettingsSetMixerTrack {
                channel_id: chan_id,
                track: (target_track + 1).min(125),
            },
        );
        let routing_row = row![track_dec, track_txt, track_inc]
            .spacing(4)
            .align_y(iced::Alignment::Center);

        // Tuning & Levels
        let pitch_slider = slider(-1200..=1200, (pitch_st as i32) * 100, move |cents| {
            UiMessage::ChannelSettingsSetPitch {
                channel_id: chan_id,
                pitch_cents: cents,
            }
        })
        .step(10)
        .width(Length::Fixed(120.0));
        let pitch_txt = text(format!("Pitch: {:+} ct", (pitch_st as i32) * 100)).size(9);

        let pan_slider = slider(-1.0..=1.0, pan, move |p| UiMessage::ChannelSettingsSetPan {
            channel_id: chan_id,
            pan: p,
        })
        .step(0.02_f32)
        .width(Length::Fixed(120.0));
        let pan_txt = text(format!("Pan: {:.2}", pan)).size(9);

        let vol_slider = slider(0.0..=1.25, vol, move |v| {
            UiMessage::ChannelSettingsSetVolume {
                channel_id: chan_id,
                volume: v,
            }
        })
        .step(0.02_f32)
        .width(Length::Fixed(120.0));
        let vol_txt = text(format!("Vol: {:.0}%", vol * 100.0)).size(9);

        let left_col = column![
            text("GENERAL / ROUTING")
                .size(10)
                .color(Color::from_rgb8(0x5a, 0xe0, 0xff)),
            routing_row,
            row![text("Pitch:").size(9), pitch_slider, pitch_txt]
                .spacing(4)
                .align_y(iced::Alignment::Center),
            row![text("Pan:").size(9), pan_slider, pan_txt]
                .spacing(4)
                .align_y(iced::Alignment::Center),
            row![text("Vol:").size(9), vol_slider, vol_txt]
                .spacing(4)
                .align_y(iced::Alignment::Center),
        ]
        .spacing(6);

        let tab_smp = button(text("SMP").size(9))
            .style(self.theme.button_style(
                self.wrapper_state.active_tab == crate::fl_studio::wrapper::WrapperTab::Sampler,
            ))
            .on_press(UiMessage::WrapperSetTab(
                crate::fl_studio::wrapper::WrapperTab::Sampler,
            ));
        let tab_env = button(text("ENV").size(9))
            .style(self.theme.button_style(
                self.wrapper_state.active_tab == crate::fl_studio::wrapper::WrapperTab::Envelope,
            ))
            .on_press(UiMessage::WrapperSetTab(
                crate::fl_studio::wrapper::WrapperTab::Envelope,
            ));
        let tab_misc = button(text("MISC").size(9))
            .style(self.theme.button_style(
                self.wrapper_state.active_tab == crate::fl_studio::wrapper::WrapperTab::Misc,
            ))
            .on_press(UiMessage::WrapperSetTab(
                crate::fl_studio::wrapper::WrapperTab::Misc,
            ));

        let tabs_bar = row![tab_smp, tab_env, tab_misc].spacing(4);

        let env = &self.wrapper_state.volume_env;
        let att = env.attack_ms;
        let dec = env.decay_ms;
        let sus = env.sustain_level;
        let rel = env.release_ms;

        let att_slider = slider(0.0..=1000.0, att, move |v| UiMessage::WrapperSetAdsr {
            channel_id: chan_id,
            attack: v,
            decay: dec,
            sustain: sus,
            release: rel,
        })
        .step(1.0_f32)
        .width(Length::Fixed(70.0));

        let dec_slider = slider(0.0..=2000.0, dec, move |v| UiMessage::WrapperSetAdsr {
            channel_id: chan_id,
            attack: att,
            decay: v,
            sustain: sus,
            release: rel,
        })
        .step(5.0_f32)
        .width(Length::Fixed(70.0));

        let sus_slider = slider(0.0..=1.0, sus, move |v| UiMessage::WrapperSetAdsr {
            channel_id: chan_id,
            attack: att,
            decay: dec,
            sustain: v,
            release: rel,
        })
        .step(0.02_f32)
        .width(Length::Fixed(70.0));

        let rel_slider = slider(0.0..=3000.0, rel, move |v| UiMessage::WrapperSetAdsr {
            channel_id: chan_id,
            attack: att,
            decay: dec,
            sustain: sus,
            release: v,
        })
        .step(10.0_f32)
        .width(Length::Fixed(70.0));

        let adsr_controls = row![
            column![
                text("ATT").size(8),
                att_slider,
                text(format!("{:.0}ms", att)).size(8)
            ]
            .spacing(2),
            column![
                text("DEC").size(8),
                dec_slider,
                text(format!("{:.0}ms", dec)).size(8)
            ]
            .spacing(2),
            column![
                text("SUS").size(8),
                sus_slider,
                text(format!("{:.0}%", sus * 100.0)).size(8)
            ]
            .spacing(2),
            column![
                text("REL").size(8),
                rel_slider,
                text(format!("{:.0}ms", rel)).size(8)
            ]
            .spacing(2),
        ]
        .spacing(6);

        let stretch_info = row![
            text("Mode: Auto Stretch").size(9),
            text("|").size(9),
            text("Polyphony: 32v").size(9),
            text("|").size(9),
            text("Porta: Off").size(9),
        ]
        .spacing(4);

        let right_col = column![
            row![
                text("VOLUME ADSR ENVELOPE")
                    .size(10)
                    .color(Color::from_rgb8(0x5a, 0xe0, 0xff)),
                Space::new().width(Length::Fill),
                tabs_bar,
            ]
            .spacing(4)
            .align_y(iced::Alignment::Center),
            adsr_controls,
            stretch_info,
        ]
        .spacing(6);

        container(
            column![
                title_bar,
                row![left_col, Space::new().width(Length::Fixed(16.0)), right_col].spacing(8),
            ]
            .spacing(8),
        )
        .style(|_t| self.theme.oled_vfd_display_style())
        .width(Length::Fixed(560.0))
        .padding(10)
        .into()
    }
}

/// 12-segment graduated LED ladder meter with peak hold and clip overload indicator
pub fn render_led_meter<'a>(
    peak_db: f32,
    _rms_db: f32,
    peak_hold_db: f32,
    auto_mute_tripped: bool,
) -> Element<'a, UiMessage> {
    const THRESHOLDS: [f32; 12] = [
        -60.0, -48.0, -36.0, -24.0, -18.0, -14.0, -12.0, -9.0, -6.0, -3.0, -1.0, 0.0,
    ];

    let mut segments = row![].spacing(1);

    for (i, &thresh) in THRESHOLDS.iter().enumerate() {
        let is_lit = peak_db >= thresh || (auto_mute_tripped && i == 11);
        let next_thresh = THRESHOLDS.get(i + 1).copied().unwrap_or(100.0);
        let is_peak_hold = peak_hold_db >= thresh && peak_hold_db < next_thresh;

        let seg_color = match i {
            0..=6 => {
                if is_lit || is_peak_hold {
                    Color::from_rgb8(0x2e, 0xcc, 0x71) // Active Green (-60 to -12 dBFS)
                } else {
                    Color::from_rgb8(0x13, 0x2e, 0x1d) // Dim Green
                }
            }
            7..=9 => {
                if is_lit || is_peak_hold {
                    Color::from_rgb8(0xf3, 0x9c, 0x12) // Active Amber (-9 to -3 dBFS)
                } else {
                    Color::from_rgb8(0x38, 0x24, 0x0a) // Dim Amber
                }
            }
            10 => {
                if is_lit || is_peak_hold {
                    Color::from_rgb8(0xe6, 0x64, 0x14) // Active Orange (-1 dBFS)
                } else {
                    Color::from_rgb8(0x38, 0x1a, 0x08) // Dim Orange
                }
            }
            _ => {
                if is_lit || is_peak_hold || auto_mute_tripped || peak_db > 0.0 {
                    Color::from_rgb8(0xe7, 0x4c, 0x3c) // Active Red (0 dBFS / CLIP)
                } else {
                    Color::from_rgb8(0x38, 0x14, 0x14) // Dim Red
                }
            }
        };

        let seg_box = container(column![])
            .width(Length::Fixed(3.0))
            .height(Length::Fixed(5.0))
            .style(move |_t| container::Style {
                background: Some(iced::Background::Color(seg_color)),
                border: iced::Border {
                    radius: 1.0.into(),
                    ..Default::default()
                },
                ..container::Style::default()
            });

        segments = segments.push(seg_box);
    }

    let readout_text = if auto_mute_tripped {
        text("MUTE")
            .size(8)
            .color(Color::from_rgb8(0xe7, 0x4c, 0x3c))
    } else if peak_db > 0.0 {
        text(format!("+{:.1}", peak_db))
            .size(8)
            .color(Color::from_rgb8(0xe7, 0x4c, 0x3c))
    } else if peak_db > -60.0 {
        text(format!("{:.1}", peak_db))
            .size(8)
            .color(Color::from_rgb8(0xba, 0xbe, 0xc6))
    } else {
        text("-inf")
            .size(8)
            .color(Color::from_rgb8(0x5a, 0x5e, 0x66))
    };

    row![segments, readout_text]
        .spacing(3)
        .align_y(iced::Alignment::Center)
        .into()
}

/// 16-segment vertical graduated LED ladder meter with peak hold, RMS, and clip overload indicator
pub fn render_vertical_led_meter<'a>(
    peak_db: f32,
    _rms_db: f32,
    peak_hold_db: f32,
    auto_mute_tripped: bool,
) -> Element<'a, UiMessage> {
    const THRESHOLDS: [f32; 16] = [
        0.0, -0.5, -1.0, -2.0, -3.0, -4.5, -6.0, -9.0, -12.0, -15.0, -18.0, -24.0, -30.0, -36.0,
        -48.0, -60.0,
    ];

    let mut segments = column![].spacing(1);

    for (i, &thresh) in THRESHOLDS.iter().enumerate() {
        let is_lit = peak_db >= thresh || (auto_mute_tripped && i == 0);
        let higher_thresh = if i == 0 { 100.0 } else { THRESHOLDS[i - 1] };
        let is_peak_hold = peak_hold_db >= thresh && peak_hold_db < higher_thresh;

        let (lit_color, dim_color) = match i {
            0..=1 => (
                Color::from_rgb8(0xff, 0x33, 0x44), // Neon Clip Red
                Color::from_rgb8(0x35, 0x12, 0x16),
            ),
            2 => (
                Color::from_rgb8(0xf8, 0x77, 0x24), // Meter1 Orange: -14387208 (#f87724)
                Color::from_rgb8(0x38, 0x1a, 0x0a),
            ),
            3..=6 => (
                Color::from_rgb8(0xf6, 0xa1, 0x03), // Meter2 Amber: -16539146 (#f6a103)
                Color::from_rgb8(0x3a, 0x24, 0x08),
            ),
            _ => (
                Color::from_rgb8(0x2e, 0xcc, 0x71), // Clean Emerald / Lime
                Color::from_rgb8(0x0e, 0x24, 0x14),
            ),
        };

        let seg_color = if is_lit || (auto_mute_tripped && i == 0) {
            lit_color
        } else if is_peak_hold {
            // Peak hold line shines brighter
            Color::from_rgba8(255, 255, 255, 0.90)
        } else {
            dim_color
        };

        let seg_box = container(column![])
            .width(Length::Fixed(4.0))
            .height(Length::Fixed(14.0))
            .style(move |_t| container::Style {
                background: Some(iced::Background::Color(seg_color)),
                border: iced::Border {
                    radius: 1.0.into(),
                    ..Default::default()
                },
                ..container::Style::default()
            });

        segments = segments.push(seg_box);
    }

    segments.into()
}
