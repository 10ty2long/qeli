"""Regression checks for legal resources needed at Docker compile time (issue 200)."""
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parent.parent


class DockerBuildInputTests(unittest.TestCase):
    def test_embedded_licenses_are_copied_before_compilation(self):
        recipe = (ROOT / "release/docker/Dockerfile").read_text(encoding="utf8")
        build = recipe.split(" AS build\n", 1)[1].split("# ---- runtime stage", 1)[0]
        self.assertIn("COPY qeli/third-party-notices/rust ./third-party-notices/rust", build)
        self.assertLess(build.index("COPY qeli/third-party-notices/rust"),
                        build.index("cargo build --locked --release --features jemalloc --bin qeli"))

    def test_runtime_legal_resources_are_still_distributed(self):
        recipe = (ROOT / "release/docker/Dockerfile").read_text(encoding="utf8")
        runtime = recipe.split("# ---- runtime stage", 1)[1]
        self.assertIn("COPY qeli/third-party-notices/rust /usr/share/doc/qeli/licenses/rust", runtime)


if __name__ == "__main__":
    unittest.main()
