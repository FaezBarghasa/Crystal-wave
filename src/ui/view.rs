use crate::ui::app::CrystalWaveApp;
use crate::ui::message::UiMessage;
use iced::widget::{button, column, container, row, text};
use iced::{Element, Length};

impl CrystalWaveApp {
    pub fn view(&self) -> Element<'_, UiMessage> {
        let control_bar = self.view_control_bar();
        let main_workspace = self.view_main_workspace();
        let bottom_drawer = self.view_bottom_drawer();

        column![control_bar, main_workspace, bottom_drawer]
            .width(Length::Fill)
            .height(Length::Fill)
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
            let inspector_box = container(text(format!(
                "Inspector (Track {})",
                self.workspace.selected_track_id
            )))
            .width(Length::Fixed(220.0));
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

        container(text(format!("Drawer: {:?}", self.workspace.active_editor)))
            .height(Length::Fixed(180.0))
            .width(Length::Fill)
            .into()
    }
}
