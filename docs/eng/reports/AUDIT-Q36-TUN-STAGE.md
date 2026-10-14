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

## 14 October: current CLI stock TUN read events

Execution source 66cf9a2d5d40; current CLI SHA `6f617bdcab5aecf3068ffa9bd6a11006aecbfead09cbb657d60671f0340b7363`, actual build e0ce217b630d9dfd7db5f59936bb41e6d68eb69d; unchanged product tree `8c50631e7c253ab6cacd0056098ecad83f393487`. Two fresh sessions/four eight-second windows: unlimited uplink with a separate 100 Mbps or unlimited downlink, legacy one-carrier on both peers. One observed session and one control; no three-pair statistical or full-matrix replacement claim.

Used stock sys_enter_read/sys_exit_read, small own-qbench0 net_dev_start_xmit and tun_do_read consume_skb in a private trace instance. Verified both server reader TIDs and real TUN descriptors against namespace/inode, /proc and executable SHA. No BPF/kprobe/strace. The two mid-transfer trace sections contain 168599 and 562547 records with zero trace loss and confirmed overlap with both actual iperf children. Workload roles/settings, PCAP SHA/nanotimestamps, clock continuity and zero tcpdump drops were qualified.

Without stock tracing, uplink was 254.24 Mbps with paced downlink and 126.61 Mbps with unlimited downlink; matched identical ACK end-to-end p50 was 0.958656 versus 14.155557 ms. The observed session produced 316.40/99.75 Mbps and ACK p50 1.011884/22.166426 ms. These are whole-phase metrics, while kernel traces are shorter. One control cannot separate observer overhead from inter-session variation; observation neutrality is not established.

Only hashed SKB/length pairs originating in previously verified server threads were admitted. For 52-byte packets, 5079 paced and 998 unlimited pairs yield TUN queue p50 0.010/0.011 ms and p99 2.515/3.718 ms. Explicit incomplete coverage remains: 12944/3695 successful small reads without an admitted server xmit, not packet-loss counts. Kernel records contain no IP/TCP header; 52 bytes cannot establish identity with a particular PCAP ACK. Active-reader between-read p99 is 0.021/0.008 ms, combining pool acquisition, handoff and scheduling. A long blocking read can represent no packet availability, rather than delivery latency.

The first pilot is retained as invalid stage observation: a fifth collector queued behind four occupied controller threads and enabled tracing after transfer completion. Fixed only the external diagnostic driver (max_workers=5), adding overlap/nonempty gates; no product change. Both packets completed process/namespace cleanup, SHA/inventory archive and owned root/archive removal. Global tracing controls, services and staged CLI preserved. This narrows an observed boundary but does not establish the original F023 stage: incomplete individual ACK correspondence and changed observed/control outcomes remain. Do not repeat this heavy four-event observer to claim the original cause. Q36 IN_PROGRESS; NO_PUSH_DEPLOY_RELEASE.

Evidence: `release/certification/evidence/q36-stock-read-pilot-20261014.json`.
