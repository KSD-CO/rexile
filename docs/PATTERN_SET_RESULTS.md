# PatternSet validation results

**Acceptance: FAIL.** The feature and correctness checks are implemented, but the requirement that every primary case beat its applicable regex baseline is unmet. These results do not support a universal speed claim.

The initial PatternSet measurements below are for commit `3acd195`, before the
literal-dispatch follow-up described later on this page. Measured on Apple M1
Pro, rustc 1.98.0 (88d9e12ae 2026-08-18) (Homebrew), against regex 1.13.1. Source
fingerprint: `dc971fa570d7cfd3f6725ecf864bd892abb6035492a2ff7a16aeb65646c48fbd`.

The frozen corpus contains 144 input configurations. All three runs completed, with 100 samples per primary measurement and 4,246 benchmark records per run. Earliest-result baselines stop at a provably optimal start-zero match; capture comparisons include direct and span-first strategies.

| Operation | Geometric mean speedup | Target |
|---|---:|---:|
| `compile` | 3.470× | 2.0× |
| `is_match` | 11.806× | 1.3× |
| `matches` | 10.726× | 1.3× |
| `find` | 6.298× | 1.5× |
| `find_each` | 11.694× | 1.5× |
| `find_iter` | 6.218× | 1.5× |
| `captures` | 4.644× | 1.5× |
| `captures_each` | 11.048× | 1.5× |
| `captures_iter` | 5.992× | 1.5× |
| `visit_captures` | 8.000× | 1.5× |

Peak heap geometric mean ratio: 0.131, a reduction of 86.9%. No primary memory case increased peak heap. Timings were measured separately from allocator instrumentation.

## Unmet cases

These three configurations were slower in all three runs. All other primary comparisons passed their individual timing checks.

| Case | Rexile time / best baseline time |
|---|---:|
| `keywords/32/65536/dense/captures_each` | 1.101–1.102× |
| `mixed/32/65536/dense/captures_each` | 1.018–1.018× |
| `mixed/32/65536/dense/find_each` | 1.014–1.017× |

The control workloads also expose limits: nested captures, prefix churn, and Unicode fallbacks can be substantially slower than regex. They are retained in the raw report and are not covered by the primary-corpus speed targets. Lookaround and backreference controls have no regex baseline because that crate rejects those extensions.

## Correctness and compatibility

- 231 unit, integration, and documentation tests passed.
- The full frozen corpus has identical IDs, spans, and numbered captures for the compared syntax.
- Both visitor modes and cached queries allocate zero additional heap blocks after warm-up on the primary corpus.
- Formatting, Clippy with warnings denied, documentation builds, and all required examples passed.
- An isolated Rust 1.70 consumer compiled and ran the library; the full development suite used Rust 1.98.
- The existing `Pattern` performance check passed all 90 cases across three adjacent pairs. The maximum mean time ratio was 1.046×, below the 1.05 limit.

## Follow-up: single-pattern literal matching

Review of `perf_compare` found a drop from
[13/22 on the base commit](https://github.com/KSD-CO/rexile/actions/runs/33860527716)
to [11/22 on the initial PR](https://github.com/KSD-CO/rexile/actions/runs/34008819890).
The two lost wins were `ERROR` and `calculate_total`. Rexile's reported times
increased from 14.6 to 20.6 ns and from 14.7 to 17.4 ns, respectively. These
counts concern ordinary `Pattern::is_match`, not PatternSet. The two CI runs
also used different runner images and regex versions (1.12.4 and 1.13.1).

The new boolean fast path sent literal queries through a second enum dispatch
in `find`. The follow-up dispatches literal and case-insensitive literal
queries directly to their existing search functions. It also adds
`literal_log` and `literal_code` Criterion cases with the exact inputs from
`perf_compare`; the 22-case example itself is unchanged.

A diagnostic comparison on the same M1 Pro compiled the base, initial PR, and
fix with identical dependencies. It measured all 22 example cases in 15
rounds, rotating engine order, with 500,000 iterations per measurement.
Medians for the two literals were:

| Pattern | Base (ns) | Initial PR (ns) | Fix (ns) |
|---|---:|---:|---:|
| `ERROR` | 6.80 | 7.39 | 6.49 |
| `calculate_total` | 8.38 | 8.75 | 8.03 |

All three versions won 13/22 comparisons against regex in this diagnostic.
The count alone therefore missed the smaller literal slowdowns on M1. The
earlier 90-case regression check used different literal inputs and did not
cover these exact cases.

The added Criterion cases measured the fix at 6.467 ns versus regex at
9.397 ns for the log, and 8.075 ns versus 10.571 ns for the code snippet.
These are local measurements, not a prediction of the win count on other
machines. The initial full PatternSet table has not been remeasured for this
follow-up.

```sh
cargo bench --locked --bench rexile_benchmark -- 'is_match_supported_subset/(rexile|regex)/literal_(log|code)$' --noplot
```

## Reproduction

See [the benchmark instructions](PATTERN_SET.md#reproducible-performance-checks).
The benchmark runners write local artifacts under `target/pattern_set_validation/`:

- `report.md` and `verdict.json`: primary timing and heap results.
- `pattern-regression-paired.md`: paired single-pattern regression results.
- `source.zip`: the source snapshot used for the primary measurements.

Raw samples, compiler and machine metadata, and allocation measurements are in
the same directory. These generated files are not checked into Git; this page
records the measured summary. The source snapshot permits reproducing the
measured implementation after later workspace edits. The local validation also
recorded the combined status in `acceptance.json`; that file is a validation
summary, not an output of either benchmark runner.
