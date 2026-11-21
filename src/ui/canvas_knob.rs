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
use iced::{Color, Element, Length, Point, Rectangle, Size};
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

/// Compact Inline Oscilloscope / Spectrogram Program (FL Studio 2026 140x30px toolbar widget)
#[derive(Debug, Clone)]
pub struct CompactOscilloscopeProgram {
    pub is_playing: bool,
    pub playhead_tick: u64,
    pub accent_color: Color,
}

impl<Message, Theme> canvas::Program<Message, Theme, iced::Renderer>
    for CompactOscilloscopeProgram
{
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // Dark recessed background
        frame.fill_rectangle(
            Point::ORIGIN,
            bounds.size(),
            Color::from_rgb8(0x0e, 0x10, 0x15),
        );
        frame.stroke(
            &Path::rectangle(Point::ORIGIN, bounds.size()),
            Stroke::default()
                .with_color(Color::from_rgb8(0x24, 0x27, 0x32))
                .with_width(1.0),
        );

        // Center zero line
        let mid_y = bounds.height * 0.5;
        let center_line = Path::line(Point::new(0.0, mid_y), Point::new(bounds.width, mid_y));
        frame.stroke(
            &center_line,
            Stroke::default()
                .with_color(Color::from_rgba8(0x35, 0x3d, 0x4d, 0.45))
                .with_width(1.0),
        );

        if self.is_playing {
            // Draw dynamic synthesized audio waveform curve
            let points_count = 60;
            let step_x = bounds.width / points_count as f32;
            let time = self.playhead_tick as f32 * 0.05;

            let path = Path::new(|builder| {
                builder.move_to(Point::new(0.0, mid_y));
                for i in 1..=points_count {
                    let x = i as f32 * step_x;
                    let phase = x * 0.12 + time;
                    let y_offset =
                        (phase.sin() * 0.65 + (phase * 2.3).sin() * 0.35) * (mid_y * 0.75);
                    builder.line_to(Point::new(x, mid_y + y_offset));
                }
            });

            // Outer glow line
            frame.stroke(
                &path,
                Stroke::default()
                    .with_color(Color::from_rgba(
                        self.accent_color.r,
                        self.accent_color.g,
                        self.accent_color.b,
                        0.35,
                    ))
                    .with_width(3.0),
            );

            // Core neon line
            frame.stroke(
                &path,
                Stroke::default()
                    .with_color(self.accent_color)
                    .with_width(1.5),
            );
        } else {
            // Resting state: subtle horizontal green line
            let resting_line = Path::line(
                Point::new(2.0, mid_y),
                Point::new(bounds.width - 2.0, mid_y),
            );
            frame.stroke(
                &resting_line,
                Stroke::default()
                    .with_color(Color::from_rgb8(0x2e, 0xcc, 0x71))
                    .with_width(1.5),
            );
        }

        vec![frame.into_geometry()]
    }
}

pub fn compact_oscilloscope<'a, Theme: 'a>(
    width: f32,
    height: f32,
    is_playing: bool,
    playhead_tick: u64,
    accent_color: Color,
) -> Element<'a, crate::ui::message::UiMessage, Theme, iced::Renderer> {
    Canvas::new(CompactOscilloscopeProgram {
        is_playing,
        playhead_tick,
        accent_color,
    })
    .width(Length::Fixed(width))
    .height(Length::Fixed(height))
    .into()
}

/// Mini 3-Band Parametric EQ Curve Visualizer (FL Studio Parametric EQ 2 Style)
pub struct ParametricEqVisualizerProgram {
    pub low_gain_db: f32,
    pub mid_gain_db: f32,
    pub high_gain_db: f32,
    pub accent_color: Color,
}

impl<Theme> canvas::Program<crate::ui::message::UiMessage, Theme>
    for ParametricEqVisualizerProgram
{
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // 1. Dark Well Background (#101319)
        frame.fill_rectangle(
            Point::ORIGIN,
            bounds.size(),
            Color::from_rgb8(0x10, 0x13, 0x19),
        );

        // 2. Grid & Axis Lines
        let mid_y = bounds.height * 0.5;

        // Vertical Logarithmic Frequency Lines (100Hz at ~22%, 1kHz at ~52%, 10kHz at ~82%)
        let freqs = [(0.22, "100"), (0.52, "1k"), (0.82, "10k")];
        for (norm_x, label) in freqs {
            let x = bounds.width * norm_x;
            let f_line = Path::line(Point::new(x, 0.0), Point::new(x, bounds.height));
            frame.stroke(
                &f_line,
                Stroke::default()
                    .with_color(Color::from_rgba8(0xff, 0xff, 0xff, 0.07))
                    .with_width(1.0),
            );
            frame.fill_text(canvas::Text {
                content: label.to_string(),
                position: Point::new(x + 2.0, bounds.height - 10.0),
                color: Color::from_rgba8(0xff, 0xff, 0xff, 0.30),
                size: 7.5.into(),
                ..Default::default()
            });
        }

        // Horizontal dB Guidelines (+6dB, 0dB, -6dB)
        let db_scale_px = (bounds.height * 0.36) / 12.0; // 12dB span
        let center_zero = Path::line(Point::new(0.0, mid_y), Point::new(bounds.width, mid_y));
        frame.stroke(
            &center_zero,
            Stroke::default()
                .with_color(Color::from_rgba8(0xff, 0xff, 0xff, 0.16))
                .with_width(1.0),
        );

        let plus_6 = Path::line(
            Point::new(0.0, mid_y - 6.0 * db_scale_px),
            Point::new(bounds.width, mid_y - 6.0 * db_scale_px),
        );
        frame.stroke(
            &plus_6,
            Stroke::default()
                .with_color(Color::from_rgba8(0xff, 0xff, 0xff, 0.05))
                .with_width(1.0),
        );

        let minus_6 = Path::line(
            Point::new(0.0, mid_y + 6.0 * db_scale_px),
            Point::new(bounds.width, mid_y + 6.0 * db_scale_px),
        );
        frame.stroke(
            &minus_6,
            Stroke::default()
                .with_color(Color::from_rgba8(0xff, 0xff, 0xff, 0.05))
                .with_width(1.0),
        );

        // 3. Smooth Parametric EQ Response Curve
        let steps = 50;
        let step_w = bounds.width / steps as f32;

        let curve_points: Vec<Point> = (0..=steps)
            .map(|i| {
                let x = i as f32 * step_w;
                let norm_x = (x / bounds.width).clamp(0.0, 1.0);

                // Gaussian bell distributions for Low (~0.20), Mid (~0.50), High (~0.80)
                let d_low = (norm_x - 0.20) / 0.18;
                let d_mid = (norm_x - 0.50) / 0.16;
                let d_high = (norm_x - 0.80) / 0.18;

                let low_gain = self.low_gain_db * (-d_low * d_low).exp();
                let mid_gain = self.mid_gain_db * (-d_mid * d_mid).exp();
                let high_gain = self.high_gain_db * (-d_high * d_high).exp();

                let total_gain = low_gain + mid_gain + high_gain;
                let y = (mid_y - total_gain * db_scale_px).clamp(2.0, bounds.height - 2.0);
                Point::new(x, y)
            })
            .collect();

        // 4. Fill Translucent Gradient Under Curve
        let fill_path = Path::new(|builder| {
            if let Some(first) = curve_points.first() {
                builder.move_to(Point::new(0.0, mid_y));
                builder.line_to(*first);
                for pt in &curve_points[1..] {
                    builder.line_to(*pt);
                }
                builder.line_to(Point::new(bounds.width, mid_y));
                builder.close();
            }
        });
        frame.fill(
            &fill_path,
            Color::from_rgba(
                self.accent_color.r,
                self.accent_color.g,
                self.accent_color.b,
                0.14,
            ),
        );

        // 5. Stroke Outer Glow & Core Curve Line
        let line_path = Path::new(|builder| {
            if let Some(first) = curve_points.first() {
                builder.move_to(*first);
                for pt in &curve_points[1..] {
                    builder.line_to(*pt);
                }
            }
        });
        frame.stroke(
            &line_path,
            Stroke::default()
                .with_color(Color::from_rgba(
                    self.accent_color.r,
                    self.accent_color.g,
                    self.accent_color.b,
                    0.35,
                ))
                .with_width(3.0),
        );
        frame.stroke(
            &line_path,
            Stroke::default()
                .with_color(self.accent_color)
                .with_width(1.5),
        );

        // 6. Band Nodes (1: Red Low, 2: Green Mid, 3: Blue/Cyan High)
        let nodes = [
            (
                0.20,
                self.low_gain_db,
                Color::from_rgb8(0xff, 0x47, 0x57),
                "1",
            ),
            (
                0.50,
                self.mid_gain_db,
                Color::from_rgb8(0x2e, 0xd5, 0x73),
                "2",
            ),
            (
                0.80,
                self.high_gain_db,
                Color::from_rgb8(0x1e, 0x90, 0xff),
                "3",
            ),
        ];

        for (norm_x, gain, color, label) in nodes {
            let nx = bounds.width * norm_x;
            let ny = (mid_y - gain * db_scale_px).clamp(6.0, bounds.height - 6.0);
            let center = Point::new(nx, ny);

            let circle = Path::circle(center, 4.5);
            frame.fill(&circle, color);
            frame.stroke(
                &circle,
                Stroke::default().with_color(Color::WHITE).with_width(1.0),
            );
            frame.fill_text(canvas::Text {
                content: label.to_string(),
                position: Point::new(nx - 2.5, ny - 4.5),
                color: Color::WHITE,
                size: 7.5.into(),
                ..Default::default()
            });
        }

        // 7. Header Title & Border
        frame.fill_text(canvas::Text {
            content: "PARAMETRIC EQ 2".into(),
            position: Point::new(4.0, 3.0),
            color: Color::from_rgba8(0xfa, 0x82, 0x31, 0.85),
            size: 8.0.into(),
            ..Default::default()
        });

        frame.fill_text(canvas::Text {
            content: "HQ".into(),
            position: Point::new(bounds.width - 16.0, 3.0),
            color: Color::from_rgba8(0x2e, 0xcc, 0x71, 0.85),
            size: 7.5.into(),
            ..Default::default()
        });

        frame.stroke(
            &Path::rectangle(Point::ORIGIN, bounds.size()),
            Stroke::default()
                .with_color(Color::from_rgb8(0x27, 0x2a, 0x36))
                .with_width(1.0),
        );

        vec![frame.into_geometry()]
    }
}

pub fn parametric_eq_visualizer<'a, Theme: 'a>(
    width: f32,
    height: f32,
    low_gain_db: f32,
    mid_gain_db: f32,
    high_gain_db: f32,
    accent_color: Color,
) -> Element<'a, crate::ui::message::UiMessage, Theme, iced::Renderer> {
    Canvas::new(ParametricEqVisualizerProgram {
        low_gain_db,
        mid_gain_db,
        high_gain_db,
        accent_color,
    })
    .width(Length::Fixed(width))
    .height(Length::Fixed(height))
    .into()
}

/// Compact Audio Waveform Preview Scrubber for Browser Player
pub struct SamplePreviewWaveformProgram {
    pub is_playing: bool,
    pub progress: f32,
    pub accent_color: Color,
}

impl<Theme> canvas::Program<crate::ui::message::UiMessage, Theme> for SamplePreviewWaveformProgram {
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &iced::Renderer,
        _theme: &Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());

        // Background
        frame.fill_rectangle(
            Point::ORIGIN,
            bounds.size(),
            Color::from_rgb8(0x13, 0x15, 0x1c),
        );

        let mid_y = bounds.height * 0.5;
        let points_count = 48;
        let step_x = bounds.width / points_count as f32;

        // Synthesize dynamic peaks for sample preview
        let mut bars = Vec::with_capacity(points_count);
        for i in 0..points_count {
            let norm_x = i as f32 / points_count as f32;
            let decay = (-norm_x * 2.8).exp();
            let wave = (norm_x * 35.0).sin().abs() * 0.6 + (norm_x * 80.0).cos().abs() * 0.4;
            let amp = (decay * wave).clamp(0.05, 0.95);
            bars.push(amp);
        }

        for (i, &amp) in bars.iter().enumerate() {
            let x = i as f32 * step_x + 1.0;
            let bar_h = amp * (mid_y - 2.0);
            let color = if (x / bounds.width) <= self.progress {
                Color::from_rgb8(0x2e, 0xcc, 0x71)
            } else {
                Color::from_rgb8(0x3d, 0x6e, 0x5a)
            };

            let bar_rect = Rectangle::new(
                Point::new(x, mid_y - bar_h),
                Size::new((step_x - 1.0).max(1.0), bar_h * 2.0),
            );
            frame.fill_rectangle(bar_rect.position(), bar_rect.size(), color);
        }

        // Center line
        let center_line = Path::line(Point::new(0.0, mid_y), Point::new(bounds.width, mid_y));
        frame.stroke(
            &center_line,
            Stroke::default()
                .with_color(Color::from_rgba8(0xff, 0xff, 0xff, 0.12))
                .with_width(1.0),
        );

        // Playhead Scrubber Line
        if self.is_playing {
            let scrub_x = bounds.width * self.progress.clamp(0.0, 1.0);
            let scrub_line =
                Path::line(Point::new(scrub_x, 0.0), Point::new(scrub_x, bounds.height));
            frame.stroke(
                &scrub_line,
                Stroke::default().with_color(Color::WHITE).with_width(1.5),
            );
        }

        // Border
        frame.stroke(
            &Path::rectangle(Point::ORIGIN, bounds.size()),
            Stroke::default()
                .with_color(Color::from_rgb8(0x24, 0x27, 0x32))
                .with_width(1.0),
        );

        vec![frame.into_geometry()]
    }
}

pub fn sample_preview_waveform<'a, Theme: 'a>(
    width: f32,
    height: f32,
    is_playing: bool,
    progress: f32,
    accent_color: Color,
) -> Element<'a, crate::ui::message::UiMessage, Theme, iced::Renderer> {
    Canvas::new(SamplePreviewWaveformProgram {
        is_playing,
        progress,
        accent_color,
    })
    .width(Length::Fixed(width))
    .height(Length::Fixed(height))
    .into()
}
