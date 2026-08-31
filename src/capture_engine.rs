//! Capture-aware execution for the compiled matcher tree.
//!
//! Matching and capture extraction must follow the same path. In particular,
//! a failed alternation or a backtracking attempt must not leak slots from the
//! abandoned path into the final `Captures` value. `CaptureState` uses a small
//! undo log so those paths can be rolled back without cloning a capture vector
//! for every branch.

use crate::parser::quantifier::Quantifier;
use crate::{safe_slice, safe_slice_range, CompiledCaptureElement, Matcher};

/// A lowercased haystack with byte-boundary mappings back to the source text.
///
/// Rust's Unicode lowercasing can expand one source character into multiple
/// normalized characters. Capture offsets are only valid at source boundaries,
/// so callers can reject a match whose end falls inside such an expansion.
pub(super) struct CaseFoldedText {
    text: String,
    source_to_folded: Vec<Option<usize>>,
    folded_to_source: Vec<Option<usize>>,
}

impl CaseFoldedText {
    pub(super) fn new(source: &str) -> Self {
        let mut text = String::new();
        let mut source_to_folded = vec![None; source.len() + 1];
        let mut folded_to_source = Vec::new();

        for (source_start, character) in source.char_indices() {
            let source_end = source_start + character.len_utf8();
            let folded_start = text.len();
            let folded = character.to_lowercase().collect::<String>();
            let folded_end = folded_start + folded.len();

            source_to_folded[source_start] = Some(folded_start);
            if folded_to_source.len() <= folded_end {
                folded_to_source.resize(folded_end + 1, None);
            }
            folded_to_source[folded_start] = Some(source_start);
            folded_to_source[folded_end] = Some(source_end);
            text.push_str(&folded);
        }

        source_to_folded[source.len()] = Some(text.len());
        if folded_to_source.len() <= text.len() {
            folded_to_source.resize(text.len() + 1, None);
        }
        folded_to_source[text.len()] = Some(source.len());

        Self {
            text,
            source_to_folded,
            folded_to_source,
        }
    }

    pub(super) fn text(&self) -> &str {
        &self.text
    }

    pub(super) fn folded_offset(&self, source_offset: usize) -> Option<usize> {
        self.source_to_folded
            .get(source_offset)
            .and_then(|&offset| offset)
    }

    pub(super) fn source_positions(
        &self,
        positions: Vec<Option<(usize, usize)>>,
    ) -> Option<Vec<Option<(usize, usize)>>> {
        positions
            .into_iter()
            .map(|position| {
                position.map(|(start, end)| {
                    Some((
                        self.folded_to_source
                            .get(start)
                            .and_then(|&offset| offset)?,
                        self.folded_to_source.get(end).and_then(|&offset| offset)?,
                    ))
                })
            })
            .collect()
    }
}

/// Mutable capture slots for one match attempt.
pub(super) struct CaptureState {
    positions: Vec<Option<(usize, usize)>>,
    undo: Vec<(usize, Option<(usize, usize)>)>,
}

impl CaptureState {
    pub(super) fn new(group_count: usize) -> Self {
        Self {
            positions: vec![None; group_count + 1],
            undo: Vec::new(),
        }
    }

    fn checkpoint(&self) -> usize {
        self.undo.len()
    }

    fn restore(&mut self, checkpoint: usize) {
        while self.undo.len() > checkpoint {
            let (index, previous) = self.undo.pop().expect("undo length was checked");
            self.positions[index] = previous;
        }
    }

    fn set(&mut self, index: usize, value: Option<(usize, usize)>) {
        let Some(slot) = self.positions.get_mut(index) else {
            return;
        };

        if *slot != value {
            self.undo.push((index, *slot));
            *slot = value;
        }
    }

    fn get(&self, index: usize) -> Option<(usize, usize)> {
        self.positions.get(index).and_then(|&position| position)
    }

    pub(super) fn into_positions(
        mut self,
        full_match: (usize, usize),
    ) -> Vec<Option<(usize, usize)>> {
        self.positions[0] = Some(full_match);
        self.positions
    }
}

impl Matcher {
    /// Return the highest capture-group index in this compiled matcher.
    pub(super) fn capture_group_count(&self) -> usize {
        match self {
            Matcher::Capture(inner, group_index) => (*group_index).max(inner.capture_group_count()),
            Matcher::PatternWithCaptures { total_groups, .. }
            | Matcher::AlternationWithCaptures { total_groups, .. } => *total_groups,
            Matcher::AnchoredPattern { inner, .. }
            | Matcher::CaseInsensitive(inner)
            | Matcher::QuantifiedCapture(inner, _)
            | Matcher::Lookaround(_, inner) => inner.capture_group_count(),
            Matcher::CombinedWithLookaround {
                prefix,
                lookaround_matcher,
                ..
            } => prefix
                .capture_group_count()
                .max(lookaround_matcher.capture_group_count()),
            Matcher::LookbehindWithSuffix {
                lookbehind_matcher,
                suffix,
                ..
            } => lookbehind_matcher
                .capture_group_count()
                .max(suffix.capture_group_count()),
            _ => 0,
        }
    }

    /// Match at `start` and write capture slots for the successful path only.
    pub(super) fn match_at_with_captures(
        &self,
        text: &str,
        start: usize,
        captures: &mut CaptureState,
    ) -> Option<usize> {
        let checkpoint = captures.checkpoint();
        let matched = self.match_at_with_captures_inner(text, start, captures);
        if matched.is_none() {
            captures.restore(checkpoint);
        }
        matched
    }

    fn match_at_with_captures_inner(
        &self,
        text: &str,
        start: usize,
        captures: &mut CaptureState,
    ) -> Option<usize> {
        match self {
            Matcher::AnchoredPattern {
                inner,
                start: anchored_start,
                end: anchored_end,
            } => {
                if *anchored_start && start != 0 {
                    return None;
                }

                let end = inner.match_at_with_captures(text, start, captures)?;
                (!*anchored_end || end == text.len()).then_some(end)
            }
            Matcher::Capture(inner, group_index) => {
                Self::match_capture_group_at(inner, *group_index, text, start, captures)
            }
            Matcher::QuantifiedCapture(inner, quantifier) => {
                Self::match_quantified_capture_at(text, start, inner, quantifier, None, captures)
            }
            Matcher::PatternWithCaptures { elements, .. } => {
                Self::match_capture_elements(text, start, elements, captures)
            }
            Matcher::AlternationWithCaptures { branches, .. } => {
                Self::match_capture_alternation_at(text, start, branches, captures)
            }
            Matcher::Backreference(group_index) => {
                let (capture_start, capture_end) = captures.get(*group_index)?;
                let captured = safe_slice_range(text, capture_start, capture_end)?;
                text.get(start..)?
                    .starts_with(captured)
                    .then_some(start + captured.len())
            }
            Matcher::CombinedWithLookaround {
                prefix,
                lookaround,
                lookaround_matcher,
            } => {
                let end = prefix.match_at_with_captures(text, start, captures)?;
                lookaround
                    .matches_at(text, end, lookaround_matcher)
                    .then_some(end)
            }
            Matcher::LookbehindWithSuffix {
                lookbehind,
                lookbehind_matcher,
                suffix,
            } => lookbehind
                .matches_at(text, start, lookbehind_matcher)
                .then(|| suffix.match_at_with_captures(text, start, captures))
                .flatten(),
            Matcher::CaseInsensitive(inner) if text.is_ascii() => {
                let lower_text = text.to_ascii_lowercase();
                inner.match_at_with_captures(&lower_text, start, captures)
            }
            // The existing case-insensitive matcher has Unicode semantics that
            // can change byte offsets. Preserve its exact matching behaviour
            // here; ASCII input takes the capture-aware path above.
            Matcher::CaseInsensitive(_) => self.match_at(text, start),
            _ => self.match_at(text, start),
        }
    }

    /// Match `self` against precisely `text[start..end]`, recording captures.
    fn matches_entire_with_captures(
        &self,
        text: &str,
        start: usize,
        end: usize,
        captures: &mut CaptureState,
    ) -> bool {
        let checkpoint = captures.checkpoint();
        let matched = self.matches_entire_with_captures_inner(text, start, end, captures);
        if !matched {
            captures.restore(checkpoint);
        }
        matched
    }

    fn matches_entire_with_captures_inner(
        &self,
        text: &str,
        start: usize,
        end: usize,
        captures: &mut CaptureState,
    ) -> bool {
        if safe_slice_range(text, start, end).is_none() {
            return false;
        }

        match self {
            Matcher::AnchoredPattern {
                inner,
                start: anchored_start,
                end: anchored_end,
            } => {
                (!*anchored_start || start == 0)
                    && (!*anchored_end || end == text.len())
                    && inner.matches_entire_with_captures(text, start, end, captures)
            }
            Matcher::Capture(inner, group_index) => {
                Self::matches_capture_group_entire(inner, *group_index, text, start, end, captures)
            }
            Matcher::QuantifiedCapture(inner, quantifier) => Self::match_quantified_capture_entire(
                text, start, end, inner, quantifier, None, captures,
            ),
            Matcher::PatternWithCaptures { elements, .. } => {
                Self::match_capture_elements_entire(text, start, end, elements, captures)
            }
            Matcher::AlternationWithCaptures { branches, .. } => {
                Self::match_capture_alternation_entire(text, start, end, branches, captures)
            }
            Matcher::Backreference(group_index) => captures
                .get(*group_index)
                .and_then(|(capture_start, capture_end)| {
                    safe_slice_range(text, capture_start, capture_end)
                })
                .is_some_and(|captured| safe_slice_range(text, start, end) == Some(captured)),
            Matcher::MultiLiteral { alternatives, .. } => safe_slice_range(text, start, end)
                .is_some_and(|matched| {
                    alternatives
                        .iter()
                        .any(|alternative| alternative == matched)
                }),
            Matcher::Quantified(_) => safe_slice_range(text, start, end)
                .is_some_and(|matched| Self::matches_entire(self, matched)),
            Matcher::CaseInsensitive(inner) if text.is_ascii() => {
                let lower_text = text.to_ascii_lowercase();
                inner.matches_entire_with_captures(&lower_text, start, end, captures)
            }
            Matcher::CaseInsensitive(_) => self.match_at(text, start) == Some(end),
            _ => self.match_at(text, start) == Some(end),
        }
    }

    fn match_capture_group_at(
        inner: &Matcher,
        group_index: usize,
        text: &str,
        start: usize,
        captures: &mut CaptureState,
    ) -> Option<usize> {
        let checkpoint = captures.checkpoint();

        let matched = match inner {
            Matcher::QuantifiedCapture(quantified_inner, quantifier) => {
                Self::match_quantified_capture_at(
                    text,
                    start,
                    quantified_inner,
                    quantifier,
                    Some(group_index),
                    captures,
                )
            }
            _ => {
                let end = inner.match_at_with_captures(text, start, captures)?;
                captures.set(group_index, Some((start, end)));
                Some(end)
            }
        };

        if matched.is_none() {
            captures.restore(checkpoint);
        }
        matched
    }

    fn matches_capture_group_entire(
        inner: &Matcher,
        group_index: usize,
        text: &str,
        start: usize,
        end: usize,
        captures: &mut CaptureState,
    ) -> bool {
        let checkpoint = captures.checkpoint();

        let matched = match inner {
            Matcher::QuantifiedCapture(quantified_inner, quantifier) => {
                Self::match_quantified_capture_entire(
                    text,
                    start,
                    end,
                    quantified_inner,
                    quantifier,
                    Some(group_index),
                    captures,
                )
            }
            _ => {
                if inner.matches_entire_with_captures(text, start, end, captures) {
                    captures.set(group_index, Some((start, end)));
                    true
                } else {
                    false
                }
            }
        };

        if !matched {
            captures.restore(checkpoint);
        }
        matched
    }

    fn match_capture_elements(
        text: &str,
        start: usize,
        elements: &[CompiledCaptureElement],
        captures: &mut CaptureState,
    ) -> Option<usize> {
        let checkpoint = captures.checkpoint();
        let matched = match elements.split_first() {
            None => Some(start),
            Some((first, rest)) if !rest.is_empty() && Self::element_has_alternatives(first) => {
                Self::match_capture_element_alternatives(text, start, first, rest, captures)
            }
            Some((first, rest)) if !rest.is_empty() && Self::element_needs_backtracking(first) => {
                let matcher = Self::element_matcher(first);
                let remaining = safe_slice(text, start)?;
                let prefers_lazy = Self::prefers_lazy_backtracking(matcher);

                let mut result = None;
                for length in Self::backtracking_lengths(remaining, prefers_lazy) {
                    let end = start + length;
                    let candidate_checkpoint = captures.checkpoint();

                    if Self::capture_element_matches_entire(first, text, start, end, captures) {
                        if let Some(final_end) =
                            Self::match_capture_elements(text, end, rest, captures)
                        {
                            result = Some(final_end);
                            break;
                        }
                    }

                    captures.restore(candidate_checkpoint);
                }
                result
            }
            Some((first, rest)) => {
                let end = Self::match_capture_element_at(first, text, start, captures)?;
                Self::match_capture_elements(text, end, rest, captures)
            }
        };

        if matched.is_none() {
            captures.restore(checkpoint);
        }
        matched
    }

    /// Match a capture-element sequence against one exact range.
    ///
    /// This is distinct from the normal prefix matcher because a quantified
    /// final element may need to consume less than its standalone greedy (or
    /// lazy) match to fill the exact range selected by an outer backtrack.
    fn match_capture_elements_entire(
        text: &str,
        start: usize,
        end: usize,
        elements: &[CompiledCaptureElement],
        captures: &mut CaptureState,
    ) -> bool {
        let checkpoint = captures.checkpoint();
        let matched = match elements.split_first() {
            None => start == end,
            Some((first, rest)) if Self::element_has_alternatives(first) => {
                Self::match_capture_element_alternatives_entire(
                    text, start, end, first, rest, captures,
                )
            }
            Some((first, rest)) if Self::element_needs_backtracking(first) => {
                let matcher = Self::element_matcher(first);
                let Some(remaining) = safe_slice_range(text, start, end) else {
                    captures.restore(checkpoint);
                    return false;
                };
                let prefers_lazy = Self::prefers_lazy_backtracking(matcher);

                let mut result = false;
                for length in Self::backtracking_lengths(remaining, prefers_lazy) {
                    let next = start + length;
                    let candidate_checkpoint = captures.checkpoint();
                    if Self::capture_element_matches_entire(first, text, start, next, captures)
                        && Self::match_capture_elements_entire(text, next, end, rest, captures)
                    {
                        result = true;
                        break;
                    }
                    captures.restore(candidate_checkpoint);
                }
                result
            }
            Some((first, rest)) => Self::match_capture_element_at(first, text, start, captures)
                .is_some_and(|next| {
                    next <= end
                        && Self::match_capture_elements_entire(text, next, end, rest, captures)
                }),
        };

        if !matched {
            captures.restore(checkpoint);
        }
        matched
    }

    fn element_matcher(element: &CompiledCaptureElement) -> &Matcher {
        match element {
            CompiledCaptureElement::Capture(matcher, _)
            | CompiledCaptureElement::NonCapture(matcher) => matcher,
        }
    }

    fn element_needs_backtracking(element: &CompiledCaptureElement) -> bool {
        Self::contains_quantified(Self::element_matcher(element))
    }

    fn element_has_alternatives(element: &CompiledCaptureElement) -> bool {
        matches!(
            Self::element_matcher(element),
            Matcher::MultiLiteral { .. } | Matcher::AlternationWithCaptures { .. }
        )
    }

    fn match_capture_element_alternatives(
        text: &str,
        start: usize,
        element: &CompiledCaptureElement,
        rest: &[CompiledCaptureElement],
        captures: &mut CaptureState,
    ) -> Option<usize> {
        let checkpoint = captures.checkpoint();

        match element {
            CompiledCaptureElement::Capture(
                Matcher::MultiLiteral { alternatives, .. },
                group_index,
            ) => {
                for alternative in alternatives {
                    let branch_checkpoint = captures.checkpoint();
                    if text.get(start..)?.starts_with(alternative) {
                        let end = start + alternative.len();
                        captures.set(*group_index, Some((start, end)));
                        if let Some(final_end) =
                            Self::match_capture_elements(text, end, rest, captures)
                        {
                            return Some(final_end);
                        }
                    }
                    captures.restore(branch_checkpoint);
                }
            }
            CompiledCaptureElement::NonCapture(Matcher::MultiLiteral { alternatives, .. }) => {
                for alternative in alternatives {
                    let branch_checkpoint = captures.checkpoint();
                    if text.get(start..)?.starts_with(alternative) {
                        let end = start + alternative.len();
                        if let Some(final_end) =
                            Self::match_capture_elements(text, end, rest, captures)
                        {
                            return Some(final_end);
                        }
                    }
                    captures.restore(branch_checkpoint);
                }
            }
            CompiledCaptureElement::Capture(
                Matcher::AlternationWithCaptures { branches, .. },
                group_index,
            ) => {
                for branch in branches {
                    let branch_checkpoint = captures.checkpoint();
                    if let Some(end) = branch.match_at_with_captures(text, start, captures) {
                        captures.set(*group_index, Some((start, end)));
                        if let Some(final_end) =
                            Self::match_capture_elements(text, end, rest, captures)
                        {
                            return Some(final_end);
                        }
                    }
                    captures.restore(branch_checkpoint);
                }
            }
            CompiledCaptureElement::NonCapture(Matcher::AlternationWithCaptures {
                branches,
                ..
            }) => {
                for branch in branches {
                    let branch_checkpoint = captures.checkpoint();
                    if let Some(end) = branch.match_at_with_captures(text, start, captures) {
                        if let Some(final_end) =
                            Self::match_capture_elements(text, end, rest, captures)
                        {
                            return Some(final_end);
                        }
                    }
                    captures.restore(branch_checkpoint);
                }
            }
            _ => unreachable!("alternative matcher was checked before dispatch"),
        }

        captures.restore(checkpoint);
        None
    }

    fn match_capture_element_alternatives_entire(
        text: &str,
        start: usize,
        end: usize,
        element: &CompiledCaptureElement,
        rest: &[CompiledCaptureElement],
        captures: &mut CaptureState,
    ) -> bool {
        let checkpoint = captures.checkpoint();

        let matched = match element {
            CompiledCaptureElement::Capture(
                Matcher::MultiLiteral { alternatives, .. },
                group_index,
            ) => alternatives.iter().any(|alternative| {
                let branch_checkpoint = captures.checkpoint();
                let matched = text.get(start..).is_some_and(|remaining| {
                    remaining.starts_with(alternative) && {
                        let next = start + alternative.len();
                        captures.set(*group_index, Some((start, next)));
                        Self::match_capture_elements_entire(text, next, end, rest, captures)
                    }
                });
                if !matched {
                    captures.restore(branch_checkpoint);
                }
                matched
            }),
            CompiledCaptureElement::NonCapture(Matcher::MultiLiteral { alternatives, .. }) => {
                alternatives.iter().any(|alternative| {
                    let branch_checkpoint = captures.checkpoint();
                    let matched = text.get(start..).is_some_and(|remaining| {
                        remaining.starts_with(alternative)
                            && Self::match_capture_elements_entire(
                                text,
                                start + alternative.len(),
                                end,
                                rest,
                                captures,
                            )
                    });
                    if !matched {
                        captures.restore(branch_checkpoint);
                    }
                    matched
                })
            }
            CompiledCaptureElement::Capture(
                Matcher::AlternationWithCaptures { branches, .. },
                group_index,
            ) => branches.iter().any(|branch| {
                let branch_checkpoint = captures.checkpoint();
                let matched = branch
                    .match_at_with_captures(text, start, captures)
                    .is_some_and(|next| {
                        captures.set(*group_index, Some((start, next)));
                        Self::match_capture_elements_entire(text, next, end, rest, captures)
                    });
                if !matched {
                    captures.restore(branch_checkpoint);
                }
                matched
            }),
            CompiledCaptureElement::NonCapture(Matcher::AlternationWithCaptures {
                branches,
                ..
            }) => branches.iter().any(|branch| {
                let branch_checkpoint = captures.checkpoint();
                let matched = branch
                    .match_at_with_captures(text, start, captures)
                    .is_some_and(|next| {
                        Self::match_capture_elements_entire(text, next, end, rest, captures)
                    });
                if !matched {
                    captures.restore(branch_checkpoint);
                }
                matched
            }),
            _ => unreachable!("alternative matcher was checked before dispatch"),
        };

        if !matched {
            captures.restore(checkpoint);
        }
        matched
    }

    fn match_capture_element_at(
        element: &CompiledCaptureElement,
        text: &str,
        start: usize,
        captures: &mut CaptureState,
    ) -> Option<usize> {
        match element {
            CompiledCaptureElement::Capture(matcher, group_index) => {
                Self::match_capture_group_at(matcher, *group_index, text, start, captures)
            }
            CompiledCaptureElement::NonCapture(matcher) => {
                matcher.match_at_with_captures(text, start, captures)
            }
        }
    }

    fn capture_element_matches_entire(
        element: &CompiledCaptureElement,
        text: &str,
        start: usize,
        end: usize,
        captures: &mut CaptureState,
    ) -> bool {
        match element {
            CompiledCaptureElement::Capture(matcher, group_index) => {
                Self::matches_capture_group_entire(
                    matcher,
                    *group_index,
                    text,
                    start,
                    end,
                    captures,
                )
            }
            CompiledCaptureElement::NonCapture(matcher) => {
                matcher.matches_entire_with_captures(text, start, end, captures)
            }
        }
    }

    fn match_capture_alternation_at(
        text: &str,
        start: usize,
        branches: &[Matcher],
        captures: &mut CaptureState,
    ) -> Option<usize> {
        let checkpoint = captures.checkpoint();
        for branch in branches {
            captures.restore(checkpoint);
            if let Some(end) = branch.match_at_with_captures(text, start, captures) {
                return Some(end);
            }
        }
        captures.restore(checkpoint);
        None
    }

    fn match_capture_alternation_entire(
        text: &str,
        start: usize,
        end: usize,
        branches: &[Matcher],
        captures: &mut CaptureState,
    ) -> bool {
        let checkpoint = captures.checkpoint();
        for branch in branches {
            captures.restore(checkpoint);
            if branch.matches_entire_with_captures(text, start, end, captures) {
                return true;
            }
        }
        captures.restore(checkpoint);
        false
    }

    fn match_quantified_capture_at(
        text: &str,
        start: usize,
        inner: &Matcher,
        quantifier: &Quantifier,
        capture_group: Option<usize>,
        captures: &mut CaptureState,
    ) -> Option<usize> {
        let remaining = safe_slice(text, start)?;
        let checkpoint = captures.checkpoint();

        for length in Self::backtracking_lengths(remaining, quantifier.is_lazy()) {
            let end = start + length;
            if Self::match_quantified_capture_entire(
                text,
                start,
                end,
                inner,
                quantifier,
                capture_group,
                captures,
            ) {
                return Some(end);
            }
            captures.restore(checkpoint);
        }

        None
    }

    fn match_quantified_capture_entire(
        text: &str,
        start: usize,
        end: usize,
        inner: &Matcher,
        quantifier: &Quantifier,
        capture_group: Option<usize>,
        captures: &mut CaptureState,
    ) -> bool {
        let checkpoint = captures.checkpoint();

        let (min, max) = super::quantifier_bounds(quantifier);
        let matched = Self::match_quantified_capture_repetitions(
            text,
            start,
            end,
            0,
            min,
            max,
            quantifier.is_lazy(),
            inner,
            capture_group,
            captures,
        );
        if !matched {
            captures.restore(checkpoint);
        }
        matched
    }

    #[allow(clippy::too_many_arguments)]
    fn match_quantified_capture_repetitions(
        text: &str,
        position: usize,
        end: usize,
        count: usize,
        min: usize,
        max: usize,
        prefers_lazy: bool,
        inner: &Matcher,
        capture_group: Option<usize>,
        captures: &mut CaptureState,
    ) -> bool {
        if position == end && count >= min {
            return true;
        }
        if count == max || position > end {
            return false;
        }

        let Some(remaining) = safe_slice_range(text, position, end) else {
            return false;
        };

        for length in Self::backtracking_lengths(remaining, prefers_lazy) {
            let next = position + length;
            // A zero-width repetition cannot make progress through a non-empty
            // range. It is only useful while satisfying a minimum count at end.
            if next == position && position != end {
                continue;
            }

            let checkpoint = captures.checkpoint();
            if inner.matches_entire_with_captures(text, position, next, captures) {
                if let Some(group_index) = capture_group {
                    captures.set(group_index, Some((position, next)));
                }
                if Self::match_quantified_capture_repetitions(
                    text,
                    next,
                    end,
                    count + 1,
                    min,
                    max,
                    prefers_lazy,
                    inner,
                    capture_group,
                    captures,
                ) {
                    return true;
                }
            }
            captures.restore(checkpoint);
        }

        false
    }
}
