Purpose
This file instructs agentic coding assistants how to build, lint, test, and contribute to this repository.
Treat it as the single source of operational and style expectations for automated agents working here.

Repository layout (important paths)
- `Cargo.toml` - crate metadata and rust-version (1.70)
- `.github/workflows/rust.yml` - CI pipeline (format, clippy, build, tests, docs, examples)
- `src/` - library source (primary work)
- `tests/` - integration tests (each file is a test binary)
- `examples/` - runnable examples used by CI
- `benches/` - benchmark harnesses (criterion)

Build / Lint / Test (common commands)
- Build (debug): `cargo build --verbose`
- Build (release): `cargo build --release --verbose`
- Run all tests: `cargo test --verbose`
- Run doc tests: `cargo test --doc --verbose`
- Run a single integration test binary (file in `tests/`):
  `cargo test --test test_unicode`
- Run all tests in a particular integration test file but show stdout:
  `cargo test --test test_unicode -- --nocapture`
- Run a single test function (when you know the test name):
  `cargo test <test_name> -- --exact --nocapture`
  Example: run `unicode_match` inside `tests/test_unicode.rs`:
  `cargo test unicode_match --test test_unicode -- --exact --nocapture`
- Run a single unit test (module-local):
  `cargo test module_name::test_name -- --exact --nocapture`
- Run unit tests for a single module by substring filter:
  `cargo test module_name`
- Run benchmarks (criterion): `cargo bench --verbose`
- Run examples (used by CI):
  `cargo run --release --example comprehensive all`
  `cargo run --release --example perf_compare`
- Format check (CI): `cargo fmt -- --check` (to fix: `cargo fmt --all`)
- Lint (CI): `cargo clippy --all-targets -- -D warnings`

Useful runtime / debug flags
- Enable backtrace: `RUST_BACKTRACE=1 cargo test ...`
- Run tests single-threaded: add `-- --test-threads=1`
- Show stdout from tests: add `-- --nocapture`
- Limit test output & list tests: `cargo test -- --list`
- Run tests under feature flags: `cargo test --features "foo bar"`

Notes about running a single test
- Rust test selection is a substring filter by default. Use `-- --exact` when you need an exact match.
- Integration tests in `tests/` compile to separate test binaries and are invoked with `--test <name>` (omit `.rs`).
- Unit tests live in the same crate and are filtered by the standard cargo filter (no `--test`).

Environment / toolchain
- This repo targets Rust 1.70 (see `rust-version` in `Cargo.toml`). Use `rustup` to install: `rustup install 1.70.0` and `rustup override set 1.70.0` when working here.

Code style and guidelines (apply to all code changes)

Formatting
- Run `cargo fmt` before committing. CI enforces `cargo fmt -- --check`.
- Prefer short, readable lines; break complex expressions across lines with clear indentation.

Imports and module organization
- Group imports in three blocks (blank line between each):
  1) `std` imports
  2) external crates
  3) internal crate imports (`crate::...` or `super::...`)
  Example:
  `use std::fmt;`
  `use aho_corasick::AhoCorasick;`
  `use crate::parser::Sequence;`
- Prefer explicit imports; avoid `use foo::*;` except in small tests/examples where brevity helps.
- Prefer absolute `crate::` paths for internal modules to make refactors safer.

Visibility and module boundaries
- Keep functions private (`fn`) by default. Make items `pub` only when they form part of the public API.
- Document `pub` items with `///` doc comments and include usage examples for non-trivial APIs.

Types and naming
- snake_case for functions and variables, PascalCase for structs/enums/traits, UPPER_SNAKE_CASE for constants.
- Keep generic parameter names short and idiomatic (`T`, `U`, `E`); use descriptive names only when they improve readability.
- Use type aliases sparingly and only when they improve clarity across multiple modules.

Error handling
- Return `Result<T, E>` for fallible functions and propagate errors with `?` where appropriate.
- Define centralized crate error types when multiple modules share error semantics (e.g., `PatternError`).
- Avoid `unwrap()` and `expect()` in library code. They are acceptable in small examples, benchmarks, or test scaffolding.

Panics and assertions
- Avoid panicking in public library code. Use `Result`/`Option` to handle recoverable errors.
- Use `assert!`/`assert_eq!` in tests for invariants; prefer clearer error messages in `panic!` calls when panicking is unavoidable.

Documentation and comments
- Document all public types and functions with `///`. Include short examples for complex APIs.
- Use inline `//` comments to explain non-obvious algorithmic choices or performance tradeoffs.
- Keep comments factual; avoid TODOs without an associated issue number.

Performance-sensitive code
- Favor zero-allocation algorithms in hot paths. Reuse buffers and slices where possible.
- Avoid unnecessary cloning inside loops. Prefer `&str`/slices and iterator adapters.
- When `unsafe` is required, wrap it in a small, well-documented function and add a safety comment explaining invariants.

Concurrency and synchronization
- Use `OnceLock`, `Mutex`, or other standard primitives when needed. Minimize lock scope and prefer lock-free patterns in hot paths.

Tests and examples
- Add unit tests next to modules using `#[cfg(test)] mod tests`.
- Place integration tests in `tests/` and name them `tests/test_*.rs` when possible.
- Keep tests deterministic: seed RNGs and avoid time-sensitive assertions.
- Use `-- --nocapture` during local debugging to see test output.

Clippy and linting
- CI treats clippy warnings as errors: run `cargo clippy --all-targets -- -D warnings` locally before pushing.
- The repository contains targeted Clippy allowances in `src/lib.rs`: do not remove those allowances without a strong rationale and a follow-up PR.

Benchmarks and performance work
- Add new benchmarks in `benches/` for performance-sensitive changes and run `cargo bench` locally.
- When introducing a micro-optimisation, include benchmarks and a short note in the PR describing the improvement.

Git / commit rules for agents
- Do not create commits unless the user explicitly asks the agent to commit changes.
- Never run destructive git commands (`git reset --hard`, `git checkout --`, force-push) unless explicitly requested.
- If asked to commit, create a concise commit message that explains the "why" (1–2 sentences) and run `cargo fmt` and tests first.

CI and release notes
- CI runs format, clippy, build, tests, doc tests and examples. Keep PRs small and self-contained to keep CI green.
- If a change affects `Cargo.toml` versioning or publishing, escalate to a human reviewer.

Cursor / Copilot rules
- No `.cursor/rules/` or `.cursorrules` were found in this repository. If you add cursor rules, place them under `.cursor/rules/` and include a short README describing behavior.
- No `.github/copilot-instructions.md` was found. If you add Copilot rules, place them at `.github/copilot-instructions.md` and reference them here.

Workflow for agents (operational)
- Make small, test-backed changes. Run `cargo test` and `cargo fmt` locally before proposing or committing code.
- Run `cargo clippy --all-targets -- -D warnings` before opening a PR.
- If the change is invasive (public API or performance-critical), add/update benchmarks and examples and document the rationale in the PR description.
- If blocked by ambiguity, ask exactly one targeted question and offer a recommended default; include the minimal files you inspected.

Quick references
- `Cargo.toml`
- `.github/workflows/rust.yml`
- `src/lib.rs`
- `tests/` directory

Contact / escalation
- If a change could affect publishing, crate versioning, or CI publishing jobs, raise it to a human reviewer.

End
