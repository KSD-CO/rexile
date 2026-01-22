#!/usr/bin/env python3
"""
Memory usage comparison between ReXile and regex crate.
Requires: valgrind, massif-visualizer or ms_print
"""

import subprocess
import re
import os

def compile_rust_test(code, output_name):
    """Compile Rust code to executable."""
    with open(f'/tmp/{output_name}.rs', 'w') as f:
        f.write(code)
    
    result = subprocess.run(
        ['rustc', '-O', f'/tmp/{output_name}.rs', '-o', f'/tmp/{output_name}'],
        capture_output=True,
        text=True
    )
    
    if result.returncode != 0:
        print(f"Compilation failed for {output_name}:")
        print(result.stderr)
        return False
    return True

def measure_memory(executable):
    """Run valgrind massif to measure memory usage."""
    massif_file = f'/tmp/massif.out.{os.getpid()}'
    
    result = subprocess.run(
        ['valgrind', '--tool=massif', f'--massif-out-file={massif_file}', executable],
        capture_output=True,
        text=True
    )
    
    # Parse massif output
    result = subprocess.run(
        ['ms_print', massif_file],
        capture_output=True,
        text=True
    )
    
    output = result.stdout
    
    # Extract peak memory
    peak_match = re.search(r'peak:\s+(\d+)', output)
    peak_bytes = int(peak_match.group(1)) if peak_match else 0
    
    # Clean up
    if os.path.exists(massif_file):
        os.remove(massif_file)
    
    return peak_bytes

def format_bytes(bytes_val):
    """Format bytes to human-readable string."""
    for unit in ['B', 'KB', 'MB', 'GB']:
        if bytes_val < 1024.0:
            return f"{bytes_val:.2f} {unit}"
        bytes_val /= 1024.0
    return f"{bytes_val:.2f} TB"

# Test 1: Simple literal pattern
rexile_literal = """
extern crate rexile;
use rexile::Pattern;

fn main() {
    let patterns: Vec<&str> = vec!["hello"; 1000];
    let mut results = Vec::new();
    
    for pattern in patterns {
        let re = Pattern::new(pattern).unwrap();
        results.push(re.is_match("hello world"));
    }
    
    println!("Matched: {}", results.iter().filter(|&&x| x).count());
}
"""

regex_literal = """
extern crate regex;
use regex::Regex;

fn main() {
    let patterns: Vec<&str> = vec!["hello"; 1000];
    let mut results = Vec::new();
    
    for pattern in patterns {
        let re = Regex::new(pattern).unwrap();
        results.push(re.is_match("hello world"));
    }
    
    println!("Matched: {}", results.iter().filter(|&&x| x).count());
}
"""

# Test 2: Complex pattern
rexile_complex = """
extern crate rexile;
use rexile::Pattern;

fn main() {
    let patterns: Vec<&str> = vec![r"\\d+", r"\\w+", "[a-z]+", "^hello", "world$"];
    let text = "hello 123 world abc";
    let mut results = Vec::new();
    
    for _ in 0..200 {
        for pattern in &patterns {
            let re = Pattern::new(pattern).unwrap();
            results.push(re.is_match(text));
        }
    }
    
    println!("Matched: {}", results.iter().filter(|&&x| x).count());
}
"""

regex_complex = """
extern crate regex;
use regex::Regex;

fn main() {
    let patterns: Vec<&str> = vec![r"\\d+", r"\\w+", "[a-z]+", "^hello", "world$"];
    let text = "hello 123 world abc";
    let mut results = Vec::new();
    
    for _ in 0..200 {
        for pattern in &patterns {
            let re = Regex::new(pattern).unwrap();
            results.push(re.is_match(text));
        }
    }
    
    println!("Matched: {}", results.iter().filter(|&&x| x).count());
}
"""

print("=" * 70)
print("Memory Usage Comparison: ReXile vs Regex Crate")
print("=" * 70)
print()

# Note: This script requires manual testing as it needs cargo dependencies
print("⚠️  This script requires manual setup:")
print("1. Create two separate Rust projects (rexile_test and regex_test)")
print("2. Add dependencies (rexile or regex)")
print("3. Run valgrind massif on each")
print()
print("Example commands:")
print("=" * 70)
print()
print("# For ReXile test:")
print("cargo new --bin rexile_memory_test")
print("cd rexile_memory_test")
print('echo \'rexile = { path = "../rexile" }\' >> Cargo.toml')
print("# Add test code to src/main.rs")
print("cargo build --release")
print("valgrind --tool=massif --massif-out-file=massif.rexile.out ./target/release/rexile_memory_test")
print("ms_print massif.rexile.out | grep peak")
print()
print("# For Regex test:")
print("cargo new --bin regex_memory_test")
print("cd regex_memory_test")
print('echo \'regex = "1"\' >> Cargo.toml')
print("# Add test code to src/main.rs")
print("cargo build --release")
print("valgrind --tool=massif --massif-out-file=massif.regex.out ./target/release/regex_memory_test")
print("ms_print massif.regex.out | grep peak")
print()
print("=" * 70)
print()
print("Quick estimation based on structure sizes:")
print()

# Estimate based on typical sizes
print("ReXile Pattern struct size: ~200-500 bytes (varies by pattern type)")
print("  - Literal: ~80 bytes (String + memchr)")
print("  - Anchored: ~100 bytes")
print("  - CharClass: ~200 bytes (ASCII bitmap)")
print("  - Sequence: ~300-500 bytes (Vec of elements)")
print()
print("Regex Pattern struct size: ~100-200 bytes (highly optimized)")
print("  - JIT compiled bytecode: Larger initial compilation")
print("  - DFA cache: Can grow during execution")
print()
print("For 1000 simple literal patterns:")
print("  - ReXile: ~80-100 KB (estimated)")
print("  - Regex: ~100-200 KB base + compilation overhead")
print()
print("Note: Regex trades compile-time memory for runtime speed.")
print("      ReXile trades runtime speed for simpler structures.")
