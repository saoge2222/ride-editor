
const LIGA_TABLE: &[(&str, char)] = &[
    ("->", '\u{2192}'),
    ("!=", '\u{2260}'),
    ("<=", '\u{2264}'),
    (">=", '\u{2265}'),
    ("===", '\u{2261}'),
    ("!==", '\u{2262}'),
    ("=>", '\u{21D2}'),
    ("<-", '\u{2190}'),
    ("<--", '\u{27F5}'),
    ("-->", '\u{27F6}'),
    ("<->", '\u{2194}'),
    ("<-->", '\u{27F7}'),
    ("...", '\u{2026}'),
    ("::", '\u{2237}'),
    ("<<", '\u{00AB}'),
    (">>", '\u{00BB}'),
    ("++", '\u{271A}'),
    ("--", '\u{2013}'),
    ("~~", '\u{2248}'),
    ("~=", '\u{2245}'),
    ("/=", '\u{2260}'),
    ("&&", '\u{2227}'),
    ("||", '\u{2228}'),
];

pub fn apply_ligatures(text: &str, glyph_present: impl Fn(char) -> bool) -> Vec<char> {
    let chars: Vec<char> = text.chars().collect();
    let n = chars.len();
    let mut result = Vec::with_capacity(n);
    let mut i = 0;
    while i < n {
        let mut best_len = 0usize;
        let mut best_target = '\0';
        let max_lookahead = if n - i < 4 { n - i } else { 4 };
        for lookahead in (1..=max_lookahead).rev() {
            let slice: String = chars[i..i + lookahead].iter().collect();
            for (pattern, target) in LIGA_TABLE {
                if slice.as_str() == *pattern && glyph_present(*target) {
                    best_len = lookahead;
                    best_target = *target;
                    break;
                }
            }
            if best_len > 0 {
                break;
            }
        }
        if best_len > 0 {
            result.push(best_target);
            i += best_len;
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::apply_ligatures;

    fn all_present(_ch: char) -> bool {
        true
    }

    #[test]
    fn arrow_ligature() {
        let result = apply_ligatures("->", all_present);
        assert_eq!(result, vec!['\u{2192}']);
    }

    #[test]
    fn not_equal_ligature() {
        let result = apply_ligatures("!=", all_present);
        assert_eq!(result, vec!['\u{2260}']);
    }

    #[test]
    fn no_ligature() {
        let result = apply_ligatures("abc", all_present);
        assert_eq!(result, vec!['a', 'b', 'c']);
    }

    #[test]
    fn longest_match_first() {
        let result = apply_ligatures("<--", all_present);
        assert_eq!(result, vec!['\u{27F5}']);
    }

    #[test]
    fn mixed_text() {
        let result = apply_ligatures("hello -> world != test", all_present);
        assert_eq!(
            result,
            vec![
                'h', 'e', 'l', 'l', 'o', ' ', '\u{2192}', ' ', 'w', 'o', 'r', 'l',
                'd', ' ', '\u{2260}', ' ', 't', 'e', 's', 't'
            ]
        );
    }
}
