//! Deterministic capture programs for ASCII sequences with disjoint delimiters.
//! Ambiguous expressions retain the general capture executor.

use crate::parser::group::GroupContent;
use crate::parser::quantifier::{QuantifiedElement, Quantifier};
use crate::parser::sequence::{Anchor, SequenceElement};
use crate::parser::{BoundaryType, CharClass, Group};
use crate::{Ast, CaptureElement};

#[derive(Debug)]
pub(crate) struct Program {
    ops: Vec<Op>,
    span_ops: Option<Vec<Op>>,
    pub(crate) unicode_safe: bool,
}

#[derive(Debug, Clone)]
enum Op {
    Literal(String),
    Run {
        mask: [u64; 2],
        range: Option<(u8, u8)>,
        min: usize,
        max: usize,
        lazy: bool,
    },
    Open(usize),
    Close(usize),
    Boundary(BoundaryType),
    Anchor(Anchor),
}

fn class_mask(class: &CharClass) -> [u64; 2] {
    if let Some(bitmap) = class.get_ascii_bitmap() {
        return if class.negated {
            [!bitmap[0], !bitmap[1]]
        } else {
            *bitmap
        };
    }
    let mut mask = [0; 2];
    for byte in 0..128u8 {
        if class.matches(byte as char) {
            mask[byte as usize / 64] |= 1 << (byte % 64);
        }
    }
    mask
}

fn contains(mask: &[u64; 2], byte: u8) -> bool {
    byte < 128 && mask[byte as usize / 64] & (1 << (byte % 64)) != 0
}

impl Program {
    pub(crate) fn group_count(&self) -> usize {
        self.ops
            .iter()
            .filter_map(|op| match op {
                Op::Open(index) => Some(*index),
                _ => None,
            })
            .max()
            .unwrap_or(0)
    }

    /// Select a mandatory internal literal with uniquely reversible preceding
    /// atoms. Disjoint delimiters prevent a backwards run swallowing an earlier
    /// literal; fixed repetitions have a single possible width.
    pub(crate) fn anchor(&self) -> Option<(usize, &str)> {
        let (index, value) = self
            .ops
            .iter()
            .enumerate()
            .filter_map(|(i, op)| match op {
                Op::Literal(value) if !value.is_empty() => Some((i, value.as_str())),
                _ => None,
            })
            .max_by_key(|(_, value)| value.len())?;
        for i in 0..index {
            match &self.ops[i] {
                Op::Literal(_) | Op::Open(_) | Op::Close(_) => {}
                Op::Run { mask, min, max, .. } => {
                    if min == max {
                        continue;
                    }
                    for previous in self.ops[..i].iter().rev() {
                        match previous {
                            Op::Open(_) | Op::Close(_) => continue,
                            Op::Literal(value) if value.is_empty() => continue,
                            Op::Literal(value) if !contains(mask, *value.as_bytes().last()?) => {
                                break
                            }
                            Op::Run { mask: other, .. }
                                if mask[0] & other[0] == 0 && mask[1] & other[1] == 0 =>
                            {
                                break
                            }
                            _ => return None,
                        }
                    }
                }
                _ => return None,
            }
        }
        Some((index, value))
    }

    pub(crate) fn candidate_start(
        &self,
        text: &str,
        anchor: usize,
        mut pos: usize,
    ) -> Option<usize> {
        let bytes = text.as_bytes();
        for op in self.ops[..anchor].iter().rev() {
            match op {
                Op::Open(_) | Op::Close(_) => {}
                Op::Literal(value) => {
                    let start = pos.checked_sub(value.len())?;
                    if bytes.get(start..pos)? != value.as_bytes() {
                        return None;
                    }
                    pos = start;
                }
                Op::Run { mask, min, max, .. } => {
                    let end = pos;
                    while pos > 0 && end - pos < *max && contains(mask, bytes[pos - 1]) {
                        pos -= 1;
                    }
                    if end - pos < *min {
                        return None;
                    }
                }
                _ => return None,
            }
        }
        Some(pos)
    }

    pub(crate) fn compile(ast: &Ast) -> Option<Self> {
        let mut program = Self {
            ops: Vec::new(),
            span_ops: None,
            unicode_safe: true,
        };
        program.ast(ast)?;
        for (i, op) in program.ops.iter().enumerate() {
            if let Op::Run { mask, min, max, .. } = op {
                if min == max {
                    continue;
                }
                for next in &program.ops[i + 1..] {
                    match next {
                        Op::Open(_) | Op::Close(_) => continue,
                        Op::Literal(literal) if literal.is_empty() => continue,
                        Op::Literal(literal) if !contains(mask, literal.as_bytes()[0]) => break,
                        Op::Run {
                            mask: next_mask,
                            min,
                            ..
                        } if *min > 0
                            && mask[0] & next_mask[0] == 0
                            && mask[1] & next_mask[1] == 0 =>
                        {
                            break
                        }
                        _ => return None,
                    }
                }
            }
        }
        // A lazy run before a proven disjoint delimiter must exhaust its
        // characters. Resolve this at compile time instead of scanning tags
        // and following instructions for every candidate.
        for i in 0..program.ops.len() {
            let trailing = program.ops[i + 1..]
                .iter()
                .all(|op| matches!(op, Op::Open(_) | Op::Close(_)));
            if let Op::Run { lazy, .. } = &mut program.ops[i] {
                *lazy &= trailing;
            }
        }
        if program
            .ops
            .iter()
            .any(|op| matches!(op, Op::Open(_) | Op::Close(_)))
        {
            // Numbered groups do not affect acceptance in this deterministic
            // subset. Span-only queries can execute without capture opcodes.
            program.span_ops = Some(
                program
                    .ops
                    .iter()
                    .filter(|op| !matches!(op, Op::Open(_) | Op::Close(_)))
                    .cloned()
                    .collect(),
            );
        }
        Some(program)
    }

    fn literal(&mut self, value: &str) {
        match self.ops.last_mut() {
            Some(Op::Literal(previous)) => previous.push_str(value),
            _ => self.ops.push(Op::Literal(value.to_owned())),
        }
    }

    fn run_char(&mut self, ch: char, quantifier: Quantifier) {
        let mut mask = [0; 2];
        if ch.is_ascii() {
            mask[ch as usize / 64] = 1 << (ch as u32 % 64);
        }
        self.run(mask, ch.is_ascii(), quantifier);
    }

    fn run_class(&mut self, class: &CharClass, quantifier: Quantifier) {
        let unicode_safe = !class.negated
            && class.chars.iter().all(char::is_ascii)
            && class.ranges.iter().all(|(_, end)| end.is_ascii());
        self.run(class_mask(class), unicode_safe, quantifier);
    }

    fn run(&mut self, mask: [u64; 2], unicode_safe: bool, quantifier: Quantifier) {
        self.unicode_safe &= unicode_safe;
        let bits = (u128::from(mask[1]) << 64) | u128::from(mask[0]);
        let range = (bits != 0
            && bits.leading_zeros() + bits.trailing_zeros() + bits.count_ones() == 128)
            .then(|| {
                (
                    bits.trailing_zeros() as u8,
                    (127 - bits.leading_zeros()) as u8,
                )
            });
        self.ops.push(Op::Run {
            mask,
            range,
            min: quantifier.min_matches(),
            max: quantifier.max_matches(),
            lazy: quantifier.is_lazy(),
        });
    }

    fn group(&mut self, group: &Group) -> Option<()> {
        if group.quantifier.is_some() {
            return None;
        }
        match &group.content {
            GroupContent::Single(value) => self.literal(value),
            GroupContent::Sequence(sequence) => self.sequence(&sequence.elements)?,
            _ => return None,
        }
        Some(())
    }

    fn sequence(&mut self, elements: &[SequenceElement]) -> Option<()> {
        for element in elements {
            match element {
                SequenceElement::Char(ch) => self.literal(ch.encode_utf8(&mut [0; 4])),
                SequenceElement::Literal(value) => self.literal(value),
                SequenceElement::CharClass(class) => self.run_class(class, Quantifier::Exactly(1)),
                SequenceElement::QuantifiedChar(ch, q) => self.run_char(*ch, *q),
                SequenceElement::QuantifiedCharClass(class, q) => self.run_class(class, *q),
                SequenceElement::Group(group) => self.group(group)?,
                SequenceElement::Boundary(boundary) => self.ops.push(Op::Boundary(*boundary)),
                SequenceElement::Anchor(anchor) => self.ops.push(Op::Anchor(*anchor)),
                _ => return None,
            }
        }
        Some(())
    }

    fn ast(&mut self, ast: &Ast) -> Option<()> {
        match ast {
            Ast::Literal(value) => self.literal(value),
            Ast::CharClass(class) => self.run_class(class, Quantifier::Exactly(1)),
            Ast::Quantified(pattern) => match &pattern.element {
                QuantifiedElement::Char(ch) => self.run_char(*ch, pattern.quantifier),
                QuantifiedElement::CharClass(class) => self.run_class(class, pattern.quantifier),
            },
            Ast::Sequence(sequence) | Ast::SequenceWithFlags(sequence, _) => {
                self.sequence(&sequence.elements)?
            }
            Ast::Capture(inner, index) => {
                self.ops.push(Op::Open(*index));
                self.ast(inner)?;
                self.ops.push(Op::Close(*index));
            }
            Ast::PatternWithCaptures { elements, .. } => {
                for element in elements {
                    match element {
                        CaptureElement::Capture(inner, index) => {
                            self.ops.push(Op::Open(*index));
                            self.ast(inner)?;
                            self.ops.push(Op::Close(*index));
                        }
                        CaptureElement::NonCapture(inner) => self.ast(inner)?,
                    }
                }
            }
            Ast::Boundary(boundary) => self.ops.push(Op::Boundary(*boundary)),
            Ast::Group(group) => self.group(group)?,
            _ => return None,
        }
        Some(())
    }

    #[inline]
    pub(crate) fn match_at(
        &self,
        text: &str,
        start: usize,
        slots: &mut [Option<(usize, usize)>],
    ) -> Option<usize> {
        let ops = if slots.is_empty() {
            self.span_ops.as_deref().unwrap_or(&self.ops)
        } else {
            &self.ops
        };
        Self::match_tail(ops, text, 0, start, slots)
    }

    /// An index already checked the complete literal at this candidate.
    /// Capture-free queries can skip the corresponding leading instructions.
    #[inline]
    pub(crate) fn verify_candidate(
        &self,
        text: &str,
        start: usize,
        prefix_len: usize,
        anchor: Option<usize>,
    ) -> bool {
        if let Some(anchor) = anchor {
            return self.candidate_start(text, anchor, start).is_some()
                && Self::match_tail(&self.ops, text, anchor + 1, start + prefix_len, &mut [])
                    .is_some();
        }
        let ops = self.span_ops.as_deref().unwrap_or(&self.ops);
        let mut consumed = 0;
        let mut first = 0;
        for op in ops {
            match op {
                Op::Open(_) | Op::Close(_) => {}
                Op::Literal(value) if consumed + value.len() <= prefix_len => {
                    consumed += value.len()
                }
                _ => break,
            }
            first += 1;
        }
        Self::match_tail(ops, text, first, start + consumed, &mut []).is_some()
    }

    #[inline]
    fn match_tail(
        ops: &[Op],
        text: &str,
        first: usize,
        start: usize,
        slots: &mut [Option<(usize, usize)>],
    ) -> Option<usize> {
        let bytes = text.as_bytes();
        let mut pos = start;
        for op in ops.iter().skip(first) {
            match op {
                Op::Literal(value) => {
                    if !bytes.get(pos..)?.starts_with(value.as_bytes()) {
                        return None;
                    }
                    pos += value.len();
                }
                Op::Run {
                    mask,
                    range,
                    min,
                    max,
                    lazy,
                } => {
                    let limit = if *lazy { *min } else { *max };
                    let from = pos;
                    let end = pos.saturating_add(limit).min(bytes.len());
                    if let Some((low, high)) = *range {
                        let width = high - low;
                        while pos < end && bytes[pos].wrapping_sub(low) <= width {
                            pos += 1;
                        }
                    } else {
                        while pos < end && contains(mask, bytes[pos]) {
                            pos += 1;
                        }
                    }
                    if pos - from < *min {
                        return None;
                    }
                }
                Op::Open(index) => {
                    if !slots.is_empty() {
                        *slots.get_mut(*index)? = Some((pos, pos));
                    }
                }
                Op::Close(index) => {
                    if !slots.is_empty() {
                        slots.get_mut(*index)?.as_mut()?.1 = pos;
                    }
                }
                Op::Boundary(boundary) => {
                    if !boundary.matches_at(text, pos) {
                        return None;
                    }
                }
                Op::Anchor(Anchor::Start { multiline }) => {
                    if pos != 0 && !(*multiline && bytes.get(pos - 1) == Some(&b'\n')) {
                        return None;
                    }
                }
                Op::Anchor(Anchor::End { multiline }) => {
                    if pos != bytes.len() && !(*multiline && bytes.get(pos) == Some(&b'\n')) {
                        return None;
                    }
                }
            }
        }
        Some(pos)
    }
}
