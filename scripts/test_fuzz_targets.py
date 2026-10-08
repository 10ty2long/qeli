"""Manifest inventory contract and CI consumers; no Rust compilation."""
import contextlib
import io
import json
from pathlib import Path
import tempfile
import tomllib
import unittest
import fuzz_targets as gate


class InventoryTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.manifest = self.root / "Cargo.toml"
        self.write([("beta", "fuzz_targets/beta.rs"), ("alpha", "fuzz_targets/alpha.rs")])

    def write(self, bins):
        self.manifest.write_text('[package]\nname="fixture"\n[package.metadata]\ncargo-fuzz=true\n' +
            ''.join('[[bin]]\nname='+json.dumps(n)+'\npath='+json.dumps(p)+'\n' for n,p in bins), encoding="utf8")
        for _,p in bins:
            path=self.root/p
            if ".." not in Path(p).parts and not Path(p).is_absolute():
                path.parent.mkdir(parents=True, exist_ok=True)
                path.write_text("// fixture\n", encoding="utf8")

    def cli(self, *args):
        out=io.StringIO();err=io.StringIO()
        with contextlib.redirect_stdout(out),contextlib.redirect_stderr(err):
            rc=gate.main(["--manifest",str(self.manifest),*args])
        return rc,out.getvalue(),err.getvalue()

    def test_repository_inventory_contains_every_registered_target(self):
        manifest=gate.ROOT/"qeli/fuzz/Cargo.toml"
        actual=gate.registered_targets(manifest)
        expected=sorted(row['name'] for row in tomllib.loads(manifest.read_text())['bin'])
        self.assertEqual(actual,expected)
        self.assertTrue({'control_v2','quic_envelope','realtls_client','websocket_frame'}<=set(actual))

    def test_new_registered_target_joins_without_another_list(self):
        self.assertEqual(gate.registered_targets(self.manifest),['alpha','beta'])
        self.write([('beta','fuzz_targets/beta.rs'),('gamma','fuzz_targets/gamma.rs'),('alpha','fuzz_targets/alpha.rs')])
        self.assertEqual(gate.registered_targets(self.manifest),['alpha','beta','gamma'])

    def test_stdout_and_output_use_the_same_sorted_targets_and_lf(self):
        output=self.root/'job-output';output.write_bytes(b'previous=value\n')
        rc,out,err=self.cli('--github-output',str(output))
        self.assertEqual((rc,out,err),(0,'alpha\nbeta\n',''))
        self.assertEqual(output.read_bytes(),b'previous=value\ntargets=["alpha","beta"]\n')

    def test_invalid_or_empty_manifests_refuse_without_changing_output(self):
        output=self.root/'job-output';old=b'previous=value\n';output.write_bytes(old)
        for source in ('bad=[', '[package]\nname="not-fuzz"\n', '[package.metadata]\ncargo-fuzz=true\n', '[package]\nmetadata="wrong"\n'):
            with self.subTest(source=source):
                self.manifest.write_text(source);rc,out,err=self.cli('--github-output',str(output))
                self.assertEqual(rc,1);self.assertEqual(out,'');self.assertIn('refused',err);self.assertEqual(output.read_bytes(),old)

    def test_missing_source_refuses_the_whole_inventory(self):
        (self.root/'fuzz_targets/beta.rs').unlink()
        rc,out,err=self.cli();self.assertEqual((rc,out),(1,''));self.assertIn('missing',err)

    def test_duplicate_names_and_output_injection_refused(self):
        for name in ('alpha','-argument','a\nkey=value','a;echo','../alpha'):
            with self.subTest(name=name):
                self.write([('alpha','fuzz_targets/alpha.rs'),(name,'fuzz_targets/other.rs')])
                rc,out,_=self.cli();self.assertEqual((rc,out),(1,''))

    def test_absolute_parent_and_missing_explicit_paths_refused(self):
        for path in (str(self.root/'fuzz_targets/alpha.rs'), '../outside.rs', '', 'bad\\source.rs'):
            with self.subTest(path=path):
                self.manifest.write_text('[package.metadata]\ncargo-fuzz=true\n[[bin]]\nname="alpha"\npath='+json.dumps(path)+'\n')
                rc,out,_=self.cli();self.assertEqual((rc,out),(1,''))

    def test_unwritable_output_is_failure(self):
        rc,out,_=self.cli('--github-output',str(self.root/'missing/outputs'))
        self.assertEqual((rc,out),(1,''))

    def test_matrix_limit_refused_before_emitting_output(self):
        self.write([(f't{i}','fuzz_targets/shared.rs') for i in range(257)])
        rc,out,err=self.cli();self.assertEqual((rc,out),(1,''));self.assertIn('matrix limit',err)


class WorkflowTests(unittest.TestCase):
    def test_smoke_and_nightly_consume_the_shared_manifest_inventory(self):
        text=(gate.ROOT/'.github/workflows/ci.yml').read_text(encoding='utf8')
        smoke=text.split('  fuzz-smoke:',1)[1].split('  fuzz-targets:',1)[0]
        setup=text.split('  fuzz-targets:',1)[1].split('  fuzz-nightly:',1)[0]
        nightly=text.split('  fuzz-nightly:',1)[1].split('  security-audit:',1)[0]
        self.assertIn('inventory="$(python3 ../scripts/fuzz_targets.py)"',smoke)
        self.assertIn('for t in "${targets[@]}"; do',smoke)
        self.assertNotIn('mapfile -t targets < <(',smoke)
        self.assertIn('--github-output "$GITHUB_OUTPUT"',setup)
        self.assertIn('needs: fuzz-targets',nightly)
        self.assertIn('target: ${{ fromJSON(needs.fuzz-targets.outputs.targets) }}',nightly)
        self.assertIn("test_fuzz_targets.py",text)


if __name__ == '__main__':
    unittest.main()
