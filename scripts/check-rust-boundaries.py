"""Inspect Cargo metadata and mutation-check real isolated manifests (Python 3.11+)."""

import json
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
ALLOWED = {
    "drape-core": set(),
    "drape-infrastructure": {"drape-core", "serde_json"},
    "drape-ui": {"drape-core", "slint", "slint-build"},
    "drape-shell-windows": {"drape-core", "windows"},
    "drape-test-support": {"drape-core"},
    "drape-guard": {"drape-core", "drape-shell-windows"},
    "drape-app": {"drape-core", "drape-infrastructure", "drape-shell-windows", "drape-ui"},
}
# Allowed only as development dependencies (ADR-031).
DEV_ONLY = {"drape-infrastructure": {"drape-test-support"}}
# The pinned windows 0.61.3 binding's reviewed transitive closure, not a general
# windows-* allowance. Direct native dependencies remain forbidden in Guard and
# every shared crate. Adding another package requires review of actual metadata.
GUARD_NATIVE_CLOSURE = {
    "windows", "windows-collections", "windows-core", "windows-future",
    "windows-implement", "windows-interface", "windows-link", "windows-numerics",
    "windows-result", "windows-strings", "windows-threading",
    "proc-macro2", "quote", "syn", "unicode-ident",
}


def cargo(root, *args):
    return subprocess.run(["cargo", *args], cwd=root, check=True,
                          capture_output=True, text=True, encoding="utf-8").stdout


def metadata(root):
    return json.loads(cargo(root, "metadata", "--offline", "--locked", "--format-version", "1"))


def violations(graph):
    packages = {p["id"]: p for p in graph["packages"]}
    members = {packages[key]["name"]: packages[key] for key in graph["workspace_members"]}
    failures = []
    if set(members) != set(ALLOWED):
        failures.append("workspace: unexpected or missing members")
    for name, package in members.items():
        # Includes all build/dev/optional/target declarations, even if inactive
        # on this host. Later tasks amend this strict baseline under review.
        for dependency in package["dependencies"]:
            if (dependency["kind"] == "dev"
                    and dependency["name"] in DEV_ONLY.get(name, set())):
                continue
            if dependency["name"] not in ALLOWED.get(name, set()):
                failures.append(f"{name}: forbidden dependency {dependency['name']}")
    nodes = {node["id"]: node for node in graph["resolve"]["nodes"]}
    pending, seen = [members["drape-guard"]["id"]], set()
    while pending:
        key = pending.pop()
        if key not in seen:
            seen.add(key)
            pending.extend(dep["pkg"] for dep in nodes[key]["deps"])
    for key in seen:
        name = packages[key]["name"]
        if name not in ALLOWED["drape-guard"] | {"drape-guard"} | GUARD_NATIVE_CLOSURE:
            failures.append(f"Guard closure: {name}")
    for node in nodes.values():
        name = packages[node["id"]]["name"]
        if name in {"i-slint-backend-qt", "wry", "webview2", "webview2-com"}:
            failures.append(f"production graph: forbidden backend {name}")
    return failures


class CargoBoundaries(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.graph = metadata(ROOT)

    def test_actual_workspace_and_guard_dependency_boundaries(self):
        self.assertEqual(violations(self.graph), [])

    def test_core_and_ui_forbid_unsafe_and_platform_access(self):
        for crate in ("drape-core", "drape-ui"):
            source = ROOT / "crates" / crate / "src"
            handwritten = "lib.rs" if crate == "drape-core" else "presentation.rs"
            self.assertIn("#![forbid(unsafe_code)]", (source / handwritten).read_text())
            if crate == "drape-ui":
                root = (source / "lib.rs").read_text()
                self.assertIn("#![deny(unsafe_code)]", root)
                self.assertEqual(root.count("#[allow(unsafe_code)]"), 1)
                self.assertIn("#[allow(unsafe_code)]\nmod generated {\n    slint::include_modules!();\n}", root)
            for path in source.rglob("*.rs"):
                contents = path.read_text()
                for term in ("std::fs", "std::net", "std::process", "std::env", "std::time", "target_os", "drape_shell", "windows::"):
                    self.assertNotIn(term, contents, f"{path}: {term}")

    def mutation_graph(self, crate, dependency):
        # No hand-edited graph fixtures: obtain fresh Cargo output after a
        # forbidden edge is injected into an isolated copy of the manifests.
        with tempfile.TemporaryDirectory(prefix="drape-boundary-") as directory:
            root = Path(directory)
            for file in ("Cargo.toml", "Cargo.lock", "rust-toolchain.toml"):
                shutil.copy2(ROOT / file, root / file)
            shutil.copytree(ROOT / "crates", root / "crates")
            with (root / "crates" / crate / "Cargo.toml").open("a") as output:
                output.write(dependency)
            cargo(root, "generate-lockfile", "--offline")
            return metadata(root)

    def test_forbidden_core_ui_dependency_mutation_is_rejected(self):
        failures = violations(self.mutation_graph("drape-core",
            '\n[target.\'cfg(windows)\'.dependencies]\nslint = { workspace = true, optional = true }\n'))
        self.assertIn("drape-core: forbidden dependency slint", failures)

    def test_guard_ui_dependency_mutation_is_rejected(self):
        failures = violations(self.mutation_graph("drape-guard",
            '\n[build-dependencies]\nslint-build.workspace = true\n'))
        self.assertIn("drape-guard: forbidden dependency slint-build", failures)
        self.assertIn("Guard closure: slint-build", failures)

    def test_shared_ui_shell_dev_dependency_mutation_is_rejected(self):
        failures = violations(self.mutation_graph("drape-ui",
            '\n[dev-dependencies]\ndrape-shell-windows.workspace = true\n'))
        self.assertIn("drape-ui: forbidden dependency drape-shell-windows", failures)

    def test_infrastructure_shell_build_dependency_mutation_is_rejected(self):
        failures = violations(self.mutation_graph("drape-infrastructure",
            '\n[build-dependencies]\ndrape-shell-windows.workspace = true\n'))
        self.assertIn("drape-infrastructure: forbidden dependency drape-shell-windows", failures)

    def test_fake_adapters_are_rejected_as_a_production_dependency(self):
        failures = violations(self.mutation_graph("drape-infrastructure",
            '\n[target.\'cfg(windows)\'.dependencies]\n'
            'drape-test-support.workspace = true\n'))
        self.assertIn(
            "drape-infrastructure: forbidden dependency drape-test-support", failures)

    def test_native_binding_dependency_is_rejected_in_shared_crates(self):
        for crate in ("drape-core", "drape-ui", "drape-infrastructure", "drape-guard"):
            with self.subTest(crate=crate):
                failures = violations(self.mutation_graph(crate,
                    '\n[target.\'cfg(windows)\'.dependencies]\n'
                    'windows = { version = "=0.61.3", default-features = false }\n'))
                self.assertIn(f"{crate}: forbidden dependency windows", failures)


if __name__ == "__main__":
    unittest.main(verbosity=2)
