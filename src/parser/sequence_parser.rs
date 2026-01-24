/// Parser for sequence patterns
///
/// Handles patterns like: ab+c*, \d+\w*, hello\d+
use crate::parser::charclass::CharClass;
use crate::parser::escape::{parse_escape, starts_with_escape};
use crate::parser::quantifier::Quantifier;
use crate::parser::sequence::{Sequence, SequenceElement};

/// Check if a pattern is a sequence (multiple elements)
pub fn is_sequence_pattern(pattern: &str) -> bool {
    // Skip anchors and simple cases
    if pattern.is_empty() || pattern.contains('|') {
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
        }
    }

    element_count > 1
}

/// Parse a sequence pattern
pub fn parse_sequence(pattern: &str) -> Result<Sequence, String> {
    let mut elements = Vec::new();
    let mut i = 0;
    let bytes = pattern.as_bytes();

    while i < pattern.len() {
        let remaining = &pattern[i..];

        // Try escape sequence
        if starts_with_escape(remaining) {
            let (seq, bytes_consumed) = parse_escape(remaining)?;
            i += bytes_consumed;

            // Check for quantifier
            if i < pattern.len() {
                let q_remaining = &pattern[i..];
                if let Some(q_char) = q_remaining.chars().next() {
                    if let Some(quantifier) = parse_simple_quantifier(q_char) {
                        i += q_char.len_utf8();

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
            }

            // No quantifier - add plain element
            if let Some(cc) = seq.to_char_class() {
                elements.push(SequenceElement::CharClass(cc));
            } else if let Some(ch) = seq.to_char() {
                elements.push(SequenceElement::Char(ch));
            } else {
                return Err("Invalid escape in sequence".to_string());
            }
            continue;
        }

        // Try character class
        if remaining.starts_with('[') {
            if let Some(close_idx) = remaining.find(']') {
                let class_content = &remaining[1..close_idx];
                let char_class = CharClass::parse(class_content)?;
                i += close_idx + 1;

                // Check for quantifier
                if i < pattern.len() {
                    let q_remaining = &pattern[i..];
                    if let Some(q_char) = q_remaining.chars().next() {
                        if let Some(quantifier) = parse_simple_quantifier(q_char) {
                            i += q_char.len_utf8();
                            elements
                                .push(SequenceElement::QuantifiedCharClass(char_class, quantifier));
                            continue;
                        }
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

            // Check for quantifier
            if i < pattern.len() {
                let q_remaining = &pattern[i..];
                if let Some(q_char) = q_remaining.chars().next() {
                    if let Some(quantifier) = parse_simple_quantifier(q_char) {
                        i += q_char.len_utf8();

                        // Special case: dot with quantifier = quantified CharClass for [^\n]
                        if ch == '.' {
                            use crate::parser::charclass::CharClass;
                            let mut dot_class = CharClass::new();
                            dot_class.add_char('\n');
                            dot_class.negate();
                            dot_class.finalize();
                            elements
                                .push(SequenceElement::QuantifiedCharClass(dot_class, quantifier));
                        } else {
                            elements.push(SequenceElement::QuantifiedChar(ch, quantifier));
                        }
                        continue;
                    }
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

fn parse_simple_quantifier(ch: char) -> Option<Quantifier> {
    match ch {
        '*' => Some(Quantifier::ZeroOrMore),
        '+' => Some(Quantifier::OneOrMore),
        '?' => Some(Quantifier::ZeroOrOne),
        _ => None,
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
