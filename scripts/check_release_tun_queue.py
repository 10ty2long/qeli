#!/usr/bin/env python3
"""Check queue-key admission on the actual packaged Linux CLI; never start VPN."""
import argparse
from pathlib import Path
import subprocess
import tempfile

BASE = '[qeli]\nserver = 127.0.0.1:443\nuser = package-check\npass = fixture-only\nbind_static = false\n'


def cases():
    for value in (None, '1', '500', '1000', '32767'):
        yield str(value), BASE + ('' if value is None else f'tun.tx_queue_len = {value}\n'), True
    for value in ('0', '-1', '32768', 'auto'):
        yield value, BASE + f'tun.tx_queue_len = {value}\n', False
    yield 'external-explicit', BASE + 'dev_attach = true\ntun.tx_queue_len = 1000\n', False
    yield 'external-owned', BASE + 'dev_attach = true\n', True


def check(binary):
    binary = Path(binary).resolve(strict=True)
    with tempfile.TemporaryDirectory(prefix='qeli-release-queue-') as directory:
        config = Path(directory) / 'client.conf'
        for label, source, accepted in cases():
            config.write_text(source, encoding='utf8')
            result = subprocess.run([str(binary), 'check-config', '--client', '--config', str(config)],
                                    capture_output=True, text=True, timeout=15)
            output = result.stdout + result.stderr
            if (result.returncode == 0) != accepted:
                raise RuntimeError(f'packaged CLI queue case {label}: exit={result.returncode}\n{output}')
            if not accepted and ('tun.tx_queue_len' not in output or 'unknown configuration field' in output):
                raise RuntimeError(f'queue case {label} failed for the wrong reason:\n{output}')
            print(f'PASS packaged CLI queue case {label}')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', required=True, type=Path)
    check(parser.parse_args().binary)
