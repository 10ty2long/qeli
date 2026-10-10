"""Pure measurement admission shared by the lab runner and offline reports."""

import math


def counter_deltas(before, after):
    for snapshot in (before, after):
        app, kernel = snapshot['application'], snapshot['kernel']
        for key in ('dropped', 'bytes_sent', 'bytes_recv', 'connected_secs'):
            if type(app[key]) is not int or app[key] < 0:
                raise ValueError('invalid application counter')
        if type(kernel['RcvbufErrors']) is not int or kernel['RcvbufErrors'] < 0:
            raise ValueError('invalid kernel drop counter')
        if any(not isinstance(app[key], str) or not app[key] for key in ('ip', 'peer')):
            raise ValueError('benchmark session identity missing')
    if any(before['application'][key] != after['application'][key] for key in ('ip', 'peer')):
        raise ValueError('benchmark session identity changed')
    if any(after['application'][key] < before['application'][key] for key in ('dropped', 'bytes_sent', 'bytes_recv', 'connected_secs')):
        raise ValueError('benchmark session counters reset')
    if after['kernel']['RcvbufErrors'] < before['kernel']['RcvbufErrors']:
        raise ValueError('benchmark kernel counter reset')
    return dict(kernel_rcvbuf_drops=after['kernel']['RcvbufErrors'] - before['kernel']['RcvbufErrors'],
                server_session_drops=after['application']['dropped'] - before['application']['dropped'])


def parse_iperf_result(raw, protocol, expected_seconds):
    """Require a completed receiver measurement, never a sender-only fallback."""
    if not isinstance(raw, dict) or 'error' in raw:
        raise ValueError('iperf error or invalid JSON result')
    end = raw.get('end')
    if not isinstance(end, dict) or not isinstance(end.get('sum_received'), dict):
        raise ValueError('iperf receiver summary missing')
    received = end['sum_received']

    def number(source, key, minimum=0, maximum=None):
        value = source.get(key)
        if type(value) not in (int, float) or not math.isfinite(value) or value < minimum or (maximum is not None and value > maximum):
            raise ValueError('invalid iperf ' + key)
        return value

    if number(received, 'seconds') < expected_seconds * .9:
        raise ValueError('iperf receiver interval truncated')
    number(received, 'bytes')
    mbps = number(received, 'bits_per_second') / 1e6
    if protocol == 'tcp':
        sent = end.get('sum_sent')
        if not isinstance(sent, dict) or type(sent.get('retransmits')) is not int or sent['retransmits'] < 0:
            raise ValueError('iperf TCP retransmits missing or invalid')
        return dict(mbps=mbps, retransmits=sent['retransmits'], lost_percent=None, jitter_ms=None)
    if protocol == 'udp':
        return dict(mbps=mbps, retransmits=None,
                    lost_percent=number(received, 'lost_percent', maximum=100),
                    jitter_ms=number(received, 'jitter_ms'))
    raise ValueError('unsupported iperf protocol')
