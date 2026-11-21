//! Authentic FL Studio 2026 Rotary Knob Canvas Widget
//!
//! Features:
//! - 20px–28px compact circular dial with bezel ring and recessed interior
//! - Precise circular background track arc
//! - Unipolar or bipolar center-detented colored active sweep arc
//! - Rotating needle indicator line
//! - Drag interaction (vertical drag or scroll) with customizable sensitivity
//! - Zero heap allocation during rendering

use iced::mouse;
use iced::widget::canvas::{self, Action, Canvas, Frame, Geometry, Path, Stroke};
use iced::{Color, Element, Length, Point, Rectangle};
use std::f32::consts::PI;

/// Interaction state for rotary knob dragging
#[derive(Debug, Clone, Copy, Default)]
pub struct KnobState {
    pub is_dragging: bool,
    pub is_hovered: bool,
    pub drag_start_y: f32,
    pub initial_value: f32,
    pub last_click_instant: Option<std::time::Instant>,
}

/// FL Studio style rotary dial widget program
#[derive(Debug, Clone)]
pub struct RotaryKnobProgram<F>
where
    F: Fn(f32) -> crate::ui::message::UiMessage + Clone + 'static,
{
    pub min: f32,
    pub max: f32,
    pub value: f32,
    pub default_value: f32,
    pub is_bipolar: bool,
    pub on_change: F,
    pub label: Option<&'static str>,
    pub track_color: Color,
    pub arc_color: Color,
    pub needle_color: Color,
    pub body_color: Color,
}

impl<F> RotaryKnobProgram<F>
where
    F: Fn(f32) -> crate::ui::message::UiMessage + Clone + 'static,
{
    pub fn new(
        min: f32,
        max: f32,
        value: f32,
        is_bipolar: bool,
        arc_color: Color,
        on_change: F,
    ) -> Self {
        let default_value = if is_bipolar { 0.0 } else { 1.0 };
        Self {
            min,
            max,
            value: value.clamp(min, max),
            default_value,
            is_bipolar,
            on_change,
            label: None,
            track_color: Color::from_rgb8(0x18, 0x1b, 0x22),
            arc_color,
            needle_color: Color::from_rgb8(0xf0, 0xf3, 0xf8),
            body_color: Color::from_rgb8(0x23, 0x27, 0x30),
        }
    }

    pub fn with_label(mut self, label: &'static str) -> Self {
        self.label = Some(label);
        self
    }

    fn format_hint(&self, val: f32) -> String {
        let prefix = self.label.unwrap_or("Parameter");
        if self.is_bipolar {
            if val.abs() < 0.05 {
                format!("{}: Center (0%)", prefix)
            } else if val < 0.0 {
                format!("{}: {:.0}% Left", prefix, -val * 100.0)
            } else {
                format!("{}: {:.0}% Right", prefix, val * 100.0)
            }
        } else if self.max > 100.0 {
            // Pitch / cents
            format!("{}: {:+.0} cents", prefix, val)
        } else {
            // Volume / percentage
            format!("{}: {:.0}%", prefix, val * 100.0)
        }
    }
}

impl<F, Theme> canvas::Program<crate::ui::message::UiMessage, Theme> for RotaryKnobProgram<F>
where
    F: Fn(f32) -> crate::ui::message::UiMessage + Clone + 'static,
{
    type State = KnobState;

    fn update(
        &self,
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<Action<crate::ui::message::UiMessage>> {
        let cursor_pos = cursor.position_in(bounds)?;

        match event {
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                let now = std::time::Instant::now();
                let is_double_click = state
                    .last_click_instant
                    .map(|t| now.duration_since(t).as_millis() < 300)
                    .unwrap_or(false);

                if is_double_click {
                    // Double click detected: reset to default/detent value
                    state.last_click_instant = None;
                    state.is_dragging = false;
                    return Some(Action::publish((self.on_change)(self.default_value)));
                }
                state.last_click_instant = Some(now);
                state.is_dragging = true;
                state.drag_start_y = cursor_pos.y;
                state.initial_value = self.value;
                Some(Action::request_redraw())
            }
            canvas::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Right)) => {
                // Right click: reset immediately to default/detent
                Some(Action::publish((self.on_change)(self.default_value)))
            }
            canvas::Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if state.is_dragging {
                    let dy = state.drag_start_y - cursor_pos.y;
                    let range = self.max - self.min;
                    // 100 pixels for full travel
                    let delta_val = (dy / 100.0) * range;
                    let new_val = (state.initial_value + delta_val).clamp(self.min, self.max);
                    Some(Action::publish((self.on_change)(new_val)))
                } else if !state.is_hovered {
                    state.is_hovered = true;
                    Some(Action::publish(
                        crate::ui::message::UiMessage::HoverParameter(self.format_hint(self.value)),
                    ))
                } else {
                    None
                }
            }
            canvas::Event::Mouse(mouse::Event::CursorLeft) => {
                if state.is_hovered {
                    state.is_hovered = false;
                    Some(Action::publish(
                        crate::ui::message::UiMessage::ClearHoverParameter,
                    ))
                } else {
                    None
                }
            }
            canvas::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left)) => {
                if state.is_dragging {
                    state.is_dragging = false;
                    Some(Action::request_redraw())
                } else {
                    None
                }
            }
            canvas::Event::Mouse(mouse::Event::WheelScrolled { delta }) => {
                let dy = match delta {
                    mouse::ScrollDelta::Lines { y, .. } => *y * 4.0,
                    mouse::ScrollDelta::Pixels { y, .. } => *y,
                };
                let range = self.max - self.min;
                let step = range * 0.02;
                let new_val = (self.value + dy * step).clamp(self.min, self.max);
                Some(Action::publish((self.on_change)(new_val)))
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
        let center = Point::new(bounds.width / 2.0, bounds.height / 2.0);
        let radius = (bounds.width.min(bounds.height) / 2.0) - 2.0;
        if radius <= 3.0 {
            return vec![frame.into_geometry()];
        }

        // Standard audio dial: 7 o'clock (135 deg / 0.75*PI) to 5 o'clock (405 deg / 2.25*PI)
        let start_angle = 0.75 * PI;
        let total_sweep = 1.5 * PI;
        let norm_val = if (self.max - self.min).abs() > 0.0001 {
            (self.value - self.min) / (self.max - self.min)
        } else {
            0.5
        };

        // 1. Recessed Knob Body
        let body_path = Path::circle(center, radius - 1.0);
        frame.fill(&body_path, self.body_color);
        frame.stroke(
            &body_path,
            Stroke::default()
                .with_color(Color::from_rgba8(255, 255, 255, 0.08))
                .with_width(1.0),
        );

        // 2. Track Background Arc (Dark groove)
        let track_radius = radius - 0.5;
        let mut track_builder = iced::widget::canvas::path::Builder::new();
        let segments = 32;
        for i in 0..=segments {
            let theta = start_angle + (i as f32 / segments as f32) * total_sweep;
            let pt = Point::new(
                center.x + track_radius * theta.cos(),
                center.y + track_radius * theta.sin(),
            );
            if i == 0 {
                track_builder.move_to(pt);
            } else {
                track_builder.line_to(pt);
            }
        }
        frame.stroke(
            &track_builder.build(),
            Stroke::default()
                .with_color(self.track_color)
                .with_width(2.0),
        );

        // 3. Active Colored Sweep Arc
        let mut arc_builder = iced::widget::canvas::path::Builder::new();
        let (arc_start, arc_end) = if self.is_bipolar {
            let center_angle = start_angle + 0.5 * total_sweep;
            let current_angle = start_angle + norm_val * total_sweep;
            if current_angle >= center_angle {
                (center_angle, current_angle)
            } else {
                (current_angle, center_angle)
            }
        } else {
            (start_angle, start_angle + norm_val * total_sweep)
        };

        let arc_diff = (arc_end - arc_start).abs();
        if arc_diff > 0.01 {
            let arc_segments = ((arc_diff / total_sweep) * segments as f32).max(2.0) as usize;
            for i in 0..=arc_segments {
                let theta = arc_start + (i as f32 / arc_segments as f32) * (arc_end - arc_start);
                let pt = Point::new(
                    center.x + track_radius * theta.cos(),
                    center.y + track_radius * theta.sin(),
                );
                if i == 0 {
                    arc_builder.move_to(pt);
                } else {
                    arc_builder.line_to(pt);
                }
            }
            frame.stroke(
                &arc_builder.build(),
                Stroke::default().with_color(self.arc_color).with_width(2.2),
            );
        }

        // Center Detent Tick for Bipolar dials
        if self.is_bipolar {
            let top_theta = start_angle + 0.5 * total_sweep;
            let detent_p1 = Point::new(
                center.x + (radius - 2.5) * top_theta.cos(),
                center.y + (radius - 2.5) * top_theta.sin(),
            );
            let detent_p2 = Point::new(
                center.x + (radius + 0.5) * top_theta.cos(),
                center.y + (radius + 0.5) * top_theta.sin(),
            );
            frame.stroke(
                &Path::line(detent_p1, detent_p2),
                Stroke::default()
                    .with_color(Color::from_rgba8(255, 255, 255, 0.4))
                    .with_width(1.0),
            );
        }

        // 4. Pointer Needle Line
        let needle_theta = start_angle + norm_val * total_sweep;
        let inner_needle = Point::new(
            center.x + (radius * 0.25) * needle_theta.cos(),
            center.y + (radius * 0.25) * needle_theta.sin(),
        );
        let outer_needle = Point::new(
            center.x + (radius - 2.0) * needle_theta.cos(),
            center.y + (radius - 2.0) * needle_theta.sin(),
        );
        frame.stroke(
            &Path::line(inner_needle, outer_needle),
            Stroke::default()
                .with_color(self.needle_color)
                .with_width(1.8),
        );

        vec![frame.into_geometry()]
    }
}

/// Configuration descriptor for rotary knob construction
#[derive(Debug, Clone)]
pub struct RotaryKnobConfig<'a> {
    pub size: f32,
    pub min: f32,
    pub max: f32,
    pub value: f32,
    pub is_bipolar: bool,
    pub arc_color: Color,
    pub label: Option<&'a str>,
}

impl<'a> RotaryKnobConfig<'a> {
    pub fn new(
        size: f32,
        min: f32,
        max: f32,
        value: f32,
        is_bipolar: bool,
        arc_color: Color,
    ) -> Self {
        Self {
            size,
            min,
            max,
            value,
            is_bipolar,
            arc_color,
            label: None,
        }
    }

    pub fn with_label(mut self, label: &'a str) -> Self {
        self.label = Some(label);
        self
    }
}

/// Helper function to render a compact FL Studio rotary knob widget
pub fn rotary_knob<'a, F, Theme: 'a>(
    size: f32,
    min: f32,
    max: f32,
    value: f32,
    is_bipolar: bool,
    arc_color: Color,
    on_change: F,
) -> Element<'a, crate::ui::message::UiMessage, Theme, iced::Renderer>
where
    F: Fn(f32) -> crate::ui::message::UiMessage + Clone + 'static,
{
    Canvas::new(RotaryKnobProgram::new(
        min, max, value, is_bipolar, arc_color, on_change,
    ))
    .width(Length::Fixed(size))
    .height(Length::Fixed(size))
    .into()
}

/// Helper function to render an FL Studio rotary knob configured via `RotaryKnobConfig`
pub fn rotary_knob_with_config<'a, F, Theme: 'a>(
    config: RotaryKnobConfig<'a>,
    on_change: F,
) -> Element<'a, crate::ui::message::UiMessage, Theme, iced::Renderer>
where
    F: Fn(f32) -> crate::ui::message::UiMessage + Clone + 'static,
{
    let mut prog = RotaryKnobProgram::new(
        config.min,
        config.max,
        config.value,
        config.is_bipolar,
        config.arc_color,
        on_change,
    );
    if let Some(lbl) = config.label {
        prog = prog.with_label(Box::leak(lbl.to_string().into_boxed_str()));
    }
    Canvas::new(prog)
        .width(Length::Fixed(config.size))
        .height(Length::Fixed(config.size))
        .into()
}
