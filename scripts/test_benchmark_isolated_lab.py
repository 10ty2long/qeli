"""Fail-closed checks for iperf results admitted by the isolated benchmark."""

import copy
import math
import os
import sys
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from benchmark_isolated_lab import parse_iperf_result


def result(protocol='tcp'):
    received = dict(seconds=8.01 if protocol == 'tcp' else 5.01,
                    bytes=8_000_000, bits_per_second=8_000_000)
    if protocol == 'udp':
        received.update(lost_percent=2.5, jitter_ms=0.4)
    return dict(end=dict(sum_received=received, sum_sent=dict(retransmits=3),
                         sum=dict(received)))


class IperfAdmissionTests(unittest.TestCase):
    def test_tcp_and_udp_receiver_metrics(self):
        self.assertEqual(parse_iperf_result(result(), 'tcp', 8),
                         dict(mbps=8.0, retransmits=3, lost_percent=None, jitter_ms=None))
        self.assertEqual(parse_iperf_result(result('udp'), 'udp', 5),
                         dict(mbps=8.0, retransmits=None, lost_percent=2.5, jitter_ms=0.4))

    def test_sender_only_summary_cannot_replace_missing_receiver(self):
        raw = result()
        del raw['end']['sum_received']
        with self.assertRaisesRegex(ValueError, 'receiver summary missing'):
            parse_iperf_result(raw, 'tcp', 8)

    def test_truncated_receiver_interval_cannot_count_as_complete(self):
        raw = result()
        raw['end']['sum_received']['seconds'] = 1
        with self.assertRaisesRegex(ValueError, 'interval truncated'):
            parse_iperf_result(raw, 'tcp', 8)

    def test_invalid_metrics_cannot_count_as_complete(self):
        mutations = (
            ('tcp', ('sum_received', 'bits_per_second'), math.nan),
            ('tcp', ('sum_received', 'bytes'), -1),
            ('tcp', ('sum_sent', 'retransmits'), None),
            ('udp', ('sum_received', 'lost_percent'), 101),
            ('udp', ('sum_received', 'jitter_ms'), math.inf),
        )
        for protocol, (section, field), value in mutations:
            with self.subTest(protocol=protocol, field=field, value=value):
                raw = copy.deepcopy(result(protocol))
                raw['end'][section][field] = value
                with self.assertRaises(ValueError):
                    parse_iperf_result(raw, protocol, 8 if protocol == 'tcp' else 5)

    def test_iperf_error_is_rejected_even_with_summary(self):
        raw = result()
        raw['error'] = 'server terminated'
        with self.assertRaisesRegex(ValueError, 'iperf error'):
            parse_iperf_result(raw, 'tcp', 8)


if __name__ == '__main__':
    unittest.main()
