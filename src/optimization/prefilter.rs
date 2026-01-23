use crate::optimization::literal::LiteralSet;
/// Prefilter using extracted literals for fast candidate finding
///
/// A prefilter quickly finds candidate match positions using literal search,
/// then the full regex engine verifies each candidate.
use memchr::{memchr, memmem};

/// A prefilter that uses literal search to find candidates
pub struct Prefilter {
    strategy: PrefilterStrategy,
}

enum PrefilterStrategy {
    /// Single byte search using memchr
    SingleByte(u8),
    /// Single string search using memmem
    SingleString(memmem::Finder<'static>),
    /// Multiple string search using aho-corasick
    MultiString {
        searcher: aho_corasick::AhoCorasick,
        patterns: Vec<String>,
    },
    /// No prefilter available
    None,
}

// Manual Debug implementation
impl std::fmt::Debug for Prefilter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.strategy {
            PrefilterStrategy::SingleByte(b) => f
                .debug_struct("Prefilter")
                .field("strategy", &"SingleByte")
                .field("byte", b)
                .finish(),
            PrefilterStrategy::SingleString(_) => f
                .debug_struct("Prefilter")
                .field("strategy", &"SingleString")
                .finish(),
            PrefilterStrategy::MultiString { patterns, .. } => f
                .debug_struct("Prefilter")
                .field("strategy", &"MultiString")
                .field("patterns", patterns)
                .finish(),
            PrefilterStrategy::None => f
                .debug_struct("Prefilter")
                .field("strategy", &"None")
                .finish(),
        }
    }
}

// Manual Clone implementation
impl Clone for Prefilter {
    fn clone(&self) -> Self {
        match &self.strategy {
            PrefilterStrategy::SingleByte(b) => Prefilter {
                strategy: PrefilterStrategy::SingleByte(*b),
            },
            PrefilterStrategy::SingleString(finder) => {
                // Recreate the finder from the pattern it was built with
                // This is a workaround since Finder doesn't implement Clone
                let pattern = finder.needle();
                let new_finder = memmem::Finder::new(pattern).into_owned();
                Prefilter {
                    strategy: PrefilterStrategy::SingleString(new_finder),
                }
            }
            PrefilterStrategy::MultiString { patterns, .. } => {
                // Rebuild the aho-corasick automaton
                let searcher = aho_corasick::AhoCorasick::builder()
                    .match_kind(aho_corasick::MatchKind::LeftmostLongest)
                    .build(patterns)
                    .expect("Failed to rebuild aho-corasick in clone");
                Prefilter {
                    strategy: PrefilterStrategy::MultiString {
                        searcher,
                        patterns: patterns.clone(),
                    },
                }
            }
            PrefilterStrategy::None => Prefilter {
                strategy: PrefilterStrategy::None,
            },
        }
    }
}

impl Prefilter {
    /// Create a prefilter from extracted literals
    pub fn from_literals(literals: &LiteralSet) -> Self {
        if literals.is_empty() {
            return Prefilter {
                strategy: PrefilterStrategy::None,
            };
        }

        // Single literal case
        if literals.literals.len() == 1 {
            let text = &literals.literals[0].text;

            // Use memchr for single byte
            if text.len() == 1 {
                let byte = text.as_bytes()[0];
                return Prefilter {
                    strategy: PrefilterStrategy::SingleByte(byte),
                };
            }

            // Use memmem for single string
            let finder = memmem::Finder::new(text).into_owned();
            return Prefilter {
                strategy: PrefilterStrategy::SingleString(finder),
            };
        }

        // Use common prefix if available and long enough
        if let Some(prefix) = literals.longest_common_prefix() {
            if prefix.len() >= 3 {
                if prefix.len() == 1 {
                    let byte = prefix.as_bytes()[0];
                    return Prefilter {
                        strategy: PrefilterStrategy::SingleByte(byte),
                    };
                }

                let finder = memmem::Finder::new(prefix).into_owned();
                return Prefilter {
                    strategy: PrefilterStrategy::SingleString(finder),
                };
            }
        }

        // Multiple literals - use aho-corasick if count is reasonable
        if literals.literals.len() <= 100 {
            let patterns: Vec<String> = literals
                .literals
                .iter()
                .map(|lit| lit.text.clone())
                .collect();

            if let Ok(searcher) = aho_corasick::AhoCorasick::builder()
                .match_kind(aho_corasick::MatchKind::LeftmostLongest)
                .build(&patterns)
            {
                return Prefilter {
                    strategy: PrefilterStrategy::MultiString { searcher, patterns },
                };
            }
        }

        // Fallback to no prefilter
        Prefilter {
            strategy: PrefilterStrategy::None,
        }
    }

    /// Check if this prefilter is available
    pub fn is_available(&self) -> bool {
        !matches!(self.strategy, PrefilterStrategy::None)
    }

    /// Find the next candidate position starting from `from`
    /// Returns the position where a candidate starts, or None if no more candidates
    pub fn find_candidate(&self, haystack: &[u8], from: usize) -> Option<usize> {
        if from >= haystack.len() {
            return None;
        }

        match &self.strategy {
            PrefilterStrategy::SingleByte(byte) => {
                memchr(*byte, &haystack[from..]).map(|pos| from + pos)
            }

            PrefilterStrategy::SingleString(finder) => {
                finder.find(&haystack[from..]).map(|pos| from + pos)
            }

            PrefilterStrategy::MultiString { searcher, .. } => {
                searcher.find(&haystack[from..]).map(|m| from + m.start())
            }

            PrefilterStrategy::None => Some(from),
        }
    }

    /// Iterate over all candidate positions
    pub fn candidates<'a>(&'a self, haystack: &'a [u8]) -> CandidateIter<'a> {
        CandidateIter {
            prefilter: self,
            haystack,
            pos: 0,
        }
    }
}

/// Iterator over candidate positions
pub struct CandidateIter<'a> {
    prefilter: &'a Prefilter,
    haystack: &'a [u8],
    pos: usize,
}

impl<'a> Iterator for CandidateIter<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<usize> {
        let candidate = self.prefilter.find_candidate(self.haystack, self.pos)?;
        self.pos = candidate + 1;
        Some(candidate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::optimization::literal::{Literal, LiteralKind, LiteralSet};

    #[test]
    fn test_single_byte_prefilter() {
        let mut literals = LiteralSet::empty();
        literals.literals.push(Literal {
            text: "@".to_string(),
            is_exact: false,
        });
        literals.kind = LiteralKind::Inner;

        let prefilter = Prefilter::from_literals(&literals);
        assert!(prefilter.is_available());

        let haystack = b"foo@example.com bar@test.org";
        let candidates: Vec<usize> = prefilter.candidates(haystack).collect();
        assert_eq!(candidates, vec![3, 19]);
    }

    #[test]
    fn test_single_string_prefilter() {
        let mut literals = LiteralSet::empty();
        literals.literals.push(Literal {
            text: "http".to_string(),
            is_exact: false,
        });
        literals.kind = LiteralKind::Prefix;

        let prefilter = Prefilter::from_literals(&literals);
        assert!(prefilter.is_available());

        let haystack = b"Visit https://example.com or http://test.org";
        let candidates: Vec<usize> = prefilter.candidates(haystack).collect();
        // "http" appears at position 6 (in "https") and 29 (in "http://")
        assert_eq!(candidates, vec![6, 29]);
    }

    #[test]
    fn test_multi_string_prefilter() {
        let mut literals = LiteralSet::empty();
        literals.literals.push(Literal {
            text: "foo".to_string(),
            is_exact: true,
        });
        literals.literals.push(Literal {
            text: "bar".to_string(),
            is_exact: true,
        });
        literals.kind = LiteralKind::Prefix;

        let prefilter = Prefilter::from_literals(&literals);
        assert!(prefilter.is_available());

        let haystack = b"foo bar baz foo";
        let candidates: Vec<usize> = prefilter.candidates(haystack).collect();
        assert_eq!(candidates, vec![0, 4, 12]);
    }
}
