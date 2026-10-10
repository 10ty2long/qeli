"""Pure measurement admission shared by the lab runner and offline reports."""

import math
import re


def parse_ping_latency(raw, expected_packets):
    """Linux iputils RTT population; lost probes are reported, never imputed."""
    if not isinstance(raw, str) or type(expected_packets) is not int or expected_packets <= 0:
        raise ValueError('invalid requested ping population')
    summaries = re.findall(r'^([0-9]+) packets transmitted, ([0-9]+) received, ([0-9.]+)% packet loss,', raw, re.M)
    if len(summaries) != 1:
        raise ValueError('ping summary missing or ambiguous')
    sent, received, loss = summaries[0]
    sent, received, loss = int(sent), int(received), float(loss)
    if sent != expected_packets or not 0 < received <= sent or not math.isfinite(loss):
        raise ValueError('ping population truncated or invalid')
    actual_loss = 100 * (sent - received) / sent
    # iputils rounds the printed loss to its display precision.
    if abs(actual_loss - loss) > .051:
        raise ValueError('ping loss disagrees with population')
    replies = []
    for line in raw.splitlines():
        if 'icmp_seq=' not in line:
            continue
        match = re.fullmatch(r'\[([0-9]+\.[0-9]+)\] [0-9]+ bytes from .+: icmp_seq=([0-9]+) ttl=[0-9]+ time=([0-9]+(?:\.[0-9]+)?) ms', line)
        if not match:
            raise ValueError('unsupported or censored ping reply')
        epoch, sequence, rtt = float(match[1]), int(match[2]), float(match[3])
        if not math.isfinite(epoch) or not math.isfinite(rtt) or not 1 <= sequence <= sent:
            raise ValueError('invalid ping reply')
        replies.append(dict(epoch=epoch, sequence=sequence, rtt_ms=rtt))
    if len(replies) != received or len({r['sequence'] for r in replies}) != received:
        raise ValueError('ping replies missing or duplicated')
    ordered = sorted(r['rtt_ms'] for r in replies)
    return dict(transmitted=sent, received=received, lost=sent-received,
                lost_percent=actual_loss, percentile_method='nearest-rank',
                p50_ms=ordered[math.ceil(.50*received)-1],
                p95_ms=ordered[math.ceil(.95*received)-1],
                p99_ms=ordered[math.ceil(.99*received)-1],
                min_ms=ordered[0], max_ms=ordered[-1], replies=replies)


def measurement_phases(profile, transport):
    if transport not in ('direct', 'tcp', 'udp'):
        raise ValueError('unsupported measurement transport')
    if profile == 'tcp-bidir':
        return ('tcp_bidir',)
    if profile != 'standard' or transport not in ('direct', 'tcp', 'udp'):
        raise ValueError('unsupported measurement profile or transport')
    if transport == 'direct':
        return ('tcp_up', 'tcp_down', 'udp_500')
    return ('tcp_up', 'tcp_down', 'udp_100', 'udp_500') if transport == 'udp' else ('tcp_up', 'tcp_down')


def bidir_counter_deltas(before, after):
    result = counter_deltas(before, after)
    client = counter_deltas(dict(application=before['application'], kernel=before['client_kernel']),
                            dict(application=after['application'], kernel=after['client_kernel']))
    return dict(server_session_drops=result['server_session_drops'],
                server_kernel_rcvbuf_drops=result['kernel_rcvbuf_drops'],
                client_kernel_rcvbuf_drops=client['kernel_rcvbuf_drops'])


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


def parse_iperf_bidir(raw, expected_seconds, parallel, *, expected_bitrate=0):
    """Both independent receiver summaries are required for a TCP bidir phase."""
    if not isinstance(raw, dict) or 'error' in raw:
        raise ValueError('iperf error or invalid JSON result')
    start = raw.get('start')
    if not isinstance(start, dict) or not isinstance(start.get('test_start'), dict):
        raise ValueError('iperf bidir start settings missing')
    if type(expected_bitrate) is not int or expected_bitrate < 0:
        raise ValueError('invalid requested bidir bitrate')
    # iperf3 reports this twice. A paced diagnostic must never silently enter
    # the ordinary unlimited-throughput population (nor the reverse).
    for settings in (start, start['test_start']):
        if type(settings.get('target_bitrate')) is not int or settings['target_bitrate'] != expected_bitrate:
            raise ValueError('iperf bidir bitrate disagrees with requested phase')
    start = start['test_start']
    if (start.get('protocol') != 'TCP' or type(start.get('bidir')) is not int or start['bidir'] != 1
            or start.get('reverse') != 0 or type(start.get('num_streams')) is not int
            or start['num_streams'] != parallel or start.get('duration') != expected_seconds):
        raise ValueError('iperf bidir settings disagree with requested phase')
    end = raw.get('end')
    if not isinstance(end, dict):
        raise ValueError('iperf bidir receiver summary missing')
    metrics = {}
    for direction, suffix, sender in (('up', '', True), ('down', '_bidir_reverse', False)):
        received, sent = end.get('sum_received' + suffix), end.get('sum_sent' + suffix)
        if not isinstance(received, dict) or not isinstance(sent, dict) or received.get('sender') is not sender or sent.get('sender') is not sender:
            raise ValueError('iperf bidir direction or receiver summary missing')
        # Reuse the same admission rules for goodput, duration and retransmits.
        result = parse_iperf_result(dict(end=dict(sum_received=received, sum_sent=sent)), 'tcp', expected_seconds)
        if result['mbps'] <= 0 or received['bytes'] <= 0:
            raise ValueError('iperf bidir receiver made no progress')
        metrics.update({direction + '_' + key: result[key] for key in ('mbps', 'retransmits')})
    return metrics
