"""Release admission with real Git refs and controlled CI responses, no publication."""
import contextlib
import io
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch
import release_tag as gate
from native_repro import DEFAULT_RUST_TOOLCHAIN


class TagTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(); self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name); (self.root / "qeli").mkdir()
        (self.root / "qeli/Cargo.toml").write_text('[package]\nname="qeli"\nversion="0.8.2"\n', encoding="utf-8")
        self.git("init", "--quiet"); self.git("add", "."); self.git("commit", "--quiet", "-m", "fixture")
        self.head = self.git("rev-parse", "HEAD")
    def git(self, *args):
        return subprocess.check_output(["git", "-c", "core.autocrlf=false", "-c", "user.name=Audit Fixture", "-c", "user.email=audit@localhost", *args], cwd=self.root, text=True).strip()
    def tag(self, annotated=False):
        self.git("tag", *(["-a", "-m", "fixture"] if annotated else []), "v0.8.2")
    def cli(self, extra=()):
        output=io.StringIO()
        with contextlib.redirect_stdout(output), contextlib.redirect_stderr(output):
            rc=gate.main(["--repo", str(self.root), "--tag", "v0.8.2", *extra])
        return rc, output.getvalue()
    def test_lightweight_tag_matches_exact_source_and_pin(self):
        self.tag(); self.assertEqual(gate.admit_tag(self.root,"v0.8.2"),dict(commit=self.head,version="0.8.2",rust_toolchain=DEFAULT_RUST_TOOLCHAIN))
    def test_annotated_tag_peeled_to_source_commit(self):
        self.tag(True); self.assertEqual(gate.admit_tag(self.root,"v0.8.2")["commit"],self.head)
    def test_branch_only_is_not_a_release_tag(self):
        self.git("branch","v0.8.2")
        with self.assertRaises(ValueError):gate.admit_tag(self.root,"v0.8.2")
    def test_same_name_branch_cannot_replace_other_tag_commit(self):
        self.tag();self.git("commit","--quiet","--allow-empty","-m","next");self.git("branch","v0.8.2")
        with self.assertRaisesRegex(ValueError,"checked-out"):gate.admit_tag(self.root,"v0.8.2")
    def test_missing_tag(self):
        with self.assertRaises(ValueError):gate.admit_tag(self.root,"v0.8.2")
    def test_version_mismatch(self):
        self.git("tag","v0.8.3")
        with self.assertRaisesRegex(ValueError,"Cargo version"):gate.admit_tag(self.root,"v0.8.3")
    def test_dirty_tracked_source_refused(self):
        self.tag();(self.root/"qeli/Cargo.toml").write_text('bad')
        with self.assertRaisesRegex(ValueError,"clean"):gate.admit_tag(self.root,"v0.8.2")
    def test_untracked_input_refused(self):
        self.tag();(self.root/"unexpected").write_text('input')
        with self.assertRaisesRegex(ValueError,"clean"):gate.admit_tag(self.root,"v0.8.2")
    def test_unsafe_tag_refused_before_git(self):
        for tag in ['--all','refs/heads/dev','v0.8.2\nvalue=x','v0.8.2;touch x','v0.8.2-01','v0.8.2/child']:
            with self.subTest(tag=tag),patch.object(gate,'git',side_effect=AssertionError('Git called')):
                with self.assertRaises(ValueError):gate.admit_tag(self.root,tag)
    def test_git_trust_flags_cannot_hide_modified_source(self):
        self.tag();self.git("update-index","--assume-unchanged","qeli/Cargo.toml")
        (self.root/"qeli/Cargo.toml").write_text('[package]\nversion="0.8.3"\n')
        with self.assertRaisesRegex(ValueError,"Git flags"):gate.admit_tag(self.root,"v0.8.2")
    def test_skip_worktree_source_refused(self):
        self.tag();self.git("update-index","--skip-worktree","qeli/Cargo.toml")
        with self.assertRaisesRegex(ValueError,"Git flags"):gate.admit_tag(self.root,"v0.8.2")
    def test_ci_failure_creates_no_output(self):
        self.tag();out=self.root.parent/(self.root.name+'-out');self.addCleanup(lambda:out.unlink(missing_ok=True))
        with patch.object(gate,'check_release_ci',return_value=['CI failed']):
            rc,text=self.cli(['--require-ci','--github-output',str(out)])
        self.assertEqual(rc,1);self.assertIn('CI failed',text);self.assertFalse(out.exists())
    def test_exact_commit_ci_selector_and_outputs(self):
        self.tag();out=self.root.parent/(self.root.name+'-out');self.addCleanup(lambda:out.unlink(missing_ok=True))
        with patch.object(gate,'check_release_ci',return_value=[]) as ci:
            rc,text=self.cli(['--require-ci','--github-output',str(out)])
        self.assertEqual(rc,0);ci.assert_called_once_with(self.root,'v0.8.2',by_commit=True)
        self.assertEqual(out.read_bytes(),f'commit={self.head}\nversion=0.8.2\nrust_toolchain={DEFAULT_RUST_TOOLCHAIN}\n'.encode())
    def test_source_change_during_ci_admission_refused(self):
        self.tag()
        def change(*args,**kwargs):self.git('commit','--quiet','--allow-empty','-m','changed');return []
        with patch.object(gate,'check_release_ci',side_effect=change):rc,text=self.cli(['--require-ci'])
        self.assertEqual(rc,1);self.assertIn('checked-out',text)
    def test_missing_git_or_timeout_returns_refusal(self):
        for error in [FileNotFoundError('git'),subprocess.TimeoutExpired('git',30)]:
            with patch.object(gate.subprocess,'run',side_effect=error):rc,text=self.cli()
            self.assertEqual(rc,1)


class WorkflowTests(unittest.TestCase):
    def test_release_and_docker_bind_pin_and_legal_gates(self):
        root=Path(__file__).resolve().parent.parent
        s=(root/'.github/workflows/release-attest.yml').read_text(encoding='utf-8')
        self.assertIn('ref: refs/tags/${{',s);self.assertIn('actions: read',s)
        self.assertLess(s.index('--require-ci'),s.index('Install the portable'))
        self.assertIn('toolchain: ${{ steps.meta.outputs.rust_toolchain }}',s)
        self.assertIn('--bundle "$extracted/usr/share/doc/qeli/licenses/rust"',s)
        s=(root/'.github/workflows/docker-publish.yml').read_text(encoding='utf-8')
        self.assertLess(s.index('python3 scripts/native_licenses.py'),s.index('docker/login-action@'))
        s=(root/'release/docker/Dockerfile').read_text(encoding='utf-8')
        self.assertRegex(s,r'FROM rust:'+DEFAULT_RUST_TOOLCHAIN.replace('.',r'\.')+r'-bookworm@sha256:[0-9a-f]{64} AS build')
        self.assertRegex(s,r'FROM debian:bookworm-slim@sha256:[0-9a-f]{64}')
        self.assertIn('COPY LICENSE LICENSING.md /usr/share/doc/qeli/',s)
        self.assertIn('COPY qeli/third-party-notices/rust /usr/share/doc/qeli/licenses/rust',s)
    def test_build_attestation_covers_only_the_locally_built_package(self):
        source=(Path(__file__).resolve().parent.parent/'.github/workflows/release-attest.yml').read_text(encoding='utf-8')
        actions=[line.strip() for line in source.splitlines() if 'uses: actions/attest' in line]
        self.assertEqual(actions,['uses: actions/attest-build-provenance@96278af6caaf10aea03fd8d33a09a777ca52d62f # v3.2.0'])
        attest=source.split('      - name: Attest the CI-built Linux package',1)[1].split('      - name:',1)[0]
        self.assertIn('subject-path: ${{ steps.assets.outputs.subject }}',attest)
        self.assertNotIn('dist/*',attest)
        self.assertIn('echo "subject=dist/${{ steps.pkg.outputs.name }}"',source)

    def test_docker_release_uses_exact_tag_version_ci_admission_before_login(self):
        source=(Path(__file__).resolve().parent.parent/'.github/workflows/docker-publish.yml').read_text(encoding='utf-8')
        self.assertIn('actions: read',source)
        self.assertIn('ref: ${{ github.ref }}',source)
        self.assertIn('fetch-depth: 0',source)
        self.assertIn("if: steps.ref.outputs.release_tag == 'true'",source)
        command='python3 scripts/release_tag.py --tag "$GITHUB_REF_NAME" --require-ci'
        self.assertLess(source.index('python3 -B scripts/docker_publish_ref.py'),source.index(command))
        self.assertLess(source.index(command),source.index('docker/login-action@'))
        self.assertIn('GH_TOKEN: ${{ github.token }}',source)

    def test_ci_covers_every_workflow_change(self):
        s=(Path(__file__).resolve().parent.parent/'.github/workflows/ci.yml').read_text(encoding='utf-8')
        self.assertEqual(s.count("      - '.github/workflows/**'"),2)
        self.assertIn("'test_release_tag.py'",s)


if __name__=='__main__':unittest.main()
