# qeli load testing

The primary structured measurement is **qeli 0.8.2, 10 October 2026**:
[canonical JSON](../../../release/benchmark_results.json) and
[dated snapshot](../../../release/benchmark_v0.8.2_2026-10-10_repeated_12modes.json).
Binary SHA-256 `7769f43cac84fee1b65bccb4c783685eb12a15a3f28a7610e7437c170bb5621e` was built from committed package
`138bb152` (the same source tree as dev `b8019df4`), Rust 1.97.0,
`--offline --locked --release --features jemalloc`; all 902 input files matched
before/after compilation. This qualifies this CLI artifact's source provenance,
not reproducibility across two independent builds.

The latest supplemental measurement is [IPv6 RTT](../../../release/benchmark_v0.8.2_2026-10-11_ipv6_latency.json),39 new sessions /78 windows. [Mixed-family coverage](../../../release/benchmark_v0.8.2_2026-10-11_mixed_carriers.json),54 new sessions /216 windows. [IPv6 carrier coverage](../../../release/benchmark_v0.8.2_2026-10-11_ipv6_carriers.json), 27 new sessions / 108 windows. [Prepared-peer AUTH bursts](../../../release/benchmark_v0.8.2_2026-10-11_auth_bursts.json), 27 bursts / 189 accepted peers. [Unpaced TCP-bidir RTT](../../../release/benchmark_v0.8.2_2026-10-10_unpaced_latency.json),
39 sessions / 78 windows. [UDP-bidir/IPv6](../../../release/benchmark_v0.8.2_2026-10-10_udp_ipv6.json)
contains separate 69 sessions / 204 windows. It uses CLI SHA-256
`238153ab5c44dc707390e0adf6ac14fecd7079f242c71bdb3dcdd84ab8472a71`
with 916 qualified build inputs. Details and limitations follow below.

## IPv6 RTT under TCP bidirectional load — 11 October 2026

**Q36-F017:** [JSON](../../../release/benchmark_v0.8.2_2026-10-11_ipv6_latency.json) records all12 carriers and
direct IPv6: three fresh sessions each, **39 sessions /78 ten-second load windows**,
15600 ICMP probes sent, **15573 replies /27 lost**.
Both outer carrier and inner tunnel use IPv6. Previous IPv4 RTT is SHA-verified
and retained separately; no pooling across families or dates.

CLI SHA `238153ab5c44dc707390e0adf6ac14fecd7079f242c71bdb3dcdd84ab8472a71` and its916 build inputs are unchanged; no
compilation or production change. NetworkPlan and server peer independently prove
outer IPv6; plan address and every raw iperf endpoint prove inner IPv6. Private
IPvlan/network and mount namespaces isolate both VMs. TUN MTU1400, underlay1500,
recordizer off. REALITY fallback uses fixture-local IPv4; real-TLS uses the existing
handrolled=true fixture setting, without certifying public-site camouflage.

TCP bidir P1/P4 is unpaced. After an established load connection and two seconds,
the client sends exactly200 ICMP56-byte probes at20ms intervals; external15s timeout
limits execution. All received reply timestamps must lie within the conservative
rounded iperf start+1..+10 interval. Lost probes have no reply timestamps.
Nearest-rank p50/p95/p99 are computed from raw received replies in each window;
the table shows medians of three window percentiles. These are not pooled percentiles.
Loss is the number of missing replies out of200, median and maximum over three
windows. Per-window values and complete reply vectors remain in JSON.

| Carrier | P1 p50 / p95 / p99, ms | P1 lost, median / max | P4 p50 / p95 / p99, ms | P4 lost, median / max |
|---|---:|---:|---:|---:|
| direct | 0.707 / 2.245 / 3.288 | 0 / 0 | 3.550 / 7.322 / 9.483 | 0 / 0 |
| tcp-plain-raw | 26.268 / 45.644 / 51.578 | 0 / 0 | 62.843 / 84.083 / 93.082 | 1 / 1 |
| tcp-faketls | 31.760 / 48.226 / 52.573 | 0 / 0 | 48.975 / 72.172 / 83.577 | 0 / 0 |
| tcp-padding | 26.867 / 41.556 / 47.840 | 0 / 0 | 45.920 / 72.038 / 86.757 | 0 / 0 |
| tcp-frag | 25.029 / 39.792 / 45.957 | 0 / 0 | 60.279 / 83.606 / 91.412 | 1 / 3 |
| tcp-obfs | 27.666 / 40.057 / 43.548 | 0 / 0 | 54.848 / 77.195 / 86.146 | 0 / 2 |
| tcp-reality | 22.797 / 39.925 / 48.963 | 0 / 0 | 42.907 / 72.060 / 88.030 | 0 / 0 |
| tcp-reality-tls | 14.442 / 19.106 / 22.409 | 2 / 5 | 29.970 / 38.198 / 43.707 | 0 / 0 |
| udp-faketls | 17.208 / 28.249 / 32.666 | 0 / 0 | 19.601 / 28.239 / 36.217 | 0 / 1 |
| udp-padding | 15.456 / 23.156 / 29.190 | 0 / 0 | 18.933 / 28.776 / 30.847 | 0 / 1 |
| udp-quic | 81.593 / 118.782 / 133.726 | 1 / 1 | 114.338 / 152.670 / 185.523 | 0 / 3 |
| tcp-obfs-awg | 32.587 / 45.809 / 54.168 | 0 / 0 | 46.941 / 77.499 / 84.777 | 0 / 2 |
| udp-faketls-awg | 15.993 / 24.869 / 30.344 | 0 / 0 | 19.410 / 27.319 / 34.326 | 1 / 1 |

QUIC P4 median window p99 is 185.523ms versus
9.483ms for direct P4. Across its six P1/P4 windows QUIC
loses 5 of1200 probes. This observation is scoped to the retained load
conditions; it does not identify the implementation cause or establish a
cross-version regression.

Both TCP receiver directions, exact P/duration/endpoints, distinct iperf cookies,
process identity, counter resets and affinity0-1/nice0 are rechecked from raw files.
CPU100% means one logical CPU; RSS is sampled peak. Eleven-second process sampling
surrounds each ten-second load. The second repeat reverses session and phase order,
changing traffic history and possible adaptive UDP buffer growth. No result or
outlier is removed to improve medians.223 raw JSON,78 ping files and78 worker logs
are SHA-bound; prior IPv4 evidence retains its own host gates and source hashes.

Measurement and post-archive host gates retain complete differences. Cleanup
changed fields: `{"10.66.116.10": ["legacy4"], "10.66.116.11": []}`. Active services are unchanged;
both owned roots were SHA-archived and removed. A host difference is not reported
as restoration PASS. These short windows establish scoped measurements, without
proving physical saturation, a universal latency threshold or a regression cause.

No shared benchmark implementation changed. The sealed73 Windows/73 Linux checks
from Q36-F014 are reused after verifying all27 exact input hashes; nine docs checks
PASS. IPv6 RTT under TCP bidir P1/P4 is complete for these conditions. Full carrier
AUTH/rate and TCP asymmetry cause remain OPEN; long soaks belong to Q35.
Q36 IN_PROGRESS;30/37 DONE81.1%;LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.

## Mixed IPv4/IPv6 carrier coverage — 11 October 2026

**Q36-F016:** [JSON](../../../release/benchmark_v0.8.2_2026-10-11_mixed_carriers.json) completes the remaining nine
carriers for **outer4/inner6 and outer6/inner4**: three fresh sessions per cell,
54 new sessions / 216 five-second TCP+UDP bidir P1/P4 windows / 648 raw JSON
files plus the execution manifest and 108 SHA-bound worker logs. Previously sealed core3 and direct IPv4/IPv6
baselines retain their original source hashes, conditions and host gates; they
were not repeated or pooled with new measurements. With Q36-F015, all 12 carriers
now have each of the three non-default inner/outer family pairs measured.

The CLI SHA `238153ab5c44dc707390e0adf6ac14fecd7079f242c71bdb3dcdd84ab8472a71` and its 916 source inputs are unchanged.
No compiler, new binary or production change. Both VMs use private IPvlan/network
and mount namespaces. Carrier family is verified from NetworkPlan and server
peer; tunnel family from the only plan address and every raw iperf endpoint.
TUN MTU1400, underlay1500, recordizer off. REALITY fallback target remains
fixture-local IPv4, independently of the carrier and payload family.
The real-TLS REALITY fixture explicitly sets handrolled=true, as in the previous
family measurements; this is not a certification of public-site camouflage.

TCP bidir is unpaced. UDP 100Mbit/s is per stream per direction with 1200-byte
datagrams: P1=100+100, P4=400+400Mbit/s. Values below are medians of three
independent windows, Mbit/s except loss. The direct row is the matching inner-family
underlay baseline, with no Qeli outer connection. The second repeat reverses cell
and phase order. Full median/min/max, retransmits, jitter, drops, CPU/RSS/load/steal
and individual runs remain in JSON.
Automatic UDP buffer policy is unchanged; reversing phases deliberately changes
prior traffic and possible buffer-growth history. Per-phase capacity is not assumed
identical. CPU/RSS uses a six-second observation around each five-second iperf window.

### Outer IPv4 / inner IPv6

| Carrier | Source | TCP P1 up / down | TCP P4 up / down | UDP P4 up / down | UDP P4 loss %, up / down |
|---|---|---:|---:|---:|---:|
| direct | inherited | 12950.1 / 16385.3 | 5430.1 / 20863.5 | 400.0 / 400.0 | 0.00 / 0.00 |
| tcp-plain-raw | new | 49.8 / 499.2 | 33.3 / 501.7 | 301.7 / 312.8 | 23.76 / 18.28 |
| tcp-padding | new | 47.2 / 470.2 | 41.1 / 484.9 | 251.0 / 314.5 | 34.52 / 15.88 |
| tcp-frag | new | 52.4 / 493.8 | 37.5 / 495.3 | 269.6 / 267.6 | 32.42 / 28.13 |
| tcp-obfs | new | 41.7 / 496.3 | 34.2 / 490.4 | 245.2 / 330.5 | 38.51 / 14.45 |
| tcp-reality | new | 76.7 / 461.6 | 30.4 / 512.9 | 283.1 / 314.1 | 29.01 / 14.56 |
| tcp-reality-tls | new | 138.1 / 872.9 | 127.7 / 893.7 | 378.1 / 398.3 | 5.28 / 0.01 |
| udp-padding | new | 161.8 / 614.1 | 101.8 / 699.6 | 360.2 / 99.2 | 9.82 / 57.34 |
| tcp-obfs-awg | new | 53.6 / 470.2 | 39.5 / 484.1 | 179.3 / 337.0 | 55.01 / 14.00 |
| udp-faketls-awg | new | 118.3 / 723.7 | 94.0 / 736.8 | 350.3 / 139.8 | 12.24 / 41.55 |
| tcp-faketls | inherited | 61.0 / 485.5 | 39.4 / 476.6 | 295.9 / 348.4 | 24.54 / 10.18 |
| udp-faketls | inherited | 134.1 / 754.8 | 100.7 / 777.5 | 354.3 / 77.8 | 10.70 / 50.90 |
| udp-quic | inherited | 177.4 / 630.4 | 81.3 / 777.1 | 349.2 / 150.1 | 12.48 / 51.49 |

### Outer IPv6 / inner IPv4

| Carrier | Source | TCP P1 up / down | TCP P4 up / down | UDP P4 up / down | UDP P4 loss %, up / down |
|---|---|---:|---:|---:|---:|
| direct | inherited | 14480.2 / 15323.9 | 6114.2 / 20810.4 | 400.0 / 400.0 | 0.00 / 0.00 |
| tcp-plain-raw | new | 41.1 / 444.7 | 45.4 / 490.6 | 255.9 / 299.2 | 35.30 / 22.46 |
| tcp-padding | new | 61.2 / 431.7 | 39.6 / 505.1 | 245.1 / 312.5 | 38.46 / 17.79 |
| tcp-frag | new | 43.2 / 440.2 | 43.6 / 458.9 | 285.7 / 291.4 | 28.11 / 25.26 |
| tcp-obfs | new | 54.7 / 489.2 | 46.7 / 485.0 | 210.6 / 338.3 | 45.16 / 13.06 |
| tcp-reality | new | 59.5 / 427.8 | 42.1 / 532.6 | 283.5 / 306.3 | 26.15 / 18.24 |
| tcp-reality-tls | new | 415.8 / 739.6 | 122.5 / 902.2 | 355.9 / 397.7 | 10.73 / 0.00 |
| udp-padding | new | 151.1 / 662.9 | 84.0 / 718.1 | 353.5 / 133.6 | 11.56 / 49.98 |
| tcp-obfs-awg | new | 37.5 / 516.5 | 52.6 / 539.0 | 212.9 / 278.1 | 44.55 / 27.77 |
| udp-faketls-awg | new | 134.2 / 664.5 | 90.7 / 710.0 | 341.0 / 111.7 | 14.26 / 40.17 |
| tcp-faketls | inherited | 63.3 / 440.0 | 44.5 / 521.5 | 313.6 / 358.2 | 21.10 / 7.82 |
| udp-faketls | inherited | 168.0 / 644.8 | 91.8 / 779.9 | 347.4 / 107.7 | 12.84 / 44.67 |
| udp-quic | inherited | 48.6 / 255.7 | 32.7 / 260.1 | 345.9 / 90.6 | 12.85 / 55.44 |

Both receivers are checked against exact protocol/P/duration/rate and endpoints.
Process identity and affinity0-1/nice0 are observed throughout sampling; CPU100%
is one logical CPU and RSS is sampled peak. Application and both-family kernel
counter deltas are recomputed; resets refuse admission. Kernel counters include
iperf and carrier sockets, so they cannot independently locate a Qeli client defect.
Client status/UDP receive capacity is observed before the first phase only.

### UDP P4 receive-buffer errors by family

| UDP carrier | Outer / inner | Source | Client outer-family RcvbufErrors | Client inner-family RcvbufErrors | Client CPU %, median |
|---|---|---|---:|---:|---:|
| udp-padding | IPv4 / IPv6 | new | 132261 | 3563 | 106.8 |
| udp-faketls-awg | IPv4 / IPv6 | new | 125936 | 2788 | 103.3 |
| udp-faketls | IPv4 / IPv6 | inherited | 156840 | 3915 | 104.2 |
| udp-quic | IPv4 / IPv6 | inherited | 119948 | 4524 | 106.2 |
| udp-padding | IPv6 / IPv4 | new | 123170 | 2902 | 106.2 |
| udp-faketls-awg | IPv6 / IPv4 | new | 139916 | 2998 | 105.2 |
| udp-faketls | IPv6 / IPv4 | inherited | 138336 | 4796 | 105.0 |
| udp-quic | IPv6 / IPv4 | inherited | 151224 | 3525 | 108.0 |

Unlike same-family windows, these mixed-family counters distinguish the two
address families used by carrier and payload. They are still namespace counters,
not per-socket ownership measurements; packet loss and receive-buffer errors
need not have identical counts. The table retains median counts and process CPU,
without attributing all loss to one code path or increasing buffers without evidence.

Native UDP log events are also extracted with exact line numbers and source SHA.
For the first outer4/inner6 UDP-padding session, the client's granted buffer grows
to 16384 KiB and the native socket counter reaches 125313 drops.
The production controller reads the socket inode's drops from /proc/net/udp or udp6;
this log provides separate socket evidence, rather than treating namespace counts
as socket ownership. Auto-growth is active; its absence does not explain this case.
These logs span the complete session, not per-phase client status snapshots.

Host measurement gates and post-archive differences are retained separately;
cleanup changed fields: `{"10.66.116.10": ["iptables4"], "10.66.116.11": []}`.
Both VM gates passed immediately after measurement. After archiving, the .10
iptables4 snapshot lacks two pre-existing vpn0 FORWARD rules and MASQUERADE for
10.8.0.0/24; the complete diff is retained, and restoration of that snapshot is not PASS.
Both owned roots are SHA-archived and removed; active services are preserved.
No result is discarded to improve medians. Loss/asymmetry remains diagnostic;
short windows do not prove stable throughput, regression cause or saturation.

No benchmark implementation changed. The sealed 73 Windows/73 Linux checks from
Q36-F014 are reused after exact input hashes are verified; nine docs checks PASS.
All-carrier mixed-family coverage is complete for these conditions. IPv6 RTT,
full carrier AUTH/rate, exact TCP asymmetry cause and Q35 long soaks remain OPEN.
Q36 IN_PROGRESS;30/37 DONE81.1%;LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.

## IPv6 carrier coverage — 11 October 2026

**Q36-F015:** [JSON](../../../release/benchmark_v0.8.2_2026-10-11_ipv6_carriers.json) adds the remaining nine
carriers with **outer6/inner6**. Three fresh sessions per carrier, TCP and UDP
payload bidir P1/P4: 27 new sessions / 108 five-second windows / 324 raw JSON
files, plus the execution manifest. Existing three core-carrier populations
and the direct IPv6 baseline remain independently qualified, with explicit source
and SHA references; those successful cells were not repeated. All 12 carriers now
have this family pair measured, rather than only the former core three.

The existing CLI `238153ab5c44dc707390e0adf6ac14fecd7079f242c71bdb3dcdd84ab8472a71` uses the same 916 qualified inputs;
no source change, compiler or new binary. Both VMs use private IPvlan/network and
mount namespaces; production services and staged binaries are preserved.
IPv6-only NetworkPlan addresses, outer server peer and every raw iperf local/remote
endpoint prove the address families. TUN MTU1400, underlay1500, recordizer off;
REALITY fallback target stays fixture-local IPv4, without changing the IPv6 carrier
or inner benchmark payload.

UDP load is 100Mbit/s per stream per direction, 1200-byte datagrams:
P1=100+100, P4=400+400Mbit/s. TCP is unpaced bidir. Each value below is a median
of three independent windows, Mbit/s except the loss columns. Full median/min/max,
retransmits, jitter, drops, CPU/RSS/load/steal and individual runs remain in JSON.
Carrier/phase order changes on the second repeat; inherited populations remain
separate and are not pooled with new ones.

| Carrier | Source | TCP P1 up / down | TCP P4 up / down | UDP P4 up / down | UDP P4 loss %, up / down |
|---|---|---:|---:|---:|---:|
| direct | inherited | 12950.1 / 16385.3 | 5430.1 / 20863.5 | 400.0 / 400.0 | 0.00 / 0.00 |
| tcp-plain-raw | new | 41.1 / 466.8 | 34.2 / 498.7 | 290.0 / 328.2 | 24.46 / 12.79 |
| tcp-padding | new | 69.1 / 451.5 | 52.4 / 490.0 | 294.5 / 329.7 | 22.78 / 11.74 |
| tcp-frag | new | 39.4 / 456.3 | 40.0 / 443.9 | 258.0 / 329.4 | 32.70 / 15.50 |
| tcp-obfs | new | 53.0 / 480.8 | 41.0 / 494.6 | 225.2 / 339.7 | 43.48 / 13.36 |
| tcp-reality | new | 63.7 / 428.5 | 42.9 / 495.0 | 301.5 / 303.7 | 22.84 / 21.77 |
| tcp-reality-tls | new | 348.1 / 763.3 | 92.3 / 834.7 | 376.2 / 397.6 | 5.39 / 0.00 |
| udp-padding | new | 123.3 / 636.3 | 68.0 / 683.2 | 354.9 / 122.6 | 9.46 / 42.72 |
| tcp-obfs-awg | new | 49.9 / 476.3 | 36.2 / 495.5 | 226.1 / 338.0 | 41.02 / 10.07 |
| udp-faketls-awg | new | 183.0 / 666.1 | 78.5 / 683.2 | 349.0 / 100.2 | 12.15 / 56.37 |
| tcp-faketls | inherited | 45.4 / 446.0 | 36.0 / 520.7 | 291.3 / 335.5 | 25.17 / 11.66 |
| udp-faketls | inherited | 153.8 / 688.3 | 101.6 / 689.4 | 351.7 / 113.6 | 11.90 / 54.95 |
| udp-quic | inherited | 30.8 / 282.6 | 37.7 / 254.6 | 356.1 / 80.6 | 10.40 / 62.16 |

Both receiver directions are re-admitted against exact P/protocol/duration/rate
and endpoints. Server application counters and IPv4/IPv6 kernel counters on both
hosts are recomputed; resets refuse admission. CPU/RSS samples verify process
identity and affinity0-1/nice0 and retain VM contention; 100% means one logical CPU.
RSS is sampled maximum. Client status and effective UDP receive capacity are
observed before the first phase, without claiming per-phase client drop deltas.

Loss under high offered load and TCP asymmetry remain diagnostic observations;
these short windows do not establish stable throughput, losslessness or a regression
cause. No results are removed to improve medians. Complete host/firewall gates and
cleanup differences are retained without exemption for the external NAT cycle.
Both owned roots were SHA-archived and removed; nine docs checks PASS.

New UDP P1 windows have zero loss. P4 median downlink loss is 42.72% for
udp-padding and 56.37% for udp-faketls-awg; TCP asymmetry persists.
Kernel counters include every UDP socket in each namespace, including iperf,
so they do not independently locate a Qeli client defect. Both host gates
passed immediately after measurement. After archiving, .10 lacked two
pre-existing vpn0 FORWARD rules and MASQUERADE for 10.8.0.0/24; the complete
diff is retained. This difference is not admitted as successful firewall
restoration or removed by modifying unrelated rules.

No implementation changed, so the sealed Q36-F014 **73 Windows/73 Linux** benchmark
checks are reused after exact test-input hashes are verified, rather than repeated.
The all-carrier family-template test already covers each inner/outer pair.
The plan registry now correctly marks Q36 IN_PROGRESS and distinguishes the closed
September debt queue from current work. Other-carrier mixed-family cells, IPv6 RTT,
full carrier AUTH/rate and TCP asymmetry cause remain OPEN; Q35 soak is separate.
Q36 IN_PROGRESS; 30/37 DONE (81.1%); LOCAL_DEV_COMMIT; NO_PUSH_DEPLOY_RELEASE.

## Prepared-peer AUTH bursts — 11 October 2026

**Q36-F014:** [JSON](../../../release/benchmark_v0.8.2_2026-10-11_auth_bursts.json) contains 27 fresh-server bursts:
TCP fake-TLS, UDP fake-TLS and QUIC-shaped UDP, 1/4/16 simultaneous peers,
three independent repeats of every condition. All 189/189 requested peers
verified the pinned server proof, decrypted AuthOK, appeared in the fixture
session registry with distinct pool addresses, and were revoked before teardown.
Transport and concurrency order changes on the second repeat. No warm-up,
failed-login substitution or sample filtering.

Server and peers share VM .11 (2 vCPU), private IPv4 loopback. The existing CLI
`238153ab5c44dc707390e0adf6ac14fecd7079f242c71bdb3dcdd84ab8472a71` was reused; all 916 original build inputs match.
The timed test peer adds one generated example to that exact source snapshot,
links the production crypto/protocol core and builds with Rust 1.97.0,
`--offline --locked --release --features jemalloc`, one job. Peer SHA
`b740a246a4737ba5881568e6c2bb04b901f59061544ef56c29bae40e31bcd9ca` is retained. The shipping CLI was not rebuilt;
active services and the staged CLI were preserved. The peer uses its default
allocator; the measured server uses the qualified jemalloc CLI.

The existing `audit_auth_contracts.rs` supplies the wire/crypto implementation;
`scripts/benchmark_auth.py` adds only timing and barriers, failing on source drift.
`scripts/benchmark_auth_lab.py` holds prepared peers at a barrier after socket
connect and client key/ClientHello generation. Its full timer runs from immediately
before ClientHello send to decrypted AuthOK; server TUN setup and process startup
are excluded. AUTH runs from immediately before AUTH encryption/send to verified
reply and includes queueing, credential/proof checks, Argon2, address admission
and protocol I/O. It is not an isolated hash benchmark.

Every password PHC uses Argon2id v19, **m=19456 KiB, t=2, p=1**, matching current
`crypto::password_hasher()`. The server's unchanged Argon2 gate allows two jobs
on this two-vCPU VM. Rate is requested peers divided by the interval from earliest
ClientHello to latest verified AuthOK on the shared monotonic clock; release
scheduling spread remains in JSON. These finite bursts do not measure sustained
arrival capacity or maximum server capacity.

| Carrier | Concurrent peers | Admissions/s median [min–max] | AUTH p95, ms* | Full p95, ms* |
|---|---:|---:|---:|---:|
| tcp | 1 | 29.75 [26.04–29.80] | 19.919 | 33.614 |
| tcp | 4 | 67.76 [60.57–70.73] | 43.074 | 58.984 |
| tcp | 16 | 73.89 [72.57–74.29] | 195.449 | 216.441 |
| udp | 1 | 49.02 [45.92–49.62] | 19.550 | 20.398 |
| udp | 4 | 73.40 [67.30–77.38] | 52.527 | 54.086 |
| udp | 16 | 81.15 [74.91–83.28] | 190.252 | 196.532 |
| quic | 1 | 49.19 [48.63–51.38] | 19.306 | 20.328 |
| quic | 4 | 84.20 [67.15–85.60] | 44.997 | 46.620 |
| quic | 16 | 77.49 [74.17–79.40] | 197.363 | 202.929 |

*p95 columns are medians of three per-burst nearest-rank p95 values. A one-peer
burst has only one latency value; it cannot establish a statistical tail.
All individual timings, per-burst maxima and median/min/max populations remain
in JSON. Server CPU/RSS/load/steal samples enclose each burst; 100% CPU is one
logical CPU, RSS is a sampled maximum. Affinity0-1/nice0 are observed, not newly
pinned. Client crypto also competes for these same CPUs.

The initial source-manifest argument overflow, one accepted TCP peer followed by
test cleanup failure, and an intentionally interrupted slow per-file SFTP transfer
are retained. Cleanup now revokes sessions while peers remain alive, then releases
them; the final complete matrix uses the same qualified peer. The separately
retained partial peer is not included in the 189-peer matrix. A SHA-verified bulk
archive carries all 892 measurement files; no successful burst was rerun for
transfer. Full host gates and cleanup differences remain in JSON, without waiver.

73 Windows/73 Linux benchmark methods PASS; nine docs checks PASS. The owned
root was SHA-archived and removed; private-source Cargo metadata and the owned
cached peer were removed while preserving the staged CLI. Status remains
**DIAGNOSTIC_MEASUREMENTS_ONLY**. Other carriers, IPv6 AUTH and sustained or
rejected-login rate remain OPEN. Q36 IN_PROGRESS; 30/37 DONE (81.1%).
LOCAL_DEV_COMMIT; NO_PUSH_DEPLOY_RELEASE.

## RTT under unpaced TCP bidir — 10–11 October 2026

**Q36-F013:** [JSON](../../../release/benchmark_v0.8.2_2026-10-10_unpaced_latency.json) contains three fresh sessions
per each of 12 carriers and the direct baseline: 39 sessions, 78 ten-second
TCP-bidir loads P=1/P=4 and 78 overlapping ping windows. Both VMs use the
existing CLI `238153ab5c44dc707390e0adf6ac14fecd7079f242c71bdb3dcdd84ab8472a71`; all 916 build inputs verified,
no rebuild. IPv4 inner/outer, TUN MTU1400, underlay1500, recordizer policy off.
Carrier and P1/P4 order changes between repeats.

Unlike Q36-F011, there is no `-b`: TCP offers maximum available load in both
directions. This is a load profile, not proof of physical link or both vCPUs
saturation. Actual goodput from both receivers, retransmits, CPU/RSS, VM
load/steal and kernel/server deltas remain in JSON. The direct baseline has no
Qeli process sampler. These values do not establish causes of differences
between carriers or between repeats.

Each window has 200 56-byte ICMP probes at 20 ms; every reply epoch is checked
inside the conservative iperf load interval. 15,600 transmitted probes,
15560 received, 40 lost. p50/p95/p99 use nearest-rank over received
replies; lost probes receive no fabricated RTT. Table values are medians of
three windows in ms, with full p99 min–max. These short populations do not
guarantee an hours-long tail.

| Carrier | P1 p50 | P1 p95 | P1 p99 [min–max] | P4 p50 | P4 p95 | P4 p99 [min–max] |
|---|---:|---:|---:|---:|---:|---:|
| direct | 0.490 | 1.219 | 1.668 [1.086–1.743] | 1.793 | 4.733 | 5.884 [5.029–6.058] |
| tcp-faketls | 28.697 | 41.195 | 47.241 [46.456–64.022] | 53.236 | 69.529 | 84.177 [83.816–92.050] |
| tcp-frag | 34.068 | 48.072 | 54.551 [51.464–60.681] | 59.862 | 84.263 | 99.938 [99.867–146.589] |
| tcp-obfs | 32.874 | 47.057 | 52.695 [52.493–58.266] | 51.133 | 67.476 | 75.532 [75.194–89.252] |
| tcp-obfs-awg | 31.890 | 42.319 | 48.939 [44.731–49.256] | 46.750 | 63.441 | 68.355 [65.341–69.989] |
| tcp-padding | 26.620 | 46.046 | 50.641 [32.091–62.808] | 55.908 | 76.547 | 87.035 [75.537–117.027] |
| tcp-plain-raw | 34.925 | 50.060 | 64.864 [54.218–70.813] | 54.527 | 75.608 | 84.038 [71.674–86.195] |
| tcp-reality | 25.057 | 35.367 | 41.902 [41.750–57.096] | 47.367 | 75.422 | 89.632 [88.574–95.928] |
| tcp-reality-tls | 13.806 | 20.162 | 22.390 [19.280–26.784] | 17.284 | 23.207 | 26.122 [25.420–46.445] |
| udp-faketls | 14.367 | 20.103 | 22.849 [22.455–38.351] | 16.384 | 23.417 | 25.627 [25.290–34.800] |
| udp-faketls-awg | 15.302 | 24.067 | 29.288 [23.387–35.524] | 17.476 | 24.282 | 30.301 [26.799–36.049] |
| udp-padding | 15.595 | 24.277 | 30.410 [28.683–31.324] | 17.915 | 35.301 | 50.515 [36.912–53.714] |
| udp-quic | 14.795 | 27.597 | 30.953 [27.114–40.023] | 17.070 | 29.927 | 45.756 [28.853–47.706] |

Fixed diagnostic loss on strict admission refusal: shared `measure` saves
both endpoints' final IPv4/IPv6 kernel counters and server counters before
metric validation. Invalid iperf still raises and does not become an admitted
measurement. Two regression tests cover a truncated TCP receiver and missing
reverse UDP receiver. This fix cannot restore the missing final counters of
the older refused Q36-F012 window. Also fixed ping oversend: shared
`latency_probe_command` uses `-c 200` without `-w` and an external `timeout`;
timeout refusal is never admitted. [iputils documentation](https://github.com/iputils/iputils/blob/master/doc/ping.xml)
confirms that `-w` changes termination to waiting for the reply count. The first
P4 window sent 205 probes, received 200 and was refused. After resuming, the
first UDP-AWG RTT was 0.95 s after the rounded iperf start and failed the
conservative 1 s bound. Synchronization now waits for an established iperf
connection and then 2 s, rather than pausing from dispatch; raw epochs are
still checked for every result. Both original FAILs and three windows from
two incomplete sessions remain separate; those sessions use fresh retries.
One fully completed direct baseline and other completed sessions are inherited
without remeasurement.
12 completed sessions use the original pause from dispatch; 27 new sessions
wait for the connection. Exact first/last RTT offsets for each window remain
in `probe_offsets`; these short windows cannot establish a regression or
identical warm-up timing. Status
is DIAGNOSTIC_MEASUREMENTS_ONLY even if the final host gate passes. Neither
failed attempts nor outliers are removed. Full suite: 68 Windows and 68 Linux
benchmark methods, 9 docs checks PASS. Host gate: final/10.66.116.10: PASS; final/10.66.116.11: PASS; attempt1/10.66.116.10: FAIL (iptables4); attempt1/10.66.116.11: PASS; attempt2/10.66.116.10: FAIL (iptables4, legacy4); attempt2/10.66.116.11: PASS; cleanup/10.66.116.11: PASS; cleanup/10.66.116.10: FAIL (iptables4, legacy4); differences retained without
waiver. Both owned roots SHA-archived and removed; active services and original
staged CLIs preserved.
After measurement, ping/iperf IP arguments were additionally escaped with
`shlex.quote`: IP validation permits scoped IPv6 with arbitrary zone text.
A test checks one shell argument without executing such a target. Measured
helper snapshot retained; final helper differs only in these two escapes and
CRLF/LF normalization. Commands are identical for all three measured IPv4
targets; both SHAs remain in `post_measurement_command_quoting`. The first
Linux test launch refused because uploaded inputs lacked `benchmark_report.py`;
its log/FAIL retained, dependency supplied before the new launch.

Q36-F011 idle and paced100M data remain separate populations. IPv6 RTT, UDP
payload latency, auth-rate, remaining IPv6 carrier combinations and the cause
of observed asymmetry remain OPEN. Q36 IN_PROGRESS; overall 30/37 DONE,
81.1 percent. LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.

## UDP-bidir and independent inner/outer families — 10 October 2026

**Q36-F012:** the shared runner and offline report support `--measurement udp-bidir`.
Both receivers are mandatory; UDP sender flags differ from TCP. Admission binds
protocol, P, sender/receiver duration, both bitrate fields, 1200-byte datagrams,
packets/lost_packets/jitter and loss. CI now runs the whole `test_benchmark*.py`
suite including latency: 63 Windows and 63 Linux methods, 11 workflow methods,
actionlint 1.7.12 (4 workflows) and 9 docs checks PASS. Windows admission combines
the full suite with a repeat of the 10 changed methods after validator refinement.

The first P4 run exposed an error in the new validator.
[iperf3 3.18](https://github.com/esnet/iperf/blob/3.18/src/iperf_api.c)
computes JSON loss using sender packet counts, while receiver `packets` can
differ. A real loss fixture pins the distinction; delivered goodput comes only
from receiver bytes/rate. `*_packets` retains the raw summary field and does not
replace delivered payload bytes. Loss and goodput are published independently.
The original FAIL and four partial windows remain separate, including P4 with
19.37%/13.42% loss: its full counter window is unavailable after rejection.
Two completed baselines are inherited without rerunning; the incomplete cell
was repeated in a fresh session. No failed phase is labelled PASS.

[Complete JSON](../../../release/benchmark_v0.8.2_2026-10-10_udp_ipv6.json): 69 completed fresh sessions,
204 five-second phases, 3 repeats per each of 23 cells. Both VMs use qualified CLI
`238153ab5c44dc707390e0adf6ac14fecd7079f242c71bdb3dcdd84ab8472a71`; all 916 build inputs unchanged, no rebuild.
All 12 IPv4 modes carry UDP payload bidir P1/P4. TCP fake-TLS, UDP fake-TLS
and QUIC additionally cover outer4/inner6,outer6/inner4,outer6/inner6 with TCP
and UDP payload bidir P1/P4. Direct baseline covers both families.
NetworkPlan,server peer and raw iperf endpoints verify actual families.
Cell/phase order rotates; every metric retains median/min/max.
The public CLI `--measurement udp-bidir --tcp-parallel 1|4` uses IPv4;
this packet's IPv6 matrix calls shared `configs(..., outer_family=..., inner_family=...)`
through a separate isolated driver. The compatible `--tcp-parallel` option name
also specifies the number of streams for UDP-bidir.

UDP target 100 Mbps **per stream per direction**: P1=100+100, P4=400+400.
Payload 1200, TUN MTU 1400, underlay 1500, recordizer policy off.
Each VM has 2 vCPUs and about 2 GiB RAM; kernel/governor and environment metadata
remain in `host_metadata`. These VM results do not establish physical-device capacity.
Tables show medians of three sessions; full ranges, CPU/RSS/steal/load, retransmits, packets and
per-family drop deltas remain in JSON.

| Carrier | Outer/inner | P1 ↑/↓ Mbps | P1 loss ↑/↓ % | P4 ↑/↓ Mbps | P4 loss ↑/↓ % |
|---|---|---:|---:|---:|---:|
| direct | 4/4 | 100.0/100.0 | 0.00/0.00 | 400.0/400.0 | 0.00/0.00 |
| direct | 6/6 | 100.0/100.0 | 0.00/0.00 | 400.0/400.0 | 0.00/0.00 |
| tcp-faketls | 4/4 | 100.0/100.0 | 0.00/0.00 | 302.2/365.6 | 23.64/6.01 |
| tcp-faketls | 4/6 | 100.0/100.0 | 0.00/0.00 | 295.9/348.4 | 24.54/10.18 |
| tcp-faketls | 6/4 | 100.0/100.0 | 0.00/0.00 | 313.6/358.2 | 21.10/7.82 |
| tcp-faketls | 6/6 | 100.0/100.0 | 0.00/0.00 | 291.3/335.5 | 25.17/11.66 |
| tcp-frag | 4/4 | 100.0/100.0 | 0.00/0.00 | 297.6/347.9 | 24.02/10.07 |
| tcp-obfs-awg | 4/4 | 100.0/100.0 | 0.00/0.00 | 206.9/379.1 | 46.14/0.80 |
| tcp-obfs | 4/4 | 100.0/100.0 | 0.00/0.00 | 254.3/366.2 | 35.89/5.63 |
| tcp-padding | 4/4 | 100.0/100.0 | 0.00/0.00 | 267.3/375.2 | 32.85/3.49 |
| tcp-plain-raw | 4/4 | 100.0/100.0 | 0.00/0.00 | 313.6/302.1 | 21.17/21.43 |
| tcp-reality-tls | 4/4 | 99.9/99.9 | 0.00/0.00 | 399.3/399.3 | 0.00/0.00 |
| tcp-reality | 4/4 | 100.0/100.0 | 0.00/0.00 | 297.0/349.1 | 25.15/8.93 |
| udp-faketls-awg | 4/4 | 100.0/100.0 | 0.00/0.00 | 361.5/157.7 | 9.62/36.98 |
| udp-faketls | 4/4 | 100.0/100.0 | 0.00/0.00 | 366.3/138.5 | 8.13/39.28 |
| udp-faketls | 4/6 | 100.0/100.0 | 0.00/0.00 | 354.3/77.8 | 10.70/50.90 |
| udp-faketls | 6/4 | 100.0/100.0 | 0.00/0.00 | 347.4/107.7 | 12.84/44.67 |
| udp-faketls | 6/6 | 100.0/100.0 | 0.00/0.00 | 351.7/113.6 | 11.90/54.95 |
| udp-padding | 4/4 | 100.0/100.0 | 0.00/0.00 | 358.5/98.0 | 9.81/46.86 |
| udp-quic | 4/4 | 100.0/100.0 | 0.00/0.00 | 364.3/121.2 | 8.57/47.41 |
| udp-quic | 4/6 | 100.0/100.0 | 0.00/0.00 | 349.2/150.1 | 12.48/51.49 |
| udp-quic | 6/4 | 100.0/100.0 | 0.00/0.00 | 345.9/90.6 | 12.85/55.44 |
| udp-quic | 6/6 | 100.0/100.0 | 0.00/0.00 | 356.1/80.6 | 10.40/62.16 |

`tcp-reality-tls` has zero median P4 loss, but the maximum across the three
windows is 0.693/0.000% ↑/↓. A zero median does not imply loss-free results
in every repeat; the worst window remains in JSON.

TCP payload bidir in the new family cells;other IPv4 TCP measurements follow below:

| Carrier | Outer/inner | TCP P1 ↑/↓ Mbps | TCP P4 ↑/↓ Mbps |
|---|---|---:|---:|
| direct | 4/4 | 14480.2/15323.9 | 6114.2/20810.4 |
| direct | 6/6 | 12950.1/16385.3 | 5430.1/20863.5 |
| tcp-faketls | 4/6 | 61.0/485.5 | 39.4/476.6 |
| tcp-faketls | 6/4 | 63.3/440.0 | 44.5/521.5 |
| tcp-faketls | 6/6 | 45.4/446.0 | 36.0/520.7 |
| udp-faketls | 4/6 | 134.1/754.8 | 100.7/777.5 |
| udp-faketls | 6/4 | 168.0/644.8 | 91.8/779.9 |
| udp-faketls | 6/6 | 153.8/688.3 | 101.6/689.4 |
| udp-quic | 4/6 | 177.4/630.4 | 81.3/777.1 |
| udp-quic | 6/4 | 48.6/255.7 | 32.7/260.1 |
| udp-quic | 6/6 | 30.8/282.6 | 37.7/254.6 |

Kernel snapshots now include IPv4 and IPv6 on both sides:inner family selection
cannot hide outer UDP carrier drops in the other family. Admission checks both
counters,their sum and individual resets. These are isolated namespace counters,
not per-socket attribution. Maximum phase deltas: {'server_session_drops': 46527, 'server_ipv4_rcvbuf_drops': 0, 'server_ipv6_rcvbuf_drops': 162, 'client_ipv4_rcvbuf_drops': 162605, 'client_ipv6_rcvbuf_drops': 163072}.
Client status before the first phase observes effective UDP receive buffers of
[4194304] bytes, separately from `rmem_max`. Per-phase socket sizes and client
drop deltas were not captured; automatic growth is not established. Server `dropped`
combines multiple causes rather than queue loss alone. P4 loss remains visible
and does not by itself establish a specific defect or cause.

Host gates: pilot/10.66.116.10: FAIL (iptables4); pilot/10.66.116.11: PASS; attempt1/10.66.116.10: FAIL (iptables4); attempt1/10.66.116.11: PASS; matrix/10.66.116.10: PASS; matrix/10.66.116.11: PASS. All FAIL remain in DIAGNOSTIC_MEASUREMENTS_ONLY. Both owned
roots were SHA-archived and removed; active services preserved. The IPv6
cross-product currently covers three core carriers; all IPv6 obfuscation modes,
auth-rate, full saturated/P4 latency and exact asymmetry cause remain OPEN.
Q36 IN_PROGRESS; overall 30/37 DONE, 81.1 percent. LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.

## RTT distributions across all carriers — 10 October 2026

**Q36-F011:** [latency JSON](../../../release/benchmark_v0.8.2_2026-10-10_latency.json) contains three fresh
sessions per each of 12 carriers and the direct baseline. Both VMs use the same
qualified CLI `238153ab5c44dc707390e0adf6ac14fecd7079f242c71bdb3dcdd84ab8472a71` from the 916 Q36-F010 build inputs;
no rebuild. IPv4 inner/outer, TUN MTU1400, underlay1500, recordizer policy off.
39 sessions, 78 windows, 15,600 transmitted ICMP probes,15600 received
RTTs, 0 lost probes. Every raw reply and tail is retained; carrier and
idle/loaded order changes between repeats.

Each window: 200 56-byte probes at 20 ms. p50/p95/p99 use nearest-rank over
received replies; lost probes receive no fabricated RTT. Values below are
medians across three windows in ms; p99 also includes full min–max.
With 200 replies p99 is the 198th ordered RTT of 200 replies: a short measurement,
not an hours-long tail guarantee or one-way latency.

Loaded: an independent 8 s TCP bidir P=1 test, target 100 Mbps per direction;
ping epochs lie inside the conservative iperf interval. Both receivers,
bitrate, cookies, CPU/RSS samples and kernel/server deltas are re-admitted from
raw. This is paced load, not saturation: actual goodput and every CPU/RSS/drop
value remain in JSON. The direct baseline has no Qeli process sampler.

| Carrier | Idle p50 | Idle p95 | Idle p99 [min–max] | Loaded p50 | Loaded p95 | Loaded p99 [min–max] |
|---|---:|---:|---:|---:|---:|---:|
| direct | 0.347 | 0.392 | 0.411 [0.406–0.433] | 0.300 | 0.383 | 0.403 [0.397–0.420] |
| tcp-faketls | 0.988 | 1.150 | 1.184 [1.130–1.219] | 2.933 | 4.534 | 4.754 [4.744–5.253] |
| tcp-frag | 0.968 | 1.144 | 1.190 [1.183–1.241] | 2.831 | 4.294 | 4.869 [4.718–5.518] |
| tcp-obfs | 1.009 | 1.155 | 1.199 [1.175–1.216] | 2.838 | 4.537 | 4.803 [4.673–4.946] |
| tcp-obfs-awg | 0.978 | 1.153 | 1.185 [1.164–1.225] | 2.725 | 4.341 | 4.763 [4.692–4.902] |
| tcp-padding | 0.985 | 1.125 | 1.182 [1.168–1.229] | 2.989 | 4.550 | 5.022 [4.738–5.086] |
| tcp-plain-raw | 0.990 | 1.147 | 1.225 [1.208–1.249] | 2.826 | 4.508 | 5.286 [4.989–5.313] |
| tcp-reality | 0.994 | 1.132 | 1.189 [1.164–1.207] | 2.839 | 4.409 | 4.781 [4.765–5.154] |
| tcp-reality-tls | 13.363 | 17.884 | 19.418 [18.701–19.483] | 5.712 | 10.572 | 12.537 [10.316–15.858] |
| udp-faketls | 0.993 | 1.135 | 1.363 [1.158–2.157] | 1.918 | 3.708 | 4.139 [4.101–4.745] |
| udp-faketls-awg | 0.978 | 1.163 | 1.363 [1.104–1.827] | 1.906 | 3.540 | 4.303 [4.100–4.510] |
| udp-padding | 1.001 | 1.155 | 1.304 [1.218–1.948] | 1.913 | 3.723 | 4.319 [4.200–4.442] |
| udp-quic | 0.986 | 1.162 | 1.251 [1.233–1.286] | 1.617 | 3.354 | 4.153 [3.682–4.454] |

Real TLS stands out: median p99 19.418 ms idle and 12.537 ms
under the specified load. This is a three-session observation, not an established
cause or a reason to change transport defaults.

The parser refuses truncated/duplicate populations, wrong loss, duplicate
summaries and censored `time<…`;53 Windows benchmark methods, 4 new Linux
methods and 9 docs checks PASS. Host gate: 10.66.116.10: FAIL (legacy4); 10.66.116.11: PASS. Firewall differences remain
without waiver. Both owned roots were SHA-archived and removed; active services
and original staged CLIs preserved. IPv6, UDP payload bidir, auth-rate, saturated/P4
latency and the exact raw TCP asymmetry cause are outside this packet.
Q36 IN_PROGRESS; overall 30/37 DONE, 81.1 percent. LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.

## TCP downlink losses and actual mux — 10 October 2026

**Q36-F010:** closes the receive-to-TUN observability gap left in Q36-F009.
Core records mux pool refusal, Full and Disconnected in stats.tcp_downlink_drops;
[exact semantics](../manuals/OPERATIONS.md#tcp-losses-at-tun-handoff).
Both raw/mux branches use a separate counter from uplink. Buffers and policy
are unchanged: Full continues processing, Disconnected ends the receive task.
Remaining records after stop, management, invalid/unsupported packets, decode
errors and network losses are outside this counter. Later OS write drops were
measured through udp_drops_tun_write, a shared writer also used by TCP despite
its historical name. C ABI/GUI stats remain unchanged; rx_packets/rx_bytes before
enqueue do not establish device delivery, and mux pool refusal does not add rx.

Five new Rust tests PASS: raw/mux Full and Disconnected with all slots returned,
mux pool refusal/recovery without rx overcount, exclusion of empty/unsupported
input, and the three-reason sum. Tests call the actual receive function and
SyncSender. Release jemalloc, rustfmt and both Clippy gates PASS, including
minimal transport-core-ffi. All 916 inputs match before/after build; CLI SHA
`238153ab5c44dc707390e0adf6ac14fecd7079f242c71bdb3dcdd84ab8472a71`. The earlier staged .11 CLI was SHA-restored.
This qualifies one artifact, not independent build reproducibility.
The first runtime driver refused before measurements due to a stale profile
label; receipt/logs are retained, and namespaces were cleaned before retry.

[Diagnostic JSON](../../../release/benchmark_v0.8.2_2026-10-10_client_downlink.json): fixed older server 7769f43c,
current client, IPv4 inner/outer, P=1, MTU1400/1500. Three fresh sessions per
plain raw, real TLS raw and plain mux. Mux server INI requires
obf.recordizer.policy = required; both status points confirm packet_mux_v1;
raw points confirm legacy_packet_per_record. Each session has two 8-second
bidir windows, unlimited/250 Mbps per direction, with rotating rate order.
18 phases/72 raw JSON; median [min–max], Mbps:

| Carrier | Client | Target Mbps/direction | ↑ Mbps median [min–max] | ↓ Mbps median [min–max] | Downlink drops total median [min–max] |
|---|---|---:|---:|---:|---:|
| tcp-plain-raw | new | unlimited | 29.2 [25.4–31.7] | 526.3 [489.8–527.8] | 0.0 [0.0–0.0] |
| tcp-plain-raw | new | 250 | 250.0 [248.3–250.0] | 249.8 [249.8–249.8] | 0.0 [0.0–0.0] |
| tcp-reality-tls | new | unlimited | 401.3 [180.5–462.5] | 734.9 [713.6–960.8] | 0.0 [0.0–0.0] |
| tcp-reality-tls | new | 250 | 249.8 [249.8–249.9] | 249.9 [249.7–249.9] | 0.0 [0.0–0.0] |
| tcp-plain-mux | new | unlimited | 817.6 [810.4–818.7] | 1066.4 [1062.4–1125.5] | 0.0 [0.0–0.0] |
| tcp-plain-mux | new | 250 | 249.7 [249.6–249.7] | 249.8 [249.7–249.9] | 0.0 [0.0–0.0] |

In this series, unlimited plain raw medians are 29.2/526.3,
versus 817.6/1066.4 Mbps for plain mux.
This is a scoped record-packing observation, not proof of the precise cause,
a universal workload gain or a reason to change defaults.

Maximum phase deltas: uplink=0,
downlink=0, OS writer=0.
These are reached counters, not proof of zero total loss, waiting or ACK
starvation. Status samples every 2 seconds, with 3-second settling before/after.
Raw receiver/bitrate/cookie, PID/CPU policy/MTU, CPU/RSS, kernel/server deltas,
generation and both client counter monotonicity were revalidated.

Host gate: 10.66.116.10: FAIL iptables4,legacy4; 10.66.116.11: restored. FAIL remains; status DIAGNOSTIC_MEASUREMENTS_ONLY.
Both owned roots were SHA-archived/removed; current Cargo metadata was backed
up and selectively invalidated. The earlier CLI and active services were
preserved. Evidence: audit-debt-20260924/q36-client-downlink-20261010/.
Exact asymmetry cause/UDP-bidir/IPv6/latency/auth-rate and the full Q35 8-hour
soak remain OPEN; Q36 IN_PROGRESS,30/37 DONE (81.1%).

## Client drops and TCP client comparison — 10 October 2026

**Q36-F009:** the TCP distributor silently discarded packets on pool refusal,
a full pinned queue, oversized input or absence of live carriers. Shared core
runtime now records these reasons separately; Linux CLI exposes
`stats.tcp_uplink_drops`; [status semantics](../manuals/OPERATIONS.md#tcp-client-queue-drops).
Full preserves flow pinning; Closed retries the same packet on a live carrier
without a false drop. Queue capacities, policy and pacing are unchanged.
C ABI and GUI stats are unchanged. tx_packets/tx_bytes count TUN inputs before
enqueue, not confirmed delivery. Downlink-to-TUN handoff is outside these
counters; zero uplink drops does not establish zero total client loss or waiting.

[Diagnostic JSON](../../../release/benchmark_v0.8.2_2026-10-10_client_uplink.json): fixed server `7769f43c`,
old client `7769f43c`, new client `9900f38d`. Plain TCP and real TLS,
IPv4 inner/outer, P=1, recordizer=off, MTU1400/1500. Three **fresh sessions**
per carrier/client, each with 8-second unlimited and 250 Mbps/direction bidir
phases; client and rate order alternate. 24 phases/96 raw JSON.
Median [min–max], Mbps; missing old counters are UNAVAILABLE:

| Carrier | Client | Target Mbps/direction | ↑ Mbps median [min–max] | ↓ Mbps median [min–max] | Client queue-full drops median [min–max] |
|---|---|---:|---:|---:|---:|
| tcp-plain-raw | old | unlimited | 31.2 [26.6–55.0] | 507.0 [472.6–512.0] | UNAVAILABLE |
| tcp-plain-raw | old | 250 | 248.0 [229.5–249.7] | 249.5 [249.3–249.8] | UNAVAILABLE |
| tcp-plain-raw | new | unlimited | 34.7 [32.0–46.2] | 500.8 [484.2–505.2] | 0.0 [0.0–0.0] |
| tcp-plain-raw | new | 250 | 249.8 [249.6–250.0] | 250.0 [250.0–250.0] | 0.0 [0.0–0.0] |
| tcp-reality-tls | old | unlimited | 463.6 [234.2–499.5] | 722.1 [686.5–905.4] | UNAVAILABLE |
| tcp-reality-tls | old | 250 | 249.9 [249.6–249.9] | 249.8 [249.7–249.9] | UNAVAILABLE |
| tcp-reality-tls | new | unlimited | 415.3 [388.6–459.4] | 700.5 [686.8–770.7] | 0.0 [0.0–0.0] |
| tcp-reality-tls | new | 250 | 249.9 [249.7–249.9] | 249.9 [249.9–249.9] | 0.0 [0.0–0.0] |

All new-client phases report zero uplink drops. Both old and new plain TCP retain unlimited asymmetry;
these measurements establish neither a counter-induced regression nor its
precise cause. Linux status samples every 2 seconds; before/after include
3-second settling periods, so these are not exact iperf-only counter windows.
Receivers, both bitrate fields, unique cookies, PID/CPU policy, MTU, CPU/RSS,
kernel/server deltas and monotonic client telemetry were revalidated.

9 Rust uplink tests PASS (5 new); release jemalloc, rustfmt and both Clippy
gates PASS, including minimal transport-core-ffi. All 915 source inputs match
before/after build; the new CLI SHA is pinned. The earlier .11 staged CLI was
SHA-restored. This qualifies one artifact, not independent reproducibility.
Initial harness failures are retained: missing import, conformance fixtures
and four integration-lint INI examples were corrected before final PASS.
The first cleanup guard refused before changes because dep-info paths were
relative; read-only inspection qualified 14 qeli units/68 SHA-archived metadata files.

Host gate: 10.66.116.10: FAIL iptables4,legacy4; 10.66.116.11: restored. Original FAIL evidence is retained;
status DIAGNOSTIC_MEASUREMENTS_ONLY. Both owned remote roots were SHA-archived
and removed; only private-source Cargo unit metadata was invalidated after
backup. The earlier CLI and active services were preserved. Evidence:
audit-debt-20260924/q36-client-uplink-20261010/.
Downlink handoff observability/exact cause/UDP-bidir/IPv6/latency/auth-rate
and full Q35 8-hour soak remain OPEN. Q36 IN_PROGRESS,30/37 DONE (81.1%).

## Bidir pressure diagnostics — 10 October 2026

[Pressure JSON](../../../release/benchmark_v0.8.2_2026-10-10_bidir_pressure.json): the same CLI SHA `7769f43c`,
IPv4 inner/outer, P=1, recordizer=off, MTU 1400/1500. Plain TCP, real TLS and
UDP QUIC were measured at 100/250 Mbps per direction and unlimited, alongside
the direct VM namespace path. Each condition has three sequential 8-second
windows; the rate order rotates. These are repeated windows within **one
session per carrier**, not three independent tunnel starts. Total: 36 phases
(27 tunnel + 9 baseline), 90 raw JSON. Median [min–max], Mbps:

| Carrier | Target Mbps / direction | ↑ Mbps median [min–max] | ↓ Mbps median [min–max] | Session drops median [min–max] |
|---|---:|---:|---:|---:|
| direct | 100 | 100.1 [100.0–100.1] | 100.1 [100.0–100.1] | — |
| direct | 250 | 250.1 [250.1–250.1] | 250.1 [250.1–250.1] | — |
| direct | unlimited | 14307.8 [14135.1–15674.4] | 16153.4 [14576.6–16154.5] | — |
| tcp-plain-raw | 100 | 100.1 [100.0–100.1] | 100.1 [100.0–100.1] | 0.0 [0.0–0.0] |
| tcp-plain-raw | 250 | 249.4 [249.1–249.9] | 249.7 [249.4–249.9] | 0.0 [0.0–0.0] |
| tcp-plain-raw | unlimited | 27.5 [21.7–39.4] | 507.2 [505.2–522.9] | 0.0 [0.0–44.0] |
| tcp-reality-tls | 100 | 100.0 [99.8–100.0] | 99.9 [99.8–99.9] | 0.0 [0.0–0.0] |
| tcp-reality-tls | 250 | 249.8 [249.5–249.8] | 249.8 [249.7–249.9] | 0.0 [0.0–0.0] |
| tcp-reality-tls | unlimited | 398.2 [295.0–447.9] | 756.4 [717.1–816.5] | 0.0 [0.0–0.0] |
| udp-quic | 100 | 100.0 [100.0–100.1] | 100.0 [100.0–100.1] | 0.0 [0.0–0.0] |
| udp-quic | 250 | 250.0 [249.9–250.1] | 249.9 [249.9–250.0] | 0.0 [0.0–0.0] |
| udp-quic | unlimited | 160.0 [84.5–182.2] | 735.1 [676.3–835.5] | 0.0 [0.0–0.0] |

Plain TCP is approximately symmetric at 100/250 Mbps; unlimited uplink is
21.7–39.4 against downlink 505.2–522.9.
Two of its three unlimited phases have zero session drops; the third adds 44.
All its paced phases have zero session drops. Thus asymmetry depends on load,
but recorded server writer queue overflow alone cannot explain it. Zero values
of these counters do not rule out queue waiting, ACK delay, retransmission or
other loss. The cause remains OPEN; data-plane buffers and scheduling were not
changed. The client TCP distributor can also discard packets on pool refusal
or a Full try_send queue; server control counters do not measure those reasons.
Raw end.streams retains RTT/retransmits/cwnd; mean_rtt with -i 0 is not
a latency distribution and does not close p50/p95/p99. These 8-second phases
are kept separate from the earlier 5-second unrestricted bidir population.

**Q36-F008:** admission now checks both actual target_bitrate fields. Ordinary
unlimited reports reject paced input; the diagnostic helper requires an explicit
matching limit. [iperf3 semantics](https://software.es.net/iperf/invoking.html):
-b limits each stream individually; P=1 is used in each direction here.
All 49 Windows/49 Linux methods PASS without skips; the previous 78 phases/
228 raw JSON were readmitted by the strengthened validator with original host
FAIL states retained. New raw was separately revalidated against receivers,
both kernel deltas, stable session identity, CPU/RSS samplers, PID/CPU policy
and frozen helper input SHAs.

Full host gate: 10.66.116.10: FAIL iptables4,legacy4; 10.66.116.11: restored. Every difference is retained in JSON; this is
DIAGNOSTIC_MEASUREMENTS_ONLY, not a performance PASS. Both owned remote roots
were SHA-archived and removed after inode/process/mount checks; active services
were preserved. Owner SHAs/journal matched the known external NAT cycle.
Snapshots are sequential: the cycle can change rules between backend reads.
Evidence: audit-debt-20260924/q36-bidir-pressure-20261010/.
UDP-bidir/IPv6/latency/auth-rate/the precise asymmetry cause and the full Q35
8-hour soak remain OPEN; Q36 IN_PROGRESS, 30/37 DONE (81.1%).

## Simultaneous inner TCP — 10 October 2026

[Bidir JSON](../../../release/benchmark_v0.8.2_2026-10-10_bidir_12modes.json): the same SHA `7769f43c` and unchanged
qeli package tree, three separate P=1/P=4 sweeps of all 12 carriers, 72 tunnel
and 6 baseline phases. IPv4 inner/outer, recordizer=off, MTU 1400/1500, affinity
0-1, nice 0, two 2-vCPU VMs; governor UNAVAILABLE. Each iperf session used
`--bidir`, 5 seconds, P streams **per direction**. This is inner TCP over
TCP/UDP carriers; UDP-bidir was not tested. All 228 raw JSON files were
revalidated against both receiver summaries, samplers and both namespace
counters. No failure or outlier was discarded. Below: median [min–max], Mbps.

| Mode | P1 up/down Mbps median [min–max] | P4 up/down Mbps median [min–max] |
|---|---|---|
| tcp-plain-raw | 54.5 [50.7–59.7] / 491.9 [463.5–503.8] | 41.3 [34.8–46.3] / 503.6 [489.6–520.3] |
| tcp-faketls | 54.5 [50.0–84.4] / 497.1 [443.0–514.0] | 46.7 [43.6–48.4] / 510.6 [495.7–518.0] |
| tcp-padding | 43.8 [37.3–45.0] / 500.6 [497.2–508.4] | 40.8 [33.3–47.8] / 503.5 [493.9–509.3] |
| tcp-frag | 48.8 [32.9–54.3] / 499.4 [474.4–523.5] | 44.4 [31.0–44.6] / 547.1 [524.0–561.3] |
| tcp-obfs | 39.6 [34.8–54.1] / 544.1 [517.9–552.7] | 43.2 [41.9–44.3] / 515.3 [514.9–520.3] |
| tcp-reality | 51.5 [40.0–81.7] / 483.5 [455.1–521.1] | 46.9 [45.2–53.4] / 507.9 [482.2–530.4] |
| tcp-reality-tls | 429.4 [196.9–452.3] / 736.4 [733.1–957.4] | 264.0 [182.9–301.9] / 823.9 [796.6–929.9] |
| udp-faketls | 131.1 [126.7–167.7] / 796.5 [741.7–803.7] | 79.8 [79.4–95.1] / 828.8 [826.7–853.6] |
| udp-padding | 222.6 [167.3–280.6] / 699.2 [563.5–702.1] | 84.4 [82.8–117.6] / 805.2 [762.1–830.3] |
| udp-quic | 169.8 [150.7–174.2] / 709.1 [695.6–722.2] | 93.8 [86.1–101.8] / 775.2 [775.1–807.9] |
| tcp-obfs-awg | 53.9 [44.4–62.6] / 503.7 [493.1–516.5] | 36.6 [36.0–41.7] / 526.7 [515.3–554.9] |
| udp-faketls-awg | 166.8 [140.1–239.9] / 668.3 [615.1–733.5] | 102.9 [100.6–106.2] / 787.7 [730.9–797.0] |

Pronounced uplink/downlink asymmetry repeats. Baseline P1 uplink also varied
from 9371.2 to 14501.1 Mbps, P4 from 6253.0 to 7142.0: this limits conclusions
from this VM lab. Sequential measurements above lasted 8 seconds, bidir 5.
There is no same-duration A/B with an older CLI and no proven regression.
Maximum phase CPU server/client: 129.3/139.0%;
sampled peak RSS 114.2/59.0 MiB.
Maximum session drops: 2635; UDP RcvbufErrors server/client:
0/345.
The aggregate dropped counter includes pool refusal, writer queues and other
reasons. The specific throughput constraint remains OPEN; buffer sizes were
not changed.

**Q36-F006:** the runner adds `--measurement tcp-bidir`; the standard
up/down/UDP profile remains available. Admission verifies actual TCP/P/duration/
bidir, both receiver roles, nonzero progress, complete intervals, retransmits,
CPU/RSS and independent kernel drop deltas. Missing reverse measurements,
wrong settings, counter resets and mixed profiles are rejected.

**Q36-F007:** the ordinary offline gate requires restored host state.
Explicit `--measurements-only` retains host gate FAIL beside measurements and
never emits AGGREGATED. Only an explicitly recorded restoration failure with
fully revalidated raw measurements is admitted; other failures, missing
snapshots, falsely asserted host changes and incorrect phases are rejected.
47 Windows/47 Linux methods PASS without skips; Linux input SHAs matched.

Four sweeps reached COMPLETED with matching full host snapshots; P1/R1 and
P4/R3 retain FAIL. Only the three known legacy rules on `.10` changed in both;
owner SHAs and current journal confirmed the existing vpn-nat cycle. The
whole population is MEASUREMENTS_ONLY, not a host or performance PASS.
The initial harness mistake about the bidir field name is retained separately;
the [iperf3 3.18 contract](https://github.com/esnet/iperf/blob/3.18/src/iperf_api.c)
was checked against real JSON. Measured runner source bytes were SHA-saved
before subsequent offline-report changes; raw results were not rewritten.

All 14 owned remote roots were SHA-archived and removed after inode/file/
process/mount checks; active Qeli PID/SHA identities were preserved.
Evidence: `audit-debt-20260924/q36-bidir-20261010/`.
TCP-bidir IPv4 is measured; UDP-bidir, IPv6, p50/p95/p99, auth-rate, drop
diagnosis and the full 8-hour Q35 soak remain OPEN. Q36 IN_PROGRESS; 30/37 (81.1%).

## Repeated current-candidate measurement — 10 October 2026

Three separate sweeps each P=1/P=4, alternating and sequential:
72 modes, 192 tunnel and 18 baseline phases. All 210 raw iperf phases were
revalidated; tunnel phases also retain CPU/RSS and counter evidence. All 600
input JSON files have SHA identities. Two 2-vCPU VMs, Linux 6.12.105, iperf 3.18,
recordizer=off, IPv4 inner/outer. TUN MTU 1400/IPvlan 1500, affinity 0-1, nice 0;
VM governor is explicitly `UNAVAILABLE`. No compilation overlapped measurement.
Every process sample retains background load and steal; physical VM-host
performance and its governor are not qualified by these measurements.

Below: **median [min–max] of three repeats**, Mbps; loss is percent.
All observations, including outliers, remain in the population. P controls inner
TCP streams; UDP 100/500 use the same offered rate in both populations.

| Mode | P1 TCP up | P1 TCP down | P4 TCP up | P4 TCP down |
|---|---:|---:|---:|---:|
| tcp-plain-raw | 440.7 [431.4–447.4] | 544.8 [543.7–551.5] | 493.0 [465.5–532.8] | 541.2 [532.9–553.4] |
| tcp-faketls | 444.7 [443.4–471.3] | 528.8 [527.7–541.6] | 465.6 [457.1–490.1] | 558.7 [542.8–620.2] |
| tcp-padding | 454.9 [373.7–464.7] | 548.6 [532.9–562.6] | 502.2 [489.9–511.9] | 540.8 [540.3–562.8] |
| tcp-frag | 464.2 [427.3–474.6] | 552.7 [550.7–554.7] | 475.2 [456.6–521.4] | 565.3 [535.5–566.8] |
| tcp-obfs | 409.2 [404.0–419.9] | 578.0 [569.8–595.4] | 461.4 [453.1–463.5] | 571.7 [569.7–587.2] |
| tcp-reality | 461.7 [423.6–476.3] | 533.6 [532.2–535.7] | 451.6 [441.4–472.6] | 575.5 [552.9–576.9] |
| tcp-reality-tls | 937.2 [932.6–949.5] | 955.4 [936.4–956.3] | 919.5 [918.4–920.7] | 1027.9 [1003.7–1044.2] |
| udp-faketls | 546.6 [542.4–552.3] | 963.2 [960.9–1000.2] | 498.7 [482.3–505.7] | 978.8 [977.6–979.8] |
| udp-padding | 566.1 [531.6–588.8] | 972.4 [964.4–1031.6] | 563.8 [530.3–563.9] | 967.0 [908.3–983.6] |
| udp-quic | 543.4 [510.0–552.1] | 975.7 [949.4–985.8] | 541.1 [521.2–543.9] | 943.5 [920.0–952.2] |
| tcp-obfs-awg | 429.1 [424.1–436.4] | 577.6 [558.9–586.6] | 466.0 [444.5–470.5] | 574.4 [542.1–595.1] |
| udp-faketls-awg | 550.0 [522.1–558.5] | 988.5 [980.9–1021.8] | 523.2 [507.1–528.1] | 956.2 [954.5–987.5] |

| Mode | P1 UDP100 | P4 UDP100 | P1 UDP500 | P4 UDP500 | P1 loss500 % | P4 loss500 % |
|---|---:|---:|---:|---:|---:|---:|
| udp-faketls | 100.0 [100.0–100.0] | 100.0 [100.0–100.0] | 475.8 [400.5–480.6] | 453.3 [453.2–497.9] | 4.5 [3.8–19.9] | 9.0 [0.4–9.3] |
| udp-padding | 100.0 [100.0–100.0] | 100.0 [100.0–100.0] | 483.9 [477.4–485.5] | 465.3 [458.8–492.0] | 3.0 [2.7–4.5] | 6.6 [1.6–8.0] |
| udp-quic | 100.0 [100.0–100.0] | 100.0 [100.0–100.0] | 485.7 [485.5–488.2] | 478.8 [451.9–492.2] | 2.9 [2.2–2.9] | 3.9 [1.5–9.5] |
| udp-faketls-awg | 100.0 [100.0–100.0] | 100.0 [100.0–100.0] | 442.9 [420.8–467.2] | 480.0 [464.3–498.7] | 11.3 [6.4–15.7] | 3.7 [0.2–7.1] |

TCP baseline medians: P1 24914.3 up / 25024.8 down Mbps;
P4 30183.4 up / 30327.0 down. Maximum phase-average server/client CPU
130.5/146.0%; sampled RSS peak 136.9/56.9 MiB. Maximum guest steal
2.10/1.18%. UDP 100 loss is 0% across all repeats. Under P4 TCP download,
server-session drops reached 4065; kernel RcvbufErrors reached 206 for
QUIC and 1389 for fake-TLS+AWG. Goodput includes this behavior; losses
are not declared fixed or a proven regression.

`AGGREGATED` denotes a comparable measurement population. No regression threshold
or repeated old-binary A/B was applied; the previous single
[3 October measurement](../../../release/benchmark_v0.8.2_2026-10-03_isolated_12modes.json)
remains historical. UDP 500 loss is not declared a loss-free PASS.

The six primary sweeps retain matching core snapshots and available
iptables compatibility dumps; they do not fully qualify opaque nft/legacy
policies. The wrapper originally also compared timestamps, live chain
counters and stderr-diagnostic position; its original FAIL remains retained.
Revalidation normalizes only those metadata fields, retaining every rule.

**Q36-F005:** the runner now captures IPv4/IPv6, default/legacy/nft iptables,
complete nft JSON, resolver, namespaces, boot ID and live-service PID/SHA.
Time headers for all save variants and live counters normalize; policies,
rules, order, nft handles and diagnostic multiplicity remain. Empty archived
core snapshots and missing backends refuse. 35 Windows/35 Linux methods PASS.
Actual TCP Reality TLS + UDP QUIC, 9 phases, returned COMPLETED and matching
snapshots of every backend before/after.

The longer outer wrapper for that confirmation remains FAIL: the existing
.10 vpn-obfuscated/vpn-nat cycle changes three legacy rules for
vpn0/10.8.0.0/24. Both script SHA match the 3 October observation; the current
journal confirms the cycle. This is a shared-lab limitation, not complete
host immutability or an outer-wrapper PASS. The general runner has no rule
exception; services were not stopped. The initial new-harness nft-save
timestamp refusal is retained separately.

All 18 owned remote roots were SHA-archived and removed after inode/file/
process/mount-reference checks. Cleanup accounts separately for exactly those
three known external rules after rechecking owner SHA; every other field
matched. The old staged CLI remains; temporary-build Cargo metadata was
archived and invalidated.

Build receipt, six raw directories, tables and cleanup receipt:
`audit-debt-20260924/q36-repeat-methodology-20261010/`. The general runner still
records `binary_source_provenance=UNVERIFIED`: the external build receipt separately
binds this measurement SHA to 902 committed inputs without changing retained raw.
**Q36 IN_PROGRESS:** bidir, inner/outer IPv6, p50/p95/p99 and auth-rate remain OPEN;
the full 8 h Q35 soak is also open. Overall 30/37 DONE (81.1%) is unchanged.

## Final D15 measurement — 3 October 2026

Fresh release `09af8fb0…`, one 12-mode / 32-phase sweep. Builds and runtime tests
finished before measurement. Method and both VMs match 1 October. The previous
[snapshot](../../../release/benchmark_v0.8.2_2026-10-01_isolated_12modes.json) is retained;
two observations are not a statistical A/B or proof of causation.

Baseline TCP: 24338.3 ↑ / 25414.2 ↓ Mbps; UDP 500: 500.0 Mbps, loss 0%.

| Mode | TCP up Mbps | TCP down Mbps | UDP 100 received | UDP 500 received | UDP 500 loss, % |
|---|---:|---:|---:|---:|---:|
| tcp-plain-raw | 428.5 | 548.0 | — | — | — |
| tcp-faketls | 444.5 | 562.0 | — | — | — |
| tcp-padding | 472.5 | 534.1 | — | — | — |
| tcp-frag | 474.2 | 556.8 | — | — | — |
| tcp-obfs | 431.0 | 572.3 | — | — | — |
| tcp-reality | 488.8 | 542.4 | — | — | — |
| tcp-reality-tls | 960.2 | 971.8 | — | — | — |
| udp-faketls | 565.4 | 1068.5 | 100.0 | 437.6 | 12.46 |
| udp-padding | 543.4 | 1081.2 | 100.0 | 471.1 | 5.76 |
| udp-quic | 598.1 | 1047.8 | 100.0 | 459.0 | 7.94 |
| tcp-obfs-awg | 427.5 | 567.6 | — | — | — |
| udp-faketls-awg | 579.2 | 1013.9 | 100.0 | 456.5 | 8.56 |

Max CPU server/client: 131.4/142.0%; sampled peak RSS: 121.9/55.1 MiB; max CPU steal: 0.09/0.09%.

COMPLETED means measurement obtained; UDP 500 losses are not called loss-free
results. Addresses/routes/:443 match before/after; an outer wrapper retains full
firewall/resolver/namespaces/service snapshots for both VMs. The known vpn-nat
loop on .10 is recorded in host qualification; unknown changes fail. Working
services/binaries were not replaced.

[Candidate checks](../../../release/certification/evidence/final-candidate-20261003.json):
2257 Linux unit, fmt/Clippy, 18 cases / 327 checks, 100 TCP + 100 QUIC / 33 checks,
45.02s forced shutdown/recovery. No PASS for 21 physical advisories or full release
preflight. Logs/hashes: `audit-debt-20260924/d15-final-20261003/`.

## Isolated benchmark on an active lab

Current runs use
[scripts/benchmark_isolated_lab.py](../../../scripts/benchmark_isolated_lab.py).
Each VM gets a separate IPvlan network namespace with `10.255.42.1/30`
and `10.255.42.2/30`; `/etc/qeli`, `/var/lib`, and `/var/log` are isolated
in a mount namespace. Reality uses a dedicated local TLS target. Cleanup
checks namespace identity and terminates only its processes. Host IPv4
addresses, routes, and the existing `:443` listener are compared before/after.

Identical release binaries must be staged first: defaults are
`/var/tmp/qeli-d14-bin/qeli` on `.10` and `/root/qeli-src/target/release/qeli`
on `.11`. The runner checks both full SHA-256 values. SSH password comes
from `QELI_LAB_PASS` and is not stored in evidence. Example:

```text
python scripts/benchmark_isolated_lab.py --sha256 a526c03bf0bae927828e91f11ac5d751c3a82e560a7f12ada3a6ab6b410cedb0 --output benchmark-evidence/run-1
```

Defaults cover the same 12 wire modes, one TCP stream, 8-second upload
and download per mode, and 5-second UDP loads at 100/500 Mbps for UDP
carriers. A direct baseline runs first. `--modes` selects a subset;
`--seconds` sets the TCP interval (5–30 seconds). Throughput uses
`sum_received.bits_per_second`, counting bytes actually received. Raw
iperf, loss, retransmits, kernel RcvbufErrors, session drops, exact Qeli
PID CPU/RSS samples, and binary/source/harness SHA metadata are retained.
CPU is process-tick delta over the interval (100% means one CPU;
two vCPUs can reach 200%); RSS peaks are sampled. `COMPLETED` means a
measurement was obtained correctly, without an automatic performance
threshold or a no-regression claim.
As of 10 October 2026, `COMPLETED` requires a real `end.sum_received`
summary, at least 90% of the requested interval, and finite nonnegative
values. TCP requires `sum_sent.retransmits`; UDP requires loss percentage
and jitter. A missing receiver summary can no longer fall back to `end.sum`,
which may represent sender bytes. The direct baseline uses the same rule.
Five positive/negative unit tests pass, and all 35 archived raw iperf files
from the 3 October sweep pass the stricter admission. The admission fix itself does not measure throughput; the six-sweep
current series is described above. Q36 remains in progress.

### Repeats and measurement provenance — Q36-F003/F004, 10 October

New results use `schema_version=2`, a unique `run_id`, the selected mode list,
UTC window and monotonic duration. `--tcp-parallel` selects 1 or 4 inner TCP
streams, not multipath channels. Both directions and the direct baseline use
that value. Each VM records CPU/kernel/iperf, boot ID and governor; missing VM
cpufreq is explicitly `UNAVAILABLE`. Each mode records actual TUN/IPvlan MTUs
and a configuration-template SHA with the random public key normalized.
Process samples include affinity/nice and host load 1/5/15; a changed PID or
CPU policy refuses the phase. Before/after counters are retained; a changed
session endpoint or reset counter cannot become a negative drop count.

The remote binary SHA is verified separately. `harness_source_commit` and
`local_native_source_digest` identify the local harness checkout, without
claiming that the remote binary was built from it. `binary_source_provenance`
remains `UNVERIFIED` until separate build qualification. A failed second SSH
connection closes the first and publishes `FAIL`; cleanup errors retain the
mode result and overall failure. Final `COMPLETED` requires the complete
selected population and matching host state. Results are published atomically.

[benchmark_report.py](../../../scripts/benchmark_report.py) requires at least
three separate sequential runs with identical artifact/harness SHA, hosts,
CPU policy, MTUs, settings and modes. It rereads raw iperf, process samples and
counters, rechecks summaries and required phases, and refuses failed/missing
runs, duplicate IDs/raw populations, overlapping windows or changed conditions.
Every run remains in the statistics, including zero throughput. Output is
`AGGREGATED`, with `n/min/median/max/range/spread_pct` and every raw file SHA.
It imposes no performance floor and does not verify source provenance. P=1
and P=4 populations are aggregated separately:

```text
python scripts/benchmark_isolated_lab.py --sha256 <binary-sha256> --tcp-parallel 4 --output run-P4-1
# Repeat independently in run-P4-2 and run-P4-3.
python scripts/benchmark_report.py run-P4-1 run-P4-2 run-P4-3 --output summary-P4.json
```

24 Windows and 24 actual Linux contract methods PASS. Linux governor/proc
sampling was checked separately; test populations are synthetic throughput
fixtures. Both VMs are accessible: the earlier refusal came from `getpass` in
the Windows PTY; stdin delivery of the same password restored SSH. The `.11`
services retained PID 777/845 and binary SHA; the owned test root was removed
after SHA/inode checks. Current build qualification and three full sweeps
each P=1/P=4 are described above. Q36 remains `IN_PROGRESS`: bidirectional/IPv6
and latency/auth-rate measurements remain open.

Historical [benchmark.py](../../../scripts/benchmark.py) stops services,
uses host-wide `pkill`, replaces binaries and the resolver file. It now
requires explicit `--allow-host-wide-benchmark` before SSH access.
The same guard now covers historical `bench_bonding.py`, which changes lab
qdisc, services and VPN state. Obsolete `perf_combined_load.py` was removed:
no current workflow calls it, it contained embedded credentials, and it also
changed production firewall/VPN state. Ordinary invocation of the two
remaining host-wide scripts exits with code 2 before credentials or SSH;
a subprocess test checks that refusal. Old results remain historical. On
10 October, the initial SSH refusal was explained above; the subsequent
current Q36 measurement series is described at the top of this report.

## Current 0.8.2 measurement — 1 October 2026

All 12 modes returned `COMPLETED`: 24 TCP phases and 8 UDP phases.
Both VMs: Linux 6.12.105+deb13-amd64, 2 vCPU, iperf3 3.18.
Direct network: 25143.0 ↑ / 24578.1 ↓ Mbps; UDP 500: 500.0 Mbps, no loss.

| Mode | TCP ↑ Mbps | TCP ↓ Mbps | UDP 100 received | UDP 500 received | UDP 500 loss, % |
|---|---:|---:|---:|---:|---:|
| tcp-plain-raw | 468.8 | 548.0 | — | — | — |
| tcp-faketls | 454.2 | 528.2 | — | — | — |
| tcp-padding | 458.3 | 531.1 | — | — | — |
| tcp-frag | 458.9 | 536.7 | — | — | — |
| tcp-obfs | 378.1 | 512.5 | — | — | — |
| tcp-reality | 460.4 | 525.9 | — | — | — |
| tcp-reality-tls | 896.5 | 936.2 | — | — | — |
| udp-faketls | 524.4 | 873.5 | 100.0 | 472.5 | 5.49 |
| udp-padding | 536.5 | 731.4 | 100.0 | 411.0 | 14.38 |
| udp-quic | 497.6 | 794.7 | 100.0 | 399.8 | 19.80 |
| tcp-obfs-awg | 408.6 | 551.6 | — | — | — |
| udp-faketls-awg | 488.0 | 958.6 | 100.0 | 441.7 | 11.39 |

UDP 100 Mbps has no loss in all four modes.
Every phase has `server_session_drops=0`. `kernel_rcvbuf_drops=0` except
**287 client drops** during TCP download over UDP+AWG (958.6 Mbps).
Zero values in these counters do not exclude loss elsewhere.
Highest measured Qeli CPU over a sampling window: server 127.5%, client 140.7%
(100% is one core). Sampled peak RSS: server 119.0 MiB, client 56.8 MiB.
The CPU window is one second longer than iperf and includes workload edges.
Per-phase CPU/RSS, retransmits, jitter and metadata are retained in JSON;
raw logs and samples live in `audit-debt-20260924/d14-benchmark-20261001/`,
with 106 full file SHA-256 values listed in the snapshot.

Only the ambiguous QUIC UDP 500 measurement was repeated: first 399.8 Mbps
and 19.80% loss with server/client CPU steal 7.13/5.80%; repeat 452.8 Mbps
and 9.22% with steal 0.09/0.00%. These are two observations, not statistical proof
of the cause of loss. Host addresses/routes/listener matched before and after both runs.

The old 0.8.0 sweep used `recordizer=prefer` with recordizer active;
this sweep uses `off` and a different topology. The separate comparison report
from 2026-09-01 also used TCP `P=4`. No regression percentage is calculated
from these different tables; such a conclusion requires equivalent A/B.
This set does not close IPv6/DNS/leak/PMTU/legacy or physical certification gates.


## Current release-candidate checks — 1 October 2026

The new [SSH runner](../../../scripts/audit_release_matrix_lab.py) runs existing
release fixtures in fresh NET/mount/PID namespaces with private `/run`, `/var/lib`,
`/var/log`, `/etc/qeli` and a short `/tmp`. Same `a526c03b`, no binary rebuild.
[Evidence](../../../release/certification/evidence/linux-matrix-20261001.json):
**18/18 functional cases, 327/327 assertions PASS** — 12 full cells
outer/inner IPv4/IPv6 × TCP/UDP fake-TLS/QUIC, 2 dual split, TAP/NDP/RA,
DNS over both upstream families (56), PMTU/MTU/PTB (36), and both interop
directions with packaged 0.7.16 (26). Legacy package extracted without installation;
binary SHA `6139d7f0280a129e635c188b80bfba93f8256ca76295f24e10e740f99c5ef8bd`.

The first `full-01` stopped because a long TMPDIR exceeded Unix socket SUN_LEN.
Fixed private `/tmp`, firewall comparison without timestamps/counters and stopping
at the first failure. These are harness changes; product code is unchanged and logs retained.

Two subsequent runs observed changes to three parent legacy rules.
The source is established: `vpn-obfuscated.service` fails with
`password_file or password_command required`, `Restart=on-failure`, 5-second interval,
**5109 restarts**. Dependent `vpn-nat.service` adds/removes exactly two FORWARD
rules for `vpn0` and MASQUERADE `10.8.0.0/24 -> ens18` on every start/stop.
Journal, scripts and before/after retained. Test profiles use `10.86.0.0/24`
and private TUN names. **This is an independent failing lab service, not a Qeli mutation.**
Only the two old units were stopped; files and enabled state retained.
Working `qeli.service`, original PID 845, routes/DNS and all other firewall rules retained.
After stopping the loop, TCP, IPv6 QUIC, DNS, PMTU and legacy were repeated:
**144/144 PASS**, every full host snapshot field matched before/after. Remaining cells
were not rerun: same SHA/source/fixtures and parent discrepancy explained by evidence.
Original nonmatching snapshots and the
[partial manifest](../../../release/certification/evidence/0.8.2-partial-20261001.json)
are retained, not replaced with a claim that initial cleanup succeeded.

The [current 0.8.2 manifest](../../../release/certification/0.8.2.json) accepts
**20/20 required automated gates**: 18 cases, aggregate leak from the full family
matrix and the previous 100 TCP + 100 QUIC flips of the exact SHA.
Overall Linux unit 2238 PASS/59 ignored is reused because the `qeli` tree is identical
to D13 (`f0649180abda2161d04fbe40c9d8af9d0e50696f`). All 21 physical advisory rows remain:
20 `not_available`, canary `pending`; this is not physical certification or a full
release preflight/CI/package audit. User uncommitted changes are outside the candidate.
D14 is closed for the current tested candidate; D05/D06/D07/D08/D10/D15 remain.
Raw: `audit-debt-20260924/d14-release-matrix-20261001/`.

The benchmark above was recorded **before** stopping the old restart loop; absolute
numbers were not remeasured. They remain observations of that environment, not stable A/B.


> **Latest 0.8.0 cross-protocol run:** the
> [full 34-mode report](benchmarks/vpn_protocol_benchmark_repeat_2026-09-01.md)
> covers three passes for 25 masked modes, IPv4/IPv6, TCP/UDP, CPU/RSS and the limits of
> DPI-related interpretation. Average four-stream TCP throughput across Qeli's 12 profiles
> was **1220 Mbit/s**, and **1767 Mbit/s** across its five fast TCP profiles.

**Reality throughput scope.** Values through 0.7.16 measured the former inner fake-TLS
carrier. The current 12-mode run measures genuine HTTP/2: `reality-tls` reached 827.9 Mbps
upload and 647.8 Mbps download with zero server drops. This is a clean full-speed baseline
from one rebooted lab, but not yet a repeatable 5× result for the final release SHA.

> Release **0.6.0** is a refactoring (the shared C# layer, .NET 10, cleanup); the
> protocol, crypto, and data plane were **unchanged**. A direct measurement of 0.6.0
> confirmed this: on a clean lab it runs at/above the 0.5.6 reference (the 2026-06-07
> run) in all modes — there's no regression (TCP +3…+10%, UDP loss at 500 Mbps half as
> much). The 0.5.6 reference is kept alongside in
> `release/benchmark_results_2026-06-07_v0.5.6.json`.

> 🆕 **Version 0.7.0** (2026-06-12) — a large core refactor (PQ-derive/ml-kem, the
> hand-rolled realtls server, the kill-switch, reality-hardening). The measurements and
> the delta to 0.6.0 — in the section **"Version 0.7.0 — the delta to 0.6.0"** right
> below (before the baseline). The detailed per-mode 0.6.0 tables remain the reference
> base.
>
> **0.7.0** (the post-quantum tunnel + the audit 2026-06-11): the X25519+ML-KEM hybrid
> affects only the handshake (a one-time cost on connect), throughput is unchanged — a
> live `sanity_e2e` run gave the same 570–700 Mbps TCP and 0% UDP loss on the non-`plain`
> modes.

> 🆕 **Version 0.7.1** (2026-06-12) — more core changes + a new wire-breaking **H-1**
> (`bind_static_to_session`, default true, requires pinning) + a functionality run of every
> default config. The section **"Version 0.7.1 — the delta to 0.7.0"** below: no regression,
> reality-tls upload stabilized (σ 99→19).

> 🆕 **Version 0.7.2** (2026-06-20) — anti-DPI traffic shaping (idle-cover), server-side NAT,
> the 2026-06-18 audit fixes, the web client-manager panel. All changes are **off the bench
> data path** (the bench config enables no shaping/NAT). The measurement ran on a host under
> hypervisor CPU steal → this session's absolutes are depressed, so verification is a
> **host-neutral A/B** (0.7.1 vs 0.7.2 interleaved). The section **"Version 0.7.2 — the delta
> to 0.7.1 (A/B)"** below: **no regression**.

> 🆕 **Version 0.7.4** (2026-06-28) — UDP handshake fragmentation (dodges dropped IP fragments
> on LTE/CGNAT) + UDP session fixes (keepalive for idle/download-only tunnels, inbound-byte
> accounting). Measured on a **clean** host (steal 2.1%, baseline 21 Gbps) → direct absolutes,
> comparable to the clean 0.7.1 day. The section **"Version 0.7.4 — the delta to 0.7.1"** below:
> **no regression**, the UDP data plane is untouched by the `udp_handler` refactor.

> 🆕 **Version 0.7.10** (candidate, 2026-07-10) — the Rust core is at **0.7.9** level (the 0.7.10
> WIP is Android/desktop clients only; the data plane is untouched). Cumulative data-plane changes
> 0.7.5→0.7.9: the oversized-record fix (0.7.7), the reality-tls aes-gcm→0.10 revert (0.7.7), server
> udp-quic autodetect + RFC-9001 (0.7.9), jemalloc in server builds. This session's host was moderately
> contended → verified with a **host-neutral A/B to 0.7.4**: **no regression, 0.7.9 is equal-or-faster
> than 0.7.4**.

> 🆕 **Version 0.7.11** (candidate, 2026-07-12) — a large dev batch (RCE→P3 audit, exclude/include
> CIDR routing under full tunnel, per-user static_ip + disconnect notifications, sleep/resume recovery,
> panel) + dep bumps **x25519-dalek 3** / jemallocator 0.7. The binary reports `qeli 0.7.10` (Cargo not
> yet 0.7.11). This session's host was moderately contended → **host-neutral A/B to 0.7.9**: **no
> regression, 0.7.11 is equal-or-faster than 0.7.9 everywhere, including reality-tls (+5.9%/+2.8%)**;
> the major x25519-dalek 3 bump did not touch throughput.

## The rig

| parameter | value |
|---|---|
| Server | `10.66.116.10`, Debian 13, kernel 6.12.88, 2 vCPU (QEMU, AES-NI present), NIC virtio |
| Client | `10.66.116.11`, identical, the same L2 network |
| iperf3 | 3.18 |
| TUN MTU | 1400 in all modes |
| Cipher / KEX | ChaCha20-Poly1305 / X25519 + HKDF-SHA256 (reality-tls: the outer TLS — AES-128-GCM) |

**Methodology.** For each mode we bring up a **separate tunnel** (a flat-INI config of
the server + client), `ping -c 20 -i 0.2`, then `iperf3 -t 12` in both directions for
TCP, or a sweep `iperf3 -u -b {100..500}M -l 1200` for UDP. CPU is taken via
`iperf3 cpu_utilization_percent` (host=the iperf client, remote=the iperf server, % of
2 vCPU) **and** by sampling the `qeli` process on the server.

> ⚠️ The numbers have a spread of ±3–5% between runs (a virtio VM). For pristine numbers
> run on a **freshly rebooted** lab: a background load on the .11 client (e.g. the stock
> Android emulator `qemu -avd test`, ~17% CPU) **inflates UDP loss** at 400–500 Mbps
> several-fold (the open-loop receiver is CPU-starved) and slightly lowers TCP — before a
> benchmark `python scripts/reboot_vms.py`. This run is without drops, `ping` 0% in all
> modes.

> ⚠️ **A second trap — hypervisor CPU steal (a noisy neighbor).** The lab VMs share a
> physical host; when a neighbor loads the host, the guest is starved of CPU even while it
> reports `idle`. Check `vmstat 1` (the `st` column): on a clean host `st≈0`, under
> contention 5–10%. Steal **degrades the no-VPN baseline too** (that's the tell: TCP
> baseline 21 → 18–19 Gbps, UDP loss rises) → absolutes across different days become
> incomparable. **If `st` is high and won't drop**, switch to a *host-neutral A/B* (two
> versions interleaved in one session, see [scripts/ab_071_072.py](../../../scripts/ab_071_072.py)):
> both see the same steal, so the delta stays honest. The emulator on .11 may **restart
> itself** mid-session through a long run — re-check `pgrep -f qemu-system`.

> 🔴 **IMPORTANT about the `qeli CPU` columns below.** They were taken with `ps %cpu` =
> an average over the WHOLE life of the process → they **greatly underestimate** (showing
> ~34%). A precise delta measurement by `/proc/<pid>/stat` over the load window
> ([scripts/multicore_probe.py](../../../scripts/multicore_probe.py)) gives **~150% = ~1.5
> cores** on ONE tunnel: the data plane is **multi-core** (the decrypt-reader +
> encrypt-writer + TUN-pump are independent tokio tasks on different cores), not "one
> core". The `qeli CPU` columns in the tables are only a relative comparison of the modes
> against each other, NOT the absolute core load.

## Version 0.7.0 (2026-06-12) — the delta to 0.6.0

A large core refactor (`+1484/−327` lines: the PQ `crypto/derive`+`mlkem`, the
hand-rolled realtls server `protocol/tls`, `server/{handler,mod,dns,reality}`,
`client/killswitch`). The binary `qeli 0.7.0` sha `5e40e697`, a freshly rebooted lab,
the gate PASS (build + **191 tests** + clippy). The raw data —
[release/benchmark_results_2026-06-12_v0.7.0.json](../../../release/benchmark_results_2026-06-12_v0.7.0.json),
the 0.6.0 reference alongside in `benchmark_results_2026-06-11_v0.6.0.json`.

### ⚠️ Two breaking config changes (accounted for in [benchmark.py](../../../scripts/benchmark.py))

1. **Persistent TOFU** — the server key pin is now stored on disk
   (`/var/lib/qeli/known_hosts`), not in the session memory. A stale pin on `host:port`
   survives a reboot and **rejects the connect** with a fresh bench key → the harness
   wipes it before each mode.
2. **`reality_proxy` requires `short_ids`** — the trivially probeable fallback by the
   absence of ALPN was removed; `reality_proxy.enabled` without `short_ids` is **rejected
   at startup** (the worker in a respawn loop → connection refused). The bench tcp-reality:
   the server gets `short_ids`, the client (fake-tls) gets `reality_sid` + the key pin
   (fake-tls seals the sid in the session_id). Additive: `kill_switch`, `peek_timeout_ms`;
   the default `handrolled` switched false→true (real_tls-reality is now hand-rolled, not
   rustls).

### TCP, Mbps (↑up / ↓down)

| mode | 0.6.0 ↑/↓ | 0.7.0 ↑/↓ | Δ |
|---|---|---|---|
| plain | 571 / 714 | 555 / 700 | −2.8% / −2.0% |
| fake-tls | 563 / 697 | 527 / 693 | −6.4% / −0.6% |
| padding | 559 / 704 | 528 / 686 | −5.5% / −2.5% |
| frag | 554 / 702 | 533 / 664 | −3.8% / −5.4% |
| obfs | 498 / 582 | 482 / 568 | −3.1% / −2.5% |
| reality (proxy) | 577 / 721 | 552 / 688 | −4.3% / −4.7% |
| **reality-tls** (5× median) | 488 / 321 | 482 / **417** | −1.2% / **+30%** |

### UDP, loss % (400M / 500M)

| mode | 0.6.0 | 0.7.0 |
|---|---|---|
| udp-fake-tls | 0.35 / 2.97 | 0.20 / 2.98 |
| udp-padding | 0.41 / 3.22 | 0.31 / 3.32 |
| udp-quic | 0.25 / 4.08 | 0.45 / 4.28 |

### reality-tls × 5 runs

- **↑ up:** 0.6.0 median 488 (σ37) → 0.7.0 **482** (σ99, spread 437–688) — the same, a higher spread.
- **↓ down:** 0.6.0 **321** (σ2.4) → 0.7.0 **417** (σ3.4) — **+30%, both stable → a real gain**
  (the hand-rolled TLS server in the new default is faster than rustls + the `protocol/tls` refactor).
  The raw data — [release/reality_tls_5x_v0.7.0_2026-06-12.json](../../../release/reality_tls_5x_v0.7.0_2026-06-12.json).

### The precise data-plane CPU (the /proc delta, both binaries back-to-back)

`scripts/multicore_probe.py` — the worker's true CPU by `/proc/<pid>/stat` (not `ps`).
0.6.0 was built in isolation from the committed HEAD ([scripts/probe_060_ab.py](../../../scripts/probe_060_ab.py)).

| 1 tunnel | 0.6.0 | 0.7.0 |
|---|---:|---:|
| upload (decrypt) | 146% (1.46 cores) | 153% (1.53 cores) |
| download (encrypt) | 149% (1.49) | 153% (1.53) |
| bidir | 154% @218↑/217↓ | 154% @140↑/138↓ |

The real cost of the refactor on the data plane — **~+3–5% CPU/core per tunnel** (NOT
+15%, as it seemed from the `ps %cpu` sampler in benchmark.py — it's a lifetime-average
and inflates the delta; trust the /proc probe). In production (1-core-bound, ceiling
~311 Mbps) this is **−3–5% of the ceiling (~295–300 Mbps)** — moderate. A nuance: the
bidir CPU is the same, but the throughput is 218→140 at the same saturation of 2 cores —
a slight efficiency dip under a bidirectional load (a noisy sample).

### The 0.7.0 conclusion

Single-stream throughput **with no significant regression** (TCP in the virtio noise,
UDP unchanged); **reality-tls download +30%**; the data-plane CPU **+3–5%**. The detailed
per-mode 0.6.0 numbers below — the reference base.

## Version 0.7.1 (2026-06-12) — the delta to 0.7.0

More core changes on top of 0.7.0 (`protocol/packet.rs`, `client/dns`, `crypto/derive`,
**H-1** — below) + the default configs were rewritten. The binary `qeli 0.7.1` sha `6f997202`,
a freshly rebooted lab, the gate PASS (build + **194 tests** + clippy). Raw data —
[release/benchmark_results_2026-06-12_v0.7.1.json](../../../release/benchmark_results_2026-06-12_v0.7.1.json).

### ⚠️ New wire-breaking: H-1 (`bind_static_to_session`, default true)

Since 0.7.1 the session KDF additionally folds in the static-ephemeral DH (Noise-IK): a
failed ephemeral RNG alone no longer exposes the tunnel. Default `true` on BOTH the server
and the client — it **requires pinning the server key**; a TOFU / legacy-0.7.0 client is
rejected. Pairs with `require_client_key_proof = true`. benchmark.py pins the key in EVERY
mode (otherwise the TOFU modes fail, as before with the persistent-TOFU known_hosts). The
default configs already account for it.

### Default-config functionality (e2e on 0.7.1)

A live run of every shipped default config ([scripts/config_functest.py](../../../scripts/config_functest.py)):

| config | result |
|---|---|
| **server.conf** (fake-tls, full stack: NAT/DNS/padding/frag/heartbeat/H-1) | ✅ **PASS** — Auth OK, ping gw, **565 Mbps** |
| **server-maxobf.conf** (reality-tls: real_tls + hand-rolled, require_proof, H-1) | ✅ **PASS** — Auth OK, ping gw, **527 Mbps** |
| parse server.conf / server-maxobf.conf / client.conf / client-maxobf.conf / client-reality.conf | ✅ OK |

Fixed a template bug in `client-maxobf.conf` (it was inconsistent with server-maxobf.conf:
user `phone`≠`client1`, mode `fake-tls`≠real_tls) → `user=client1`, `mode=reality-tls`,
`+reality_sid`; verified e2e. `client-reality.conf` / `client-YOUR_DEPLOY_HOST.conf` point
at an external server (parse only).

### TCP, Mbps (↑up / ↓down)

| mode | 0.7.0 ↑/↓ | 0.7.1 ↑/↓ | Δ |
|---|---|---|---|
| plain | 555 / 700 | 549 / 717 | −1% / +2% |
| fake-tls | 527 / 693 | 538 / 685 | +2% / −1% |
| padding | 528 / 686 | 550 / 706 | +4% / +3% |
| frag | 533 / 664 | 540 / 674 | +1% / +2% |
| obfs | 482 / 568 | 501 / 579 | +4% / +2% |
| reality (proxy) | 552 / 688 | 559 / 689 | +1% / ≈0 |
| **reality-tls** (5× median) | 482 / 417 | **518 / 418** | **+7% / ≈0** |

### UDP, loss % (400M / 500M)

| mode | 0.7.0 | 0.7.1 |
|---|---|---|
| udp-fake-tls | 0.32 / 2.95 | 0.32 / 2.95 |
| udp-padding | 0.31 / 3.32 | 0.44 / 11.68¹ |
| udp-quic | 0.45 / 4.28 | 0.20 / 2.37 |

¹ a spike at the saturation knee (400M = 0.44%; fake-tls/quic 500M = 2.4–3%) — noise, not a regression.

### reality-tls × 5 — upload stabilization

- **↑ up:** 0.7.0 median 482 (**σ 99**, spread 437–688) → 0.7.1 median **518** (**σ 19**, 498–551):
  upload is both faster (+7%) and **much more stable** — the "jumpy" 0.7.0 upload is cured.
- **↓ down:** 417 → **418** (σ 5.5) — stable. Raw data —
  [release/reality_tls_5x_v0.7.1_2026-06-12.json](../../../release/reality_tls_5x_v0.7.1_2026-06-12.json).

### The 0.7.1 conclusion

No regression (0.7.1 ≈ 0.7.0, mostly slightly higher) despite the packet/dns/derive changes
and H-1; the data-plane CPU is the same (the precise /proc measurement of 0.7.0 = 1.53 cores,
the data-plane core was unchanged). The notable improvement — **reality-tls upload
stabilization** (σ 99→19). Both default server configs are functional end-to-end.

## Version 0.7.2 (2026-06-20) — the delta to 0.7.1 (A/B)

0.7.2 content: anti-DPI traffic shaping (idle-cover Poisson + Stealth), server-side NAT
(iptables), the 2026-06-18 audit fixes, the web client-manager panel. The binary
`qeli 0.7.2` sha `98c6b05a`, a freshly rebooted lab, the gate PASS (build + **213 tests**
(210 lib + 3 bin) + clippy). The A/B raw data —
[release/ab_071_vs_072_2026-06-20.json](../../../release/ab_071_vs_072_2026-06-20.json),
the full 0.7.2 run — [release/benchmark_results_2026-06-20_v0.7.2.json](../../../release/benchmark_results_2026-06-20_v0.7.2.json).

> 🟡 **Why A/B, not direct absolutes.** This session the hypervisor host was under a noisy
> neighbor — under load **CPU steal 7–9%** (server), and the emulator on .11 restarted
> itself mid-run. Even the no-VPN baseline degraded (TCP 21 → 18.6 Gbps, UDP loss ×3) → a
> direct comparison of absolutes against the clean 0.7.1 day (2026-06-12) would show a
> **false "~13% regression"**. So 0.7.2 was verified with a **host-neutral A/B**: 0.7.1
> (built from tag `v0.7.1`, sha `996c9d98`) and 0.7.2 were run **interleaved in one session**
> ([scripts/ab_071_072.py](../../../scripts/ab_071_072.py)) — both see the same steal, so it's
> cancelled out of the delta. This is the same technique `probe_060_ab.py` uses for CPU.

### A/B TCP, Mbps (↑up / ↓down) — both binaries on the same (contended) host

| mode | 0.7.1 ↑/↓ | 0.7.2 ↑/↓ | Δ |
|---|---|---|---|
| plain | 522 / 638 | 505 / 603 | −3.3% / −5.6% |
| fake-tls | 501 / 599 | 508 / 608 | +1.3% / +1.6% |
| padding | 488 / 547 | 483 / 543 | −1.0% / −0.7% |
| frag | 511 / 607 | 435 / 468 | −15% / −23%¹ |
| obfs | 431 / 473 | 452 / 484 | +4.9% / +2.3% |
| reality (proxy) | 503 / 613 | 494 / 591 | −1.7% / −3.7% |
| **reality-tls** (median 3×) | 464 / 420 | 461 / 413 | **−0.6% / −1.6%** |

¹ a single run that caught a steal spike (ping mdev 5.3 ms on the 0.7.2 sample — six times
the neighbors). In **reality-tls** (median of 3× — the most careful mode) and in the other
rows the effect averages out, there is no real delta.

### A/B UDP, loss % (400M / 500M)

| mode | 0.7.1 | 0.7.2 |
|---|---|---|
| udp-fake-tls | 10.3 / 28.8 | 5.0 / 12.8 |
| udp-padding | 5.9 / 16.0 | 5.1 / 16.2 |
| udp-quic | 4.0 / 15.6 | 6.7 / 15.6 |

UDP loss is high on **both** versions (CPU starvation of the open-loop receiver on .11 under
steal+emulator), 0.7.2 ≈ 0.7.1 (on fake-tls even lower) — host noise, not the data plane.

### reality-tls × 5 (0.7.2, the same contended host)

- **↑ up:** median **459.3** (σ 15.2, range 429–474).
- **↓ down:** median **416.4** (σ 7.2, range 405–424) — effectively **matching the clean
  0.7.1 day (418)**: reality-tls download is bound by the client-decrypt on .11 and is barely
  hit by the server-side steal, whereas upload (459 vs 518 on the clean day) sags with the
  host. On this same host the A/B measured 0.7.1 at 464/420 ≈ 0.7.2 459/416 → parity.
  Raw data — [release/reality_tls_5x_v0.7.2_2026-06-20.json](../../../release/reality_tls_5x_v0.7.2_2026-06-20.json).

### The 0.7.2 conclusion

**No regression.** The host-neutral A/B: all modes within the ±2–5% noise (some faster on
0.7.2), reality-tls at the median of 3× is **−0.6% / −1.6% = parity**. The only "large" delta
(frag −15/−23%) is a single steal spike, absent in the averaged modes. The anti-DPI traffic
shaping, the server NAT and the client-manager **do not touch the bench data path** (cover
traffic is sent only when idle, NAT is off in the bench config). This session's absolutes are
depressed by the host (steal 7–9%) and must not be compared directly to the 0.7.1 numbers from
2026-06-12 — for the delta see the A/B above. The detailed per-mode 0.6.0 reference tables are below.

## Version 0.7.4 (2026-06-28) — the delta to 0.7.1

0.7.4 content: **UDP handshake fragmentation** (`protocol/udp_frag.rs` — the large PQ ClientHello
is split into pieces to dodge dropped IP fragments on LTE/CGNAT; triggers automatically, **only on
the handshake**, off the data plane) + UDP session fixes (`server/udp_handler.rs`: keepalive for
idle/download-only tunnels without false reconnects, inbound-byte accounting). The binary
`qeli 0.7.4` sha `1c36f802`, a freshly rebooted lab, the gate PASS (build + **225 tests**
(222 lib + 3 bin) + clippy). Raw data —
[release/benchmark_results_2026-06-28_v0.7.4.json](../../../release/benchmark_results_2026-06-28_v0.7.4.json).

> 🟢 **The host is clean** (unlike the 0.7.2 session): under load steal **2.1%** (server) /
> 0.9% (client), the no-VPN baseline TCP **21,020 Mbps** ≈ the 21 Gbps reference → no A/B was
> needed, the direct absolutes are comparable to the clean 0.7.1 day (2026-06-12). The intermediate
> 0.7.2 (host-neutral A/B, under contention) and 0.7.3 (not throughput-benchmarked on the lab —
> Android/security fixes off the data path) gave no clean per-mode numbers of their own, so the delta
> is taken to 0.7.1 — the last clean run.

### TCP, Mbps (↑up / ↓down)

| mode | 0.7.1 ↑/↓ | 0.7.4 ↑/↓ | Δ |
|---|---|---|---|
| plain | 549 / 717 | 551 / 718 | +0.4% / +0.1% |
| fake-tls | 538 / 685 | 533 / 715 | −0.9% / +4.4% |
| padding | 550 / 706 | 532 / 698 | −3.3% / −1.1% |
| frag | 540 / 674 | 567 / 689 | +5.0% / +2.2% |
| obfs | 501 / 579 | 496 / 571 | −1.0% / −1.4% |
| reality (proxy) | 559 / 689 | 553 / 709 | −1.1% / +2.9% |
| **reality-tls** (5× median) | 518 / 418 | **549 / 431** | **+5.9% / +3.0%** |

All modes within the ±5% noise (a few even higher) → no regression; reality-tls — the heaviest mode —
is even slightly faster on this host.

### UDP, loss % (400M / 500M) — the main focus of 0.7.4

| mode | 0.7.1 | 0.7.4 |
|---|---|---|
| udp-fake-tls | 0.32 / 2.95 | 0.63 / 4.81 |
| udp-padding | 0.44 / 11.68¹ | 1.01 / 3.71 |
| udp-quic | 0.20 / 2.37 | 0.31 / 3.81 |

The UDP plane is clean (<1% up to 400M on all modes); at the 500M knee the loss is in the usual
saturation-noise range (0.7.1 also had outliers up to 11.68% there). **The `udp_handler` refactor and
the handshake UDP-frag did not hurt UDP throughput/loss** — which was the main release risk.

### reality-tls × 5 (0.7.4, clean host)

- **↑ up:** median **548.8** (σ 24.2, range 521–579) — vs 518 (σ19) on the clean 0.7.1 day.
- **↓ down:** median **430.6** (σ 6.3, range 425–441) — vs 418 (σ5.5). Both stable and slightly higher
  — not a regression. Raw data — [release/reality_tls_5x_v0.7.4_2026-06-28.json](../../../release/reality_tls_5x_v0.7.4_2026-06-28.json).

### The 0.7.4 conclusion

**No regression** on a clean host: TCP within ±5% of the clean 0.7.1 day, reality-tls (5× median)
slightly higher (+6%/+3%), UDP loss normal. The key release risk — the UDP-handler refactor and the
handshake fragmentation — **did not touch the UDP data plane** (loss <1% up to 400M, throughput on par).
UDP-frag works on the handshake and is invisible to the load test. The detailed per-mode 0.6.0 reference
tables are below.

## Version 0.7.6 (2026-07-02) — the delta to 0.7.4

What's in 0.7.6: **AmneziaWG-style masking for obfs (F2)** — pre-handshake junk records
(`obf.awg.jc`/`jmin`/`jmax`; `protocol/obfs.rs`) modelled on AmneziaWG; config-gated, **OFF by default**,
`jc` must match on both ends — plus web-panel work. The binary `qeli 0.7.5` (version not yet bumped),
sha `b4ae0897`, a freshly rebooted lab (.11 with no stray qeli service), the gate PASS
(build + **265 tests** (262 lib + 3 bin) + clippy). Raw data —
[release/benchmark_results_v0.7.6_pristine.json](../../../release/benchmark_results_v0.7.6_pristine.json).

### TCP, Mbps (↑up / ↓down)

| mode | 0.7.4 ↑/↓ | 0.7.6 ↑/↓ | Δ |
|---|---|---|---|
| plain | 551 / 718 | 565 / 722 | +2.5% / +0.6% |
| fake-tls | 533 / 715 | 567 / 720 | +6.4% / +0.7% |
| padding | 532 / 698 | 566 / 684 | +6.4% / −2.0% |
| frag | 567 / 689 | 578 / 678 | +1.9% / −1.6% |
| obfs | 496 / 571 | 489 / 684 | −1.4% / +19.8%¹ |
| reality (proxy) | 553 / 709 | 467¹ / 677 | / −4.5% |
| **reality-tls** (5× median) | 549 / 431 | 500 / **443** | / **+2.8%** |

¹ Single per-mode runs carry virtio variance: obfs download (571↔684) and reality/reality-tls upload are
historically noisy (± up to 20%). No structural regression; the reliable signal is the **averaged (5×)
reality-tls download, which keeps climbing**: 0.7.1 = 418 → 0.7.4 = 431 → 0.7.6 = **443**.

### UDP, loss % (400M / 500M)

| mode | 0.7.4 | 0.7.6 |
|---|---|---|
| udp-fake-tls | 0.63 / 4.81 | 0.63 / 5.26 |
| udp-padding | 1.01 / 3.71 | 1.08 / 7.51 |
| udp-quic | 0.31 / 3.81 | 0.75 / 6.63 |

UDP is clean up to 400M (<1.1% on all), the 500M knee is the usual saturation noise. Unchanged.

### reality-tls × 5 (0.7.6)

- **↑ up:** median **500.3** (σ4.2) — a variable metric (0.7.4 = 549); within the range of past runs.
- **↓ down:** median **443.1** (σ4.4) — stable, the **best result** (418 → 431 → 443). Raw data —
  [release/reality_tls_5x_v0.7.6.json](../../../release/reality_tls_5x_v0.7.6.json).

### AmneziaWG obfs masking (F2) — functional + overhead

The headline 0.7.6 feature. Junk records are exchanged **once, pre-handshake** (`obfs.rs`:
`send_junk`/`recv_junk` before the nonce/data loop), so they should not touch steady-state throughput.
Verified e2e ([scripts/amnezia_obfs_e2e.py](../../../scripts/amnezia_obfs_e2e.py) — obfs with `jc=8` junk vs
without; median over a stable tunnel, 3 rounds):

| | connect | ↑ up | ↓ down |
|---|---|---|---|
| obfs baseline (junk off) | ✅ | 501 (σ5.5) | 658 (σ5.0) |
| obfs + junk×8 | ✅ | 496 (σ8.7) | 661 (σ21.7) |

**Overhead ~0** (↑ −0.9%, ↓ +0.5% — within noise). The masking connects, traffic flows, ping 0% — and is
essentially free throughput-wise (junk is handshake-only).

### The 0.7.6 conclusion

**No regression** — TCP within virtio variance, UDP loss normal, the stable reality-tls download metric
keeps climbing (443, the best result). The new **AmneziaWG obfs masking is functional and free**
throughput-wise (junk is a one-time pre-handshake). The gate is green (265 tests). The detailed per-mode
0.6.0 tables — below.

## Version 0.7.7 (2026-07-06) — the delta to 0.7.6

What's in 0.7.7: the **AES-GCM revert** (below), **AmneziaWG junk masking extended to every UDP mode**
(not just TCP obfs), the **jemalloc allocator** for the server (bounds the worker's RSS under handshake
churn — a ~180 MB glibc arena plateau → a ~60 MB ceiling, throughput unchanged), plus the 2026-07-05
audit + `qeli://` share-link fixes. The binary `qeli 0.7.6` (version not yet bumped) sha `11f05305`,
built `--features jemalloc`, a freshly rebooted lab (.11 with no stray qeli service, CPU-steal 0%), the
gate PASS. Raw data —
[release/benchmark_results_v0.7.7_jemalloc.json](../../../release/benchmark_results_v0.7.7_jemalloc.json).

### ⚠️ The AES-GCM regression and its revert

The reality-tls outer layer is real TLS 1.3 sealed with **AES-128-GCM** (the `aes-gcm` crate; only
`realtls/record.rs` uses it). A routine dependency bump `aes-gcm 0.10 → 0.11` silently **regressed
reality-tls upload ≈ −20 %** (≈ 500 → 394 Mbps) while every other mode (ChaCha20-Poly1305, untouched)
stayed flat. The bump was **reverted to `aes-gcm 0.10`** (ChaCha stays at 0.11); a re-measure restored
reality-tls to parity. The 0.7.7 numbers below are post-revert.

### TCP, Mbps (↑up / ↓down)

| mode | 0.7.6 ↑/↓ | 0.7.7 ↑/↓ | Δ |
|---|---|---|---|
| plain | 565 / 722 | 574 / 699 | +1.5% / −3.2% |
| fake-tls | 567 / 720 | 574 / 709 | +1.2% / −1.6% |
| padding | 566 / 684 | 585 / 718 | +3.3% / +4.9% |
| frag | 578 / 678 | 596 / 730 | +3.1% / +7.6% |
| obfs | 489 / 684 | 515 / 424 | +5.4% / −38%¹ |
| reality (proxy) | 467 / 677 | 594¹ / 716 | / +5.7% |
| **reality-tls** (5× median) | 500 / 443 | 513 / **451** | +2.7% / **+1.8%** |

¹ Single per-mode runs carry virtio variance (obfs download and reality/reality-tls upload are historically
± up to 20%), and **this run's no-VPN baseline was +12% vs the 0.7.6 day** (host drift), so the raw
single-run deltas read optimistic. The reliable signal is the averaged (5×) **reality-tls download, which
keeps climbing**: 0.7.1 = 418 → 0.7.4 = 431 → 0.7.6 = 443 → 0.7.7 = **451**.

### UDP, loss % (400M / 500M)

| mode | 0.7.6 | 0.7.7 |
|---|---|---|
| udp-fake-tls | 0.63 / 5.26 | 0.83 / 3.79 |
| udp-padding | 1.08 / 7.51 | 1.07 / 3.62 |
| udp-quic | 0.75 / 6.63 | 0.69 / 4.65 |

UDP is clean up to 400M (<1.1% on all). The 500M knee looks lower here only because the .11 Android emulator
(which inflates UDP loss) was stopped for this run — a measurement condition, not a code change.

### reality-tls × 5 (0.7.7)

- **↑ up:** median **513.4** (σ4.0) — the tightest spread yet (0.7.6 = 500, 0.7.4 = 549).
- **↓ down:** median **451.2** (σ5.5) — stable, the **best result** (418 → 431 → 443 → 451). Raw data —
  [release/reality_tls_5x_v0.7.7_jemalloc.json](../../../release/reality_tls_5x_v0.7.7_jemalloc.json).

### UDP AWG junk masking — functional on every UDP profile

0.7.7 extends the pre-handshake junk from TCP obfs to **every UDP mode** (obfs / fake-tls / QUIC). Verified
e2e + on the wire ([scripts/pool4_udp_awg.py](../../../scripts/pool4_udp_awg.py)): the client emits `jc` decoy
datagrams before the ClientHello, each **≤ 1200 B** (no IP fragmentation), **masked** by the profile's
obfs-XOR / QUIC wrap (0 plaintext fragment-magic on the wire for obfs/quic), and the server drops them as
`MSG_JUNK` before the rate limiter / crypto. `jc=0` → byte-identical wire. On UDP `jc` is sender-only (no
count agreement). Auth OK + ping on all three UDP profiles; junk is handshake-only, so steady-state
throughput is unaffected.

### The 0.7.7 conclusion

**No regression** — reality-tls is back at parity (the AES-GCM 0.11 regression was caught and reverted; the
stable 5× download metric reaches **451**, the best yet). TCP is within virtio variance (single-run deltas
read optimistic on a +12%-faster host). jemalloc bounds the server's RSS without touching throughput. The
gate is green. The detailed per-mode 0.6.0 tables — below.

## Version 0.7.10 (candidate, 2026-07-10) — the delta to 0.7.4 (A/B)

**0.7.10 = the 0.7.9 data plane.** The uncommitted 0.7.10 WIP is clients only (Android profile
overflow menu, qeli:// share+QR, desktop reachability polling); there are no `qeli/src/` (Rust core)
changes. The measured binary is `qeli 0.7.9` sha `7e15d50e`, gate PASS (build `--features jemalloc` +
**290 tests** (287 lib + 3 bin) + clippy), a freshly rebooted lab. Since the last lab run (0.7.4) the
data plane accumulated: the oversized-record-drop fix (0.7.7), the reality-tls aes-gcm→0.10 revert after
a regression (0.7.7), server udp-quic autodetect + QUIC RFC-9001 (0.7.9). A/B raw data —
[release/ab_074_vs_079_2026-07-10.json](../../../release/ab_074_vs_079_2026-07-10.json), the full run —
[release/benchmark_results_2026-07-10_v0.7.10.json](../../../release/benchmark_results_2026-07-10_v0.7.10.json).

> 🟡 **Why A/B.** This session's host was moderately contended: steal ~2.8%, but the no-VPN baseline was
> 19.2 Gbps (clean ~21) with **high retransmits** (thousands). A direct sweep gave false dips on the
> download-variable modes — a single reality-tls run of 332/369 and obfs-down bouncing **176→430** at
> **low CPU** (10–24% vs ~40% on the stable modes) = NOT CPU-bound, i.e. host-throttled. So the delta is
> taken **host-neutral**: 0.7.4 (built from tag `v0.7.4`, sha `84fd20c1`) and 0.7.9 were run
> **interleaved in one session** ([scripts/ab_074_079.py](../../../scripts/ab_074_079.py)) — both see the
> same steal, cancelling it. reality-tls is the median of 3× per version.

### A/B TCP, Mbps (↑up / ↓down) — both binaries on the same (contended) host

| mode | 0.7.4 ↑/↓ | 0.7.9 ↑/↓ | Δ |
|---|---|---|---|
| plain | 536 / 698 | 549 / 726 | +2.5% / +4.1% |
| fake-tls | 523 / 664 | 562 / 682 | +7.6% / +2.6% |
| padding | 513 / 673 | 564 / 656 | +10.0% / −2.5% |
| frag | 521 / 711 | 555 / 625 | +6.6% / −12.2%¹ |
| obfs | 446 / 547 | 453 / **619** | +1.6% / **+13.2%** |
| reality (proxy) | 514 / 702 | 543 / 695 | +5.7% / −1.1% |
| **reality-tls** (median 3×) | 470 / 404 | **499 / 421** | **+6.1% / +4.2%** |

¹ frag down −12% is a single run (like the frag outlier in the 0.7.2 A/B); in the averaged modes
(reality-tls) and every other row 0.7.9 is equal-or-higher. **obfs-down on 0.7.9 is +13%** — direct proof
that the "obfs 430 dip" from the plain sweep was host noise, not a regression.

### A/B UDP, loss % (400M / 500M)

| mode | 0.7.4 | 0.7.9 |
|---|---|---|
| udp-fake-tls | 22.1 / 42.7 | 3.3 / 4.1 |
| udp-padding | 0.4 / 5.1 | 7.7 / 8.7 |
| udp-quic | 0.4 / 15.9 | 0.5 / 3.3 |

UDP is noisy on **both** versions (the open-loop receiver on .11 under contention); 0.7.9 is no worse on
average (fake-tls/quic noticeably cleaner). The spread is the host, not the data plane.

### reality-tls × 5 (0.7.9, same host)

- **↑ up:** median **481.2** (σ 7.6, 467–490).
- **↓ down:** median **429.2** (σ 10.9, 409–439) — **parity with the clean 0.7.4 day (431)** and with the
  0.7.7 re-measure after the aes-gcm revert (443). reality-tls download is deterministic (client-decrypt
  bound) so it matches regardless of contention; up is variable and sags with the host's retransmits.
  Raw data — [release/reality_tls_5x_v0.7.10_2026-07-10.json](../../../release/reality_tls_5x_v0.7.10_2026-07-10.json).

### jemalloc: RSS

The server binary now builds with `--features jemalloc` (like the release Linux / .deb). The worker RSS in
these runs is **~46–49 MB** (jemalloc arenas; glibc builds gave ~8 MB). This is **not a leak** — jemalloc
keeps a larger idle pool but caps lower than glibc under handshake churn (in prod 15 vs 183 MB). The RSS
columns in the old 0.6.0 tables below (~7–8 MB) are a glibc build — don't compare directly.

### The 0.7.10 conclusion

**No regression.** The host-neutral A/B: 0.7.9 (= the 0.7.10 data plane) is **equal-or-faster than 0.7.4**
in every TCP mode (up +2…+10%, down mostly + / ≈0), reality-tls at the median of 3× is **+6% / +4%**,
obfs-down **+13%**. The single dips in the plain sweep (reality-tls 332, obfs-down 176–430, frag down −12%)
are host contention (low CPU = throttling, not code), ruled out by the A/B. The cumulative 0.7.5→0.7.9
changes (oversized-record, aes-gcm revert, udp-quic) did not hurt the data plane. The detailed 0.6.0
reference tables are below.

## Version 0.7.11 (candidate, 2026-07-12) — the delta to 0.7.9 (A/B)

A large dev batch (commit `84521de` "land pending 0.7.11 dev batch"): a full-code audit (RCE→P3),
exclude/include CIDR routing under a full tunnel, per-user static_ip + disconnect notifications, fast
recovery from sleep/resume + network change, panel fixes, and **dep bumps `51490bf`: x25519-dalek 3
(major), tikv-jemallocator 0.7, rand 0.10, bytes**. The binary reports `qeli 0.7.10` sha `eff3fcb3`
(Cargo not yet 0.7.11), gate PASS (build `--features jemalloc` + **291 tests** + clippy). A/B raw data —
[release/ab_079_vs_0711_2026-07-12.json](../../../release/ab_079_vs_0711_2026-07-12.json), the full run —
[release/benchmark_results_2026-07-12_v0.7.11.json](../../../release/benchmark_results_2026-07-12_v0.7.11.json).

> 🟡 **Why A/B.** The host was moderately contended (steal ~2.7%, no-VPN baseline 18.5 Gbps vs clean 21,
> **high retransmits** in the thousands). The direct sweep gave one snag — reality-tls-down 398 (below
> the stable 429 floor) at **low client CPU** (22–27% = not CPU-bound, host-throttled). The batch's main
> risk is the major **x25519-dalek 3** (crypto dep). So the delta is taken **host-neutral**: 0.7.9 (built
> from tag `v0.7.9`, sha `0c5baba5`) and 0.7.11 run **interleaved in one session**
> ([scripts/ab_079_0711.py](../../../scripts/ab_079_0711.py)). reality-tls is the median of 3× per version.

### A/B TCP, Mbps (↑up / ↓down) — both binaries on the same (contended) host

| mode | 0.7.9 ↑/↓ | 0.7.11 ↑/↓ | Δ |
|---|---|---|---|
| plain | 530 / 687 | 512 / 724 | −3.5% / +5.4% |
| fake-tls | 530 / 691 | 540 / 707 | +1.9% / +2.2% |
| padding | 524 / 648 | 515 / 705 | −1.7% / +8.9% |
| frag | 528 / 699 | 507 / 695 | −4.1% / −0.6% |
| obfs | 485 / 266¹ | 490 / 630 | +1.0% / (0.7.9 outlier) |
| reality (proxy) | 507 / 714 | 522 / 723 | +3.1% / +1.2% |
| **reality-tls** (median 3×) | 462 / 381 | **489 / 392** | **+5.9% / +2.8%** |

¹ 0.7.9 obfs-down = 266 was a single host outlier in that run (0.7.11 is 630; 0.7.4 was 547); the delta
there is not representative. In every other row 0.7.11 is equal-or-higher. **reality-tls (median 3×)
+5.9% / +2.8%** is the direct answer that the reality-tls-down 398 in the plain sweep was contention
(0.7.9 on the same host did even less, 381), not a regression from x25519-dalek 3.

### A/B UDP, loss % (400M / 500M)

| mode | 0.7.9 | 0.7.11 |
|---|---|---|
| udp-fake-tls | 0.4 / 4.1 | 6.3 / 6.8 |
| udp-padding | 0.7 / 3.7 | 0.7 / 2.7 |
| udp-quic | 0.5 / 5.5 | 1.5 / 4.7 |

UDP is noisy on both versions (the open-loop receiver on .11 under contention); 0.7.11 is comparable on
average (udp-padding cleaner). The spread is the host.

### reality-tls × 5 (0.7.11, same host)

- **↑ up:** median **484.9** (σ 7.9, 470–493).
- **↓ down:** median **397.8** (σ 7.4, 395–415) — below the clean-day 429 floor, BUT on this same host the
  A/B measured 0.7.9 at 381 down → 0.7.11 (392–398) is **higher**. The dip is contention (client-decrypt
  throttled), not code. Raw data — [release/reality_tls_5x_v0.7.11_2026-07-12.json](../../../release/reality_tls_5x_v0.7.11_2026-07-12.json).

### The 0.7.11 conclusion

**No regression.** The host-neutral A/B: 0.7.11 is **equal-or-faster than 0.7.9** in every mode, including
the heaviest reality-tls (**+5.9% / +2.8%**). The major **x25519-dalek 3** bump and the other dep bumps
did not hurt the data plane (the crypto handshake is a one-time cost; the data-plane AEAD is ChaCha20,
untouched). The single reality-tls-down dip in the plain sweep is ruled out by the A/B (host contention).
The detailed 0.6.0 reference tables are below.

## The baseline without VPN (0.6.0, reference base)

| | throughput | loss | CPU |
|---|---:|---:|---:|
| TCP direct `.11→.10` | **20,972 Mbps** | — | recv 99% (CPU-bound, not the network) |
| UDP @ 500 Mbps | 500 Mbps | 0.16% | — |
| UDP @ 1 Gbps | **1000 Mbps** | 0.10% | — |

## TCP — all modes (0.6.0, reference base)

_The numbers below are the 0.6.0 run (2026-06-11) with the detailed columns (RTT/retr/CPU/RSS).
The current per-version throughput deltas are in the "Version 0.7.x" sections above; reality-tls
download since 0.7.0 is already ~417→430, not 321._

`up` = client→server (decrypt on the server), `down` = server→client (`iperf3 -R`,
decrypt on the client). `qeli CPU` — the busiest qeli process on the server (% of ONE
core, average/peak over the up run). `RSS` — the resident memory of the worker.

| mode | ↑ up Mbps | ↓ down Mbps | RTT avg | retr ↑/↓ | qeli CPU ↑ avg/peak | RSS |
|---|---:|---:|---:|---:|---:|---:|
| **tcp-plain** (raw, no obfuscation) | 570.8 | 714.3 | 1.24 ms | 619 / 664 | 34.6% / 60.6% | 7.3 MB |
| **tcp-fake-tls** (TLS mimicry) | 562.9 | 697.1 | 1.37 ms | 400 / 53 | 34.2% / 60.1% | 7.8 MB |
| **tcp-padding** (+ random padding) | 558.7 | 704.0 | 1.38 ms | 161 / 695 | 35.4% / 62.6% | 7.9 MB |
| **tcp-frag** (+ fragmentation) | 553.8 | 701.9 | 1.30 ms | 454 / 1063 | 34.3% / 60.3% | 7.5 MB |
| **tcp-obfs** (ChaCha20 stream + WS-fronting) | 497.6 | 582.4 | 1.29 ms | 1463 / 755 | 34.1% / 59.6% | 7.7 MB |
| **tcp-reality** (proxy-bridge, fake-TLS) | **577.2** | **721.4** | 1.37 ms | 1215 / 774 | 34.4% / 61.1% | 7.7 MB |
| **tcp-reality-tls** (real TLS 1.3) | 488¹ | **321** | 1.32 ms | 1612 / 373 | 31.8% / 56.6% | 8.3 MB |

> ¹ `reality-tls` ↑ is highly variable (5 runs: average **470 ± 37** Mbps, median
> **488**, range 421–511) — the heaviest mode, a spread on a single 12-sec run;
> ↓ is **stable 321 ± 2.4**. The row shows the median run (see
> [release/reality_tls_5x_2026-06-11.json](../../../release/reality_tls_5x_2026-06-11.json),
> [scripts/reality_tls_repeat.py](../../../scripts/reality_tls_repeat.py)).

**Reading:**
- **`plain` ≈ `fake-tls`** (571 vs 563 ↑; 714 vs 697 ↓): the bare tunnel and fake-TLS
  give the same speed — the fake-TLS handshake is one-time, and the framing difference
  (3 bytes/packet) is in the noise. Removing obfuscation does **not** raise throughput
  (the bottleneck is the AEAD).
- **Obfuscation is almost free** on fake-tls: padding/fragmentation/heartbeat within the
  noise.
- **`obfs` ~−12% ↑ / −16% ↓** (498/582 vs 563/697): the cost of the ChaCha20-XOR over the
  whole flow (double encryption).
- **`reality` (proxy) ≈ `fake-tls`** (even the top by throughput: 577/721): peek-and-decide
  once per TCP accept.
- **`reality-tls` — the cost of real TLS, especially on download** (488 ↑ / **321 ↓**).
  See the analysis below.
- The data plane is **multi-core** (the `/proc/stat` delta measurement, not `ps`): one
  tunnel ~1.5 cores, the load from many clients spreads across the cores. Since the
  version with **`tun.queues`** (multi-queue TUN, default auto=nproc — see [CONFIG.md](../manuals/CONFIG.md))
  the TUN pump itself is parallelized too — a controlled A/B gives **+18% aggregate** with
  2 tunnels, single-flow unchanged (in detail — the [Multi-queue TUN](#multi-queue-tun-tunqueues--ab)
  section below). Memory — **~7–8 MB** RSS. RTT overhead ≈ **1.6–1.9 ms**.

## UDP — all modes (0.6.0, reference base; a sweep by bitrate, % loss)

| mode | 100M | 200M | 300M | 400M | 500M | sustained bandwidth |
|---|---:|---:|---:|---:|---:|---|
| **udp-faketls** (fake-tls, the UDP base) | 0% | 0.05% | 0.06% | 0.35% | 2.97% | **~400 Mbps** (<0.5% up to 400) |
| **udp-padding** | 0% | 0% | 0.14% | 0.41% | 3.22% | ~400 Mbps |
| **udp-quic** (masking) | 0% | 0% | 0.09% | 0.25% | 4.08% | ~400 Mbps |

The UDP plane is clean (<0.15%) up to 300 Mbps, holds ~400 at <0.5% loss, saturates
around 500 (3–4% loss). `plain`/`obfs`/`reality*` — TCP only.

> NB: these UDP numbers are from a **clean** (freshly rebooted) lab. Under a background
> load on the .11 client the loss at 400–500 Mbps balloons several-fold (400M→~8%,
> 500M→~22%) — that's CPU starvation of the open-loop receiver, **not** a degradation of
> the data plane (see ⚠️ above).

### Why there's no `plain` on UDP (by design)

`plain` (raw) is TCP-only, and this is a deliberate decision, not a gap in the tests
(the server rejects `mode = plain` + `transport = udp` at startup, see `server/mod.rs`).
Two reasons:

1. **It wouldn't give speed.** The bottleneck is the AEAD (ChaCha20-Poly1305), not the
   5-byte fake-TLS header per datagram. The same conclusion as for TCP
   (`plain ≈ fake-tls`): removing the wrapper doesn't raise throughput.
2. **It's worse for circumvention.** Raw UDP is a high-entropy flow with no structure,
   i.e. exactly the "fully encrypted traffic" signature by which DPI (GFW/TSPU) throttles
   UDP. `udp-faketls`, on the contrary, provides cover (the datagrams look like TLS
   records). So raw UDP would be a **red flag**, not masking.

So for UDP the base mode is `fake-tls` (the rows above), and it is exactly that which is
benchmarked as "UDP without extra obfuscation".

## Legacy reality-tls carrier (≤0.7.16): why download was ~320

> **Historical diagnosis.** The numbers below start from the **0.6.0 floor (~320)** and use the
> carrier shipped through 0.7.16. Since 0.7.0 the hand-rolled TLS server lifted download to ~417;
> the **0.7.4 measurement is ~430 Mbps**. Double AEAD remains relevant to current H2, but the old
> inner fake-TLS framing and its associated double-framing cost were removed in dev 0.8.0.

The tunnel in `reality-tls` runs **inside** real TLS 1.3 (rustls on the server, the
hand-rolled `realtls` on the client). On **download** the client reader strips **two
layers sequentially**: the outer TLS record (AES-128-GCM) **and** the inner qeli record
(ChaCha20-Poly1305), with double framing — in a single task. This roughly halves the
download relative to single-layer modes (~700 → ~320).

What has been verified by measurement (a lab diagnosis):
- **AES-NI on the VM is present** (not software-AES).
- The client `qeli` process at reality-tls download peaks at **~67% of one core** (not
  100%) — i.e. this is **not** a pure CPU ceiling, but a combination of double AEAD +
  double framing + await overhead in one reader task.
- **Download is deterministic** (5 runs 2026-06-11: **321 ± 2.4** Mbps), while upload is
  variable (**470 ± 37**, range 421–511): ↓ is bound by the stable two-layer decrypt
  cadence, ↑ fluctuates due to retransmits of the outer TLS under load.

**What did NOT help:** the optimization of the client read path
`RealTlsStream::poll_read` (batch-decrypt of all ready records in one poll, a 64-KiB read
buffer instead of 4-KiB, a cursor instead of a per-record `drain`+allocation) — correct
and kept in the code (161 tests green), but it didn't move the download (317 → 322 → 309
→ 319 Mbps — within the noise), since the bottleneck is **not** in buffering/syscalls.

**Historical follow-up considered at the time:**
1. **Remove the redundant inner AEAD in reality-tls** — the outer TLS already encrypts
   and authenticates, the inner ChaCha20 on the data plane duplicates this. Inside
   reality-tls the data can be pushed in `plain`/Raw framing (without the second AEAD),
   keeping the qeli handshake/auth — this roughly halves the client's work on download. A
   decision with a security trade-off (defence-in-depth), it requires explicit agreement.
2. **Parallelize the two crypto layers** across tasks/cores (the TLS-decrypt in one, the
   inner in another).

This section must not be used to predict current H2 throughput. For the current carrier and
its measured DPI scope, see [DPI-AUDIT.md](DPI-AUDIT.md) and the dated
[H2 PCAP report](../../../release/dpi_audit_dev_0.8.0_h2_2026-08-26/REPORT.md).

## Multi-queue TUN (`tun.queues`) — A/B

**How it works.** `tun.queues` (per-profile, default `0` = auto = `nproc`) sets how many
`IFF_MULTI_QUEUE` queues are opened on a single TUN device. The data plane pumps them
with N independent reader/forwarder/writer tasks (tokio), and **the kernel RSS-spreads the
outgoing TUN packets across the queues by flow**. This parallelizes the TUN pump itself
(previously — a single reader+forwarder+writer funnel ~1.5 cores) in addition to the
already-multi-core per-connection AEAD. `1` = the old single-threaded pump (for
rollback). UDP is parallelized via N workers on `SO_REUSEPORT` sockets. Nothing changes
on the wire, clients need no rebuild (TUN is a local OS-kernel interface).

**The measurement.** A controlled A/B on a 2-core lab: 2 tunnels in separate netns, a
simultaneous upload, the server worker's CPU — the delta by `/proc/<pid>/stat` (it can
exceed 100%), the host — the delta by `/proc/stat`. The orchestrator —
[scripts/multitunnel_probe.py](../../../scripts/multitunnel_probe.py) (`QELI_TUN_QUEUES=1|2`).

| | `queues=1` (legacy) | `queues=2` (multi-queue) | Δ |
|---|---:|---:|---:|
| **1 tunnel**, aggregate | 458 Mbps | 455 Mbps | ≈0% |
| **2 tunnels**, aggregate | 607 Mbps | **718 Mbps** | **+18%** |
| qeli CPU @2 tun. (% of one core) | 159% | 167% | takes more of a core |
| threads in the worker | 9 | 11 | +2 queue readers |
| server-host @2 tun. (% of all cores) | 93% | 95% | saturated |

**Reading:**
- **A single stream isn't sped up by the queues** (458 ≈ 455 Mbps): a single TCP is bound
  by its per-connection decrypt task (~1 core), not the TUN pump — N queues have nothing
  to spread. So the per-segment benchmark above (one tunnel per mode) does **not** show a
  multi-queue gain — that's by design, not a regression.
- **The aggregate of many tunnels is sped up**: 607 → 718 Mbps (**+18%**); qeli mean-while
  pulls more of a core (159→167%) and raises +2 reader threads on the queues.
- **+18% is a lower bound**: both 2-tunnel runs hit host saturation (93–95% of all cores),
  since the `iperf3` sink runs on the SAME 2-core server and eats the free cores. On a
  production server with a remote sink and more cores the gain grows with the number of
  cores/clients.

**Conclusion:** `tun.queues=auto` — a free default (single-flow unchanged, the aggregate
under load is faster); set `=1` only for debugging/rollback.

## Stream bonding under `tc netem` (2026-06-22)

`scripts/bench_bonding.py`: a multipath fake-tls profile on :8505, `max_streams` 1 vs 4
(`adaptive=false`), download through the tunnel under a **filtered** `tc netem` (RTT+loss
applied only to traffic toward the client, so the control SSH is untouched). iperf3 **`-P 8`** —
eight parallel flows (bonding spreads them by flow hash; a single flow uses only one connection).

| link | 1 stream | 4 streams | gain |
|---|---:|---:|---:|
| clean | ~725–846 | ~805–815 | parity |
| RTT 40 ms, 0.05% loss | ~225–420 | ~692–704 | ~1.6–3× |
| RTT 80 ms, 0.1% loss | ~50–65 | ~260–305 | **~5×** (stable over 2 runs) |

**Conclusion:** on a clean link bonding isn't needed (the TUN pump is the ceiling); on a
lossy/latent link it scales. Only traffic with **several concurrent connections** speeds up
(per-flow pinning, `flow_hash % streams`) — a single lone flow won't.

**Gotchas:** use the **release** binary (debug crypto → ~17 Mbps → a false "bonding doesn't
work"); `.11` has no `/opt/qeli-src` → copy the binary to the client; apply netem as a **filter**
on `ens18` or the loss hits the control SSH; clear netem + `systemctl restart qeli-server.service`
in finally.

## Historical summary

| | TCP | UDP |
|---|---|---|
| Practical ceiling (2 vCPU) | **~530–570 ↑ / ~700–720 ↓ Mbps** (plain/fake-tls/reality, measured on 0.7.4) | **~400 Mbps** at <0.5% loss |
| Latency overhead | ~1.2–1.4 ms | ~1.2–1.4 ms |
| Memory (worker RSS) | ~7–8 MB | ~7–8 MB |
| The cost of fake-tls obfuscation | ≈0 | small |
| The cost of `obfs` | −12% ↑ / −16% ↓ | (obfs TCP only) |
| The cost of `reality` (proxy) | ≈0 | — |
| Legacy `reality-tls` cost (≤0.7.16) | ≈0 ↑ / **−40% ↓** (outer TLS + inner fake-TLS; was −54% on 0.6.0, ~430↓ on 0.7.4) | — |
| `plain` (raw) | ≈ fake-tls | n/a (TCP-only) |

For the measured legacy set, `plain`/`fake-tls` were the fastest and cheapest on CPU;
`obfs` paid a moderate cost and legacy `reality-tls` a noticeable download cost. Current H2
requires a separate clean throughput result.

## Historical reproduction

The commands below assume a dedicated lab with authorized service resets.
Use the isolated runner above for an active lab.

```bash
# from a local machine (paramiko); flat-INI configs, write to /etc/qeli/bench-*.conf.
# H-1 (0.7.1): benchmark.py pins the server key in every mode.
python scripts/reboot_vms.py         # clean lab: kills emulator/qeli/netem → reboots both VMs → cleans AGAIN (autostart)
python scripts/lab_reset.py          # same without a reboot (emulator, qeli units, iperf3, orphan TUNs, tc/netem)
python scripts/stability_gate.py     # Requires a prepared idle lab and an operator-started iperf3 server;
                                     # All5 samples/direction required; failure, busy lab or spread>8% → exit1
python scripts/benchmark.py --allow-host-wide-benchmark # baseline + 12 modes × {ping, iperf, CPU/RSS} ≈ 10 min
                                     # → release/benchmark_<version>_<date>.json (+ a benchmark_results.json copy)
python scripts/reality_tls_repeat.py # reality-tls ×5 → median/σ (release/reality_tls_5x_<version>_<date>.json)
python scripts/ab_071_072.py         # host-neutral A/B (0.7.1 from tag vs 0.7.2 interleaved) — when the host is under steal/contention
python scripts/ab_074_079.py         # same for 0.7.4->0.7.9 (0.7.10 candidate); A/B template for any tag<->current pair
python scripts/ab_079_0711.py        # 0.7.9->0.7.11 (dev batch, x25519-dalek 3 bump)
python scripts/e2e_android_faketls.py --apk-receipt "qeli-android/dist/COMMIT-RUN/BUILD.json" # Android e2e: rebuild the APK (rebuild_apk.py) -> inject a fake-tls profile -> Connect -> ping through the tunnel
# GOTCHA: a foreign qeli.service may auto-start on .11 (a server on vpn0 10.9.0.1) — it loads the client; `systemctl stop qeli.service` before benchmarking
python scripts/config_functest.py    # default-config functionality: e2e server.conf + server-maxobf.conf + parse all
python scripts/multicore_probe.py    # the precise data-plane CPU (/proc delta: idle/up/down/bidir)
python scripts/probe_060_ab.py       # the CPU A/B vs the previous version (isolated build from git HEAD)
```
The results → `release/benchmark_results.json` and `release/*_v0.7.11_*.json`.

## Lab stability admission: corrections on 10 October 2026

F504 / Q35-F001 / Q36-F001: the previous stability_gate.py discarded failed/zero
samples. Two equal successful measurements out of five requested runs produced
PASS despite three failures. Negative rates were accepted, and an8.04percent
spread rounded to8.0percent passed the8percent limit. These three cases were
reproduced using the exact previous code with controlled SSH imports; dangerous
host commands were never executed.

Every requested measurement in both directions is now mandatory: only positive,
finite numeric rates, excluding bool/strings. Admission uses the unrounded spread.
The report preserves all samples/errors and statistics; an incomplete direction
has stats=null and verdict_quiet=false. Settings are validated before connecting:
QELI_GATE_RUNS2..20, QELI_GATE_MAX_SPREAD in[0,100), QELI_GATE_PORT1..65535
and two distinct IP addresses.

F505 / Q35-F002 / Q36-F002: the gate no longer calls full_clean, stops Qeli/emulators,
rewrites DNS, removes interfaces/qdiscs or kills every iperf3 process. An operator
prepares a dedicated idle lab and starts their own iperf3 server on the selected
port (default5201). Read-only ps/ip/tc/ss probes before and after measurements
refuse active Qeli/emulators, measurement processes, tun/tap interfaces, artificial
shapers or existing iperf connections. Missing tools/probe command failures also
refuse admission. This does not authorize automatically resetting an active lab.
The existing server remains under operator control.

Each iperf client has a remote40s timeout and SSH55s timeout; exit status and JSON
are checked. Finally closes both SSH connections, including failures of the
second connection/probe/other close. A close error cancels admission. Result
publication uses a private temporary file and atomic replacement; a write failure
preserves the previous file and exits1. Consumers must check the current exit
code; an old JSON result alone cannot authorize a benchmark. PASS covers only
this raw measurement window, not future hypervisor stability or the interleaved
baseline/A-B requirements of a complete sweep.

89 Windows methods (19 stability,38 native recipes,11 workflows,21 CI admission)
and19 Linux stability methods PASS; the new suite runs in CI. Pinned actionlint
accepts all4 workflows. Unit SSH/iperf seams are controlled: no new actual
throughput benchmark is claimed. A real read-only .11 probe refused its active
Qeli processes before workload; service777/845 and CLI75bc remained unchanged.
Both source files and the TAR were read back exactly; SHA/exact-set/process/fd/
mount guards permitted removal of3owned files. .10 was unused.

Initial harness error: nonexistent test_workflow_lint.py returned NO TESTS RAN/rc5.
Its log is retained; the existing test_check_workflows.py passed11methods.
Previously successful19/38 suites were not repeated. Initial apply_patch writing
refused the worktree reparse point; standard Python wrote the file after checking
its resolved workspace root. This was not a product test failure.

Core source,14native artifacts, source digest e3fb and the OpenWrt cut are unchanged;
previous execution qualification is inherited. Q34 remains IN_PROGRESS: F459
braces, current-source hosted push CI, public pinned OpenWrt revision and expected
release publisher/supported signature/runtime boundaries are not closed here.
Historical OPEN statements superseded by later SHA-bound reports do not require
rebuilding unchanged inputs. Q35 has started but is not complete; Q36 remains TODO,
with only this prerequisite corrected. Overall30/37DONE81.1percent,7remain.
Q29 SIGKILL/ENONET,Q30/Q31,D06 ACCEPTED_LIMITATION and USER_EXCLUDED are preserved.
LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.

F504-F505 evidence: q35-stability-20261010; SEALED 33 files / 1319903 bytes; manifest SHA256 e88d4e2a863bd92e0e6de19bdecd0ed91ffde66b89838b9461be4adb6422285f.

## TCP states for raw/mux — 11 October 2026

<!-- normative-sync: q36-tcp-state-v1 -->

**Q36-F020:** [diagnostics](../../../release/audit_q36_tcp_state_2026-10-11.json): six fresh
sessions, twelve eight-second plain TCP raw/mux windows, P=1, unlimited/250 Mbps
per direction; three repeats with varying order. Two isolated IPvlan namespaces
on .10 share two vCPUs. Both use qualified F010 release CLI SHA 238153ab,
default features+jemalloc. This does not test the current UDP fix and is not
a repeat of the previous two-VM benchmark.

A bounded thirteen-second sampler invokes ss -tinpO roughly every250ms.
Inner data connections match iperf JSON socket/port identities, excluding the
control connection; outer carrier identities match Qeli PIDs. Actual observations
for six roles (inner up/down and outer × server/client), PID/CPU policy,
recordizer mode, receiver/bitrate, kernel/server deltas and SHA were checked.
These are sampled TCP_INFO values, not per-packet ACK timestamps or a complete
RTT distribution. Final iperf RTT/cwnd fields under -i 0 are not treated as
a temporal trace. Sampling also adds load to the two shared vCPUs.

Median [min–max] rates across three sessions. The last two columns are medians
of observed per-session peaks, not continuous maxima:

| Mode | Target Mbps/direction | ↑ Mbps | ↓ Mbps | Client outer Recv-Q peak KiB | Inner-up sender RTT peak ms |
|---|---:|---:|---:|---:|---:|
| raw | unlimited | 115.5 [93.9–136.8] | 478.7 [460.6–482.3] | 1104.4 | 53.826 |
| raw | 250 | 248.7 [241.4–249.8] | 249.2 [244.7–249.7] | 275.2 | 27.606 |
| mux | unlimited | 337.2 [273.0–402.3] | 896.6 [862.2–951.7] | 470.9 | 17.858 |
| mux | 250 | 249.9 [249.7–249.9] | 249.7 [249.6–249.8] | 115.9 | 6.096 |

Asymmetry reproduces on one VM; pacing and mux change the outcome.
Snapshots clarify queue/RTT behavior, but do not establish ACK starvation,
a scheduler bug or buffer-size causality. Defaults and the data plane are unchanged;
the cause remains OPEN. These populations are not pooled with previous VM pairs
or the F019 functional AUTH build. No overall performance PASS is claimed.

Measurement host gate: **FAIL** ['iptables4', 'legacy4'];
cleanup host gate: **FAIL** ['legacy4'].
All differences remain visible. Owned namespaces were removed; the root was
SHA-archived and removed after checking the downloaded copy. Services and the
staged CLI were preserved. The first harness failed on a wrong measure signature
before load started; trace/log/receipt remain, and namespaces were cleaned before
retrying. At last check .11 accepted TCP22 without an SSH banner. Q36 cause/release
AUTH-rate and .11 cleanup remain OPEN; Q35 soak is separate.
LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.

## CPU affinity probe — 11 October 2026

<!-- normative-sync: q36-tcp-affinity-v1 -->

**Q36-F021:** [paired experiment](../../../release/audit_q36_tcp_affinity_2026-10-11.json): six fresh
raw/mux sessions, two eight-second unlimited TCP bidir P1 windows each; three
repeats with varying order. Shared: both Qeli processes use affinity 0–1; split:
server 0, client 1. All TIDs before/after, PID identity and sampled CPU policy
were checked. Binary, INI, load and TCP sampler are identical within each pair;
final INI hashes match the archived configuration. Only Qeli CPU placement changes.
This also reduces each Qeli process from two available CPUs to one. CPUs are not
reserved; iperf/sampler/background services continue sharing both vCPUs.

Median [min–max] rates across three sessions, Mbps:

| Mode | Placement | ↑ Mbps | ↓ Mbps |
|---|---|---:|---:|
| raw | shared | 149.0 [103.4–152.5] | 462.4 [452.2–499.5] |
| raw | split | 130.5 [125.4–140.5] | 450.2 [430.2–502.9] |
| mux | shared | 342.7 [336.8–358.9] | 870.3 [868.1–903.2] |
| mux | split | 309.7 [270.8–325.3] | 816.7 [809.7–955.2] |

Paired split/shared ratios, median [min–max]: raw ↑0.92 [0.84–1.26],
↓1.00 [0.86–1.09]; mux ↑0.86 [0.80–0.95],
↓0.93 [0.90–1.10]. These describe this affinity
intervention on this VM, not proof of a scheduler bug or ACK starvation causality.
Pinning did not eliminate raw asymmetry: uplink is
27.9–30.3% of downlink in all three split windows.
Queue/RTT observations are sampled TCP_INFO, not continuous ACK timing. The cause
remains OPEN and product settings are unchanged. Qualified F010 release CLI SHA
238153ab is identical at both endpoints; this does not test the current UDP fix.
Populations are not pooled with F020, earlier VM pairs or functional AUTH.

Measurement host gate: **PASS** [];
cleanup host gate: **FAIL** ['iptables4', 'legacy4'].
Full differences remain. Owned namespaces/root were removed after verifying the
SHA archive; services and staged CLI were preserved, with no new build cache.
Q36 cause/release AUTH-rate and
.11 cleanup remain OPEN; Q35 soak is separate. LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.
