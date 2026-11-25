"""Reject an empty or incomplete llvm-cov report before accepting its percentage."""

import json
from pathlib import Path
import sys


def check_report(report):
    files = [file for data in report["data"] for file in data["files"]]
    core_files = [file for file in files if "/crates/drape-core/src/" in
                  file["filename"].replace("\\", "/")]
    sources = list((Path(__file__).resolve().parents[2] / "crates/drape-core/src").rglob("*.rs"))
    reported = {Path(file["filename"]).resolve() for file in core_files}
    missing = [str(source) for source in sources if source.resolve() not in reported]
    if not sources or missing:
        raise ValueError(f"Core source files absent from coverage: {missing}")
    # Reject malformed counters instead of accepting an impossible percentage.
    for file in core_files:
        lines = file["summary"]["lines"]
        count, covered = lines["count"], lines["covered"]
        if (type(count) is not int or type(covered) is not int or
                count < 0 or covered < 0 or covered > count):
            raise ValueError(f"Invalid Core coverage counters: {lines}")
    count = sum(file["summary"]["lines"]["count"] for file in core_files)
    covered = sum(file["summary"]["lines"]["covered"] for file in core_files)
    if count <= 0 or covered / count < 0.8:
        raise ValueError(f"Core coverage below 80% or empty: {covered}/{count}")
    print(f"Core source coverage: {covered}/{count} lines ({100 * covered / count:.2f}%)")


if __name__ == "__main__":
    check_report(json.loads(Path(sys.argv[1]).read_text(encoding="utf-8")))
