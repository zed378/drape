"""Negative cases for the coverage gate, independent of percentage formatting."""
import importlib.util
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("coverage_gate", Path(__file__).with_name("check-core-coverage.py"))
gate = importlib.util.module_from_spec(spec)
spec.loader.exec_module(gate)


class CoreCoverageGate(unittest.TestCase):
    def report(self, count, covered):
        sources = (Path(__file__).resolve().parents[2] / "crates/drape-core/src").rglob("*.rs")
        return {"data": [{"files": [
            {"filename": str(source), "summary": {"lines": {"count": count, "covered": covered}}}
            for source in sources
        ]}]}

    def test_accepts_exactly_eighty_percent(self):
        gate.check_report(self.report(10, 8))

    def test_rejects_below_eighty_percent(self):
        with self.assertRaises(ValueError):
            gate.check_report(self.report(10, 7))

    def test_rejects_zero_executable_lines(self):
        with self.assertRaises(ValueError):
            gate.check_report(self.report(0, 0))

    def test_rejects_impossible_coverage_counters(self):
        for count, covered in [(10, 11), (10, -1), (-10, -8), (True, 1)]:
            with self.subTest(count=count, covered=covered):
                with self.assertRaises(ValueError):
                    gate.check_report(self.report(count, covered))

    def test_rejects_missing_core_files(self):
        with self.assertRaises(ValueError):
            gate.check_report({"data": [{"files": []}]})


if __name__ == "__main__":
    unittest.main(verbosity=2)
