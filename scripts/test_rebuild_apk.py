"""APK builder lifecycle and concurrent-build regression contracts; no lab needed."""
import contextlib
import io
from pathlib import Path
import tempfile
import unittest
from unittest.mock import MagicMock, patch
import subprocess
import sys
import rebuild_apk as apk


class BuilderTests(unittest.TestCase):
    def test_import_does_not_connect_or_require_password(self):
        code = "import native_lab; native_lab.connect_lab=lambda *a: (_ for _ in ()).throw(AssertionError('SSH on import')); import rebuild_apk"
        result = subprocess.run([sys.executable, "-B", "-c", code], cwd=Path(__file__).parent, capture_output=True)
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_invalid_argument_refuses_before_connection(self):
        with patch.object(apk, "conn", side_effect=AssertionError("SSH before admission")), contextlib.redirect_stderr(io.StringIO()):
            with self.assertRaises(SystemExit) as error:
                apk.main(["--invalid"])
        self.assertEqual(error.exception.code, 2)

    def test_default_and_release_variants(self):
        for argv, release in [([], False), (["--release"], True)]:
            connection = MagicMock()
            with patch.object(apk, "conn", return_value=connection), patch.object(apk, "build") as build:
                self.assertEqual(apk.main(argv), 0)
                build.assert_called_once_with(connection, connection.open_sftp.return_value, release)
            connection.open_sftp.return_value.close.assert_called_once()
            connection.close.assert_called_once()

    def test_build_failure_closes_both_handles(self):
        connection = MagicMock()
        with patch.object(apk, "conn", return_value=connection), patch.object(apk, "build", side_effect=RuntimeError("compile failure")):
            with self.assertRaisesRegex(RuntimeError, "compile failure"):
                apk.main([])
        connection.open_sftp.return_value.close.assert_called_once()
        connection.close.assert_called_once()

    def test_sftp_open_failure_closes_connection(self):
        connection = MagicMock()
        connection.open_sftp.side_effect = OSError("SFTP failure")
        with patch.object(apk, "conn", return_value=connection):
            with self.assertRaisesRegex(OSError, "SFTP failure"):
                apk.main([])
        connection.close.assert_called_once()

    def test_sftp_close_failure_still_closes_connection(self):
        connection = MagicMock()
        connection.open_sftp.return_value.close.side_effect = OSError("close failure")
        with patch.object(apk, "conn", return_value=connection), patch.object(apk, "build"):
            with self.assertRaisesRegex(OSError, "close failure"):
                apk.main([])
        connection.close.assert_called_once()

    def test_actual_build_orchestration_does_not_kill_other_builds_or_delete_cache(self):
        for release in [False, True]:
            with tempfile.TemporaryDirectory() as temporary:
                root = Path(temporary)
                app = root / "android"
                for abi in ["arm64-v8a", "x86_64"]:
                    f = app / "app/src/main/jniLibs" / abi / "libqeli.so"
                    f.parent.mkdir(parents=True, exist_ok=True)
                    f.write_bytes(b"fixture native core")
                corpus = root / "conformance"
                corpus.mkdir()
                calls = []
                def command(connection, cmd, t=1200):
                    calls.append(cmd)
                    if "testDebugUnitTest" in cmd:
                        return "fixture compile failure", 7
                    if "nm -D" in cmd:
                        return "1", 0
                    return "", 0
                with patch.object(apk, "LOCAL", str(app)), patch.object(apk, "CONFORMANCE", corpus), patch.object(apk, "sync_qeli_source", return_value=1), patch.object(apk, "sh", side_effect=command), contextlib.redirect_stdout(io.StringIO()):
                    with self.assertRaisesRegex(RuntimeError, "APK build failed"):
                        apk.build(MagicMock(), MagicMock(), release)
                self.assertFalse(any("pkill" in cmd or "journal-1" in cmd for cmd in calls), calls)
                builds = [cmd for cmd in calls if "testDebugUnitTest" in cmd]
                self.assertEqual(len(builds), 1)
                self.assertIn("assembleRelease" if release else "assembleDebug", builds[0])
                self.assertIn("--no-daemon", builds[0])
                self.assertIn("--offline", builds[0])
                self.assertIn("--max-workers=1", builds[0])


if __name__ == "__main__":
    unittest.main()
