# Q36 — actual TUN boundary diagnostics and observer effects

<!-- normative-sync: audit-q36-tun-stage-v1 -->

13 October 2026. Six separate 8-second raw TCP duplex phases ran on .10:
downlink 100 Mbps and unlimited, control before observation, observation and
control after. One immutable factory CLI `52511ceb`, one session and unchanged
INI were used; Qeli was not rebuilt. strace 6.13 observed two exact owned
fixture TIDs for 0.2 seconds each, without BPF or follow-forks.
[Actual results](../../../release/certification/evidence/q36-tun-stage-20261013.json).

All six phases, paired PCAPs and archiving completed. Full file inventory,
binary/helper/config SHA and exact-byte/multiplicity correlation passed.
Both tracers detached; endpoint PID/start ticks, production, the full host
snapshot and original defaults were preserved. Owned root/netns/archive were
removed. The first run failed before measurement because client INI was sent
to server `check-config`; its documented `--client` flag was added and the
original failure retained.

## Comparable 150 ms windows

The medians below use only ACKs captured by the server within the selected
window with unique exact client-capture counterparts. Additional syscall
candidates outside the capture window are excluded from this comparison.

| Downlink | Control before, ms | Observed, ms | Control after, ms | Observed / maximum control |
|---|---:|---:|---:|---:|
| 100 Mbps | 0.401259 | 13.325437 | 1.155161 | 11.5356 |
| Unlimited | 21.403810 | 123.972554 | 15.420216 | 5.7921 |

Observed windows differ substantially from both controls. This series cannot
explain the original delay or production capacity. That is an observation
limitation, not an established cause of the original asymmetry. Whole-phase
eight-second throughput does not replace the short traced-window check.

The server profile has two actual readers, q0/q1, with default `tun.queues=0`
(auto). The selected q0 trace contains only an unfinished read at detach and
no successful read. Client traces contain successful ACK writes, but no packet
has a unique match at all four boundaries. Assignment of a specific ACK to q1
was not directly measured. Missing reads in this trace do not establish loss.
Selecting another queue would not resolve the traced/control window difference;
this series is not repeated to infer the original delay stage.

No clock discontinuity was detected, but absolute runtime clock uncertainty
is not bounded. Successful TUN write does not prove application delivery;
intermediate queues/crypto/socket boundaries are unmeasured. Q36 cause remains
`NOT_ESTABLISHED`. Next is current steady data-plane qualification without
the observer; any later stage method must first justify its overhead and
account for all TUN queues. Full Q36 remains IN_PROGRESS.
`NO_PUSH_DEPLOY_RELEASE`.
