"""Portable admission checks; mock codesign, never claim actual Apple qualification."""
from pathlib import Path
import plistlib
import subprocess
import tempfile
import unittest
from unittest.mock import patch

import verify_ios_simulator as gate


class SimulatorHostTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.app = Path(self.tmp.name) / "Qeli.app"
        self.app.mkdir()
        self.group = "ru.qeli.app.shared"  # Ad-hoc simulator, no device Team ID.
        self.info = {"CFBundlePackageType": "APPL", "QeliKeychainAccessGroup": self.group}
        self.write_info()

    def write_info(self):
        (self.app / "Info.plist").write_bytes(plistlib.dumps(self.info))

    def check(self, entitlements=None, verify_exit=0, dump_exit=0):
        if entitlements is None:
            entitlements = {"keychain-access-groups": [self.group]}
        dump = plistlib.dumps(entitlements)
        with patch.object(gate.sys, "platform", "darwin"), patch.object(gate.subprocess, "run") as run:
            run.side_effect = [subprocess.CompletedProcess([], verify_exit, b"", b""),
                               subprocess.CompletedProcess([], dump_exit, dump, b"diagnostic")]
            errors = gate.validate_host(self.app)
        return errors, run.call_args_list

    def test_actual_signature_and_dump_both_required(self):
        errors, calls = self.check()
        self.assertEqual(errors, [])
        self.assertEqual(calls[0].args[0], ["codesign", "--verify", "--strict", str(self.app)])
        self.assertEqual(calls[1].args[0], ["codesign", "-d", "--entitlements", ":-", str(self.app)])
        self.assertTrue(all(c.kwargs["timeout"] == 30 for c in calls))

    def test_signature_failure_refuses_before_reading_entitlements(self):
        errors, calls = self.check(verify_exit=1)
        self.assertTrue(errors)
        self.assertEqual(len(calls), 1)

    def test_failed_dump_cannot_admit_plausible_xml(self):
        self.assertTrue(self.check(dump_exit=1)[0])

    def test_missing_foreign_and_scalar_groups_refused(self):
        for entitlements in [{}, {"keychain-access-groups": ["foreign"]},
                             {"keychain-access-groups": self.group},
                             {"keychain-access-groups": [self.group, 1]}]:
            with self.subTest(entitlements=entitlements):
                self.assertTrue(self.check(entitlements)[0])

    def test_invalid_host_metadata_refuses_before_codesign(self):
        for value in ["", "$(QELI_KEYCHAIN_GROUP)", None, 1]:
            with self.subTest(group=value):
                self.info["QeliKeychainAccessGroup"] = value
                if value is None:
                    self.info.pop("QeliKeychainAccessGroup")
                self.write_info()
                errors, calls = self.check()
                self.assertTrue(errors)
                self.assertFalse(calls)

    def test_extension_cannot_be_used_as_the_test_host(self):
        self.info["CFBundlePackageType"] = "XPC!"
        self.write_info()
        errors, calls = self.check()
        self.assertTrue(errors)
        self.assertFalse(calls)

    def test_missing_or_malformed_plist_refused(self):
        path = self.app / "Info.plist"
        path.unlink()
        self.assertTrue(self.check()[0])
        path.write_bytes(b"not a plist")
        self.assertTrue(self.check()[0])
        path.write_bytes(plistlib.dumps(["not a dictionary"]))
        self.assertTrue(self.check()[0])

    def test_codesign_timeout_and_launch_failure_refused(self):
        for error in [subprocess.TimeoutExpired("codesign", 30), FileNotFoundError("codesign")]:
            with patch.object(gate.sys, "platform", "darwin"), patch.object(gate.subprocess, "run", side_effect=error):
                self.assertTrue(gate.validate_host(self.app))

    def test_entitlement_dump_timeout_is_not_signature_success(self):
        with patch.object(gate.sys, "platform", "darwin"), patch.object(gate.subprocess, "run") as run:
            run.side_effect = [subprocess.CompletedProcess([], 0, b"", b""),
                               subprocess.TimeoutExpired("codesign dump", 30)]
            self.assertTrue(gate.validate_host(self.app))
            self.assertEqual(run.call_count, 2)

    def test_empty_or_malformed_signed_entitlements_refused(self):
        for output in [b"", b"<?xml version='1.0'?><plist>broken</plist>", plistlib.dumps([self.group])]:
            with patch.object(gate.sys, "platform", "darwin"), patch.object(gate.subprocess, "run") as run:
                run.side_effect = [subprocess.CompletedProcess([], 0, b"", b""),
                                   subprocess.CompletedProcess([], 0, output, b"")]
                self.assertTrue(gate.validate_host(self.app))

    def test_non_apple_host_cannot_report_signature_pass(self):
        with patch.object(gate.sys, "platform", "linux"), patch.object(gate.subprocess, "run") as run:
            self.assertTrue(gate.validate_host(self.app))
            run.assert_not_called()


if __name__ == "__main__":
    unittest.main()
