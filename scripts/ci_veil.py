#!/usr/bin/env python3
"""Scan tracked files, verify reviewed exceptions, and emit a snippet-free report."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--veil-bin", default="veil")
    parser.add_argument("--report", required=True, type=Path)
    parser.add_argument("--summary", type=Path)
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    report_path = args.report.resolve()
    if report_path.is_relative_to(root):
        raise ValueError("Report must be outside the repository")
    if args.summary and args.summary.resolve().is_relative_to(root):
        raise ValueError("Summary must be outside the repository")

    tracked = subprocess.check_output(["git", "ls-files", "-z"], cwd=root)
    files = [os.fsdecode(path) for path in tracked.split(b"\0") if path]
    if not files:
        raise ValueError("No tracked files; refusing an empty scan")

    baseline_path = root / "veil.baseline.json"
    baseline = json.loads(baseline_path.read_text())
    if baseline["schema"] != "veil.baseline.v1":
        raise ValueError("Unsupported baseline schema")
    for entry in baseline["entries"]:
        path = entry["path"]
        source = (root / path).resolve()
        if path not in files or not source.is_relative_to(root):
            raise ValueError(f"Baseline source is outside tracked scope: {path}")
        lines = source.read_bytes().splitlines()
        line = entry["line"]
        if not 1 <= line <= len(lines):
            raise ValueError(f"Baseline line moved; review {path}:{line}")
        digest = hashlib.sha256(lines[line - 1]).hexdigest()
        if digest != entry["source_line_sha256"] or not entry["reason"].strip():
            raise ValueError(f"Baseline source changed; review {path}:{line}")

    # Explicit file arguments include tracked dotfiles and ignored-but-tracked files.
    # Keep one invocation so limits and the exit code apply to the complete scan.
    result = subprocess.run(
        [
            args.veil_bin, "--no-color", "scan", "--format", "json",
            "--mask-mode", "redact", "--fail-on-severity", "High",
            "--limit", "2000", "--baseline", str(baseline_path), "--", *files,
        ],
        cwd=root,
        stdout=subprocess.PIPE,
        check=False,
    )
    if result.returncode not in (0, 1, 2):
        raise ValueError(f"Unexpected Veil exit code: {result.returncode}")
    if not result.stdout:
        return 2
    raw = json.loads(result.stdout)
    if raw["schemaVersion"] != "veil-v1":
        raise ValueError("Unsupported report schema")
    summary = raw["summary"]
    # Do not persist snippets or matched values, even when Veil has masked them.
    report = {
        "schemaVersion": raw["schemaVersion"],
        "scope": "git-tracked working tree; explicit file paths; no Git history",
        "exit_code": result.returncode,
        "summary": {key: summary[key] for key in (
            "total_files", "scanned_files", "skipped_files", "total_findings",
            "new_findings", "baseline_suppressed", "limit_reached",
            "file_limit_reached", "max_file_size_reached", "read_error_reached",
            "duration_ms", "severity_counts",
        )},
        "findings": [
            {key: finding[key] for key in (
                "path", "line_number", "rule_id", "severity", "score",
            )}
            for finding in raw["findings"]
        ],
    }
    report_path.parent.mkdir(parents=True, exist_ok=True)
    report_path.write_text(json.dumps(report, ensure_ascii=False, indent=2) + "\n")

    lines = [
        "## Veil scan", "",
        f"Exit code: **{result.returncode}** (0: threshold not met; 1: score >= 70; 2: error/incomplete)",
        "", "| Metric | Count |", "| --- | ---: |",
    ]
    lines.extend(f"| {key} | {summary[key]} |" for key in (
        "total_files", "scanned_files", "skipped_files", "total_findings",
        "new_findings", "baseline_suppressed",
    ))
    lines.extend(["", "New findings by severity: " + json.dumps(summary["severity_counts"], sort_keys=True), ""])
    incomplete = [key for key in (
        "limit_reached", "file_limit_reached", "max_file_size_reached", "read_error_reached",
    ) if summary[key]]
    lines.append("Incomplete flags: " + (", ".join(incomplete) if incomplete else "none"))
    lines.extend(["", "Report: `veil-report.json` (locations/rules only; no source snippets).", ""])
    text = "\n".join(lines)
    print(text)
    if args.summary:
        with args.summary.open("a") as destination:
            destination.write(text)
    # Preserve Veil's error/incomplete status as well as its finding status.
    return result.returncode


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError) as error:
        print(f"Veil CI error: {error}", file=sys.stderr)
        sys.exit(2)
