use crate::ui::message::UiMessage;
use iced::mouse;
use iced::widget::canvas::{self, Action, Canvas, Frame, Geometry, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle, Size};

#[derive(Debug, Clone, Default)]
pub struct CanvasInteractionState {
    pub is_dragging_playhead: bool,
    pub dragged_clip_id: Option<usize>,
    pub drag_start_offset_ticks: u64,
    pub drag_action: Option<crate::pro_tools::smart_tool::SmartToolAction>,
    pub last_cursor_pos: Option<Point>,
}

#[derive(Debug, Clone)]
pub struct WaveformPeak {
    pub min: f32,
    pub max: f32,
}

#[derive(Debug, Clone)]
pub struct TimelineClipDisplay {
    pub id: usize,
    pub track_index: usize,
    pub start_tick: u64,
    pub length_ticks: u64,
    pub name: String,
    pub color_rgb: [u8; 3],
}

pub struct ArrangementCanvasProgram {
    pub playhead_tick: u64,
    pub visible_start_tick: u64,
    pub visible_end_tick: u64,
    pub horizontal_zoom: f32,
    pub track_count: usize,
    pub loop_range: Option<(u64, u64)>,
    pub sample_peaks: Vec<WaveformPeak>,
    pub clips: Vec<TimelineClipDisplay>,
}

impl ArrangementCanvasProgram {
    pub fn new(playhead_tick: u64, horizontal_zoom: f32, track_count: usize) -> Self {
        Self {
            playhead_tick,
            visible_start_tick: 0,
            visible_end_tick: 3840 * 16,
            horizontal_zoom,
            track_count: track_count.max(1),
            loop_range: Some((0, 3840 * 4)),
            sample_peaks: Vec::new(),
            clips: Vec::new(),
        }
    }

    pub fn tick_to_x(&self, tick: u64, bounds_width: f32) -> f32 {
        let total_ticks = self
            .visible_end_tick
            .saturating_sub(self.visible_start_tick)
            .max(1);
        let progress = (tick.saturating_sub(self.visible_start_tick) as f32) / (total_ticks as f32);
        (progress * bounds_width * self.horizontal_zoom).clamp(0.0, bounds_width * 100.0)
    }

    pub fn x_to_tick(&self, x: f32, bounds_width: f32) -> u64 {
        if bounds_width <= 0.0 || self.horizontal_zoom <= 0.0 {
            return 0;
        }
        let total_ticks = self
            .visible_end_tick
            .saturating_sub(self.visible_start_tick)
            .max(1);
        let progress = (x / (bounds_width * self.horizontal_zoom)).clamp(0.0, 1.0);
        self.visible_start_tick + (progress * total_ticks as f32) as u64
    }

    pub fn y_to_track(&self, y: f32, bounds_height: f32) -> usize {
        let lane_h = (bounds_height / self.track_count as f32).clamp(30.0, 120.0);
        ((y / lane_h) as usize).min(self.track_count.saturating_sub(1))
    }
}

impl<Theme> canvas::Program<UiMessage, Theme> for ArrangementCanvasProgram {
    type State = CanvasInteractionState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<UiMessage>> {
        let cursor_position = cursor.position_in(bounds)?;

        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let target_tick = self.x_to_tick(cursor_position.x, bounds.width);
                let target_trk = self.y_to_track(cursor_position.y, bounds.height);

                // Check if user clicked directly on an existing clip (enabling Drag-and-Drop)
                let clicked_clip = self.clips.iter().find(|clip| {
                    clip.track_index == target_trk
                        && target_tick >= clip.start_tick
                        && target_tick < (clip.start_tick + clip.length_ticks)
                });

                if let Some(clip) = clicked_clip {
                    state.dragged_clip_id = Some(clip.id);
                    state.drag_start_offset_ticks = target_tick.saturating_sub(clip.start_tick);
                    state.is_dragging_playhead = false;
                    None
                } else {
                    state.is_dragging_playhead = true;
                    Some(Action::publish(UiMessage::SetPlayhead(target_tick)))
                }
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if let Some(clip_id) = state.dragged_clip_id.take() {
                    let target_tick = self.x_to_tick(cursor_position.x, bounds.width);
                    let target_trk = self.y_to_track(cursor_position.y, bounds.height);
                    let new_start_tick = target_tick.saturating_sub(state.drag_start_offset_ticks);
                    // Snap to 16th note (240 ticks) for clean studio alignment
                    let snapped_tick = (new_start_tick / 240) * 240;
                    Some(Action::publish(UiMessage::TimelineMoveClip {
                        clip_id,
                        new_track_id: target_trk,
                        new_start_tick: snapped_tick,
                    }))
                } else {
                    state.is_dragging_playhead = false;
                    None
                }
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.is_dragging_playhead {
                    let target_tick = self.x_to_tick(cursor_position.x, bounds.width);
                    Some(Action::publish(UiMessage::SetPlayhead(target_tick)))
                } else {
                    None
                }
            }
            canvas::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let scroll_y = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y,
                    mouse::ScrollDelta::Pixels { y, .. } => *y / 20.0,
                };
                let new_zoom = (self.horizontal_zoom + scroll_y * 0.1).clamp(0.1, 20.0);
                Some(Action::publish(UiMessage::SetHorizontalZoom(new_zoom)))
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

        // 1. Background dark canvas
        frame.fill_rectangle(
            Point::ORIGIN,
            bounds.size(),
            Color::from_rgb8(0x1e, 0x20, 0x22),
        );

        // 2. Track lanes
        let lane_height = (bounds.height / self.track_count as f32).clamp(30.0, 120.0);
        for i in 0..self.track_count {
            let y = i as f32 * lane_height;
            let bg_color = if i % 2 == 0 {
                Color::from_rgb8(0x23, 0x26, 0x29)
            } else {
                Color::from_rgb8(0x1a, 0x1c, 0x1e)
            };
            frame.fill_rectangle(
                Point::new(0.0, y),
                Size::new(bounds.width, lane_height - 1.0),
                bg_color,
            );
        }

        // 3. Measure Grid lines (every 960 ticks = 1 beat, 3840 ticks = 1 bar)
        let bar_ticks = 3840;
        let mut tick = self.visible_start_tick;
        while tick <= self.visible_end_tick {
            let x = self.tick_to_x(tick, bounds.width);
            if x <= bounds.width {
                let stroke_color = if tick.is_multiple_of(bar_ticks) {
                    Color::from_rgba8(0x60, 0x65, 0x70, 0.5)
                } else {
                    Color::from_rgba8(0x40, 0x45, 0x50, 0.25)
                };
                let line = Path::line(Point::new(x, 0.0), Point::new(x, bounds.height));
                frame.stroke(
                    &line,
                    Stroke::default().with_color(stroke_color).with_width(1.0),
                );
            }
            tick += 960;
        }

        // 4. Track Audio/MIDI Clips & Waveform Contours
        for clip in &self.clips {
            if clip.track_index < self.track_count {
                let y = clip.track_index as f32 * lane_height + 2.0;
                let h = (lane_height - 5.0).max(10.0);
                let x_start = self.tick_to_x(clip.start_tick, bounds.width);
                let x_end = self.tick_to_x(clip.start_tick + clip.length_ticks, bounds.width);
                let w = (x_end - x_start).max(4.0);

                if x_start < bounds.width && x_end > 0.0 {
                    // Clip background
                    frame.fill_rectangle(
                        Point::new(x_start, y),
                        Size::new(w, h),
                        Color::from_rgb8(clip.color_rgb[0], clip.color_rgb[1], clip.color_rgb[2]),
                    );

                    // Waveform center line
                    let mid_y = y + h * 0.5;
                    let center_line =
                        Path::line(Point::new(x_start, mid_y), Point::new(x_start + w, mid_y));
                    frame.stroke(
                        &center_line,
                        Stroke::default()
                            .with_color(Color::from_rgba8(255, 255, 255, 0.4))
                            .with_width(1.0),
                    );

                    // Clip outline border
                    let outline_rect = Path::rectangle(Point::new(x_start, y), Size::new(w, h));
                    frame.stroke(
                        &outline_rect,
                        Stroke::default()
                            .with_color(Color::from_rgba8(255, 255, 255, 0.6))
                            .with_width(1.0),
                    );
                }
            }
        }

        // 5. Loop Cycle Range Highlight
        if let Some((start_tick, end_tick)) = self.loop_range {
            let start_x = self.tick_to_x(start_tick, bounds.width);
            let end_x = self.tick_to_x(end_tick, bounds.width);
            if end_x > start_x {
                frame.fill_rectangle(
                    Point::new(start_x, 0.0),
                    Size::new(end_x - start_x, 8.0),
                    Color::from_rgb8(0xf5, 0xa6, 0x23), // Logic Pro cycle orange/yellow
                );
            }
        }

        // 5. Playhead Line
        let playhead_x = self.tick_to_x(self.playhead_tick, bounds.width);
        let playhead_path = Path::line(
            Point::new(playhead_x, 0.0),
            Point::new(playhead_x, bounds.height),
        );
        frame.stroke(
            &playhead_path,
            Stroke::default().with_color(Color::WHITE).with_width(2.0),
        );

        vec![frame.into_geometry()]
    }
}

pub fn view_canvas_timeline<'a>(program: ArrangementCanvasProgram) -> Element<'a, UiMessage> {
    Canvas::new(program)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
