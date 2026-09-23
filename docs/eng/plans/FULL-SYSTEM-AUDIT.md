# Full Qeli audit: previous coverage and sequential test plan

<!-- normative-sync: full-system-audit-v1 -->

Inventory date: **22 September 2026**. Baseline: branch `dev`, commit
`fc6f4a5dc8df7f119f2d99a6b72b08916ae7a268`, development **0.8.2**.
The working tree was clean before this documentation. Previous panel and manual/NDP
fixes are committed. This inventory did not modify product code.

The objective is to audit every Qeli system for functional defects, trust-boundary errors,
races, resource/traffic leaks, platform inconsistencies, dead code and documentation drift.
This is an executable plan for a new cycle, not a claim that every system has passed.

## 1. Reconstructed audit history

Sources include this task, relevant tasks titled «аудит» and «ipv6», Qeli project memory,
saved reports, Git and current source structure. Historical notes can refer to removed
implementations. A remembered fix does not close its regression on the current commit.
These records do not establish an independent external cryptographic audit.

| Source | Period and recovered coverage | Evidence and limits |
|---|---|---|
| H01 | 10–12 June: crypto/KDF/replay, handshake/framing, auth/Argon2, users/sessions, Windows storage, DNS/kill switch and INI | [10 June](../archive/audits/AUDIT-2026-06-10.md), [11 June](../archive/audits/AUDIT-2026-06-11.md), [12 June](../archive/audits/AUDIT-2026-06-12.md); historical versions |
| H02 | 18 June–5 July: panel/CSRF, notification SSRF, DNS/DHCP, supervisor/control, NAT cleanup, obfs/WS/AWG and serializers | Qeli memory and [5 July summary](../../archive/audits/AUDIT-FIXES-2026-07-05.md); some historical suspicions were disproved |
| H03 | 11–12 July: hooks/restore/client-save trust boundaries, session teardown, quota/iroute, clients and backups | `project_qeli_audit_2026-07-11.md` memory record; historical fixes/build gates, not fresh E2E |
| H04 | 23–27 July: core/data plane, UDP auth/HOL, pool races, max_clients, NAT tags, routes, parsers, all clients and supply chain | Core/client audit memory from 24/25 July and [27 July summary](../../archive/audits/AUDIT-2026-07-27-FIXES.md); some platform checks were compile-only |
| H05 | 4 August: broad core/client/panel/script audit | `project_qeli_audit_2026-08-04.md`; historical candidate list, not a current defect register |
| H06 | August–September: IPv6/TAP/NetworkPlan/PMTU/DATA_FRAG, roaming/CONTROL_V2 and native/core parity | [IPv6 plan](IPV6-IMPLEMENTATION-PLAN.md), [roaming](ROAMING.md), [transport core](../reference/TRANSPORT-CORE.md), `release/ipv6_lab_matrix_dev.json`; evidence applicability to current SHA must be checked |
| H07 | 26 August–1 September: REALITY/H2 captures, detectability and throughput/CPU/RSS | [DPI](../reports/DPI-AUDIT.md), [comparative benchmark](../reports/benchmarks/vpn_protocol_benchmark_repeat_2026-09-01.md); measurements are for 0.8.0, not 0.8.2 |
| H08 | 16 September: broad A01–A14 audit and fixes | Local `audit-vpn-20260916/AUDIT.md` and `FIX-VERIFICATION.md`; Rust/managed builds, probes and Linux tests, not physical platform E2E |
| H09 | 16–17 September: detailed panel/parser A01–A14 audit | Local `audit-panel-20260916/AUDIT.md`, `panel-fixes-20260917/RESULT.md`; Rust/JS probes, saves, ACL fields, secrets, drafts and INI round trips |
| H10 | 22 September: panel follow-up A15–A22 | Local `audit-panel-20260922/AUDIT.md`, `panel-fixes-20260922/RESULT.md`; restart/preflight, staged restore, share transaction, quoting/dev, notification INI; commit `a8c5050b` |
| H11 | 22 September: IPv6 manual/NDP and documentation | Local `ipv6-manual-20260922/RESULT.md`; commit `fc6f4a5d`; 632 portable Rust tests, 25 JS groups, 16 isolated runtime probes; no real Linux NDP/firewall E2E |

H08–H11 local reports are outside the repository; their names identify the working archive.
Secrets and infrastructure addresses are deliberately absent. **A01–A14 were reused in
separate audits:** every finding reference must include H08 or H09. Counts from different
feature/OS suites must not be added or compared as measures of coverage growth.

## 2. Rules for the new cycle

Each section has separate **historical coverage** and **new-run status**. New statuses:
`TODO`, `IN_PROGRESS`, `PASS`, `FAIL`, `BLOCKED`, `N/A`. `BLOCKED` requires a reason and
required environment; `N/A` requires proof of non-applicability. An unavailable runtime
is not a PASS. A fixed finding closes after verification; unresolved findings stay queued.

Every section follows the same levels:

1. **Structure:** inputs/outputs, state owners, dependencies, trust boundaries and callers;
   OS/feature gates, FFI/reflection/generated code and possible dead branches.
2. **Contract:** positive, boundary and malformed-input cases, defaults and semantic
   preservation; differential/round-trip checks with independent implementations.
3. **Failures:** inject acquire/apply/save failures, partial I/O, concurrency, deadlines,
   cancellation/crash/restart, and verify recovery by the resource owner.
4. **Integration:** real process/API/browser/TUN/firewall/OS adapter, then real devices
   wherever behavior depends on drivers, operating systems or mobile networks.
5. **Regression:** reproduce on the original defect, fix, rerun affected checks, update
   documentation and tie results to exact source SHA/features/target.

Each run records ID/date, commit plus dirty diff/hash, tool versions, OS/arch/features,
command, fixtures/seed, expected/actual results, stdout/stderr, exit code, captures/host
state when relevant, finding ID/severity and limitations. Separate hypotheses from
reproduced bugs; accepted risks need justification. Long runs need deadlines and a stop
procedure. Measure leaks instead of inferring them from UI status.

Before invoking an old lab/E2E script, inspect targets, hardcoded endpoints, cleanup,
credential handling and destructive defaults. A listed test is a **harness-review entry
point**, not proof that it is safe or covers all scenarios. Harness unit tests are not E2E.
Network/destructive scenarios use isolated snapshot-backed labs, not production.

## 3. Environments and mandatory matrix

| Environment | Purpose |
|---|---|
| Local portable | Parser/crypto/protocol/KAT, JS state, Python harness and docs; not Linux server runtime |
| Isolated Linux | Server/CLI/API/backup/systemd/TUN/routes/DNS/NAT/NDP; iptables and nft/firewalld, multiprofile and crash recovery |
| Windows VM | GUI/LocalSystem/Wintun/WinDivert/ACL/DPAPI and real networking/recovery |
| macOS Intel + ARM | launchd/utun/pf/Keychain/Network Extension/per-app; build and runtime separately |
| Android device | Release APK/JNI/VpnService, Wi-Fi/LTE/Doze/always-on/lockdown and sleep/wake |
| iOS device | Signed IPA/PacketTunnel/On Demand/NAT64/per-app/MDM; simulator separately |
| OpenWrt/Keenetic | MIPS/ARM and 32-bit ABI, init/UCI/LuCI, real networks and bounded memory |
| Load lab | Controlled bandwidth/RTT/jitter/loss/reorder/MTU, separate traffic generator and observer |

Derive **supported** transport × obfuscation combinations from source; Quick Start is
not the complete protocol capability list. Unsupported combinations must fail clearly,
not be counted as missing positive tests. For every supported mode: connect/auth,
upload/download, DNS, reconnect and stop/cleanup. Additional axes:

- outer IPv4/IPv6 × inner IPv4/IPv6/dual; IPv6-only uplink/NAT64;
- full/split, TUN/TAP, client-to-client/site-to-site/per-app;
- IPv6 `off/manual/route/nat66` × NDP `off/auto/required`, every transition;
- old/new server and client, incompatible ABI/capabilities, off/prefer/required;
- single/multiple profiles, sessions and paths, empty/exhausted pools;
- stop, timeout, revoke, crash, reload/restart, suspend/resume and network changes.

Execute security-critical combinations exhaustively. Other interactions may use explicit
pairwise coverage, not a claim to cover the whole Cartesian product. Leak checks require
tunnel and physical-interface captures; cleanup needs before/after routes/DNS/firewall/
sysctls/fds/tasks.

## 4. Stage 00 — baseline

**DONE: inventory and limited local checks.** This does not complete the product audit.
The next working section is **01: server INI**, followed by 02–07. Linux E2E for recent
administrative and IPv6 changes remains mandatory.

| Check on `fc6f4a5d` | Result | Limit |
|---|---|---|
| `check_panel.py` | PASS: 11 templates, 1142 RU strings | Static |
| `test_panel_editors.cjs` | PASS: 25 groups | Actual JS components in Node, not browser E2E |
| `unittest discover -s scripts -p 'test_native_*.py'` | PASS: 67 tests | Tooling/contracts, not native core rebuilds |
| `sync_version.py` | PASS: dev/planned 0.8.2, released 0.8.1 | Version consistency |
| `native-libs/provenance.py --check` | **FAIL: STALE NATIVE CORES** | Recorded and current source digests differ |
| `release_certification.py --quiet` | **FAIL: manifest missing** | No `release/certification/0.8.2.json` |

**B00-01:** recorded native digest `85f2f17ed9f58e7ad8d0368b936542f2391961bf0a739c7985a93208d6a1cb80`,
actual `3deb3da9d8306e0eaf4fd3d3505a7ad8f4e822b7bd502e8e748f632a852baa52`.
Packaged cores do not validate current Rust. Close in 22/34 through rebuild and provenance
verification; merely updating recorded digests is insufficient.

**B00-02:** a missing manifest does not prove a runtime defect, but means 0.8.2 release
readiness lacks complete evidence. Close in 34 using real matrix results, never a formal
manifest with unexecuted scenarios marked passed.

## 5. Module register: previous audits and the new pass

The following is the complete reconstructed list in execution order. H references point
to section 1 and denote historical review/tests, not current PASS. Cross-cutting work in
35–37 also applies to every section in 01–34.

| ID | Module | Historical checks | New pass |
|---|---|---|---|
| 01 | Server INI and schema | H01, H04, H08–H10 | IN_PROGRESS |
| 02 | Client parsers and qeli:// | H04, H06, H08–H10 | IN_PROGRESS |
| 03 | Panel UI and state | H02, H09–H11 | TODO |
| 04 | Web auth and API protection | H01–H03, H08–H09 | TODO |
| 05 | Config transactions and restart | H08–H10 | TODO |
| 06 | Users, groups and provisioning | H01, H04, H09–H10 | TODO |
| 07 | Backup, restore and history | H03–H04, H08, H10 | TODO |
| 08 | Cryptography, identity and keys | H01, H04, H08 | TODO |
| 09 | Handshake and TCP/UDP pre-auth | H01, H04, H08 | TODO |
| 10 | PacketCodec, replay and control framing | H01, H04, H08 | TODO |
| 11 | REALITY, TLS 1.3 and HTTP/2 | H07–H08 | TODO |
| 12 | Transports and wire camouflage | H02, H07–H08 | TODO |
| 13 | Recordizer, padding and shaping | H02, H07–H08 | TODO |
| 14 | Supervisor, workers and profiles | H02–H03, H08 | IN_PROGRESS |
| 15 | Sessions, IP pools and limits | H01, H03–H04, H08 | IN_PROGRESS |
| 16 | ACL, pushed routes and site-to-site | H03–H04, H06 | TODO |
| 17 | IPv4 NAT, forwarding and sysctls | H02, H04, H08 | TODO |
| 18 | IPv6 off/manual/route/nat66 and NDP | H06, H11 | TODO |
| 19 | Server and client DNS | H01–H02, H05–H06 | IN_PROGRESS |
| 20 | DHCP and lease lifecycle | H02, H05 | TODO |
| 21 | TUN/TAP, IP, MTU/PMTU and fragmentation | H06, H08 | TODO |
| 22 | Transport core, FFI/JNI and memory | H06, H08 | IN_PROGRESS |
| 23 | Roaming, resume and CONTROL_V2 | H06, H08 | TODO |
| 24 | Multipath, bonding and shared budgets | H04, H06, H08 | TODO |
| 25 | Linux CLI and network recovery | H01, H04, H08 | TODO |
| 26 | Shared C# and managed/native boundary | H04, H06, H08 | TODO |
| 27 | Windows GUI, service and drivers | H01, H04, H08 | TODO |
| 28 | macOS daemon, utun, pf and Network Extension | H04, H08 | TODO |
| 29 | Android VpnService, JNI and lifecycle | H04, H06, H08 | TODO |
| 30 | iOS PacketTunnel, Swift and MDM | H04, H06, H08 | TODO |
| 31 | OpenWrt, LuCI and Keenetic | H04, H06, H08 | TODO |
| 32 | Metrics, usage, logs and notifications | H02–H03, H08, H10 | IN_PROGRESS |
| 33 | Installation, updates, file permissions and hooks | H01, H04, H08 | TODO |
| 34 | CI, dependencies, native provenance and release | H04, H06, H08 | TODO |
| 35 | Fuzzing, concurrency, DoS and soak | H04, H06, H08 | TODO |
| 36 | Benchmarks and measurement methodology | H07 | TODO |
| 37 | Documentation, test harnesses and dead code | H06, H08–H09, H11 | TODO |

## 6. Scenarios for each section

Order: 01 → 37. If a check needs an unavailable environment, mark only that part BLOCKED;
continue independent analysis of the next section while keeping the blocker queued.
Section PASS requires every mandatory level from section 2.

### 01. Server INI and schema

**Source:** `qeli/src/config`.

Trace every key through parse → validate → runtime → serialize; defaults, ranges, duplicates, unknown keys, quotes/TAB/Unicode. Reject invalid input before writing. Configuration is INI-only; internal JSON API remains.

**Existing harness/fixtures:** `qeli/tests/config_examples.rs`.

- [ ] Review and dead code.
- [x] Parser positive, boundary and negative scenarios: 12 new regressions and the existing suite.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [x] First-pass fixes, retesting and evidence (Q01-F001–F007).

**Status: IN_PROGRESS.**

**First pass, 2026-09-22:** [Q01 report](../reports/AUDIT-Q01-SERVER-INI.md).
Fixed 6 INI processing defects and a fixture coverage gap. 651 portable Rust tests,
25 JS groups and the Linux all-targets check passed. Fixture coverage checks 163 key
names and 3 dynamic families. Diagnostic isolation was exercised across 32 parses.

**Open:** complete per-field runtime tracing and broader failures; Linux startup/CLI/
SIGHUP/HTTP save execution is BLOCKED by the missing configured Linux environment.
Compilation does not close this item. The section does not receive overall PASS yet.

### 02. Client parsers and qeli://

**Source:** `qeli/src/config/client.rs`, `qeli/src/config/share.rs`, `conformance`.

Verify the current 81-key Rust/C#/Kotlin/Swift contract; INI ↔ forms ↔ URI, foreign fields and secrets. Test malformed pins/ports/IPv6/MTU and prohibit silent fallback to TOFU.

**Existing harness/fixtures:** `scripts/test_native_config_keys.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: IN_PROGRESS.**

**URI pass, 2026-09-22:** [Q02 report](../reports/AUDIT-Q02-CLIENT-PARSERS.md).
Fixed Q02-F001–F006: ambiguous query parameters, scalar/UTF-8/default behavior and skipped
JVM corpus reruns. 651 Rust tests, 438 C# checks and 137 JVM tests passed.
Shared corpus: 21 valid + 29 reject; the 81-key-name contract is preserved.

**Open:** complete INI/editor pass, Swift build/test and platform integration.
**User's architecture proposal:** [shared configuration module](CLIENT-CONFIG-CORE.md)
inside the existing Rust core is implemented in source (ABI 1.16). Local parsers are removed; generated projections/defaults and shared routing/reconnect/version/route-file policies are active. Apple runtime and release-native A/B rebuilding remain open.

**22 September continuation — configuration boundaries:** eight reproduced INI/URI
scenarios (Q02-F007–F014) were fixed in the core: character/DNS-error loss, invalid overlay names, BOM
validation bypass, secret redaction and misplaced `logging.*` fields. Regressions exercise
client model save paths; details and results are in the
[shared configuration report](CLIENT-CONFIG-CORE.md#configuration-boundary-audit--22-september-2026).
This does not close the full section: Linux E2E and target-device checks remain open.

**Parameters before runtime, 2026-09-22:** Q02-F015–F019 close malformed zero PIN,
invalid host, unsupplied-port repair, old panel validation and automatic dev-insertion defects. Regressions and positive compatibility
cases are recorded in the [Q02 register](../reports/AUDIT-Q02-CLIENT-PARSERS.md).

**Runtime follow-up, 2026-09-22:** the same plan records the shared retry-budget/delay policy,
monotonic established-session timing, finite-limit and offline-recovery fixes, and interruptible
Linux backoff. Rust/C ABI/JNI regressions and Linux cross-Clippy passed. This does not close
real-device/network-change, Apple runtime or release-library gates.

### 03. Panel UI and state

**Source:** `qeli/src/web/templates`, `qeli/src/web/assets`, `qeli/src/web/pages`.

Loading/error/retry, dirty state, delayed replies, concurrent edits, Form/INI, deletion, secret masks, dates and quotas. Real browser: every page, RU/EN, keyboard and mobile layout; failures must not save defaults.

**Existing harness/fixtures:** `scripts/check_panel.py`, `scripts/test_panel_editors.cjs`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 04. Web auth and API protection

**Source:** `qeli/src/web/auth.rs`, `qeli/src/web/mod.rs`, `qeli/src/web/api`.

Inventory routes/guards, Basic/cookie/TOTP, logout/expiry, CSRF, reverse proxies, base_path and allowed_ips. Test concurrent Argon2 budgets/rate limits. Unauthorized requests reveal no secrets and cause no side effects.

**Existing harness/fixtures:** `scripts/check_panel.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 05. Config transactions and restart

**Source:** `qeli/src/web/api/config.rs`, `qeli/src/web/api/control.rs`, `qeli/src/server/preflight.rs`, `qeli/src/util.rs`.

Common validation/preflight for Form/INI/API/history/Quick Start/worker/full restart. Test stale revisions, concurrent writers, ENOSPC/EACCES and snapshot/rename/restart crashes. Failed preflight preserves the running service and valid configuration.

**Existing harness/fixtures:** `scripts/test_web_reload.py`, `scripts/test_panel_editors.cjs`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 06. Users, groups and provisioning

**Source:** `qeli/src/config/users.rs`, `qeli/src/web/api/users.rs`, `qeli/src/web/api/share.rs`, `qeli/src/web/api/identity.rs`.

Inline + users_file, duplicate precedence, missing groups, invalid types versus restriction removal, static addresses, quota/expiry. Identity failure must not change a password during link creation. Revoke applies to existing TCP/UDP sessions.

**Existing harness/fixtures:** `scripts/test_user_reload.py`, `scripts/test_l3_user_limits.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 07. Backup, restore and history

**Source:** `qeli/src/web/api/backup.rs`, `qeli/src/web/api/backup_listing.rs`.

Fresh restore with custom paths, mixed user sources, identity and panel secret. Tar bombs, traversal, links, missing files, overlay/exact, staged compatibility and concurrent restore. Snapshots never recursively archive themselves; prove recovery after interrupted publication.

**Existing harness/fixtures:** `scripts/test_web_reload.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 08. Cryptography, identity and keys

**Source:** `qeli/src/crypto`, `qeli/src/server/reality.rs`, `qeli/src/web/api/identity.rs`.

X25519/ML-KEM/HKDF/AEAD KAT and negative vectors; static binding, proof before credentials, pin/TOFU, RNG, nonce exhaustion, rotation and zeroization. Key ownership/permissions/links/atomic writes. Unit tests do not replace independent cryptanalysis.

**Existing harness/fixtures:** `conformance/hkdf.json`, `conformance/prp-nonce.json`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 09. Handshake and TCP/UDP pre-auth

**Source:** `qeli/src/server/handler.rs`, `qeli/src/server/udp_handler.rs`, `qeli/src/protocol/capabilities.rs`.

Truncation/replay/reorder/slow peers and invalid PQ/proof/password. Permits before spawn, pending caps, anti-amplification, tarpit and cancellation/deadlines. One login must not block UDP reception; failures release resources without unauthorized downgrade.

**Existing harness/fixtures:** `qeli/fuzz/fuzz_targets/clienthello.rs`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 10. PacketCodec, replay and control framing

**Source:** `qeli/src/protocol/packet.rs`, `qeli/src/protocol/ctrl.rs`, `qeli/src/protocol/control_v2.rs`.

Lengths 0/min/max/overflow, AEAD tags, sequences around 2^63/2^64, replay windows and unknown types/generations. Malformed packets must not panic/abort or allocate without bounds; valid traffic works after rejection.

**Existing harness/fixtures:** `conformance/packet-decode.json`, `conformance/replay-window.json`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 11. REALITY, TLS 1.3 and HTTP/2

**Source:** `qeli/src/protocol/realtls`, `qeli/src/protocol/h2_carrier.rs`, `qeli/src/protocol/h2_carrier`.

Transcripts/replay/decoys and TLS key budgets. H2 zero/small windows, SETTINGS/WINDOW_UPDATE/GOAWAY/RST, partial I/O and backpressure. Stop/timeout releases tasks/sockets/permits. PCAP/active probing is separate from tunnel functionality.

**Existing harness/fixtures:** `qeli/src/protocol/h2_carrier/hardening_tests.rs`, `scripts/reality_tls_repeat.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 12. Transports and wire camouflage

**Source:** `qeli/src/protocol/tls.rs`, `qeli/src/protocol/obfs.rs`, `qeli/src/protocol/quic.rs`, `qeli/src/transport`.

Derive supported runtime/Quick Start combinations: plain/fake-tls/reality/reality-tls/WS/obfs/UDP-QUIC/AWG. Test WS masking/control caps, junk counters, fallback and invalid combinations. Separate protocol compliance, DPI detectability and goodput.

**Existing harness/fixtures:** `conformance/quic.json`, `qeli/fuzz/fuzz_targets/websocket_head.rs`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 13. Recordizer, padding and shaping

**Source:** `qeli/src/protocol/recordizer.rs`, `qeli/src/protocol/shaper.rs`, `qeli/src/protocol/obfuscate.rs`.

Off/prefer/required and legacy peers; batch/reassembly caps, flush deadlines, cancellation and aggregate budgets. Junk/heartbeat must not starve payloads. Compare on/off on identical workloads; inspect bounded memory, jitter and periodic PCAP signals.

**Existing harness/fixtures:** `scripts/validate_shaping.py`, `scripts/bench_stealth.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 14. Supervisor, workers and profiles

**Source:** `qeli/src/server/mod.rs`, `qeli/src/server/tasks.rs`, `qeli/src/server/supervisor.rs`, `qeli/src/server/control.rs`, `qeli/src/server/control_io.rs`, `qeli/src/server/control_socket.rs`, `qeli/src/main.rs`, `qeli/src/hooks.rs`, `qeli/src/hooks/process.rs`.

Start/stop/reload/crash/respawn, occupied bind/TUN, profile deletion/rename, hook failures and dead control clients. Lock ordering, backoff, watchdogs and task ownership. Cleanup is idempotent and isolated between profiles.

**Existing harness/fixtures:** `scripts/test_web_reload.py`, `scripts/test_tun_reclaim.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: IN_PROGRESS.**

**Task ownership, 23 September 2026:**
[Q14/Q19 pass](../reports/AUDIT-Q14-Q19-LIFECYCLE.md) fixes Q14-F001–F002: joining
services after early startup errors and a concurrent/cancelled shutdown barrier.
Seven task-ownership tests include a race over 1,024 resources; actual DNS listeners
are exercised over loopback. Linux TUN/firewall E2E, forced wrapper cancellation,
watch/control, hooks and restart/reload remain open.

**Supervisor and control events, 23 September 2026:**
[Q14 follow-up](../reports/AUDIT-Q14-SUPERVISOR.md) fixes Q14-F003–F007: stop during
spawn failures, Child/PID ownership, a 60-second grace deadline, Restart/Reload
queuing and early signal installation. Thirteen behavioral tests plus a child
fixture; 812 Rust tests overall pass. Tests use real isolated host processes,
not the Linux TUN worker. Control sockets, hooks, Unix signals and Linux rollback
remain open.

**Control socket and hooks, 23 September 2026:**
[Q14-F008–F013 pass](../reports/AUDIT-Q14-CONTROL.md): Unix socket ownership,
safe runtime directory permissions, message bounds, deadlines, handler drain before
profile teardown, and post_down only for ready generations. 819 host Rust tests pass;
12 new Unix tests compiled only. Linux runtime/systemd/hooks and forced cancellation
remain open; next are hook processes/output and startup rollback.

**Hook processes, 23 September 2026:**
[Q14-F014–F015](../reports/AUDIT-Q14-HOOKS.md): shared server/client runner retains
8 KiB per stdout/stderr stream and terminates Linux process groups on timeout/cancellation,
while preserving intentional redirected background services. 828 host Rust tests pass;
four new Linux group tests compiled only. Next: startup rollback, background worker
services and binding trusted config to parsed contents. Linux E2E remains open.

**Worker services and accounting, 23 September 2026:**
[Q14-F016–F017 / Q15-F001](../reports/AUDIT-Q14-Q15-WORKER-USAGE.md): owned and monitored
periodic tasks, draining before profile cleanup, writable accounting only after acquiring
the worker lease, final persistence on both stop paths. Short sessions and final counter
tails now survive registry removal. 848 host Rust tests PASS; Linux cross-check only.
Notification tasks, forced outer cancellation and Linux E2E remain open.

**Notification ownership, 23 September 2026:**
[Q14-F018 / Q32-F001](../reports/AUDIT-Q14-Q32-NOTIFICATIONS.md): 128 accepted deliveries,
8 active requests per process, shared panel probe admission, bounded payloads and a
10-second drain after producers stop. No detached notification wrappers remain.
864 host Rust tests PASS; Linux all-targets cross-check only. Supervisor panel/metrics/
autostart ownership, config trust and Linux runtime E2E remain open.

### 15. Sessions, IP pools and limits

**Source:** `qeli/src/server/pool.rs`, `qeli/src/server/handler.rs`, `qeli/src/server/udp_handler.rs`, `qeli/src/server/usage.rs`.

Concurrent allocate/auth/reconnect/evict/reap/revoke/quota, atomic v4+v6, reservations/exclusions and exhaustion. Shared TCP/UDP/bonding caps. No duplicate IP or leaked lease/token/task/client_subnet after any termination path.

**Existing harness/fixtures:** `scripts/test_udp_reap.py`, `scripts/test_maxsessions.py`, `scripts/test_multidevice.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: IN_PROGRESS.**

**Accounting subpass:** [Q15-F001](../reports/AUDIT-Q14-Q15-WORKER-USAGE.md) covers
short TCP/UDP sessions, writer tails, reset baselines and counter retirement. 14 portable
accounting tests pass. Pool allocation, concurrent authentication/reconnect/revoke and
real Linux quota enforcement still need the remaining section-15 scenarios.

### 16. ACL, pushed routes and site-to-site

**Source:** `qeli/src/server/acl.rs`, `qeli/src/config/users.rs`, `qeli/src/transport_core/network.rs`.

User/group/profile precedence, longest prefixes, client_to_client, spoofed sources, overlaps, /0 and client_subnet return paths. Cover TCP/UDP and v4/v6. Server-side enforcement; revoke/route reassignment removes access from the previous session.

**Existing harness/fixtures:** `scripts/test_push_matrix.py`, `scripts/test_route_push.py`, `scripts/test_l3_user_limits.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 17. IPv4 NAT, forwarding and sysctls

**Source:** `qeli/src/server/nat.rs`, `qeli/src/client/sysctl.rs`, `qeli/src/client/gateway.rs`.

NAT44/forward_private/gateway_nat/MSS and iptables/nft backend errors. Before/after rules/routes/sysctls with multiple profiles. Exact tags, ownership, crash journals and boot IDs; preserve administrator rules and values.

**Existing harness/fixtures:** `scripts/test_gateway_nat.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 18. IPv6 off/manual/route/nat66 and NDP

**Source:** `qeli/src/server/nat.rs`, `qeli/src/server/ndp_proxy.rs`, `qeli/src/config/server.rs`.

All 4×3 egress/NDP combinations and every transition for ipv4/dual/ipv6. Linux E2E: off blocks transit; manual adds no IPv6 firewall/DNS/sysctl; route preserves sources; nat66 masquerades. Test RA, exact cleanup, NS validation, session ownership/revoke, required failure, DNS 53/5353 and independent IPv4.

**Existing harness/fixtures:** `scripts/test_panel_ipv6_e2e.py`, `scripts/run_ipv6_release_matrix.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 19. Server and client DNS

**Source:** `qeli/src/server/dns.rs`, `qeli/src/server/dns/resolver.rs`, `qeli/src/client/dns.rs`, `qeli/src/transport_core/network.rs`.

UDP/TCP upstreams, truncation fallback, timeouts, malformed packets, caching/eviction/blocklists. Full/split, resolved/resolv.conf and OS resolvers, v4/v6 leaks, failed apply before Connected and crash recovery. Custom ports/manual IPv6; rejected DoT is not implemented DoT.

**Existing harness/fixtures:** `scripts/test_dns_test_server.py`, `scripts/test_panel_route_dns.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: IN_PROGRESS.**

**Shared network-plan pass, 22–23 September 2026:**
[Q19/Q22 report](../reports/AUDIT-Q19-Q22-NETWORK-PLAN.md). Fixed legacy/v2 DNS differences
and order-dependent route-budget failures; removed obsolete Linux DNS test code.
This validates the shared planner, not the entire module. DNS proxy/cache, actual
OS apply/rollback and concurrent lifecycle checks remain open.

**Server DNS, 23 September 2026:** [Q19 report](../reports/AUDIT-Q19-DNS-PROXY.md).
Fixed Q19-F004–F006: NODATA TTL after CNAME, compressed-name validation and failover
after TCP TC. One production engine is tested locally: 22 DNS tests with UDP/TCP,
754 Rust tests overall — PASS. Linux lifecycle, advanced DNS types and load checks
remain open; the overall section is not complete.


**Cache and signed exchanges, 23 September 2026:**
[Q19 follow-up](../reports/AUDIT-Q19-DNS-CACHE.md) closes Q19-F007–F010: a 16 MiB
per-profile packet budget, byte-preserving uncached TSIG/SIG(0) relay, and request
header/opcode checks. The legacy panel lab script no longer opens SSH on import. 36 DNS tests, 768 Rust tests overall — PASS. Real Linux
lifecycle, sustained concurrent load/RSS, advanced EDNS/RDATA and external signed
interoperability remain open; section 19 stays **IN_PROGRESS**.

**EDNS/RDATA, 23 September 2026:**
[Next Q19 pass](../reports/AUDIT-Q19-DNS-EDNS.md) fixes Q19-F011–F015: TTL across
all replayed sections, common RDATA framing, OPT/TLV validation and BADVERS,
extended RCODE during truncation, and uncached EDNS options with fresh OPT on
ordinary cache hits. 51 DNS tests including IPv6 loopback UDP/TCP; 783 Rust tests
in total — PASS. Linux lifecycle, sustained load/RSS, DNSSEC/RRset semantics,
external interoperability and OS DNS apply/rollback remain open.

**DNS listeners and cleanup, 23 September 2026:**
[Q14/Q19 pass](../reports/AUDIT-Q14-Q19-LIFECYCLE.md): production UDP/TCP listeners
are host-testable and no longer take an unused ServerState. Seven listener plus
52 resolver tests cover IPv4/IPv6, persistent/pipelined TCP, deadlines, the
512-connection limit, cancellation and port rebinding. 798 Rust tests overall
pass. Shared lifecycle fixes are Q14-F001–F002; real Linux runtime and sustained
load remain open.

### 20. DHCP and lease lifecycle

**Source:** `qeli/src/server/dhcp.rs`, `qeli/src/config/server.rs`.

DISCOVER/OFFER/REQUEST/ACK/NAK/RELEASE, invalid requested_ip, duplicate xid/MAC, expiry and malformed options. NAK consumes no lease; empty pools do not underflow. Supported TAP/IPv4 only; verify panel round trips.

**Existing harness/fixtures:** `qeli/tests/config_examples.rs`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 21. TUN/TAP, IP, MTU/PMTU and fragmentation

**Source:** `qeli/src/tun`, `qeli/src/protocol/ip.rs`, `qeli/src/protocol/icmp.rs`, `qeli/src/protocol/data_frag.rs`, `qeli/src/protocol/udp_frag.rs`.

TUN host prefixes, TAP ARP/NDP/RA/DAD and unsupported EtherType/VLAN/multicast. IPv6 MTU 1280, small outer PMTU, spoofed PTB and path changes. Reassembly duplicates/overlaps/gaps/order/expiry/ID reuse and memory caps; inspect unwanted outer fragmentation.

**Existing harness/fixtures:** `scripts/test_tap_ipv6_control_probe.py`, `qeli/fuzz/fuzz_targets/data_frag.rs`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 22. Transport core, FFI/JNI and memory

**Source:** `qeli/src/transport_core`, `qeli/include/qeli_transport_core.h`, `native-libs`.

Create/start/PREPARE/APPLY/COMMIT/stop/free, callbacks, buffers, queues and generation/cancellation. Stale handles, double free, callbacks after disposal, partial failure and ABI/feature mismatches. Packaged cores must match source, not merely load successfully.

**Existing harness/fixtures:** `scripts/test_native_repro.py`, `native-libs/provenance.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: IN_PROGRESS.**

**Shared network-plan pass, 22–23 September 2026:**
[Q19/Q22 report](../reports/AUDIT-Q19-Q22-NETWORK-PLAN.md). Fixed legacy/v2 DNS differences
and order-dependent route-budget failures; removed obsolete Linux DNS test code.
This validates the shared planner, not the entire module. DNS proxy/cache, actual
OS apply/rollback and concurrent lifecycle checks remain open.


### 23. Roaming, resume and CONTROL_V2

**Source:** `qeli/src/protocol/roaming.rs`, `qeli/src/protocol/control_v2.rs`, `qeli/src/transport_core`.

TCP make-before-break/UDP migration: proof/path validation, anti-amplification, grace expiry, replay/revoke and candidate races. NAT rebinding, family changes, Wi-Fi/LTE, sleep/wake, server restart/APPLY rollback and PMTU reset. Verify cross-server boundaries; a PUSH_CONFIG constant is not an implementation.

**Existing harness/fixtures:** `scripts/roaming_tcp_all_modes_netns_e2e.sh`, `scripts/roaming_udp_all_modes_netns_e2e.sh`, `scripts/roaming_mixed_version_netns_e2e.sh`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 24. Multipath, bonding and shared budgets

**Source:** `qeli/src/transport_core/carrier.rs`, `qeli/src/transport_core/session.rs`, `qeli/src/server/handler.rs`.

JOIN proof, stream caps, asymmetric RTT/loss, one/all path failures and ordering/starvation. Bandwidth/quota/buffer caps must not multiply by stream count. Resume/reconnect/stream close preserve valid sessions and release obsolete carriers.

**Existing harness/fixtures:** `scripts/test_multipath_bonding.py`, `scripts/test_multipath_resilience.py`, `scripts/test_multipath_allmodes.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 25. Linux CLI and network recovery

**Source:** `qeli/src/client`, `qeli/src/client_main.rs`, `qeli/src/hooks.rs`.

Endpoint route pinning/same-LAN, full/split, includes/excludes, leak policies/kill switch. Stop/SIGTERM/SIGKILL/reconnect/failed setup: before/after routes/DNS/firewall preserve foreign state. Trusted hooks/password_command, deadlines and honest cleanup status.

**Existing harness/fixtures:** `scripts/test_gateway_nat.py`, `scripts/test_tun_reclaim.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 26. Shared C# and managed/native boundary

**Source:** `qeli-shared/QeliShared`, `qeli-shared/QeliConformance`.

Rust validation parity, import/export/storage and handle/callback lifetimes. Mandatory-fixture conformance and platform selftests. Check OS/features/reflection before deleting legacy codecs; builds/selftests do not replace live client connections.

**Existing harness/fixtures:** `conformance/README.md`, `.github/workflows/ci.yml`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 27. Windows GUI, service and drivers

**Source:** `qeli-win/QeliWin`, `qeli/src/transport_core/wintun.rs`.

LocalSystem IPC/ACL/SIDs, DPAPI, protected directories, atomic service profiles and DLL loading. Windows VM: Wintun/WinDivert/per-app, routes/DNS/firewall, stop during connect, sleep/wake and boot service. Cleanup errors remain visible; prevent UAF/double close.

**Existing harness/fixtures:** `scripts/e2e_windows_native.py`, `scripts/verify_windows_drivers.ps1`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 28. macOS daemon, utun, pf and Network Extension

**Source:** `qeli-mac/QeliMac`, `qeli-mac/per-app`.

Daemon IPC/ownership/modes, Keychain, selected profiles, Intel/ARM ABI and DNS journals. Real Mac: preserve foreign pf/nat/rdr anchors; test per-app entitlements, DNS leaks, reconnect, crashes and sleep/wake. Separate GUI/daemon/Network Extension execution.

**Existing harness/fixtures:** `qeli-mac/README.md`, `.github/workflows/ci.yml`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 29. Android VpnService, JNI and lifecycle

**Source:** `qeli-android/app`.

Protect/TUN retention/generations during reconnect/cancel/stop. Keystore, INI migration, encrypted backups/lost-key recovery; manifest exports/deep links/boot. Device: Wi-Fi/LTE, always-on/lockdown, Doze, process kill, IPv6-only/NAT64 and Release/R8.

**Existing harness/fixtures:** `scripts/roaming_android_sleep_wake_gate.py`, `scripts/roaming_android_udp_grace_expiry_gate.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 30. iOS PacketTunnel, Swift and MDM

**Source:** `qeli-ios/QeliCore`, `qeli-ios/QeliPacketTunnel`, `qeli-ios/QeliIOS`, `qeli-ios/MDM`, `qeli-ios/QeliIOSTests`.

Exactly-once start/stop completion, generation/cancellation, Keychain/app groups and extension memory. Device: On Demand, sleep/wake, captive portals, NAT64/DNS, per-app/MDM and settings rollback. Simulator builds, signed IPA and physical evidence are distinct.

**Existing harness/fixtures:** `qeli-ios/PARITY.md`, `scripts/test_verify_ios_ipa.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 31. OpenWrt, LuCI and Keenetic

**Source:** `qeli-openwrt`, `scripts/build_keenetic.py`.

UCI → INI escaping/shell injection, LuCI ACLs, flash secrets, init/procd and upgrade/rollback. ARM/MIPS/mipsel, endian/32-bit ABI and client-only features. Real router: WAN renewal/reboot, DNS/firewall/hooks and memory/throughput; cross-build is not device testing.

**Existing harness/fixtures:** `scripts/keenetic_verify.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 32. Metrics, usage, logs and notifications

**Source:** `qeli/src/server/metrics.rs`, `qeli/src/server/usage.rs`, `qeli/src/server/notify.rs`, `qeli/src/server/roaming_metrics.rs`, `qeli/src/trace.rs`, `qeli/src/web/api/logs.rs`.

Counters/quota/session accounting across reconnect/reap/crash, corrupt stores and bounded logs/SSE/backpressure. Notification INI/tokens/load races, SSRF/DNS rebinding/redirects/deadlines/rate limits. Logs reveal no credentials; disabled tracing leaves the hot path unchanged.

**Existing harness/fixtures:** `scripts/test_roaming_control_stats.py`, `scripts/test_blocked_settings.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: IN_PROGRESS.**

**Notification ownership, 23 September 2026:**
[Q14-F018 / Q32-F001](../reports/AUDIT-Q14-Q32-NOTIFICATIONS.md): 128 accepted deliveries,
8 active requests per process, shared panel probe admission, bounded payloads and a
10-second drain after producers stop. No detached notification wrappers remain.
864 host Rust tests PASS; Linux all-targets cross-check only. Supervisor panel/metrics/
autostart ownership, config trust and Linux runtime E2E remain open.

### 33. Installation, updates, file permissions and hooks

**Source:** `qeli/debian`, `qeli/src/server/update.rs`, `qeli/src/util.rs`, `qeli/src/hooks.rs`, `release/docker`.

Install/upgrade/downgrade/remove, systemd sandbox, identity/user preservation, checksums/attestation and atomic replacement. Docker digest/recreation/health/rollback. File locks/links/owners/ENOSPC, PATH hijacking, panel/restore command injection and SSH deadlines.

**Existing harness/fixtures:** `scripts/test_ssh_run.ps1`, `scripts/release_preflight.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 34. CI, dependencies, native provenance and release

**Source:** `qeli/Cargo.toml`, `qeli/Cargo.lock`, `.github/workflows`, `native-libs`, `release/certification`.

Feature/debug/release/jemalloc matrices, lockfiles, current advisories/licenses and pinned Actions/SDKs. Independent A/B core rebuilds, hashes/ABI/provenance and driver/APK/IPA signatures. Certification requires real evidence tied to the source SHA, never edited digest/status substitutes.

**Existing harness/fixtures:** `scripts/test_native_recipes.py`, `scripts/test_native_repro.py`, `scripts/test_release_certification.py`, `scripts/release_certification.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 35. Fuzzing, concurrency, DoS and soak

**Source:** `qeli/fuzz`, `scripts/stability_gate.py`.

Fuzz INI/hello/packet/WS/realtls/QUIC/IP/fragments/roaming with retained corpora. Inject acquire/apply/save failures and stop/auth/reload/reap races. Run 30–60 minute smoke and ≥8-hour release soaks measuring RSS/fds/tasks/leases/rules with predefined growth thresholds.

**Existing harness/fixtures:** `qeli/fuzz/README.md`, `scripts/roaming_udp_resource_soak_netns_gate.sh`, `scripts/linux_roaming_release_soak.sh`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 36. Benchmarks and measurement methodology

**Source:** `scripts/benchmark.py`, `qeli/src/packet_bench_main.rs`, `test`, `release/benchmark_results.json`.

Pin source/binaries, CPU/governor/affinity/VM contention, MTU/modes and background load. P=1/P=4, up/down/bidirectional, inner/outer v4/v6, TCP/UDP goodput, latency percentiles, loss/jitter, CPU/RSS/auth rate. ≥3 independent runs, median/spread/raw results; 0.8.0 numbers do not represent current development.

**Existing harness/fixtures:** `scripts/perf_combined_load.py`, `scripts/bench_bonding.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

### 37. Documentation, test harnesses and dead code

**Source:** `docs`, `qeli/config`, `scripts`, `conformance`, `site`.

Compare keys/defaults/errors with runtime, execute complete examples, verify RU/EN, stable/dev and ABI/benchmark dates. Inspect legacy JSON config, unused dependencies/helpers/routes/flags and disconnected tests. Prove dead code across OS/features/FFI/reflection/generators; regressions must fail on the original defect.

**Existing harness/fixtures:** `scripts/check_docs.py`, `scripts/check_panel.py`, `scripts/test_site_docs.js`, `scripts/sync_version.py`.

- [ ] Review and dead code.
- [ ] Positive, boundary and negative scenarios.
- [ ] Failures and concurrency.
- [ ] Integration and target platform.
- [ ] Fixes, retesting and evidence.

**Status: TODO.**

## 7. Baseline and subsequent commands

Run from the checkout root using the project toolchain; retain one log per invocation.
These commands do not imply permission to run arbitrary network/deployment scripts
mentioned earlier.

```text
python scripts/check_docs.py
python scripts/check_panel.py
node scripts/test_panel_editors.cjs
node scripts/test_site_docs.js
python -m unittest discover -s scripts -p "test_native_*.py"
python scripts/sync_version.py
python native-libs/provenance.py --check
python scripts/release_certification.py --quiet
```

Run the complete server suite **on Linux**. The same command on Windows has different
cfg coverage. Record the exact target and features:

```text
cargo test --locked --manifest-path qeli/Cargo.toml --workspace -- --test-threads=1
cargo clippy --locked --manifest-path qeli/Cargo.toml --all-targets -- -D warnings
cargo test --locked --manifest-path qeli/Cargo.toml --features transport-core-ffi transport_core -- --test-threads=1
cargo run --locked --manifest-path qeli/Cargo.toml --features conformance-gen --bin gen-conformance -- --check
```

Use current [CI](../../../.github/workflows/ci.yml) commands for Windows/macOS/Android/iOS
builds/selftests, preserving environment and fixture guards. Do not automatically carry
forward old Clippy exceptions: record the toolchain and justify each exception. A cross
check is not Linux runtime execution.

## 8. Closing sections and the full cycle

Name new findings `Q<section>-F<number>`, for example `Q01-F001`, avoiding reused
historical A-identifiers. Record impact, preconditions, reachable path, reproduction,
fix and regression evidence. P0/P1 block the affected release until fixed or explicitly
resolved; P2/P3 remain concrete tracked tasks despite successful builds.

After each section, update its register row, attach results and identify the next section.
A changed contract reopens regression checks for its consumers. Final PASS requires all
mandatory sections closed, resolved blockers, justified N/A cases, matching native/source
SHA, physical scenario evidence, reproducible benchmarks and accurate support limits.

**Next work:** section 01, then 02–07; retain the Linux restart/restore/manual+NDP E2E
and platform-certification backlog. Run the full new benchmark after fixes stabilize;
performance results do not replace a functional audit.
