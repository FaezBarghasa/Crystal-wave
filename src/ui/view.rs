use crate::ui::app::CrystalWaveApp;
use crate::ui::message::{MenuCommand, UiMessage};
use iced::widget::{button, column, container, row, text};
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
            .into()
    }

    fn view_menu_bar(&self) -> Element<'_, UiMessage> {
        let file_new = button(text("New")).on_press(UiMessage::MenuAction(MenuCommand::NewProject));
        let file_save =
            button(text("Save")).on_press(UiMessage::MenuAction(MenuCommand::SaveSession));
        let file_bounce =
            button(text("Bounce WAV")).on_press(UiMessage::MenuAction(MenuCommand::BounceWav));
        let file_aaf =
            button(text("Export AAF")).on_press(UiMessage::MenuAction(MenuCommand::ExportAaf));

        let edit_split =
            button(text("Split (B)")).on_press(UiMessage::MenuAction(MenuCommand::SplitAtCursor));
        let track_add_audio =
            button(text("+ Audio")).on_press(UiMessage::MenuAction(MenuCommand::AddAudioTrack));
        let track_add_midi =
            button(text("+ MIDI")).on_press(UiMessage::MenuAction(MenuCommand::AddMidiTrack));

        let win_mixer =
            button(text("Mixer (F9)")).on_press(UiMessage::MenuAction(MenuCommand::ToggleMixer));
        let win_piano_roll = button(text("Piano Roll (F7)"))
            .on_press(UiMessage::MenuAction(MenuCommand::TogglePianoRoll));
        let win_inspector = button(text("Inspector (I)"))
            .on_press(UiMessage::MenuAction(MenuCommand::ToggleInspector));

        row![
            container(text("CrystalWave")).padding(4),
            file_new,
            file_save,
            file_bounce,
            file_aaf,
            edit_split,
            track_add_audio,
            track_add_midi,
            win_mixer,
            win_piano_roll,
            win_inspector
        ]
        .spacing(6)
        .padding(4)
        .into()
    }

    fn view_control_bar(&self) -> Element<'_, UiMessage> {
        let play_btn = button(text(if self.is_playing { "Pause" } else { "Play" })).on_press(
            if self.is_playing {
                UiMessage::TransportPause
            } else {
                UiMessage::TransportPlay
            },
        );
        let stop_btn = button(text("Stop")).on_press(UiMessage::TransportStop);
        let record_btn = button(text("Record")).on_press(UiMessage::TransportRecord);

        let lcd = container(text(format!(
            "BPM: {:.1} | Sig: {}/{} | Pos: {} | CPU: {:.1}%",
            self.workspace.lcd_diagnostic.project_tempo_bpm,
            self.workspace.lcd_diagnostic.time_signature.0,
            self.workspace.lcd_diagnostic.time_signature.1,
            self.playhead_tick,
            self.workspace.lcd_diagnostic.cpu_load_percent,
        )))
        .padding(6);

        row![play_btn, stop_btn, record_btn, lcd]
            .spacing(8)
            .padding(4)
            .into()
    }

    fn view_main_workspace(&self) -> Element<'_, UiMessage> {
        let mut content = row![].spacing(4).width(Length::Fill).height(Length::Fill);

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

            let mute_btn = button(text(if is_muted { "Muted" } else { "Mute" }))
                .on_press(UiMessage::ToggleTrackMute(trk_id));

            let solo_btn = button(text(if is_solo { "Soloed" } else { "Solo" }))
                .on_press(UiMessage::ToggleTrackSolo(trk_id));

            let inspector_box = container(
                column![
                    text(format!("Inspector: Track {}", trk_id)),
                    text(format!("Name: {}", channel_name)),
                    text(format!("Level: {:.1} dB", fader_val)),
                    text(format!("Pan: {:.2}", pan_val)),
                    row![mute_btn, solo_btn].spacing(4),
                ]
                .spacing(6),
            )
            .width(Length::Fixed(220.0))
            .padding(6);

            content = content.push(inspector_box);
        }

        let canvas_program = crate::ui::canvas_timeline::ArrangementCanvasProgram::new(
            self.playhead_tick,
            self.arrangement.horizontal_zoom,
            self.mixer.channels.len(),
        );
        let arrangement_canvas = crate::ui::canvas_timeline::view_canvas_timeline(canvas_program);
        content = content.push(arrangement_canvas);

        content.into()
    }

    fn view_bottom_drawer(&self) -> Element<'_, UiMessage> {
        if !self.workspace.editor_drawer_open && !self.workspace.mixer_drawer_open {
            return column![].into();
        }

        if self.workspace.mixer_drawer_open {
            let mut channel_strips = row![].spacing(8).padding(6);

            for (idx, ch) in self.mixer.channels.iter().enumerate() {
                let mute_btn = button(text(if ch.muted { "M" } else { "m" }))
                    .on_press(UiMessage::ToggleTrackMute(idx));

                let solo_btn = button(text(if ch.solo { "S" } else { "s" }))
                    .on_press(UiMessage::ToggleTrackSolo(idx));

                let fader_up = button(text("+")).on_press(UiMessage::SetTrackFader {
                    track_id: idx,
                    volume_db: (ch.volume_fader_db + 1.0).min(6.0),
                });

                let fader_down = button(text("-")).on_press(UiMessage::SetTrackFader {
                    track_id: idx,
                    volume_db: (ch.volume_fader_db - 1.0).max(-120.0),
                });

                let strip = container(
                    column![
                        text(&ch.name),
                        text(format!("{:.1} dB", ch.volume_fader_db)),
                        row![fader_down, fader_up].spacing(2),
                        text(format!("Pan: {:.2}", ch.pan.balance)),
                        row![mute_btn, solo_btn].spacing(4),
                    ]
                    .spacing(4),
                )
                .padding(4)
                .width(Length::Fixed(100.0));

                channel_strips = channel_strips.push(strip);
            }

            return container(channel_strips)
                .height(Length::Fixed(180.0))
                .width(Length::Fill)
                .into();
        }

        container(text(format!("Editor: {:?}", self.workspace.active_editor)))
            .height(Length::Fixed(180.0))
            .width(Length::Fill)
            .padding(6)
            .into()
    }
}
