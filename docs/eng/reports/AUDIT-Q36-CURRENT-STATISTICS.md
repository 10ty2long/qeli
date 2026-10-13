# Q36: current data-plane statistical population

13 October 2026. All 12 modes have three fresh native sessions each: **36 sessions / 144 eight-second windows**, TCP/UDP bidir P1/P4, outer IPv4 / inner IPv4. Receiver results, requested iperf settings, unique session/cookies, client generation, CPU policy, CPU/RSS, kernel/application counters and SHA were independently admitted. No outliers or loss samples were excluded. This fills the missing statistical population for the adopted data-plane code; Q36 remains IN_PROGRESS.

## Conditions and provenance

Stock server/client run in separate network/private mount namespaces on one two-vCPU .10 VM, affinity0–1/nice0. TUN MTU1400 and carrier MTU1500; queue defaults unchanged. Actual CLI SHA `52511cebd43aca364e317c04955287630dc24055de32168206d280ddbee82069` is the qualified fat-LTO/codegen1/jemalloc build. All916 inputs verified; buffer_pool has only two previously verified formatting changes. assets.rs changed only under cfg(test), panel CSS changed, web disabled in these sessions. No byte equality with a newly built complete CLI after panel changes and no new rebuild are claimed.

The second pass reverses mode and phase order. Every condition has three independent sessions; raw JSON and all72 worker logs retained. No strace/BPF or changed product defaults. CPU/RSS use the existing qualified sampler without claiming zero observer cost.

## TCP medians and spread

↑ client to server, ↓ server to client. Spread = `(1-min/max)*100`, not a confidence interval. Full min/max and individual samples are in [evidence](../../../release/certification/evidence/q36-current-statistics-20261013.json).

| Mode | P1 ↑ Mbps | P1 ↓ Mbps | P4 ↑ Mbps | P4 ↓ Mbps | P1 spread ↑/↓ % | P4 spread ↑/↓ % |
|---|---:|---:|---:|---:|---:|---:|
| tcp-plain-raw | 173.93 | 580.03 | 151.12 | 575.30 | 38.05/18.23 | 34.28/9.53 |
| tcp-faketls | 169.27 | 580.85 | 121.94 | 590.12 | 25.98/15.62 | 31.08/9.84 |
| tcp-padding | 153.02 | 570.44 | 154.57 | 575.13 | 16.94/6.41 | 17.53/7.36 |
| tcp-frag | 160.32 | 583.79 | 156.22 | 550.75 | 24.96/14.13 | 25.56/6.95 |
| tcp-obfs | 121.47 | 554.75 | 103.14 | 527.51 | 15.66/8.53 | 24.68/6.32 |
| tcp-reality | 166.90 | 564.96 | 147.43 | 544.32 | 12.46/9.17 | 25.37/6.65 |
| tcp-reality-tls | 260.71 | 647.77 | 251.08 | 646.39 | 5.29/0.49 | 7.86/6.07 |
| udp-faketls | 255.16 | 525.24 | 203.33 | 547.61 | 21.00/12.06 | 12.48/2.94 |
| udp-padding | 188.60 | 583.37 | 213.36 | 537.88 | 18.66/5.32 | 5.62/4.29 |
| udp-quic | 232.61 | 561.31 | 189.66 | 527.73 | 10.88/19.59 | 17.19/7.40 |
| tcp-obfs-awg | 142.15 | 548.94 | 139.29 | 526.28 | 23.89/5.15 | 31.28/5.60 |
| udp-faketls-awg | 222.36 | 582.14 | 198.22 | 514.79 | 29.04/15.53 | 16.22/15.74 |

## UDP offered load and delivered data

100Mbps **per stream per direction**,1200-byte datagrams: P1=100+100Mbps, P4=400+400Mbps. Table values are three-session medians. Loss under this offered load is retained, never replaced with sender throughput or an overall performance PASS.

| Mode | P1 loss ↑/↓ % | P4 goodput ↑/↓ Mbps | P4 loss ↑/↓ % |
|---|---:|---:|---:|
| tcp-plain-raw | 0.00/0.00 | 202.42/283.19 | 49.28/27.26 |
| tcp-faketls | 0.00/0.00 | 196.37/284.49 | 48.94/27.00 |
| tcp-padding | 0.00/0.00 | 228.03/237.99 | 42.90/38.43 |
| tcp-frag | 0.00/0.00 | 209.32/274.89 | 47.49/29.90 |
| tcp-obfs | 0.00/0.00 | 175.22/235.47 | 56.06/39.27 |
| tcp-reality | 0.00/0.00 | 191.95/274.05 | 51.89/29.47 |
| tcp-reality-tls | 0.00/0.00 | 252.34/343.67 | 36.64/12.47 |
| udp-faketls | 0.00/0.00 | 220.59/170.91 | 44.67/46.70 |
| udp-padding | 0.00/0.00 | 222.39/132.03 | 44.27/54.29 |
| udp-quic | 0.00/0.00 | 220.59/165.26 | 44.68/48.14 |
| tcp-obfs-awg | 0.00/0.00 | 158.65/247.09 | 60.19/36.22 |
| udp-faketls-awg | 0.00/0.00 | 226.18/134.03 | 42.96/53.37 |

## Resources, cleanup and remaining work

Maximum single-Qeli-process RSS: 128492KiB; maximum window host steal: 0.06%. Complete per-cell CPU/RSS/load/steal and counters retained in evidence. The shared VM limits portability to separate physical hosts and does not establish maximum service capacity.

All owned namespaces/processes removed. Root/archive SHA verified; original INI/logs retained locally, remote temporary files cleaned. Full host gate, production services and staged CLI unchanged. Local qualification: `CURRENT_ALL_12_MODES_THREE_SESSION_DATA_PLANE_POPULATION_QUALIFIED`.

Not pooled with historical F029 CLI35da83fd or the single-sample48window packet. Historical FAIL/host differences and the original endurance performance FAIL remain in their packets. TCP asymmetry cause, exact ACK-delay stage and load limits remain open; no new performance budget PASS or saturation capacity claim. Q36 is not closed. Overall **33/37 (89.2%)**. NO_PUSH_DEPLOY_RELEASE.


[Q36 matched-total load and TUN counters](AUDIT-Q36-MATCHED-LOAD.md):6sessions/60load windows +3sessions/12boundary windows +3paired sessions/12queue-length windows; scoped qualification, full host/archive/cleanup PASS. Q36 IN_PROGRESS,33/37(89.2%).
