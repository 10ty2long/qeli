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
