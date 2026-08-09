use super::render_font::Table;

const TAG_CCMP: [u8; 4] = *b"ccmp";
const TAG_CALT: [u8; 4] = *b"calt";
const TAG_LIGA: [u8; 4] = *b"liga";
const TAG_RLIG: [u8; 4] = *b"rlig";
const FEATURE_APPLY_ORDER: [[u8; 4]; 4] = [TAG_CCMP, TAG_CALT, TAG_LIGA, TAG_RLIG];

const GSUB_FEATURE_LIST_OFFSET: usize = 6;
const GSUB_LOOKUP_LIST_OFFSET: usize = 8;

const LOOKUP_TYPE_SINGLE: u16 = 1;
const LOOKUP_TYPE_ALTERNATE: u16 = 3;
const LOOKUP_TYPE_LIGATURE: u16 = 4;
const LOOKUP_TYPE_CHAIN: u16 = 6;

const WORD: usize = 2;

fn u16_at(data: &[u8], offset: usize) -> u16 {
    if offset + WORD > data.len() {
        return 0;
    }
    u16::from_be_bytes([data[offset], data[offset + 1]])
}

fn i16_at(data: &[u8], offset: usize) -> i16 {
    u16_at(data, offset) as i16
}

pub struct GsubLookup {
    pub lookup_type: u16,
    pub lookup_flag: u16,
    pub offset: usize,
    subtable_offsets: Vec<u16>,
}

pub struct FeatureLookup {
    pub tag: [u8; 4],
    pub lookup_indices: Vec<u16>,
}

pub struct GsubTableData {
    pub lookups: Vec<GsubLookup>,
    pub features: Vec<FeatureLookup>,
}

pub(crate) fn parse_gsub(data: &[u8], gsub_table: Table) -> Option<GsubTableData> {
    let base = gsub_table.offset;
    let feature_list = u16_at(data, base + GSUB_FEATURE_LIST_OFFSET) as usize;
    let lookup_list = u16_at(data, base + GSUB_LOOKUP_LIST_OFFSET) as usize;
    if feature_list == 0 || lookup_list == 0 {
        return None;
    }
    let lookups = parse_lookup_list(data, base, lookup_list)?;
    let features = parse_feature_list(data, base, feature_list)?;
    Some(GsubTableData { lookups, features })
}

pub fn apply_gsub(data: &[u8], gsub: &GsubTableData, glyphs: &[u16]) -> Vec<(u16, usize)> {
    let mut sequence: Vec<(u16, usize)> = glyphs.iter().map(|&glyph| (glyph, 1)).collect();
    for tag in FEATURE_APPLY_ORDER {
        for feature in gsub.features.iter().filter(|feature| feature.tag == tag) {
            for &lookup_index in &feature.lookup_indices {
                if let Some(lookup) = gsub.lookups.get(lookup_index as usize) {
                    apply_lookup(data, gsub, lookup, &mut sequence);
                }
            }
        }
    }
    sequence
}

fn apply_lookup(
    data: &[u8],
    gsub: &GsubTableData,
    lookup: &GsubLookup,
    sequence: &mut Vec<(u16, usize)>,
) {
    let mut position = 0;
    while position < sequence.len() {
        if let Some(consumed) = try_apply_at(data, gsub, lookup, sequence, position) {
            position += consumed;
        } else {
            position += 1;
        }
    }
}

fn apply_lookup_at(
    data: &[u8],
    gsub: &GsubTableData,
    lookup: &GsubLookup,
    sequence: &mut Vec<(u16, usize)>,
    position: usize,
) {
    let _ = try_apply_at(data, gsub, lookup, sequence, position);
}

fn try_apply_at(
    data: &[u8],
    gsub: &GsubTableData,
    lookup: &GsubLookup,
    sequence: &mut Vec<(u16, usize)>,
    position: usize,
) -> Option<usize> {
    let glyph = sequence[position].0;
    for &sub_offset in &lookup.subtable_offsets {
        let sub_base = lookup.offset + sub_offset as usize;
        let result = match lookup.lookup_type {
            LOOKUP_TYPE_SINGLE => {
                if let Some(replacement) = apply_single(data, sub_base, glyph) {
                    sequence[position].0 = replacement;
                    Some(1)
                } else {
                    None
                }
            }
            LOOKUP_TYPE_ALTERNATE => {
                if let Some(replacement) = apply_alternate(data, sub_base, glyph) {
                    sequence[position].0 = replacement;
                    Some(1)
                } else {
                    None
                }
            }
            LOOKUP_TYPE_LIGATURE => apply_ligature(data, sub_base, glyph, sequence, position),
            LOOKUP_TYPE_CHAIN => apply_chain(data, gsub, sub_base, sequence, position),
            _ => None,
        };
        if result.is_some() {
            return result;
        }
    }
    None
}

fn coverage_index(data: &[u8], coverage_base: usize, glyph: u16) -> Option<u16> {
    let format = u16_at(data, coverage_base);
    if format == 1 {
        let count = u16_at(data, coverage_base + WORD) as usize;
        let array = coverage_base + WORD * 2;
        for index in 0..count {
            if u16_at(data, array + index * WORD) == glyph {
                return Some(index as u16);
            }
        }
        None
    } else {
        let range_count = u16_at(data, coverage_base + WORD) as usize;
        let ranges = coverage_base + WORD * 2;
        for index in 0..range_count {
            let entry = ranges + index * 6;
            let start = u16_at(data, entry);
            let end = u16_at(data, entry + WORD);
            let start_coverage = u16_at(data, entry + 4);
            if glyph >= start && glyph <= end {
                return Some(start_coverage + (glyph - start));
            }
        }
        None
    }
}

fn apply_single(data: &[u8], sub_base: usize, glyph: u16) -> Option<u16> {
    let coverage_offset = u16_at(data, sub_base + WORD) as usize;
    let coverage = sub_base + coverage_offset;
    let index = coverage_index(data, coverage, glyph)?;
    let format = u16_at(data, sub_base);
    if format == 1 {
        let delta = i16_at(data, sub_base + 4);
        Some((glyph as i32 + delta as i32) as u16)
    } else {
        let count = u16_at(data, sub_base + 4) as usize;
        if index as usize >= count {
            return None;
        }
        Some(u16_at(data, sub_base + 6 + index as usize * WORD))
    }
}

fn apply_alternate(data: &[u8], sub_base: usize, glyph: u16) -> Option<u16> {
    let coverage_offset = u16_at(data, sub_base + WORD) as usize;
    let coverage = sub_base + coverage_offset;
    let index = coverage_index(data, coverage, glyph)? as usize;
    let set_count = u16_at(data, sub_base + 4) as usize;
    if index >= set_count {
        return None;
    }
    let set_offset = u16_at(data, sub_base + 6 + index * WORD) as usize;
    let set_base = sub_base + set_offset;
    let alternate_count = u16_at(data, set_base) as usize;
    if alternate_count == 0 {
        return None;
    }
    Some(u16_at(data, set_base + WORD))
}

fn apply_ligature(
    data: &[u8],
    sub_base: usize,
    glyph: u16,
    sequence: &mut Vec<(u16, usize)>,
    position: usize,
) -> Option<usize> {
    let coverage_offset = u16_at(data, sub_base + WORD) as usize;
    let coverage = sub_base + coverage_offset;
    let lig_index = coverage_index(data, coverage, glyph)? as usize;
    let lig_set_count = u16_at(data, sub_base + 4) as usize;
    if lig_index >= lig_set_count {
        return None;
    }
    let set_offset = u16_at(data, sub_base + 6 + lig_index * WORD) as usize;
    let set_base = sub_base + set_offset;
    let ligature_count = u16_at(data, set_base) as usize;
    for ligature_index in 0..ligature_count {
        let entry_offset = u16_at(data, set_base + WORD + ligature_index * WORD) as usize;
        let entry = set_base + entry_offset;
        let ligature_glyph = u16_at(data, entry);
        let component_count = u16_at(data, entry + WORD) as usize;
        if component_count < 1 {
            continue;
        }
        if component_count - 1 > sequence.len().saturating_sub(position + 1) {
            continue;
        }
        let mut matches = true;
        for component in 0..component_count - 1 {
            let expected = u16_at(data, entry + 4 + component * WORD);
            if sequence[position + 1 + component].0 != expected {
                matches = false;
                break;
            }
        }
        if matches {
            let total_consumed: usize = sequence[position..position + component_count]
                .iter()
                .map(|(_, consumed)| consumed)
                .sum();
            sequence.splice(
                position..position + component_count,
                vec![(ligature_glyph, total_consumed)],
            );
            return Some(component_count);
        }
    }
    None
}

fn apply_chain(
    data: &[u8],
    gsub: &GsubTableData,
    sub_base: usize,
    sequence: &mut Vec<(u16, usize)>,
    position: usize,
) -> Option<usize> {
    let format = u16_at(data, sub_base);
    match format {
        1 => apply_chain_format1(data, gsub, sub_base, sequence, position),
        3 => apply_chain_format3(data, gsub, sub_base, sequence, position),
        _ => None,
    }
}

fn apply_chain_format1(
    data: &[u8],
    gsub: &GsubTableData,
    sub_base: usize,
    sequence: &mut Vec<(u16, usize)>,
    position: usize,
) -> Option<usize> {
    let coverage_offset = u16_at(data, sub_base + WORD) as usize;
    let coverage = sub_base + coverage_offset;
    let input_glyph = sequence[position].0;
    let rule_set_index = coverage_index(data, coverage, input_glyph)? as usize;
    let rule_set_count = u16_at(data, sub_base + 4) as usize;
    if rule_set_index >= rule_set_count {
        return None;
    }
    let set_offset = u16_at(data, sub_base + 6 + rule_set_index * WORD) as usize;
    let set_base = sub_base + 6 + set_offset;
    let rule_count = u16_at(data, set_base) as usize;
    for rule_index in 0..rule_count {
        let rule_offset = u16_at(data, set_base + WORD + rule_index * WORD) as usize;
        let rule = set_base + WORD + rule_offset;
        let mut cursor = rule;
        let mut ok = true;

        let backtrack_count = u16_at(data, cursor) as usize;
        cursor += WORD;
        for backtrack in 0..backtrack_count {
            if (position as isize - 1 - backtrack as isize) < 0 {
                ok = false;
                break;
            }
            if sequence[position - 1 - backtrack].0 != u16_at(data, cursor + backtrack * WORD) {
                ok = false;
                break;
            }
        }
        cursor += backtrack_count * WORD;

        let input_count = u16_at(data, cursor) as usize;
        cursor += WORD;
        for input in 0..input_count {
            if position + input >= sequence.len() {
                ok = false;
                break;
            }
            let wanted = u16_at(data, cursor + input * WORD);
            let actual = sequence[position + input].0;
            if coverage_index(data, coverage, actual) != Some(wanted) {
                ok = false;
                break;
            }
        }
        cursor += input_count * WORD;

        let lookahead_count = u16_at(data, cursor) as usize;
        cursor += WORD;
        for lookahead in 0..lookahead_count {
            if position + input_count + lookahead >= sequence.len() {
                ok = false;
                break;
            }
            if sequence[position + input_count + lookahead].0
                != u16_at(data, cursor + lookahead * WORD)
            {
                ok = false;
                break;
            }
        }
        cursor += lookahead_count * WORD;

        if !ok {
            continue;
        }

        let sub_lookup_count = u16_at(data, cursor) as usize;
        let sub_array = cursor + WORD;
        for sub in 0..sub_lookup_count.min(input_count) {
            let sub_offset = u16_at(data, sub_array + sub * WORD) as usize;
            let lookup_index = u16_at(data, sub_array + sub_offset) as usize;
            if let Some(lookup) = gsub.lookups.get(lookup_index) {
                apply_lookup_at(data, gsub, lookup, sequence, position + sub);
            }
        }
        return Some(input_count);
    }
    None
}

fn apply_chain_format3(
    data: &[u8],
    gsub: &GsubTableData,
    sub_base: usize,
    sequence: &mut Vec<(u16, usize)>,
    position: usize,
) -> Option<usize> {
    let backtrack_count = u16_at(data, sub_base + WORD) as usize;
    let mut cursor = sub_base + WORD * 2;
    let backtrack_coverages = cursor;
    cursor += backtrack_count * WORD;

    let input_count = u16_at(data, cursor) as usize;
    cursor += WORD;
    let input_coverages = cursor;
    cursor += input_count * WORD;

    let lookahead_count = u16_at(data, cursor) as usize;
    cursor += WORD;
    let lookahead_coverages = cursor;
    cursor += lookahead_count * WORD;

    let sub_lookup_count = u16_at(data, cursor) as usize;
    let sub_array = cursor + WORD;

    for backtrack in 0..backtrack_count {
        if (position as isize - 1 - backtrack as isize) < 0 {
            return None;
        }
        let coverage_offset = u16_at(data, backtrack_coverages + backtrack * WORD) as usize;
        let coverage = sub_base + coverage_offset;
        if coverage_index(data, coverage, sequence[position - 1 - backtrack].0).is_none() {
            return None;
        }
    }
    for input in 0..input_count {
        if position + input >= sequence.len() {
            return None;
        }
        let coverage_offset = u16_at(data, input_coverages + input * WORD) as usize;
        let coverage = sub_base + coverage_offset;
        if coverage_index(data, coverage, sequence[position + input].0).is_none() {
            return None;
        }
    }
    for lookahead in 0..lookahead_count {
        if position + input_count + lookahead >= sequence.len() {
            return None;
        }
        let coverage_offset = u16_at(data, lookahead_coverages + lookahead * WORD) as usize;
        let coverage = sub_base + coverage_offset;
        if coverage_index(data, coverage, sequence[position + input_count + lookahead].0).is_none()
        {
            return None;
        }
    }

    for sub in 0..sub_lookup_count.min(input_count) {
        let sub_offset = u16_at(data, sub_array + sub * WORD) as usize;
        let lookup_index = u16_at(data, sub_array + sub_offset) as usize;
        if let Some(lookup) = gsub.lookups.get(lookup_index) {
            apply_lookup_at(data, gsub, lookup, sequence, position + sub);
        }
    }
    Some(input_count)
}

fn parse_lookup_list(
    data: &[u8],
    gsub_base: usize,
    lookup_list_offset: usize,
) -> Option<Vec<GsubLookup>> {
    let base = gsub_base + lookup_list_offset;
    let lookup_count = u16_at(data, base) as usize;
    let mut lookups = Vec::with_capacity(lookup_count);
    for index in 0..lookup_count {
        let offset = gsub_base + lookup_list_offset + u16_at(data, base + WORD + index * WORD) as usize;
        let lookup_type = u16_at(data, offset);
        let lookup_flag = u16_at(data, offset + WORD);
        let subtable_count = u16_at(data, offset + 4) as usize;
        let mut subtable_offsets = Vec::with_capacity(subtable_count);
        for sub in 0..subtable_count {
            subtable_offsets.push(u16_at(data, offset + 6 + sub * WORD));
        }
        lookups.push(GsubLookup {
            lookup_type,
            lookup_flag,
            offset,
            subtable_offsets,
        });
    }
    Some(lookups)
}

fn parse_feature_list(
    data: &[u8],
    gsub_base: usize,
    feature_list_offset: usize,
) -> Option<Vec<FeatureLookup>> {
    let base = gsub_base + feature_list_offset;
    let feature_count = u16_at(data, base) as usize;
    let records = base + WORD;
    let mut features = Vec::new();
    for index in 0..feature_count {
        let record = records + index * 6;
        let tag_bytes = data.get(record..record + 4)?;
        let mut tag = [0u8; 4];
        tag.copy_from_slice(tag_bytes);
        let feature_offset = gsub_base + feature_list_offset + u16_at(data, record + 4) as usize;
        let lookup_count = u16_at(data, feature_offset + WORD) as usize;
        let mut lookup_indices = Vec::with_capacity(lookup_count);
        for lookup in 0..lookup_count {
            lookup_indices.push(u16_at(data, feature_offset + 4 + lookup * WORD));
        }
        if !lookup_indices.is_empty() {
            features.push(FeatureLookup { tag, lookup_indices });
        }
    }
    Some(features)
}

pub struct LigatureEntry {
    pub sequence: Vec<u16>,
    pub replacement: u16,
}

pub struct LigatureTrie {
    root: Vec<Option<Box<LigatureTrieNode>>>,
}

struct LigatureTrieNode {
    replacement: Option<u16>,
    children: Vec<Option<Box<LigatureTrieNode>>>,
}

impl LigatureTrie {
    pub fn new() -> Self {
        Self { root: Vec::new() }
    }

    pub fn insert(&mut self, sequence: &[u16], replacement: u16) {
        if sequence.is_empty() {
            return;
        }
        let index = sequence[0] as usize;
        if index >= self.root.len() {
            self.root.resize_with(index + 1, || None);
        }
        let mut node = self.root[index].get_or_insert_with(|| {
            Box::new(LigatureTrieNode {
                replacement: None,
                children: Vec::new(),
            })
        });
        for &glyph in &sequence[1..] {
            let child_index = glyph as usize;
            if child_index >= node.children.len() {
                node.children.resize_with(child_index + 1, || None);
            }
            let next = node.children[child_index].get_or_insert_with(|| {
                Box::new(LigatureTrieNode {
                    replacement: None,
                    children: Vec::new(),
                })
            });
            node = next;
        }
        node.replacement = Some(replacement);
    }

    pub fn match_ligature(&self, glyph_ids: &[u16], pos: usize) -> Option<(usize, u16)> {
        if pos >= glyph_ids.len() {
            return None;
        }
        let index = glyph_ids[pos] as usize;
        if index >= self.root.len() {
            return None;
        }
        let Some(ref node) = self.root[index] else {
            return None;
        };
        let mut current = node;
        let mut best: Option<(usize, u16)> = None;
        let mut lookahead = 1;
        loop {
            if let Some(replacement) = current.replacement {
                best = Some((lookahead, replacement));
            }
            if pos + lookahead >= glyph_ids.len() {
                break;
            }
            let next_index = glyph_ids[pos + lookahead] as usize;
            if next_index >= current.children.len() {
                break;
            }
            match &current.children[next_index] {
                Some(child) => current = child,
                None => break,
            }
            lookahead += 1;
        }
        best
    }
}

pub fn build_ligature_trie(entries: Vec<LigatureEntry>) -> LigatureTrie {
    let mut trie = LigatureTrie::new();
    for entry in entries {
        trie.insert(&entry.sequence, entry.replacement);
    }
    trie
}

#[cfg(test)]
mod tests {
    use super::LigatureTrie;

    #[test]
    fn trie_matches_single() {
        let mut trie = LigatureTrie::new();
        trie.insert(&[10, 20], 99);
        let result = trie.match_ligature(&[10, 20, 30], 0);
        assert_eq!(result, Some((2, 99)));
    }

    #[test]
    fn trie_no_match() {
        let mut trie = LigatureTrie::new();
        trie.insert(&[10, 20], 99);
        let result = trie.match_ligature(&[10, 30], 0);
        assert!(result.is_none());
    }

    #[test]
    fn trie_prefers_longest() {
        let mut trie = LigatureTrie::new();
        trie.insert(&[10, 20], 50);
        trie.insert(&[10, 20, 30], 99);
        let result = trie.match_ligature(&[10, 20, 30], 0);
        assert_eq!(result, Some((3, 99)));
    }
}
