"""Exercise real rustfmt/Clippy failures in an isolated dependency-free crate."""

import os
from pathlib import Path
import subprocess
import tempfile
import unittest


class QualityGateMutations(unittest.TestCase):
    def setUp(self):
        # Keep temporary build artefacts inside the workspace on restricted hosts.
        root = Path(__file__).resolve().parents[2]
        self.directory = tempfile.TemporaryDirectory(prefix=".tmp-rust-gates-", dir=root)
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        (self.root / "src").mkdir()
        (self.root / "Cargo.toml").write_text(
            '[package]\nname = "gate-probe"\nversion = "0.0.0"\nedition = "2024"\n'
            '[workspace]\n', encoding="utf-8"
        )
        self.environment = {
            **os.environ,
            "CARGO_TARGET_DIR": str(self.root / "target"),
            # These probes have no dependencies; avoid unrelated install/cache locks.
            "CARGO_HOME": str(self.root / "cargo-home"),
        }

    def run_cargo(self, *arguments):
        result = subprocess.run(
            ["cargo", *arguments], cwd=self.root, env=self.environment,
            capture_output=True, text=True, encoding="utf-8", timeout=120,
        )
        return result.returncode, result.stdout + result.stderr

    def write_source(self, source):
        (self.root / "src/lib.rs").write_text(source, encoding="utf-8")

    def test_rustfmt_accepts_clean_source_and_rejects_format_mutation(self):
        self.write_source("pub fn value() -> u8 {\n    1\n}\n")
        code, output = self.run_cargo("fmt", "--all", "--", "--check")
        self.assertEqual(code, 0, output)
        self.write_source("pub fn value()->u8{1}\n")
        code, output = self.run_cargo("fmt", "--all", "--", "--check")
        self.assertNotEqual(code, 0, output)
        self.assertIn("Diff", output)

    def test_clippy_accepts_clean_source_and_denies_warning_mutation(self):
        self.write_source("pub fn value() -> u8 {\n    1\n}\n")
        arguments = ("clippy", "--offline", "--all-targets", "--", "-D", "warnings")
        code, output = self.run_cargo(*arguments)
        self.assertEqual(code, 0, output)
        self.write_source("pub fn value() -> u8 {\n    let unused_probe = 1;\n    1\n}\n")
        code, output = self.run_cargo(*arguments)
        self.assertNotEqual(code, 0, output)
        self.assertIn("unused_probe", output)
        self.assertIn("unused-variables", output)


if __name__ == "__main__":
    unittest.main(verbosity=2)
