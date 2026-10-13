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
