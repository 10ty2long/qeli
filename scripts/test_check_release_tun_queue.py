import subprocess
import unittest
from unittest.mock import patch
from check_release_tun_queue import cases, check


class ReleaseQueueTest(unittest.TestCase):
    def test_explicit_1000_and_boundaries_are_in_actual_binary_contract(self):
        rows = {label: (source, accepted) for label, source, accepted in cases()}
        for value in ('1', '500', '1000', '32767'):
            self.assertTrue(rows[value][1])
            self.assertIn(f'tun.tx_queue_len = {value}', rows[value][0])
        for value in ('0', '-1', '32768', 'auto', 'external-explicit'):
            self.assertFalse(rows[value][1])

    @patch('check_release_tun_queue.subprocess.run')
    def test_pre_feature_binary_is_rejected(self, run):
        run.side_effect = [subprocess.CompletedProcess([], 0, 'OK', ''),
                           subprocess.CompletedProcess([], 1, '', 'qeli.tun.tx_queue_len: unknown configuration field')]
        with self.assertRaisesRegex(RuntimeError, 'packaged CLI queue case 1'):
            check(__file__)
        self.assertEqual(run.call_count, 2)


if __name__ == '__main__':
    unittest.main()
