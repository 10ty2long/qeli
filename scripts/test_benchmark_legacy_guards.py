"""Historical host-mutating benchmarks must refuse an ordinary invocation."""

import os
from pathlib import Path
import subprocess
import sys
import unittest


SCRIPTS = Path(__file__).resolve().parent


class LegacyBenchmarkGuards(unittest.TestCase):
    def test_host_mutating_runners_refuse_before_credentials_or_network(self):
        environment = {key: value for key, value in os.environ.items()
                       if not key.startswith('QELI_')}
        for name in ('benchmark.py', 'bench_bonding.py'):
            with self.subTest(script=name):
                result = subprocess.run([sys.executable, '-B', str(SCRIPTS / name)],
                                        cwd=SCRIPTS.parent, env=environment,
                                        capture_output=True, text=True, timeout=10)
                self.assertEqual(result.returncode, 2, result.stderr)
                self.assertIn('--allow-host-wide-benchmark', result.stderr)
                self.assertNotIn('Traceback', result.stderr)


if __name__ == '__main__':
    unittest.main()
