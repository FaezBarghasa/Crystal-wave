//! Edison Waveform Editor GPU Canvas Program
//!
//! Renders peak-decimated stereo waveform geometry, timecode grid lines,
//! selection highlighting, and interactive scrub/selection dragging.

use crate::fl_studio::edison::EdisonMarker;
use crate::ui::message::UiMessage;
use iced::mouse;
use iced::widget::canvas::{self, Action, Canvas, Frame, Geometry, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Size, Theme};

/// Interactive state for the Edison canvas (drag selection)
#[derive(Debug, Clone, Copy, Default)]
pub struct EdisonCanvasState {
    pub drag_start_x: Option<f32>,
    pub current_drag_x: Option<f32>,
}

pub struct EdisonCanvasProgram<'a> {
    pub samples_l: &'a [f32],
    pub samples_r: &'a [f32],
    pub sample_rate: f32,
    pub selection_start: usize,
    pub selection_end: usize,
    pub cursor_pos: usize,
    pub markers: &'a [EdisonMarker],
}

impl<'a> EdisonCanvasProgram<'a> {
    pub fn new(
        samples_l: &'a [f32],
        samples_r: &'a [f32],
        sample_rate: f32,
        selection_start: usize,
        selection_end: usize,
        cursor_pos: usize,
        markers: &'a [EdisonMarker],
    ) -> Self {
        Self {
            samples_l,
            samples_r,
            sample_rate,
            selection_start,
            selection_end,
            cursor_pos,
            markers,
        }
    }
}

impl<'a> canvas::Program<UiMessage> for EdisonCanvasProgram<'a> {
    type State = EdisonCanvasState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<UiMessage>> {
        let cursor_pos = cursor.position_in(bounds)?;
        let total_samples = self.samples_l.len();
        if total_samples == 0 || bounds.width <= 0.0 {
            return None;
        }

        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                state.drag_start_x = Some(cursor_pos.x);
                state.current_drag_x = Some(cursor_pos.x);

                let sample_idx = ((cursor_pos.x / bounds.width) * total_samples as f32) as usize;
                let sample_idx = sample_idx.min(total_samples);
                Some(Action::publish(UiMessage::EdisonSetCursor(sample_idx)))
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if let Some(start_x) = state.drag_start_x {
                    state.current_drag_x = Some(cursor_pos.x);
                    let s_start = ((start_x.min(cursor_pos.x) / bounds.width)
                        * total_samples as f32) as usize;
                    let s_end = ((start_x.max(cursor_pos.x) / bounds.width) * total_samples as f32)
                        as usize;
                    Some(Action::publish(UiMessage::EdisonSetSelection {
                        start: s_start.min(total_samples),
                        end: s_end.min(total_samples),
                    }))
                } else {
                    None
                }
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if let (Some(start_x), Some(end_x)) =
                    (state.drag_start_x.take(), state.current_drag_x.take())
                {
                    let s_start =
                        ((start_x.min(end_x) / bounds.width) * total_samples as f32) as usize;
                    let s_end =
                        ((start_x.max(end_x) / bounds.width) * total_samples as f32) as usize;
                    Some(Action::publish(UiMessage::EdisonSetSelection {
                        start: s_start.min(total_samples),
                        end: s_end.min(total_samples),
                    }))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let total_samples = self.samples_l.len();

        // 1. Dark obsidian background
        frame.fill_rectangle(
            Point::ORIGIN,
            bounds.size(),
            Color::from_rgb8(0x10, 0x12, 0x16),
        );

        if total_samples == 0 || bounds.width <= 0.0 || bounds.height <= 0.0 {
            return vec![frame.into_geometry()];
        }

        let mid_y = bounds.height * 0.5;
        let half_h = mid_y * 0.92;

        // 2. Time Grid Lines
        let duration = total_samples as f32 / self.sample_rate.max(1.0);
        let grid_interval = if duration < 1.0 {
            0.1
        } else if duration < 5.0 {
            0.5
        } else {
            1.0
        };

        let mut t = 0.0;
        while t <= duration {
            let x = (t / duration) * bounds.width;
            frame.stroke(
                &Path::line(Point::new(x, 0.0), Point::new(x, bounds.height)),
                Stroke::default()
                    .with_color(Color::from_rgba8(0x2a, 0x30, 0x40, 0.45))
                    .with_width(1.0),
            );
            t += grid_interval;
        }

        // 3. Center Zero-Crossing Line
        frame.stroke(
            &Path::line(Point::new(0.0, mid_y), Point::new(bounds.width, mid_y)),
            Stroke::default()
                .with_color(Color::from_rgb8(0x28, 0x2c, 0x38))
                .with_width(1.0),
        );

        // 4. Selection Highlight Background
        if self.selection_start < self.selection_end {
            let sel_x1 = (self.selection_start as f32 / total_samples as f32) * bounds.width;
            let sel_x2 = (self.selection_end as f32 / total_samples as f32) * bounds.width;
            let sel_w = (sel_x2 - sel_x1).max(1.0);

            frame.fill_rectangle(
                Point::new(sel_x1, 0.0),
                Size::new(sel_w, bounds.height),
                Color::from_rgba8(0xff, 0x85, 0x00, 0.22),
            );

            // Selection boundary lines
            frame.stroke(
                &Path::line(Point::new(sel_x1, 0.0), Point::new(sel_x1, bounds.height)),
                Stroke::default()
                    .with_color(Color::from_rgb8(0xff, 0x85, 0x00))
                    .with_width(1.0),
            );
            frame.stroke(
                &Path::line(Point::new(sel_x2, 0.0), Point::new(sel_x2, bounds.height)),
                Stroke::default()
                    .with_color(Color::from_rgb8(0xff, 0x85, 0x00))
                    .with_width(1.0),
            );
        }

        // 5. Waveform Geometry
        let pixel_cols = bounds.width.ceil() as usize;
        let samples_per_col = (total_samples as f32 / pixel_cols.max(1) as f32).max(1.0);

        let unselected_color = Color::from_rgb8(0x00, 0xf2, 0xfe); // Vibrant cyan
        let selected_color = Color::from_rgb8(0xff, 0x85, 0x00); // Citrus orange

        for px in 0..pixel_cols {
            let x = px as f32;
            let start_idx = (px as f32 * samples_per_col) as usize;
            let end_idx = (((px + 1) as f32 * samples_per_col) as usize).min(total_samples);

            if start_idx >= total_samples {
                break;
            }

            let in_selection = start_idx >= self.selection_start && start_idx <= self.selection_end;
            let wave_color = if in_selection {
                selected_color
            } else {
                unselected_color
            };

            let mut max_val = 0.0_f32;
            let mut min_val = 0.0_f32;

            for i in start_idx..end_idx {
                let sl = self.samples_l[i];
                let sr = self.samples_r.get(i).copied().unwrap_or(sl);
                let s = (sl + sr) * 0.5;
                if s > max_val {
                    max_val = s;
                }
                if s < min_val {
                    min_val = s;
                }
            }

            let y_top = mid_y - (max_val * half_h);
            let y_bot = mid_y - (min_val * half_h);
            let actual_top = y_top.min(y_bot).min(mid_y - 1.0);
            let actual_bot = y_top.max(y_bot).max(mid_y + 1.0);

            frame.stroke(
                &Path::line(Point::new(x, actual_top), Point::new(x, actual_bot)),
                Stroke::default().with_color(wave_color).with_width(1.0),
            );
        }

        // 6. Markers
        for marker in self.markers {
            let mx = (marker.sample_index as f32 / total_samples as f32) * bounds.width;
            frame.stroke(
                &Path::line(Point::new(mx, 0.0), Point::new(mx, bounds.height)),
                Stroke::default()
                    .with_color(Color::from_rgb8(0xf1, 0xc4, 0x0f))
                    .with_width(1.5),
            );
        }

        // 7. Cursor / Playhead Line
        let cur_x = (self.cursor_pos as f32 / total_samples as f32) * bounds.width;
        frame.stroke(
            &Path::line(Point::new(cur_x, 0.0), Point::new(cur_x, bounds.height)),
            Stroke::default()
                .with_color(Color::from_rgb8(0x38, 0xef, 0x7d))
                .with_width(1.5),
        );

        vec![frame.into_geometry()]
    }
}

pub fn view_canvas_edison<'a>(
    samples_l: &'a [f32],
    samples_r: &'a [f32],
    sample_rate: f32,
    selection_start: usize,
    selection_end: usize,
    cursor_pos: usize,
    markers: &'a [EdisonMarker],
) -> Element<'a, UiMessage> {
    Canvas::new(EdisonCanvasProgram::new(
        samples_l,
        samples_r,
        sample_rate,
        selection_start,
        selection_end,
        cursor_pos,
        markers,
    ))
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
