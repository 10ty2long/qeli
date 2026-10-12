# Q35 — concurrent extended TCP and UDP roaming soak

<!-- normative-sync: audit-q35-long-soak-v1 -->

On 10 October 2026, lab .11 ran two real VPN scenarios concurrently: TCP fake-TLS and UDP QUIC, with **900 route switches each**. Both finished without failures: respectively 16 and 17 scenario assertions passed. Total elapsed time was **1850.9 seconds**; 368 complete process snapshots show **1835.9 seconds** of simultaneous live VPN activity, exceeding the predeclared 1800-second minimum.

The scenarios ran in distinct network and PID namespaces inside an isolated environment, with separate directories and control sockets. Their checkpoints verified the working tunnel and route; TCP left no orphan sessions, while UDP left no candidates and retained three CID aliases. The collector sampled each transport's client and worker every five seconds, including stable process/config identity, RSS, threads, file descriptors and sockets. Maximum growth was **12,584 KiB RSS** (UDP client), **11 descriptors**, **2 sockets** and **3 threads**, within the declared 32-MiB, 16, 8 and 8 peak limits. Final descriptor growth was at most 4, socket growth at most 1, and thread growth at most 1. The scenarios' built-in resource checks also passed.

The run reused the previously qualified binary SHA256 `98335046d0c16b2126b22334102011ecfaff8390e0a3b09be5e6c7df87104bc2` and input archive SHA256 `89405cacf3923c8c8b4caccae701b0e6650d694250630cccd0e9a0034e977617`, overlaying seven current scripts with individual SHA checks. Production Qeli source has not changed since that qualification; no new Rust build is claimed here. Complete before/after host snapshots were retained and matched structurally after normalizing only packet and byte counters in built-in firewall chains. The service retained PIDs 777/845 and `/usr/bin/qeli` SHA. All 67 regular files from the test root were backed up with SHA verification; its owned directory was removed after process, descriptor and mount ownership checks. The initial CRLF `outer.sh` harness failure is preserved separately; it was fixed before VPN execution.

This qualifies 900 cycles and 30 minutes for these two transports, not the full 10,000/1,000-cycle release endurance, an eight-hour stress test, other adapters, or complete DoS coverage. Q35 remains `IN_PROGRESS`; the overall ledger is **30/37 DONE (81.1%)**, with seven sections open. Raw logs, samples, thresholds and cleanup proof reside in the local, out-of-Git packet `audit-debt-20260924/q35-long-soak-20261010`. `LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE`.

## Release endurance on 11–12 October: resource phases complete, performance gate FAIL

<!-- normative-sync: audit-q35-release-endurance-failed-perf-v1 -->

The next isolated run on lab .11 lasted **46,992.3 seconds (13 h 3 min)**.
It completed 10,000 TCP switches, 10,000 UDP QUIC switches, and 1,000 each for
UDP fake-TLS, obfs and obfs-awg. The five resource scenarios independently
verified 4,014 / 4,013 / 402 / 401 / 400 snapshots of stable client and worker
processes. RSS, threads, descriptors and sockets stayed within the predeclared
limits; maximum built-in RSS growth was 18,984 KiB against a 32-MiB budget.
Every archived file was SHA-verified, and the owned root and remote archive
were removed. Structural host state and production/staged binaries were preserved.

The overall release gate returned **FAIL** on phase seven of eight.
Negotiated TCP roaming produced a median upload rate of **475 Mbps versus
510 Mbps** for the roaming-off baseline: **6.8627%** lower against the unchanged
**5%** budget. Three upload results were baseline 510 / 513 / 445 and
roaming-required 475 / 487 / 409 Mbps. Download (597 versus 587 Mbps) and
combined CPU (159.962% versus 162.747%) passed their comparisons.
The cause of the upload decrease is not established. The budget was not changed,
and the failure remains preserved. The final cross-node phase did not run.
The completed resource scenarios do not need replay; neither full release PASS
nor Q35 closure is claimed.

The run used the previous qualified CLI SHA-256
`35da83fd0a727a31a93c4527f8c814777aabcc7ef48dd92e0a0de65f5cbc7050`
with this job's pinned inputs. It is not the new lazy wire-pool candidate and
does not establish whole-current-HEAD source equivalence. Raw results,
independent completed-prefix qualification, the original FAIL and cleanup
proof remain in the out-of-Git packet `audit-debt-20260924/q35-release-endurance-20261011`.
Q35 remains `IN_PROGRESS`; `NO_PUSH_DEPLOY_RELEASE`.
