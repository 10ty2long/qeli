"""Offline discovery must never start the remote panel/DNS mutation scenario."""
import contextlib
import io
from pathlib import Path
import runpy
import unittest
from unittest.mock import patch


class AuditScriptImportTests(unittest.TestCase):
    def test_panel_dns_scenario_import_has_no_network_or_stdout_side_effects(self):
        output = io.StringIO()
        with patch("socket.socket", side_effect=AssertionError("network during import")):
            with contextlib.redirect_stdout(output):
                runpy.run_path(str(Path(__file__).with_name("test_panel_route_dns.py")),
                               run_name="audit_discovery")
        self.assertEqual(output.getvalue(), "")


if __name__ == "__main__":
    unittest.main()
