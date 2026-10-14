# Q36: matched-total P1/P4 load and TUN boundary counters

13 October 2026. Six independent sessions/60 eight-second windows: TCP raw and QUIC with UDP payload at 100/150/200/300/400Mbps **per direction**, P1/P4. P4 per-stream bitrate is divided by 4 to match P1 total load. Repeat2 reverses mode and load order. All three samples, losses and outliers remain retained.

Private sibling namespaces share the two-vCPU .10 VM; outer/inner IPv4, TUN 1400/carrier 1500, datagram 1200, affinity 0–1/nice 0. Existing qualified CLI 52511cebd43a was used without rebuilding. All 916 data-plane inputs match prior source applicability; subsequent documentation/Android changes are separate. No whole-current-stock binary equivalence is claimed and no defaults changed.

Medians of three independent sessions; ↑ client→server, ↓ server→client. Complete samples/min/max/spread/CPU/RSS/UDP counters are in evidence.

|Mode|Mbps/direction|P1 delivered ↑/↓|P4 delivered ↑/↓|P1 loss ↑/↓ %|P4 loss ↑/↓ %|
|---|---:|---:|---:|---:|---:|
|tcp-plain-raw|100|99.99/99.99|99.98/99.99|0.00/0.00|0.00/0.00|
|tcp-plain-raw|150|149.99/149.99|149.99/149.99|0.00/0.00|0.00/0.00|
|tcp-plain-raw|200|199.98/199.98|199.98/199.97|0.00/0.00|0.00/0.00|
|tcp-plain-raw|300|292.13/298.98|213.02/286.56|0.15/0.00|28.80/2.70|
|tcp-plain-raw|400|253.84/390.11|202.80/284.90|36.39/1.99|49.14/26.90|
|udp-quic|100|99.99/99.99|99.99/100.00|0.00/0.00|0.00/0.00|
|udp-quic|150|149.98/149.98|149.98/149.98|0.00/0.00|0.00/0.00|
|udp-quic|200|199.98/199.97|199.98/199.98|0.00/0.00|0.00/0.00|
|udp-quic|300|281.94/289.48|276.21/152.87|5.88/0.00|7.81/43.04|
|udp-quic|400|277.83/212.26|219.21/145.15|28.73/39.45|45.05/53.03|

All 36 windows at 100–200 Mbps per direction recorded zero loss. Losses and stream-count effects at 300–400 Mbps were measured at identical offered totals. This bounds short windows on this fixture, not universal/long-term product capacity or all12 modes. Earlier P4=400+400 and P1=100+100 measurements remain separate populations.

## Recorded loss boundaries

Three additional TCP raw sessions/12 windows: UDP 300 Mbps per direction and TCP bidir at P1/P4. Kernel link64-bit packet/drop counters and native client status were read at two-second idle boundaries. PID/start ticks, TUN ifindex/MTU/qdisc/txqlen, status generation/plan and monotonicity were verified. No packet tracing, BPF or queue changes. The existing CPU sampler remains; zero observation overhead is not claimed.

|Payload|Streams|Delivered ↑/↓ Mbps|Client TUN TX drops|Server TUN TX drops|Native TCP uplink drops|
|---|---:|---:|---:|---:|---:|
|udp|1|289.94/299.95|7798|0|0|
|udp|4|217.14/294.15|68508|0|0|
|tcp|1|170.76/551.97|226|499|0|
|tcp|4|149.78/567.39|1159|1449|0|

Client TUN recorded 257250 TX drops across these 12 windows; native TCP uplink drops stayed zero. TUN TX is kernel→reader. These counters are separate from server session drops and socket RcvbufErrors. This localizes recorded boundary losses; it does not prove per-packet stage latency or exact payload-to-kernel-event matching. The final cause of TCP asymmetry remains open.

Independent raw receiver/cookie/rate/duration/process/counter admission passed. Full host/production-service/staged-binary guards passed; all three owned roots and archives were SHA-verified and removed. No Rust/native rebuild. Q36 IN_PROGRESS; overall 33/37(89.2%). NO_PUSH_DEPLOY_RELEASE.

Evidence: `release/certification/evidence/q36-equal-load-20261013.json` and `q36-queue-boundaries-20261013.json`.

## Paired client queue-length intervention

Three more sessions/12 windows compare client txqueuelen 500/1000 within the same generation on the owned TUN only. UDP 300 Mbps P4 and TCP bidir P1; repeat2 reverses order. Product defaults remain unchanged.

|Payload/streams|Client queue|Delivered ↑/↓ Mbps|Receiver loss ↑/↓ %|Client TUN TX drops|
|---|---:|---:|---:|---:|
|udp_p4|500|202.47/288.21|32.34/2.61|80910|
|udp_p4|1000|212.56/276.37|28.58/5.96|71633|
|tcp_p1|500|170.29/580.71|—|384|
|tcp_p1|1000|218.61/320.10|—|211|

In 3/3 TCP pairs, aggregate goodput at 1000 was below 500. Median paired changes: uplink +27.01%, downlink -44.88%, sum -25.10%. This measures queue influence on the directional balance. The increase is not qualified as a general performance fix; defaults remain unchanged and the final original TCP-asymmetry cause stays open. All individual pairs are retained.

Paired intervention evidence: `release/certification/evidence/q36-queue-length-control-20261013.json`.


## Separate TCP directions: 13 October

Following the queue-length control, **3 fresh sessions /9 eight-second windows** measured TCP raw P1 upload, download and bidirectional traffic within the same generation in each session. Repeat2 reverses order. Original queue lengths/settings retained. IPv4/IPv4, same two-vCPU VM, affinity0–1/nice0, TUN1400/carrier1500, CLI52511cebd43a and all916 native/config/helper inputs unchanged. No tracer, BPF or rebuild. Earlier populations remain separate.

Medians of three sessions; process CPU percentages use one core as100%. A dash means no payload was requested in that direction, not a measured zero.

|Window|Delivered ↑ Mbps|Delivered ↓ Mbps|Server CPU %|Client CPU %|Client TUN TX drops|Server TUN TX drops|
|---|---:|---:|---:|---:|---:|---:|
|Upload only|139.93|—|16.33|19.67|53.00|0.00|
|Download only|—|692.55|78.11|73.33|0.00|1019.00|
|Bidirectional|172.11|567.44|79.22|77.88|270.00|298.00|

Upload is low without concurrent download. Upload-only process CPU is substantially below one fully occupied core, so bidirectional CPU competition alone does not explain this population. Window-average CPU does not exclude short saturation episodes.

All three upload-only sender retransmit counts equal client TUN TX drops: **34/62/53**. Download-only counts equal server TUN TX drops: **1034/1019/952**. Bidirectional counts do not universally match; TUN also carries ACK/control packets. Client native TCP uplink/downlink drop counters remained unchanged in all nine windows. Equal aggregate counts associate these results with TUN losses; they do not establish individual packet correspondence, driver-drop mechanism, ACK-delay stage or an asymmetry fix. Eight-second windows do not exclude congestion/slow-start effects; TCP-over-TCP and buffer limits remain hypotheses.

Source review: both senders write a complete encrypted record through write_all_flush; separate header/body writes were not found. Client TUN read pool is bounded to64 buffers/4MiB, with fq_codel/500; server uses a shared32MiB pool and multiqueue/1000. These differences are not causal proof. No speculative product queue/pool increase was applied.

The external Windows collector wrote local log snapshots in CP1251/CRLF. The initial UTF8-only qualifier refused; its second revision refused nonnumeric TCP-only loss/jitter=None. Both refusals are retained. The final qualifier reads complete original UTF8 logs from the SHA-verified cleanup archive and verifies each local snapshot as its exact normalized text prefix. None remains in samples without zero imputation or numeric summaries. All38 raw JSON /6 logs are SHA-bound; no network rerun. The future external collector template writes explicit UTF8 bytes; it has not been claimed as another runtime. The common benchmark helper already writes UTF8 worker logs and is unchanged.

Full source/host/service/staged CLI/archive/cleanup gates PASS; owned processes/namespaces and remote root/archive removed. CURRENT_SAME_SESSION_TCP_DIRECTION_CONTROL_QUALIFIED. Q36 remains open;33/37 (89.2%); NO_PUSH_DEPLOY_RELEASE.


## Upload duration and producer pacing: 13 October

Another **6 fresh sessions /12 windows** tested two independent hypotheses. Three paired sessions compared TCPraw P1 upload8/32s; three separate sessions compared unlimited and iperf TCP producer300Mbps in eight-second windows. Each pair retains PID/NetworkPlan/generation/TUN; repeat2 reverses order. IPv4/IPv4, same two-vCPU VM, affinity0–1/nice0, MTU1400/1500, CLI52511cebd43a and all916 frozen inputs unchanged. No product/queue/qdisc changes, builds/caches/tracers/BPF. Populations are not pooled.

### Longer window

Three-session medians; process CPU uses one core as100%.

|Seconds|Receiver Mbps|Server CPU %|Client CPU %|Client TUN TX drops|Retransmits|
|---|---:|---:|---:|---:|---:|
|8|173.19|20.78|25.55|43.00|43.00|
|32|165.15|20.18|24.76|25.00|25.00|

Thirty-two seconds did not recover upload to the separately measured download level. Second-half sender intervals still contain retransmits in all three32s runs:6/54/7; losses are not confined to startup. One-second sender bytes/retransmits/TCP_INFO(cwnd, RTT, rttvar, PMTU) retain contiguous intervals, exact summary sums, no omitted/warmup. Sender interval bitrate is not delivered goodput: final receiver summary is qualified separately. This32s control does not exclude all longer congestion changes and is not capacity/soak.

### TCP producer pacing

Only iperf `-b 300000000` changes: software-paced test production before TUN, not a Qeli limit/fix. Unlimited has no `-b`; startup traffic is retained.

|TCP producer|Receiver Mbps|Server CPU %|Client CPU %|Client TUN TX drops|Retransmits|
|---|---:|---:|---:|---:|---:|
|Unlimited|145.99|18.78|22.89|50.00|50.00|
|300Mbps|300.06|36.11|40.22|0.00|0.00|

Paced TCP delivered **300.03–300.07Mbps in3/3 windows, with zero retransmits/TUN TX drops** and zero client native TCP drops. Unlimited delivered132.16–161.10Mbps, with kernel drops/retransmits50/8/63. Median paired receiver-speed gain:2.06×. Thus this uplink can carry300Mbps;≈146Mbps is not an immutable compute ceiling of the build. Dependence on producer mode is confirmed; exact per-packet burst mechanism and runtime drop reason are not observed. This is a measured producer mitigation, not a universal transformation of other applications or maximum throughput. Benchmark defaults retain unlimited results rather than substituting paced load.

### Upstream TUN source

In [upstream6.12.105](https://github.com/gregkh/linux/blob/v6.12.105/drivers/net/tun.c), a full pointer ring can increment TX dropped. Other errors use that counter too, so aggregates do not prove FULL_RING. Changing txqueuelen also resizes driver rings. Version matches the lab kernel number, but exact Debian-build equivalence is unverified. [Upstream backpressure was reverted](https://github.com/torvalds/linux/commit/f11b48aa674b475f196bede7d69593c050107fc5) after another test regressed; no kernel change here. This is source review, not kernel-runtime qualification.

Both runtime/source/host/service/staged CLI/archive/cleanup gates PASS; owned resources removed after SHA archival. Explicit UTF8 logs qualified on the first attempt. Each experiment binds26 raw JSON/6 worker logs; complete sender telemetry remains in original iperf JSON. Q36 remains open: producer mitigation measured;33/37(89.2%) unchanged. A paired Linux TUN pacing/qdisc upload+bidir control is appropriate before product changes; larger queues already showed a negative tradeoff. NO_PUSH_DEPLOY_RELEASE.


## Client TUN: fq_codel versus fq, 13 October

**3 fresh sessions /12 eight-second windows**, TCPraw P1 unlimited upload/bidir, outer/innerIPv4. Within each session only the owned client TUN qdisc changes: original fq_codel options versus standard fq. Generation/PID/NetworkPlan, MTU1400/1500, txqueuelen500, CLI52511cebd43a and all916 frozen inputs retained; repeat2 reverses order. These classes have different natural parameters: a configuration comparison, not one isolated algorithm with identical numeric settings. No product/kernel source/default changes, builds/tracers/BPF.

Three-session medians; dash means no requested payload. Complete receiver/retransmit/CPU/RSS/TUN/native/tc-s-JSON records retained.

|Client qdisc|Payload|Delivered ↑ Mbps|Delivered ↓ Mbps|Client TUN TX drops|Client qdisc drops|
|---|---|---:|---:|---:|---:|
|fq_codel|tcp_up|136.09|—|50.00|0.00|
|fq_codel|tcp_bidir|160.49|562.79|242.00|0.00|
|fq|tcp_up|131.34|—|36.00|0.00|
|fq|tcp_bidir|167.05|563.19|249.00|0.00|

Decisions use within-pair changes, not ratios of separate medians. Bidir sums both receivers from the same pair.

|Repeat|Upload change %|Bidir sum change %|
|---|---:|---:|
|1|-18.96|-11.88|
|2|-15.46|+8.94|
|3|+1.31|-7.71|

Median paired upload change **-15.46%**, bidir sum **-7.71%**. One bidir repeat improved and two regressed; none excluded. A default switch to fq **is unsupported**, so fq_codel stays.

FQ was applied: kind/handle/options verified and throttled counter increased in all6 FQ windows (1657/2611/2069/4002/1896/3570). All12 qdisc-drop/client native TCP-drop deltas are0; kernel TUN TX losses persist. Not a loss fix or specific FULL_RING-event proof. The [upstream iproute2 manual](https://man7.org/linux/man-pages/man8/tc-fq.8.html) describes TCP pacing and socket-flow support; it does not guarantee a TUN fix. Scope is TCPraw/P1/this VM, not every mode/platform.

### Experiment-controller correction

Two initial attempts each preserved two fq_codel windows but refused before FQ measurements. The controller requested fq at handle1 already used by fq_codel; the same refusal persisted after successful module loading. The earlier autoload-cause explanation **is withdrawn**. [Source6.12.105](https://github.com/gregkh/linux/blob/v6.12.105/net/sched/sch_api.c) checks kind against an existing handle. R3 uses fq_codel1/fq2 and a pre-payload1→2→1 smoke, PASS in all3 sessions. Both original FAILs remain separate; their4 windows are not pooled. This was a measurement-controller error, not a Qeli bug.

Stock sch_fq explicitly loaded temporarily; after owned processes/namespaces/root/archive removal, the introduced module was unloaded at refcnt0. Original module set/global qdiscs/default/provider SHA/services/staged CLI/full host/source/archive/cleanup gates PASS. Both refused attempts and module probe are also fully restored; no new lab cleanup debt.50 raw JSON/6 worker logs qualified on first attempt. Negative candidate remains visible without a performance fix/PASS claim.

Q36 IN_PROGRESS,33/37(89.2%). Qdisc replacement is not a confirmed universal fix. Prior producer300Mbps pacing remains a measured test-producer mitigation; a targeted next candidate could control pre-TUN rate without caller cooperation, with bidir tradeoff validation. NO_PUSH_DEPLOY_RELEASE.


## Pre-TUN rate control, 13 October

**3 new paired sessions / 12 eight-second windows:** original client TUN fq_codel versus fq maxrate300Mbit per flow. TCPraw P1 upload/bidir, outer/innerIPv4, same two-vCPU VM. Caller unlimited: iperf target_bitrate=0, no `-b`. Only the owned client TUN qdisc changed; original fq_codel options, txqueuelen500, MTU1400/1500, generation/PID/NetworkPlan and all916 frozen native/config/helper inputs retained. Repeat2 reverses order. No new builds, tracing, BPF or product changes.

`tc -j` verifies maxrate37500000 bytes/s before payload and at both boundaries of every capped window. This limits **each flow**, not the profile aggregate. [Linux6.12.105 source](https://github.com/gregkh/linux/blob/v6.12.105/net/sched/sch_fq.c) advances the next-packet time for a configured maximum even with TCP EDT. Upstream review is not compiled Debian-kernel byte equivalence; runtime separately records the stock module SHA. Scheduler packet rate includes more than TCP application payload, so delivered payload may be below300Mbps.

Three-session medians; dash denotes no requested payload in that direction. Original receiver/retransmit/CPU/RSS/counter records retained without exclusions.

|Client qdisc|Payload|Delivered ↑ Mbps|Delivered ↓ Mbps|Client TUN TX drops|Server TUN TX drops|Client qdisc drops|
|---|---|---:|---:|---:|---:|---:|
|original fq_codel|tcp_up|137.51|—|36.00|0.00|0.00|
|original fq_codel|tcp_bidir|167.11|534.53|274.00|643.00|0.00|
|fq maxrate 300Mbit/flow|tcp_up|288.60|—|0.00|0.00|0.00|
|fq maxrate 300Mbit/flow|tcp_bidir|184.54|532.82|380.00|679.00|0.00|

All3 capped upload windows deliver288.56–288.84Mbps with zero TUN TX drops at both ends and zero TCP retransmits. Baseline upload133.86–156.27Mbps with drops/retransmits36/28/115. A reproduced upload mitigation on this VM without caller cooperation. All12 client qdisc-drop/native client TCP-drop deltas are0; FQ throttled increases in all6 capped windows, verifying application of the control.

Within-session paired changes; bidir sums both receivers of the same pair. Decisions do not use ratios of separate medians.

|Repeat|Upload change %|Bidir upload change %|Bidir download change %|Bidir sum change %|
|---|---:|---:|---:|---:|
|1|+115.77|-6.33|+11.58|+7.74|
|2|+109.84|+15.68|-4.53|+0.26|
|3|+84.69|+10.20|-0.32|+2.19|

Median paired upload gain **+109.84%**, bidir sum **+2.19%**. However, every capped bidir window retains client TUN TX drops380/523/335 and server TUN TX drops679/429/705; upload retransmits185/375/252. Both-direction losses persist; upload falls in one pair and download falls in another. Neither the300Mbit cap nor fq becomes a default. Qdisc kind plus rate cap is a configuration comparison, not isolation of one parameter. Earlier unlimited-fq and producer-pacing populations stay separate.

Pre-payload fq_codel1→fq2→fq_codel1 smoke PASS3/3 restores original fq_codel options exactly. After SHA archiving, owned processes/namespaces/root/remote archive removed. Introduced sch_fq unloaded at refcnt0; original module set/global qdiscs/default/services/provider/staged CLI SHA/full host/source gates PASS.50 raw JSON/6 worker logs qualified on first attempt. No new lab cleanup debt.

Q36 IN_PROGRESS: rate control helps upload, but no general TUN/ACK loss fix or qualified capacity limit. Aggregate TX dropped is not an exact FULL_RING-event diagnosis.33/37(89.2%), NO_PUSH_DEPLOY_RELEASE.


## Negotiated recordizer versus legacy, 13 October

**6 fresh sessions /18 eight-second windows**,3 matched off/required pairs, TCPraw P1 upload/download/bidir, outer/innerIPv4. Repeat2 reverses session/payload order. Same CLI52511cebd43a,916 frozen native inputs and9 unchanged helpers, original MTU1400/1500/qdiscs/queues. Unlimited caller, no omit/warmup. Separate connection generations; no live same-generation toggle. No native/kernel/default changes, builds, pacing, tracing or BPF.

New profiles use prefer in the [constructor](../../../qeli/src/config/server.rs); historical baseline and [benchmark](../../../scripts/benchmark.py) use off. Required enforces actual negotiation without silent fallback: NetworkPlan shows packet_mux_v1 in all3 required sessions and legacy_packet_per_record in all3 off sessions. Independent client logs confirm PACKET_MUX_V1. SHA-archived INI receipts verify policy is the only specified recordizer field; default batching2–8ms/16frames, randomized size morphology and fragmentation retained. A negotiated-mode comparison, not a separate prefer-fallback runtime test.

Three-session medians; dash means no requested payload. All raw receiver/retransmit/CPU/RSS/TUN/native records retained without sample exclusions.

|Policy|Payload|Delivered ↑ Mbps|Delivered ↓ Mbps|Client TUN TX drops|Server TUN TX drops|Client CPU %|Server CPU %|
|---|---|---:|---:|---:|---:|---:|---:|
|off|tcp_up|159.51|—|90.00|0.00|21.55|17.89|
|off|tcp_down|—|686.43|0.00|500.00|73.11|78.88|
|off|tcp_bidir|154.29|600.28|432.00|697.00|77.66|79.77|
|required|tcp_up|1421.05|—|182.00|0.00|71.44|72.00|
|required|tcp_down|—|1530.85|0.00|318.00|71.89|67.33|
|required|tcp_bidir|436.40|1189.07|501.00|93.00|76.22|73.11|

Matched-session changes in each repeat; bidir sum adds both receivers within the corresponding session.

|Repeat|Upload change %|Download change %|Bidir upload change %|Bidir download change %|Bidir sum change %|
|---|---:|---:|---:|---:|---:|
|1|+938.33|+120.53|+208.41|+98.09|+118.45|
|2|+753.27|+142.25|+140.67|+118.82|+124.28|
|3|+795.19|+115.73|+194.31|+91.36|+112.41|

Median paired changes: upload **+795.19%**, download **+120.53%**, bidir sum **+118.45%**. Direction differences remain visible; aggregate effects cannot be assigned to every flow/mode/platform. Separate evidence counts windows with TUN/native losses; zero median does not mean all repeats are loss-free. No universal performance/capacity PASS or automatic default change.

Source review of the [client writer](../../../qeli/src/client/mod.rs) confirms current TunPacket release before awaiting outer write **in both branches**. Current-buffer-held-during-write hypothesis unsupported. [Linux pump](../../../qeli/src/transport_core/linux_tun.rs) caps read buffers at64 despite a nominal4096-slot channel; bounded memory/backpressure, not itself a reproduced bug. Recordizer simultaneously changes batching/size/fragmentation/flush timing; this comparison does not identify exact ACK delay or a FULL_RING event.

Source/generation/negotiated-mode/full host/services/staged CLI/SHA archive/cleanup gates PASS. Owned processes/namespaces/root/remote archive removed;74 raw JSON/12 worker logs qualified. No new lab cleanup debt. Q36 IN_PROGRESS,33/37(89.2%), NO_PUSH_DEPLOY_RELEASE.

Initial attempt retained3 baseline windows, then worker refused before required measurements: experiment-owned Unix control-socket path exceeded SUN_LEN. R2 uses short directory labels with path-length preflight. Original FAIL/3 windows remain separate, excluded from R2 tables; both attempts fully cleaned. Experiment-controller correction, not a recordizer bug.


## Recordizer deadline:2–8ms versus1ms,13 October

**6 new sessions /18 eight-second windows**,3 matched required-recordizer pairs: default2–8ms versus fixed1ms. TCPraw P1 upload/download/bidir, outer/innerIPv4, same2vCPU VM. Only two INI delay fields change; max_packets16/random sizes/fragmentation/other options retained. Repeat2 reverses session/payload order. Both conditions retain1s iperf intervals, unlimited caller and no omit/warmup. CLI52511cebd43a,916 native inputs/9 helpers unchanged, original MTU/queues/qdisc/affinity. Separate fresh population, not pooled with prior off/required or pacing results.

NetworkPlan and client log verify PACKET_MUX_V1 in all6 sessions. SHA-archived INI: required in both, fixed1ms alone adds `obf.recordizer.batch.delay_min_ms=1`/`obf.recordizer.batch.delay_max_ms=1`. No native builds/product/default changes/tracing/BPF.

Three-session medians. RTT column first takes each window's median **client-sender interval TCP_INFO snapshots**, then median of3 sessions. Not a per-packet RTT distribution or exact ACK-stage latency. Reverse client cannot observe remote sender TCP_INFO: N/A, no zero imputation.

|Deadline|Payload|Delivered ↑ Mbps|Delivered ↓ Mbps|Client TUN TX drops|Server TUN TX drops|Median sampled client-sender RTT µs|
|---|---|---:|---:|---:|---:|---:|
|default2–8ms|tcp_up|1416.59|—|173.00|0.00|3221.50|
|default2–8ms|tcp_down|—|1492.80|0.00|380.00|N/A|
|default2–8ms|tcp_bidir|443.35|1186.68|499.00|165.00|7166.00|
|fixed1ms|tcp_up|1364.67|—|250.00|0.00|3687.00|
|fixed1ms|tcp_down|—|1441.48|0.00|220.00|N/A|
|fixed1ms|tcp_bidir|412.69|1238.19|404.00|238.00|7404.50|

Before payload,30 idle ICMP probes per session; raw ping times/averages verified. Separate idle ICMP, not loaded TCP RTT.

|Deadline|Idle ICMP RTT: median of3 session averages ms|
|---|---:|
|default|13.770|
|1ms|4.919|

Within-repeat changes; bidir sum adds both receivers of the corresponding session.

|Repeat|Upload change %|Download change %|Bidir upload change %|Bidir download change %|Bidir sum change %|Idle ICMP average change %|
|---|---:|---:|---:|---:|---:|---:|
|1|+2.92|-3.71|-14.39|+5.96|+0.34|-63.83|
|2|-7.89|-7.04|+21.01|+13.57|+15.61|-64.28|
|3|+11.39|+7.04|-6.92|+2.62|+0.06|-66.04|

Median paired changes: upload **+2.92%**, download **-3.71%**, bidir sum **+0.34%**, idle ICMP average **-64.28%**. All repeats/retransmits/TUN/native drops/CPU/RSS retained. Zero median is not zero loss in every window; separate loss-window counts retained in analysis evidence.

[Source review](../../../qeli/src/protocol/recordizer.rs): incomplete records share a deadline without ACK-only classification; size/count limits can flush sooner. Default2–8ms is not an unconditional delay for every packet. Fixed1ms retains batching but removes deadline randomization; DPI equivalence untested. Defaults unchanged. This comparison tests deadline effects, not an exact FULL_RING/ACK stage or universal capacity.

Source/generation/host/services/staged CLI/SHA archive/cleanup gates PASS. Owned processes/namespaces/root/remote archive removed;80 raw JSON/12 worker logs qualified. No new cleanup debt. Q36 IN_PROGRESS,33/37(89.2%),NO_PUSH_DEPLOY_RELEASE.


## Fixed1 versus2 TCP carriers,13 October

**Q36-F030 (P2), REPRODUCED / FIX_PENDING:** upload collapses in all3 two-carrier pairs; all native uplink drops are QueueFull, not PoolExhausted/Oversize/NoCarrier. One-carrier native uplink drops0 in all9 windows.4MiB reserves64 slots of65535 bytes; width2 reserve formula allows only21 packets/queue. Negotiated recordizer already limits inner packets to MTU+64,1464 bytes here. Proposed slot sizing must use that existing accepted bound while preserving total byte budget and other-carrier/candidate reserves; not applied/validated yet. No exact ACK-packet or kernel FULL_RING event claim.

**6 new sessions /18 eight-second windows**,3 matched1/2 outer TCP-carrier pairs, TCPraw P1 upload/download/bidir, outer/innerIPv4, same2vCPU VM. Both INI set multipath enabled=true/adaptive=false, required recordizer default2–8ms. Only max_streams1/2 differs. Repeat2 reverses session/payload order. CLI52511cebd43a,916 native inputs/9 helpers, original MTU1400/1500/qdiscs/queues/affinity. Unlimited caller, no omit/warmup. Separate population, no pooling with earlier recordizer/pacing results.

Plan max_streams alone is insufficient: actual ESTABLISHED sockets filtered by native PID/outer endpoint on both ends before payload and at every boundary. All42 snapshots have mirrored endpoint pairs, correct1/2 width and stable tuples. NetworkPlan/client log verify PACKET_MUX_V1; width2 log also confirms two active fixed streams. SHA-archived INI verified. No builds/tracing/BPF/product/default changes.

[Flow hash](../../../qeli/src/protocol/mod.rs) pins each inner flow to one carrier, no per-packet round-robin striping. Peers independently hash their own outbound directions; distinct flows can collide. [Bonded path](../../../qeli/src/client/mod.rs) also enables a bounded4MiB copy pool/different queue capacities, releasing original TunPacket after copy. Not one isolated ACK stage; two established sockets do not prove exact placement of every inner flow. Queue Full records a native drop rather than rerouting onto another live carrier.

Three-session medians, all repeats/retransmits/CPU/RSS/kernel/native counters retained. RTT is median client-sender interval TCP_INFO snapshots, then median3 sessions. Reverse sender RTT N/A, no zero imputation; not a per-packet RTT distribution.

|Carriers|Payload|Delivered ↑ Mbps|Delivered ↓ Mbps|Client TUN TX drops|Server TUN TX drops|Client native uplink drops|Client native downlink drops|Median sampled client-sender RTT µs|
|---|---|---:|---:|---:|---:|---:|---:|---:|
|1|tcp_up|1433.04|—|167.00|0.00|0.00|0.00|3369.50|
|1|tcp_down|—|1506.22|0.00|231.00|0.00|0.00|N/A|
|1|tcp_bidir|425.55|1192.89|319.00|309.00|0.00|0.00|7015.50|
|2|tcp_up|24.59|—|0.00|0.00|148.00|0.00|8565.00|
|2|tcp_down|—|1494.84|0.00|187.00|34348.00|0.00|N/A|
|2|tcp_bidir|20.80|1490.85|0.00|257.00|37085.00|0.00|8564.00|

Separate30 idleICMP probes/session:

|Carriers|Idle ICMP RTT: median of3 session averages ms|
|---|---:|
|1|13.729|
|2|13.020|

Within-repeat changes; bidir sums both receivers in the corresponding session.

|Repeat|Upload change %|Download change %|Bidir upload change %|Bidir download change %|Bidir sum change %|Idle ICMP average change %|
|---|---:|---:|---:|---:|---:|---:|
|1|-98.28|-16.51|-99.02|+20.71|-10.77|-5.88|
|2|-98.12|+0.13|-94.82|+22.45|-7.57|+1.91|
|3|-98.28|+0.14|-95.25|+29.39|-4.56|-0.25|

Median paired changes: upload **-98.28%**, download **+0.13%**, bidir sum **-7.57%**. Exact native-loss samples/window counts retained in analysis evidence; zero median does not guarantee zero every window. Not a universal loss/asymmetry fix, ACK-priority feature, capacity limit or automatic all-profile multipath default.

Source/generation/actual socket/host/services/staged CLI/SHA archive/cleanup gates PASS. Owned processes/namespaces/root/remote archive removed;80 raw JSON/12 worker logs qualified. Defaults unchanged, no new cleanup debt. Q36 IN_PROGRESS,33/37(89.2%),NO_PUSH_DEPLOY_RELEASE.


## Q36-F030: QueueFull candidate validated in Linux,13 October

**LINUX_CANDIDATE_VALIDATED; source adoption/native consumers pending.** Working source unchanged. One-file clientmod candidate sizes bonded-copy slots from the existing accepted packet bound: legacy MAX_TUNNEL_MTU or mux max_packet_bytes,2KiB floor/existing u16 limit. MTU1400 yields2048 two-KiB slots/682 width2 queue entries versus64/21. Same4MiB storage budget and other-carrier/temporary-candidate reserve helper; no fallback allocations or rerouting Full flows.

Actual isolated Rust1.97 build:4 strict feature checks (default/server/client-bin/transport-core-ffi), **15 unit tests** (6 client uplink/8 buffer pool/1 server wire pool). New test queues128 pinned1400/40-byte packets before drain, verifies FIFO/no drops and stalled-queue reserve. Synthetic unit burst, not real ACK capture. Packet/byte bounds and original Full/Closed/Oversize/PoolExhausted/release cases checked. Same factory release profile/jemalloc CLI, one compiler job. Rustfmt normalized only candidate clientmod. Not a substitute for platform artifact refresh/source adoption.

Exact frozen factory916 inputs plus one clientmod variant. Current applicability retains7 known format/panel-resource differences from factory; no whole-current CLI byte equivalence. **Server stays exact baseline52511cebd43a in every runtime**, client baseline or candidate524443061b95, isolating client change from server rebuild.

**6 fresh sessions /18 eight-second windows**,3 baseline/candidate client pairs, fixed2carriers/required recordizer default2–8ms, TCPraw P1 up/down/bidir, outer/innerIPv4 on2vCPU VM. Repeat2 reverses session/payload order. Original INI/MTU/qdisc/queues/affinity/unlimited caller/no omit,1s senderTCP_INFO and30idleICMP eachsession. ProcPID CLI SHA and actual mirrored stable socket tuples checked at42 snapshots. Previous1/2carrier population not pooled.

Three-session medians; all samples and kernel/native counters retained:

|Client|Payload|Delivered ↑ Mbps|Delivered ↓ Mbps|Native uplink drops|Client TUN TX drops|Server TUN TX drops|Client peak RSS KiB|
|---|---|---:|---:|---:|---:|---:|---:|
|baseline|tcp_up|24.34|—|127.00|0.00|0.00|59172.00|
|baseline|tcp_down|—|1500.03|36565.00|0.00|132.00|77948.00|
|baseline|tcp_bidir|29.72|1467.71|38521.00|0.00|189.00|81536.00|
|candidate|tcp_up|1278.97|—|0.00|217.00|0.00|57912.00|
|candidate|tcp_down|—|1438.19|0.00|0.00|108.00|78536.00|
|candidate|tcp_bidir|519.27|1104.03|0.00|703.00|166.00|78708.00|

Candidate **0 native uplink/downlink drops in all9 windows**; baseline QueueFull every window, other reasons0. Upload recovered in all3 pairs; median paired ratio **52.91×**. Maximum observed worker RSS **88632KiB**, below existing512MiB product budget.1536MiB compiler limit/temporary build swap are separate.

|Repeat|Upload ratio|Download change %|Bidir sum change %|
|---|---:|---:|---:|
|1|53.41×|+1.43|+8.16|
|2|52.91×|-4.67|+12.81|
|3|51.47×|-7.11|-7.14|

Median paired bidir sum **+8.16%**, one repeat regressed. Bidir download shares throughput with recovered upload; not universal performance/capacity PASS, no exclusions. Kernel TUN losses persist; specific native QueueFull addressed, not an exact kernel FULL_RING/ACK event diagnosis.

After unit drain, exact UUID/inode-owned swap removed, original inventory restored. Owned compiler target/registry/source/root and runtime processes/namespaces/root/remote archives SHA-verified/removed; services/stagedCLI/full host/source gates PASS. Independent build/runtime qualification;80 raw JSON/12logs qualified first attempt.

Legacy bonded/one-carrier checks and native consumer refresh/qualification remain before source adoption. F030 not marked finally FIXED; defaults unchanged. Q36 IN_PROGRESS,33/37(89.2%),NO_PUSH_DEPLOY_RELEASE.


## Q36-F030: one-carrier compatibility and unresolved legacy

**PARTIAL35_QUALIFIED; source/native adoption pending.**12actualsessions/36attempted eight-second windows: **35admitted measurements and1iperf3 role-invalid bidir window**. Required recordizer/one carrier and off/legacy/two carriers, TCPraw P1 up/down/bidir,outer/innerIPv4,original MTU/qdisc/queues/affinity,unlimited producer/no omit. Server exactbaseline52511cebd43a throughout, client baseline or previously compiled candidate524443061b95; no rebuild/default change. Prior18fixed2mux windows not pooled.

One carrier:3fullpairs,0native uplink/downlink drops in all18baseline/candidate windows. Upload median1447.89→1414.29Mbps;median paired upload−1.26%,download+4.55%,bidir sum+2.34%. Preserved behavior in tested cases, not universal speed equivalence; kernel TUN losses remain.

Legacy:3upload/download pairs. Upload median435.57→574.22Mbps,paired+32.27%;download736.23→687.54Mbps,paired−6.63%. QueueFull persists in all8admitted candidate windows;Oversize/PoolExhausted/NoCarrier0. Candidate bidir has only2admitted repeats:individual diagnostic values retained, no three-sample statistical qualification for that cell. Legacy16602B/252slots yields83width2queue entries versus21baseline/682mux atMTU1400. Maximum accepted legacy bound unchanged. Executed128burst unit is mux only; capacity arithmetic does not diagnose every runtime drop.

Table:N3medians;N2two separate diagnostic values divided byslash,no statistics.

|Condition|Client|Phase|N|Up Mbps|Down Mbps|Native uplink drops|Client TUN TX drops|Server TUN TX drops|
|---|---|---|---:|---:|---:|---:|---:|---:|
|required/carriers-1|baseline|tcp_up|3|1447.89|—|0.00|174.00|0.00|
|required/carriers-1|baseline|tcp_down|3|—|1463.53|0.00|0.00|186.00|
|required/carriers-1|baseline|tcp_bidir|3|423.96|1160.85|0.00|645.00|271.00|
|required/carriers-1|candidate|tcp_up|3|1414.29|—|0.00|202.00|0.00|
|required/carriers-1|candidate|tcp_down|3|—|1549.98|0.00|0.00|442.00|
|required/carriers-1|candidate|tcp_bidir|3|457.97|1185.90|0.00|464.00|321.00|
|off/carriers-2|baseline|tcp_up|3|435.57|—|5059.00|0.00|0.00|
|off/carriers-2|baseline|tcp_down|3|—|736.23|18448.00|0.00|0.00|
|off/carriers-2|baseline|tcp_bidir|3|36.27|657.31|22031.00|0.00|0.00|
|off/carriers-2|candidate|tcp_up|3|574.22|—|2706.00|0.00|0.00|
|off/carriers-2|candidate|tcp_down|3|—|687.54|6350.00|0.00|0.00|
|off/carriers-2|candidate|tcp_bidir|2|115.06/118.70|622.61/633.70|6893.00/8282.00|0.00/0.00|0.00/0.00|

|Condition|Repeat|Upload change %|Download change %|Bidir sum change %|
|---|---:|---:|---:|---:|
|required/carriers-1|1|-1.26|+4.55|+9.64|
|required/carriers-1|2|-4.78|+2.01|+0.40|
|required/carriers-1|3|-0.35|+8.45|+2.34|
|off/carriers-2|1|+7.21|-16.26|+7.38|
|off/carriers-2|2|+32.27|-6.61|+6.70|
|off/carriers-2|3|+35.09|-6.63|role-invalid|

Original900scontroller budget ended after11complete sessions/33windows;TimeoutError retained. Only missinglegacycandidate session executed:upload/down completed,bidir bothreceivers0. RawJSON plus archivedserver iperf log proveTX/TX onclientport33208 andRX/RX on33220;all11priorbidir endpoint roles match correctly. [iperf3 3.18 server](https://github.com/esnet/iperf/blob/3.18/src/iperf_server_api.c#L762-L769) assigns roles byaccept order;[client](https://github.com/esnet/iperf/blob/3.18/src/iperf_client_api.c#L324-L330) createsTX thenRX. This explains the invalid measurement; it does not establish a Qeli stall. OriginalFAIL retained,no0speed imputation,no retry-until-success.

r1-m1-c/upload finalsender1417281536B versus1sintervalsum1417150464B:131072Bdifference,oneconfiguredblock;retransmits282match. Both separatehorizons retained;deliveredreceiver summary unchanged. Initial exactbytesum assertion refusal and subsequent population(N2) refusal preserved. NoN2statistics computed;existing3sampleminimum unchanged. Allother senderbyte/retransmitsums exact.

Independent qualification35windows/157rawJSON/24logs andboth originalexecution receipts.82valid-window native socket snapshots stable/mirrored;failedbidir lacksfullafter-boundary snapshot,no suchPASS claimed. MaxworkerRSS 95304KiB below512MiB. Structuralhoststate matches acrossstages;both fullhost/firewall/routes/resolver/services/stagedCLI/source gatesPASS,ownedprocesses/namespaces/roots/remotearchives SHAverified/removed. BothruntimeFAILs andfailedrawdata retained;workingclientmod/nativecopies unchanged.

Next:address instrument direction-role pairing before furtherbidir measurements,remaining legacyQueueFull,actualnativeconsumer refresh/qualification duringpatchadoption. Q36IN_PROGRESS,F030not finallyFIXED,33/37(89.2%),NO_PUSH_DEPLOY_RELEASE.

## TCP-bidir stream-role validation added to the harness

New TCP-bidir runs request `--get-server-output`. The shared parser matches both endpoints by IP/port and requires opposite TX/RX roles and a complete `2 × P` connection map. TX/TX, RX/RX, missing or ambiguous evidence rejects admission; raw iperf, counters and both process samplers are saved before validation. Socket accept order no longer serves as direction evidence. Server JSON/text and IPv4-mapped IPv6 normalization are supported.

Offline reports repeat this check for new results declaring `tcp_bidir_stream_roles_required=true`; this policy is part of run comparability. Historical records without server output retain their format and cannot be pooled with the new policy. Whenever server output is present, its roles are checked even on an old record. UDP-bidir and one-way admission are unchanged.

**86 tests PASS and one independent live TCPraw P1 bidir collector check on .10 PASS.** iperf3.18 returned server output text: clientport44654 TX ↔ server RX and clientport44662 RX ↔ server TX. Both receiver summaries were admitted; 63.26/695.34Mbps are instrumentation diagnostics, not a new statistical population or replacement third legacy sample. Historical TimeoutError/role-invalid FAIL and N2 remain. Exact PID/CLI SHA, stable mirrored outer sockets, counters and process samplers verified. Fullhost/firewall/routes/resolver/services/stagedCLI restored; owned namespaces/root/remote archive removed after SHA verification of the local archive.

Only collector/parser/report and tests changed: Rust/INI/Cargo, all916 build/config inputs and native digest remain unchanged. This is separate executable-tool qualification, not a Markdown-only transfer. Original20 Linux certification executions retain their actual dates/raw/CLI and do not use the changed bidir collector. F030 core patch/native consumer copies remain unadopted; legacy QueueFull remains. Q36 IN_PROGRESS,33/37(89.2%),NO_PUSH_DEPLOY_RELEASE.

## Qualified slot-sizing candidate adopted into source

Adopted the exact actually built and tested `client/mod.rs`: SHA256 `02690ef6b1666f87a3e70a4a2965bee0d9ebe154a0ed5cd4c7bd32124f922dc5`. Slot size follows the existing accepted packet bound: negotiated mux MTU+64, legacy MAX_TUNNEL_MTU16602; shared4MiB budget, queue reserves and flow pinning remain unchanged. Packets exceeding the existing wire bound do not become admissible; INI/defaults/ABI do not change.

Previously executed15Rust tests/4feature checks,18fixed2mux windows and35admitted compatibility windows bind this exact file but do not establish whole-current-build byte identity. Fresh current-tree compilation, native A/B/exports/consumer checks and Linux certification applicability are **pending**. Existing native binaries/certification retain their original SHA/source and must not be presented as rebuilt for this patch. Remaining legacy QueueFull/download regression and all historical FAIL receipts remain visible. Q36 IN_PROGRESS,33/37(89.2%),NO_PUSH_DEPLOY_RELEASE.

## Exact slot patch native consumer refresh completed locally

Current tree compiled:4strictfeature checks,15Rust tests and releaseCLI SHA256 `6f617bdcab5aecf3068ffa9bd6a11006aecbfead09cbb657d60671f0340b7363`. Independent925input qualification retains these completed stages despite a later native-controller failure.

Windows/macOS passA retained. During owned target-cache deletion, `du` encountered a disappearing entry and aborted the first controller; compilation itself passed. OriginalFAIL/archive retained. New size sampler tolerates disappearing entries only, retains6GiB budget and propagates other errors;4Linux regressions PASS. First B attempt stopped before compilation because the fixture expected final rather than sampled directory metadata; fixture-only correction and refusal retained. Only missing passB compiled with independent source/cache/target after verified old-root removal. A/B bytes match. Same source/registry path with a new owned directory object preserves embedded Cargo paths; recipe/flags/epoch unchanged.

Windows/macOS:33CAPIexports onWindows and eachmacOSslice,22actualWindowsABI/localhost-cancellation checks PASS. Android:ARM64/x64 independentA/B,33CAPI+21transportJNI+1ConfigJNI=55exports/ABI PASS.3freshAPKs built/verified; R8/signatures/native payload PASS,23JNI+2packagedINI onAPI34 emulator PASS.201unchanged JVM tests/lint verified from original XML/archive;5Kotlin/XML files differ onlyCRLF/LF and README is documentation; no new JVM executions claimed. OriginalAVD/SDK/userdata/services/host/source preserved; ownswap/roots/remote archives/localAPKprojection SHAarchived/removed. Original archives retained.

Four canonical libraries and four consumer copies refreshed locally; checksum/provenance PASS, native source digest `0dfac80b50d2bdd1964817dee8ac87f7311cde63bf61efade7a40abd7e32b635`. PhysicalMac/iOS/router/WindowsnetworkVM excluded by accepted scope. StrictLinuxcertification still identifies the old tree: **fresh20-case matrix required for the Rust delta**; original results not relabeled. LegacyQueueFull/download regression and role-invalid/TimeoutError remain visible; no universalzero-loss/performancePASS. Q36 IN_PROGRESS,33/37(89.2%),NO_PUSH_DEPLOY_RELEASE.

## 14 October: fresh current-core Linux matrix completed

All 20 required criteria passed with current CLI `6f617bdcab5aecf3068ffa9bd6a11006aecbfead09cbb657d60671f0340b7363`: 19 independent executions, 409 assertions and a leak criterion derived from passing full/split cases. Coverage includes TCP, UDP fake-TLS, QUIC, outer/inner IPv4/IPv6, split/full, actual systemd-resolved A/AAAA and forced TCP, MTU 1280/DATA_FRAG/PTB, TAP NDP/RA, bidirectional legacy 0.7.16 compatibility, and 100 TCP plus 100 QUIC network flips. This is a bounded functional matrix; no fresh eight-hour endurance execution is claimed.

This is a new execution after the Rust patch, not old evidence transferred across a core delta. Original execution commit `55eb810865e290e427f6cd2b5838e4278aa304bf`, actual CLI build `e0ce217b630d9dfd7db5f59936bb41e6d68eb69d`; their build inputs match. Raw logs/archives verified by SHA. Owned root/archive/unit removed; host/firewall/routes/DNS/accounts, production processes and staged CLI preserved. Certification uses fresh results and retains the superseded manifest separately. Subsequent applicability is restricted to four Markdown report/plan files; original execution time and identity remain unchanged.

Required functional qualification debt for adopted Q36-F030 is closed alongside earlier native A/B, ABI/JNI and package checks. Legacy QueueFull, the 6.63% legacy download decrease, one bidirectional outlier and TUN losses remain visible; no universal zero-loss/performance PASS. Q36 remains IN_PROGRESS, as do Q29/Q31/Q34. Overall 33/37 (89.2%); no push/deploy/release.

## 14 October: fresh current-product legacy/mux cohort

Current source commit `3a7e25dcdf1af6f3a761dfa883851df807535fb0` ran six fresh sessions / 18 eight-second windows: TCP raw P1, fixed two carriers, legacy `off` versus recordizer `required`, three paired upload/download/bidir repeats. Repeat two reverses protocol and direction order. Every server and client uses the same already-built CLI `6f617bdcab5aecf3068ffa9bd6a11006aecbfead09cbb657d60671f0340b7363`, actual build commit `e0ce217b630d9dfd7db5f59936bb41e6d68eb69d`. The full `qeli/` Git tree matches that build commit; 919 current inputs and exact measurement helpers are frozen. No compilation or MTU/default/queue/qdisc/pacing change.

Medians of three sessions; native/TUN drops below sum counter deltas across boundaries around nine windows per protocol, not individual iperf packet correspondence.

|Protocol|Upload Mbps|Download Mbps|Bidir up/down Mbps|Native uplink/downlink drops|TUN TX drops client/server|
|---|---:|---:|---:|---:|---:|
|Legacy|583.38|709.27|105.73/666.36|43676/0|0/0|
|Recordizer|1214.62|1448.40|490.63/1166.87|0/0|1723/1684|

Recordizer relative to legacy: median paired upload change +108.20%, download +104.21%, bidir sum +113.10%. All three samples, min/max/spread, direction tradeoffs, CPU/RSS/steal and counters are retained. Maximum observed worker RSS is 95020 KiB. These are finite same-VM sibling-namespace windows, not WAN/physical capacity or a general performance PASS.

All observed legacy native uplink losses (43676) are QueueFull; other uplink reasons and native downlink losses are zero. Recordizer has zero native uplink/downlink drops in all nine windows, while TUN losses remain in the table. This confirms remaining legacy queue pressure on the current product, not a pool leak or an all-loss fix. The accepted legacy packet bound, queue and pinned-flow/nonblocking-drop contract were unchanged.

All six bidir windows were admitted with opposite TX/RX roles and mirrored endpoint tuples in client/server output. Two real native-PID carrier sockets were retained at 42 boundaries; VPN generation stayed unchanged within each session. Raw iperf summaries, sender interval bytes/retransmits and TCP_INFO RTT were checked; a reverse-only client is not a sender and its sender RTT remains unknown. Final/interval byte horizon differences are retained without editing raw JSON. Counters show observed loss boundaries, not the exact cause of every kernel drop.

This is a fresh current-product population. It is not appended to the historical candidate/baseline set, does not replace the role-invalid window, and is not a before/after F030 effect measurement. Historical legacy download -6.63% and N2 bidir retain their own binary identities and dates. First preparation refused before lab access because whole-worker and independently qualified CLI statuses were confused; refusal retained. A subsequent interactive controller disappeared before completion: one window is retained separately, its resources archived and cleaned, host/staged CLI restored. The final controller ran hidden outside the interactive job; the password was passed only via stdin.

Actual runtime was 514.06 seconds with a 900-second measurement budget. Full host/firewall/routes/resolver/services and staged CLI were preserved. Owned processes/namespaces/root and remote archive were removed after inode/PID/SHA/full archive inventory checks. Original full archive and raw are retained. Current legacy/mux measurement debt is closed within this scope; no all-loss fix or whole-Q36 closure is claimed. Q29/Q31/Q34/Q36 remain open, 33/37 (89.2%), NO_PUSH_DEPLOY_RELEASE. Evidence: `release/certification/evidence/q36-current-legacy-mux-20261014.json`.

## 14 October: legacy queue contract and counter boundaries

A separate offline component check ran on Linux x86_64 with Rust 1.97.0: exact bodies of `bonded_uplink_pool_layout`, `prepare_bonded_uplink`, `queue_tcp_uplink`, carrier selection and related types were extracted from current source with SHA bindings; BufferPool and UplinkCounters were included byte for byte. Actual Tokio mpsc/Semaphore and PooledBuffer are used. The harness has a separate Vec alias for the unused Tun variant and only the RuntimeConfig structure; every probe packet uses Bonded. This checks these components, not the full Qeli crate or TUN/driver/crypto/wire runtime.

11 positive tests PASS: eight existing BufferPool tests, one UplinkCounters test and two new legacy probes. A 4 MiB storage budget / 16,602 bytes yields 252 slots; reserving capacity for two carriers/candidate leaves 83 entries in a pinned queue. A controlled undrained 128-packet burst of 1400/40-byte packets preserves the first 83 in FIFO and records exactly 45 QueueFull; PoolExhausted/Oversize/NoCarrier remain zero and the other carrier is not used to repin the flow. After draining, all 252 slots are available again. This establishes no pool-slot loss on that tested path, not absence of every race or loss under arbitrary load.

A 16,602-byte payload fits PooledBuffer byte for byte; 16,603 bytes is refused as Oversize with its slot returned. An intentionally reduced 2 KiB legacy slot fails this test, so that shortcut cannot fix QueueFull while retaining the accepted contract. A second negative control intentionally forgets the buffer on Full: the test detects missing slots after draining. Restored source passes both probes again. Negative changes existed only in owned probe projections; product code was unchanged.

Clarification of prior measurements: `tcp_uplink_drops` counts bounded hand-off losses at the TCP queue (pool, copy size, Full, no carrier). It does not count every failure after dequeue, such as recordizer rejection or encryption/write failure. A zero value therefore means zero recorded losses in that counter's domain, not complete delivery through the native data plane. Delivery, TUN drops and retransmits are checked separately. Actual values and full raw in the current legacy/mux report are retained; no additional zeros or performance PASS were inserted.

The audit harness used 21 cached crate archives with SHA matching canonical Cargo.lock; 12 registry packages were actually compiled. No network, jobs=1, private vendor/Cargo home/target, MemoryMax 768 MiB/SwapMax 256 MiB/CPUQuota 100%/900-second runtime. Successful 11 tests and size control were not replayed after the CRLF substitution refusal in the second control: only the missing leak control and two restored probes ran. The original overall FAIL remains; successful substeps from two source/lock/toolchain-identical packets were admitted separately. An earlier preflight refused before root creation because another host's staged path was absent; actual absence was checked before/after on this host.

Owned project/target/vendor/cache/unit/remote archives were removed after SHA/inode/process guards; host/services/firewall/routes/resolver and global crate archives were preserved. Full local source/log archives remain. No new native/CLI build or VPN runtime ran; current certification does not relabel them. Legacy burst/accepted-bound verification debt is closed within this component scope. Remaining legacy pressure, TUN loss and historical direction tradeoffs are not claimed fixed. Q36 IN_PROGRESS; 33/37 (89.2%), NO_PUSH_DEPLOY_RELEASE. Evidence: `release/certification/evidence/q36-legacy-contract-20261014.json`.

## 14 October: F031/F032 — retired scenarios and phase independence

Q36-F031: six perf_* scratch scenarios had no effective admission guard. No executable callers in tracked code/docs/workflows; references are historical SHA inventories only. All six original files with fixture-only environment reached the Paramiko constructor in a controlled probe; a substituted constructor raised BaseException before SSH/sockets. Four scripts contained host-wide process/firewall/network orchestration; two performed old production diagnostics. No real connections or commands ran, and embedded profile/credential values were not logged into evidence.

Removed718 legacy implementation lines from perf_allmodes_prod, perf_ab_p3b1, perf_netns_cpu, perf_modes_test, perf_prod_live_monitor and perf_facts. Only three-line explicit RETIRED refusals pointing at benchmark_isolated_lab.py remain, so an old path cannot silently become a different measurement. Configured credentials or the old --allow-host-wide-benchmark flag cannot revive them. The active isolated runner and guarded benchmark.py/bench_bonding.py are unchanged.

Q36-F032: aggregate checked whole raw-population equality, allowing one copied phase among otherwise different phases to count as an independent repetition. The negative baseline admitted tcp_up n3/median200 from two distinct raw measurements after copying the first run's phase to the second and matching its summary. The fix checks SHA for each identical mode/phase separately and refuses duplicates in AGGREGATED and MEASUREMENTS_ONLY. Equal Mbps from distinct raw files remain valid; outliers and original results are not discarded.

Windows and Linux each passed22 methods:3 methods/14 subprocess invocations/6 imports cover retired/guarded scenarios;19 report methods cover individual TCP/UDP phase copies and equal-rate distinct raws. Old false admission and blocked SSH constructor paths remain baseline facts, not rewritten PASS. The exact new guard additionally checked SHA18 raw files from six already qualified current legacy/mux sessions: three repetitions each off/required, no repeated same-phase raw. An in-memory copy of one actual phase identity was refused. No full aggregate replay of these current diagnostic sessions; throughput/statistics/raw bytes are unchanged.

The prior runtime's imported population and sample_policy functions are byte-identical; only aggregate admission changed. No new Rust/native/CLI/VPN build or load measurement. Both owned lab roots/archives were removed after SHA/process/inode guards; host/service/firewall/routes/resolver preserved. This methodology defect and dead-code perimeter are closed within this scope; legacy queue pressure, TUN losses/TCP asymmetry stage and whole Q36 acceptance remain open.33/37(89.2%),NO_PUSH_DEPLOY_RELEASE. Evidence: release/certification/evidence/q36-perf-methodology-20261014.json. Raw: q36-retired-perf-scenarios-20261014 and q36-benchmark-phase-independence-20261014.
