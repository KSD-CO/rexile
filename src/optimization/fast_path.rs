use aho_corasick::AhoCorasick;
/// Specialized fast path implementations for common patterns
/// These bypass the general matching machinery for maximum speed
use memchr::{memchr, memchr_iter, memmem};
use std::sync::Arc;

/// Fast path for literal strings (no special chars)
#[inline]
pub fn find_literal(text: &str, literal: &str) -> Option<(usize, usize)> {
    if literal.len() >= 3 {
        let finder = memmem::Finder::new(literal.as_bytes());
        finder
            .find(text.as_bytes())
            .map(|pos| (pos, pos + literal.len()))
    } else if literal.len() == 1 {
        let byte = literal.as_bytes()[0];
        memchr(byte, text.as_bytes()).map(|pos| (pos, pos + 1))
    } else {
        text.find(literal).map(|pos| (pos, pos + literal.len()))
    }
}

/// Fast path for literal + quantified char: "rule\s+"
#[inline]
pub fn find_literal_plus_whitespace(text: &str, literal: &str) -> Option<(usize, usize)> {
    let finder = memmem::Finder::new(literal.as_bytes());

    for pos in finder.find_iter(text.as_bytes()) {
        let after = pos + literal.len();
        if after >= text.len() {
            continue;
        }

        // Match at least one whitespace
        let rest = &text[after..];
        let mut matched = 0;
        for ch in rest.chars() {
            if ch.is_whitespace() {
                matched += ch.len_utf8();
            } else {
                break;
            }
        }

        if matched > 0 {
            return Some((pos, after + matched));
        }
    }

    None
}

/// Fast path for \d+ (digit run)
#[inline]
pub fn find_digit_run(text: &str) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            return Some((start, i));
        }
        i += 1;
    }

    None
}

/// Fast path for \w+ (word run)
#[inline]
pub fn find_word_run(text: &str) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_alphanumeric() || b == b'_' {
            let start = i;
            while i < bytes.len() {
                let b = bytes[i];
                if b.is_ascii_alphanumeric() || b == b'_' {
                    i += 1;
                } else {
                    break;
                }
            }
            return Some((start, i));
        }
        i += 1;
    }

    None
}

/// Fast path for quoted strings: "[^"]+"
#[inline]
pub fn find_quoted_string(text: &str) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();

    // Find first quote
    let start = memchr(b'"', bytes)?;

    // Find closing quote
    if start + 1 >= bytes.len() {
        return None;
    }

    let end = memchr(b'"', &bytes[start + 1..])?;

    // Must have at least one char between quotes
    if end == 0 {
        return None;
    }

    Some((start, start + 1 + end + 1))
}

/// Fast path for find_all: literal strings
#[inline]
pub fn find_literal_all(text: &str, literal: &str) -> Vec<(usize, usize)> {
    let mut results = Vec::new();
    let len = literal.len();

    if len >= 3 {
        let finder = memmem::Finder::new(literal.as_bytes());
        for pos in finder.find_iter(text.as_bytes()) {
            results.push((pos, pos + len));
        }
    } else if len == 1 {
        let byte = literal.as_bytes()[0];
        for pos in memchr_iter(byte, text.as_bytes()) {
            results.push((pos, pos + 1));
        }
    } else {
        let mut pos = 0;
        while let Some(idx) = text[pos..].find(literal) {
            let abs_pos = pos + idx;
            results.push((abs_pos, abs_pos + len));
            pos = abs_pos + len;
        }
    }

    results
}

/// Fast path for find_all: literal + whitespace
#[inline]
pub fn find_literal_plus_whitespace_all(text: &str, literal: &str) -> Vec<(usize, usize)> {
    let mut results = Vec::new();
    let finder = memmem::Finder::new(literal.as_bytes());

    for pos in finder.find_iter(text.as_bytes()) {
        let after = pos + literal.len();
        if after >= text.len() {
            continue;
        }

        // Match at least one whitespace
        let rest = &text[after..];
        let mut matched = 0;
        for ch in rest.chars() {
            if ch.is_whitespace() {
                matched += ch.len_utf8();
            } else {
                break;
            }
        }

        if matched > 0 {
            results.push((pos, after + matched));
        }
    }

    results
}

/// Fast path for find_all: digit runs
#[inline]
pub fn find_digit_run_all(text: &str) -> Vec<(usize, usize)> {
    let mut results = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let start = i;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                i += 1;
            }
            results.push((start, i));
        } else {
            i += 1;
        }
    }

    results
}

/// Fast path for find_all: word runs
#[inline]
pub fn find_word_run_all(text: &str) -> Vec<(usize, usize)> {
    let mut results = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_alphanumeric() || b == b'_' {
            let start = i;
            while i < bytes.len() {
                let b = bytes[i];
                if b.is_ascii_alphanumeric() || b == b'_' {
                    i += 1;
                } else {
                    break;
                }
            }
            results.push((start, i));
        } else {
            i += 1;
        }
    }

    results
}

/// Fast path for: identifier pattern [a-zA-Z_]\w*
/// Matches: letter or underscore followed by zero or more word chars
#[inline]
pub fn find_identifier_run(text: &str) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();

    for i in 0..bytes.len() {
        let b = bytes[i];
        // Must start with letter or underscore
        if b.is_ascii_alphabetic() || b == b'_' {
            let start = i;
            let mut j = i + 1;
            // Followed by zero or more word chars (alphanumeric or underscore)
            while j < bytes.len() {
                let b = bytes[j];
                if b.is_ascii_alphanumeric() || b == b'_' {
                    j += 1;
                } else {
                    break;
                }
            }
            return Some((start, j));
        }
    }

    None
}

/// Fast path for find_all: identifier pattern [a-zA-Z_]\w*
#[inline]
pub fn find_identifier_run_all(text: &str) -> Vec<(usize, usize)> {
    let mut results = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];
        // Must start with letter or underscore
        if b.is_ascii_alphabetic() || b == b'_' {
            let start = i;
            i += 1;
            // Followed by zero or more word chars
            while i < bytes.len() {
                let b = bytes[i];
                if b.is_ascii_alphanumeric() || b == b'_' {
                    i += 1;
                } else {
                    break;
                }
            }
            results.push((start, i));
        } else {
            i += 1;
        }
    }

    results
}

/// Fast path for find_all: quoted strings
#[inline]
pub fn find_quoted_string_all(text: &str) -> Vec<(usize, usize)> {
    let mut results = Vec::new();
    let bytes = text.as_bytes();
    let mut pos = 0;

    while pos < bytes.len() {
        // Find next quote
        if let Some(start) = memchr(b'"', &bytes[pos..]) {
            let abs_start = pos + start;

            // Find closing quote
            if abs_start + 1 < bytes.len() {
                if let Some(end) = memchr(b'"', &bytes[abs_start + 1..]) {
                    // Must have at least one char between quotes
                    if end > 0 {
                        results.push((abs_start, abs_start + 1 + end + 1));
                        pos = abs_start + 1 + end + 1;
                        continue;
                    }
                }
            }

            pos = abs_start + 1;
        } else {
            break;
        }
    }

    results
}

/// Fast path for: literal + whitespace + quoted string (rule\s+"[^"]+")
#[inline]
pub fn find_literal_ws_quoted(text: &str, literal: &str) -> Option<(usize, usize)> {
    let finder = memmem::Finder::new(literal.as_bytes());

    for pos in finder.find_iter(text.as_bytes()) {
        let after = pos + literal.len();
        if after >= text.len() {
            continue;
        }

        // Match at least one whitespace
        let rest = &text[after..];
        let mut ws_end = 0;
        for ch in rest.chars() {
            if ch.is_whitespace() {
                ws_end += ch.len_utf8();
            } else {
                break;
            }
        }

        if ws_end > 0 && after + ws_end < text.len() {
            // Check for quoted string after whitespace
            let after_ws = after + ws_end;
            if text.as_bytes()[after_ws] == b'"' {
                // Find closing quote
                if let Some(end) = memchr(b'"', &text.as_bytes()[after_ws + 1..]) {
                    if end > 0 {
                        return Some((pos, after_ws + 1 + end + 1));
                    }
                }
            }
        }
    }

    None
}

/// Fast path for find_all: literal + whitespace + quoted string
#[inline]
pub fn find_literal_ws_quoted_all(text: &str, literal: &str) -> Vec<(usize, usize)> {
    let mut results = Vec::new();
    let finder = memmem::Finder::new(literal.as_bytes());

    for pos in finder.find_iter(text.as_bytes()) {
        let after = pos + literal.len();
        if after >= text.len() {
            continue;
        }

        // Match at least one whitespace
        let rest = &text[after..];
        let mut ws_end = 0;
        for ch in rest.chars() {
            if ch.is_whitespace() {
                ws_end += ch.len_utf8();
            } else {
                break;
            }
        }

        if ws_end > 0 && after + ws_end < text.len() {
            // Check for quoted string after whitespace
            let after_ws = after + ws_end;
            if text.as_bytes()[after_ws] == b'"' {
                // Find closing quote
                if let Some(end) = memchr(b'"', &text.as_bytes()[after_ws + 1..]) {
                    if end > 0 {
                        results.push((pos, after_ws + 1 + end + 1));
                    }
                }
            }
        }
    }

    results
}

/// Fast path for: literal + whitespace + digits (salience\s+\d+)
#[inline]
pub fn find_literal_ws_digits(text: &str, literal: &str) -> Option<(usize, usize)> {
    let finder = memmem::Finder::new(literal.as_bytes());

    for pos in finder.find_iter(text.as_bytes()) {
        let after = pos + literal.len();
        if after >= text.len() {
            continue;
        }

        // Match at least one whitespace
        let rest = &text[after..];
        let bytes = rest.as_bytes();
        let mut ws_end = 0;
        for ch in rest.chars() {
            if ch.is_whitespace() {
                ws_end += ch.len_utf8();
            } else {
                break;
            }
        }

        if ws_end > 0 && ws_end < bytes.len() && bytes[ws_end].is_ascii_digit() {
            // Match digit run
            let mut digit_end = ws_end;
            while digit_end < bytes.len() && bytes[digit_end].is_ascii_digit() {
                digit_end += 1;
            }
            return Some((pos, after + digit_end));
        }
    }

    None
}

/// Fast path for find_all: literal + whitespace + digits
#[inline]
pub fn find_literal_ws_digits_all(text: &str, literal: &str) -> Vec<(usize, usize)> {
    let mut results = Vec::new();
    let finder = memmem::Finder::new(literal.as_bytes());

    for pos in finder.find_iter(text.as_bytes()) {
        let after = pos + literal.len();
        if after >= text.len() {
            continue;
        }


/// Fast path for literal + whitespace + word: when\s+\w+
        // Match at least one whitespace
        let rest = &text[after..];
        let bytes = rest.as_bytes();
        let mut ws_end = 0;
        for ch in rest.chars() {
            if ch.is_whitespace() {
                ws_end += ch.len_utf8();
            } else {
                break;
            }
        }

        if ws_end > 0 && ws_end < bytes.len() && bytes[ws_end].is_ascii_digit() {
            // Match digit run
            let mut digit_end = ws_end;
            while digit_end < bytes.len() && bytes[digit_end].is_ascii_digit() {
                digit_end += 1;
            }
            results.push((pos, after + digit_end));
        }
    }

    results
}

/// Fast path for: word + optional ws + >= + optional ws + digits (\w+\s*>=\s*\d+)
#[inline]
pub fn find_word_compare_digit(text: &str) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_alphanumeric() || b == b'_' {
            let start = i;
            // Match word
            while i < bytes.len() {
                let b = bytes[i];
                if b.is_ascii_alphanumeric() || b == b'_' {
                    i += 1;
                } else {
                    break;
                }
            }

            // Skip optional whitespace
            while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b'\n') {
                i += 1;
            }

            // Check for >=
            if i + 1 < bytes.len() && bytes[i] == b'>' && bytes[i + 1] == b'=' {
                i += 2;

                // Skip optional whitespace
                while i < bytes.len()
                    && (bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b'\n')
                {
                    i += 1;
                }

                // Check for digit
                if i < bytes.len() && bytes[i].is_ascii_digit() {
                    while i < bytes.len() && bytes[i].is_ascii_digit() {
                        i += 1;
                    }
                    return Some((start, i));
                }
            }

            i = start + 1;
        } else {
            i += 1;
        }
    }

    None
}

/// Fast path for find_all: word + optional ws + >= + optional ws + digits
#[inline]
pub fn find_word_compare_digit_all(text: &str) -> Vec<(usize, usize)> {
    let mut results = Vec::new();
    let bytes = text.as_bytes();
    let mut i = 0;

    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_alphanumeric() || b == b'_' {
            let start = i;
            // Match word
            while i < bytes.len() {
                let b = bytes[i];
                if b.is_ascii_alphanumeric() || b == b'_' {
                    i += 1;
                } else {
                    break;
                }
            }

            let word_end = i;
            let saved_i = i; // Save position before checking pattern

            // Skip optional whitespace
            while i < bytes.len() && (bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b'\n') {
                i += 1;
            }

            // Check for >=
            if i + 1 < bytes.len() && bytes[i] == b'>' && bytes[i + 1] == b'=' {
                i += 2;

                // Skip optional whitespace
                while i < bytes.len()
                    && (bytes[i] == b' ' || bytes[i] == b'\t' || bytes[i] == b'\n')
                {
                    i += 1;
                }

                // Check for digit
                if i < bytes.len() && bytes[i].is_ascii_digit() {
                    while i < bytes.len() && bytes[i].is_ascii_digit() {
                        i += 1;
                    }
                    results.push((start, i));
                    continue; // Continue from after the match
                }
            }

            // No match - continue from start + 1 to avoid infinite loop
            i = start + 1;
        } else {
            i += 1;
        }
    }

    results
}

/// Fast path for: alternation of literals (word1|word2|word3)
#[inline]
pub fn find_alternation(ac: &AhoCorasick, text: &str) -> Option<(usize, usize)> {
    ac.find(text).map(|m| (m.start(), m.end()))
}

/// Fast path for find_all: alternation of literals
/// Uses pre-built aho-corasick automaton for O(n + z) performance
#[inline]
pub fn find_alternation_all(ac: &AhoCorasick, text: &str) -> Vec<(usize, usize)> {
    ac.find_iter(text).map(|m| (m.start(), m.end())).collect()
}

/// Detect pattern type and return specialized function

/// Strip simple capture groups for fast path detection
/// Allows patterns like "when\s+(\w+)" to match as "when\s+\w+"
fn strip_simple_captures(pattern: &str) -> String {
    let mut result = String::with_capacity(pattern.len());
    let mut depth = 0;
    let chars: Vec<char> = pattern.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        match chars[i] {
            '(' => {
                // Skip capture group markers but keep content
                depth += 1;
                i += 1;
            }
            ')' => {
                if depth > 0 {
                    depth -= 1;
                }
                i += 1;
            }
            _ => {
                result.push(chars[i]);
                i += 1;
            }
        }
    }
    
    result
}

pub fn detect_fast_path(pattern: &str) -> Option<FastPath> {
    // Don't use fast path for anchored patterns - they need special handling
    if pattern.starts_with('^') || pattern.ends_with('$') {
        return None;
    }

    // Strip captures to allow detection of patterns like "when\s+(\w+)"
    let normalized = strip_simple_captures(pattern);

    // Check for simple literal
    if !normalized.contains(['\\', '[', ']', '(', ')', '*', '+', '?', '{', '}', '|', '.']) {
        return Some(FastPath::Literal(normalized.to_string()));
    }

    // Check for digit run
    if normalized == r"\d+" {
        return Some(FastPath::DigitRun);
    }

    // Check for word run
    if normalized == r"\w+" {
        return Some(FastPath::WordRun);
    }

    // Check for identifier pattern: [a-zA-Z_]\w*
    if normalized == r"[a-zA-Z_]\w*" {
        return Some(FastPath::IdentifierRun);
    }

    // Check for literal + whitespace
    if let Some(rest) = normalized.strip_suffix(r"\s+") {
        if !rest.contains(['\\', '[', ']', '(', ')', '*', '+', '?', '{', '}', '|', '.']) {
            return Some(FastPath::LiteralPlusWhitespace(rest.to_string()));
        }
    }

    // Check for quoted string
    if normalized == r#""[^"]+""# {
        return Some(FastPath::QuotedString);
    }

    // Check for literal + whitespace + quoted string: rule\s+"[^"]+"
    if let Some(mid) = normalized.strip_suffix(r#""[^"]+""#) {
        if let Some(literal) = mid.strip_suffix(r"\s+") {
            if !literal.contains(['\\', '[', ']', '(', ')', '*', '+', '?', '{', '}', '|', '.']) {
                return Some(FastPath::LiteralWhitespaceQuoted(literal.to_string()));
            }
        }
    }

    // Check for literal + whitespace + digits: salience\s+\d+
    if let Some(mid) = normalized.strip_suffix(r"\d+") {
        if let Some(literal) = mid.strip_suffix(r"\s+") {
            if !literal.contains(['\\', '[', ']', '(', ')', '*', '+', '?', '{', '}', '|', '.']) {
                return Some(FastPath::LiteralWhitespaceDigits(literal.to_string()));
            }
        }
    }

    // Check for literal + whitespace + word: when\s+\w+
    if let Some(mid) = normalized.strip_suffix(r"\w+") {
        if let Some(literal) = mid.strip_suffix(r"\s+") {
            if !literal.contains(['\\', '[', ']', '(', ')', '*', '+', '?', '{', '}', '|', '.']) {
                return Some(FastPath::LiteralWhitespaceWord(literal.to_string()));
            }
        }
    }

    // Check for word + optional ws + >= + optional ws + digits: \w+\s*>=\s*\d+
    // DISABLED: Performance regression - use normal matcher instead
    // if pattern == r"\w+\s*>=\s*\d+" {
    //     return Some(FastPath::WordCompareDigit);
    // }

    // Check for simple alternation of literals: word1|word2|word3
    if normalized.contains('|')
        && !pattern.contains(['\\', '[', ']', '(', ')', '*', '+', '?', '{', '}', '.'])
    {
        let alternatives: Vec<String> = normalized.split('|').map(|s| s.to_string()).collect();
        // Only use fast path if all alternatives are simple literals
        if alternatives.iter().all(|alt| !alt.is_empty()) {
            // Pre-build the aho-corasick automaton once during pattern detection
            // Use fastest configuration for small pattern sets
            if let Ok(ac) = AhoCorasick::builder()
                .match_kind(aho_corasick::MatchKind::LeftmostFirst)
                .build(&alternatives)
            {
                return Some(FastPath::Alternation(Arc::new(ac)));
            }
        }
    }

    None
}


// ============================================================================
// Lazy iteration - find_at() helpers
// ============================================================================

/// Find digit run starting from position
#[inline]
pub fn find_digit_run_at(text: &str, start_pos: usize) -> Option<(usize, usize)> {
    if start_pos >= text.len() {
        return None;
    }
    let bytes = &text.as_bytes()[start_pos..];
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_digit() {
            let match_start = start_pos + i;
            let mut end = i;
            while end < bytes.len() && bytes[end].is_ascii_digit() {
                end += 1;
            }
            return Some((match_start, start_pos + end));
        }
        i += 1;
    }
    None
}

/// Find word run starting from position
#[inline]
pub fn find_word_run_at(text: &str, start_pos: usize) -> Option<(usize, usize)> {
    if start_pos >= text.len() {
        return None;
    }
    let bytes = &text.as_bytes()[start_pos..];
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_alphanumeric() || b == b'_' {
            let match_start = start_pos + i;
            let mut end = i;
            while end < bytes.len() {
                let b = bytes[end];
                if b.is_ascii_alphanumeric() || b == b'_' {
                    end += 1;
                } else {
                    break;
                }
            }
            return Some((match_start, start_pos + end));
        }
        i += 1;
    }
    None
}

/// Find identifier starting from position
#[inline]
pub fn find_identifier_run_at(text: &str, start_pos: usize) -> Option<(usize, usize)> {
    if start_pos >= text.len() {
        return None;
    }
    let bytes = &text.as_bytes()[start_pos..];
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_alphabetic() || b == b'_' {
            let match_start = start_pos + i;
            let mut end = i + 1;
            while end < bytes.len() {
                let b = bytes[end];
                if b.is_ascii_alphanumeric() || b == b'_' {
                    end += 1;
                } else {
                    break;
                }
            }
            return Some((match_start, start_pos + end));
        }
        i += 1;
    }
    None
}

/// Find quoted string starting from position
#[inline]
pub fn find_quoted_string_at(text: &str, start_pos: usize) -> Option<(usize, usize)> {
    if start_pos >= text.len() {
        return None;
    }
    let bytes = text.as_bytes();
    let search_bytes = &bytes[start_pos..];
    let first_quote = memchr(b'"', search_bytes)?;
    let quote_pos = start_pos + first_quote;
    if quote_pos + 1 >= bytes.len() {
        return None;
    }
    let closing_quote = memchr(b'"', &bytes[quote_pos + 1..])?;
    if closing_quote == 0 {
        return None;
    }
    Some((quote_pos, quote_pos + 1 + closing_quote + 1))
}

/// Find literal starting from position
#[inline]
pub fn find_literal_at(text: &str, literal: &str, start_pos: usize) -> Option<(usize, usize)> {
    if start_pos >= text.len() {
        return None;
    }
    let search_text = &text[start_pos..];
    let len = literal.len();
    if len >= 3 {
        let finder = memmem::Finder::new(literal.as_bytes());
        finder.find(search_text.as_bytes()).map(|pos| (start_pos + pos, start_pos + pos + len))
    } else if len == 1 {
        let byte = literal.as_bytes()[0];
        memchr(byte, search_text.as_bytes()).map(|pos| (start_pos + pos, start_pos + pos + 1))
    } else {
        search_text.find(literal).map(|pos| (start_pos + pos, start_pos + pos + len))
    }
}

/// Find literal + whitespace starting from position
#[inline]
pub fn find_literal_plus_whitespace_at(text: &str, literal: &str, start_pos: usize) -> Option<(usize, usize)> {
    if start_pos >= text.len() {
        return None;
    }
    let search_text = &text[start_pos..];
    let finder = memmem::Finder::new(literal.as_bytes());
    let lit_pos = finder.find(search_text.as_bytes())?;
    let abs_pos = start_pos + lit_pos;
    let after = abs_pos + literal.len();
    if after >= text.len() {
        return None;
    }
    let rest = &text[after..];
    let mut matched = 0;
    for ch in rest.chars() {
        if ch.is_whitespace() {
            matched += ch.len_utf8();
        } else {
            break;
        }
    }
    if matched > 0 {
        Some((abs_pos, after + matched))
    } else {
        None
    }
}

#[inline]
pub fn find_literal_ws_word(text: &str, literal: &str) -> Option<(usize, usize)> {
    let finder = memmem::Finder::new(literal.as_bytes());
    
    for pos in finder.find_iter(text.as_bytes()) {
        let after = pos + literal.len();
        if after >= text.len() {
            continue;
        }
        
        // Match at least one whitespace
        let rest = &text[after..];
        let mut ws_count = 0;
        for ch in rest.chars() {
            if ch.is_whitespace() {
                ws_count += ch.len_utf8();
            } else {
                break;
            }
        }
        
        if ws_count == 0 {
            continue;
        }
        
        // Match word run
        let word_start = after + ws_count;
        if word_start >= text.len() {
            continue;
        }
        
        let bytes = &text.as_bytes()[word_start..];
        let mut word_len = 0;
        for &b in bytes {
            if b.is_ascii_alphanumeric() || b == b'_' {
                word_len += 1;
            } else {
                break;
            }
        }
        
        if word_len > 0 {
            return Some((pos, word_start + word_len));
        }
    }
    
    None
}

/// Fast path for find_all: literal + whitespace + word
#[inline]
pub fn find_literal_ws_word_all(text: &str, literal: &str) -> Vec<(usize, usize)> {
    let mut results = Vec::new();
    let finder = memmem::Finder::new(literal.as_bytes());
    
    for pos in finder.find_iter(text.as_bytes()) {
        let after = pos + literal.len();
        if after >= text.len() {
            continue;
        }
        
        // Match at least one whitespace
        let rest = &text[after..];
        let mut ws_count = 0;
        for ch in rest.chars() {
            if ch.is_whitespace() {
                ws_count += ch.len_utf8();
            } else {
                break;
            }
        }
        
        if ws_count == 0 {
            continue;
        }
        
        // Match word run
        let word_start = after + ws_count;
        if word_start >= text.len() {
            continue;
        }
        
        let bytes = &text.as_bytes()[word_start..];
        let mut word_len = 0;
        for &b in bytes {
            if b.is_ascii_alphanumeric() || b == b'_' {
                word_len += 1;
            } else {
                break;
            }
        }
        
        if word_len > 0 {
            results.push((pos, word_start + word_len));
        }
    }
    
    results
}

/// Find literal + whitespace + word starting from position
#[inline]
pub fn find_literal_ws_word_at(text: &str, literal: &str, start_pos: usize) -> Option<(usize, usize)> {
    if start_pos >= text.len() {
        return None;
    }
    let search_text = &text[start_pos..];
    find_literal_ws_word(search_text, literal).map(|(rel_start, rel_end)| {
        (start_pos + rel_start, start_pos + rel_end)
    })
}

#[derive(Clone)]
pub enum FastPath {
    Literal(String),
    LiteralPlusWhitespace(String),
    LiteralWhitespaceQuoted(String), // rule\s+"[^"]+"
    LiteralWhitespaceDigits(String), // salience\s+\d+
    LiteralWhitespaceWord(String),   // when\s+\w+
    WordCompareDigit,                // \w+\s*>=\s*\d+
    Alternation(Arc<AhoCorasick>),   // Pre-built automaton for word1|word2|word3
    DigitRun,
    WordRun,
    IdentifierRun, // [a-zA-Z_]\w* - identifier pattern
    QuotedString,
}

impl std::fmt::Debug for FastPath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FastPath::Literal(s) => write!(f, "Literal({:?})", s),
            FastPath::LiteralPlusWhitespace(s) => write!(f, "LiteralPlusWhitespace({:?})", s),
            FastPath::LiteralWhitespaceQuoted(s) => write!(f, "LiteralWhitespaceQuoted({:?})", s),
            FastPath::LiteralWhitespaceDigits(s) => write!(f, "LiteralWhitespaceDigits({:?})", s),
            FastPath::LiteralWhitespaceWord(s) => write!(f, "LiteralWhitespaceWord({:?})", s),
            FastPath::WordCompareDigit => write!(f, "WordCompareDigit"),
            FastPath::Alternation(_) => write!(f, "Alternation(<AhoCorasick>)"),
            FastPath::DigitRun => write!(f, "DigitRun"),
            FastPath::WordRun => write!(f, "WordRun"),
            FastPath::IdentifierRun => write!(f, "IdentifierRun"),
            FastPath::QuotedString => write!(f, "QuotedString"),
        }
    }
}

impl FastPath {
    #[inline]
    pub fn find(&self, text: &str) -> Option<(usize, usize)> {
        match self {
            FastPath::Literal(s) => find_literal(text, s),
            FastPath::LiteralPlusWhitespace(s) => find_literal_plus_whitespace(text, s),
            FastPath::LiteralWhitespaceQuoted(s) => find_literal_ws_quoted(text, s),
            FastPath::LiteralWhitespaceDigits(s) => find_literal_ws_digits(text, s),
            FastPath::LiteralWhitespaceWord(s) => find_literal_ws_word(text, s),
            FastPath::WordCompareDigit => find_word_compare_digit(text),
            FastPath::Alternation(ac) => find_alternation(ac, text),
            FastPath::DigitRun => find_digit_run(text),
            FastPath::WordRun => find_word_run(text),
            FastPath::IdentifierRun => find_identifier_run(text),
            FastPath::QuotedString => find_quoted_string(text),
        }
    }

    #[inline]
    pub fn find_all(&self, text: &str) -> Vec<(usize, usize)> {
        match self {
            FastPath::Literal(s) => find_literal_all(text, s),
            FastPath::LiteralPlusWhitespace(s) => find_literal_plus_whitespace_all(text, s),
            FastPath::LiteralWhitespaceQuoted(s) => find_literal_ws_quoted_all(text, s),
            FastPath::LiteralWhitespaceDigits(s) => find_literal_ws_digits_all(text, s),
            FastPath::LiteralWhitespaceWord(s) => find_literal_ws_word_all(text, s),
            FastPath::WordCompareDigit => find_word_compare_digit_all(text),
            FastPath::Alternation(ac) => find_alternation_all(ac, text),
            FastPath::DigitRun => find_digit_run_all(text),
            FastPath::WordRun => find_word_run_all(text),
            FastPath::IdentifierRun => find_identifier_run_all(text),
            FastPath::QuotedString => find_quoted_string_all(text),
        }
    }

    /// Find next match starting from position (for lazy iteration)
    #[inline]
    pub fn find_at(&self, text: &str, start_pos: usize) -> Option<(usize, usize)> {
        match self {
            FastPath::Literal(s) => find_literal_at(text, s, start_pos),
            FastPath::LiteralPlusWhitespace(s) => find_literal_plus_whitespace_at(text, s, start_pos),
            FastPath::LiteralWhitespaceWord(s) => find_literal_ws_word_at(text, s, start_pos),
            FastPath::DigitRun => find_digit_run_at(text, start_pos),
            FastPath::WordRun => find_word_run_at(text, start_pos),
            FastPath::IdentifierRun => find_identifier_run_at(text, start_pos),
            FastPath::QuotedString => find_quoted_string_at(text, start_pos),
            // For complex patterns, use find() on remaining text
            _ => {
                if start_pos >= text.len() {
                    return None;
                }
                let remaining = &text[start_pos..];
                self.find(remaining).map(|(rel_start, rel_end)| (start_pos + rel_start, start_pos + rel_end))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_fast_paths() {
        assert!(matches!(detect_fast_path(r"\d+"), Some(FastPath::DigitRun)));
        assert!(matches!(detect_fast_path(r"\w+"), Some(FastPath::WordRun)));
        assert!(matches!(
            detect_fast_path("hello"),
            Some(FastPath::Literal(_))
        ));
        assert!(matches!(
            detect_fast_path(r"rule\s+"),
            Some(FastPath::LiteralPlusWhitespace(_))
        ));
    }

    #[test]
    fn test_fast_paths() {
        let text = "rule Test 123 hello";

        // Test digit run
        let fp = FastPath::DigitRun;
        assert_eq!(fp.find(text), Some((10, 13)));

        // Test word run
        let fp = FastPath::WordRun;
        assert_eq!(fp.find(text), Some((0, 4)));

        // Test literal
        let fp = FastPath::Literal("hello".to_string());
        assert_eq!(fp.find(text), Some((14, 19)));
    }
}
