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

## 14 October: small TUN packets only

Execution source 553db95a5ddf; same current CLI 6f617bdcab5a and unchanged product tree. Two new sessions/four eight-second legacy one-carrier windows. Replaced all read/consume records with stock own-qbench0 net_dev_start_xmit and 1..128-byte skb_copy_datagram_iovec on both verified server reader TIDs. Private buffer 8192 KiB per CPU; no BPF/kprobe/strace or product change.

Paced/unlimited downlink yielded 34346/35357 records, zero trace loss: 4.91/15.91 times fewer records than the previous four-event observation, comparing different actual sessions rather than claiming a speedup. All 14558/6090 small-copy events had a latest preceding stock xmit with matching hashed SKB and length. Previously unknown origin contexts were mostly new iperf3 threads; PID/comm and frequent pointer reuse retained. Events contain no IP/TCP header and do not identify a particular ACK. Copy trace fires before user-buffer copying, not after read completion.

For 52 bytes:14475/6089 pairs; driver-entry to copy-entry p50 0.010/0.017 ms, p99 1.210/7.177 ms, max 4.801/8.607 ms. Matched ACK p50 in the same short estimated spans was 0.922168/18.530356 ms; controls at corresponding relative offsets were 0.777087/11.157324 ms. Equal counts of 6089 copy and matched ACK records in the unlimited span alone do not establish individual packet identity. PCAP span uses capture-ready clock offset estimate without an absolute clock-error bound. Client records outside the selected server span are expected unmatched destinations, not loss counts.

Whole-phase observed/control uplink:346.90/299.33 Mbps with paced downlink,99.14/175.02 with unlimited downlink. Short-span ACK medians differ by +18.67/+66.08%; one control cannot separate observer overhead from inter-session variation. Lower event volume did not establish observation neutrality. Original F023 cause remains unestablished; no queue/default change justified. Do not repeat another stock-trace variant merely to close the checklist: a next step must test a concrete causal hypothesis with comparable controls.

Workload/ACK/trace qualification, actual overlap, archive SHA/inventory and complete own process/namespace/root/archive/trace-instance cleanup passed. Host/firewall/services/staged CLI and global trace controls preserved. Kernel boundaries measured only in this observer-present pilot; other queues/crypto/socket/client delivery remain unmeasured. Q36 IN_PROGRESS;33/37 (89.2%), NO_PUSH_DEPLOY_RELEASE.

Source context: [Linux6.12 tun_put_user](https://raw.githubusercontent.com/torvalds/linux/refs/tags/v6.12/drivers/net/tun.c), [skb_copy_datagram_iter](https://raw.githubusercontent.com/torvalds/linux/v6.12/net/core/datagram.c), [stock event fields](https://raw.githubusercontent.com/torvalds/linux/v6.12/include/trace/events/skb.h). Primary upstream sources are context, not byte equivalence evidence for the running Debian module.

Evidence: `release/certification/evidence/q36-stock-small-copy-pilot-20261014.json`.

## 14 October: causal TCP socket-buffer tests without kernel tracing

Execution source 989c99d60554, same CLI6f617bdcab5a, unchanged product tree8c50631e7c253ab6cacd0056098ecad83f393487. Two separate six-session/twelve-window series: tcp-plain-raw, legacy one-carrier, unlimited uplink plus separate100M/unlimited downlink, P1/eight seconds. Condition/rate ordering reversed in repeat2. No kernel trace/BPF/kprobe/strace; identical ACK-only tcpdump, process sampler and ss in both conditions. Not a144-window matrix rerun or production-capacity claim.

First hypothesis: reducing server send capacity may prevent ACK accumulation after server TUN. Compared **explicit send=0 (kernel-auto)** with send=16384, receive262144 unchanged. Internal case label `default` means explicit0, not an absent INI key: stock Qeli default is262144. This is not the historical stock-default baseline. All steady snapshots confirm small tb32768 and control tb>32768. Client INI templates match except per-session pin; server templates differ by exactly one key.

| Repeat | Downlink | Uplink control / changed, Mbps | ACK p50 control / changed, ms |
|---|---|---|---|
| 1 | 100M | 252.62 / 265.06 | 0.818935 / 1.095095 |
| 1 | unlimited | 124.35 / 116.68 | 12.078993 / 30.485677 |
| 2 | 100M | 215.45 / 277.33 | 1.022938 / 0.999153 |
| 2 | unlimited | 207.75 / 144.98 | 9.931232 / 31.834243 |
| 3 | 100M | 295.20 / 252.41 | 0.860319 / 1.097533 |
| 3 | unlimited | 167.54 / 116.96 | 9.976525 / 37.260997 |

Unlimited tiny-send ACK p50 worsened in all three pairs: 2.524–3.735×, paired median3.205×; uplink ratio0.698–0.938, median0.698. Reject tiny SO_SNDBUF as a universal workaround in this topology. This does not eliminate kernel-send-queue latency: restricting a socket can move backpressure to userspace. Defaults unchanged.

Second hypothesis: fixed stock receive262144 limits receive-window/memory growth under duplex load. Compared **stock receive262144** with receive0, explicitly holding send262144 in both conditions. Stock rb524288 and held tb524288 verified; auto rb was1272344–2838678bytes and exceeded fixed capacity in every window. Sparse-INI parser baseline includes an explicit nested tcp object, applying per-field defaults; derived Default alone is not that parser baseline.

| Repeat | Downlink | Uplink control / changed, Mbps | ACK p50 control / changed, ms |
|---|---|---|---|
| 1 | 100M | 317.60 / 445.26 | 0.841706 / 0.864485 |
| 1 | unlimited | 130.02 / 153.65 | 19.583172 / 15.429859 |
| 2 | 100M | 245.72 / 427.21 | 0.868478 / 0.884169 |
| 2 | unlimited | 150.18 / 117.20 | 11.769458 / 18.860308 |
| 3 | 100M | 306.22 / 480.69 | 1.184153 / 0.796215 |
| 3 | unlimited | 149.19 / 142.70 | 13.145937 / 14.609256 |

With100M downlink, auto/fixed uplink improved in all3pairs: 1.402–1.739×, median1.570×. Steady socket-drop growth fixed/auto by repeat=[(199, 0), (288, 0), (106, 0)]; this is skmem.d growth within sampled steady spans, not whole-tunnel/whole-phase loss. ACK p50 ratio min/median/max=0.672/1.018/1.027. This establishes a separate fixed receive-capacity uplink constraint for this load, without resolving original unlimited ACK asymmetry.

Unlimited paired uplink auto/fixed min/median/max=0.780/0.957/1.182; ACK p50 auto/fixed=0.788/1.111/1.602. Raw metrics, unmatched headers, client drops and actual socket memory/steady receive-drop growth retained. One mode on a shared two-vCPU lab does not identify every internal stage, qualify an old-CLI fix or justify changing defaults across clients/transports.

Nonzero SO_SNDBUF/SO_RCVBUF disables corresponding kernel autotuning; explicit0 leaves the socket option untouched. RU/EN CONFIG now distinguishes absent keys262144 from explicit0 and server TCP keys from client UDP buffer settings. Primary semantics: [Linux6.12 tcp_rmem/tcp_wmem](https://www.kernel.org/doc/html/v6.12/networking/ip-sysctl.html). No product code/artifact changes.

Both series passed workload settings/roles, identical-header PCAP SHA/timestamps/zero capture loss, topology/source, complete host/firewall/services/staged preservation and owned root/archive cleanup. Both SHA/inventory backups retained, no large compiler caches. Q36 IN_PROGRESS;33/37 (89.2%), NO_PUSH_DEPLOY_RELEASE.

Evidence: `release/certification/evidence/q36-server-sndbuf-causal-20261014.json`, `release/certification/evidence/q36-server-rcvbuf-causal-20261014.json`.
