/// Literal extraction for prefilter optimization
///
/// Extracts literal strings from patterns to use as fast prefilters
/// before running the full regex engine.

/// Represents extracted literals from a pattern
#[derive(Debug, Clone, PartialEq)]
pub struct LiteralSet {
    /// The extracted literals
    pub literals: Vec<Literal>,
    /// Type of literal extraction
    pub kind: LiteralKind,
}

/// A single literal string with metadata
#[derive(Debug, Clone, PartialEq)]
pub struct Literal {
    /// The literal string
    pub text: String,
    /// Whether this literal is exact (reaches match state)
    pub is_exact: bool,
}

/// Type of literal extraction
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LiteralKind {
    /// Literals at the start of the pattern (prefix)
    Prefix,
    /// Literals at the end of the pattern (suffix)  
    Suffix,
    /// Literals in the middle of the pattern
    Inner,
    /// No useful literals found
    None,
}

impl LiteralSet {
    /// Create an empty literal set
    pub fn empty() -> Self {
        LiteralSet {
            literals: Vec::new(),
            kind: LiteralKind::None,
        }
    }

    /// Check if this set is empty
    pub fn is_empty(&self) -> bool {
        self.literals.is_empty()
    }

    /// Get longest common prefix of all literals
    pub fn longest_common_prefix(&self) -> Option<&str> {
        if self.literals.is_empty() {
            return None;
        }

        let first = &self.literals[0].text;
        let mut prefix_len = first.len();

        for lit in &self.literals[1..] {
            let common = first
                .chars()
                .zip(lit.text.chars())
                .take_while(|(a, b)| a == b)
                .count();
            prefix_len = prefix_len.min(common);
            if prefix_len == 0 {
                return None;
            }
        }

        Some(
            &first[..first
                .char_indices()
                .nth(prefix_len)
                .map(|(i, _)| i)
                .unwrap_or(first.len())],
        )
    }
}

/// Extract literals from a pattern string
pub fn extract_from_pattern(pattern: &str) -> LiteralSet {
    // Look for alternation patterns first - extract all branches
    if let Some(literals) = extract_alternation_literals(pattern) {
        if !literals.is_empty() {
            return LiteralSet {
                literals,
                kind: LiteralKind::Prefix,
            };
        }
    }

    // Quick check for prefix literals
    if let Some(prefix) = extract_simple_prefix(pattern) {
        if prefix.len() >= 3 {
            return LiteralSet {
                literals: vec![Literal {
                    text: prefix,
                    is_exact: true,
                }],
                kind: LiteralKind::Prefix,
            };
        }
    }

    // Look for inner anchor characters like '@' in email patterns
    if let Some(anchor) = find_inner_anchor(pattern) {
        return LiteralSet {
            literals: vec![Literal {
                text: anchor,
                is_exact: false,
            }],
            kind: LiteralKind::Inner,
        };
    }

    LiteralSet::empty()
}

/// Extract simple literal prefix (before any meta characters)
fn extract_simple_prefix(pattern: &str) -> Option<String> {
    let mut prefix = String::new();
    let mut chars = pattern.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            '.' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}' | '|' | '^' | '$' => break,
            '\\' => {
                chars.next();
                if let Some(next) = chars.peek() {
                    if !matches!(next, 'd' | 'w' | 's' | 'D' | 'W' | 'S' | 'b' | 'B') {
                        prefix.push(*next);
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
            _ => {
                prefix.push(ch);
                chars.next();
            }
        }
    }

    if prefix.is_empty() {
        None
    } else {
        Some(prefix)
    }
}

/// Extract common prefix from alternation like (http|https)
fn extract_alternation_prefix(pattern: &str) -> Option<String> {
    if !pattern.starts_with('(') {
        return None;
    }

    let end = pattern.find(')')?;
    let inner = &pattern[1..end];

    if !inner.contains('|') {
        return None;
    }

    let branches: Vec<&str> = inner.split('|').collect();
    if branches.len() < 2 {
        return None;
    }

    let first = branches[0];
    let mut prefix_len = first.len();

    for branch in &branches[1..] {
        let common = first
            .chars()
            .zip(branch.chars())
            .take_while(|(a, b)| a == b)
            .count();
        prefix_len = prefix_len.min(common);
        if prefix_len == 0 {
            return None;
        }
    }

    Some(first[..prefix_len].to_string())
}

/// Extract all branches from alternation pattern
fn extract_alternation_literals(pattern: &str) -> Option<Vec<Literal>> {
    // Handle both (foo|bar|baz) and foo|bar|baz formats
    let inner = if pattern.starts_with('(') {
        let end = pattern.find(')')?;
        &pattern[1..end]
    } else if pattern.contains('|') && !pattern.contains('(') {
        // Simple alternation without parens
        pattern
    } else {
        return None;
    };

    if !inner.contains('|') {
        return None;
    }

    let branches: Vec<&str> = inner.split('|').collect();
    if branches.len() < 2 {
        return None;
    }

    // Extract literal prefix from each branch
    let mut literals = Vec::new();
    for branch in branches {
        if let Some(prefix) = extract_simple_prefix(branch) {
            if prefix.len() >= 2 {
                // At least 2 chars to be useful
                literals.push(Literal {
                    text: prefix,
                    is_exact: false,
                });
            }
        } else if !branch.is_empty() {
            // If no prefix, but branch is not empty, try first char
            let first_char: String = branch.chars().take(1).collect();
            if !first_char.is_empty() {
                literals.push(Literal {
                    text: first_char,
                    is_exact: false,
                });
            }
        }
    }

    if literals.is_empty() {
        None
    } else {
        Some(literals)
    }
}

/// Find anchor character like '@' in \w+@\w+
fn find_inner_anchor(pattern: &str) -> Option<String> {
    let bytes = pattern.as_bytes();
    let mut i = 1;

    while i < bytes.len() {
        let prev = bytes[i - 1];
        let curr = bytes[i];

        // If current is a backslash, handle escaped sequence
        if curr == b'\\' && i + 1 < bytes.len() {
            let next = bytes[i + 1];
            // Check if this is an escaped literal character (not a meta character like \d, \w, etc)
            if matches!(
                next,
                b'.' | b'*' | b'+' | b'?' | b'[' | b']' | b'(' | b')' | b'{' | b'}' | b'\\' | b'|'
            ) {
                // This is an escaped literal - check if it comes after a quantifier
                if matches!(prev, b'+' | b'*' | b'?') {
                    return Some((next as char).to_string());
                }
            }
            i += 2; // Skip the backslash and the next character
            continue;
        }

        // Check for unescaped special characters after quantifiers
        if matches!(prev, b'+' | b'*' | b'?')
            && curr.is_ascii()
            && !curr.is_ascii_alphanumeric()
            && curr != b'\\'
        {
            // Make sure it's not part of an escape sequence
            if i >= 2 && bytes[i - 2] == b'\\' {
                i += 1;
                continue;
            }

            return Some((curr as char).to_string());
        }

        i += 1;
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_from_pattern() {
        // Prefix
        let lits = extract_from_pattern("hello.*");
        assert_eq!(lits.literals.len(), 1);
        assert_eq!(lits.literals[0].text, "hello");

        // Alternation - extracts all branches
        let lits = extract_from_pattern("(http|https)://.*");
        assert_eq!(lits.literals.len(), 2);
        assert_eq!(lits.literals[0].text, "http");
        assert_eq!(lits.literals[1].text, "https");
        assert_eq!(lits.kind, LiteralKind::Prefix);

        // Inner anchor
        let lits = extract_from_pattern(r"\w+@\w+\.\w+");
        assert_eq!(lits.literals.len(), 1);
        assert_eq!(lits.literals[0].text, "@");
        assert_eq!(lits.kind, LiteralKind::Inner);
    }
}
