use std::collections::HashMap;
use std::sync::Arc;

use vulkano::command_buffer::allocator::StandardCommandBufferAllocator;
use vulkano::device::{Device, Queue};
use vulkano::image::sampler::{Filter, Sampler, SamplerCreateInfo};
use vulkano::memory::allocator::StandardMemoryAllocator;

use super::render_font::{Contour, Font};
use super::render_texture::Texture;

const ATLAS_SIZE: u32 = 512;
const SUPERSAMPLE: u32 = 4;
const PADDING: u32 = 2;
const RGBA_CHANNELS: usize = 4;
const COVERAGE_DIVISOR: u32 = SUPERSAMPLE * SUPERSAMPLE;
const WHITE: u8 = 255;
const QUAD_STEPS: u32 = 12;
const FIRST_ASCII: u32 = 32;
const LAST_ASCII: u32 = 126;
const UNKNOWN_GLYPH: char = '?';
const ZERO: f32 = 0.0;

pub struct GlyphPlacement {
    pub uv: [f32; 4],
    pub advance_px: f32,
    pub left_px: f32,
    pub top_px: f32,
    pub width_px: u32,
    pub height_px: u32,
}

pub struct GlyphAtlas {
    pub texture: Texture,
    pub sampler: Arc<Sampler>,
    glyphs: HashMap<char, GlyphPlacement>,
}

impl GlyphAtlas {
    pub fn new(
        device: Arc<Device>,
        queue: Arc<Queue>,
        allocator: Arc<StandardMemoryAllocator>,
        command_allocator: &Arc<StandardCommandBufferAllocator>,
        font: &Font,
        pixel_size: u32,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (buffer, placements) = build_atlas(font, pixel_size);
        let texture = Texture::from_rgba(
            device.clone(),
            queue,
            allocator,
            command_allocator,
            ATLAS_SIZE,
            ATLAS_SIZE,
            &buffer,
        )?;
        let sampler = Sampler::new(
            device,
            SamplerCreateInfo {
                mag_filter: Filter::Linear,
                min_filter: Filter::Linear,
                ..Default::default()
            },
        )?;
        Ok(Self {
            texture,
            sampler,
            glyphs: placements.into_iter().collect(),
        })
    }

    pub fn glyph(&self, ch: char) -> Option<&GlyphPlacement> {
        self.glyphs
            .get(&ch)
            .or_else(|| self.glyphs.get(&UNKNOWN_GLYPH))
    }
}

struct GlyphBitmap {
    pixels: Vec<u8>,
    width: u32,
    height: u32,
    left_px: f32,
    top_px: f32,
    advance_px: f32,
}

fn build_atlas(font: &Font, pixel_size: u32) -> (Vec<u8>, Vec<(char, GlyphPlacement)>) {
    let mut buffer = vec![0u8; ATLAS_SIZE as usize * ATLAS_SIZE as usize * RGBA_CHANNELS];
    let mut placements = Vec::new();
    let mut cursor_x = 0u32;
    let mut cursor_y = 0u32;
    let mut row_height = 0u32;
    for code in FIRST_ASCII..=LAST_ASCII {
        let ch = char::from_u32(code).expect("ascii code point");
        let Some(bitmap) = rasterize_glyph(font, ch, pixel_size) else {
            continue;
        };
        let (width, height) = (bitmap.width, bitmap.height);
        if width > 0 && height > 0 {
            if cursor_x + width + PADDING > ATLAS_SIZE {
                cursor_x = 0;
                cursor_y += row_height + PADDING;
                row_height = 0;
            }
            if cursor_y + height > ATLAS_SIZE {
                break;
            }
            copy_bitmap_into(&mut buffer, &bitmap.pixels, cursor_x, cursor_y, width, height);
            placements.push((
                ch,
                GlyphPlacement {
                    uv: [
                        cursor_x as f32 / ATLAS_SIZE as f32,
                        cursor_y as f32 / ATLAS_SIZE as f32,
                        (cursor_x + width) as f32 / ATLAS_SIZE as f32,
                        (cursor_y + height) as f32 / ATLAS_SIZE as f32,
                    ],
                    advance_px: bitmap.advance_px,
                    left_px: bitmap.left_px,
                    top_px: bitmap.top_px,
                    width_px: width,
                    height_px: height,
                },
            ));
            cursor_x += width + PADDING;
            row_height = row_height.max(height);
        } else {
            placements.push((
                ch,
                GlyphPlacement {
                    uv: [ZERO, ZERO, ZERO, ZERO],
                    advance_px: bitmap.advance_px,
                    left_px: ZERO,
                    top_px: ZERO,
                    width_px: 0,
                    height_px: 0,
                },
            ));
        }
    }
    (buffer, placements)
}

fn copy_bitmap_into(
    buffer: &mut [u8],
    pixels: &[u8],
    dest_x: u32,
    dest_y: u32,
    width: u32,
    height: u32,
) {
    for row in 0..height {
        let source_start = (row as usize) * width as usize * RGBA_CHANNELS;
        let source_end = source_start + width as usize * RGBA_CHANNELS;
        let destination_start =
            ((dest_y + row) as usize * ATLAS_SIZE as usize + dest_x as usize) * RGBA_CHANNELS;
        buffer[destination_start..destination_start + (source_end - source_start)]
            .copy_from_slice(&pixels[source_start..source_end]);
    }
}

fn rasterize_glyph(font: &Font, ch: char, pixel_size: u32) -> Option<GlyphBitmap> {
    let glyph_id = font.glyph_index(ch)?;
    let scale = pixel_size as f32 / font.units_per_em();
    let advance_px = font.advance_width(glyph_id) * scale;
    if ch.is_whitespace() {
        return Some(empty_bitmap(advance_px));
    }

    let mut polygons = Vec::new();
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    if let Some(contours) = font.glyph_outline(glyph_id) {
        for contour in contours {
            let mut polygon = Vec::new();
            flatten_contour(&contour, scale, &mut polygon);
            if polygon.is_empty() {
                continue;
            }
            for point in &polygon {
                min_x = min_x.min(point[0]);
                max_x = max_x.max(point[0]);
                min_y = min_y.min(point[1]);
                max_y = max_y.max(point[1]);
            }
            polygons.push(polygon);
        }
    }

    if polygons.is_empty() {
        return Some(empty_bitmap(advance_px));
    }

    let min_x_i = min_x.floor() as i32;
    let min_y_i = min_y.floor() as i32;
    let max_x_i = max_x.ceil() as i32;
    let max_y_i = max_y.ceil() as i32;
    let width = (max_x_i - min_x_i) as u32;
    let height = (max_y_i - min_y_i) as u32;
    if width == 0 || height == 0 {
        return Some(empty_bitmap(advance_px));
    }

    let mut pixels = vec![0u8; width as usize * height as usize * RGBA_CHANNELS];
    let samples = SUPERSAMPLE as f32;
    for py in 0..height {
        for px in 0..width {
            let mut coverage = 0u32;
            for sy in 0..SUPERSAMPLE {
                for sx in 0..SUPERSAMPLE {
                    let sample_x = min_x_i as f32 + px as f32 + (sx as f32 + 0.5) / samples;
                    let sample_y = max_y_i as f32 - (py as f32 + (sy as f32 + 0.5) / samples);
                    if point_inside(&polygons, sample_x, sample_y) {
                        coverage += 1;
                    }
                }
            }
            let alpha = (coverage * 255 / COVERAGE_DIVISOR) as u8;
            if alpha > 0 {
                let index = (py * width + px) as usize * RGBA_CHANNELS;
                pixels[index] = WHITE;
                pixels[index + 1] = WHITE;
                pixels[index + 2] = WHITE;
                pixels[index + 3] = alpha;
            }
        }
    }

    Some(GlyphBitmap {
        pixels,
        width,
        height,
        left_px: min_x_i as f32,
        top_px: max_y_i as f32,
        advance_px,
    })
}

fn empty_bitmap(advance_px: f32) -> GlyphBitmap {
    GlyphBitmap {
        pixels: Vec::new(),
        width: 0,
        height: 0,
        left_px: ZERO,
        top_px: ZERO,
        advance_px,
    }
}

fn flatten_contour(contour: &Contour, scale: f32, output: &mut Vec<[f32; 2]>) {
    let points = &contour.points;
    let count = points.len();
    if count < 2 {
        return;
    }
    let first_on = (0..count).find(|&index| points[index].on_curve);
    let Some(first_on) = first_on else {
        return;
    };
    let mut scaled = Vec::with_capacity(count + 1);
    for index in 0..count {
        let point = &points[(first_on + index) % count];
        scaled.push((
            point.x as f32 * scale,
            point.y as f32 * scale,
            point.on_curve,
        ));
    }
    let start = [scaled[0].0, scaled[0].1];
    scaled.push((start[0], start[1], true));
    let mut prev_on: [f32; 2] = start;
    let mut pending_off: Option<[f32; 2]> = None;
    for index in 1..scaled.len() {
        let point = scaled[index];
        let position = [point.0, point.1];
        if point.2 {
            if let Some(control) = pending_off {
                flatten_quadratic(prev_on, control, position, output);
                pending_off = None;
            } else {
                output.push(position);
            }
            prev_on = position;
        } else if let Some(previous_control) = pending_off {
            let midpoint = [
                (previous_control[0] + position[0]) / 2.0,
                (previous_control[1] + position[1]) / 2.0,
            ];
            flatten_quadratic(prev_on, previous_control, midpoint, output);
            prev_on = midpoint;
            pending_off = Some(position);
        } else {
            pending_off = Some(position);
        }
    }
}

fn flatten_quadratic(
    start: [f32; 2],
    control: [f32; 2],
    end: [f32; 2],
    output: &mut Vec<[f32; 2]>,
) {
    for step in 1..=QUAD_STEPS {
        let t = step as f32 / QUAD_STEPS as f32;
        let inverse = 1.0 - t;
        output.push([
            inverse * inverse * start[0] + 2.0 * inverse * t * control[0] + t * t * end[0],
            inverse * inverse * start[1] + 2.0 * inverse * t * control[1] + t * t * end[1],
        ]);
    }
}

fn point_inside(polygons: &[Vec<[f32; 2]>], x: f32, y: f32) -> bool {
    let mut contained = 0usize;
    for polygon in polygons {
        let count = polygon.len();
        let mut inside = false;
        for index in 0..count {
            let current = polygon[index];
            let next = polygon[(index + 1) % count];
            if (current[1] > y) != (next[1] > y) {
                let intersection =
                    current[0] + (y - current[1]) * (next[0] - current[0]) / (next[1] - current[1]);
                if x < intersection {
                    inside = !inside;
                }
            }
        }
        if inside {
            contained += 1;
        }
    }
    contained % 2 == 1
}

#[cfg(test)]
mod tests {
    use super::{rasterize_glyph, Font};

    #[test]
    fn rasterizes_letter_a() {
        let font = Font::embedded();
        let bitmap = rasterize_glyph(&font, 'A', 20).expect("glyph A rasterizes");
        assert!(bitmap.width > 0);
        assert!(bitmap.height > 0);
        assert!(bitmap.advance_px > 0.0);
        assert!(bitmap.pixels.iter().any(|&byte| byte > 0));
    }

    #[test]
    fn rasterizes_space_as_empty() {
        let font = Font::embedded();
        let bitmap = rasterize_glyph(&font, ' ', 20).expect("glyph space handles empty");
        assert_eq!(bitmap.width, 0);
        assert_eq!(bitmap.height, 0);
        assert!(bitmap.advance_px > 0.0);
    }
}
