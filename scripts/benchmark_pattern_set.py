#!/usr/bin/env python3
"""Run reproducible PatternSet comparisons and fail closed on unmet gates."""

import argparse
import hashlib
import json
import math
import os
import platform
from pathlib import Path
import subprocess
import sys
import time
import zipfile

ROOT = Path(__file__).resolve().parents[1]
FAMILIES = ("keywords", "fields", "captures", "mixed")
COUNTS = (32, 128, 1024)
LENGTHS = (256, 4096, 65536)
DENSITIES = ("none", "early", "late", "dense")
OPERATIONS = (
    "is_match", "matches", "find", "find_each", "find_iter",
    "captures", "captures_each", "captures_iter", "visit_captures",
)


def source_hash():
    digest = hashlib.sha256()
    files = list((ROOT / "src").rglob("*.rs"))
    files += list((ROOT / "benches").rglob("*.rs"))
    files += [ROOT / "Cargo.toml", ROOT / "Cargo.lock"]
    files += [ROOT / "examples/pattern_set_memory.rs"]
    files += [Path(__file__).resolve()]
    for path in sorted(files):
        digest.update(str(path.relative_to(ROOT)).encode())
        digest.update(path.read_bytes())
    return digest.hexdigest()


def run(command, log):
    with log.open("w") as stream:
        subprocess.run(command, cwd=ROOT, stdout=stream, stderr=subprocess.STDOUT, check=True)


def collect(baseline):
    records = {}
    for path in (ROOT / "target/criterion").rglob("benchmark.json"):
        if path.parent.name != baseline:
            continue
        benchmark = json.loads(path.read_text())
        if not benchmark["full_id"].startswith(("pattern_set_compile/", "pattern_set_search/", "pattern_set_control/")):
            continue
        estimates = json.loads((path.parent / "estimates.json").read_text())
        sample = json.loads((path.parent / "sample.json").read_text())
        records[benchmark["full_id"]] = {
            "benchmark": benchmark, "estimates": estimates, "sample": sample,
        }
    return records


def geometric_mean(values):
    return math.exp(sum(map(math.log, values)) / len(values)) if values else 0.0


def analyze(runs, memory, expected_source):
    failures = []
    ratios = {"compile": []}
    bounds = {}
    rows = []
    for operation in OPERATIONS:
        ratios[operation] = []
    if len(runs) < 3:
        failures.append("Fewer than three timing runs.")
    for number, experiment in enumerate(runs, 1):
        if experiment["source_sha256"] != expected_source:
            failures.append(f"Run {number} has a different source fingerprint.")
        records = experiment["records"]
        if experiment.get("quick"):
            failures.append(f"Run {number} uses Criterion --quick and is diagnostic only.")
        for family in FAMILIES:
            for count in COUNTS:
                pairs = [("compile", f"pattern_set_compile/{{engine}}/{family}/{count}", ("regex_set", "regex_vec", "regex_set_vec"))]
                for length in LENGTHS:
                    for density in DENSITIES:
                        for operation in OPERATIONS:
                            engines = ("regex_set",) if operation in ("is_match", "matches") else ("regex_vec", "regex_set_vec")
                            if operation == "captures":
                                engines += ("regex_vec_direct", "regex_set_vec_direct")
                            pairs.append((operation, f"pattern_set_search/{family}/{count}/{length}/{density}/{operation}/{{engine}}", engines))
                for operation, template, engines in pairs:
                    name = template.format(engine="rexile")
                    required = [name] + [template.format(engine=engine) for engine in engines]
                    if any(key not in records for key in required):
                        failures.append(f"Run {number} missing comparisons: {name}")
                        continue
                    selected = min(engines, key=lambda engine: records[template.format(engine=engine)]["estimates"]["median"]["point_estimate"])
                    ours = records[name]
                    theirs = records[template.format(engine=selected)]
                    if min(len(ours["sample"]["times"]), len(theirs["sample"]["times"])) < 100:
                        failures.append(f"Run {number} has fewer than 100 samples: {name}")
                    ours_estimate = ours["estimates"]["median"]
                    theirs_estimate = theirs["estimates"]["median"]
                    speedup = theirs_estimate["point_estimate"] / ours_estimate["point_estimate"]
                    ratios[operation].append(speedup)
                    # This is an envelope from the two reported confidence
                    # intervals, not a claimed 95% interval for their ratio.
                    lower = theirs_estimate["confidence_interval"]["lower_bound"] / ours_estimate["confidence_interval"]["upper_bound"]
                    bounds.setdefault(operation, []).append(lower)
                    rows.append({"run": number, "case": name, "baseline": selected, "speedup": speedup, "interval_envelope_lower": lower})
                    if speedup < 1:
                        failures.append(f"Run {number} slower: {name} ({speedup:.3f}x)")
                    elif lower < 1:
                        failures.append(f"Run {number} statistically unresolved: {name}")
    summary = {}
    for operation, values in ratios.items():
        target = 2.0 if operation == "compile" else 1.3 if operation in ("is_match", "matches") else 1.5
        achieved = geometric_mean(values)
        conservative = geometric_mean(bounds.get(operation, []))
        summary[operation] = {"speedup": achieved, "target": target, "interval_envelope_lower": conservative}
        if not values or conservative < target:
            failures.append(f"{operation} aggregate below target: {achieved:.3f}x; conservative envelope {conservative:.3f}x < {target}x")
    memory_ratios = []
    for family in FAMILIES:
        for count in COUNTS:
            for length in LENGTHS:
                for density in DENSITIES:
                    for mode in ("ids", "captures"):
                        case = f"{family}/{count}/{length}/{density}/{mode}"
                        group = memory.get(case, {})
                        engines = ("regex_set",) if mode == "ids" else ("regex_vec", "regex_set_vec")
                        if any(engine not in group for engine in ("rexile", *engines)):
                            failures.append(f"Missing memory comparison: {case}")
                            continue
                        if len({row["checksum"] for row in group.values()}) != 1:
                            failures.append(f"Memory workload results differ: {case}")
                        baseline = min(group[engine]["total_peak"] for engine in engines)
                        ratio = group["rexile"]["total_peak"] / baseline
                        memory_ratios.append(ratio)
                        if ratio > 1:
                            failures.append(f"Peak heap increased: {case} ({ratio:.3f}x)")
    memory_ratio = geometric_mean(memory_ratios)
    if not memory_ratios or memory_ratio > 0.8:
        failures.append(f"Peak heap reduction below 20%: ratio {memory_ratio:.3f}")
    controls = []
    for number, experiment in enumerate(runs, 1):
        records = experiment["records"]
        for key, record in records.items():
            if not key.startswith("pattern_set_control/") or not key.endswith("/rexile"):
                continue
            prefix = key.rsplit("/", 1)[0] + "/"
            comparisons = {name[len(prefix):]: item for name, item in records.items() if name.startswith(prefix) and name != key}
            best = min(comparisons, key=lambda name: comparisons[name]["estimates"]["median"]["point_estimate"]) if comparisons else None
            controls.append({
                "run": number, "case": key, "baseline": best,
                "rexile_ns": record["estimates"]["median"]["point_estimate"],
                "speedup": comparisons[best]["estimates"]["median"]["point_estimate"] / record["estimates"]["median"]["point_estimate"] if best else None,
            })
    return {
        "passed": not failures, "summary": summary, "memory_ratio": memory_ratio,
        "failures": failures, "rows": rows, "controls": controls,
        "note": "Allocation-free visitors and ordinary Pattern regressions are separate required checks.",
    }


def write_report(directory, verdict):
    lines = [
        "# PatternSet benchmark report", "",
        f"Timing and heap gates: **{'PASS' if verdict['passed'] else 'FAIL / INCOMPLETE'}**", "",
        "Ratios compare the fastest applicable baseline for the same operation. "
        "Quick runs, missing cases, changed source, and inconclusive intervals fail closed.", "",
        "| Operation | Geometric mean speedup | Required |",
        "|---|---:|---:|",
    ]
    for operation, result in verdict["summary"].items():
        lines.append(f"| {operation} | {result['speedup']:.3f}× | {result['target']:.1f}× |")
    lines += ["", f"Peak heap ratio: {verdict['memory_ratio']:.3f}; required ≤ 0.800.", "",
              f"Unmet checks: {len(verdict['failures'])}. Full details and every case are in verdict.json.", ""]
    lines += ["- " + failure for failure in verdict["failures"][:40]]
    lines += ["", "## Control workloads", "", "| Run | Case | Baseline | Speedup |", "|---:|---|---|---:|"]
    for control in verdict["controls"]:
        ratio = f"{control['speedup']:.3f}×" if control["speedup"] is not None else "Unsupported by regex"
        lines.append(f"| {control['run']} | {control['case']} | {control['baseline'] or '—'} | {ratio} |")
    lines += ["", verdict["note"], ""]
    (directory / "report.md").write_text("\n".join(lines))
    (directory / "verdict.json").write_text(json.dumps(verdict, indent=2) + "\n")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=ROOT / "target/pattern_set_validation")
    parser.add_argument("--runs", type=int, default=3)
    parser.add_argument("--quick", action="store_true", help="Diagnostic only; cannot pass acceptance.")
    parser.add_argument("--filter", help="Criterion filter; incomplete matrices cannot pass acceptance.")
    parser.add_argument("--resume", action="store_true", help="Reuse complete runs with identical source and options.")
    parser.add_argument("--analyze-only", action="store_true")
    args = parser.parse_args()
    directory = args.output.resolve()
    directory.mkdir(parents=True, exist_ok=True)
    fingerprint = source_hash()
    if not args.analyze_only:
        files = [ROOT / "Cargo.toml", ROOT / "Cargo.lock", ROOT / "README.md"]
        files.extend(ROOT.glob("LICENSE*"))
        for name in ("src", "benches", "tests", "examples"):
            files.extend((ROOT / name).rglob("*.rs"))
        files.extend((ROOT / "scripts").glob("*.py"))
        with zipfile.ZipFile(directory / "source.zip", "w", zipfile.ZIP_DEFLATED) as snapshot:
            for path in sorted(files):
                snapshot.write(path, path.relative_to(ROOT))
    runs = []
    metadata = {
        "source_sha256": fingerprint,
        "source_snapshot": "source.zip",
        "rustc": subprocess.check_output(["rustc", "--version", "--verbose"], text=True),
        "platform": platform.platform(),
        "machine": platform.machine(),
        "regex": "1.13.1",
        "quick": args.quick, "filter": args.filter,
        "rustflags": os.environ.get("RUSTFLAGS", ""),
        "cargo_build_target": os.environ.get("CARGO_BUILD_TARGET", ""),
    }
    if sys.platform == "darwin":
        metadata["cpu"] = subprocess.check_output(["sysctl", "-n", "machdep.cpu.brand_string"], text=True).strip()
    for number in range(1, args.runs + 1):
        path = directory / f"run-{number}.json"
        cached = json.loads(path.read_text()) if path.exists() else None
        compatible = cached and all(cached.get(key) == metadata[key] for key in ("source_sha256", "quick", "filter"))
        if args.analyze_only:
            if cached:
                runs.append(cached)
            continue
        if args.resume and compatible:
            runs.append(cached)
            continue
        baseline = f"patternset-{fingerprint[:12]}-{number}"
        command = ["cargo", "bench", "--locked", "--bench", "pattern_set", "--", "--noplot", "--save-baseline", baseline]
        if args.quick:
            command.append("--quick")
        if args.filter:
            command.append(args.filter)
        print(f"Timing run {number}/{args.runs}; log: {directory / f'run-{number}.log'}", flush=True)
        started = time.time()
        run(command, directory / f"run-{number}.log")
        if source_hash() != fingerprint:
            raise RuntimeError("Source changed during timing; results are not accepted.")
        experiment = dict(metadata, command=command, started=started, elapsed=time.time() - started, records=collect(baseline))
        path.write_text(json.dumps(experiment) + "\n")
        runs.append(experiment)
    memory_path = directory / "memory.json"
    memory_document = json.loads(memory_path.read_text()) if memory_path.exists() else {}
    memory = memory_document.get("cases", {})
    if not args.analyze_only and not (args.resume and memory_document.get("source_sha256") == fingerprint):
        print("Measuring heap in isolated processes.", flush=True)
        run(["cargo", "build", "--locked", "--release", "--example", "pattern_set_memory"], directory / "memory-build.log")
        memory = {}
        for family in FAMILIES:
            for count in COUNTS:
                for length in LENGTHS:
                    for density in DENSITIES:
                        for mode in ("ids", "captures"):
                            engines = ("rexile", "regex_set") if mode == "ids" else ("rexile", "regex_vec", "regex_set_vec")
                            case = f"{family}/{count}/{length}/{density}/{mode}"
                            memory[case] = {}
                            for engine in engines:
                                command = [str(ROOT / "target/release/examples/pattern_set_memory"), engine, family, str(count), str(length), density, mode]
                                memory[case][engine] = json.loads(subprocess.check_output(command, cwd=ROOT, text=True))
        memory_path.write_text(json.dumps({"source_sha256": fingerprint, "cases": memory}) + "\n")
    if memory_document and args.analyze_only and memory_document.get("source_sha256") != fingerprint:
        memory = {}
    verdict = analyze(runs, memory, fingerprint)
    write_report(directory, verdict)
    print(directory / "report.md", flush=True)
    return 0 if verdict["passed"] else 2


if __name__ == "__main__":
    sys.exit(main())
