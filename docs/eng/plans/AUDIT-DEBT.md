# Technical debt from started audits

<!-- normative-sync: audit-debt-v1 -->

Reconciled on 24 September 2026. At the user’s request, new full-audit sections
are paused until this register is closed. These are **15 groups of obligations**,
not 15 confirmed bugs or a completion percentage for all 37 sections. Sources: 57 baseline
`AUDIT-Q*.md` reports and `CLIENT-CONFIG-CORE.md`; repeated limitations are consolidated.

`DONE` requires a fix/justification and the required verification. `BLOCKED` denotes an
unavailable external prerequisite, never success. The user supplied Linux and an Android
emulator on 24 September; the historical “no Linux environment” limitation is obsolete.
Connections to both Linux VMs were verified; the running server and its files were not replaced.

[Full plan](FULL-SYSTEM-AUDIT.md) · [Shared configuration core](CLIENT-CONFIG-CORE.md)

| ID | Sections | Status | Debt | Closure criteria / current evidence |
|---|---|---|---|---|
| D01 | 14/17/18/25 | DONE | NAT and retired-generation failures | Retain exact rules before mutation; retry and final failure; prevent restart after incomplete cleanup; release IPv4 forwarding. Unit/cross, 18 native and 8 worker E2E PASS; baseline IPv4 leak reproduced. [Report](../reports/AUDIT-Q14-RETAINED-CLEANUP.md). |
| D02 | 14/25 | IN_PROGRESS | Internal sysctl boundaries | Verify namespace after lock waits and across I/O/pruning; protect journal reads; distinguish original and replaced links. Identity-loss regressions and native restore. |
| D03 | 22/25 | DONE | Standalone kill switch | Pinned namespace, retained exact-family owner, fail-closed reconnect and safe address rotation. 11 portable + 2 native regressions; actual IPv4/IPv6 filter counters and 2 baseline failures. [Report](../reports/AUDIT-Q25-KILL-SWITCH-IDENTITY.md). |
| D04 | 14/19/22/25 | TODO | Crash recovery | Define and implement safe exact firewall/DNS/route recovery, including mixed nft, SIGKILL and deleted profiles. A process-local registry is not a persistent journal. |
| D05 | 05/14/25 | IN_PROGRESS | Whole-operation deadlines and locks | Move synchronous preflight out of async handlers/long config locks; bound command sequences and waits; verify cancellation and sibling request availability. |
| D06 | 21/22/23/25 | IN_PROGRESS | External network-resource context | Verify WAN identity, resolved/bus context, sysfs/procfs and attach/name contracts; process-global DNS/carrier state and dynamic IPv6. Document supported combinations. |
| D07 | 01/05/09/11 | TODO | Server configuration at runtime | Trace field → parse/validate/runtime/serialize; malformed/oversized input; check-config/startup/SIGHUP/HTTP save/Quick Start preserving active state on failure. |
| D08 | 02/24/27 | TODO | Shared client configuration | Verify the complete 81+3 field contract, INI/import/URI/QR/form/store/reconnect through real adapters; fuzz/budget and concurrent edits. |
| D09 | 14/15/25/32/33 | IN_PROGRESS | Linux lifecycle and system failures | Run Linux flock/permissions/control/hooks/process-group and worker/services/TUN/route/DNS tests; retain stdout, exit status, SHA and before/after state. Run privileged ignored tests explicitly. |
| D10 | 17/18/19/21/22/23 | TODO | Network integration matrix | Verify off/manual/route/nat66 × NDP, DNS UDP/TCP, multiple profiles, iptables/nft/firewalld, setup rollback/stop/restart and preservation of foreign resources. |
| D11 | 00/24/27/34 | TODO | Current native cores and provenance | Rebuild affected cores from a clean commit using pinned recipes, compare A/B outputs, update copies and genuine provenance; verify ABI/exports and packages. |
| D12 | 24/25/27/34 | BLOCKED | Platform evidence | Android emulator is available in the lab but still needs a run. Windows runtime, Mac/Xcode, iOS and router runtime remain unverified; required environments are not confirmed. Compilation is not a substitute. |
| D13 | 14/19/22/25 | TODO | Resource retention under load | Measure fd/tasks/threads/TUN/routes/firewall/journals/RSS before and after churn/reconnect/stop, including failures and multiple profiles; bounded duration and explicit growth criteria. |
| D14 | 00/34 | TODO | Current benchmark and certification | After correctness, run reproducible benchmarks for required modes with the current SHA, environment and metrics; build certification only from actual results. Historical 0.8.0 results do not certify 0.8.2. |
| D15 | All started sections | IN_PROGRESS | Evidence and documentation reconciliation | Map historical open items to later fixes; verify patch applicability, diff/commit and RU/EN links. Close each debt item with evidence, not a commit count. |

## Closure order

1. D01–D06: code and regressions for confirmed defects; run D09 alongside useful local work.
2. D07–D10: runtime/contracts and failure cases on the corrected snapshot.
3. D11–D13: clean builds, clients/devices and resource measurements.
4. D14–D15: current measurements, package/report reconciliation and final closure evidence.

Privileged external mutation between a check and a write has no atomic protection
guarantee; this is an OS-interface limitation, not automatically a new feature defect.
However, identity loss, command errors and unknown outcomes must retain recovery evidence
and must not produce false success.

## Sources

- [AUDIT-Q01-SERVER-INI](../reports/AUDIT-Q01-SERVER-INI.md)
- [AUDIT-Q02-CLIENT-PARSERS](../reports/AUDIT-Q02-CLIENT-PARSERS.md)
- [AUDIT-Q05-PREFLIGHT](../reports/AUDIT-Q05-PREFLIGHT.md)
- [AUDIT-Q14-CONTROL](../reports/AUDIT-Q14-CONTROL.md)
- [AUDIT-Q14-DNS-OWNERSHIP](../reports/AUDIT-Q14-DNS-OWNERSHIP.md)
- [AUDIT-Q14-H2-TASKS](../reports/AUDIT-Q14-H2-TASKS.md)
- [AUDIT-Q14-HOOKS](../reports/AUDIT-Q14-HOOKS.md)
- [AUDIT-Q14-IPV6-PARTIAL-ACQUIRE](../reports/AUDIT-Q14-IPV6-PARTIAL-ACQUIRE.md)
- [AUDIT-Q14-NAT-CLEANUP](../reports/AUDIT-Q14-NAT-CLEANUP.md)
- [AUDIT-Q14-NAT-COMMANDS](../reports/AUDIT-Q14-NAT-COMMANDS.md)
- [AUDIT-Q14-OWNED-SHUTDOWN](../reports/AUDIT-Q14-OWNED-SHUTDOWN.md)
- [AUDIT-Q14-PROFILE-SHUTDOWN](../reports/AUDIT-Q14-PROFILE-SHUTDOWN.md)
- [AUDIT-Q14-Q15-WORKER-USAGE](../reports/AUDIT-Q14-Q15-WORKER-USAGE.md)
- [AUDIT-Q14-Q19-LIFECYCLE](../reports/AUDIT-Q14-Q19-LIFECYCLE.md)
- [AUDIT-Q14-Q25-FIREWALL-CHECKS](../reports/AUDIT-Q14-Q25-FIREWALL-CHECKS.md)
- [AUDIT-Q14-Q32-NOTIFICATIONS](../reports/AUDIT-Q14-Q32-NOTIFICATIONS.md)
- [AUDIT-Q14-Q33-CONFIG-TRUST](../reports/AUDIT-Q14-Q33-CONFIG-TRUST.md)
- [AUDIT-Q14-SUPERVISOR](../reports/AUDIT-Q14-SUPERVISOR.md)
- [AUDIT-Q14-SYSCTL-RECOVERY](../reports/AUDIT-Q14-SYSCTL-RECOVERY.md)
- [AUDIT-Q19-DNS-CACHE](../reports/AUDIT-Q19-DNS-CACHE.md)
- [AUDIT-Q19-DNS-EDNS](../reports/AUDIT-Q19-DNS-EDNS.md)
- [AUDIT-Q19-DNS-PROXY](../reports/AUDIT-Q19-DNS-PROXY.md)
- [AUDIT-Q19-Q22-NETWORK-PLAN](../reports/AUDIT-Q19-Q22-NETWORK-PLAN.md)
- [AUDIT-Q25-CLIENT-COMMANDS](../reports/AUDIT-Q25-CLIENT-COMMANDS.md)
- [AUDIT-Q25-CLIENT-NAMESPACE](../reports/AUDIT-Q25-CLIENT-NAMESPACE.md)
- [AUDIT-Q25-CORE-LIFECYCLE](../reports/AUDIT-Q25-CORE-LIFECYCLE.md)
- [AUDIT-Q25-CREDENTIAL-COMMANDS](../reports/AUDIT-Q25-CREDENTIAL-COMMANDS.md)
- [AUDIT-Q25-DNS-LEASES](../reports/AUDIT-Q25-DNS-LEASES.md)
- [AUDIT-Q25-DNS-RECOVERY](../reports/AUDIT-Q25-DNS-RECOVERY.md)
- [AUDIT-Q25-EXIT-OWNERSHIP](../reports/AUDIT-Q25-EXIT-OWNERSHIP.md)
- [AUDIT-Q25-GATEWAY-IDENTITY](../reports/AUDIT-Q25-GATEWAY-IDENTITY.md)
- [AUDIT-Q25-GATEWAY-ROLLBACK](../reports/AUDIT-Q25-GATEWAY-ROLLBACK.md)
- [AUDIT-Q25-GATEWAY-WAN](../reports/AUDIT-Q25-GATEWAY-WAN.md)
- [AUDIT-Q25-H2-TASKS](../reports/AUDIT-Q25-H2-TASKS.md)
- [AUDIT-Q25-KILL-SWITCH-LIFETIME](../reports/AUDIT-Q25-KILL-SWITCH-LIFETIME.md)
- [AUDIT-Q25-NETWORK-CLEANUP](../reports/AUDIT-Q25-NETWORK-CLEANUP.md)
- [AUDIT-Q25-PASSWORD-FILES](../reports/AUDIT-Q25-PASSWORD-FILES.md)
- [AUDIT-Q25-PATH-MONITOR](../reports/AUDIT-Q25-PATH-MONITOR.md)
- [AUDIT-Q25-ROUTE-IDENTITY](../reports/AUDIT-Q25-ROUTE-IDENTITY.md)
- [AUDIT-Q25-ROUTE-OUTCOME](../reports/AUDIT-Q25-ROUTE-OUTCOME.md)
- [AUDIT-Q25-ROUTE-OWNERSHIP](../reports/AUDIT-Q25-ROUTE-OWNERSHIP.md)
- [AUDIT-Q25-ROUTE-PENDING](../reports/AUDIT-Q25-ROUTE-PENDING.md)
- [AUDIT-Q25-ROUTE-POSTCONDITIONS](../reports/AUDIT-Q25-ROUTE-POSTCONDITIONS.md)
- [AUDIT-Q25-ROUTE-SCOPE](../reports/AUDIT-Q25-ROUTE-SCOPE.md)
- [AUDIT-Q25-SETUP-FLUSH](../reports/AUDIT-Q25-SETUP-FLUSH.md)
- [AUDIT-Q25-SETUP-IDENTITY](../reports/AUDIT-Q25-SETUP-IDENTITY.md)
- [AUDIT-Q25-SYSCTL-NAMESPACE](../reports/AUDIT-Q25-SYSCTL-NAMESPACE.md)
- [AUDIT-Q25-SYSCTL-OWNER-EVIDENCE](../reports/AUDIT-Q25-SYSCTL-OWNER-EVIDENCE.md)
- [AUDIT-Q25-SYSTEM-COMMANDS](../reports/AUDIT-Q25-SYSTEM-COMMANDS.md)
- [AUDIT-Q25-TCP-TASKS](../reports/AUDIT-Q25-TCP-TASKS.md)
- [AUDIT-Q25-TUN-ADMISSION](../reports/AUDIT-Q25-TUN-ADMISSION.md)
- [AUDIT-Q25-TUN-ATTACH](../reports/AUDIT-Q25-TUN-ATTACH.md)
- [AUDIT-Q25-TUN-CLEANUP](../reports/AUDIT-Q25-TUN-CLEANUP.md)
- [AUDIT-Q25-TUN-LIFETIME](../reports/AUDIT-Q25-TUN-LIFETIME.md)
- [AUDIT-Q25-TUN-WORKERS](../reports/AUDIT-Q25-TUN-WORKERS.md)
- [AUDIT-Q25-TUNNEL-ROUTES](../reports/AUDIT-Q25-TUNNEL-ROUTES.md)
- [AUDIT-Q25-UDP-TASKS](../reports/AUDIT-Q25-UDP-TASKS.md)

D02/D05: [sysctl journal reads and lock waits](../reports/AUDIT-Q25-SYSCTL-JOURNAL-IO.md) are fixed within the stated scope; remaining row criteria are open.

D03: [kill-switch namespace / reconnect](../reports/AUDIT-Q25-KILL-SWITCH-IDENTITY.md).

D02/D06: [namespace-aware link observation](../reports/AUDIT-Q25-LINK-OBSERVATION.md).

D05: [async preflight and panel transaction lifetime](../reports/AUDIT-Q05-PANEL-TRANSACTIONS.md). Overall backup/restore and other network sequence budgets remain open.

D05/D09 update: [backup/restore budget and snapshot completeness](../reports/AUDIT-Q05-ARCHIVE-BUDGET.md). Other network sequences and filesystem fault E2E remain open.
