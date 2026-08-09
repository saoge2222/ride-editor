use std::fs;
use std::path::Path;

use super::render_font::Font;

pub struct FontCollection {
    pub fonts: Vec<Font>,
}

impl FontCollection {
    pub fn from_bytes(data: Vec<u8>) -> Result<Self, String> {
        if data.len() < 16 {
            return Err("TTC data too short".into());
        }
        let tag = super::render_font::u32_be(&data, 0);
        if tag != 0x74746366 {
            return Err("not a TTC collection".into());
        }
        let num_fonts = super::render_font::u32_be(&data, 8) as usize;
        if num_fonts == 0 {
            return Err("empty TTC".into());
        }
        let offsets_offset = 12;
        let mut fonts = Vec::with_capacity(num_fonts);
        for i in 0..num_fonts {
            let font_offset =
                super::render_font::u32_be(&data, offsets_offset + i * 4) as usize;
            if font_offset >= data.len() {
                return Err("TTC font offset out of bounds".into());
            }
            let font = Font::from_bytes_at(&data, font_offset)?;
            fonts.push(font);
        }
        Ok(Self { fonts })
    }

    pub fn from_path(path: &Path) -> Result<Self, String> {
        let data = fs::read(path).map_err(|e| e.to_string())?;
        Self::from_bytes(data)
    }

    pub fn first_face(&self) -> Option<&Font> {
        self.fonts.first()
    }

    pub fn face_count(&self) -> usize {
        self.fonts.len()
    }

    pub fn into_first_face(mut self) -> Option<Font> {
        if self.fonts.is_empty() {
            None
        } else {
            Some(self.fonts.swap_remove(0))
        }
    }
}
