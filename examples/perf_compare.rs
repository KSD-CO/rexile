use std::time::{Duration, Instant};

fn measure_time<F: Fn() -> bool>(f: F, iterations: u32) -> (Duration, bool) {
    let result = f();
    for _ in 0..100 {
        std::hint::black_box(f());
    }
    let start = Instant::now();
    for _ in 0..iterations {
        std::hint::black_box(f());
    }
    (start.elapsed(), result)
}

fn get_rss_kb() -> usize {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|s| {
            s.lines()
                .find(|l| l.starts_with("VmRSS:"))
                .and_then(|l| l.split_whitespace().nth(1)?.parse().ok())
        })
        .unwrap_or(0)
}

fn main() {
    let iterations = 200_000u32;
    let text_short = "hello world 42";
    let text_version = "Server version 10.2.34-beta running on port 8080";
    let text_log =
        "2024-01-15 ERROR [main] Connection timeout after 30s retry=3 user=admin@example.com";
    let text_code = "fn calculate_total(items: Vec<Item>) -> Result<f64, Error> { Ok(0.0) }";
    let text_long = &("ab12 ".repeat(200) + "target xyz999 end");

    let test_cases: Vec<(&str, &str, &str)> = vec![
        ("[a-z]+.+[0-9]+", text_short, "overlap: [a-z]+.+[0-9]+"),
        ("[a-z]+.+[0-9]+", text_long, "overlap: long text"),
        ("[A-Z]+.+[a-z]+", text_log, "overlap: [A-Z]+.+[a-z]+"),
        ("[a-z]+[0-9]+", text_short, "adjacent: [a-z]+[0-9]+"),
        ("\\w+\\s+\\d+", text_log, "adjacent: \\w+\\s+\\d+"),
        ("\\d+\\.\\d+", text_version, "dfa: \\d+.\\d+"),
        ("\\d+\\.\\d+\\.\\d+", text_version, "dfa: \\d+.\\d+.\\d+"),
        ("ERROR", text_log, "literal: ERROR"),
        ("calculate_total", text_code, "literal: calculate_total"),
        ("ERROR|WARN|INFO", text_log, "alt: ERROR|WARN|INFO"),
        (
            "import|export|function|return",
            text_code,
            "alt: 4 keywords",
        ),
        ("\\d+", text_log, "escape: \\d+"),
        ("\\w+@\\w+", text_log, "escape: \\w+@\\w+"),
        ("[0-9]+", text_version, "charclass: [0-9]+"),
        ("[a-zA-Z_]+", text_code, "charclass: [a-zA-Z_]+"),
    ];

    let mem_before = get_rss_kb();
    let rexile_patterns: Vec<_> = test_cases
        .iter()
        .map(|(pat, _, _)| rexile::Pattern::new(pat).unwrap())
        .collect();
    let mem_rexile = get_rss_kb();
    let regex_patterns: Vec<_> = test_cases
        .iter()
        .map(|(pat, _, _)| regex::Regex::new(pat).unwrap())
        .collect();
    let mem_regex = get_rss_kb();

    println!(
        "{:<35} {:>10} {:>10} {:>8} {:>4}",
        "Test", "rexile", "regex", "ratio", "ok"
    );
    println!("{}", "-".repeat(72));

    let mut faster = 0;
    let mut total_r = 0.0f64;
    let mut total_x = 0.0f64;

    for (i, (_, text, name)) in test_cases.iter().enumerate() {
        let rp = &rexile_patterns[i];
        let xp = &regex_patterns[i];
        let (rd, rr) = measure_time(|| rp.is_match(text), iterations);
        let (xd, xr) = measure_time(|| xp.is_match(text), iterations);
        let rns = rd.as_nanos() as f64 / iterations as f64;
        let xns = xd.as_nanos() as f64 / iterations as f64;
        let ratio = rns / xns;
        let ok = rr == xr;
        total_r += rns;
        total_x += xns;
        if rns < xns {
            faster += 1;
        }
        let arrow = if ratio < 1.0 {
            "◀"
        } else if ratio > 2.0 {
            "▶▶"
        } else {
            ""
        };
        println!(
            "{:<35} {:>7.1}ns {:>7.1}ns {:>7.2}x {:>3} {}",
            name,
            rns,
            xns,
            ratio,
            if ok { "✓" } else { "✗" },
            arrow
        );
    }

    println!("{}", "-".repeat(72));
    println!(
        "{:<35} {:>7.0}ns {:>7.0}ns {:>7.2}x",
        "TOTAL",
        total_r,
        total_x,
        total_r / total_x
    );
    println!("\nrexile faster: {}/{}", faster, test_cases.len());

    println!("\n--- Memory (RSS) ---");
    println!(
        "  rexile: +{} KB | regex: +{} KB | ratio: {:.1}x less",
        mem_rexile.saturating_sub(mem_before),
        mem_regex.saturating_sub(mem_rexile),
        (mem_regex.saturating_sub(mem_rexile)) as f64
            / (mem_rexile.saturating_sub(mem_before)).max(1) as f64
    );

    println!("\n--- Compile Time ---");
    let ci = 10_000u32;
    let start = Instant::now();
    for _ in 0..ci {
        for (p, ..) in &test_cases {
            std::hint::black_box(rexile::Pattern::new(p).unwrap());
        }
    }
    let rc = start.elapsed();
    let start = Instant::now();
    for _ in 0..ci {
        for (p, ..) in &test_cases {
            std::hint::black_box(regex::Regex::new(p).unwrap());
        }
    }
    let xc = start.elapsed();
    let rcp = rc.as_nanos() as f64 / (ci as f64 * test_cases.len() as f64);
    let xcp = xc.as_nanos() as f64 / (ci as f64 * test_cases.len() as f64);
    println!(
        "  rexile: {:.0}ns/pat | regex: {:.0}ns/pat | {:.1}x faster",
        rcp,
        xcp,
        xcp / rcp
    );
}
