//! Isolated allocator measurements; never use these timings as latency results.
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

use regex::{Regex, RegexSet};

use rexile::PatternSet;

#[path = "../benches/support/pattern_set_workloads.rs"]
mod workloads;

struct Tracking;
static LIVE: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);
static CALLS: AtomicUsize = AtomicUsize::new(0);

fn allocated(bytes: usize) {
    let live = LIVE.fetch_add(bytes, Ordering::Relaxed) + bytes;
    PEAK.fetch_max(live, Ordering::Relaxed);
    CALLS.fetch_add(1, Ordering::Relaxed);
}

// SAFETY: Allocation operations and their layouts are forwarded unchanged to
// System. Accounting uses independent atomics and never dereferences pointers.
unsafe impl GlobalAlloc for Tracking {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        // SAFETY: The caller provides a valid layout.
        let pointer = unsafe { System.alloc(layout) };
        if !pointer.is_null() {
            allocated(layout.size());
        }
        pointer
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
        // SAFETY: The allocation and matching layout come from the caller.
        unsafe { System.dealloc(pointer, layout) }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        // SAFETY: The original allocation and requested size come from the caller.
        let result = unsafe { System.realloc(pointer, layout, size) };
        if !result.is_null() {
            LIVE.fetch_sub(layout.size(), Ordering::Relaxed);
            allocated(size);
        }
        result
    }
}
#[global_allocator]
static ALLOCATOR: Tracking = Tracking;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args().collect();
    if args.len() != 7 {
        return Err(
            "usage: pattern_set_memory ENGINE FAMILY COUNT LENGTH DENSITY ids|captures".into(),
        );
    }
    let engine = args[1].as_str();
    let family = args[2].as_str();
    let count: usize = args[3].parse()?;
    let length: usize = args[4].parse()?;
    let density = args[5].as_str();
    let captures = match args[6].as_str() {
        "ids" => false,
        "captures" => true,
        _ => return Err("unknown result mode".into()),
    };
    if !workloads::FAMILIES.contains(&family)
        || !workloads::COUNTS.contains(&count)
        || !workloads::LENGTHS.contains(&length)
        || !workloads::DENSITIES.contains(&density)
    {
        return Err("case is outside the frozen acceptance corpus".into());
    }
    let patterns = workloads::patterns(family, count);
    let text = workloads::haystack(family, count, length, density);
    let baseline = LIVE.load(Ordering::Relaxed);
    PEAK.store(baseline, Ordering::Relaxed);
    let calls = CALLS.load(Ordering::Relaxed);
    let (compile_peak, retained, checksum, total_peak) = match engine {
        "rexile" => {
            let set = PatternSet::new(&patterns)?;
            let compile_peak = PEAK.load(Ordering::Relaxed) - baseline;
            let retained = LIVE.load(Ordering::Relaxed) - baseline;
            let checksum = if captures {
                let hits = set.captures_each(&text);
                std::hint::black_box(&hits);
                hits.iter()
                    .map(|hit| hit.pattern_id() + hit.pos(0).unwrap().1)
                    .sum::<usize>()
            } else {
                let ids = set.matches(&text);
                std::hint::black_box(&ids);
                ids.iter().sum::<usize>()
            };
            (
                compile_peak,
                retained,
                checksum,
                PEAK.load(Ordering::Relaxed) - baseline,
            )
        }
        "regex_set" | "regex_vec" | "regex_set_vec" => {
            let set = if engine != "regex_vec" {
                Some(RegexSet::new(&patterns)?)
            } else {
                None
            };
            let regexes = if engine != "regex_set" {
                patterns
                    .iter()
                    .map(|pattern| Regex::new(pattern))
                    .collect::<Result<Vec<_>, _>>()?
            } else {
                Vec::new()
            };
            if captures && regexes.is_empty() {
                return Err("capture mode requires regex_vec or regex_set_vec".into());
            }
            let compile_peak = PEAK.load(Ordering::Relaxed) - baseline;
            let retained = LIVE.load(Ordering::Relaxed) - baseline;
            let matched_ids = set.as_ref().map(|set| set.matches(&text));
            let checksum = if captures {
                let hits: Vec<_> = regexes
                    .iter()
                    .enumerate()
                    .filter(|(id, _)| matched_ids.as_ref().map_or(true, |ids| ids.matched(*id)))
                    .filter_map(|(id, re)| re.captures(&text).map(|caps| (id, caps)))
                    .collect();
                std::hint::black_box(&hits);
                hits.iter()
                    .map(|(id, hit)| id + hit.get(0).unwrap().end())
                    .sum::<usize>()
            } else if let Some(ids) = matched_ids {
                ids.iter().sum()
            } else {
                regexes
                    .iter()
                    .enumerate()
                    .filter_map(|(id, re)| re.is_match(&text).then_some(id))
                    .sum()
            };
            (
                compile_peak,
                retained,
                checksum,
                PEAK.load(Ordering::Relaxed) - baseline,
            )
        }
        _ => return Err("unknown engine".into()),
    };
    let allocations = CALLS.load(Ordering::Relaxed) - calls;
    println!("{{\"engine\":\"{engine}\",\"family\":\"{family}\",\"count\":{count},\"length\":{length},\"density\":\"{density}\",\"mode\":\"{}\",\"compile_peak\":{compile_peak},\"retained\":{retained},\"total_peak\":{total_peak},\"allocations\":{allocations},\"checksum\":{checksum}}}", args[6]);
    Ok(())
}
