#!/usr/bin/env python3
"""Compare the unchanged single-Pattern corpus against the original worktree."""

import argparse
import json
from pathlib import Path
import sys


def read(root, baseline):
    results = {}
    for path in root.rglob("benchmark.json"):
        if path.parent.name != baseline:
            continue
        record = json.loads(path.read_text())
        if "rexile" not in record["full_id"].split("/"):
            continue
        results[record["full_id"]] = json.loads((path.parent / "estimates.json").read_text())["median"]
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--original", type=Path, default=Path("target/patternset-baseline/target/criterion"))
    parser.add_argument("--current", type=Path, default=Path("target/criterion"))
    parser.add_argument("--output", type=Path, default=Path("target/pattern_set_validation"))
    args = parser.parse_args()
    original = read(args.original, "original-b428e80")
    current = read(args.current, "patternset-current")
    rows = []
    failures = []
    if not original:
        failures.append("No original measurements found.")
    for case, before in sorted(original.items()):
        if case not in current:
            failures.append(f"Missing current measurement: {case}")
            continue
        after = current[case]
        ratio = after["point_estimate"] / before["point_estimate"]
        lower = after["confidence_interval"]["lower_bound"] / before["confidence_interval"]["upper_bound"]
        rows.append({"case": case, "time_ratio": ratio, "interval_envelope_lower": lower,
                     "before_ns": before["point_estimate"], "after_ns": after["point_estimate"]})
        if ratio > 1.05:
            evidence = "significant" if lower > 1.05 else "unresolved"
            failures.append(f"{case}: {ratio:.3f}x ({evidence})")
    report = {"passed": not failures, "limit": 1.05, "failures": failures, "rows": rows}
    args.output.mkdir(parents=True, exist_ok=True)
    (args.output / "pattern-regression.json").write_text(json.dumps(report, indent=2) + "\n")
    lines = ["# Existing Pattern performance", "", f"Gate: **{'PASS' if report['passed'] else 'FAIL / UNRESOLVED'}**.",
             "", "Before: detached b428e80 worktree. After: current implementation.",
             "Both use the repository's unchanged single-Pattern Criterion workloads.",
             "Ratios above 1.05 fail this check; reported envelopes are not ratio confidence intervals.",
             "", "| Case | Current / original time |", "|---|---:|"]
    lines += [f"| {row['case']} | {row['time_ratio']:.3f}× |" for row in rows]
    (args.output / "pattern-regression.md").write_text("\n".join(lines) + "\n")
    print(f"Compared {len(rows)} cases; {len(failures)} unmet checks.")
    for failure in failures[:20]:
        print(failure)
    return 0 if report["passed"] else 2


if __name__ == "__main__":
    sys.exit(main())
