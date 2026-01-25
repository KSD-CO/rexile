/// Parser for sequence patterns
///
/// Handles patterns like: ab+c*, \d+\w*, hello\d+
use crate::parser::charclass::CharClass;
use crate::parser::escape::{parse_escape, starts_with_escape};
use crate::parser::group::{Group, GroupContent};
use crate::parser::quantifier::Quantifier;
use crate::parser::sequence::{Sequence, SequenceElement};

/// Check if a pattern is a sequence (multiple elements)
pub fn is_sequence_pattern(pattern: &str) -> bool {
    // Skip anchors and simple cases
    if pattern.is_empty() {
        return false;
    }

    // Check if | exists outside of groups - if so, it's an alternation, not a sequence
    if has_top_level_alternation(pattern) {
        return false;
    }

    // Count distinct elements
    let mut element_count = 0;
    let mut i = 0;
    let chars: Vec<char> = pattern.chars().collect();

    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            // Escape sequence
            element_count += 1;
            i += 2;
            // Skip quantifier if present
            if i < chars.len() {
                if matches!(chars[i], '*' | '+' | '?') {
                    i += 1;
                    // Check for lazy modifier
                    if i < chars.len() && chars[i] == '?' {
                        i += 1;
                    }
                } else if chars[i] == '{' {
                    // Skip {n}, {n,}, {n,m} quantifier
                    while i < chars.len() && chars[i] != '}' {
                        i += 1;
                    }
                    if i < chars.len() {
                        i += 1; // Skip '}'
                    }
                }
            }
        } else if chars[i] == '[' {
            // Character class
            element_count += 1;
            while i < chars.len() && chars[i] != ']' {
                i += 1;
            }
            i += 1; // Skip ']'
                    // Skip quantifier if present
            if i < chars.len() {
                if matches!(chars[i], '*' | '+' | '?') {
                    i += 1;
                    if i < chars.len() && chars[i] == '?' {
                        i += 1;
                    }
                } else if chars[i] == '{' {
                    while i < chars.len() && chars[i] != '}' {
                        i += 1;
                    }
                    if i < chars.len() {
                        i += 1;
                    }
                }
            }
        } else if chars[i] == '(' {
            // Group - count as single element, skip to matching ')'
            element_count += 1;
            let mut depth = 1;
            i += 1;
            while i < chars.len() && depth > 0 {
                if chars[i] == '(' {
                    depth += 1;
                } else if chars[i] == ')' {
                    depth -= 1;
                } else if chars[i] == '\\' {
                    i += 1; // Skip escaped char
                }
                i += 1;
            }
            // Skip quantifier if present
            if i < chars.len() {
                if matches!(chars[i], '*' | '+' | '?') {
                    i += 1;
                    if i < chars.len() && chars[i] == '?' {
                        i += 1;
                    }
                } else if chars[i] == '{' {
                    while i < chars.len() && chars[i] != '}' {
                        i += 1;
                    }
                    if i < chars.len() {
                        i += 1;
                    }
                }
            }
        } else if matches!(chars[i], '*' | '+' | '?' | '^' | '$') {
            // Quantifier or anchor - skip
            i += 1;
        } else {
            // Regular character
            element_count += 1;
            i += 1;
            // Skip quantifier if present
            if i < chars.len() {
                if matches!(chars[i], '*' | '+' | '?') {
                    i += 1;
                    if i < chars.len() && chars[i] == '?' {
                        i += 1;
                    }
                } else if chars[i] == '{' {
                    while i < chars.len() && chars[i] != '}' {
                        i += 1;
                    }
                    if i < chars.len() {
                        i += 1;
                    }
                }
            }
        }
    }

    element_count > 1
}

/// Check if pattern has | at the top level (not inside groups or character classes)
fn has_top_level_alternation(pattern: &str) -> bool {
    let mut depth = 0;
    let mut in_charclass = false;
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            '\\' => {
                i += 2;
                continue;
            }
            '[' if !in_charclass => {
                in_charclass = true;
            }
            ']' if in_charclass => {
                in_charclass = false;
            }
            '(' if !in_charclass => {
                depth += 1;
            }
            ')' if !in_charclass => {
                depth -= 1;
            }
            '|' if !in_charclass && depth == 0 => {
                return true;
            }
            _ => {}
        }
        i += 1;
    }
    false
}

/// Parse a sequence pattern
pub fn parse_sequence(pattern: &str) -> Result<Sequence, String> {
    let mut elements = Vec::new();
    let mut i = 0;
    let _bytes = pattern.as_bytes();

    while i < pattern.len() {
        let remaining = &pattern[i..];

        // Try escape sequence
        if starts_with_escape(remaining) {
            let (seq, bytes_consumed) = parse_escape(remaining)?;
            i += bytes_consumed;

            // Check for quantifier (including lazy quantifiers like *?, +?, ??)
            if i < pattern.len() {
                let q_remaining = &pattern[i..];
                if let Some((quantifier, q_bytes)) = parse_quantifier_with_lazy(q_remaining) {
                    i += q_bytes;

                    // Add quantified element
                    if let Some(cc) = seq.to_char_class() {
                        elements.push(SequenceElement::QuantifiedCharClass(cc, quantifier));
                    } else if let Some(ch) = seq.to_char() {
                        elements.push(SequenceElement::QuantifiedChar(ch, quantifier));
                    } else {
                        return Err("Cannot quantify this escape".to_string());
                    }
                    continue;
                }
            }

            // No quantifier - add plain element
            if let Some(cc) = seq.to_char_class() {
                elements.push(SequenceElement::CharClass(cc));
            } else if let Some(ch) = seq.to_char() {
                elements.push(SequenceElement::Char(ch));
            } else if let Some(boundary_type) = seq.to_boundary() {
                elements.push(SequenceElement::Boundary(boundary_type));
            } else {
                return Err("Invalid escape in sequence".to_string());
            }
            continue;
        }

        // Try group (...) or (?:...)
        if remaining.starts_with('(') {
            // Find matching closing paren
            if let Some(close_idx) = find_matching_paren_in_str(remaining) {
                let group_content_start = if remaining.starts_with("(?:") { 3 } else { 1 };
                let is_capturing = !remaining.starts_with("(?:");
                let inner = &remaining[group_content_start..close_idx];

                // Parse group content: alternation or sequence
                let content = if has_top_level_alternation_in(inner, '[', ']') {
                    // Split on top-level | (not inside character classes or groups)
                    let parts = split_top_level(inner);
                    // Check if any alternative needs regex parsing
                    let needs_parsing = parts.iter().any(|p| {
                        p.contains('\\')
                            || p.contains('[')
                            || p.contains('.')
                            || p.contains('*')
                            || p.contains('+')
                            || p.contains('?')
                            || p.contains('(')
                    });
                    if needs_parsing {
                        // Parse each alternative as a sequence
                        let mut sequences = Vec::new();
                        for part in &parts {
                            if is_sequence_pattern(part) {
                                match parse_sequence(part) {
                                    Ok(seq) => sequences.push(seq),
                                    Err(_) => {
                                        // Fallback: single-char sequence
                                        let elems: Vec<SequenceElement> = part
                                            .chars()
                                            .map(|c| SequenceElement::Char(c))
                                            .collect();
                                        sequences.push(Sequence::new(elems));
                                    }
                                }
                            } else {
                                // Simple literal alternative
                                let elems: Vec<SequenceElement> =
                                    part.chars().map(|c| SequenceElement::Char(c)).collect();
                                sequences.push(Sequence::new(elems));
                            }
                        }
                        GroupContent::ParsedAlternation(sequences)
                    } else {
                        GroupContent::Alternation(parts.iter().map(|s| s.to_string()).collect())
                    }
                } else if is_sequence_pattern(inner) {
                    match parse_sequence(inner) {
                        Ok(seq) => GroupContent::Sequence(seq),
                        Err(_) => GroupContent::Single(inner.to_string()),
                    }
                } else {
                    GroupContent::Single(inner.to_string())
                };

                let group = if is_capturing {
                    Group::new_capturing(content)
                } else {
                    Group::new_non_capturing(content)
                };

                i += close_idx + 1; // Skip past ')'

                // Check for quantifier
                if i < pattern.len() {
                    let q_remaining = &pattern[i..];
                    if let Some((quantifier, q_bytes)) = parse_quantifier_with_lazy(q_remaining) {
                        i += q_bytes;
                        elements.push(SequenceElement::QuantifiedGroup(
                            group.with_quantifier(quantifier.clone()),
                            quantifier,
                        ));
                        continue;
                    }
                }

                elements.push(SequenceElement::Group(group));
                continue;
            }
        }

        // Try character class
        if remaining.starts_with('[') {
            if let Some(close_idx) = remaining.find(']') {
                let class_content = &remaining[1..close_idx];
                let char_class = CharClass::parse(class_content)?;
                i += close_idx + 1;

                // Check for quantifier (including lazy)
                if i < pattern.len() {
                    let q_remaining = &pattern[i..];
                    if let Some((quantifier, q_bytes)) = parse_quantifier_with_lazy(q_remaining) {
                        i += q_bytes;
                        elements.push(SequenceElement::QuantifiedCharClass(char_class, quantifier));
                        continue;
                    }
                }

                // No quantifier
                elements.push(SequenceElement::CharClass(char_class));
                continue;
            }
        }

        // Regular character
        if let Some(ch) = remaining.chars().next() {
            let ch_len = ch.len_utf8();
            i += ch_len;

            // Check for quantifier (including lazy)
            if i < pattern.len() {
                let q_remaining = &pattern[i..];
                if let Some((quantifier, q_bytes)) = parse_quantifier_with_lazy(q_remaining) {
                    i += q_bytes;

                    // Special case: dot with quantifier = quantified CharClass for [^\n]
                    if ch == '.' {
                        use crate::parser::charclass::CharClass;
                        let mut dot_class = CharClass::new();
                        dot_class.add_char('\n');
                        dot_class.negate();
                        dot_class.finalize();
                        elements.push(SequenceElement::QuantifiedCharClass(dot_class, quantifier));
                    } else {
                        elements.push(SequenceElement::QuantifiedChar(ch, quantifier));
                    }
                    continue;
                }
            }

            // No quantifier
            if ch == '.' {
                elements.push(SequenceElement::Dot);
            } else {
                elements.push(SequenceElement::Char(ch));
            }
        } else {
            break;
        }
    }

    if elements.is_empty() {
        return Err("Empty sequence".to_string());
    }

    Ok(Sequence::new(elements))
}

/// Find the index of the matching closing paren for an opening paren at position 0
fn find_matching_paren_in_str(s: &str) -> Option<usize> {
    let mut depth = 0;
    let mut i = 0;
    let bytes = s.as_bytes();

    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                i += 2;
                continue;
            } // Skip escaped chars
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// Check if a string has | at the top level (not inside brackets of the given type)
fn has_top_level_alternation_in(s: &str, _open: char, _close: char) -> bool {
    let mut depth = 0;
    let mut in_bracket = false;
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                i += 2;
                continue;
            }
            b'[' => in_bracket = true,
            b']' => in_bracket = false,
            b'(' => depth += 1,
            b')' => depth -= 1,
            b'|' if !in_bracket && depth == 0 => return true,
            _ => {}
        }
        i += 1;
    }
    false
}

/// Split a string on top-level | (not inside groups or character classes)
fn split_top_level(s: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut in_bracket = false;
    let mut start = 0;
    let bytes = s.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'\\' => {
                i += 2;
                continue;
            }
            b'[' => in_bracket = true,
            b']' => in_bracket = false,
            b'(' => depth += 1,
            b')' => depth -= 1,
            b'|' if !in_bracket && depth == 0 => {
                parts.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    parts.push(&s[start..]);
    parts
}

fn parse_simple_quantifier(ch: char) -> Option<Quantifier> {
    match ch {
        '*' => Some(Quantifier::ZeroOrMore),
        '+' => Some(Quantifier::OneOrMore),
        '?' => Some(Quantifier::ZeroOrOne),
        _ => None,
    }
}

/// Parse a quantifier that might be lazy (e.g., *?, +?, ??)
/// Returns (Quantifier, bytes_consumed)
fn parse_quantifier_with_lazy(s: &str) -> Option<(Quantifier, usize)> {
    let mut chars = s.chars();
    let first = chars.next()?;

    // Handle {n}, {n,}, {n,m} range quantifiers
    if first == '{' {
        // Find the closing }
        if let Some(close_idx) = s.find('}') {
            let inner = &s[1..close_idx];
            let bytes_consumed = close_idx + 1;

            // Parse the range quantifier
            if let Ok(n) = inner.parse::<usize>() {
                // {n} - exactly n times
                return Some((Quantifier::Exactly(n), bytes_consumed));
            } else if inner.contains(',') {
                let parts: Vec<&str> = inner.split(',').collect();
                if parts.len() == 2 {
                    if parts[1].is_empty() {
                        // {n,} - at least n times
                        if let Ok(min) = parts[0].parse() {
                            return Some((Quantifier::AtLeast(min), bytes_consumed));
                        }
                    } else {
                        // {n,m} - between n and m times
                        if let (Ok(min), Ok(max)) = (parts[0].parse(), parts[1].parse()) {
                            return Some((Quantifier::Between(min, max), bytes_consumed));
                        }
                    }
                }
            }
        }
        return None;
    }

    let base_quantifier = match first {
        '*' => Quantifier::ZeroOrMore,
        '+' => Quantifier::OneOrMore,
        '?' => Quantifier::ZeroOrOne,
        _ => return None,
    };

    // Check for lazy modifier
    if let Some('?') = chars.next() {
        let lazy_quantifier = match base_quantifier {
            Quantifier::ZeroOrMore => Quantifier::ZeroOrMoreLazy,
            Quantifier::OneOrMore => Quantifier::OneOrMoreLazy,
            Quantifier::ZeroOrOne => Quantifier::ZeroOrOneLazy,
            _ => return Some((base_quantifier, 1)),
        };
        Some((lazy_quantifier, 2))
    } else {
        Some((base_quantifier, 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_sequence() {
        assert!(is_sequence_pattern("abc"));
        assert!(is_sequence_pattern("a+b"));
        assert!(is_sequence_pattern("\\d+\\w*"));
        assert!(is_sequence_pattern("[0-9]+[a-z]"));

        assert!(!is_sequence_pattern("a"));
        assert!(!is_sequence_pattern("a+"));
        assert!(!is_sequence_pattern("[0-9]+"));
        assert!(!is_sequence_pattern("a|b"));
    }

    #[test]
    fn test_parse_simple() {
        let seq = parse_sequence("abc").unwrap();
        assert_eq!(seq.elements.len(), 3);
    }

    #[test]
    fn test_parse_quantified() {
        let seq = parse_sequence("a+b*c?").unwrap();
        assert_eq!(seq.elements.len(), 3);
    }

    #[test]
    fn test_parse_charclass() {
        let seq = parse_sequence("[0-9]+[a-z]").unwrap();
        assert_eq!(seq.elements.len(), 2);
    }

    #[test]
    fn test_parse_escape() {
        let seq = parse_sequence("\\d+\\w*").unwrap();
        assert_eq!(seq.elements.len(), 2);
    }

    #[test]
    fn test_parse_mixed() {
        let seq = parse_sequence("hello\\d+").unwrap();
        assert_eq!(seq.elements.len(), 6); // h,e,l,l,o,\d+
    }
}
