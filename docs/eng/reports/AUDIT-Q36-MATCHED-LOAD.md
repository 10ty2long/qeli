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
