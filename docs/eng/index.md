# Qeli documentation — map

> Development performance measurements for unreleased 0.8.2 have been withdrawn from publication. Complete original data and reports are retained locally for internal audit. They are not a public release benchmark or a basis for comparing version performance.


The documentation is organized by **document type**. The same directory layout is used for
English and Russian, and every active page is linked from this map.

> New here? Start with **[Getting started](manuals/GETTING-STARTED.md)**, then
> **[Configuration](manuals/CONFIG.md)**. If something does not work, open
> **[Troubleshooting](manuals/TROUBLESHOOTING.md)**.

**Русская версия → [../ru/index.md](../ru/index.md)**

## Overview

| Document | What it covers |
|---|---|
| [README.md](README.md) | Project overview: purpose, wire modes, crypto stack and repository layout |

## Manuals (`manuals/`)

Practical installation, configuration and operations guides.

| Document | What it covers |
|---|---|
| [GETTING-STARTED.md](manuals/GETTING-STARTED.md) | Installation and first run, step by step |
| [CONFIG.md](manuals/CONFIG.md) | Complete flat-INI server and client configuration reference |
| [OPERATIONS.md](manuals/OPERATIONS.md) | Compatibility, upgrades, rollback, backup and firewall operations |
| [PANEL.md](manuals/PANEL.md) | Web panel installation and use |
| [IPV6.md](manuals/IPV6.md) | IPv4/IPv6/dual-stack setup, `off/manual/route/nat66`, NDP proxy and diagnostics |
| [OBFUSCATION.md](manuals/OBFUSCATION.md) | Recordizer setup, masking-layer compatibility and tuning profiles |
| [TROUBLESHOOTING.md](manuals/TROUBLESHOOTING.md) | Connection diagnostics and error reference |
| [KEENETIC-DEPLOY.md](manuals/KEENETIC-DEPLOY.md) | Step-by-step client deployment on Keenetic |

## Reference (`reference/`)

Technical contracts and architecture that describe the current implementation.

| Document | What it covers |
|---|---|
| [CLIENT-CONFIG-MATRIX.md](reference/CLIENT-CONFIG-MATRIX.md) | Current client-key contract by platform and migration history |
| [THREAT-MODEL.md](reference/THREAT-MODEL.md) | Threat model, trust boundaries and assurance status |
| [TRANSPORT-CORE.md](reference/TRANSPORT-CORE.md) | Shared Rust transport core, source/ABI contract and release gates |
| [KEENETIC-PORT.md](reference/KEENETIC-PORT.md) | Keenetic port architecture and dual-arch build rationale |

## Plans (`plans/`)

Active development direction and implementation plans. These are not end-user instructions.

| Document | What it covers |
|---|---|
| [ROADMAP.md](plans/ROADMAP.md) | Product and engineering roadmap |
| [ROAMING.md](plans/ROAMING.md) | Normative client-roaming implementation plan |
| [IPV6-IMPLEMENTATION-PLAN.md](plans/IPV6-IMPLEMENTATION-PLAN.md) | IPv6 architecture, stages and release gates |
| [CLIENT-CONFIG-CORE.md](plans/CLIENT-CONFIG-CORE.md) | Shared Rust INI/URI APIs, removed client duplicates and remaining checks |
| [FULL-SYSTEM-AUDIT.md](plans/FULL-SYSTEM-AUDIT.md) | Audit history, 37-section test plan, environments and progress |
| [AUDIT-DEBT.md](plans/AUDIT-DEBT.md) | Required fixes and verification before starting new audit sections |

## Reports (`reports/`)

[Post-audit 0.8.2 documentation review — 16 October](reports/audit-0.8.2/DOCUMENTATION-REVIEW.md)

[Release 0.8.2 audit: Q01–Q37 reports](reports/audit-0.8.2/README.md)

Current analyses and measured results. Dated, frozen reports live in the archive.

| Document | What it covers |
|---|---|
| [AUDIT-Q02-CLIENT-PARSERS.md](reports/audit-0.8.2/AUDIT-Q02-CLIENT-PARSERS.md) | INI/URI and editors: 19 findings, common corpus, Rust/C#/Kotlin tests and Swift limits |
| [AUDIT-Q03-PANEL-STATE.md](reports/audit-0.8.2/AUDIT-Q03-PANEL-STATE.md) | Panel policy/defaults loading, real browser RU/EN and qualification limits |
| [AUDIT-Q19-Q22-NETWORK-PLAN.md](reports/audit-0.8.2/AUDIT-Q19-Q22-NETWORK-PLAN.md) | Shared DNS plan, legacy/v2 and CIDR exclusions: fixes, regressions and verification limits |
| [AUDIT-Q19-DNS-PROXY.md](reports/audit-0.8.2/AUDIT-Q19-DNS-PROXY.md) | Server DNS: CNAME/NODATA, compressed names, TCP failover and loopback network tests |
| [AUDIT-Q19-DNS-CACHE.md](reports/audit-0.8.2/AUDIT-Q19-DNS-CACHE.md) | DNS cache byte budget, TSIG/SIG(0) relay and request validation |
| [AUDIT-Q19-DNS-EDNS.md](reports/audit-0.8.2/AUDIT-Q19-DNS-EDNS.md) | EDNS, record validation, whole-message TTLs and IPv6 upstream tests |
| [AUDIT-Q14-HOOKS.md](reports/audit-0.8.2/AUDIT-Q14-HOOKS.md) | Bounded stdout/stderr, timeout/cancellation and lifecycle hook descendants |
| [AUDIT-Q14-Q15-WORKER-USAGE.md](reports/audit-0.8.2/AUDIT-Q14-Q15-WORKER-USAGE.md) | Worker service ownership, shutdown persistence and short-session accounting |
| [AUDIT-Q14-Q32-NOTIFICATIONS.md](reports/audit-0.8.2/AUDIT-Q14-Q32-NOTIFICATIONS.md) | Bounded notification delivery, panel probes and shutdown |
| [AUDIT-Q14-Q33-CONFIG-TRUST.md](reports/audit-0.8.2/AUDIT-Q14-Q33-CONFIG-TRUST.md) | Config snapshot trust, file races and ready-generation cleanup |
| [AUDIT-Q25-CREDENTIAL-COMMANDS.md](reports/audit-0.8.2/AUDIT-Q25-CREDENTIAL-COMMANDS.md) | Bounded password_command, secret-safe errors, early stop and feature isolation |
| [AUDIT-Q25-PASSWORD-FILES.md](reports/audit-0.8.2/AUDIT-Q25-PASSWORD-FILES.md) | Bounded password files, shared secret buffer and final client status |
| [AUDIT-Q25-NETWORK-CLEANUP.md](reports/audit-0.8.2/AUDIT-Q25-NETWORK-CLEANUP.md) | Keep the kill-switch after failed forwarding cleanup |
| [AUDIT-Q25-CORE-LIFECYCLE.md](reports/audit-0.8.2/AUDIT-Q25-CORE-LIFECYCLE.md) | Core startup/teardown failures, terminal hooks and kill-switch retention |
| [AUDIT-Q25-DNS-RECOVERY.md](reports/audit-0.8.2/AUDIT-Q25-DNS-RECOVERY.md) | Legacy DNS recovery: checked operations and retained snapshots |
| [AUDIT-Q25-TUN-CLEANUP.md](reports/audit-0.8.2/AUDIT-Q25-TUN-CLEANUP.md) | TUN/DNS/route cleanup errors, guarded plan handoff and terminal kick preservation |
| [AUDIT-Q14-OWNED-SHUTDOWN.md](reports/audit-0.8.2/AUDIT-Q14-OWNED-SHUTDOWN.md) | Final DNS/IPv6 sysctl lease checks and worker/supervisor error propagation; partial Q14-F027 fix |
| [AUDIT-Q14-RETAINED-CLEANUP.md](reports/audit-0.8.2/AUDIT-Q14-RETAINED-CLEANUP.md) | Exact NAT rules, retired-generation failures and verified Linux lifecycle |
| [AUDIT-Q14-PROFILE-SHUTDOWN.md](reports/audit-0.8.2/AUDIT-Q14-PROFILE-SHUTDOWN.md) | Profile task failures, TUN queue timeout/panic and TUN deletion errors in shutdown outcome; partial Q14-F027 |
| [AUDIT-Q36-UDP-SHUTDOWN.md](reports/audit-0.8.2/AUDIT-Q36-UDP-SHUTDOWN.md) | UDP shutdown: regressions PASS, full AUTH runtime OPEN |
| [AUDIT-Q14-SYSCTL-RECOVERY.md](reports/audit-0.8.2/AUDIT-Q14-SYSCTL-RECOVERY.md) | Q14-F029/F030: sysctl recovery failures and preserving prior ownership on repeated acquisition |
| [AUDIT-Q14-IPV6-PARTIAL-ACQUIRE.md](reports/audit-0.8.2/AUDIT-Q14-IPV6-PARTIAL-ACQUIRE.md) | Q14-F031: track partial IPv6 acquisition and retry rollback until release succeeds |
| [AUDIT-Q05-PREFLIGHT.md](reports/audit-0.8.2/AUDIT-Q05-PREFLIGHT.md) | Q05-F001: bounded preflight commands and partial IPv4/IPv6 snapshot policy |
| [AUDIT-Q05-PANEL-TRANSACTIONS.md](reports/audit-0.8.2/AUDIT-Q05-PANEL-TRANSACTIONS.md) | Async preflight, bounded config lock waits and backup/restore cancellation |
| [AUDIT-Q05-ARCHIVE-BUDGET.md](reports/audit-0.8.2/AUDIT-Q05-ARCHIVE-BUDGET.md) | Backup/restore budget, bounded stdin and complete rollback snapshots |
| [AUDIT-Q05-HEALTH-PROBES.md](reports/audit-0.8.2/AUDIT-Q05-HEALTH-PROBES.md) | Async Status/Transport health probes, shared admission and HTTP-router responsiveness |
| [AUDIT-Q14-NAT-COMMANDS.md](reports/audit-0.8.2/AUDIT-Q14-NAT-COMMANDS.md) | Q14-F032: server NAT shares the bounded command runner; ownership after timeout |
| [AUDIT-Q14-NAT-CLEANUP-BUDGET.md](reports/audit-0.8.2/AUDIT-Q14-NAT-CLEANUP-BUDGET.md) | Shared NAT/DNS cleanup deadline and retained unverified rules |
| [AUDIT-Q14-DNS-INPUT-BUDGET.md](reports/audit-0.8.2/AUDIT-Q14-DNS-INPUT-BUDGET.md) | DNS INPUT lease deadlines and retirement without lock waits |
| [AUDIT-Q14-NAT-SETUP-BUDGET.md](reports/audit-0.8.2/AUDIT-Q14-NAT-SETUP-BUDGET.md) | NAT/forwarding setup and exact rollback deadlines |
| [AUDIT-Q14-DNS-OWNERSHIP.md](reports/audit-0.8.2/AUDIT-Q14-DNS-OWNERSHIP.md) | Retained DNS rule specifications after cleanup/rollback failure, retries and generation identity |
| [AUDIT-Q14-Q25-FIREWALL-CHECKS.md](reports/audit-0.8.2/AUDIT-Q14-Q25-FIREWALL-CHECKS.md) | Shared server/client firewall checks, exact DNS cleanup and the 1024-rule boundary |
| [AUDIT-Q14-NAT-CLEANUP.md](reports/audit-0.8.2/AUDIT-Q14-NAT-CLEANUP.md) | Finite NAT cleanup, verification, diagnostics and open teardown findings |
| [AUDIT-Q25-ROUTE-OWNERSHIP.md](reports/audit-0.8.2/AUDIT-Q25-ROUTE-OWNERSHIP.md) | Q25-F025/F026: preserve changed routes and verify cleanup before forgetting ownership |
| [AUDIT-Q25-TUNNEL-ROUTES.md](reports/audit-0.8.2/AUDIT-Q25-TUNNEL-ROUTES.md) | Q25-F037–F039: shared TUN/TAP installer, strict route_local and obsolete parser removal |
| [AUDIT-Q25-GATEWAY-ROLLBACK.md](reports/audit-0.8.2/AUDIT-Q25-GATEWAY-ROLLBACK.md) | Q25-F040–F042: gateway ownership, partial rollback and kill-switch inspection |
| [AUDIT-Q25-GATEWAY-BUDGET.md](reports/audit-0.8.2/AUDIT-Q25-GATEWAY-BUDGET.md) | Gateway/exit-node deadline and retained rollback |
| [AUDIT-Q25-ROUTE-BUDGET.md](reports/audit-0.8.2/AUDIT-Q25-ROUTE-BUDGET.md) | Shared route transaction deadline and separate verified rollback |
| [AUDIT-Q25-EXIT-OWNERSHIP.md](reports/audit-0.8.2/AUDIT-Q25-EXIT-OWNERSHIP.md) | Q25-F043–F045: independent exit NAT and conflicting kill-switch admission |
| [AUDIT-Q25-KILL-SWITCH-LIFETIME.md](reports/audit-0.8.2/AUDIT-Q25-KILL-SWITCH-LIFETIME.md) | Q25-F046–F047: Linux kill-switch lifetime lease and fail-closed IPv6 |
| [AUDIT-Q25-CLIENT-NAMESPACE.md](reports/audit-0.8.2/AUDIT-Q25-CLIENT-NAMESPACE.md) | Q25-F048–F049: disabled IPv6 and shared client TUN reservations |
| [AUDIT-Q25-SYSCTL-OWNER-EVIDENCE.md](reports/audit-0.8.2/AUDIT-Q25-SYSCTL-OWNER-EVIDENCE.md) | Q25-F050–F051: sysctl owner evidence and preserving incomplete recovery |
| [AUDIT-Q25-SYSCTL-JOURNAL-IO.md](reports/audit-0.8.2/AUDIT-Q25-SYSCTL-JOURNAL-IO.md) | Bounded journal reads from one fd, FIFO locks and sysctl lock contention deadline |
| [AUDIT-Q25-SYSCTL-CONTEXT-IO.md](reports/audit-0.8.2/AUDIT-Q25-SYSCTL-CONTEXT-IO.md) | Namespace checks around PID/sysctl I/O and persistence; retained evidence after context loss |
| [AUDIT-Q25-ATOMIC-STATE.md](reports/audit-0.8.2/AUDIT-Q25-ATOMIC-STATE.md) | Temporary-file cleanup, directory fsync and uncertain publication outcomes |
| [AUDIT-Q25-STATE-DIRECTORY.md](reports/audit-0.8.2/AUDIT-Q25-STATE-DIRECTORY.md) | Pinned trusted sysctl directory, root/User=qeli and shared-lock replacement during waiting |
| [AUDIT-Q25-NAMESPACE-PIN.md](reports/audit-0.8.2/AUDIT-Q25-NAMESPACE-PIN.md) | Network/PID/time namespace fd lifetime across lock waits and sysctl I/O |
| [AUDIT-Q25-NAMESPACE-GENERATION.md](reports/audit-0.8.2/AUDIT-Q25-NAMESPACE-GENERATION.md) | Network namespace generation, journal v4 and D02 closure |
| [AUDIT-Q25-SYSCTL-TARGET.md](reports/audit-0.8.2/AUDIT-Q25-SYSCTL-TARGET.md) | Original sysctl fd, rename/replacement/crash refusal and journal v3 migration |
| [AUDIT-Q25-DNS-BUDGET.md](reports/audit-0.8.2/AUDIT-Q25-DNS-BUDGET.md) | Shared DNS application deadline and retained lease for separate rollback |
| [AUDIT-Q25-RESOLVER-CONFIG.md](reports/audit-0.8.2/AUDIT-Q25-RESOLVER-CONFIG.md) | Strict resolver configuration and bounded file reading |
| [AUDIT-Q25-RESOLVER-CONTEXT.md](reports/audit-0.8.2/AUDIT-Q25-RESOLVER-CONTEXT.md) | D-Bus/resolved context, direct calls and DNS ports |
| [AUDIT-Q34-ANDROID-RUNTIME.md](reports/audit-0.8.2/AUDIT-Q34-ANDROID-RUNTIME.md) | Fresh Android JNI, cargo-ndk fixes and 154 JVM + 6 emulator tests |
| [AUDIT-Q34-LINUX-MATRIX.md](reports/audit-0.8.2/AUDIT-Q34-LINUX-MATRIX.md) | Linux packet matrix and RSS soak |
| [AUDIT-Q34-RELEASE-SOAK.md](reports/audit-0.8.2/AUDIT-Q34-RELEASE-SOAK.md) | Release TCP/UDP memory across 100 handovers |
| [AUDIT-Q35-FRAGMENT-FUZZ.md](reports/audit-0.8.2/AUDIT-Q35-FRAGMENT-FUZZ.md) | ASan fuzz oracles for UDP fragmentation and limits |
| [AUDIT-Q35-LONG-SOAK.md](reports/audit-0.8.2/AUDIT-Q35-LONG-SOAK.md) | Concurrent 30-minute TCP/UDP roaming soak with 900 switches each |
| [AUDIT-Q35-CLIENT-CONFIG-FUZZ.md](reports/audit-0.8.2/AUDIT-Q35-CLIENT-CONFIG-FUZZ.md) | ASan fuzz oracles for client INI and qeli:// canonical reparse |
| [AUDIT-Q35-RELEASE-ADMISSION.md](reports/audit-0.8.2/AUDIT-Q35-RELEASE-ADMISSION.md) | Release-soak false-PASS refusal and eight-hour minimum |
| [AUDIT-Q25-KILL-SWITCH-IDENTITY.md](reports/audit-0.8.2/AUDIT-Q25-KILL-SWITCH-IDENTITY.md) | Namespace ownership, reconnect protection and exact family cleanup |
| [AUDIT-Q25-KILL-SWITCH-BUDGET.md](reports/audit-0.8.2/AUDIT-Q25-KILL-SWITCH-BUDGET.md) | Shared kill-switch cleanup deadline, partial cleanup and verified retry |
| [AUDIT-Q25-KILL-SWITCH-REFRESH.md](reports/audit-0.8.2/AUDIT-Q25-KILL-SWITCH-REFRESH.md) | Shared refresh deadline and refusing insertion after unknown inspection |
| [AUDIT-Q25-KILL-SWITCH-SETUP.md](reports/audit-0.8.2/AUDIT-Q25-KILL-SWITCH-SETUP.md) | Shared kill-switch setup deadline and verified rollback |
| [AUDIT-Q25-LINK-OBSERVATION.md](reports/audit-0.8.2/AUDIT-Q25-LINK-OBSERVATION.md) | Shared namespace-correct link observations for NDP, TAP, hooks, sysctl and panel |
| [AUDIT-Q25-SYSCTL-NAMESPACE.md](reports/audit-0.8.2/AUDIT-Q25-SYSCTL-NAMESPACE.md) | Q25-F052–F053: sysctl namespace isolation and journal v2 migration |
| [AUDIT-Q25-TUN-ADMISSION.md](reports/audit-0.8.2/AUDIT-Q25-TUN-ADMISSION.md) | Q25-F054–F056: passive TUN admission and exclusive queue creation |
| [AUDIT-Q25-TUN-ATTACH.md](reports/audit-0.8.2/AUDIT-Q25-TUN-ATTACH.md) | Q25-F057–F058: prevent creation during attach and preserve TUN framing |
| [AUDIT-Q25-TUN-LIFETIME.md](reports/audit-0.8.2/AUDIT-Q25-TUN-LIFETIME.md) | Q25-F059–F060: retain original TUN descriptors through cleanup |
| [AUDIT-Q25-DNS-LEASES.md](reports/audit-0.8.2/AUDIT-Q25-DNS-LEASES.md) | Q25-F061–F062: generation-owned DNS and descriptor identity |
| [AUDIT-Q25-ROUTE-IDENTITY.md](reports/audit-0.8.2/AUDIT-Q25-ROUTE-IDENTITY.md) | Q25-F063–F064: original TUN/namespace evidence before route cleanup |
| [AUDIT-Q25-SETUP-IDENTITY.md](reports/audit-0.8.2/AUDIT-Q25-SETUP-IDENTITY.md) | Q25-F065–F066: original TUN during setup/roaming and terminal identity failure |
| [AUDIT-Q25-GATEWAY-IDENTITY.md](reports/audit-0.8.2/AUDIT-Q25-GATEWAY-IDENTITY.md) | Q25-F067–F068: gateway ownership, namespace/TUN checks and cleanup before fd release |
| [AUDIT-Q25-CLIENT-COMMANDS.md](reports/audit-0.8.2/AUDIT-Q25-CLIENT-COMMANDS.md) | Q25-F035/F036: route/firewall command bounds and protection for an unknown IPv4 path |
| [AUDIT-Q25-SETUP-FLUSH.md](reports/audit-0.8.2/AUDIT-Q25-SETUP-FLUSH.md) | Q25-F033/F034: shared initial setup and verified IPv4/IPv6 flush |
| [AUDIT-Q25-ROUTE-PENDING.md](reports/audit-0.8.2/AUDIT-Q25-ROUTE-PENDING.md) | Q25-F031/F032: pending mutations and release of absent orphan routes |
| [AUDIT-Q25-ROUTE-POSTCONDITIONS.md](reports/audit-0.8.2/AUDIT-Q25-ROUTE-POSTCONDITIONS.md) | Q25-F029/F030: verified carrier-route retirement and restoration |
| [AUDIT-Q25-ROUTE-SCOPE.md](reports/audit-0.8.2/AUDIT-Q25-ROUTE-SCOPE.md) | Q25-F027/F028: scoped route journal and rejection of commits after cleanup starts |
| [AUDIT-Q25-ROUTE-OUTCOME.md](reports/audit-0.8.2/AUDIT-Q25-ROUTE-OUTCOME.md) | Q25-F023/F024: verify failed route mutations before reversible rejection; reject ambiguous snapshots |
| [AUDIT-Q25-GATEWAY-WAN.md](reports/audit-0.8.2/AUDIT-Q25-GATEWAY-WAN.md) | Q25-F021/F022: bounded WAN discovery and lazy cleanup of remembered interfaces |
| [AUDIT-Q25-PATH-MONITOR.md](reports/audit-0.8.2/AUDIT-Q25-PATH-MONITOR.md) | Q25-F020: bounded Linux monitor queries and child ownership during stop |
| [AUDIT-Q25-SYSTEM-COMMANDS.md](reports/audit-0.8.2/AUDIT-Q25-SYSTEM-COMMANDS.md) | TUN/resolvectl execution and output bounds, child lifetime and DNS markers |
| [AUDIT-Q25-TCP-TASKS.md](reports/audit-0.8.2/AUDIT-Q25-TCP-TASKS.md) | TCP task ownership, closed admission and Linux path-worker joining |
| [AUDIT-Q25-UDP-TASKS.md](reports/audit-0.8.2/AUDIT-Q25-UDP-TASKS.md) | UDP task ownership, path transfer and joining before rollback |
| [AUDIT-Q25-TUN-WORKERS.md](reports/audit-0.8.2/AUDIT-Q25-TUN-WORKERS.md) | Shared TUN/Wintun worker ownership across cancelled shutdown |
| [AUDIT-Q25-H2-TASKS.md](reports/audit-0.8.2/AUDIT-Q25-H2-TASKS.md) | H2 driver/bridge ownership from connect through TCP-generation shutdown |
| [AUDIT-Q14-H2-TASKS.md](reports/audit-0.8.2/AUDIT-Q14-H2-TASKS.md) | Server-profile H2 tasks, joining before teardown and rejected-connection admission |
| [AUDIT-Q14-CONTROL.md](reports/audit-0.8.2/AUDIT-Q14-CONTROL.md) | Control socket ownership, API bounds, handler shutdown and hook pairing |
| [AUDIT-Q14-WORKER-NETWORK-LEASE.md](reports/audit-0.8.2/AUDIT-Q14-WORKER-NETWORK-LEASE.md) | One server worker per network namespace, crash/restart and deleted profiles |
| [AUDIT-Q14-FIREWALL-JOURNAL.md](reports/audit-0.8.2/AUDIT-Q14-FIREWALL-JOURNAL.md) | Exact server firewall journal, SIGKILL and recovery without listing |
| [AUDIT-Q25-DNS-MARKER-STORAGE.md](reports/audit-0.8.2/AUDIT-Q25-DNS-MARKER-STORAGE.md) | Trusted DNS state, namespace cookie v2 and SIGKILL/restart |
| [AUDIT-Q25-KILL-SWITCH-REBUILD.md](reports/audit-0.8.2/AUDIT-Q25-KILL-SWITCH-REBUILD.md) | Kill-switch protection across SIGKILL, failed rebuild and retry |
| [AUDIT-Q14-SUPERVISOR.md](reports/audit-0.8.2/AUDIT-Q14-SUPERVISOR.md) | Supervisor: stop/retry, Child/PID ownership, commands and termination deadlines |
| [AUDIT-Q14-Q19-LIFECYCLE.md](reports/audit-0.8.2/AUDIT-Q14-Q19-LIFECYCLE.md) | Profile shutdown, early startup errors, DNS listeners and socket release |
| [AUDIT-Q01-SERVER-INI.md](reports/audit-0.8.2/AUDIT-Q01-SERVER-INI.md) | First server INI pass: 7 findings, fixes, tests and remaining limits |
| [AUDIT.md](reports/AUDIT.md) | Current security model and audit status |
| [DPI-AUDIT.md](reports/DPI-AUDIT.md) | DPI detectability analysis and mitigations |
| [BENCHMARK.md](reports/BENCHMARK.md) | Load-testing method and per-mode measurements |
| [AUDIT-Q36-CURRENT-STATISTICS.md](reports/audit-0.8.2/AUDIT-Q36-CURRENT-STATISTICS.md) | Development measurements withdrawn; original data retained locally |
| [AUDIT-Q36-MATCHED-LOAD.md](reports/audit-0.8.2/AUDIT-Q36-MATCHED-LOAD.md) | Publication status of laboratory measurements |
| [AUDIT-Q36-TUN-STAGE.md](reports/audit-0.8.2/AUDIT-Q36-TUN-STAGE.md) | Publication status of diagnostic measurements |
| [AUDIT-Q36-FINAL.md](reports/audit-0.8.2/AUDIT-Q36-FINAL.md) | Audit-publication scope; not a release benchmark |
| [Qeli 0.8.0: 34 VPN modes](reports/benchmarks/vpn_protocol_benchmark_repeat_2026-09-01.md) | Full dated cross-protocol run, CPU/RSS, and interpretation limits |
| [COMPARISON.md](reports/COMPARISON.md) | Comparison with WireGuard, OpenVPN and V2Ray |

## Archive (`archive/`)

Frozen historical documents are preserved for traceability and are not maintained as current
guidance. Start with the **[archive map](archive/README.md)**.

### Completed plans and design logs

| Document | Frozen context |
|---|---|
| [REFACTOR-PLAN.md](archive/plans/REFACTOR-PLAN.md) | Completed production-duplicate removal plan and log |
| [DESIGN-remaining.md](archive/plans/DESIGN-remaining.md) | June 2026 REALITY development snapshot |
| [RELEASE-FIXES.md](archive/plans/RELEASE-FIXES.md) | Historical stabilization plan for early pre-1.0 releases |

### Historical audits

| Document | Date |
|---|---|
| [AUDIT-2026-06-10.md](archive/audits/AUDIT-2026-06-10.md) | 2026-06-10 — security and reliability audit |
| [AUDIT-2026-06-11.md](archive/audits/AUDIT-2026-06-11.md) | 2026-06-11 — external audit review and fixes |
| [AUDIT-2026-06-11-external2.md](archive/audits/AUDIT-2026-06-11-external2.md) | 2026-06-11 — second external audit review |
| [AUDIT-2026-06-12.md](archive/audits/AUDIT-2026-06-12.md) | 2026-06-12 — audit and fixes for 0.7.1 |

## Client documentation (next to client code)

| Client | Document |
|---|---|
| Windows | [qeli-win/README.md](../../qeli-win/README.md) |
| macOS | [qeli-mac/README.md](../../qeli-mac/README.md) |
| iOS ⚠️ | [qeli-ios/README.md](../../qeli-ios/README.md) · MDM: [qeli-ios/MDM/README.md](../../qeli-ios/MDM/README.md) — feature-complete but **never run on a device**, and nothing ships from it |
| Routers (OpenWrt) | [qeli-openwrt/README.md](../../qeli-openwrt/README.md) · Keenetic: [KEENETIC-DEPLOY.md](manuals/KEENETIC-DEPLOY.md) |
| Android | [qeli-android/README.md](../../qeli-android/README.md) (in Russian) |
| Linux CLI | [GETTING-STARTED §8.2](manuals/GETTING-STARTED.md) |

## Outside this directory

- **[../../CHANGELOG.md](../../CHANGELOG.md)** — all changes by version.
- **[../../release/RELEASE_NOTES_0.8.1.md](../../release/RELEASE_NOTES_0.8.1.md)** — bilingual
  0.8 architecture consolidation, user-visible outcomes, upgrade order, artifacts and release
  validation.
- **[../../release/RELEASE_NOTES_0.8.0.md](../../release/RELEASE_NOTES_0.8.0.md)** — development
  Reality/H2 migration, defaults, upgrade order and verification.
- **[../../release/dpi_audit_dev_0.8.0_h2_2026-08-26/REPORT.md](../../release/dpi_audit_dev_0.8.0_h2_2026-08-26/REPORT.md)** — dated H2 PCAP/DPI result and limitations.
- **[../../release/RELEASE_NOTES_0.7.16.md](../../release/RELEASE_NOTES_0.7.16.md)** — bilingual
  `0.7.16` release notes and upgrade impact.
- **[../../SECURITY.md](../../SECURITY.md)** — security policy and reporting.
- **[../../CONTRIBUTING.md](../../CONTRIBUTING.md)** — how to contribute.
- **[../../release/docker/README.md](../../release/docker/README.md)** — running the server in Docker.

- [Q25-F099: ownership of changed Linux routes](reports/audit-0.8.2/AUDIT-Q25-ROUTE-ATTRIBUTES.md).

- [Q25-F100: physical route journal and SIGKILL recovery](reports/audit-0.8.2/AUDIT-Q25-ROUTE-JOURNAL.md).

- [Q25-F101: refuse unscoped legacy global DNS recovery](reports/audit-0.8.2/AUDIT-Q25-LEGACY-DNS.md).

- [Q25: persistent TUN/TAP runtime and manual recovery validation](reports/audit-0.8.2/AUDIT-Q25-PERSISTENT-TUN.md).

- [Q14: mixed nft/legacy/firewalld server recovery, 16 scenarios](reports/audit-0.8.2/AUDIT-Q14-MIXED-FIREWALL.md).

- [Q25-F102: client mixed nft/legacy/firewalld and crash recovery](reports/audit-0.8.2/AUDIT-Q25-CLIENT-MIXED-FIREWALL.md).
- [Q25-F103: shared DNS/NSS, cancellation and client shutdown](reports/audit-0.8.2/AUDIT-Q25-SYSTEM-RESOLVER.md).

- [Q25-F104: resolver-file reads, shared parser and DNS allowances](reports/audit-0.8.2/AUDIT-Q25-RESOLVER-FILES.md).

- [Q25-F105: asynchronous NetworkPlan application and rollback ownership](reports/audit-0.8.2/AUDIT-Q25-NETWORK-TASK.md).

- [Q25-F106: asynchronous teardown of established tunnels](reports/audit-0.8.2/AUDIT-Q25-TUN-TEARDOWN.md).

- [Q25-F107/F108: asynchronous firewall and chain retention on unhook failure](reports/audit-0.8.2/AUDIT-Q25-FIREWALL-TASK.md).

- [Q15-F002: wildcard UDP local reply address](reports/audit-0.8.2/AUDIT-Q15-UDP-LOCAL-ADDRESS.md)

- [Q25-F109: startup recovery, stop and retained network lease](reports/audit-0.8.2/AUDIT-Q25-STARTUP-RECOVERY-TASK.md)

- [Q25-F110: TUN pump startup and early rollback on a joined worker](reports/audit-0.8.2/AUDIT-Q25-PUMP-START.md)

- [Q25-F111: ordered client diagnostics and joined final publication](reports/audit-0.8.2/AUDIT-Q25-STATUS-WRITER.md)

- [Q25-F112/F113: stable device-id, bounded reads and TOFU integrity](reports/audit-0.8.2/AUDIT-Q25-IDENTITY-FILES.md)

- [Q25-F114: owned TOFU worker and cancellation-safe errors](reports/audit-0.8.2/AUDIT-Q25-IDENTITY-WORKER.md)

- [Q25-F115: joined server cleanup worker](reports/audit-0.8.2/AUDIT-Q25-SERVER-CLEANUP-WORKER.md)

- [Q25-F116: server profile TUN/NAT setup off the executor](reports/audit-0.8.2/AUDIT-Q25-SERVER-SETUP-WORKER.md)

- [Q25-F117: server profile DNS firewall setup off the executor](reports/audit-0.8.2/AUDIT-Q25-SERVER-DNS-SETUP-WORKER.md)

- [Q25-F118: NDP proxy bind off the executor](reports/audit-0.8.2/AUDIT-Q25-SERVER-NDP-BIND-WORKER.md)

- [Q25-F119: shared profile setup budget and listener readiness](reports/audit-0.8.2/AUDIT-Q25-SERVER-SETUP-BUDGET.md)

- [Q25-F120: recheck the IPv6 exit WAN during roaming refresh](reports/audit-0.8.2/AUDIT-Q25-GATEWAY-IPV6-ROAM.md)

- [Q25-F121: late IPv6 cannot bypass the kill switch](reports/audit-0.8.2/AUDIT-Q25-KILL-SWITCH-DYNAMIC-IPV6.md)

- [Q25-F122: a late IPv4 route cannot bypass the kill switch](reports/audit-0.8.2/AUDIT-Q25-KILL-SWITCH-DYNAMIC-IPV4.md)

- [Q25-F123: TUN attach across network namespaces](reports/audit-0.8.2/AUDIT-Q25-TUN-ATTACH-CONTEXT.md)

- [Q25-F124: guard exit-node against off-WAN source leaks](reports/audit-0.8.2/AUDIT-Q25-EXIT-POLICY-ROUTING.md)

- [Q25-A125: physical WAN name reuse](reports/audit-0.8.2/AUDIT-Q25-WAN-NAME-REUSE.md)

- [Q25-F126: select the default WAN by route metric](reports/audit-0.8.2/AUDIT-Q25-WAN-METRIC.md)

- [Q25-F127: exit WAN monitor without VPN path COMMIT](reports/audit-0.8.2/AUDIT-Q25-EXIT-WAN-MONITOR.md)

- [Q25-F128: reject the exit TUN as its own WAN](reports/audit-0.8.2/AUDIT-Q25-EXIT-SELF-WAN.md)

- [Q25-F129: ambiguous ECMP default WAN](reports/audit-0.8.2/AUDIT-Q25-WAN-ECMP.md)

- [D09: Linux lifecycle and system-failure closure](reports/audit-0.8.2/AUDIT-Q25-LINUX-LIFECYCLE-CLOSURE.md)

- [Q25-F130: retain server worker lease after forced cancellation](reports/audit-0.8.2/AUDIT-Q25-SERVER-FORCED-DROP-LEASE.md)

- [Q25-F131: verify WAN presence before server rule setup](reports/audit-0.8.2/AUDIT-Q25-SERVER-WAN-PRESENCE.md)

- [Q25-F132: reject ambiguous server auto-WAN](reports/audit-0.8.2/AUDIT-Q25-SERVER-WAN-ECMP.md)

- [Q25-F133: fix the client-only Linux build](reports/audit-0.8.2/AUDIT-Q25-CLIENT-ONLY-BUILD.md)
- [Q25-F134: bind NAT66 transit to the selected WAN](reports/audit-0.8.2/AUDIT-Q25-SERVER-NAT66-EGRESS.md)
- [Q25-F135: refuse NAT auto-WAN without default routes](reports/audit-0.8.2/AUDIT-Q25-SERVER-AUTO-WAN-FAILURE.md)
- [Q25-F136: NAT44 off-WAN boundary and private LAN](reports/audit-0.8.2/AUDIT-Q25-SERVER-NAT44-EGRESS.md)
- [Q25-F137: reject partial VPN-auth SIGHUP changes](reports/audit-0.8.2/AUDIT-Q25-SERVER-SIGHUP-AUTH.md)
- [Q25-F138: one 16 MiB limit for server INI](reports/audit-0.8.2/AUDIT-Q25-SERVER-INI-SIZE.md)
- [Q25-F139: panel live state after saving INI](reports/audit-0.8.2/AUDIT-Q25-SERVER-WEB-LIVE.md)
- [Q25-F140: private panel writes for server INI](reports/audit-0.8.2/AUDIT-Q25-SERVER-INI-PERMISSIONS.md)
- [Q25-F141: consistent CLI, panel and runtime admission](reports/audit-0.8.2/AUDIT-Q25-SERVER-CHECK-CONFIG-PARITY.md)
- [Q25-F142: profile name and server identity-key path](reports/audit-0.8.2/AUDIT-Q25-SERVER-IDENTITY-NAME.md)
- [Q25-F143: trust for identity_key and logging.file](reports/audit-0.8.2/AUDIT-Q25-SERVER-IDENTITY-TRUST.md)
- [Q25-F144: trust for custom auth.users_file](reports/audit-0.8.2/AUDIT-Q25-SERVER-USERS-PATH-TRUST.md)
- [Q25-F145: SIGHUP and restart for users_file](reports/audit-0.8.2/AUDIT-Q25-SERVER-USERS-SIGHUP-RESTART.md)
- [Q25-F146: concurrent server INI writes](reports/audit-0.8.2/AUDIT-Q25-SERVER-CONCURRENT-WRITES.md)
- [Q25-F147: TLS PEM and incomplete auto pair](reports/audit-0.8.2/AUDIT-Q25-SERVER-TLS-PEM.md)
- [Q25-F148: TLS validation in check-config](reports/audit-0.8.2/AUDIT-Q25-SERVER-TLS-CHECK-CONFIG.md)
- [Q25-F149: TLS path trust and Let's Encrypt panel saves](reports/audit-0.8.2/AUDIT-Q25-SERVER-TLS-PATH-TRUST.md)
- [Q25-F150: INI trust in panel identity and Share](reports/audit-0.8.2/AUDIT-Q25-SERVER-PANEL-SNAPSHOT-TRUST.md)
- [Q25-F151: panel saves and INI trust](reports/audit-0.8.2/AUDIT-Q25-SERVER-PANEL-SAVE-TRUST.md)
- [Q25-F152: INI matrix and logging restart](reports/audit-0.8.2/AUDIT-Q25-SERVER-FIELD-MATRIX-LOGGING.md)
- [Q25-F153: web-auth validation before save](reports/audit-0.8.2/AUDIT-Q25-SERVER-WEB-AUTH-SAVE.md)
- [Q25-F154: Quick Start and lockout policy saves](reports/audit-0.8.2/AUDIT-Q25-SERVER-QUICKSTART-BF-SAVE.md)
- [Q25-F155: SIGHUP restart hints for unapplied settings](reports/audit-0.8.2/AUDIT-Q25-SERVER-SIGHUP-RESTART-HINTS.md)
- [Q25-F156: panel Argon2 validation at admission and live reload](reports/audit-0.8.2/AUDIT-Q25-SERVER-WEB-HASH-VALIDATION.md)
- [Q25-F157: truthful panel live-reload result](reports/audit-0.8.2/AUDIT-Q25-SERVER-WEB-RELOAD-RESULT.md)
- [Q25-F158: saved versus active lockout policy](reports/audit-0.8.2/AUDIT-Q25-SERVER-BLOCKED-LIVE.md)
- [Q25-F159: stable ordering of dynamic INI fields](reports/audit-0.8.2/AUDIT-Q25-SERVER-INI-ORDER.md)
- [D07: global server INI fields](reports/audit-0.8.2/AUDIT-Q25-SERVER-GLOBAL-FIELDS.md)
- [D07: foundational server profile fields](reports/audit-0.8.2/AUDIT-Q25-SERVER-PROFILE-FOUNDATION.md)
- [D07: server profile obf.* fields](reports/audit-0.8.2/AUDIT-Q25-SERVER-PROFILE-OBF.md)
- [Q25-F160: INI revision for lockout-policy saves](reports/audit-0.8.2/AUDIT-Q25-SERVER-BLOCKED-REVISION.md)
- [Q25-F161: archive restore directory scan errors](reports/audit-0.8.2/AUDIT-Q25-SERVER-ARCHIVE-SCAN.md)
- [Q25-F162: INI-only notification settings](reports/audit-0.8.2/AUDIT-Q25-SERVER-NOTIFY-INI.md)
- [Q25-F163: concurrent notification saves](reports/audit-0.8.2/AUDIT-Q25-SERVER-NOTIFY-REVISION.md)
- [Q25-F164: panel client INI revisions](reports/audit-0.8.2/AUDIT-Q25-CLIENT-PROFILE-REVISION.md)
- [Q25-F165: client autostart and diagnostic bounds](reports/audit-0.8.2/AUDIT-Q25-CLIENT-PROFILE-BOUNDS.md)
- [Q25-F166: stale Delete and applying client INI changes](reports/audit-0.8.2/AUDIT-Q25-CLIENT-DELETE-REVISION.md)
- [Q25-F167: byte-faithful Android client INI import](reports/audit-0.8.2/AUDIT-Q25-ANDROID-FILE-IMPORT.md)
- [Q25-F168: file-based client INI in desktop CLI](reports/audit-0.8.2/AUDIT-Q25-DESKTOP-CONFIG-FILES.md)
- [Q25-F169: preserve client INI text until the shared parser](reports/audit-0.8.2/AUDIT-Q25-CLIENT-UI-INI-INPUT.md)
- [Q25-F170: mobile client INI size budget](reports/audit-0.8.2/AUDIT-Q25-MOBILE-CONFIG-BUDGET.md)
- [Q25-F171: desktop profile-store encoding](reports/audit-0.8.2/AUDIT-Q25-DESKTOP-STORE-UTF8.md)
- [Q25-F172: desktop profile-store recovery](reports/audit-0.8.2/AUDIT-Q25-DESKTOP-STORE-RECOVERY.md)
- [Q25-F173: desktop UI profile transactions](reports/audit-0.8.2/AUDIT-Q25-DESKTOP-PROFILE-TRANSACTIONS.md)
- [Q25-F174: confirmed Android profile writes](reports/audit-0.8.2/AUDIT-Q25-ANDROID-PROFILE-TRANSACTIONS.md)
- [Q25-F175: protect mobile profiles after load failure](reports/audit-0.8.2/AUDIT-Q25-MOBILE-STORE-LOAD-FAILURE.md)
- [Q25-F176: strict Android profile-store loading](reports/audit-0.8.2/AUDIT-Q25-ANDROID-STORE-DECODING.md)
- [Q25-F177: client UI revisions and active-profile application](reports/audit-0.8.2/AUDIT-Q25-CLIENT-PROFILE-UI-REVISION.md)
- [Q25-F178: cross-process desktop profile-store writes](reports/audit-0.8.2/AUDIT-Q25-DESKTOP-STORE-CONCURRENCY.md)
- [Q25-F179: Android profile store rejects stale writes](reports/audit-0.8.2/AUDIT-Q25-ANDROID-STORE-VERSION.md)
- [Q25-F180: empty iOS profile archives are not replaced by a template](reports/audit-0.8.2/AUDIT-Q25-IOS-EMPTY-ARCHIVE.md)
- [Q25-F181: iOS notice when an active profile needs reconnect](reports/audit-0.8.2/AUDIT-Q25-IOS-ACTIVE-PROFILE-RECONNECT.md)
- [Q25-F182: iOS profile snapshots for Edit and Delete](reports/audit-0.8.2/AUDIT-Q25-IOS-PROFILE-SNAPSHOT.md)
- [Q25-F183: bounded desktop profile-store reads](reports/audit-0.8.2/AUDIT-Q25-DESKTOP-STORE-BOUNDS.md)
- [Q25-F184: bounded desktop profile-store lock wait](reports/audit-0.8.2/AUDIT-Q25-DESKTOP-STORE-LOCK-WAIT.md)
- [Q25-F185: desktop profile-store structure](reports/audit-0.8.2/AUDIT-Q25-DESKTOP-STORE-STRUCTURE.md)
- [Q25-F186: Android active-profile index](reports/audit-0.8.2/AUDIT-Q25-ANDROID-ACTIVE-INDEX.md)
- [Q25-F187: active-profile index on iOS Restore](reports/audit-0.8.2/AUDIT-Q25-IOS-ACTIVE-INDEX.md)
- [Q25-F188: mobile store and backup bounds](reports/audit-0.8.2/AUDIT-Q25-MOBILE-STORE-BOUNDS.md)
- [Q25-F189: stable desktop profile IDs](reports/audit-0.8.2/AUDIT-Q25-DESKTOP-PROFILE-IDENTITY.md)
- [Q25-F190: server INI history errors](reports/audit-0.8.2/AUDIT-Q25-SERVER-INI-HISTORY.md)
- [Q25-F191: null fields in desktop profile stores](reports/audit-0.8.2/AUDIT-Q25-DESKTOP-STORE-NULL-FIELDS.md)
- [Q25-F192: shared Windows/macOS settings storage](reports/audit-0.8.2/AUDIT-Q25-DESKTOP-SETTINGS-STORE.md)
- [Q25-F193: Android per-app editor and INI sections](reports/audit-0.8.2/AUDIT-Q25-ANDROID-APPS-INI.md)
- [Q25-F194: portable password in share links and editor build](reports/audit-0.8.2/AUDIT-Q25-CLIENT-SHARE-PASSWORD.md)
- [Q25-F195: share-link errors in client UI](reports/audit-0.8.2/AUDIT-Q25-CLIENT-SHARE-UI.md)
- [Q25-F196: share URI password contract across adapters](reports/audit-0.8.2/AUDIT-Q25-CLIENT-SHARE-PARITY.md)
- [Q25-F197: shared INI editor matrix](reports/audit-0.8.2/AUDIT-Q25-CLIENT-INI-MATRIX.md)
- [Q25-F198: direct qeli:// import budget](reports/audit-0.8.2/AUDIT-Q25-CLIENT-URI-BUDGET.md)
- [Q25-F199: reproducible native build inputs](reports/audit-0.8.2/AUDIT-Q25-NATIVE-BUILD-INPUTS.md)
- [Q25-F200: audit report structure and language parity](reports/audit-0.8.2/AUDIT-Q25-DOC-PARITY.md)
- [Q25-F201: patch archive and evidence age](reports/audit-0.8.2/AUDIT-Q25-PATCH-RECONCILIATION.md)
- [Q25-F202: reproducible native cores and Android runtime](reports/audit-0.8.2/AUDIT-Q25-NATIVE-REBUILD.md)
- [Q25-F203: Android Private DNS, VPN traffic and Wi-Fi change](reports/audit-0.8.2/AUDIT-Q25-ANDROID-NETWORK-IDENTITY.md)
- [Q25-F204: Linux worker resource churn](reports/audit-0.8.2/AUDIT-Q25-WORKER-RESOURCE-CHURN.md)

- [Q04: panel authentication and API boundaries](reports/audit-0.8.2/AUDIT-Q04-WEB-AUTH.md)

- [Q05: configuration transactions and restart](reports/audit-0.8.2/AUDIT-Q05-HTTP-TRANSACTIONS.md)

- [Q06: users and access revocation](reports/audit-0.8.2/AUDIT-Q06-USERS-ACCESS.md)

- [Q07: backup, restore and history](reports/audit-0.8.2/AUDIT-Q07-BACKUP-RESTORE.md)

- [Q08: cryptography and key storage](reports/audit-0.8.2/AUDIT-Q08-CRYPTO-KEYS.md)
- [Q09: UDP AUTH ownership and pre-authentication PMTU](reports/audit-0.8.2/AUDIT-Q09-UDP-AUTH.md)
- [Q09: TCP authentication and ClientHello](reports/audit-0.8.2/AUDIT-Q09-TCP-PARSER.md)
- [Q09: UDP handshake and admission](reports/audit-0.8.2/AUDIT-Q09-UDP-CONTRACTS.md)
- [Q09: final audit and KICK retention](reports/audit-0.8.2/AUDIT-Q09-FINAL.md)

- [Q10: PacketCodec, replay and CONTROL_V2](reports/audit-0.8.2/AUDIT-Q10-CODEC-CONTROL.md)

- [Q11: REALITY, TLS 1.3 and HTTP/2](reports/audit-0.8.2/AUDIT-Q11-REALITY-TLS-H2.md)
- [Q12: transports and wire camouflage — complete](reports/audit-0.8.2/AUDIT-Q12-TRANSPORTS.md)
- [Q13: recordizer, padding and shaping — complete](reports/audit-0.8.2/AUDIT-Q13-MORPHOLOGY.md)
- [Q14: supervisor, workers and profiles — complete](reports/audit-0.8.2/AUDIT-Q14-CURRENT-LIFECYCLE.md)
- [Q15: sessions, IP pools and limits — complete](reports/audit-0.8.2/AUDIT-Q15-CURRENT-SESSIONS.md)
- [Q16: ACL and site-to-site — complete](reports/audit-0.8.2/AUDIT-Q16-ACL-ROUTES.md)
- [Q17: IPv4 NAT, forwarding and sysctls — complete](reports/audit-0.8.2/AUDIT-Q17-IPV4-NETWORK.md)
- [Q18: IPv6 and NDP — complete](reports/audit-0.8.2/AUDIT-Q18-IPV6-NDP.md)
- [Q19: server and client DNS — complete](reports/audit-0.8.2/AUDIT-Q19-DNS-FINAL.md)

- [Q20: DHCP and lease lifecycle — complete](reports/audit-0.8.2/AUDIT-Q20-DHCP-FINAL.md)
- [Q21: TUN/TAP, IP and MTU/PMTU — complete](reports/audit-0.8.2/AUDIT-Q21-PACKETS-FINAL.md)
- [Q22: transport core, FFI/JNI and memory — complete](reports/audit-0.8.2/AUDIT-Q22-CORE-FFI-FINAL.md)
- [Q23: roaming, resume and CONTROL_V2 — complete](reports/audit-0.8.2/AUDIT-Q23-ROAMING-FINAL.md)
- [Q24: multipath and shared budgets — DONE/PASS](reports/audit-0.8.2/AUDIT-Q24-BONDING.md)

- [Q25: shared Linux CLI INI bootstrap — verified](reports/audit-0.8.2/AUDIT-Q25-CLI-BOOTSTRAP.md)

- [Q25: Linux CLI and network recovery — DONE/PASS](reports/audit-0.8.2/AUDIT-Q25-LINUX-CLI-FINAL.md)

- [Q26: shared C# and managed/native — DONE/PASS](reports/audit-0.8.2/AUDIT-Q26-MANAGED-FINAL.md)

- [Q27: Windows storage — stage PASS, section IN_PROGRESS](reports/audit-0.8.2/AUDIT-Q27-WINDOWS-STORAGE.md)

- [Q27: service transitions and autostart — stage PASS](reports/audit-0.8.2/AUDIT-Q27-WINDOWS-CONTROL.md)

- [Q27: service registration, status and logs — stage PASS](reports/audit-0.8.2/AUDIT-Q27-WINDOWS-OBSERVATION.md)

- [Q27: Windows GUI, service and drivers — DONE/PASS](reports/audit-0.8.2/AUDIT-Q27-WINDOWS-FINAL.md)

- [Q28: macOS storage and keys — stage PASS](reports/audit-0.8.2/AUDIT-Q28-MACOS-STORAGE.md)

- [Q28: macOS daemon, helper and observation — PASS](reports/audit-0.8.2/AUDIT-Q28-MACOS-CONTROL.md)

- [Q28: macOS network cleanup — PASS](reports/audit-0.8.2/AUDIT-Q28-MACOS-NETWORK.md)

- [Q28: per-app bridge / Swift / build paths](reports/audit-0.8.2/AUDIT-Q28-MACOS-PERAPP.md)

- [Q28: forwarding ownership and recovery](reports/audit-0.8.2/AUDIT-Q28-MACOS-FORWARDING.md)

- [Q28: guardian readiness and generation ownership](reports/audit-0.8.2/AUDIT-Q28-MACOS-GUARDIAN.md)

- [Q28: native socket lifetime and I/O budgets](reports/audit-0.8.2/AUDIT-Q28-MACOS-SOCKETS.md)

- [Q28: final macOS integration](reports/audit-0.8.2/AUDIT-Q28-MACOS-INTEGRATION.md)

- [Q29: Android storage, profile editor, export and Release](reports/audit-0.8.2/AUDIT-Q29-ANDROID-STORAGE-PACKAGE.md)

- [Q29: concurrent TUN/JNI teardown](reports/audit-0.8.2/AUDIT-Q29-ANDROID-LIFECYCLE.md)
- [Q29: foreground service commands and lifecycle](reports/audit-0.8.2/AUDIT-Q29-ANDROID-SERVICE.md)
- [Q29: VPN payload and post-auth lifecycle](reports/audit-0.8.2/AUDIT-Q29-ANDROID-DATA.md)
- [Q29: system lockdown, process death and revoke](reports/audit-0.8.2/AUDIT-Q29-ANDROID-SYSTEM.md)
- [Q29: screen, deep idle and TCP recovery](reports/audit-0.8.2/AUDIT-Q29-ANDROID-POWER.md)
- [Q29: UDP/QUIC recovery and grace expiry](reports/audit-0.8.2/AUDIT-Q29-ANDROID-UDP-RECOVERY.md)
- [Q29: Wi-Fi/Cellular switching and TUN retention](reports/audit-0.8.2/AUDIT-Q29-ANDROID-HANDOVER.md)
- [Q29: TCP across Android Wi-Fi/Cellular switching](reports/audit-0.8.2/AUDIT-Q29-ANDROID-TCP-HANDOVER.md)
- [Q29: IPv4/IPv6 TCP/UDP after Android network switching](reports/audit-0.8.2/AUDIT-Q29-ANDROID-HANDOVER-PAYLOAD.md)
- [Q29: minified Release and Android network runtime](reports/audit-0.8.2/AUDIT-Q29-ANDROID-RELEASE-RUNTIME.md)
- [Q29: Android Release sleep and transport recovery](reports/audit-0.8.2/AUDIT-Q29-ANDROID-RELEASE-FAULTS.md)
- [Q29: Android Release on an IPv6-only DNS64/NAT64 network](reports/audit-0.8.2/AUDIT-Q29-ANDROID-NAT64.md)
- [Q29: packets crossing carrier transitions and force-stop](reports/audit-0.8.2/AUDIT-Q29-ANDROID-LEAK-BURSTS.md)
- [Q29: system DNS and cold Android Release enable](reports/audit-0.8.2/AUDIT-Q29-ANDROID-STARTUP-DNS.md)
- [Q29: comparing Android DNS APIs](reports/audit-0.8.2/AUDIT-Q29-ANDROID-RESOLVER-DIAGNOSTIC.md)
- [Q29: VPN publication during cold startup](reports/audit-0.8.2/AUDIT-Q29-ANDROID-STARTUP-STATE.md)
- [Q29: CONNECTED after Android VPN publication](reports/audit-0.8.2/AUDIT-Q29-ANDROID-CONNECTED-GATE.md)

- [Q29: instrumented Release runner](reports/audit-0.8.2/AUDIT-Q29-ANDROID-RELEASE-RUNNER.md)

- [Q29: per-app UID / Private DNS](reports/audit-0.8.2/AUDIT-Q29-ANDROID-APP-POLICY.md)

- [Q29: Doze / repeated carrier handover](reports/audit-0.8.2/AUDIT-Q29-ANDROID-ENDURANCE.md)
- [Q29: authenticated Private DNS / DoT](reports/audit-0.8.2/AUDIT-Q29-ANDROID-TRUSTED-DOT.md)

- [Q29: connection permission ownership](reports/audit-0.8.2/AUDIT-Q29-ANDROID-PERMISSIONS.md)

- [Q29: trusted Wi-Fi runtime](reports/audit-0.8.2/AUDIT-Q29-ANDROID-TRUSTED-WIFI.md)
- [Q29 F281: trusted Wi-Fi with active lockdown](reports/audit-0.8.2/AUDIT-Q29-ANDROID-TRUSTED-LOCKDOWN.md)
- [Q29 F282–F283: protect/callback ownership and SSID redaction](reports/audit-0.8.2/AUDIT-Q29-ANDROID-CONTROLLER.md)

- [Q29 F286–F288: bounded input, UI state and source reconciliation](reports/audit-0.8.2/AUDIT-Q29-ANDROID-SOURCE.md)

- [Q29 F289: LAN settings reconfiguration](reports/audit-0.8.2/AUDIT-Q29-ANDROID-SETTINGS.md)
- [Q29: current VPN integrations and Android fixture reliability](reports/audit-0.8.2/AUDIT-Q29-ANDROID-CURRENT-VPN.md)

- [Q30: iOS MDM / provider snapshot](reports/audit-0.8.2/AUDIT-Q30-IOS-POLICY-SNAPSHOT.md)

- [Q30: iOS Keychain / TOFU](reports/audit-0.8.2/AUDIT-Q30-IOS-KEYCHAIN.md)

- [Q30: iOS lifecycle / provider ownership](reports/audit-0.8.2/AUDIT-Q30-IOS-LIFECYCLE.md)

- [Q30: iOS app / managed preferences](reports/audit-0.8.2/AUDIT-Q30-IOS-APP-PREFERENCES.md)

- [Q30: iOS provider messages / backup UI](reports/audit-0.8.2/AUDIT-Q30-IOS-MESSAGES-BACKUP.md)

- [Q30: iOS source / memory reconciliation](reports/audit-0.8.2/AUDIT-Q30-IOS-SOURCE-MEMORY.md)

- [Q31: OpenWrt / LuCI control](reports/audit-0.8.2/AUDIT-Q31-OPENWRT-CONTROLS.md)

- [Q32: metrics across missing samples and worker changes](reports/audit-0.8.2/AUDIT-Q32-METRICS.md)
- [Q32: usage reset after publication](reports/audit-0.8.2/AUDIT-Q32-USAGE.md)
- [Q32: bounded logs and trace publication](reports/audit-0.8.2/AUDIT-Q32-LOGS-TRACE.md)
- [AUDIT-Q32-NOTIFICATIONS](reports/audit-0.8.2/AUDIT-Q32-NOTIFICATIONS.md)
- [Q33: Docker entrypoint paths and first-run files](reports/audit-0.8.2/AUDIT-Q33-DOCKER-ENTRYPOINT.md)

- [Q33: server updater transactions](reports/audit-0.8.2/AUDIT-Q33-SERVER-UPDATER.md)

- [Q33: Debian package lifecycle](reports/audit-0.8.2/AUDIT-Q33-DEBIAN-LIFECYCLE.md)

- [Q33: binary installer preparation](reports/audit-0.8.2/AUDIT-Q33-BINARY-INSTALLER.md)

- [Q33: installer dependencies and package admission](reports/audit-0.8.2/AUDIT-Q33-INSTALLER-ADMISSION.md)

- [Q33: installer secret/config publication](reports/audit-0.8.2/AUDIT-Q33-INSTALLER-SECRETS.md)

- [Q33: installer preflight and staged INI](reports/audit-0.8.2/AUDIT-Q33-INSTALLER-CONFIG.md)

- [Q33: actual installer CLI/version compatibility](reports/audit-0.8.2/AUDIT-Q33-INSTALLER-CLI.md)

- [Q33: service-user transitions and secret ownership](reports/audit-0.8.2/AUDIT-Q33-SERVICE-USER.md)

- [Q33: real systemd sandbox, reload and root mode](reports/audit-0.8.2/AUDIT-Q33-SYSTEMD.md)

- [Q33: actual DEB, update and rollback](reports/audit-0.8.2/AUDIT-Q33-PACKAGE-MANAGER.md)

- [Q33: final reconciliation, hooks and restore](reports/audit-0.8.2/AUDIT-Q33-COMPLETION.md)

- [Q34: Rust dependencies, CI policy and TLS PEM](reports/audit-0.8.2/AUDIT-Q34-DEPENDENCY-POLICY.md)

- [Q34: client build dependencies and Windows signatures](reports/audit-0.8.2/AUDIT-Q34-CLIENT-SUPPLY.md)

- [Q34: .NET SDK and dependency locks](reports/audit-0.8.2/AUDIT-Q34-SDK-LOCKS.md)
- [Q34: Android dependency locks and checksum verification](reports/audit-0.8.2/AUDIT-Q34-ANDROID-DEPENDENCIES.md)
- [Q34: Android Maven security and license inventory](reports/audit-0.8.2/AUDIT-Q34-MAVEN-SECURITY.md)
- [Q34: Android project licenses and Maven notices](reports/audit-0.8.2/AUDIT-Q34-ANDROID-NOTICES.md)

- [Q34: common Cargo legal resources](reports/audit-0.8.2/AUDIT-Q34-NATIVE-NOTICES.md)

- [Q34: .NET notices and embedded Inter font license](reports/audit-0.8.2/AUDIT-Q34-MANAGED-NOTICES.md)

- [Q34: toolchain runtime and standard-library notices](reports/audit-0.8.2/AUDIT-Q34-TOOLCHAIN-NOTICES.md)

- [Q34: release source admission and Docker notices](reports/audit-0.8.2/AUDIT-Q34-PUBLICATION-GATES.md)

- [Q34: artifact provenance and atomic checksums](reports/audit-0.8.2/AUDIT-Q34-ATTESTATION-CHECKSUMS.md)

- [Q34: full amd64 Docker build and release admission](reports/audit-0.8.2/AUDIT-Q34-DOCKER-BUILD.md)

- [Q34: immutable Docker frontend and APT inputs](reports/audit-0.8.2/AUDIT-Q34-DOCKER-INPUTS.md)

- [Q34: OpenWrt source Git flags](reports/audit-0.8.2/AUDIT-Q34-OPENWRT-SOURCE-FLAGS.md)

- [Q34: OpenWrt exact local source cut](reports/audit-0.8.2/AUDIT-Q34-OPENWRT-SOURCE-CUT.md)

- [Q34: Fuzz CI inventory](reports/audit-0.8.2/AUDIT-Q34-FUZZ-INVENTORY.md)

- [Q34: Fuzz toolchain and ASan smoke](reports/audit-0.8.2/AUDIT-Q34-FUZZ-TOOLCHAIN.md)

- [Permanent Actions lint](reports/audit-0.8.2/AUDIT-Q34-WORKFLOW-LINT.md) — Q34/F488, 9 October 2026.

- [MicroCom source identity and compatibility](reports/audit-0.8.2/AUDIT-Q34-SOURCE-BINDING.md) — Q34/F489, 9October2026.

- [Q37: documentation and test harnesses](reports/audit-0.8.2/AUDIT-Q37-DOCUMENTATION.md) — 11 October 2026.

- [Q34: native libraries for the adopted patch](reports/audit-0.8.2/AUDIT-Q34-ADOPTED-NATIVE.md) — 13 October 2026.

- [Q35: final five-criterion mapping](reports/audit-0.8.2/AUDIT-Q35-FINAL.md) — 13 October 2026.

- [Q36: current statistical population](reports/audit-0.8.2/AUDIT-Q36-CURRENT-STATISTICS.md)

**Internal audit: 37/37 sections complete within the agreed scope.** Generic/null DnsResolver ENONET on the tested API34 image is an accepted limitation and remains an unfixed FAIL. D06 and the physical-platform exclusions are retained. [Current result and history](plans/FULL-SYSTEM-AUDIT.md) · [Android limitation](reports/audit-0.8.2/AUDIT-Q29-ANDROID-RESOLVER-DIAGNOSTIC.md). [Android recovery](reports/audit-0.8.2/AUDIT-Q29-ANDROID-RECOVERY.md).
