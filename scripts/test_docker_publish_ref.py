"""Exercise publication ref policy and the real workflow contract without pushing."""
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest
from docker_publish_ref import classify

ROOT = Path(__file__).resolve().parent.parent


class RefTests(unittest.TestCase):
    def test_manual_branches_never_publish_latest(self):
        for branch in ["dev", "main", "v1.2.3", "release/1.2", "codex/fix"]:
            self.assertEqual(classify("branch", branch), {"release_tag": "false", "stable_release": "false"})

    def test_stable_tags_allow_latest(self):
        for tag in ["v0.8.2", "v1.2.3", "v10.20.30", "v1.2.3+build-42"]:
            self.assertEqual(classify("tag", tag), {"release_tag": "true", "stable_release": "true"})

    def test_prereleases_never_publish_latest(self):
        for tag in ["v1.2.3-rc1", "v1.2.3-rc.1", "v1.2.3-0", "v1.2.3-rc+build"]:
            self.assertEqual(classify("tag", tag), {"release_tag": "true", "stable_release": "false"})

    def test_invalid_tags_refuse_publication(self):
        for tag in ["vtest", "1.2.3", "v01.2.3", "v1.02.3", "v1.2.03", "v1.2.3-01", "v1.2.3-", "v1.2.3+", "v1.2.3\n", "v1.2.3;echo bad"]:
            with self.assertRaises(ValueError):
                classify("tag", tag)

    def test_unexpected_refs_refuse(self):
        for kind, name in [("pull", "1"), ("", "dev"), ("branch", "")]:
            with self.assertRaises(ValueError):
                classify(kind, name)

    def test_real_cli_appends_outputs_and_refuses_before_writing(self):
        with tempfile.TemporaryDirectory() as temporary:
            output = Path(temporary) / "outputs"
            output.write_text("existing=value\n")
            env = {**os.environ, "GITHUB_REF_TYPE": "branch", "GITHUB_REF_NAME": "dev", "GITHUB_OUTPUT": str(output)}
            p = subprocess.run([sys.executable, "-B", str(ROOT / "scripts/docker_publish_ref.py")], env=env, capture_output=True)
            self.assertEqual(p.returncode, 0)
            self.assertEqual(output.read_text(), "existing=value\nrelease_tag=false\nstable_release=false\n")
            before = output.read_bytes()
            env.update(GITHUB_REF_TYPE="tag", GITHUB_REF_NAME="vmalformed")
            p = subprocess.run([sys.executable, "-B", str(ROOT / "scripts/docker_publish_ref.py")], env=env, capture_output=True)
            self.assertNotEqual(p.returncode, 0)
            self.assertEqual(output.read_bytes(), before)

    def test_workflow_guards_and_branch_fallback(self):
        workflow = (ROOT / ".github/workflows/docker-publish.yml").read_text(encoding="utf8")
        self.assertLess(workflow.index("python3 -B scripts/docker_publish_ref.py"), workflow.index("docker/login-action@"))
        self.assertIn("flavor: latest=false", workflow)
        self.assertIn("type=semver,pattern={{version}},enable=${{ steps.ref.outputs.release_tag }}", workflow)
        self.assertIn("type=sha,format=long", workflow)
        self.assertIn("type=raw,value=latest,enable=${{ steps.ref.outputs.stable_release }}", workflow)
        self.assertNotIn("!contains(github.ref_name, '-')", workflow)


if __name__ == "__main__":
    unittest.main()
