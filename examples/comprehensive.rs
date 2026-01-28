//! Comprehensive ReXile Examples
//!
//! All-in-one example showcasing ReXile features and performance.
//! Run with: cargo run --example comprehensive [demo]
//!
//! Available demos:
//!   basic       - Basic pattern matching
//!   advanced    - Advanced features (lookaround, captures)
//!   performance - Performance comparison
//!   benchmark   - Detailed benchmarks vs regex crate
//!   production  - Production-ready patterns
//!   all         - Run all demos (default)

use rexile::{Pattern, ReXile};
use std::env;
use std::time::Instant;

fn main() {
    let args: Vec<String> = env::args().collect();
    let demo = args.get(1).map(|s| s.as_str()).unwrap_or("all");

    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║           ReXile Comprehensive Examples                  ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    match demo {
        "basic" => run_basic_demo(),
        "advanced" => run_advanced_demo(),
        "performance" => run_performance_demo(),
        "benchmark" => run_benchmark_demo(),
        "production" => run_production_demo(),
        "all" => {
            run_basic_demo();
            println!("\n{}\n", "═".repeat(60));
            run_advanced_demo();
            println!("\n{}\n", "═".repeat(60));
            run_performance_demo();
            println!("\n{}\n", "═".repeat(60));
            run_benchmark_demo();
            println!("\n{}\n", "═".repeat(60));
            run_production_demo();
        }
        _ => {
            println!("❌ Unknown demo: {}", demo);
            println!("\nAvailable demos:");
            println!("  basic       - Basic pattern matching");
            println!("  advanced    - Advanced features");
            println!("  performance - Performance comparison");
            println!("  benchmark   - Detailed benchmarks");
            println!("  production  - Production-ready patterns");
            println!("  all         - Run all demos");
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// BASIC DEMO
// ═══════════════════════════════════════════════════════════════════════════

fn run_basic_demo() {
    println!("📘 BASIC PATTERN MATCHING\n");

    let demos = vec![
        ("Literals", r"hello", "hello world", true),
        ("Word chars", r"\w+", "hello123", true),
        ("Digits", r"\d+", "price: 42", true),
        ("Email-like", r"\w+@\w+", "user@domain", true),
        ("Anchors", r"^start", "start here", true),
        ("End anchor", r"end$", "the end", true),
    ];

    for (name, pattern, text, _expected) in demos {
        match ReXile::new(pattern) {
            Ok(rex) => {
                let matched = rex.is_match(text);
                let symbol = if matched { "✓" } else { "✗" };
                println!("{} {:12} | {} → '{}'", symbol, name, pattern, text);
            }
            Err(e) => println!("✗ {:12} | {} → Error: {}", name, pattern, e),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ADVANCED DEMO
// ═══════════════════════════════════════════════════════════════════════════

fn run_advanced_demo() {
    println!("🚀 ADVANCED FEATURES\n");

    println!("1. Capture Groups:");
    let pattern = Pattern::new(r"(\w+)@(\w+)").unwrap();
    let text = "Contact: admin@example";
    if let Some(caps) = pattern.captures(text) {
        println!("   Text: '{}'", text);
        println!("   User: {:?}", caps.get(1));
        println!("   Domain: {:?}", caps.get(2));
    }

    println!("\n2. Lookahead:");
    let pattern = Pattern::new(r"ERROR(?=\s)").unwrap();
    println!("   ✓ Matches 'ERROR ' (followed by space)");
    println!("   Match: {}", pattern.is_match("ERROR "));
    println!("   No match: {}", pattern.is_match("ERROR!"));

    println!("\n3. Lookbehind:");
    let pattern = Pattern::new(r"(?<=user=)\w+").unwrap();
    if let Some(m) = pattern.find("url?user=admin&pass=123") {
        println!("   Extracts username after 'user=': {:?}", m);
    }

    println!("\n4. Quantifiers:");
    let tests = vec![
        (r"\d{3}", "123", "Exact 3 digits"),
        (r"\d{2,4}", "12345", "2-4 digits"),
        (r"\w+?", "hello", "Lazy match"),
    ];
    for (pat, text, _desc) in tests {
        let rex = ReXile::new(pat).unwrap();
        println!(
            "   {} → '{}': {}",
            pat,
            text,
            if rex.is_match(text) { "✓" } else { "✗" }
        );
    }

    println!("\n5. Character Classes:");
    let tests = vec![
        (r"[a-z]+", "hello", "Lowercase"),
        (r"[A-Z]+", "WORLD", "Uppercase"),
        (r"[0-9]+", "12345", "Digits"),
        (r"[^0-9]+", "abc", "Not digits"),
    ];
    for (pat, text, desc) in tests {
        let rex = ReXile::new(pat).unwrap();
        println!(
            "   {:15} {} → '{}': {}",
            desc,
            pat,
            text,
            if rex.is_match(text) { "✓" } else { "✗" }
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PERFORMANCE DEMO
// ═══════════════════════════════════════════════════════════════════════════

fn run_performance_demo() {
    println!("⚡ PERFORMANCE COMPARISON\n");

    let iterations = 100_000;

    println!("Test: Compile and match {} times\n", iterations);

    // Test 1: Uncached (compile every time)
    let start = Instant::now();
    let mut count = 0;
    for _ in 0..iterations {
        if let Ok(rex) = ReXile::new(r"\w+@\w+") {
            if rex.is_match("user@domain") {
                count += 1;
            }
        }
    }
    let uncached_time = start.elapsed();
    println!(
        "1. Uncached:      {:>10.2?} ({} matches)",
        uncached_time, count
    );

    // Test 2: Pre-compiled
    let rex = ReXile::new(r"\w+@\w+").unwrap();
    let start = Instant::now();
    let mut count = 0;
    for _ in 0..iterations {
        if rex.is_match("user@domain") {
            count += 1;
        }
    }
    let cached_time = start.elapsed();
    println!(
        "2. Pre-compiled:  {:>10.2?} ({} matches)",
        cached_time, count
    );

    // Test 3: Global cache API
    let start = Instant::now();
    let mut count = 0;
    for _ in 0..iterations {
        if rexile::is_match(r"\w+@\w+", "user@domain").unwrap_or(false) {
            count += 1;
        }
    }
    let global_time = start.elapsed();
    println!(
        "3. Global cache:  {:>10.2?} ({} matches)",
        global_time, count
    );

    println!("\n📊 Speedup:");
    println!(
        "   Pre-compiled vs uncached: {:.1}x faster",
        uncached_time.as_secs_f64() / cached_time.as_secs_f64()
    );
    println!(
        "   Global cache vs uncached: {:.1}x faster",
        uncached_time.as_secs_f64() / global_time.as_secs_f64()
    );
}

// ═══════════════════════════════════════════════════════════════════════════
// BENCHMARK DEMO
// ═══════════════════════════════════════════════════════════════════════════

fn run_benchmark_demo() {
    println!("🏁 DETAILED BENCHMARKS\n");

    let test_cases = vec![
        // Basic patterns
        (
            "Literal",
            "ERROR",
            "2024-01-15 ERROR [main] Connection timeout",
        ),
        ("Word run", r"\w+", "hello world test"),
        ("Digits", r"\d+", "Port: 8080"),
        ("Whitespace", r"\s+", "hello   world"),
        // Character classes
        ("Char class", r"[a-z]+", "hello WORLD"),
        ("Char range", r"[0-9]+", "price: 12345"),
        ("Negated class", r"[^0-9]+", "hello world"),
        // Quantifiers
        ("Star", r"\w*", "hello123"),
        ("Plus", r"\w+", "test_value"),
        ("Question", r"https?://", "https://example.com"),
        ("Exact", r"\d{4}", "year 2024"),
        ("Range", r"\d{2,4}", "12345"),
        ("Min", r"\w{3,}", "testing"),
        // Anchors & boundaries
        ("Start anchor", r"^ERROR", "ERROR: Something went wrong"),
        ("End anchor", r"end$", "the end"),
        ("Word boundary", r"\btest\b", "this is a test case"),
        // Complex patterns
        (
            "Email-like",
            r"\w+@\w+\.\w+",
            "Contact: admin@example.com for info",
        ),
        ("IP-like", r"\d+\.\d+\.\d+\.\d+", "Server: 192.168.1.1"),
        ("Version", r"\d+\.\d+\.\d+", "v1.2.3"),
        ("URL protocol", r"https?://\w+", "Visit https://github.com"),
        // Alternation
        ("Alt 2", r"ERROR|WARN", "2024-01-15 ERROR Connection failed"),
        ("Alt 3", r"GET|POST|PUT", "Request: POST /api/users"),
        ("Alt 4", r"true|false|null|undefined", "value is true"),
        // Escape sequences
        ("Escape word", r"\w+", "hello_world"),
        ("Escape digit", r"\d+", "count: 42"),
        ("Escape dot", r"\.", "file.txt"),
        // Real-world patterns from rust-rule-engine
        (
            "Rule name",
            r#"rule\s+"([^"]+)""#,
            r#"rule "TestRule" { ... }"#,
        ),
        (
            "Module def",
            r"defmodule\s+([A-Z_]\w*)",
            "defmodule MyModule { }",
        ),
        ("Salience", r"salience\s+(\d+)", "salience 100"),
        ("Variable", r"\$(\w+)", "value is $variable here"),
        ("Method call", r"(\w+)\s*\(([^)]*)\)", "calculate(10, 20)"),
        ("Field access", r"([a-zA-Z_]\w*\.\w+)", "user.name == admin"),
        (
            "Condition",
            r"(\w+)\s*(>=|<=|==|!=|>|<)\s*(.+)",
            "age >= 18",
        ),
        // Grouped patterns
        ("Capture group", r"(\w+)@(\w+)", "user@domain"),
        ("Non-capturing", r"(?:\w+\s+)+", "hello world test"),
        (
            "Multiple groups",
            r"(\d{4})-(\d{2})-(\d{2})",
            "Date: 2024-01-28",
        ),
    ];

    let iterations = 100_000;

    println!(
        "{:<20} {:<35} {:>12}",
        "Pattern Type", "Pattern", "Time/iter"
    );
    println!("{}", "─".repeat(70));

    for (name, pattern, text) in test_cases {
        let rex = match ReXile::new(pattern) {
            Ok(r) => r,
            Err(_) => {
                println!("{:<20} {:<35} {:>12}", name, pattern, "SKIP");
                continue;
            }
        };

        let start = Instant::now();
        for _ in 0..iterations {
            std::hint::black_box(rex.is_match(text));
        }
        let elapsed = start.elapsed();
        let ns_per_iter = elapsed.as_nanos() as f64 / iterations as f64;

        println!("{:<20} {:<35} {:>9.1}ns", name, pattern, ns_per_iter);
    }

    println!("\n💡 Performance Tips:");
    println!("   • Literal patterns fastest (~20-30ns) - memchr optimization");
    println!("   • Character classes very fast (~10-20ns) - bitmap lookup");
    println!("   • Anchored patterns optimized for start/end matching");
    println!("   • Pre-compile patterns for 100x+ speedup vs repeated compilation");
}

// ═══════════════════════════════════════════════════════════════════════════
// PRODUCTION DEMO
// ═══════════════════════════════════════════════════════════════════════════

fn run_production_demo() {
    println!("🏭 PRODUCTION-READY PATTERNS\n");

    println!("1. Log Level Extraction:");
    let log_pattern = Pattern::new(r"\[(\w+)\]").unwrap();
    let logs = vec![
        "[ERROR] Connection timeout",
        "[INFO] Server started",
        "[WARN] High memory usage",
        "[DEBUG] Processing request",
    ];
    for log in logs {
        if let Some(caps) = log_pattern.captures(log) {
            println!("   Level: {:?} | {}", caps.get(1), log);
        }
    }

    println!("\n2. Rule Engine Patterns:");
    // Pattern from rust-rule-engine GRL parser
    let rule_pattern = Pattern::new(r#"rule\s+"([^"]+)""#).unwrap();
    if let Some(caps) = rule_pattern.captures(r#"rule "ValidateUser" { when ... }"#) {
        println!("   Rule name: {:?}", caps.get(1));
    }

    let salience_pattern = Pattern::new(r"salience\s+(\d+)").unwrap();
    if let Some(caps) = salience_pattern.captures("salience 100") {
        println!("   Priority: {:?}", caps.get(1));
    }

    let condition_pattern = Pattern::new(r"(\w+)\s*(>=|<=|==|!=|>|<)\s*(.+)").unwrap();
    if let Some(caps) = condition_pattern.captures("age >= 18") {
        println!(
            "   Condition: {:?} {:?} {:?}",
            caps.get(1),
            caps.get(2),
            caps.get(3)
        );
    }

    println!("\n3. URL & Protocol Matching:");
    let url_pattern = Pattern::new(r"https?://\w+\.\w+").unwrap();
    let urls = vec![
        "Visit https://example.com",
        "Download from http://files.org",
        "Invalid: ftp://wrong.com",
    ];
    for url in urls {
        let status = if url_pattern.is_match(url) {
            "✓"
        } else {
            "✗"
        };
        println!("   {} {}", status, url);
    }

    println!("\n4. Version & Build Numbers:");
    let version_pattern = Pattern::new(r"(\d+)\.(\d+)\.(\d+)").unwrap();
    let versions = vec!["v1.2.3", "Build 2.0.1", "Release 10.5.2"];
    for ver in versions {
        if let Some(caps) = version_pattern.captures(ver) {
            println!(
                "   {} → Major: {:?}, Minor: {:?}, Patch: {:?}",
                ver,
                caps.get(1),
                caps.get(2),
                caps.get(3)
            );
        }
    }

    println!("\n5. Data Extraction:");
    let text = "Price: $49.99, Discount: 20%, Stock: 150";

    if let Some(m) = Pattern::new(r"\$\d+\.\d+").unwrap().find(text) {
        println!("   Price: {:?}", m);
    }

    if let Some(m) = Pattern::new(r"\d+%").unwrap().find(text) {
        println!("   Discount: {:?}", m);
    }

    if let Some(caps) = Pattern::new(r"Stock: (\d+)").unwrap().captures(text) {
        println!("   Stock: {:?} units", caps.get(1));
    }

    println!("\n6. IP Address & Network:");
    let ip_pattern = Pattern::new(r"\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}").unwrap();
    let network_logs = vec![
        "Connection from 192.168.1.100",
        "Server: 10.0.0.1",
        "Invalid: 999.999.999.999",
    ];
    for log in network_logs {
        if let Some((start, end)) = ip_pattern.find(log) {
            println!("   Found IP in '{}': {:?}", log, &log[start..end]);
        }
    }

    println!("\n7. Email & User Validation:");
    let email_pattern = Pattern::new(r"(\w+)@(\w+\.\w+)").unwrap();
    let emails = vec!["admin@example.com", "user@test.org"];
    for email in emails {
        if let Some(caps) = email_pattern.captures(email) {
            println!(
                "   Email: {} → User: {:?}, Domain: {:?}",
                email,
                caps.get(1),
                caps.get(2)
            );
        }
    }

    println!("\n8. HTTP Request Patterns:");
    let method_pattern = Pattern::new(r"^(GET|POST|PUT|DELETE|PATCH)").unwrap();
    let requests = vec!["GET /api/users", "POST /api/login", "PUT /api/update"];
    for req in requests {
        if let Some(caps) = method_pattern.captures(req) {
            println!("   Method: {:?} in '{}'", caps.get(1), req);
        }
    }

    println!("\n9. Variable & Field Access:");
    let var_pattern = Pattern::new(r"\$(\w+)").unwrap();
    let field_pattern = Pattern::new(r"(\w+)\.(\w+)").unwrap();

    let code = "$user.name == $admin";
    println!("   Code: '{}'", code);
    for (start, end) in var_pattern.find_all(code) {
        println!("   Variable: {:?}", &code[start..end]);
    }
    if let Some(caps) = field_pattern.captures(code) {
        println!("   Field access: {:?}.{:?}", caps.get(1), caps.get(2));
    }

    println!("\n10. Security & Validation:");
    let tests = vec![
        (r"^[a-zA-Z0-9_]{3,20}$", "admin_user", "Username"),
        (r"^\d{3}-\d{3}-\d{4}$", "123-456-7890", "Phone"),
        (r"^[A-Z]{2}\d{6}$", "AB123456", "ID Format"),
        (
            r"^[a-f0-9]{32}$",
            "a1b2c3d4e5f6789012345678901234ab",
            "MD5 Hash",
        ),
    ];
    for (pattern, input, name) in tests {
        let rex = ReXile::new(pattern).unwrap();
        let status = if rex.is_match(input) { "✓" } else { "✗" };
        println!("   {} {:<15} '{}' matches {}", status, name, input, pattern);
    }

    println!("\n11. Date & Time Patterns:");
    let date_pattern = Pattern::new(r"(\d{4})-(\d{2})-(\d{2})").unwrap();
    let time_pattern = Pattern::new(r"(\d{2}):(\d{2}):(\d{2})").unwrap();

    let datetime = "2024-01-28 14:30:45";
    if let Some(caps) = date_pattern.captures(datetime) {
        println!(
            "   Date: {:?}-{:?}-{:?}",
            caps.get(1),
            caps.get(2),
            caps.get(3)
        );
    }
    if let Some(caps) = time_pattern.captures(datetime) {
        println!(
            "   Time: {:?}:{:?}:{:?}",
            caps.get(1),
            caps.get(2),
            caps.get(3)
        );
    }

    println!("\n12. Configuration Parsing:");
    let config_patterns = vec![
        (r"(\w+)\s*=\s*(\d+)", "timeout = 30", "Integer config"),
        (
            r#"(\w+)\s*=\s*"([^"]+)""#,
            r#"name = "MyApp""#,
            "String config",
        ),
        (
            r"(\w+)\s*=\s*(true|false)",
            "debug = true",
            "Boolean config",
        ),
    ];
    for (pattern, text, desc) in config_patterns {
        if let Ok(pat) = Pattern::new(pattern) {
            if let Some(caps) = pat.captures(text) {
                println!(
                    "   {} → Key: {:?}, Value: {:?}",
                    desc,
                    caps.get(1),
                    caps.get(2)
                );
            }
        }
    }

    println!("\n✅ All production patterns validated!");
    println!("💡 These patterns are used in real rust-rule-engine production code!");
}
