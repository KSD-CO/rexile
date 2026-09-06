#!/usr/bin/env python3
"""Compare every original Pattern case in three adjacent, alternating pairs."""

import argparse
import hashlib
import json
import math
import os
from pathlib import Path
import re
import runpy
import subprocess
import sys
import time

ROOT = Path(__file__).resolve().parents[1]


def geometric_mean(values):
    return math.exp(sum(map(math.log, values)) / len(values))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "target/pattern_set_validation")
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    source_hash = runpy.run_path(str(ROOT / "scripts/benchmark_pattern_set.py"))["source_hash"]
    fingerprint = source_hash()
    script_fingerprint = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
    worktree = ROOT / "target/patternset-baseline"
    if subprocess.check_output(["git", "-C", str(worktree), "rev-parse", "HEAD"], text=True).strip() != subprocess.check_output(["git", "rev-parse", "b428e80"], cwd=ROOT, text=True).strip():
        raise RuntimeError("The original worktree is not at b428e80.")
    subprocess.run(["git", "-C", str(worktree), "diff", "--exit-code"], check=True, stdout=subprocess.DEVNULL)
    roots = {"original": worktree / "target", "current": ROOT / "target"}
    binaries = {}
    with (output / "pattern-paired-build.log").open("w") as log:
        for side in roots:
            manifest = worktree / "Cargo.toml" if side == "original" else ROOT / "Cargo.toml"
            command = ["cargo", "bench", "--locked", "--manifest-path", str(manifest), "--target-dir", str(roots[side]), "--bench", "rexile_benchmark", "--no-run", "--message-format=json"]
            result = subprocess.run(command, cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=log, check=True)
            for line in result.stdout.splitlines():
                artifact = json.loads(line)
                if artifact.get("reason") == "compiler-artifact" and artifact.get("executable") and artifact["target"]["name"] == "rexile_benchmark":
                    binaries[side] = artifact["executable"]
    if len(binaries) != 2:
        raise RuntimeError("Both benchmark executables are required.")
    cases = {}
    for path in (roots["original"] / "criterion").rglob("benchmark.json"):
        if path.parent.name == "original-b428e80":
            record = json.loads(path.read_text())
            if "rexile" in record["full_id"].split("/"):
                cases[record["full_id"]] = path.parent.parent.relative_to(roots["original"] / "criterion")
    if not cases:
        raise RuntimeError("Run the original Criterion baseline first.")
    document = {
        "source_sha256": fingerprint,
        "runner_sha256": script_fingerprint,
        "original_commit": "b428e80",
        "rustc": subprocess.check_output(["rustc", "--version", "--verbose"], text=True),
        "cpu": subprocess.check_output(["sysctl", "-n", "machdep.cpu.brand_string"], text=True).strip() if sys.platform == "darwin" else None,
        "started": time.time(),
        "rustflags": os.environ.get("RUSTFLAGS", ""),
        "cargo_build_target": os.environ.get("CARGO_BUILD_TARGET", ""),
        "binaries": {side: {"path": path, "sha256": hashlib.sha256(Path(path).read_bytes()).hexdigest()} for side, path in binaries.items()},
        "pairs": {case: [] for case in cases},
    }
    (output / "paired-runner.py").write_bytes(Path(__file__).read_bytes())
    raw_path = output / "pattern-paired-raw.json"
    with (output / "pattern-paired.log").open("w") as log:
        for run in range(3):
            print(f"Paired Pattern run {run + 1}/3", flush=True)
            for index, (case, directory) in enumerate(sorted(cases.items())):
                pair = {}
                order = ("original", "current") if (run + index) % 2 == 0 else ("current", "original")
                for side in order:
                    # Identical short labels avoid changing process/setup
                    # allocation layout between the two measurements. Every
                    # pair is copied into the raw document before reuse.
                    baseline = "paired"
                    command = [binaries[side], "--bench", "--noplot", "--save-baseline", baseline, "^" + re.escape(case) + "$"]
                    # Criterion uses the working directory to select target/criterion.
                    cwd = worktree if side == "original" else ROOT
                    env = os.environ.copy()
                    env["CRITERION_HOME"] = str(roots[side] / "criterion")
                    subprocess.run(command, cwd=cwd, env=env, stdout=log, stderr=subprocess.STDOUT, check=True)
                    path = roots[side] / "criterion" / directory / baseline
                    pair[side] = {"command": command, "estimates": json.loads((path / "estimates.json").read_text()), "sample": json.loads((path / "sample.json").read_text())}
                document["pairs"][case].append(pair)
                if (index + 1) % 30 == 0:
                    print(f"  {index + 1}/{len(cases)} cases", flush=True)
            if source_hash() != fingerprint or hashlib.sha256(Path(__file__).read_bytes()).hexdigest() != script_fingerprint:
                raise RuntimeError("Source or runner changed during measurement.")
            raw_path.write_text(json.dumps(document) + "\n")
    failures = []
    rows = []
    for case, pairs in sorted(document["pairs"].items()):
        if len(pairs) != 3 or any(len(pair[side]["sample"]["times"]) < 20 for pair in pairs for side in roots):
            failures.append(f"Incomplete samples: {case}")
            continue
        before = [pair["original"]["estimates"]["median"] for pair in pairs]
        after = [pair["current"]["estimates"]["median"] for pair in pairs]
        ratio = geometric_mean([a["point_estimate"] / b["point_estimate"] for a, b in zip(after, before)])
        lower = geometric_mean([a["confidence_interval"]["lower_bound"] / b["confidence_interval"]["upper_bound"] for a, b in zip(after, before)])
        rows.append({"case": case, "time_ratio": ratio, "interval_envelope_lower": lower})
        if ratio > 1.05:
            failures.append(f"{case}: {ratio:.3f}x ({'significant' if lower > 1.05 else 'unresolved'})")
    verdict = {"passed": not failures, "source_sha256": fingerprint, "limit": 1.05, "failures": failures, "rows": rows}
    (output / "pattern-regression-paired.json").write_text(json.dumps(verdict, indent=2) + "\n")
    lines = ["# Paired Pattern regression report", "", f"Gate: **{'PASS' if verdict['passed'] else 'FAIL / UNRESOLVED'}**.", "", "Every original case is measured in three adjacent pairs, alternating engine order. Ratios are geometric means across the three pairs; values above 1.05 fail. The original 20-sample Criterion settings are unchanged. Reported envelopes are not confidence intervals for ratios.", "", "| Case | Current / original time |", "|---|---:|"]
    lines += [f"| {row['case']} | {row['time_ratio']:.3f}× |" for row in rows]
    (output / "pattern-regression-paired.md").write_text("\n".join(lines) + "\n")
    print(f"Paired {len(rows)} cases; {len(failures)} unmet checks.", flush=True)
    for failure in failures:
        print(failure, flush=True)
    return 0 if verdict["passed"] else 2


if __name__ == "__main__":
    sys.exit(main())
