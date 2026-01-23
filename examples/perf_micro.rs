use std::time::Instant;

fn main() {
    let pat = "^rule\\s+\\d+$";
    let text = "rule 12345";

    // Warm-up
    for _ in 0..10 {
        let _ = rexile::is_match(pat, text);
    }

    let iters = 100_000;

    // 1) rexile cached is_match
    let start = Instant::now();
    for _ in 0..iters {
        let _ = rexile::is_match(pat, text).unwrap();
    }
    let dur = start.elapsed();
    println!(
        "rexile::is_match (cached) {} iters: {:?} (avg {:?})",
        iters,
        dur,
        dur / iters
    );

    // 2) regex compile + match (worst-case: compile every iteration)
    let start = Instant::now();
    for _ in 0..iters {
        let r = regex::Regex::new(pat).unwrap();
        let _ = r.is_match(text);
    }
    let dur2 = start.elapsed();
    println!(
        "regex compile+match {} iters: {:?} (avg {:?})",
        iters,
        dur2,
        dur2 / iters
    );

    // 3) regex compile once + repeated match
    let r_once = regex::Regex::new(pat).unwrap();
    let start = Instant::now();
    for _ in 0..iters {
        let _ = r_once.is_match(text);
    }
    let dur3 = start.elapsed();
    println!(
        "regex compile_once+match {} iters: {:?} (avg {:?})",
        iters,
        dur3,
        dur3 / iters
    );

    println!("-- Summary note: rexile caches compilation and avoids repeated compile cost.");
}
