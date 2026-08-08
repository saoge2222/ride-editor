use std::fs;
use std::path::Path;

const TAG_HEAD: [u8; 4] = *b"head";
const TAG_HHEA: [u8; 4] = *b"hhea";
const TAG_HMTX: [u8; 4] = *b"hmtx";
const TAG_CMAP: [u8; 4] = *b"cmap";
const TAG_LOCA: [u8; 4] = *b"loca";
const TAG_GLYF: [u8; 4] = *b"glyf";
const TAG_NAME: [u8; 4] = *b"name";

const SFNT_VERSION: u32 = 0x0001_0000;
const TABLE_DIRECTORY_OFFSET: usize = 12;
const DIRECTORY_ENTRY_SIZE: usize = 16;
const OFFSET_NUM_TABLES: usize = 4;

const HEAD_UNITS_PER_EM: usize = 18;
const HEAD_INDEX_TO_LOC: usize = 50;
const HHEA_ASCENDER: usize = 4;
const HHEA_DESCENDER: usize = 6;
const HHEA_NUM_H_METRICS: usize = 34;

const CMAP_WINDOWS_PLATFORM: u16 = 3;
const CMAP_WINDOWS_ENCODING: u16 = 1;
const CMAP_FORMAT4: u16 = 4;
const CMAP_NUM_SUBTABLES_OFFSET: usize = 2;
const CMAP_RECORD_SIZE: usize = 8;
const CMAP_SUBTABLE_OFFSET: usize = 4;
const CMAP_FORMAT_OFFSET: usize = 0;
const CMAP_SEGMENT_COUNT_X2: usize = 2;
const CMAP_END_CODE_OFFSET: usize = 14;
const CMAP_RESERVED_PAD: usize = 2;

const NAME_RECORD_SIZE: usize = 12;
const NAME_COUNT_OFFSET: usize = 2;
const NAME_STRING_OFFSET: usize = 4;
const NAME_RECORDS_OFFSET: usize = 6;
const NAME_WINDOWS_PLATFORM: u16 = 3;
const NAME_WINDOWS_ENCODING: u16 = 1;
const NAME_LANGUAGE_ENGLISH: u16 = 0x0409;
const NAME_ID_FAMILY: u16 = 1;

const FLAG_ON_CURVE: u8 = 0x01;
const FLAG_X_SHORT: u8 = 0x02;
const FLAG_Y_SHORT: u8 = 0x04;
const FLAG_REPEAT: u8 = 0x08;
const FLAG_X_SAME: u8 = 0x10;
const FLAG_Y_SAME: u8 = 0x20;

const COORD_OFFSET: usize = 2;
const END_POINTS_OFFSET: usize = 2;
const NUM_CONTOURS_OFFSET: usize = 0;
const INSTRUCTION_LENGTH_OFFSET: usize = 2;
const HMTX_ENTRY_SIZE: usize = 4;
const GLYPH_BBOX_BYTES: usize = 8;

pub struct Point {
    pub x: i16,
    pub y: i16,
    pub on_curve: bool,
}

pub struct Contour {
    pub points: Vec<Point>,
}

#[derive(Clone, Copy)]
struct Table {
    offset: usize,
}

pub struct Font {
    data: Vec<u8>,
    units_per_em: u16,
    num_h_metrics: u16,
    ascender: i16,
    descender: i16,
    index_to_loc_format: i16,
    loca: Option<Table>,
    glyf: Option<Table>,
    hmtx: Option<Table>,
    cmap_format4: Option<Table>,
    family_name: String,
}

impl Font {
    pub fn embedded() -> Font {
        let data = include_bytes!("../fonts/MapleMono-TTF/MapleMono-Regular.ttf").to_vec();
        Font::from_bytes(data).expect("embedded font is valid")
    }

    pub fn from_bytes(data: Vec<u8>) -> Result<Font, String> {
        if data.len() < TABLE_DIRECTORY_OFFSET {
            return Err("font data too short".into());
        }
        let sfnt_version = u32_be(&data, 0);
        if sfnt_version != SFNT_VERSION {
            return Err("not a TrueType font".into());
        }
        let num_tables = u16_be(&data, OFFSET_NUM_TABLES) as usize;
        let head = find_table(&data, num_tables, &TAG_HEAD).ok_or("missing head table")?;
        let hhea = find_table(&data, num_tables, &TAG_HHEA).ok_or("missing hhea table")?;

        let units_per_em = u16_be(&data, head.offset + HEAD_UNITS_PER_EM);
        let index_to_loc_format = i16_be(&data, head.offset + HEAD_INDEX_TO_LOC);
        let num_h_metrics = u16_be(&data, hhea.offset + HHEA_NUM_H_METRICS);
        let ascender = i16_be(&data, hhea.offset + HHEA_ASCENDER);
        let descender = i16_be(&data, hhea.offset + HHEA_DESCENDER);
        let family_name = parse_family_name(&data, num_tables).unwrap_or_default();
        let loca = find_table(&data, num_tables, &TAG_LOCA);
        let glyf = find_table(&data, num_tables, &TAG_GLYF);
        let hmtx = find_table(&data, num_tables, &TAG_HMTX);
        let cmap_format4 = find_cmap_format4(&data, num_tables);

        Ok(Font {
            data,
            units_per_em,
            num_h_metrics,
            ascender,
            descender,
            index_to_loc_format,
            loca,
            glyf,
            hmtx,
            cmap_format4,
            family_name,
        })
    }

    pub fn from_path(path: &Path) -> Result<Font, String> {
        let data = fs::read(path).map_err(|error| error.to_string())?;
        Font::from_bytes(data)
    }

    pub fn peek_family(path: &Path) -> Option<String> {
        let data = fs::read(path).ok()?;
        let num_tables = u16_be(&data, OFFSET_NUM_TABLES) as usize;
        parse_family_name(&data, num_tables)
    }

    pub fn units_per_em(&self) -> f32 {
        self.units_per_em as f32
    }

    pub fn family_name(&self) -> &str {
        &self.family_name
    }

    pub fn ascender(&self) -> f32 {
        self.ascender as f32
    }

    pub fn descender(&self) -> f32 {
        self.descender as f32
    }

    pub fn glyph_index(&self, ch: char) -> Option<u16> {
        let cp = ch as u32;
        if cp > 0xFFFF {
            return None;
        }
        let cmap = self.cmap_format4?;
        let seg_count = u16_be(&self.data, cmap.offset + CMAP_SEGMENT_COUNT_X2) as usize / 2;
        let end_base = cmap.offset + CMAP_END_CODE_OFFSET;
        let start_base = end_base + seg_count * COORD_OFFSET + CMAP_RESERVED_PAD;
        let delta_base = start_base + seg_count * COORD_OFFSET;
        let range_base = delta_base + seg_count * COORD_OFFSET;
        for seg in 0..seg_count {
            let end = u16_be(&self.data, end_base + seg * COORD_OFFSET);
            let start = u16_be(&self.data, start_base + seg * COORD_OFFSET);
            if (start as u32) <= cp && cp <= end as u32 {
                let delta = i16_be(&self.data, delta_base + seg * COORD_OFFSET);
                let range_offset = u16_be(&self.data, range_base + seg * COORD_OFFSET);
                let glyph = if range_offset == 0 {
                    (cp as i32 + delta as i32) & 0xFFFF
                } else {
                    let index = range_base + seg * COORD_OFFSET + range_offset as usize
                        + (cp as usize - start as usize) * COORD_OFFSET;
                    let raw = u16_be(&self.data, index) as i32;
                    if raw == 0 {
                        0
                    } else {
                        (raw + delta as i32) & 0xFFFF
                    }
                };
                return Some(glyph as u16);
            }
        }
        None
    }

    pub fn advance_width(&self, glyph_id: u16) -> f32 {
        let Some(hmtx) = self.hmtx else {
            return 0.0;
        };
        let index = (glyph_id as usize).min(self.num_h_metrics as usize - 1);
        u16_be(&self.data, hmtx.offset + index * HMTX_ENTRY_SIZE) as f32
    }

    pub fn glyph_outline(&self, glyph_id: u16) -> Option<Vec<Contour>> {
        self.glyph_outline_with_shift(glyph_id, GLYPH_BBOX_BYTES)
            .or_else(|| self.glyph_outline_with_shift(glyph_id, 0))
    }

    fn glyph_outline_with_shift(
        &self,
        glyph_id: u16,
        bbox_bytes: usize,
    ) -> Option<Vec<Contour>> {
        let (loca, glyf) = (self.loca?, self.glyf?);
        let start = self.loca_offset(glyph_id, loca)?;
        let end = self.loca_offset(glyph_id.wrapping_add(1), loca)?;
        if start >= end {
            return None;
        }
        let glyph_offset = glyf.offset + start;
        let num_contours = self.i16_checked(glyph_offset + NUM_CONTOURS_OFFSET)?;
        if num_contours < 0 {
            return None;
        }
        let contour_count = num_contours as usize;
        let end_pts_base = glyph_offset + END_POINTS_OFFSET + bbox_bytes;
        let last_point = self.u16_checked(end_pts_base + (contour_count - 1) * COORD_OFFSET)?;
        let total_points = last_point as usize + 1;
        let instruction_length =
            self.u16_checked(end_pts_base + contour_count * COORD_OFFSET)? as usize;
        let flags_base = end_pts_base + contour_count * COORD_OFFSET + INSTRUCTION_LENGTH_OFFSET
            + instruction_length;

        let (xs, ys, on_curve) = self.parse_points(flags_base, total_points)?;

        let mut contours = Vec::with_capacity(contour_count);
        let mut point_index = 0;
        for contour_index in 0..contour_count {
            let end = self.u16_checked(end_pts_base + contour_index * COORD_OFFSET)? as usize;
            let mut points = Vec::new();
            while point_index <= end {
                points.push(Point {
                    x: xs[point_index],
                    y: ys[point_index],
                    on_curve: on_curve[point_index],
                });
                point_index += 1;
            }
            contours.push(Contour { points });
        }
        Some(contours)
    }

    fn parse_points(
        &self,
        flags_base: usize,
        total_points: usize,
    ) -> Option<(Vec<i16>, Vec<i16>, Vec<bool>)> {
        let mut flags = vec![0u8; total_points];
        let mut on_curve = vec![false; total_points];
        let mut pos = flags_base;
        let mut repeat = 0u8;
        let mut flag = 0u8;
        for index in 0..total_points {
            if repeat == 0 {
                flag = *self.data.get(pos)?;
                pos += 1;
                if flag & FLAG_REPEAT != 0 {
                    repeat = *self.data.get(pos)?;
                    pos += 1;
                }
            } else {
                repeat -= 1;
            }
            flags[index] = flag;
            on_curve[index] = flag & FLAG_ON_CURVE != 0;
        }

        let mut xs = vec![0i16; total_points];
        let mut x = 0i32;
        for index in 0..total_points {
            let flag = flags[index];
            if flag & FLAG_X_SHORT != 0 {
                let dx = *self.data.get(pos)? as i32;
                pos += 1;
                x += if flag & FLAG_X_SAME != 0 { dx } else { -dx };
            } else if flag & FLAG_X_SAME == 0 {
                x += self.i16_checked(pos)? as i32;
                pos += COORD_OFFSET;
            }
            xs[index] = x as i16;
        }

        let mut ys = vec![0i16; total_points];
        let mut y = 0i32;
        for index in 0..total_points {
            let flag = flags[index];
            if flag & FLAG_Y_SHORT != 0 {
                let dy = *self.data.get(pos)? as i32;
                pos += 1;
                y += if flag & FLAG_Y_SAME != 0 { dy } else { -dy };
            } else if flag & FLAG_Y_SAME == 0 {
                y += self.i16_checked(pos)? as i32;
                pos += COORD_OFFSET;
            }
            ys[index] = y as i16;
        }

        Some((xs, ys, on_curve))
    }

    fn u16_checked(&self, offset: usize) -> Option<u16> {
        if offset + COORD_OFFSET > self.data.len() {
            return None;
        }
        Some(u16_be(&self.data, offset))
    }

    fn i16_checked(&self, offset: usize) -> Option<i16> {
        self.u16_checked(offset).map(|value| value as i16)
    }

    fn loca_offset(&self, glyph_id: u16, loca: Table) -> Option<usize> {
        if self.index_to_loc_format == 0 {
            Some(u16_be(&self.data, loca.offset + glyph_id as usize * COORD_OFFSET) as usize * 2)
        } else {
            Some(u32_be(&self.data, loca.offset + glyph_id as usize * 4) as usize)
        }
    }
}

fn u16_be(data: &[u8], offset: usize) -> u16 {
    ((data[offset] as u16) << 8) | data[offset + 1] as u16
}

fn i16_be(data: &[u8], offset: usize) -> i16 {
    u16_be(data, offset) as i16
}

fn u32_be(data: &[u8], offset: usize) -> u32 {
    ((data[offset] as u32) << 24)
        | ((data[offset + 1] as u32) << 16)
        | ((data[offset + 2] as u32) << 8)
        | data[offset + 3] as u32
}

fn find_table(data: &[u8], num_tables: usize, tag: &[u8; 4]) -> Option<Table> {
    for index in 0..num_tables {
        let entry = TABLE_DIRECTORY_OFFSET + index * DIRECTORY_ENTRY_SIZE;
        if entry + DIRECTORY_ENTRY_SIZE > data.len() {
            return None;
        }
        if &data[entry..entry + 4] == tag {
            return Some(Table {
                offset: u32_be(data, entry + 8) as usize,
            });
        }
    }
    None
}

fn find_cmap_format4(data: &[u8], num_tables: usize) -> Option<Table> {
    let cmap = find_table(data, num_tables, &TAG_CMAP)?;
    let num_subtables = u16_be(data, cmap.offset + CMAP_NUM_SUBTABLES_OFFSET) as usize;
    for index in 0..num_subtables {
        let record = cmap.offset + CMAP_SUBTABLE_OFFSET + index * CMAP_RECORD_SIZE;
        let platform = u16_be(data, record);
        let encoding = u16_be(data, record + 2);
        let offset = u32_be(data, record + 4) as usize;
        let subtable = cmap.offset + offset;
        if platform == CMAP_WINDOWS_PLATFORM
            && encoding == CMAP_WINDOWS_ENCODING
            && u16_be(data, subtable + CMAP_FORMAT_OFFSET) == CMAP_FORMAT4
        {
            return Some(Table {
                offset: subtable,
            });
        }
    }
    for index in 0..num_subtables {
        let record = cmap.offset + CMAP_SUBTABLE_OFFSET + index * CMAP_RECORD_SIZE;
        let offset = u32_be(data, record + 4) as usize;
        let subtable = cmap.offset + offset;
        if u16_be(data, subtable + CMAP_FORMAT_OFFSET) == CMAP_FORMAT4 {
            return Some(Table {
                offset: subtable,
            });
        }
    }
    None
}

fn parse_family_name(data: &[u8], num_tables: usize) -> Option<String> {
    let name = find_table(data, num_tables, &TAG_NAME)?;
    let count = u16_be(data, name.offset + NAME_COUNT_OFFSET) as usize;
    let string_offset = u16_be(data, name.offset + NAME_STRING_OFFSET) as usize;
    for index in 0..count {
        let record = name.offset + NAME_RECORDS_OFFSET + index * NAME_RECORD_SIZE;
        let platform = u16_be(data, record);
        let encoding = u16_be(data, record + 2);
        let language = u16_be(data, record + 4);
        let name_id = u16_be(data, record + 6);
        let length = u16_be(data, record + 8) as usize;
        let offset = u16_be(data, record + 10) as usize;
        if name_id == NAME_ID_FAMILY
            && platform == NAME_WINDOWS_PLATFORM
            && encoding == NAME_WINDOWS_ENCODING
            && language == NAME_LANGUAGE_ENGLISH
        {
            let start = name.offset + string_offset + offset;
            let end = start + length;
            if end <= data.len() {
                return Some(utf16be_to_string(&data[start..end]));
            }
        }
    }
    None
}

fn utf16be_to_string(bytes: &[u8]) -> String {
    let mut output = String::new();
    let mut index = 0;
    while index + 1 < bytes.len() {
        let unit = u16_be(bytes, index);
        output.push(char::from_u32(unit as u32).unwrap_or('?'));
        index += 2;
    }
    output
}

#[cfg(test)]
mod tests {
    use super::Font;

    #[test]
    fn embedded_font_parses() {
        let font = Font::embedded();
        assert!(!font.family_name().is_empty());
        assert!(font.glyph_index('A').is_some());
        let glyph_id = font.glyph_index('A').unwrap();
        assert!(font.advance_width(glyph_id) > 0.0);
        assert!(font.glyph_outline(glyph_id).is_some());
    }

    #[test]
    fn space_has_advance_and_parses() {
        let font = Font::embedded();
        let glyph_id = font.glyph_index(' ').unwrap();
        let _ = font.glyph_outline(glyph_id);
        assert!(font.advance_width(glyph_id) > 0.0);
    }
}

