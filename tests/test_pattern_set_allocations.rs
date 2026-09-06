use std::alloc::{GlobalAlloc, Layout, System};
use std::ops::ControlFlow;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use rexile::{PatternSet, SetSearchMode};

#[path = "../benches/support/pattern_set_workloads.rs"]
mod workloads;

struct Counter;
static COUNTING: AtomicBool = AtomicBool::new(false);
static ALLOCATIONS: AtomicUsize = AtomicUsize::new(0);

// SAFETY: Every operation delegates to System with the unchanged layout/pointer.
// Atomic counters only observe allocations and do not access allocated memory.
unsafe impl GlobalAlloc for Counter {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        // SAFETY: Forwarding the caller's valid allocation layout.
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        // SAFETY: Forwarding the caller's allocation and matching layout.
        unsafe { System.dealloc(pointer, layout) }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        if COUNTING.load(Ordering::Relaxed) {
            ALLOCATIONS.fetch_add(1, Ordering::Relaxed);
        }
        // SAFETY: Forwarding the original allocation and requested new size.
        unsafe { System.realloc(pointer, layout, size) }
    }
}

#[global_allocator]
static ALLOCATOR: Counter = Counter;

// This separate test binary has one test, so harness allocations cannot come
// from another test while counting. Result strings borrow the input.
#[test]
fn warmed_visitors_and_id_queries_do_not_allocate() {
    let set = PatternSet::new([
        r"key=([a-z]+):([0-9]+);",
        r"([0-9]+)",
        r"(?i)code=([a-z]+)",
        "key",
    ])
    .unwrap();
    let text = "key=alice:123; CODE=OK key=bob:42;";
    let mut cache = set.create_cache();
    let mut scan = || {
        let mut checksum = 0usize;
        let _ = set.visit_captures(text, &mut cache, SetSearchMode::All, |hit| {
            checksum += hit.pattern_id() + hit.get(0).unwrap().len();
            ControlFlow::<()>::Continue(())
        });
        let _ = set.visit_matches(text, &mut cache, SetSearchMode::All, |hit| {
            checksum += hit.end();
            ControlFlow::<()>::Continue(())
        });
        checksum += set.matches_with_cache(text, &mut cache).len();
        checksum += set.is_match_with_cache(text, &mut cache) as usize;
        checksum
    };
    let expected = scan();
    ALLOCATIONS.store(0, Ordering::SeqCst);
    COUNTING.store(true, Ordering::SeqCst);
    let actual = scan();
    COUNTING.store(false, Ordering::SeqCst);
    assert_eq!(actual, expected);
    assert_eq!(ALLOCATIONS.load(Ordering::SeqCst), 0);

    for &family in workloads::FAMILIES {
        for &count in workloads::COUNTS {
            let set = PatternSet::new(workloads::patterns(family, count)).unwrap();
            let mut cache = set.create_cache();
            for &length in workloads::LENGTHS {
                for &density in workloads::DENSITIES {
                    let text = workloads::haystack(family, count, length, density);
                    let mut visit = || {
                        let mut checksum = 0;
                        for mode in [SetSearchMode::All, SetSearchMode::FirstPerPattern] {
                            let _ = set.visit_captures(&text, &mut cache, mode, |hit| {
                                checksum += hit.pattern_id() + hit.pos(0).unwrap().1;
                                ControlFlow::<()>::Continue(())
                            });
                            let _ = set.visit_matches(&text, &mut cache, mode, |hit| {
                                checksum += hit.pattern_id() + hit.end();
                                ControlFlow::<()>::Continue(())
                            });
                        }
                        checksum += set.matches_with_cache(&text, &mut cache).len();
                        checksum += set.is_match_with_cache(&text, &mut cache) as usize;
                        checksum
                    };
                    let expected = visit();
                    ALLOCATIONS.store(0, Ordering::SeqCst);
                    COUNTING.store(true, Ordering::SeqCst);
                    let actual = visit();
                    COUNTING.store(false, Ordering::SeqCst);
                    assert_eq!(actual, expected);
                    assert_eq!(
                        ALLOCATIONS.load(Ordering::SeqCst),
                        0,
                        "{family}/{count}/{length}/{density}"
                    );
                }
            }
        }
    }
}
