# Q34/Q25 — memory across 100 release handovers

<!-- normative-sync: audit-q34-release-soak-v1 -->

24 September 2026. Rust 1.97.0, Linux x86_64, `release --features jemalloc`;
core sources match `ea87fd49`. This is developer integration, not final 0.8.2
certification, a throughput benchmark or A/B reproducibility.

## Measurements

100 TCP fake-tls and 100 UDP quic-shape handovers, sampled every 10 switches.
Both scenarios: **15/15 assertions PASS**, **30/30** total; exit codes `0/0`.
One authenticated session, original processes and TUN, connectivity, exactly
100 client/server COMMITs and one exact carrier bypass were preserved.

| Transport / process | RSS baseline → final, KiB | Final / sampled peak growth, KiB | fd baseline → final | socket fd baseline → final |
|---|---:|---:|---:|---:|
| TCP client | 47736 → 49496 | 1760 / 1904 | 15 → 15 | 5 → 5 |
| TCP server | 53644 → 56036 | 2392 / 2392 | 18 → 18 | 6 → 6 |
| UDP client | 49592 → 50292 | 700 / 3764 | 15 → 16 | 5 → 6 |
| UDP server | 61668 → 74248 | 12580 / 12580 | 18 → 18 | 6 → 6 |

UDP retains one additional client socket, stable from the first sample to the end;
this is not zero growth. Pending candidate=0 and CID aliases=3 after handover;
TCP orphan=0. The existing **32768 KiB RSS growth criterion is unchanged**.
Scripts now print baseline, final and sampled peak deltas, retaining the inputs.
Bash syntax and 6 Python harness tests PASS.

The [previous debug FAIL](AUDIT-Q34-LINUX-MATRIX.md) remains in the evidence.
Debug and this release differ in allocator/build profile and source revision;
this result does not isolate the cause of the earlier growth or turn that result
into PASS. It confirms the criterion for this measured release executable.

## Reproducibility and remaining scope

Worker SHA256: `93ee5dce2fc0e1066d7a1c5943b02dfaca608c1bee67323cabd6e9b52dc3b027`.
Source archive: `4ac2dbae65e440b872ff4a15222e12e60927c238d955ce02872fd448ff12d3a7`
(320 files; includes gateway fix, excludes the subsequent route deadline fix).
Lab `.11`, private net/mount/PID namespaces; server `.10` unchanged.

Evidence: `C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/`:
`release-gateway.log`, `roaming-release-soak-100/` with TCP/UDP logs, rc, executed
scripts and worker SHA; `packet-matrix-phase/roaming-scripts-measured-sha256.json`,
`harness-checks/` and `release-soak-phase/evidence.json`.

D13 stays IN_PROGRESS: stop/reconnect, failure/multiprofile cases, before/after
threads/tasks, routes/firewall/journals and final-source measurements remain.
D11/D12/D14 are also open. Windows VM/Mac/iOS/router runtime is skipped by user
decision. [Register](../plans/AUDIT-DEBT.md).
