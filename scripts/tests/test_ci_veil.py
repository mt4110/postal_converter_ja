"""Regression tests for the report boundary and reviewed baseline guards."""

import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest


class VeilRunnerTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name) / "repo"
        (self.root / "scripts").mkdir(parents=True)
        shutil.copyfile(Path(__file__).resolve().parents[1] / "ci_veil.py", self.root / "scripts/ci_veil.py")
        self.git("init", "-q")
        self.source = self.root / ".env.example"
        self.source.write_text("reviewed-public-example\n")
        self.baseline = {
            "schema": "veil.baseline.v1",
            "entries": [{
                "path": ".env.example", "line": 1,
                "source_line_sha256": hashlib.sha256(b"reviewed-public-example").hexdigest(),
                "reason": "Public example reviewed for this test.",
            }],
        }
        self.write_baseline()
        self.git("add", ".")
        self.report = self.root.parent / "report.json"
        self.summary = {
            "total_files": 3, "scanned_files": 3, "skipped_files": 0,
            "total_findings": 1, "new_findings": 1, "baseline_suppressed": 0,
            "limit_reached": False, "file_limit_reached": False,
            "max_file_size_reached": False, "read_error_reached": False,
            "duration_ms": 1, "severity_counts": {"High": 1},
        }
        self.cli = self.root.parent / "fake-veil"

    def git(self, *args):
        return subprocess.run(["git", *args], cwd=self.root, check=True, capture_output=True)

    def write_baseline(self):
        (self.root / "veil.baseline.json").write_text(json.dumps(self.baseline))

    def fake_cli(self, code):
        payload = json.dumps({
            "schemaVersion": "veil-v1", "summary": self.summary,
            "findings": [{
                "path": ".env.example", "line_number": 1,
                "rule_id": "test.synthetic", "severity": "High", "score": 90,
                "matched_content": "SYNTHETIC_PRIVATE_VALUE",
                "line_content": "SYNTHETIC_PRIVATE_VALUE",
                "masked_snippet": "SYNTHETIC_PRIVATE_VALUE",
                "context_before": ["SYNTHETIC_PRIVATE_VALUE"],
            }],
        })
        self.cli.write_text(
            f"#!{sys.executable}\nimport sys\n"
            "assert '.env.example' in sys.argv\n"
            "assert '--mask-mode' in sys.argv and 'redact' in sys.argv\n"
            "assert '--fail-on-severity' in sys.argv and 'High' in sys.argv\n"
            f"print({payload!r})\nsys.exit({code})\n"
        )
        self.cli.chmod(0o700)

    def run_scan(self, report=None):
        env = {key: value for key, value in os.environ.items() if key not in (
            "GIT_INDEX_FILE", "GIT_DIR", "GIT_WORK_TREE",
        )}
        return subprocess.run(
            [sys.executable, str(self.root / "scripts/ci_veil.py"),
             "--veil-bin", str(self.cli), "--report", str(report or self.report)],
            env=env, capture_output=True, text=True,
        )

    def test_preserves_success_finding_and_incomplete_exit_codes(self):
        for code in (0, 1, 2):
            with self.subTest(code=code):
                self.fake_cli(code)
                self.assertEqual(self.run_scan().returncode, code)
                self.assertEqual(json.loads(self.report.read_text())["exit_code"], code)

    def test_report_and_log_do_not_contain_source_snippets(self):
        self.fake_cli(1)
        result = self.run_scan()
        self.assertEqual(result.returncode, 1)
        self.assertNotIn("SYNTHETIC_PRIVATE_VALUE", self.report.read_text() + result.stdout + result.stderr)
        finding = json.loads(self.report.read_text())["findings"][0]
        self.assertEqual(set(finding), {"path", "line_number", "rule_id", "severity", "score"})

    def test_changed_reviewed_source_is_error(self):
        self.fake_cli(0)
        self.source.write_text("different-value-in-the-same-position\n")
        result = self.run_scan()
        self.assertEqual(result.returncode, 2)
        self.assertIn("Baseline source changed", result.stderr)
        self.assertFalse(self.report.exists())

    def test_moved_reviewed_source_is_error(self):
        self.fake_cli(0)
        self.source.write_text("")
        self.assertEqual(self.run_scan().returncode, 2)

    def test_untracked_baseline_source_is_error(self):
        self.fake_cli(0)
        self.baseline["entries"][0]["path"] = "untracked.txt"
        self.write_baseline()
        self.assertEqual(self.run_scan().returncode, 2)

    def test_report_inside_checkout_is_rejected(self):
        self.fake_cli(0)
        self.assertEqual(self.run_scan(self.root / "report.json").returncode, 2)

    def test_missing_cli_is_error(self):
        self.assertEqual(self.run_scan().returncode, 2)


if __name__ == "__main__":
    unittest.main()
