use std::time::Duration;

use gpui::{
    Animation, AnimationElement, AnimationExt, ElementId, IntoElement, Pixels, Point, SharedString,
    Styled, TextRun, Window, ease_out_quint, font, px,
};

pub const DEFAULT_BLINK_MS: u64 = 1000;
pub const DEFAULT_MOVE_MS: u64 = 150;
pub const DEFAULT_TRAIL_LEN: usize = 4;
pub const BLINK_ON_FRACTION: f32 = 0.5;
pub const FADE_MIN_OPACITY: f32 = 0.15;
pub const TRAIL_MAX_OPACITY: f32 = 0.35;
pub const BLOCK_CARET_OPACITY: f32 = 0.35;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaretShape {
    Line,
    Block,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlinkPreset {
    Blink,
    Fade,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MovePreset {
    Teleport,
    EaseOut,
}

#[derive(Clone, Debug)]
pub struct CaretAnimationConfig {
    pub blink_ms: u64,
    pub blink_preset: BlinkPreset,
    pub move_ms: u64,
    pub move_preset: MovePreset,
    pub trail_len: usize,
}

impl CaretAnimationConfig {
    pub fn new() -> Self {
        Self {
            blink_ms: DEFAULT_BLINK_MS,
            blink_preset: BlinkPreset::Blink,
            move_ms: DEFAULT_MOVE_MS,
            move_preset: MovePreset::EaseOut,
            trail_len: DEFAULT_TRAIL_LEN,
        }
    }

    pub fn blink_ms(mut self, ms: u64) -> Self {
        self.blink_ms = ms;
        self
    }

    pub fn blink_preset(mut self, preset: BlinkPreset) -> Self {
        self.blink_preset = preset;
        self
    }

    pub fn move_ms(mut self, ms: u64) -> Self {
        self.move_ms = ms;
        self
    }

    pub fn move_preset(mut self, preset: MovePreset) -> Self {
        self.move_preset = preset;
        self
    }

    pub fn trail_len(mut self, len: usize) -> Self {
        self.trail_len = len;
        self
    }
}

impl Default for CaretAnimationConfig {
    fn default() -> Self {
        Self::new()
    }
}

pub fn with_blink_animation<E: IntoElement + Styled + 'static>(
    element: E,
    id: impl Into<ElementId>,
    preset: BlinkPreset,
    ms: u64,
) -> AnimationElement<E> {
    element.with_animation(
        id,
        Animation::new(Duration::from_millis(ms)).repeat(),
        move |e, delta| {
            let opacity = match preset {
                BlinkPreset::Blink => {
                    if delta < BLINK_ON_FRACTION {
                        1.0
                    } else {
                        0.0
                    }
                }
                BlinkPreset::Fade => FADE_MIN_OPACITY + (1.0 - FADE_MIN_OPACITY) * delta,
            };
            e.opacity(opacity)
        },
    )
}

pub fn with_move_animation<E: IntoElement + Styled + 'static>(
    element: E,
    id: impl Into<ElementId>,
    from: Point<Pixels>,
    to: Point<Pixels>,
    ms: u64,
) -> AnimationElement<E> {
    element.with_animation(
        id,
        Animation::new(Duration::from_millis(ms)).with_easing(ease_out_quint()),
        move |e, delta| e.left(lerp(from.x, to.x, delta)).top(lerp(from.y, to.y, delta)),
    )
}

pub fn lerp(from: Pixels, to: Pixels, t: f32) -> Pixels {
    px(f32::from(from) + (f32::from(to) - f32::from(from)) * t)
}

pub fn block_width_for_char(
    window: &mut Window,
    ch: Option<char>,
    text_size: Pixels,
    family: Option<SharedString>,
    fallback: Pixels,
) -> Pixels {
    let Some(ch) = ch else {
        return fallback;
    };
    if ch == '\n' {
        return fallback;
    }
    let mut s = String::new();
    s.push(ch);
    let text: gpui::SharedString = s.into();
    let run = vec![TextRun {
        len: text.len(),
        font: font(family.unwrap_or_default()),
        color: gpui::Hsla::white(),
        background_color: None,
        underline: None,
        strikethrough: None,
    }];
    let width = window
        .text_system()
        .shape_line(text, text_size, &run, None)
        .width;
    if f32::from(width) <= 0.0 {
        return fallback;
    }
    px(f32::from(width).ceil())
}
