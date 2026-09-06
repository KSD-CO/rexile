use std::cmp::Reverse;
use std::collections::BinaryHeap;

use regex::{CaptureLocations, Regex, RegexSet};

/// A fair reusable-buffer baseline with the same ordering as PatternSet.
pub struct Visitor {
    slots: Vec<CaptureLocations>,
    progress: Vec<(usize, Option<usize>, bool)>,
    heap: BinaryHeap<Reverse<(usize, usize, usize)>>,
}

impl Visitor {
    pub fn new(regexes: &[Regex]) -> Self {
        Self {
            slots: regexes.iter().map(Regex::capture_locations).collect(),
            progress: vec![(0, None, false); regexes.len()],
            heap: BinaryHeap::new(),
        }
    }

    fn next(&mut self, regexes: &[Regex], text: &str, id: usize) -> Option<(usize, usize)> {
        let progress = &mut self.progress[id];
        while !progress.2 {
            let hit = regexes[id].captures_read_at(&mut self.slots[id], text, progress.0)?;
            let (start, end) = (hit.start(), hit.end());
            let duplicate_empty = start == end && progress.1 == Some(end);
            progress.0 = end;
            if start == end {
                match text.get(end..).and_then(|tail| tail.chars().next()) {
                    Some(ch) => progress.0 += ch.len_utf8(),
                    None => progress.2 = true,
                }
            }
            if !duplicate_empty {
                progress.1 = Some(end);
                return Some((start, end));
            }
        }
        None
    }

    pub fn visit(&mut self, regexes: &[Regex], set: Option<&RegexSet>, text: &str) -> usize {
        self.progress.fill((0, None, false));
        self.heap.clear();
        let ids = set.map(|set| set.matches(text));
        for id in 0..regexes.len() {
            if ids.as_ref().is_some_and(|ids| !ids.matched(id)) {
                continue;
            }
            if let Some((start, end)) = self.next(regexes, text, id) {
                self.heap.push(Reverse((start, id, end)));
            }
        }
        let mut checksum = 0;
        while let Some(Reverse((_, id, _))) = self.heap.pop() {
            checksum += id;
            for group in 0..self.slots[id].len() {
                if let Some((start, end)) = self.slots[id].get(group) {
                    checksum += start + end;
                }
            }
            if let Some((start, end)) = self.next(regexes, text, id) {
                self.heap.push(Reverse((start, id, end)));
            }
        }
        checksum
    }
}
