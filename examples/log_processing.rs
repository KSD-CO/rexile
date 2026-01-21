//! Log processing example - finding patterns in log files

use rexile::Pattern;

fn main() {
    println!("=== ReXile Log Processing Example ===\n");

    // Simulated log entries
    let logs = vec![
        "[ERROR] Failed to connect to database",
        "[INFO] Server started on port 8080",
        "[WARN] High memory usage detected",
        "[ERROR] Null pointer exception in handler",
        "[DEBUG] Request processed in 42ms",
        "[ERROR] Connection timeout",
    ];

    // Pattern to find different log levels
    let error_pattern = Pattern::new("ERROR").unwrap();
    let warn_pattern = Pattern::new("WARN").unwrap();
    let _level_pattern = Pattern::new("ERROR|WARN|INFO|DEBUG").unwrap();

    println!("Processing {} log entries...\n", logs.len());

    let mut error_count = 0;
    let mut warn_count = 0;

    for log in &logs {
        if error_pattern.is_match(log) {
            error_count += 1;
            println!("🔴 {}", log);
        } else if warn_pattern.is_match(log) {
            warn_count += 1;
            println!("🟡 {}", log);
        } else {
            println!("⚪ {}", log);
        }
    }

    println!("\n=== Summary ===");
    println!("Total logs: {}", logs.len());
    println!("Errors: {}", error_count);
    println!("Warnings: {}", warn_count);

    // Find specific keywords across all logs
    println!("\n=== Keyword Search ===");
    let keywords = Pattern::new("database|memory|timeout").unwrap();
    
    for log in &logs {
        if keywords.is_match(log) {
            if let Some((start, end)) = keywords.find(log) {
                println!("Found '{}' in: {}", &log[start..end], log);
            }
        }
    }
}
