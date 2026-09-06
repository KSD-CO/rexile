# PatternSet

See [the measured validation results](PATTERN_SET_RESULTS.md) before making
performance claims. The per-case speed target is not yet fully met.

PatternSet compiles multiple rexile patterns and reports their IDs, match
locations, and numbered captures. Clones share immutable compiled state; each
concurrent search owns its scratch memory.

```rust
use rexile::PatternSet;

let set = PatternSet::new([
    r"user=([a-z]+)",
    r"code=([0-9]+)",
    "user=alice",
])?;
let text = "code=42 user=alice";
assert_eq!(set.matches(text).iter().collect::<Vec<_>>(), [0, 1, 2]);

let hits: Vec<_> = set.captures_iter(text).collect();
assert_eq!(hits[0].pattern_id(), 1);
assert_eq!(hits[0].get(1), Some("42"));
assert_eq!(hits[1].get(1), Some("alice"));
assert_eq!(hits[2].get(0), Some("user=alice"));
# Ok::<(), rexile::PatternSetError>(())
```

Run the complete example with `cargo run --release --example pattern_set`.

## Search contract

| Operation | Result and ordering |
|---|---|
| `new`, `patterns`, `len`, `is_empty` | Construction and source-pattern metadata |
| `is_match` | Whether any rule matches; may stop immediately |
| `matches` | Matched IDs, iterated in ascending input order |
| `find`, `captures` | Earliest match; smallest ID wins a start-position tie |
| `find_each`, `captures_each` | First match of each matching rule, in ID order |
| `find_iter`, `captures_iter` | All independent matches, ordered by start then ID |
| `is_match_with_cache`, `matches_with_cache` | Reuse search buffers |
| `visit_matches`, `visit_captures` | Ordered callbacks with early cancellation |

An ID is the zero-based position of a pattern in the constructor input.
Duplicate patterns retain distinct IDs. An empty set is valid and never matches.
Within each rule, matching follows its own alternation and greedy/lazy priorities,
and successive matches do not overlap. Matches belonging to different rules
may overlap freely. The search does not select one winning rule and discard
the others.

All positions are byte offsets into the original UTF-8 haystack. Capture zero
is the full match; capture numbers restart for each rule. An absent group is
`None`, which differs from an empty captured string. Empty matches advance at
character boundaries and follow the usual suppression of an empty match
immediately adjacent to a previously returned match of the same rule.

`PatternSetError::Pattern` retains the failing input ID and the original
`PatternError`. Index construction errors use `PatternSetError::Build`.
Construction either returns the entire set or an error.

## Reusing memory

```rust
use std::ops::ControlFlow;
use rexile::{PatternSet, SetSearchMode};

let set = PatternSet::new([r"code=([0-9]+)"])?;
let mut cache = set.create_cache();
let mut total = 0;
let _ = set.visit_captures(
    "code=42 code=7",
    &mut cache,
    SetSearchMode::All,
    |hit| {
        total += hit.get(1).unwrap().parse::<u32>().unwrap();
        ControlFlow::<()>::Continue(())
    },
);
assert_eq!(total, 49);
# Ok::<(), rexile::PatternSetError>(())
```

Both visitor modes emit results by start then ID. `FirstPerPattern` emits at
most one result per rule; `All` emits all independent matches. Returning
`ControlFlow::Break(value)` stops the traversal and returns that value.

`SetCapturesRef` borrows capture positions for the duration of the callback.
Its extracted strings borrow the haystack and may be retained. Call
`to_owned()` to retain capture positions too. Owned capture iterators allocate
storage for returned capture groups; visitors avoid those result allocations.
The deterministic paths reuse their working storage after warm-up. General
fallback matchers may still allocate internally, and user callbacks control
their own allocations.

A cache can be reused with a different set, different haystack, or set clone;
buffers reset and grow as needed. It never retains a haystack reference.
Share a set across threads and create a separate cache for each simultaneous
search.

## Execution

Compilation reuses rexile's parser. Plain literals bypass unnecessary matcher
construction. Proven deterministic sequences compile to capture programs with
explicit group boundaries. Their ASCII predicates also identify when the
program can safely execute against arbitrary UTF-8 input. Other patterns keep
the general matcher.
Programs with captures also compile a sequence without group instructions for
boolean and span queries, so these queries do not execute capture operations.

The shared literal index maps required literals to all owning rules.
Prefix candidates are checked at their exact source start. Deterministic
programs can also recover a start from a mandatory internal literal when
disjoint delimiters make the preceding atoms unambiguous. Unsupported
optimizations always retain the general execution path.

Short, equal-width entries use SIMD candidate searches and a hash table with
randomized hashing. Other entries use Aho–Corasick; dense candidate streams
can switch to automaton scanning. A small cache avoids hashing repeatedly
encountered prefixes. Variable-width prefixes use
overlapping searches and a bounded reordering window. Internal
literal searches and general fallbacks contribute their next match to the
ordered merge. No haystack-wide list of all results is required by iterators.

## Compatibility and limits

PatternSet uses rexile's current pattern syntax and matching semantics, including
supported lookaround and backreferences. It does not introduce named captures,
Unicode property classes, scoped flags, bytes APIs, or streaming.

Rexile's `\d`, `\w`, and word boundaries use ASCII definitions. Case-insensitive Unicode
matching uses rexile's lowercase-based behavior and source-offset mappings;
it is not the complete Unicode simple-case-folding behavior of the regex
crate. Differential tests make these boundaries explicit instead of claiming
full regex syntax or Unicode equivalence. General backtracking expressions
retain their existing complexity.

## Potential use in rust-rule-engine

These are integration opportunities, not changes implemented or benchmarked in
rust-rule-engine. The references below describe its source at commit
`82fb75858a8351794b239aa1b76bfd723de49709`; its
[Rexile dependency is currently 0.5.5](https://github.com/KSD-CO/rust-rule-engine/blob/82fb75858a8351794b239aa1b76bfd723de49709/Cargo.toml#L26).
Adopting this API requires an explicit dependency update and integration.

- **Batch text predicates and validation.** The engine supports
  [custom RETE functions](https://github.com/KSD-CO/rust-rule-engine/blob/82fb75858a8351794b239aa1b76bfd723de49709/src/rete/propagation.rs#L741),
  and its [ValidateRegex action](https://github.com/KSD-CO/rust-rule-engine/blob/82fb75858a8351794b239aa1b76bfd723de49709/src/plugins/validation.rs#L108)
  compiles and evaluates one pattern per call. For a known group of regex
  predicates on the same fact field, an adapter could compile a set once,
  evaluate the group together, and map pattern IDs back to predicates. Returning
  all IDs preserves simultaneous matches; the engine still decides rule
  activation and salience. Any reuse of results across calls needs its own
  invalidation when facts or rules change: `SetCache` reuses scratch memory,
  not results.
- **Extract query options together.** The backward query parser separately
  [compiles and searches patterns for strategy, depth, limits, and flags](https://github.com/KSD-CO/rust-rule-engine/blob/82fb75858a8351794b239aa1b76bfd723de49709/src/backward/grl_query.rs#L542).
  A retained set plus `captures_each` or a `FirstPerPattern` visitor could
  identify these fields and extract their values through one API call.
  Default values and the current first-occurrence behavior must be preserved.
- **Classify GRL clauses with captures.** The existing
  [GRL parser](https://github.com/KSD-CO/rust-rule-engine/blob/82fb75858a8351794b239aa1b76bfd723de49709/src/parser/grl.rs#L1047)
  tries several cached patterns on each clause. Pattern IDs could select the
  parser branch while captures supply its fields. Existing branch priority
  must be preserved by selecting the lowest applicable pattern ID; the global
  `captures` method instead prioritizes the earliest source position. Several
  of these expressions use general fallback matching, so performance needs
  measurement on real GRL inputs.

Capture visitors also let a preprocessing adapter borrow extracted text and
reuse per-worker scratch space before creating owned engine facts. The
engine's built-in
[`matches` operator uses wildcard semantics](https://github.com/KSD-CO/rust-rule-engine/blob/82fb75858a8351794b239aa1b76bfd723de49709/src/rete/alpha.rs#L28);
PatternSet is not a direct semantic replacement for that operator. The
separate `GRLParserNoRegex` path would not benefit automatically. No
end-to-end rust-rule-engine speedup is claimed.

## Reproducible performance checks

The fixed acceptance corpus is
`benches/support/pattern_set_workloads.rs`: four rule families, three set
sizes, three haystack lengths, and four match densities (144 combinations).
Integration tests compare every match and capture with `regex 1.13.1`.

```sh
# Full timing and heap gate, with raw samples and machine/source metadata.
python3 scripts/benchmark_pattern_set.py

# Continue only completed runs with identical source and options.
python3 scripts/benchmark_pattern_set.py --resume

# Diagnostic only: explicitly cannot pass acceptance.
python3 scripts/benchmark_pattern_set.py --runs 1 --quick

# Allocation and correctness checks.
cargo test --test test_pattern_set --test test_pattern_set_corpus
cargo test --test test_pattern_set_allocations
```

The runner compares ID queries with RegexSet, and richer queries with both
independent Regex searches and RegexSet followed by matching Regex searches.
It selects the fastest applicable baseline per operation. Capture visitors
also have a reusable CaptureLocations baseline with identical result ordering.
Earliest-match baselines stop as soon as a start-zero result cannot be improved.
Global captures compare both direct capture searches and span-first searches
followed by capture extraction.
Heap measurements run in separate processes using an allocator observer;
their execution times are never used as latency measurements.

The report fails closed on missing cases, fewer than three runs, fewer than
100 samples, changed source, uncertain timing comparisons, regressions, or
unmet aggregate thresholds. Its confidence envelope is derived from the two
individual Criterion intervals; it is not presented as a confidence interval
for their ratio. Allocation-free visitors and the existing Pattern performance
regression check are separate required checks.

To reproduce the ordinary `Pattern` comparison from a fresh checkout:

```sh
git worktree add --detach target/patternset-baseline b428e80
cargo bench --locked --manifest-path target/patternset-baseline/Cargo.toml --target-dir target/patternset-baseline/target --bench rexile_benchmark -- --noplot --save-baseline original-b428e80
cargo bench --locked --bench rexile_benchmark -- --noplot --save-baseline patternset-current
python3 scripts/check_pattern_regression.py
```

Use the same compiler, build flags, and machine for both runs. The comparison
retains the original single-pattern workloads and their Criterion settings.
For a comparison less sensitive to drift between entire benchmark runs,
`python3 scripts/benchmark_pattern_regression.py` prebuilds both binaries and
measures each case in three adjacent pairs, alternating their order. It keeps
the same 5% threshold and writes separate `pattern-regression-paired` reports
and raw samples alongside the original comparison.

| Gate | Target |
|---|---|
| Compile, geometric mean | At least 2× faster |
| Boolean and ID queries, each operation | At least 1.3× faster |
| Match and capture queries, each operation | At least 1.5× faster |
| Every primary case | No slowdown versus its applicable baseline |
| Peak heap | At least 20% geometric mean reduction, no per-case increase |
| Warmed visitors on the primary corpus | Zero additional allocations |
| Existing Pattern APIs | No performance regression exceeding 5% |

Reports and raw data live in `target/pattern_set_validation/`. Treat a failed
or incomplete report as an unmet performance target, not proof of an overall
speed advantage. Performance results apply to the measured workload,
dependencies, compiler, and hardware.
The runner also saves `source.zip` with the Rust sources, tests, examples,
lockfile, and scripts needed to reproduce that implementation.
