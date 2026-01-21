//! Performance comparison: cached vs uncached pattern compilation

use rexile::Pattern;
use std::time::Instant;

fn main() {
    println!("=== ReXile Performance Example ===\n");

    let text = "The quick brown fox jumps over the lazy dog. The fox is quick and clever.";
    let pattern_str = "fox";
    let iterations = 100_000;

    // Test 1: Uncached (compile every time)
    println!("Test 1: Uncached compilation (compile {} times)", iterations);
    let start = Instant::now();
    let mut match_count = 0;
    for _ in 0..iterations {
        let pattern = Pattern::new(pattern_str).unwrap();
        if pattern.is_match(text) {
            match_count += 1;
        }
    }
    let uncached_time = start.elapsed();
    println!("  Time: {:?}", uncached_time);
    println!("  Matches: {}\n", match_count);

    // Test 2: Cached (compile once, reuse)
    println!("Test 2: Pre-compiled pattern (compile once, use {} times)", iterations);
    let pattern = Pattern::new(pattern_str).unwrap();
    let start = Instant::now();
    let mut match_count = 0;
    for _ in 0..iterations {
        if pattern.is_match(text) {
            match_count += 1;
        }
    }
    let cached_time = start.elapsed();
    println!("  Time: {:?}", cached_time);
    println!("  Matches: {}\n", match_count);

    // Test 3: Global cache API
    println!("Test 3: Global cache API (auto-caching, use {} times)", iterations);
    let start = Instant::now();
    let mut match_count = 0;
    for _ in 0..iterations {
        if rexile::is_match(pattern_str, text).unwrap() {
            match_count += 1;
        }
    }
    let global_cache_time = start.elapsed();
    println!("  Time: {:?}", cached_time);
    println!("  Matches: {}\n", match_count);

    // Summary
    println!("=== Performance Summary ===");
    println!("Uncached:     {:?}", uncached_time);
    println!("Pre-compiled: {:?} ({:.2}x faster)", cached_time, uncached_time.as_secs_f64() / cached_time.as_secs_f64());
    println!("Global cache: {:?} ({:.2}x faster)", global_cache_time, uncached_time.as_secs_f64() / global_cache_time.as_secs_f64());
    
    println!("\n💡 Tip: Use cached API (rexile::is_match) or pre-compile patterns for best performance!");
}
