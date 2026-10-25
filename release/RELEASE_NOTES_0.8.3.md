# Qeli 0.8.3 (beta) — network policy, NDP and client diagnostics

**Draft / Черновик. Not published / Не опубликован.**

## English

This update addresses TCP buffer limits on high-RTT paths and makes UDP/TUN
queue pressure easier to diagnose. It preserves explicit operator settings and
existing memory budgets.

- Client diagnostics now distinguish queued, in-flight, locally completed and
  cancelled TCP uplink work in a bounded snapshot. Local completion is not a peer ACK.
- Android adds a bounded asynchronous diagnostic journal, manual structured export
  without free-text messages, profiles, keys or network addresses, and opt-in retention
  of the last error report. Clearing invalidates queued events and pending exports.
- Windows service traffic graphs use source timestamps and sleep-inclusive uptime;
  duplicate snapshots and calendar-clock jumps no longer create false rate samples.
  These development changes are not covered by the older candidate measurements below.

- Server TCP buffers default to OS management (`perf.tcp.send_buffer_size = 0`,
  `perf.tcp.recv_buffer_size = 0`). Linux installation explicitly enables receive
  autotuning, verifies readback and retains rollback data. Existing manual values
  are preserved. New multi-profile examples and panel Quick Start use the same defaults.
- The internal UDP receive handoff stays at 128 datagrams per pump. New depth and
  wait metrics help locate pressure without unbounded queue growth. The existing
  bounded UDP socket-buffer controller and manual TUN settings remain available.
- TUN dispatch first attempts immediate delivery, retaining bounded backpressure
  when full. Local `tun-stats` separates pool waits, dispatch waits, read gaps and
  UDP batch-send time. Sustained overload and all TUN drops are not claimed fixed.
- A manual client tunnel MTU no longer disables outer UDP path probing.
  `mtu_probe = false` remains authoritative, with conservative fallback when probing fails.
- macOS ships separate self-contained `Qeli-macos-arm64.tar.gz` (Apple Silicon)
  and `Qeli-macos-x86_64.tar.gz` (Intel) archives, requiring macOS 15 or newer.
  The universal packager now refuses incompatible .NET payloads; lipo alone cannot
  combine architecture-specific managed DLLs. Native Rust core remains universal2.
- macOS fixes cover Apple Silicon `openat`, recovery of old root-owned lock-file
  permissions, launchctl service detection and disconnect/status error handling.
- Android Tink moves to 1.24.0; Rust libc moves to 0.2.191. TLS dependencies
  rustls-webpki 0.103.16 and webpki-roots 1.0.10 address the newly reported
  certificate-trust advisories RUSTSEC-2026-0382 and RUSTSEC-2026-0381.
- ML-KEM 0.3.3 removes a secret-dependent CBD lookup; ctutils 0.4.4 addresses
  RUSTSEC-2026-0385. Tokio moves to 1.54.1 and Gradle Wrapper to 9.8.2.
  The Cargo/fuzz locks and embedded original license texts are synchronized.
- Setup, configuration examples and Docker buffer diagnostics are aligned;
  license inventories are bound to the updated dependency locks.


- Issue #69 follow-up: SNI validation now follows actual TLS/WebSocket use.
  Legacy UDP-obfs SNI no longer blocks a valid profile; unused SNI is omitted
  from generated links. Desktop TCP WebSocket exposes its HTTP Host override.
- `pull_routes=false` declines server routes in the shared core while preserving
  local routing, independent full-tunnel policy and protected tunnel DNS routes.
  All clients preserve it through INI/models/backups and new share links.
  Explicit `gateway=false` now survives Rust INI serialization.
- NDP proxy uses an assigned uplink source rather than the advertised target,
  rechecks ready-session ownership before sending, joins known solicited-node
  groups through MLD and emits bounded appearance announcements. `manual` keeps
  administrator-owned routing/firewall. No arbitrary NA flags or periodic flood.
- UDP diagnostics separate inner MTU, outer payload budget and fallback reason;
  counters report encrypted-record split attempts and fragments created, not loss.
- Windows adds structured IPv4/IPv6 copy in GUI/service mode, remembered editor
  dimensions and a bounded traffic graph. Optional `windows_disable_sharing=false`
  applies only to the created Qeli Wintun with GUID/LUID/readback checks.

Scope and checks: [Issue #69 implementation plan](https://github.com/litvinovtd/qeli/blob/v0.8.3/release/certification/plans/ISSUE69_0.8.3.md).
Fit-path, periodic NDP and an adapter surviving Disconnect are separate optional
stages, not claims of this update. Real Windows adapter bindings and provider
MLD-snooping are not certified by mocked or namespace-only tests.

The recorded TCP A/B experiment used 1 or 8 independent clients, added RTT of
40/100 ms and identical manual/OS-managed buffer comparisons on a two-VM Linux lab.
One eight-client uplink case reached a median aggregate 2,456 Mbit/s with OS-managed
buffers versus 279 Mbit/s with fixed 262,144-byte buffers. This is the sum across
eight tunnels on a local lab path, not a single-client or Internet speed promise.
Results and method: [benchmark evidence](https://github.com/litvinovtd/qeli/blob/v0.8.3/docs/data/benchmarks/queue-tuning-20261021.json).

For an existing Linux TCP profile, set both server TCP buffer keys to `0`, check
`net.ipv4.tcp_moderate_rcvbuf=1` and appropriate `tcp_rmem`/`tcp_wmem` limits,
validate the configuration and restart/reconnect to create new sockets. The global
`tcp_mem` budget is preserved. Do not force reinstall just to change these settings.
[Setup guide](https://github.com/litvinovtd/qeli/blob/v0.8.3/docs/eng/manuals/GETTING-STARTED.md).

The earlier accepted Android API34 ENONET limitation and excluded physical test
platforms retain their original scope; this update does not reclassify them as fixed.

### TCP/H2 and live UDP PMTU verification — 2026-10-23

An additional **Linux candidate**, not the packaged ELF/assets listed below, was
built in release mode with `jemalloc` on build-only VM302 from
`fb741e204cc82f8e0c43172d5746c1a6afa78607`:
SHA-256 **`09d0586e03a68f2cf22aebdb8ff2dd38d04063f0c0e0b6bd1f0f271997a177bd`**.
Control: `95cf32f8a9bd76de855e1e599c88fc9feb94a677ccef99c61b35762f27d467f7`.
Existing release assets and the earlier four-profile speed table were not replaced.

VM300/server and VM301/client each had **2 vCPU / 2 GiB**, Linux
`6.12.111+deb13-amd64`, iperf3 3.18 and dedicated eth1 test addresses
172.30.82.10/11 and fd42:82:1::10/11. TUN MTU **1400**, `tun.tx_queue_len=1000`
on both peers, two server TUN queues, **Recordizer required / active PACKET_MUX_V1**.
Reality-TLS used genuine H2, handrolled and shaping, without padding/stealth/heartbeat.
No resource/queue/qdisc/sysctl tuning or reboot; builds only on302, no Proxmox
builds, installs or runtime commands. Guests restored **17/17**, prior hashes preserved.

- **37 unique Linux component regressions passed**, including independent H2
  DATA/PING deadlines, writer cancellation, PMTU confirmations/stale replies,
  DATA_FRAG retry identity and pool ownership. Their separate test/debug executable
  SHA is `7c37b758b76b3deb2dd090d57d781c5053a08bfa92c2698d58bff1ad50350cc1`;
  this is not an end-to-end check of the shipping ELF.
- The release-mode candidate passed70s healthy idle followed by useful TCP inner
  IPv4/IPv6 up/down on FakeTLS, WebSocket obfs and genuine H2 (15 windows).
- In old/new FakeTLS and H2 checks with50Mbps inner UDP over TCP and a65s
  carrier blackhole, the candidate logged **30s pending write/flush no-progress**,
  terminated the damaged carrier and recovered useful TCP without manual restart.
  Old blocked writers did not show that bounded shutdown. In the initial H2
  TCP-over-TCP case, the observed trigger was inbound liveness, not PING timeout.
  Production whole-carrier loss does not isolate WINDOW_UPDATE from PING.
- **8/8 live PMTU profile/outer-family/direction cases** were accepted after an
  explicitly retained incomplete window and a separate recheck. At10Mbps,
  inner IPv6 UDP payload1352,60s, receiver INPUT discarded outer IP packets>1280.
  Certified smaller budgets appeared in **9.6–14.0s**, with DATA_FRAG, unchanged
  TUN MTU1400 and no reconnect. Post-fault inner IPv4/IPv6 payload checks passed.

| UDP profile | Outer | Upload budget | Download budget | Steady receiver/loss |
|---|---|---|---|---|
| FakeTLS | IPv4 |1448→548|1448→548|≈10Mbps /0%|
| FakeTLS | IPv6 |1448→1232|1448→1232|≈10Mbps /0%|
| QUIC | IPv4 |1457→548|1457→548|≈10Mbps /0%|
| QUIC | IPv6 |1417→1232|1417→1232|≈10Mbps /0%|

Steady means15 receiver intervals at30–45s **while the cutoff remained active**.
Whole fault-window loss was **15.00–25.06%**, not zero. These are deliberate path
fault tests, **not UDP600 throughput measurements**. Size recovery is not seamless
for every application's shorter timeout; growth to a larger PMTU after10min,
Android native/APK, physical roaming, RTT/loss/old-peer matrices and long soak are
not qualified. This snapshot does not certify current packaged assets or all platforms.

**Initial failures remain visible:** local OUTPUT DROP caused UDP `EPERM` and was
replaced with receiver INPUT loss; one FakeTLS IPv4 download iperf ended with a
control-stream read error after the server sent about9s DATA, then a separate
60s recheck passed; old H2's first recovery iperf failed and the next passed.
The first cleanup left an empty test-created nftables table: the mismatch and
the verified empty-table-only repair are retained. Test helper revisions/binding
to TUN/delayed fault activation were recorded; the candidate and Qeli settings did
not change. Successful repeats do not erase failed attempts or establish their
impossibility in future application traffic.

[Provenance, per-case evidence, initial failures and restoration receipts](https://github.com/litvinovtd/qeli/blob/v0.8.3/docs/data/benchmarks/network_targeted_083_required_2026-10-23.json).
The release-tag evidence link is not claimed live before publication.

### Twelve-profile speed matrix, Recordizer required — 2026-10-23

The same candidate SHA `09d0586e03a68f2cf22aebdb8ff2dd38d04063f0c0e0b6bd1f0f271997a177bd` completed **36 logical profile repetitions / 600 primary windows**:
576 speed windows (three repeats of all 12 profiles) and 24 before/after baselines.
An additional 24 bridge baselines after two interruptions are retained separately:
624 complete raw windows in total. There were 37 physical session starts because
one repetition was split across sessions. This is **not a continuous run**. After 7 sessions /
124 windows the original helper failed in halt while an exiting process lost
`/proc/PID/exe`. The original FAIL/raw and successful 17/17 emergency cleanup
are immutable. Only halt was fixed (12 local harness regression tests): waiting
uses exact starttime/state; all signals still require strict ownership checks.
The continuation did not replace or repeat the 112 completed speed windows.
The ELF, loads and settings were unchanged; private path changes and the pause
are documented in JSON.
The second interruption lost the Windows controller near sandbox-service stop
requests at 16:11:10 and 16:12:05 MSK. After 180 s without heartbeat the watchdog
stopped the workloads; separate recovery verified 17/17 state fields. An external
execution-environment restart is the likely cause; its initiator/exit code is unknown.
The interrupted `tcp-obfs-awg` IPv4 UDP300 download had completed iperf but lacked
both observers: that raw remains excluded and only that cell was repeated.
Other completed windows were neither repeated nor replaced.
The 2 vCPU / 2 GiB lab and settings above were retained; no fault filters or tuning.
Outer IPv4, inner IPv4/IPv6 up/down, 10 s + 2 s omit, TCP P1/P4, UDP300/600,
payload 1200 bytes. This is the extended speed subset, not the full release/platform gate.

TCP P4 receiver Mbit/s, median of 3 repeats:

| Profile | IPv4 ↑ | IPv4 ↓ | IPv6 ↑ | IPv6 ↓ | Retr Σ | TUN TX Σ C/S |
|---|---|---|---|---|---|---|
| `tcp-plain-raw` | 1952.0 | 2047.7 | 1927.1 | 2010.0 | 16120 | 10338 / 13699 |
| `tcp-faketls` | 1974.3 | 2033.9 | 1922.3 | 2029.1 | 13914 | 9837 / 12766 |
| `tcp-padding` | 1960.2 | 2031.4 | 1919.3 | 2038.0 | 14778 | 8410 / 13940 |
| `tcp-frag` | 1972.3 | 2058.3 | 1924.1 | 2040.3 | 14779 | 10835 / 13027 |
| `tcp-obfs` | 1445.0 | 1476.8 | 1330.2 | 1483.3 | 10644 | 8862 / 10785 |
| `tcp-reality` | 1977.5 | 2066.7 | 1917.0 | 2059.0 | 13999 | 9017 / 12096 |
| `tcp-reality-tls` | 1216.9 | 1212.2 | 1208.7 | 1254.3 | 8482 | 9282 / 8578 |
| `udp-faketls` | 1117.7 | 1231.9 | 1101.5 | 1191.6 | 16423 | 11724 / 11211 |
| `udp-padding` | 1115.3 | 1173.5 | 1086.1 | 1214.8 | 13117 | 15771 / 10064 |
| `udp-quic` | 1172.6 | 1197.1 | 1118.0 | 1179.1 | 17415 | 12943 / 10511 |
| `tcp-obfs-awg` | 1391.5 | 1470.1 | 1390.5 | 1462.1 | 10066 | 11909 / 12022 |
| `udp-faketls-awg` | 1106.3 | 1208.5 | 1116.5 | 1243.3 | 20481 | 16454 / 12054 |

UDP600 receiver Mbit/s (median of 3 repeats) / worst loss across 3 repeats:

| Profile | IPv4 ↑ | IPv4 ↓ | IPv6 ↑ | IPv6 ↓ | Retr Σ | TUN TX Σ C/S |
|---|---|---|---|---|---|---|
| `tcp-plain-raw` | 599.6 / 0.0029% | 600.0 / 0.0000% | 599.7 / 0.0000% | 600.0 / 0.0072% | — | 0 / 0 |
| `tcp-faketls` | 599.5 / 0.0037% | 600.0 / 0.0051% | 599.5 / 0.0000% | 600.0 / 0.0027% | — | 0 / 0 |
| `tcp-padding` | 599.7 / 0.0099% | 600.0 / 0.0056% | 599.5 / 0.0184% | 600.0 / 0.0000% | — | 0 / 0 |
| `tcp-frag` | 599.5 / 0.0000% | 600.0 / 0.0000% | 599.5 / 0.0024% | 600.0 / 0.0000% | — | 349 / 0 |
| `tcp-obfs` | 599.6 / 0.0181% | 600.0 / 0.0214% | 599.5 / 0.2709% | 599.9 / 0.0208% | — | 1512 / 0 |
| `tcp-reality` | 599.6 / 0.0141% | 600.0 / 0.0208% | 599.5 / 0.0018% | 600.0 / 0.0054% | — | 0 / 0 |
| `tcp-reality-tls` | 599.1 / 0.0026% | 600.0 / 0.0000% | 599.5 / 0.0037% | 600.0 / 0.0000% | — | 0 / 0 |
| `udp-faketls` | 599.6 / 0.0010% | 600.0 / 0.0075% | 599.6 / 0.0000% | 600.0 / 0.0066% | — | 0 / 0 |
| `udp-padding` | 599.5 / 0.0093% | 600.0 / 0.0000% | 599.7 / 0.0027% | 600.0 / 0.0035% | — | 0 / 0 |
| `udp-quic` | 599.5 / 0.0374% | 600.0 / 0.0050% | 599.8 / 0.0019% | 600.0 / 0.0297% | — | 0 / 0 |
| `tcp-obfs-awg` | 599.5 / 0.0293% | 600.0 / 0.0134% | 599.5 / 0.0187% | 600.0 / 0.0234% | — | 0 / 0 |
| `udp-faketls-awg` | 599.5 / 0.0000% | 600.0 / 0.6616% | 599.5 / 0.0000% | 600.0 / 0.0000% | — | 0 / 3607 |

`Retr Σ` sums sender retransmits over 12 TCP P4 windows; `TUN TX Σ C/S` sums
client/server TX dropped over the corresponding 12 windows. Observation includes
warm-up/tail, iperf retransmits exclude omit; FULL_RING was not traced. UDP600 is
offered load, not a discovered ceiling. Unbounded-TCP TUN drops alone do not prove
a reader defect; resource/queue limits need separate diagnosis.

| Unstable TCP cell | Speed CV |
|---|---|
| `udp-quic` / IPv4 / upload / P1 | 6.578% |
| `udp-faketls-awg` / IPv4 / upload / P4 | 5.206% |
| `udp-faketls-awg` / IPv6 / upload / P1 | 5.007% |

These cells are not qualified as stable; CV > 5% alone does not establish regression without matched A/B.

Status: **REVIEW_REQUIRED**. CV>5% speed cells: 3/192;
UDP loss>1% groups: 0; maximum baseline CV
3.47%, drift 2.32%.
Both guests restored 17/17, before/after SHA matched and previous raw was preserved.
Raw: `/opt/qeli-release-short-083-required-speed12-20261023`, `/opt/qeli-release-short-083-required-speed12-20261023-r02`, `/opt/qeli-release-short-083-required-speed12-20261023-r03` on both guests.
Maximum bridge baseline CV/drift from the original before baseline:
2.50% (5% review threshold).
[Full TCP P1/P4, UDP300/600, per-repeat CPU/RSS and counters](https://github.com/litvinovtd/qeli/blob/v0.8.3/docs/data/benchmarks/release12_083_required_2026-10-23.json).
The link needs the future documented commit/tag; availability before it is not claimed.
There is no matched speed A/B of old/new SHA or Android qualification in this run.
Failed budgets and unstable cells remain visible; the release is not published.

### Targeted three-cell TCP repeatability follow-up — 2026-10-23

Same SHA `09d0586e03a68f2cf22aebdb8ff2dd38d04063f0c0e0b6bd1f0f271997a177bd`, Recordizer required / PACKET_MUX_V1, VM300/301:
2 vCPU / 2 GiB, kernel 6.12.111+deb13-amd64, unchanged boot IDs/config fixtures,
TUN MTU 1400 / tx_queue_len 1000, server tun.queues 2. No tuning or rebuild.
Five new repeats per upload cell, 10 s + 2 s omit, outer IPv4; 10 fresh sessions
of two profiles with alternating order, 15 speed windows. Another 12 direct
IPv4/IPv6 upload baseline windows before/after give 27 new windows in total.
The original n=3 is **neither replaced nor pooled into n=8**. Speeds are medians.

| Cell (upload) | Original n=3: Mbit/s / CV | New n=5: Mbit/s / CV | Median delta | New min–max | Retr Σ5 | TUN TX Σ5 C/S |
|---|---|---|---|---|---|---|
| `udp-faketls-awg` IPv4 P4 | 1106.3 / 5.206% | 1133.0 / 1.357% | +2.41% | 1103.5–1140.9 | 5914 | 7435 / 0 |
| `udp-faketls-awg` IPv6 P1 | 1176.2 / 5.007% | 1133.4 / 1.478% | -3.64% | 1114.9–1155.6 | 1118 | 4221 / 0 |
| `udp-quic` IPv4 P1 | 1231.9 / 6.578% | 1104.2 / 2.319% | -10.36% | 1094.2–1153.0 | 1041 | 925 / 0 |

Short-control outcome: **FOLLOWUP_REVIEW_REQUIRED**. New cells with CV > 5%:
0/3; |median delta| > 5%:
1/3. Maximum baseline CV
1.45% and drift 0.94%.
This is targeted repeatability: other original windows/their warm-up history
were not reproduced. Five successful repeats do not erase the original
REVIEW_REQUIRED, establish the cause of variance, or constitute version A/B.
Retr Σ5 and TUN TX Σ5 sum five new windows, not directly comparable n=3 totals;
per-window means and every repeat are in the data. Observation includes warm-up/tail,
iperf retransmits exclude omit; FULL_RING was not traced. Remaining drops and
retransmits are not declared fixed or hidden. Both guests restored 17/17,
before/after SHA matched, previous raw preserved. New raw on both guests: `/opt/qeli-release-short-083-required-stability3-20261023`.
[All n=3/n=5, baseline, CPU/RSS, retransmits and drops](https://github.com/litvinovtd/qeli/blob/v0.8.3/docs/data/benchmarks/tcp_stability3_083_required_2026-10-23.json).
The release-tag link is not claimed live before a future commit/tag.
Qualification is for this ELF only, not newer source changes or automatically
the packaged assets. The release remains unpublished.

### Linux candidate short test — 2026-10-22

An earlier packaged **0.8.3 candidate** was tested on two isolated Linux guests:
SHA-256 `95cf32f8a9bd76de855e1e599c88fc9feb94a677ccef99c61b35762f27d467f7`,
source `be4429ae8c870c88b2682619ba8246fae3f8a4b4`. Later transport and Android
carrier-recovery changes are outside this test; it does not qualify the current
development tree. These are not measurements
of an earlier diagnostic candidate. Each VM had **2 vCPU / 2 GiB RAM**, Intel
Core i5-8500, Linux `6.12.111+deb13-amd64`, iperf3 3.18 and a dedicated local
test link. TUN MTU **1400**, `tun.tx_queue_len=1000` on both peers, two server
TUN queues; **Recordizer required**, active PACKET_MUX_V1 verified in every session.
Reality-TLS: genuine H2, handrolled and shaping, no padding/stealth/heartbeat.
No manual tuning, reboot, package installation or rebuild was performed.

Required-only subset: 12-profile functional check, external-IPv6 smoke checks
of the four main profiles, **three speed repeats** of each main profile, extended
TCP windows and before/after baselines: **32 sessions / 288 windows**. Speed
windows were **10 s + 2 s warm-up**, TCP P1/P4 and UDP 300/600 Mbit/s, inner
IPv4/IPv6 in both directions; UDP payload 1200. Off/prefer, mixed-version/mixed-load
and long-soak gates are outside this run. UDP600 is a fixed offered load, not a ceiling.
P4 means four iperf streams inside one tunnel; `gateway=false`, `dns=off`.

TCP P4 receiver Mbit/s, median of three repeats:

| Profile | IPv4 ↑ | IPv4 ↓ | IPv6 ↑ | IPv6 ↓ | Retr Σ | TUN TX Σ C/S |
|---|---|---|---|---|---|---|
| `tcp-faketls` | 1976.8 | 2054.3 | 1921.4 | 2045.0 | 14411 | 9222 / 11837 |
| `tcp-reality-tls` | 1210.3 | 1208.0 | 1215.0 | 1291.2 | 8963 | 9370 / 9149 |
| `udp-faketls` | 1165.9 | 1220.7 | 1137.0 | 1208.8 | 15349 | 14904 / 11642 |
| `udp-quic` | 1096.3 | 1206.6 | 1104.9 | 1175.9 | 14874 | 10043 / 13197 |

UDP600 receiver Mbit/s / **worst loss %** across the three repeats:

| Profile | IPv4 ↑ | IPv4 ↓ | IPv6 ↑ | IPv6 ↓ | TUN TX Σ C/S |
|---|---|---|---|---|---|
| `tcp-faketls` | 599.6 / 0.0000 | 600.0 / 0.0088 | 599.8 / 0.0000 | 600.0 / 0.0000 | 0 / 0 |
| `tcp-reality-tls` | 599.6 / 0.0034 | 600.0 / 0.0000 | 599.1 / 0.0010 | 600.0 / 0.0000 | 0 / 0 |
| `udp-faketls` | 599.4 / 0.0000 | 600.0 / 0.0000 | 599.6 / 0.0059 | 600.0 / 0.0030 | 0 / 0 |
| `udp-quic` | 599.6 / 0.0082 | 600.0 / 0.0000 | 599.6 / 0.0000 | 600.0 / 0.0000 | 0 / 0 |

`Retr Σ` sums the 12 TCP P4 sender retransmit counts per profile. `TUN TX Σ C/S`
sums client/server TX drops across the corresponding 12 windows (TCP P4 or UDP600).
TUN observation includes warm-up and a short tail; iperf retransmits exclude warm-up.
These are not identical counter intervals; FULL_RING was not traced in this test.

Result: **`REVIEW_REQUIRED`**. 0/96 speed UDP windows exceeded 1% loss (worst 0.0088%); 3/64 speed cells had CV >5% (maximum 7.64%). Maximum baseline drift 2.07%, CV 1.46%. No all-drops-fixed or regression-free claim is made.
Both guests passed **17/17 restoration checks**; boot IDs, resources, network,
firewall, checked sysctls and prior `/opt/qeli-*` file hashes were unchanged.
The tested ELF SHA was checked again on both guests and locally. Old binaries
and raw were preserved; new raw remains separately on both guests at
`/opt/qeli-release-short-083-required-20261022-r02`.

[Full repeat-level data, TCP P1/UDP300, CPU/RSS, counters and baselines](https://github.com/litvinovtd/qeli/blob/v0.8.3/docs/data/benchmarks/release_short_083_required_2026-10-22.json);
[short-test method and limitations](https://github.com/litvinovtd/qeli/blob/v0.8.3/docs/eng/manuals/RELEASE-SHORT-TEST.md).
This local Linux test is not an Internet throughput guarantee, a matched regression
A/B or a substitute for the remaining release/platform gates. Release remains a draft.
The evidence identifies the tested snapshot; its release-tag link is not claimed live
before that tag is published.

## Русский

Обновление снимает ограничение TCP-буферов на каналах с большим RTT и добавляет
диагностику давления на очереди UDP/TUN. Ручные параметры и существующие бюджеты
памяти сохраняются.

- Диагностика клиента различает очередь, пакеты в работе, локальное завершение
  записи и отмены TCP uplink в ограниченном снимке. Локальное завершение — не peer ACK.
- Android получил ограниченный асинхронный журнал, ручной структурированный экспорт
  без свободного текста сообщений, профилей, ключей и сетевых адресов и опциональное
  сохранение последней ошибки. Очистка отсекает отложенные события и экспорты.
- График Windows-службы использует время исходного снимка и uptime с учётом сна;
  дубликаты снимков и скачки календарных часов не создают ложных отсчётов скорости.
  Эти правки разработки не охвачены замерами более раннего кандидата ниже.

- Серверные TCP-буферы по умолчанию управляются ОС: оба `perf.tcp.*_buffer_size`
  равны `0`. Установщик Linux явно включает receive autotuning, проверяет readback
  и сохраняет данные для отката. Ручные значения сохраняются; мультипрофильные
  примеры и «Быстрый запуск» панели используют те же дефолты.
- Внутренняя очередь приёма UDP фиксирована на 128 датаграммах на pump. Метрики
  глубины и ожидания помогают найти перегрузку без бесконтрольного роста очереди.
  Ограниченный контроллер socket receive buffer и ручные параметры TUN сохранены.
- TUN dispatch сначала пробует немедленную передачу и сохраняет backpressure при
  заполнении. `tun-stats` разделяет ожидания пула, передачи обработчику, паузы чтения
  и UDP batch-send. Устранение всех TUN drops и постоянной перегрузки не заявляется.
- Ручной клиентский MTU туннеля больше не отключает проверку внешнего UDP path MTU.
  `mtu_probe = false` сохраняет приоритет; при неудачной пробе действует безопасный fallback.
- macOS поставляется отдельными self-contained архивами `Qeli-macos-arm64.tar.gz`
  для Apple Silicon и `Qeli-macos-x86_64.tar.gz` для Intel; нужен macOS 15 или новее.
  Универсальный упаковщик отклоняет несовместимые .NET-компоненты: `lipo` не объединяет
  архитектурные managed DLL. Нативное Rust-ядро сохраняет обе архитектуры.
- macOS: исправлены `openat` на Apple Silicon, восстановление прав старых root-owned
  lock-файлов, распознавание состояния launchctl и обработка ошибок отключения/статуса.
- Android Tink обновлён до 1.24.0, Rust libc — до 0.2.191. rustls-webpki 0.103.16
  и webpki-roots 1.0.10 закрывают новые предупреждения о доверии сертификатам
  RUSTSEC-2026-0382 и RUSTSEC-2026-0381. Лицензии согласованы с новыми lock-файлами.
- ML-KEM 0.3.3 устраняет secret-dependent lookup при CBD sampling; ctutils 0.4.4
  закрывает RUSTSEC-2026-0385. Tokio обновлён до 1.54.1, Gradle Wrapper — до 9.8.2.
  Основной/fuzz Cargo.lock и встроенные оригинальные тексты лицензий согласованы.
- Обновлены инструкции установки, примеры конфигов и диагностика буферов Docker.

TCP A/B проверялся на 1/8 независимых клиентах с добавленным RTT 40/100 мс в лабе
из двух Linux VM. В одном сценарии upload восьми клиентов медиана суммарной
скорости составила 2 456 Мбит/с с буферами ОС против 279 Мбит/с с фиксированными
262 144 байтами. Это сумма восьми туннелей на локальном пути, а не обещание скорости
одного клиента или Интернета. [Методика и результаты](https://github.com/litvinovtd/qeli/blob/v0.8.3/docs/data/benchmarks/queue-tuning-20261021.json).

На существующем Linux-сервере задайте оба TCP buffer size в профиле равными `0`,
проверьте `tcp_moderate_rcvbuf=1` и пределы `tcp_rmem`/`tcp_wmem`, выполните проверку
конфига и перезапуск с переподключением клиентов. Глобальный `tcp_mem` сохраняется.
Принудительная переустановка для этого не нужна.
[Инструкция](https://github.com/litvinovtd/qeli/blob/v0.8.3/docs/ru/manuals/GETTING-STARTED.md).

Ранее принятое ограничение Android API34 ENONET и исключённые физические стенды
сохраняют прежний статус; это обновление не объявляет их исправленными.


### Доработки по issue #69

- Проверка SNI учитывает реальное использование TLS/WebSocket; старое SNI UDP
  obfs не блокирует рабочий профиль. Лишнее поле не экспортируется в новые ссылки.
  Desktop-редактор показывает HTTP Host для TCP WebSocket.
- Общий `pull_routes=false` отключает только серверные маршруты, сохраняя
  локальную политику и защищённый tunnel DNS. Настройка переносится всеми
  клиентами через INI/модели/backup/новые ссылки. Исправлено сохранение gateway=false.
- NDP выбирает назначенный source uplink, повторно проверяет готового владельца,
  использует MLD для известных адресов и ограниченный анонс появления. В manual
  маршруты/firewall остаются у администратора. Произвольных NA-флагов и периодической
  рассылки нет.
- UDP-диагностика разделяет inner MTU, внешний бюджет и причины fallback;
  счётчики DATA_FRAG показывают попытки разделения записей и созданные фрагменты.
- Windows: typed IPv4/IPv6/copy в GUI и службе, размеры редакторов и bounded график.
  Выключенная по умолчанию политика sharing-bindings применяется только к своему Wintun.

[План и границы проверки](https://github.com/litvinovtd/qeli/blob/v0.8.3/release/certification/plans/ISSUE69_0.8.3.md).
Fit-path, периодические NDP-анонсы и постоянный адаптер — отдельные этапы.
Реальные Windows bindings и MLD-snooping провайдера не объявляются проверенными
на основании mock/namespace-тестов.

### Проверки TCP/H2 и live UDP PMTU — 2026-10-23

Отдельный **Linux-кандидат**, а не пакетный ELF/assets ниже, собран в release mode
с `jemalloc` только на сборочной VM302 из
`fb741e204cc82f8e0c43172d5746c1a6afa78607`:
SHA-256 **`09d0586e03a68f2cf22aebdb8ff2dd38d04063f0c0e0b6bd1f0f271997a177bd`**.
Контроль: `95cf32f8a9bd76de855e1e599c88fc9feb94a677ccef99c61b35762f27d467f7`.
Прежние assets релиза и скоростная таблица четырёх профилей не заменены.

VM300/server и VM301/client: каждая **2 vCPU / 2 GiB**, Linux
`6.12.111+deb13-amd64`, iperf3 3.18, выделенная eth1 тестовая сеть
172.30.82.10/11 и fd42:82:1::10/11. TUN MTU **1400**, `tun.tx_queue_len=1000`
на обеих сторонах, две серверные TUN-очереди, **Recordizer required / active PACKET_MUX_V1**.
Reality-TLS: genuine H2, handrolled и shaping, без padding/stealth/heartbeat.
Без тюнинга ресурсов/очередей/qdisc/sysctl и reboot; сборки только на302,
на Proxmox сборки, установки и runtime-действия не выполнялись.
Восстановление ВМ **17/17**, хеши прежних файлов сохранены.

- **37 уникальных Linux-регрессий пройдены**: независимые H2 DATA/PING deadlines,
  отмена writer, PMTU confirmations/устаревшие ответы, DATA_FRAG retry identity
  и владение пулом. Отдельный test/debug executable имеет SHA
  `7c37b758b76b3deb2dd090d57d781c5053a08bfa92c2698d58bff1ad50350cc1`;
  его результат не подменяет сетевую проверку поставляемого ELF.
- Release-mode кандидат прошёл70с здорового idle с последующей полезной TCP
  передачей inner IPv4/IPv6 up/down на FakeTLS, WebSocket obfs и genuine H2 (15 окон).
- A/B старого/нового FakeTLS и H2:50Mbps inner UDP поверх TCP,65с обрыва
  carrier. Кандидат фиксирует **30с без продвижения pending write/flush**,
  терминально закрывает повреждённый carrier и возвращает полезный TCP без ручного
  restart. У старой заблокированной записи такого ограниченного завершения не было.
  В первом H2 TCP-over-TCP окне триггером был inbound liveness, не PING timeout.
  Production whole-carrier потеря не изолирует WINDOW_UPDATE от PING.
- **8/8 live PMTU сценариев профиль/outer-family/направление** приняты с учётом
  сохранённого неполного окна и отдельной перепроверки.10Mbps, inner IPv6 UDP
  payload1352,60с; INPUT получателя отбрасывал наружные IP-пакеты>1280.
  Меньший budget подтверждался за **9,6–14,0с**, DATA_FRAG сохранял TUN MTU1400,
  reconnect не требовался. Post-fault обмен inner IPv4/IPv6 подтверждён.

| UDP-профиль | Outer | Budget upload | Budget download | Steady receiver/loss |
|---|---|---|---|---|
| FakeTLS | IPv4 |1448→548|1448→548|≈10Mbps /0%|
| FakeTLS | IPv6 |1448→1232|1448→1232|≈10Mbps /0%|
| QUIC | IPv4 |1457→548|1457→548|≈10Mbps /0%|
| QUIC | IPv6 |1417→1232|1417→1232|≈10Mbps /0%|

Steady —15 receiver-интервалов30–45с **при ещё действующем ограничении**.
Потеря за всё fault-окно — **15,00–25,06%**, не нулевая. Это намеренные path fault
тесты, **не скоростные UDP600 замеры**. Восстановление размера не бесшовно для
любого короткого прикладного timeout. Рост PMTU через10мин, Android native/APK,
физический roaming, RTT/loss/старый peer и длительный soak не проверены.
Snapshot не является допуском нынешних пакетных assets или всех платформ.

**Первичные отказы сохранены:** локальный OUTPUT DROP дал UDP `EPERM`, поэтому
заменён потерей на INPUT получателя; одно FakeTLS IPv4 download iperf-окно
завершилось ошибкой чтения control stream после примерно9с DATA на сервере,
отдельный60с повтор прошёл; у старого H2 первая recovery-проверка iperf упала,
следующая прошла. Первый cleanup оставил пустую тестовую nftables-таблицу:
несовпадение и восстановление только доказанно пустой таблицы сохранены.
Версии помощника, привязка генератора к TUN и отложенное включение потерь
задокументированы; кандидат и настройки Qeli не менялись. Успех повторов не
стирает FAIL и не доказывает невозможность таких прикладных ошибок в дальнейшем.

[Provenance, данные каждого сценария, первичные отказы и восстановление](https://github.com/litvinovtd/qeli/blob/v0.8.3/docs/data/benchmarks/network_targeted_083_required_2026-10-23.json).
Доступность release-tag ссылки до публикации не заявляется.

### Скорость 12 профилей, Recordizer required — 2026-10-23

На том же кандидате SHA `09d0586e03a68f2cf22aebdb8ff2dd38d04063f0c0e0b6bd1f0f271997a177bd` завершены **36 логических повторов профилей / 600 основных окон**:
576 скоростных (три повтора всех 12 профилей) и 24 baseline до/после.
Ещё 24 bridge baseline после двух пауз сохранены отдельно: всего 624 полных
raw-окна. Физических подключений 37: один повтор разделён между двумя сессиями.
Это **не непрерывная серия**: после 7 сессий / 124 окон исходный сборщик
получил FAIL в halt при чтении исчезающего `/proc/PID/exe` во время выхода.
Первичный FAIL, raw и успешный аварийный cleanup 17/17 сохранены неизменно.
Исправлен только halt сборщика (12 локальных regression-проверок); ожидание
использует starttime/state, любые сигналы требуют строгой проверки владельца.
Продолжение не повторяло 112 завершённых скоростных окон. Бинарник, нагрузки
и настройки не менялись; разные приватные пути и пауза отражены в JSON.
Второй раз локальный Windows-контроллер исчез рядом с запросами остановки
`CodexSandboxService` в 16:11:10 и 16:12:05 МСК. Через 180 с без heartbeat
watchdog остановил нагрузку; восстановление 17/17 подтверждено отдельно.
Внешний перезапуск среды — вероятная причина, но инициатор/exit code неизвестны.
У прерванного `tcp-obfs-awg` UDP300 IPv4 download iperf завершился, однако
оба observers отсутствовали: первичный raw сохранён и не принят, повторена
только эта ячейка. Другие завершённые окна не повторялись и не заменялись.
Стенд 2 vCPU / 2 GiB и настройки выше сохранены, без фильтров неисправностей и тюнинга.
Outer IPv4, inner IPv4/IPv6 up/down, 10 с + 2 с omit, TCP P1/P4, UDP300/600,
payload 1200 байт. Это расширенная скоростная часть, не весь release/platform gate.

TCP P4 receiver Мбит/с, медиана 3 повторов:

| Профиль | IPv4 ↑ | IPv4 ↓ | IPv6 ↑ | IPv6 ↓ | Retr Σ | TUN TX Σ К/С |
|---|---|---|---|---|---|---|
| `tcp-plain-raw` | 1952.0 | 2047.7 | 1927.1 | 2010.0 | 16120 | 10338 / 13699 |
| `tcp-faketls` | 1974.3 | 2033.9 | 1922.3 | 2029.1 | 13914 | 9837 / 12766 |
| `tcp-padding` | 1960.2 | 2031.4 | 1919.3 | 2038.0 | 14778 | 8410 / 13940 |
| `tcp-frag` | 1972.3 | 2058.3 | 1924.1 | 2040.3 | 14779 | 10835 / 13027 |
| `tcp-obfs` | 1445.0 | 1476.8 | 1330.2 | 1483.3 | 10644 | 8862 / 10785 |
| `tcp-reality` | 1977.5 | 2066.7 | 1917.0 | 2059.0 | 13999 | 9017 / 12096 |
| `tcp-reality-tls` | 1216.9 | 1212.2 | 1208.7 | 1254.3 | 8482 | 9282 / 8578 |
| `udp-faketls` | 1117.7 | 1231.9 | 1101.5 | 1191.6 | 16423 | 11724 / 11211 |
| `udp-padding` | 1115.3 | 1173.5 | 1086.1 | 1214.8 | 13117 | 15771 / 10064 |
| `udp-quic` | 1172.6 | 1197.1 | 1118.0 | 1179.1 | 17415 | 12943 / 10511 |
| `tcp-obfs-awg` | 1391.5 | 1470.1 | 1390.5 | 1462.1 | 10066 | 11909 / 12022 |
| `udp-faketls-awg` | 1106.3 | 1208.5 | 1116.5 | 1243.3 | 20481 | 16454 / 12054 |

UDP600 receiver Мбит/с (медиана 3 повторов) / худшая потеря из 3 повторов:

| Профиль | IPv4 ↑ | IPv4 ↓ | IPv6 ↑ | IPv6 ↓ | Retr Σ | TUN TX Σ К/С |
|---|---|---|---|---|---|---|
| `tcp-plain-raw` | 599.6 / 0.0029% | 600.0 / 0.0000% | 599.7 / 0.0000% | 600.0 / 0.0072% | — | 0 / 0 |
| `tcp-faketls` | 599.5 / 0.0037% | 600.0 / 0.0051% | 599.5 / 0.0000% | 600.0 / 0.0027% | — | 0 / 0 |
| `tcp-padding` | 599.7 / 0.0099% | 600.0 / 0.0056% | 599.5 / 0.0184% | 600.0 / 0.0000% | — | 0 / 0 |
| `tcp-frag` | 599.5 / 0.0000% | 600.0 / 0.0000% | 599.5 / 0.0024% | 600.0 / 0.0000% | — | 349 / 0 |
| `tcp-obfs` | 599.6 / 0.0181% | 600.0 / 0.0214% | 599.5 / 0.2709% | 599.9 / 0.0208% | — | 1512 / 0 |
| `tcp-reality` | 599.6 / 0.0141% | 600.0 / 0.0208% | 599.5 / 0.0018% | 600.0 / 0.0054% | — | 0 / 0 |
| `tcp-reality-tls` | 599.1 / 0.0026% | 600.0 / 0.0000% | 599.5 / 0.0037% | 600.0 / 0.0000% | — | 0 / 0 |
| `udp-faketls` | 599.6 / 0.0010% | 600.0 / 0.0075% | 599.6 / 0.0000% | 600.0 / 0.0066% | — | 0 / 0 |
| `udp-padding` | 599.5 / 0.0093% | 600.0 / 0.0000% | 599.7 / 0.0027% | 600.0 / 0.0035% | — | 0 / 0 |
| `udp-quic` | 599.5 / 0.0374% | 600.0 / 0.0050% | 599.8 / 0.0019% | 600.0 / 0.0297% | — | 0 / 0 |
| `tcp-obfs-awg` | 599.5 / 0.0293% | 600.0 / 0.0134% | 599.5 / 0.0187% | 600.0 / 0.0234% | — | 0 / 0 |
| `udp-faketls-awg` | 599.5 / 0.0000% | 600.0 / 0.6616% | 599.5 / 0.0000% | 600.0 / 0.0000% | — | 0 / 3607 |

`Retr Σ` — сумма sender retransmits 12 TCP P4 окон; `TUN TX Σ К/С` — сумма
client/server TX dropped соответствующих 12 окон. Observer включает прогрев/tail,
iperf retransmits исключают omit; FULL_RING не трассировался. UDP600 — offered
load, не найденный потолок. Рост TUN drops при unbounded TCP не объявляется
автоматически ошибкой reader; ограничение ресурсов/очередей требует диагностики.

| Нестабильная TCP-ячейка | CV скорости |
|---|---|
| `udp-quic` / IPv4 / upload / P1 | 6.578% |
| `udp-faketls-awg` / IPv4 / upload / P4 | 5.206% |
| `udp-faketls-awg` / IPv6 / upload / P1 | 5.007% |

Эти ячейки не объявляются стабильными; CV > 5% не доказывает регрессию без matched A/B.

Статус: **REVIEW_REQUIRED**. Ячеек с CV>5%: 3/192;
UDP-групп с loss>1%: 0; максимальный baseline CV
3.47%, drift 2.32%.
Обе ВМ восстановлены 17/17, SHA до/после совпали, старые raw сохранены.
Raw: `/opt/qeli-release-short-083-required-speed12-20261023`, `/opt/qeli-release-short-083-required-speed12-20261023-r02`, `/opt/qeli-release-short-083-required-speed12-20261023-r03` на обеих ВМ.
Максимум bridge baseline CV/дрейфа от первоначального baseline:
2.50% (порог рассмотрения 5%).
[Полные TCP P1/P4, UDP300/600, каждый повтор, CPU/RSS и counters](https://github.com/litvinovtd/qeli/blob/v0.8.3/docs/data/benchmarks/release12_083_required_2026-10-23.json).
Ссылке нужен будущий документированный commit/tag; доступность до него не заявляется.
Сопоставимого скоростного A/B старого/нового SHA и Android qualification здесь нет.
Провалы бюджета и нестабильные ячейки остаются в данных, релиз не публикуется.

### Целевая перепроверка трёх TCP-ячеек — 2026-10-23

Тот же SHA `09d0586e03a68f2cf22aebdb8ff2dd38d04063f0c0e0b6bd1f0f271997a177bd`, Recordizer required / PACKET_MUX_V1, VM300/301:
2 vCPU / 2 GiB, kernel 6.12.111+deb13-amd64, прежние boot ID и конфиги,
TUN MTU 1400 / tx_queue_len 1000, server tun.queues 2. Без тюнинга и пересборки.
Пять новых повторов каждой upload-ячейки, 10 с + 2 с omit, outer IPv4;
10 свежих сессий двух профилей с чередующимся порядком, 15 скоростных окон.
Ещё 12 прямых baseline-окон IPv4/IPv6 upload до/после; всего 27 новых окон.
Исходные n=3 **не заменены и не объединены в n=8**. В таблице — медианы скорости.

| Ячейка (upload) | Исходные n=3: Мбит/с / CV | Новые n=5: Мбит/с / CV | Δ медианы | Новый min–max | Retr Σ5 | TUN TX Σ5 К/С |
|---|---|---|---|---|---|---|
| `udp-faketls-awg` IPv4 P4 | 1106.3 / 5.206% | 1133.0 / 1.357% | +2.41% | 1103.5–1140.9 | 5914 | 7435 / 0 |
| `udp-faketls-awg` IPv6 P1 | 1176.2 / 5.007% | 1133.4 / 1.478% | -3.64% | 1114.9–1155.6 | 1118 | 4221 / 0 |
| `udp-quic` IPv4 P1 | 1231.9 / 6.578% | 1104.2 / 2.319% | -10.36% | 1094.2–1153.0 | 1041 | 925 / 0 |

Итог короткого контроля: **FOLLOWUP_REVIEW_REQUIRED**. Новых ячеек с CV > 5%:
0/3; с |Δ медианы| > 5%:
1/3. Максимальные baseline CV
1.45% и drift 0.94%.
Это targeted repeatability: другие окна исходных сессий/их история разогрева
не воспроизводились. Успех пяти повторов не стирает исходный REVIEW_REQUIRED,
не доказывает устранение причины разброса и не является версионным A/B.
Retr Σ5 и TUN TX Σ5 — суммы пяти новых окон, а не сопоставимые с n=3 totals;
средние на окно и все повторы есть в данных. Observer включает warm-up/tail,
iperf retransmits исключают omit; FULL_RING не трассировался.
Счётчики drops/retransmits не объявляются исправленными и не скрыты.
VM300/301 восстановлены 17/17, SHA до/после совпали, старые raw сохранены.
Новый raw на обеих ВМ: `/opt/qeli-release-short-083-required-stability3-20261023`.
[Все n=3/n=5, baseline, CPU/RSS, retransmits и drops](https://github.com/litvinovtd/qeli/blob/v0.8.3/docs/data/benchmarks/tcp_stability3_083_required_2026-10-23.json).
Доступность release-tag ссылки до будущего commit/tag не заявляется.
Данные относятся только к этому ELF, не к более новым изменениям исходников
или автоматически к packaged assets. Релиз не публикуется.

### Короткий тест Linux-кандидата — 2026-10-22

На двух изолированных Linux-гостях проверен прежний пакетный **кандидат 0.8.3**:
SHA-256 `95cf32f8a9bd76de855e1e599c88fc9feb94a677ccef99c61b35762f27d467f7`,
исходники `be4429ae8c870c88b2682619ba8246fae3f8a4b4`. Последующие изменения
транспорта и восстановления сети Android этим тестом не охвачены; он не является
допуском текущего дерева разработки. Это не перенос замеров
прежнего диагностического кандидата. Каждая ВМ: **2 vCPU / 2 GiB RAM**, Intel
Core i5-8500, Linux `6.12.111+deb13-amd64`, iperf3 3.18, выделенный локальный
тестовый линк. TUN MTU **1400**, `tun.tx_queue_len=1000` на обеих сторонах,
две серверные TUN-очереди; **Recordizer required**, активный PACKET_MUX_V1
подтверждён в каждой сессии. Reality-TLS: genuine H2, handrolled и shaping,
без padding/stealth/heartbeat. Ручного тюнинга, reboot, установок и пересборки нет.

Required-only подмножество: функциональная проверка 12 профилей, внешний IPv6
четырёх основных, **три скоростных повтора** каждого основного профиля, удлинённые
TCP-окна и baseline до/после — **32 сессии / 288 окон**. Скоростные окна:
**10 с + 2 с прогрева**, TCP P1/P4 и UDP 300/600 Мбит/с, внутренние IPv4/IPv6
в обе стороны; UDP payload 1200. Off/prefer, смешанные версии/нагрузки и длительный
soak сюда не входят. UDP600 — фиксированная предложенная нагрузка, не потолок.
P4 — четыре потока iperf внутри одного туннеля; `gateway=false`, `dns=off`.

TCP P4: receiver Мбит/с, медиана трёх повторов:

| Профиль | IPv4 ↑ | IPv4 ↓ | IPv6 ↑ | IPv6 ↓ | Retr Σ | TUN TX Σ К/С |
|---|---|---|---|---|---|---|
| `tcp-faketls` | 1976.8 | 2054.3 | 1921.4 | 2045.0 | 14411 | 9222 / 11837 |
| `tcp-reality-tls` | 1210.3 | 1208.0 | 1215.0 | 1291.2 | 8963 | 9370 / 9149 |
| `udp-faketls` | 1165.9 | 1220.7 | 1137.0 | 1208.8 | 15349 | 14904 / 11642 |
| `udp-quic` | 1096.3 | 1206.6 | 1104.9 | 1175.9 | 14874 | 10043 / 13197 |

UDP600: receiver Мбит/с / **худшая потеря %** из трёх повторов:

| Профиль | IPv4 ↑ | IPv4 ↓ | IPv6 ↑ | IPv6 ↓ | TUN TX Σ К/С |
|---|---|---|---|---|---|
| `tcp-faketls` | 599.6 / 0.0000 | 600.0 / 0.0088 | 599.8 / 0.0000 | 600.0 / 0.0000 | 0 / 0 |
| `tcp-reality-tls` | 599.6 / 0.0034 | 600.0 / 0.0000 | 599.1 / 0.0010 | 600.0 / 0.0000 | 0 / 0 |
| `udp-faketls` | 599.4 / 0.0000 | 600.0 / 0.0000 | 599.6 / 0.0059 | 600.0 / 0.0030 | 0 / 0 |
| `udp-quic` | 599.6 / 0.0082 | 600.0 / 0.0000 | 599.6 / 0.0000 | 600.0 / 0.0000 | 0 / 0 |

`Retr Σ` — сумма sender retransmits по 12 TCP P4 окнам профиля. `TUN TX Σ К/С` —
сумма TX drops клиента/сервера по соответствующим 12 окнам (TCP P4 или UDP600).
Наблюдение TUN включает прогрев и небольшой хвост, retransmits iperf исключают
прогрев; интервалы не тождественны. FULL_RING в этом тесте не трассировался.

Результат: **`REVIEW_REQUIRED`**. Порог потерь 1% превышен в 0/96 скоростных UDP-окнах (максимум 0.0088%); CV >5% у 3/64 скоростных ячеек (максимум 7.64%). Максимальные baseline drift 2.07%, CV 1.46%. Устранение всех drops и отсутствие регрессий не заявляются.
Обе ВМ прошли **17/17 проверок восстановления**: boot IDs, ресурсы, сеть, firewall,
проверяемые sysctl и хеши прежних файлов `/opt/qeli-*` совпали. SHA проверенного
ELF повторно сверен на обоих гостях и локально. Старые бинарники и raw сохранены;
новые raw отдельно на обеих ВМ:
`/opt/qeli-release-short-083-required-20261022-r02`.

[Все повторы, TCP P1/UDP300, CPU/RSS, счётчики и baseline](https://github.com/litvinovtd/qeli/blob/v0.8.3/docs/data/benchmarks/release_short_083_required_2026-10-22.json);
[методика короткого теста и ограничения](https://github.com/litvinovtd/qeli/blob/v0.8.3/docs/ru/manuals/RELEASE-SHORT-TEST.md).
Это локальная Linux-проверка, не гарантия скорости в Интернете, не сопоставимый
регрессионный A/B и не замена остальных release/platform gates. Релиз остаётся черновиком.
Данные относятся к указанному снимку; ссылка на release tag не объявляется рабочей
до публикации соответствующего тега.
