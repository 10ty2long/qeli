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

## 14 October: client TUN writer priority

Execution source e07c2597d446, same current CLI6f617bdcab5a/product tree8c50631e7c253ab6cacd0056098ecad83f393487. New6sessions/12windows: three normal nice0 versus nice−10 client qeli-tun-writer pairs; rawTCP legacy one-carrier, P1/eight seconds, separate concurrent iperf flows with100M/unlimited downlink. Reversed condition/rate order in repeat2. Config templates match except per-session public pin; held server requested send/receive262144, actual tb/rb524288 in all steady snapshots. No other socket/queue/default change.

Before setting and before/after every window verified root/namespace cookies, /proc executable SHA/path, PID/TID starttime, unique writer comm and TUN fdinfo. Only writer receives nice−10; other client threads keep0. Ordinary SCHED_OTHER/affinity0-1 and within-session cgroup/autogroup unchanged. Readback fair weight grows only on the target TID; this affects CPU sharing, not a latency guarantee. [Linux per-thread nice semantics](https://man7.org/linux/man-pages/man2/setpriority.2.html).

| Repeat | Downlink | Uplink control / changed, Mbps | ACK p50 control / changed, ms |
|---|---|---|---|
| 1 | 100M | 237.77 / 210.45 | 0.968889 / 0.927177 |
| 1 | unlimited | 105.23 / 151.32 | 17.481170 / 15.946424 |
| 2 | 100M | 288.49 / 270.03 | 0.837279 / 0.871184 |
| 2 | unlimited | 164.18 / 139.85 | 13.142024 / 12.863850 |
| 3 | 100M | 296.41 / 347.55 | 0.867200 / 0.802712 |
| 3 | unlimited | 144.89 / 136.12 | 11.225342 / 14.199052 |

Unlimited paired uplink priority/default min/median/max=0.852/0.939/1.438; ACK p50=0.912/0.979/1.265. No consistent delay removal. Reject priority increase as a sufficient workaround in this topology; it does not eliminate writer queue/wait because CPU sharing with neighboring client workers also changes. Production priority/defaults unchanged.

Schedstat/tick deltas and sched_schedstats flag retained. Their horizon includes collection outside the eight-second transfer; disabled scheduler statistics does not imply zero runnable wait. No kernel trace/BPF/kprobe/strace. Workload/settings, identical-header PCAP SHA/timestamps/zero capture loss, topology/source, full host/firewall/services/staged preservation and SHA/inventory owned-root/archive cleanup passed. Own processes and target TID gone; no unrelated thread priorities changed.

Next: direct diagnostic timestamps at server TUN read → server wire handoff → client decrypt → client TUN write with unique packet fingerprint, finite capture and enabled/disabled observer controls. Matching sizes/counters alone cannot establish packet identity. Read-only preflight confirms installed Rust1.97/local crate cache on .10; no compiler run or compiler/cache/core modification. A diagnostic CLI needs separate source/build qualification and is not a shipping CLI/native artifact. Q36 IN_PROGRESS;33/37 (89.2%), NO_PUSH_DEPLOY_RELEASE.

Evidence: `release/certification/evidence/q36-client-writer-priority-20261014.json`.

## 14 October: direct ACK stage pilot

Separate private Linux CLI built from source 776de2a63efc with seven private source-file changes: SHA256 fingerprints of synthetic bare IPv4 ACK5203, monotonic timestamps and finite nonblocking ring262144. Product repository/native unchanged. Two selector tests compile the identical module in a small crate; its atomic-dump stub does not test file I/O. Actual Qeli atomic dump/SIGUSR1 exercised by the pilot. Original missing-conformance failure and two whole-cfg(test)-library OOM failures remain FAIL, with owned source/cache/root cleanup proved. Diagnostic release ThinLTO/codegen4 differs from shipping FatLTO/codegen1; artifact 4f550f22e274 is not shipping/native adoption.

New3sessions/6windows: current stock CLI, private diagnostic OFF, same private diagnostic ON. Legacy rawTCP one-carrier, two simultaneous P1/eight-second iperf connections, downlink100M/unlimited. Server send/recv requested262144, actual tb/rb524288, INI templates match except server pin.

| Downlink | CLI / probe | Uplink, Mbps | Downlink, Mbps | ACK p50, ms |
|---|---|---:|---:|---:|
| 100000000 | stock | 319.36 | 100.08 | 0.822205 |
| 100000000 | off | 239.01 | 100.09 | 0.949443 |
| 100000000 | on | 251.26 | 100.04 | 0.911861 |
| unlimited | stock | 159.93 | 526.51 | 15.022166 |
| unlimited | off | 162.41 | 536.84 | 14.158821 |
| unlimited | on | 150.30 | 534.13 | 17.156824 |

Observed ratios: down=100000000 stock→off: uplink×0.748, ACKp50×1.155; down=100000000 off→on: uplink×1.051, ACKp50×0.960; down=unlimited stock→off: uplink×1.016, ACKp50×0.943; down=unlimited off→on: uplink×0.925, ACKp50×1.212. One ordered pilot cannot establish observer neutrality; stock/OFF additionally includes a build difference. Do not pool with previous stock benchmarks or close the historical shipping latency cause.

Complete identical PCAP headers linked to seven SHA256 stage stamps. Repeated/incomplete identities excluded with explicit accounting in evidence; timings apply to unique complete identities only. Ring accounting: down=100000000: server events=121009 lost=103 full=0, client events=90732 lost=102 full=0; down=unlimited: server events=82600 lost=8 full=0, client events=61932 lost=24 full=0.

| Downlink | Unique complete ACKs | Segment | p50 / p99, ms |
|---|---:|---|---:|
| 100000000 | 30096 | server_read_to_dequeue | 0.122074 / 7.414881 |
| 100000000 | 30096 | server_dequeue_to_encrypt | 0.000719 / 0.001723 |
| 100000000 | 30096 | encrypt_to_client_plaintext | 0.190948 / 3.744683 |
| 100000000 | 30096 | client_plaintext_to_write_begin | 0.083340 / 2.079618 |
| 100000000 | 30096 | client_tun_write | 0.003847 / 0.109530 |
| unlimited | 20616 | server_read_to_dequeue | 14.311134 / 30.921991 |
| unlimited | 20616 | server_dequeue_to_encrypt | 0.000694 / 0.001425 |
| unlimited | 20616 | encrypt_to_client_plaintext | 0.694058 / 9.009626 |
| unlimited | 20616 | client_plaintext_to_write_begin | 0.702128 / 4.437142 |
| unlimited | 20616 | client_tun_write | 0.002850 / 0.115576 |

In the unlimited window server_read_to_dequeue was the largest segment for17453/20616 complete ACKs (84.66%); p50=14.311134ms versus0.122074ms at100M. Source review shows blocking_send/out_rx, session/routing lookup and stream rx handoff inside this interval; it is not one isolated queue. Next isolate those server boundaries with lower observer overhead. Unlimited OFF→ON ACKp50 increased21.17%; this pilot does not establish neutrality.

Encrypt→client plaintext includes encryption, outer TCP buffering/transmission and client receive/decryption. Retained signed local async write-completion overlap separately: completion may follow client plaintext and is not wire-egress time. Segment quantiles cannot be summed into an end-to-end percentile.

Workload/settings/lifetimes, actual binary/config/PCAP identities, causal stage ordering, actual buffers, complete host/firewall/services/staged preservation and root/archive SHA cleanup qualified. No active workload or owned compile cache remains. No BPF/kprobe/strace/kernel trace. Q36 remains IN_PROGRESS for observer neutrality and the shipping cause;33/37 (89.2%),NO_PUSH_DEPLOY_RELEASE.

Evidence: `release/certification/evidence/q36-ack-stage-pilot-20261014.json`.

## 14 October: separate server handoff boundaries

Private diagnostic source 8f5932cf9043, CLI c7f573dfa066, same ThinLTO/codegen4/opt3/jemalloc profile. Three tests of the identical selector module PASS; actual CLI/dump/PID/hash and bounded build cleanup qualified separately. Product qeli/native unchanged. Nominal1/16 low-nibble-zero IPv4 ID selection happens before SHA/clocks/locks. Actual selection checked against PCAP; a deterministic subset, not an unbiased random sample claim.

New6sessions/12windows: three same-binary OFF/ON pairs, reversed condition/rate order in repeat2. Two simultaneous P1/eight-second iperf connections; rawTCP legacy one-carrier. Server send/recv262144, actual tb/rb524288, INI match except per-session pin. No repeat of shipping matrix or stock pilot.

| Repeat | Downlink | Uplink OFF / ON, Mbps | ACK p50 OFF / ON, ms |
|---|---|---:|---:|
| 1 | 100000000 | 280.61 / 255.33 | 0.940306 / 0.930185 |
| 1 | unlimited | 123.59 / 173.95 | 16.699138 / 10.606932 |
| 2 | 100000000 | 189.45 / 371.11 | 1.216701 / 0.865150 |
| 2 | unlimited | 133.13 / 149.26 | 15.088365 / 11.580175 |
| 3 | 100000000 | 186.73 / 295.79 | 1.201702 / 0.843077 |
| 3 | unlimited | 136.48 / 180.26 | 13.779755 / 11.125608 |

| Repeat | Downlink | Complete sampled ACKs | Segment | p50 / p99, ms |
|---|---|---:|---|---:|
| 1 | 100000000 | 1810 | server_read_to_forwarder | 0.039255 / 2.667328 |
| 1 | 100000000 | 1810 | server_forwarder_to_stream_enqueue | 0.000744 / 0.008741 |
| 1 | 100000000 | 1810 | server_stream_queue | 0.035131 / 3.664136 |
| 1 | 100000000 | 1810 | client_plaintext_to_write_begin | 0.082666 / 1.873564 |
| 1 | unlimited | 1451 | server_read_to_forwarder | 0.211360 / 9.907375 |
| 1 | unlimited | 1451 | server_forwarder_to_stream_enqueue | 0.000728 / 0.008007 |
| 1 | unlimited | 1451 | server_stream_queue | 6.019919 / 36.660742 |
| 1 | unlimited | 1451 | client_plaintext_to_write_begin | 0.680580 / 4.067728 |
| 2 | unlimited | 1224 | server_read_to_forwarder | 0.202277 / 9.924692 |
| 2 | unlimited | 1224 | server_forwarder_to_stream_enqueue | 0.000737 / 0.001996 |
| 2 | unlimited | 1224 | server_stream_queue | 6.237488 / 35.223276 |
| 2 | unlimited | 1224 | client_plaintext_to_write_begin | 0.795184 / 4.249312 |
| 2 | 100000000 | 2629 | server_read_to_forwarder | 0.038608 / 3.063380 |
| 2 | 100000000 | 2629 | server_forwarder_to_stream_enqueue | 0.000752 / 0.008709 |
| 2 | 100000000 | 2629 | server_stream_queue | 0.046379 / 2.991839 |
| 2 | 100000000 | 2629 | client_plaintext_to_write_begin | 0.083706 / 1.821141 |
| 3 | 100000000 | 2126 | server_read_to_forwarder | 0.041898 / 2.392429 |
| 3 | 100000000 | 2126 | server_forwarder_to_stream_enqueue | 0.000737 / 0.009073 |
| 3 | 100000000 | 2126 | server_stream_queue | 0.035224 / 4.422004 |
| 3 | 100000000 | 2126 | client_plaintext_to_write_begin | 0.075225 / 2.182222 |
| 3 | unlimited | 1589 | server_read_to_forwarder | 0.203898 / 11.892317 |
| 3 | unlimited | 1589 | server_forwarder_to_stream_enqueue | 0.000708 / 0.007804 |
| 3 | unlimited | 1589 | server_stream_queue | 6.076858 / 34.961849 |
| 3 | unlimited | 1589 | client_plaintext_to_write_begin | 0.847020 / 5.155170 |

Read→forwarder includes blocking_send/out_rx; forwarder→enqueue includes session/routing lookup, stream choice and pool copy; enqueue→stream dequeue measures time until rx.recv including scheduling. Nine boundaries linked by complete identical headers; no size/counter identity substitution or sum of segment quantiles. Complete/excluded/unsampled/lost/full counts, actual horizons and signed local async-write overlap retained in evidence.

Three unlimited ON stream-queue p50 values:6.019919/6.237488/6.076858ms; at100M:0.035131/0.046379/0.035224ms. Unlimited TUN→forwarder p50=0.202277–0.211360ms; routing/copy=0.000708–0.000737ms. Stream queue is the largest individual segment for61.76–64.85% of complete sampled ACKs. Total10829 complete identities; server try_lock missed2 stamps, client lost0, full0 throughout. Actual selection6.2497–6.2532%; sampled/full PCAP p50 ratio0.99255–1.02936 across six ON windows, not a general unbiasedness guarantee.

Unlimited ON/OFF ACKp50 ratios0.63518/0.76749/0.80739 (median−23.25%); median uplink ratio1.32085. Sparse observation still does not establish neutrality:6ms is not the exact shipping latency. Next use a bounded FIFO/service versus negotiated recordizer control, verifying only stages actually available in MUX, rather than another tracing variant/full matrix replay.

Keep OFF/ON ratios and repeats separate: reduced overhead is not automatic observer neutrality. Do not pool this diagnostic artifact with shipping CLI or previous unsampled pilot. Owned processes/root/cache/archive removed; host/firewall/services/staged/source preserved. No BPF/kprobe/strace/kernel trace. Q36 IN_PROGRESS;33/37 (89.2%),NO_PUSH_DEPLOY_RELEASE.

Evidence: `release/certification/evidence/q36-server-handoff-runtime-20261014.json`.

## 14 October: recordizer and server FIFO

Execution source 770ef7e14bb9; reused private ThinLTO/codegen4 diagnostic CLI c7f573dfa066, no compilation/product/native delta. New6sessions/12windows: three server obf.recordizer.policy off/required pairs, both endpoint probes ON. Reversed condition/rate order in repeat2. Two simultaneous P1/eight-second inner TCP connections, rawTCP one carrier. Actual legacy_packet_per_record/packet_mux_v1 negotiation, PID/outer sockets/actual tb-rb524288 and original INI verified; only server policy/per-session pin differ.

| Repeat | Downlink | Uplink legacy / MUX, Mbps | ACK p50 legacy / MUX, ms | Stream queue p50 legacy / MUX, ms |
|---|---|---:|---:|---:|
| 1 | 100000000 | 289.72 / 1175.55 | 0.922641 / 1.064547 | 0.049031 / 0.006099 |
| 1 | unlimited | 103.35 / 421.51 | 29.316383 / 3.159679 | 27.069859 / 0.048029 |
| 2 | 100000000 | 290.31 / 1077.67 | 0.908023 / 1.157373 | 0.044805 / 0.007760 |
| 2 | unlimited | 140.48 / 367.76 | 14.558145 / 3.486144 | 10.075171 / 0.061062 |
| 3 | 100000000 | 214.79 / 1222.21 | 1.012247 / 1.014984 | 0.045017 / 0.005577 |
| 3 | unlimited | 188.59 / 433.51 | 10.517356 / 3.285821 | 2.677874 / 0.049269 |

Complete identical PCAP headers linked to six common boundaries: server read→forwarder→enqueue→dequeue→client TUN begin→done. Total36046 complete sampled identities; exclusions/repetitions/missed stamps/nonselected1/16 headers retained. MUX combines packets into records: per-ACK encryption/plaintext/write-completion timestamps are absent in this branch and were not substituted with whole-record times. Dequeue→client TUN combines recordization/encryption/outer TCP/decode/client queue.

Paired MUX/legacy min/median/max:100M uplink 3.712/4.057/5.690; unlimited uplink 2.299/2.618/4.078; unlimited ACKp50 0.108/0.239/0.312. A batching-policy intervention in this topology, not isolated queue-size causality: morphology/deadline/writer servicing/buffering also change. Prior OFF/ON controls demonstrated observer effects; both probes ON does not prove neutrality or equivalent observer work, because available stamps differ. No default/config/release artifact adoption, shipping pooling or DPI-equivalence claim.

Workload/negotiation/topology/headers/common-stage ordering/buffers/source/host/firewall/services/staged and SHA inventory/archive cleanup PASS; owned lab resources removed. Actual drops/retransmits and historical FAIL retained. Q36 IN_PROGRESS for shipping cause acceptance;33/37(89.2%),NO_PUSH_DEPLOY_RELEASE.

Evidence: `release/certification/evidence/q36-recordizer-handoff-20261014.json`.

## 14 October: shipping CLI and Q36 completion

Another6sessions/12windows use shipping CLI6f617bdcab5a without QELI_ACK_PROBE/SIGUSR1/compilation. Three legacy/MUX policy-only pairs, one carrier, same bounded PCAP/ss controls. No absolute zero-overhead claim. Runtime generation/source/config/PID/socket/matched headers/receiver roles/host/staged/root/archive cleanup PASS.

| Repeat | Downlink | Uplink legacy / MUX, Mbps | ACK p50 legacy / MUX, ms |
|---|---|---:|---:|
| 1 | 100000000 | 283.48 / 1240.52 | 0.773993 / 0.989687 |
| 1 | unlimited | 156.61 / 403.49 | 13.882233 / 3.277941 |
| 2 | 100000000 | 144.76 / 1240.37 | 1.197513 / 1.012411 |
| 2 | unlimited | 157.38 / 431.46 | 17.154599 / 3.166323 |
| 3 | 100000000 | 281.17 / 1235.29 | 0.874503 / 0.956438 |
| 3 | unlimited | 150.82 / 432.10 | 13.001141 / 3.119505 |

Unlimited paired MUX/legacy median uplink=2.742×, downlink=2.043×, ACKp50=0.236×. At100M median ACKp50=1.094×: batching/deadline can cost latency despite lower queue residence. Policy effect confirmed on shipping binary; private queue times are not relabelled as shipping stamps. ACKs and downlink data share legacy per-stream FIFO; batching improves writer service. No global required default: morphology/peer compatibility changes.

[Five Q36 criteria completed](AUDIT-Q36-FINAL.md). Bounded benchmark audit DONE, not capacity/performance-budget PASS. Original FAIL, kernel FULL_RING overload and legacy queue drops retained. Further latency tuning is optional optimization, not a reason to replay completed checks.34/37(91.9%);Q29/Q31/Q34 open,NO_PUSH_DEPLOY_RELEASE.
