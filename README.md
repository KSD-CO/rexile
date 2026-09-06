# ReXile 🦎

[![Crates.io](https://img.shields.io/crates/v/rexile.svg)](https://crates.io/crates/rexile)
[![Documentation](https://docs.rs/rexile/badge.svg)](https://docs.rs/rexile)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)

**A regex engine built for fast compilation.** 100% safe Rust, zero unsafe code, only `memchr` + `aho-corasick` as dependencies.

| | vs `regex` crate |
|---|---|
| ⚡ Compilation | **10-100x faster** |
| 🚀 Matching (simple patterns) | **~1.3-3x faster** |
| 📦 Memory (compilation) | **~15x less** |
| 💎 Extras | Lookaround & backreferences (not in `regex`) |

Best fit for workloads that compile many patterns at runtime — rule engines, parsers, DSLs — where startup latency matters more than raw throughput on complex patterns.

## Quick Start

Add to `Cargo.toml`:

```toml
[dependencies]
rexile = "0.7"
```

```rust
use rexile::Pattern;

let pattern = Pattern::new(r"\d+").unwrap();
assert!(pattern.is_match("Order #12345"));
assert_eq!(pattern.find("Order #12345"), Some((7, 12)));

// Capture groups
let email = Pattern::new(r"(\w+)@(\w+)").unwrap();
let caps = email.captures("admin@example").unwrap();
assert_eq!(caps.get(1), Some("admin"));
assert_eq!(caps.get(2), Some("example"));

// Replace / split
assert_eq!(pattern.replace_all("a1 b22", "X"), "aX bX");
let parts: Vec<_> = Pattern::new(r"\s+").unwrap().split("a  b   c").collect();
assert_eq!(parts, vec!["a", "b", "c"]);

// One-off matching via the cached global API (compiles once, reuses after)
assert!(rexile::is_match("ERROR", "line: ERROR occurred").unwrap());
```

More runnable examples live in [examples/](examples/) — see [examples/README.md](examples/README.md) for a guided tour, or run:

```bash
cargo run --example comprehensive            # feature + benchmark showcase
cargo run --release --example perf_compare   # vs regex crate
```

## Features

| Feature | Syntax |
|---|---|
| Literals, alternation | `hello`, `foo\|bar\|baz` |
| Character classes | `[a-z]`, `[^0-9]` |
| Quantifiers (greedy & lazy) | `*`, `+`, `?`, `{n,m}`, `*?`, `+?` |
| Escape sequences | `\d`, `\w`, `\s`, `\b`, `\B` |
| Anchors | `^`, `$`, multiline `(?m)` |
| Dot wildcard / DOTALL | `.`, `.*`, `(?s)` |
| Groups | `(...)`, `(?:...)` |
| Global flags | `(?i)`, `(?m)`, `(?s)`, combined `(?ims)` |
| Capturing groups | `(\w+)` with `.captures()` |
| Pattern sets | `PatternSet`: rule IDs, locations, captures, reusable visitors |
| Lookaround | `(?=...)`, `(?!...)`, `(?<=...)`, `(?<!...)` |
| Backreferences | `\1`, `\2` |
| Replace / split | `.replace()`, `.replace_all()`, `.split()` |

Full status and version history: [FEATURE_STATUS.md](FEATURE_STATUS.md) · [CHANGELOG.md](CHANGELOG.md)

See [PatternSet](docs/PATTERN_SET.md) for multi-rule matching, overlap semantics,
capture visitors, and reproducible performance checks.

Not yet supported: Unicode property classes (`\p{L}`), named capture groups,
scoped/toggled flags (`(?i:...)`, `(?-i)`), and `x`, `u`, `U`, `R` flags.
Flags are global and must appear at the beginning of a pattern. Multiline mode
uses LF line boundaries; CRLF mode is not supported.

## Performance

ReXile's advantage is **compilation speed** — matching speed is competitive on simple/common patterns and slower on complex backtracking patterns. Summary from the latest benchmarks (details in [PERFORMANCE_RESULTS.md](PERFORMANCE_RESULTS.md)):

- Compiling `\d+`, `[a-zA-Z_]\w*`, and similar patterns: **40-100x faster** than `regex`.
- Matching character classes and bounded quantifiers (`[0-9]+`, `\d{4}`): **~2-3x faster**.
- Matching complex captures/sequences (`(\w+)@(\w+)`, `(?i)`): **2-4x slower** — an accepted trade-off.
- Loading 1000 patterns at startup: **~100x faster** (0.02s vs ~2s).

Run the benchmarks yourself:

```bash
cargo run --release --example comprehensive benchmark
cargo run --release --example perf_compare
```

## When to use ReXile

✅ Good fit: parsers/lexers, rule engines (e.g. GRL/business rules), applications compiling many dynamic patterns, memory-constrained environments.

⚠️ Consider `regex` instead if: you need Unicode properties, or your hot path is dominated by complex backtracking patterns (case-insensitive captures, heavy sequences).

## Contributing

Issues and PRs welcome — see [open issues](https://github.com/KSD-CO/rexile/issues).

```bash
make check   # fmt + clippy + tests (run before opening a PR)
make ci      # full CI-equivalent check
```

See [AGENTS.md](AGENTS.md) for build/test/style conventions, and [ROADMAP_FULL_REGEX.md](ROADMAP_FULL_REGEX.md) for planned work.

## License

Dual-licensed under either of:

- MIT License ([LICENSE-MIT](LICENSE-MIT))
- Apache License, Version 2.0 ([apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.

## Credits

Built on [`memchr`](https://docs.rs/memchr) and [`aho-corasick`](https://docs.rs/aho-corasick) (both by Andrew Gallant) for SIMD-accelerated search. Originally developed for [rust-rule-engine](https://github.com/KSD-CO/rust-rule-engine) to speed up GRL (Grule Rule Language) parsing.
