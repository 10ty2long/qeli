# Technical debt from started audits

<!-- normative-sync: audit-debt-v14 -->

Reconciled on 25 September 2026. At the user’s request, new full-audit sections
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
| D02 | 14/25 | DONE | Internal sysctl boundaries | Lock/I/O/context, trusted directory, namespace fd pins, original per-interface fd/witness and v4 network_cookie verified. 1995 Linux + 32 privileged + 8 lifecycle and SIGKILL/mismatch worker E2E PASS. Lost interface witness stays for manual recovery; general persistent firewall/DNS/routes remains D04. [Report](../reports/AUDIT-Q25-NAMESPACE-GENERATION.md). |
| D03 | 22/25 | DONE | Standalone kill switch | Pinned namespace, retained exact-family owner, fail-closed reconnect and safe address rotation. 11 portable + 2 native regressions; actual IPv4/IPv6 filter counters and 2 baseline failures. [Report](../reports/AUDIT-Q25-KILL-SWITCH-IDENTITY.md). |
| D04 | 14/19/22/25 | DONE | Crash recovery | Persistent server firewall, DNS v2, kill-switch and physical routes verified; legacy global DNS, persistent TUN and lost sysctl witnesses have explicit safe manual boundaries. [Client mixed matrix](../reports/AUDIT-Q25-CLIENT-MIXED-FIREWALL.md): 152/152 cells, 136 crash/recovery; [server](../reports/AUDIT-Q14-MIXED-FIREWALL.md): 16/16, 476 checks rerun PASS. Arbitrary zones/policies and multiprofile remain D10; state accumulation remains D13. |
| D05 | 05/14/25 | IN_PROGRESS | Whole-operation deadlines and blocking | Async panel preflight/health, backup/restore, command budgets, DNS/NSS, resolver files, NetworkPlan application and graceful TunGuard cleanup are addressed; [kill-switch setup/refresh and terminal firewall cleanup](../reports/AUDIT-Q25-FIREWALL-TASK.md) also run on joined workers. [Startup route/DNS recovery](../reports/AUDIT-Q25-STARTUP-RECOVERY-TASK.md) also preserves its lease and late errors on a joined worker. Remaining: early error/Drop paths, other internal locks/I/O and whole NetworkPlan/shutdown deadlines. Forced Drop may synchronously join workers. [TUN pump startup and early rollback](../reports/AUDIT-Q25-PUMP-START.md) now use a joined worker; pure data-plane checks run before TUN setup. [Client diagnostics](../reports/AUDIT-Q25-STATUS-WRITER.md) now use a bounded queue and joined writer; other I/O and the overall deadline remain. [Device-id/TOFU](../reports/AUDIT-Q25-IDENTITY-FILES.md): ID is cached and loaded on a joined worker; bounded reads and atomic TOFU are fixed, its synchronous callback remains. |
| D06 | 15/21/22/23/25 | IN_PROGRESS | External network-resource context | Verify WAN identity, resolved/bus context, sysfs/procfs and attach/name contracts; process-global DNS/carrier state and dynamic IPv6. Document supported combinations. [Q15-F002](../reports/AUDIT-Q15-UDP-LOCAL-ADDRESS.md) closes multi-IP wildcard UDP: the local endpoint survives receive/reply/roaming/PMTU; other D06 criteria remain open. |
| D07 | 01/05/09/11 | TODO | Server configuration at runtime | Trace field → parse/validate/runtime/serialize; malformed/oversized input; check-config/startup/SIGHUP/HTTP save/Quick Start preserving active state on failure. |
| D08 | 02/24/27 | IN_PROGRESS | Shared client configuration | Verify the complete 81+3 field contract, INI/import/URI/QR/form/store/reconnect through real adapters; fuzz/budget and concurrent edits. |
| D09 | 14/15/25/32/33 | IN_PROGRESS | Linux lifecycle and system failures | Run Linux flock/permissions/control/hooks/process-group and worker/services/TUN/route/DNS tests; retain stdout, exit status, SHA and before/after state. Run privileged ignored tests explicitly. |
| D10 | 17/18/19/21/22/23 | IN_PROGRESS | Network integration matrix | Verify off/manual/route/nat66 × NDP, DNS UDP/TCP, multiple profiles, iptables/nft/firewalld, setup rollback/stop/restart and preservation of foreign resources. |
| D11 | 00/24/27/34 | IN_PROGRESS | Current native cores and provenance | Rebuild affected cores from a clean commit using pinned recipes, compare A/B outputs, update copies and genuine provenance; verify ABI/exports and packages. |
| D12 | 24/25/27/34 | IN_PROGRESS | Platform evidence | Android: 154 JVM + 6 API 34/x86_64 instrumentation tests PASS with fresh JNI; final snapshot remains required. Windows VM, Mac/Xcode/iOS and router runtime **SKIPPED by user decision on 24 September 2026**: those environments will not be provided. These platforms are not certified; this is a scope exclusion, not PASS. |
| D13 | 14/19/22/25 | IN_PROGRESS | Resource retention under load | Measure fd/tasks/threads/TUN/routes/firewall/journals/RSS before and after churn/reconnect/stop, including failures and multiple profiles; bounded duration and explicit growth criteria. |
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

D05: [async preflight and panel transaction lifetime](../reports/AUDIT-Q05-PANEL-TRANSACTIONS.md). The following phase closed the backup/restore budget; other network sequence budgets remain open.

D05/D09 update: [backup/restore budget and snapshot completeness](../reports/AUDIT-Q05-ARCHIVE-BUDGET.md). Other network sequences and filesystem fault E2E remain open.

D02: [internal sysctl boundary guard](../reports/AUDIT-Q25-SYSCTL-CONTEXT-IO.md); durable namespace identity and original-interface ownership remain open; a later phase closed parent trust.

D02/D05/D09: [atomic state publication](../reports/AUDIT-Q25-ATOMIC-STATE.md) cleans partial temporary files and syncs the directory on Unix; actual partial-write/fsync fault probes PASS. Other criteria of these groups remain open.

D02/D05/D09: [state-directory and lock identity](../reports/AUDIT-Q25-STATE-DIRECTORY.md). Parent trust is closed within the stated boundaries; durable namespace identity and original-interface generation remain D02. The groups are not yet fully closed.

D08/D11/D12: [Android JNI and emulator runtime](../reports/AUDIT-Q34-ANDROID-RUNTIME.md): fixed cargo-ndk cwd/API flag, removed the obsolete JSON-config harness; 154 JVM + 6 instrumentation tests PASS. The fresh dev x86_64 APK is SHA-verified. Release A/B, the full config/runtime contract and other platforms remain open.

D02: [namespace pins](../reports/AUDIT-Q25-NAMESPACE-PIN.md) retain open fds from admission through the end of the transaction; 1922 Linux + 29 privileged + 8 worker E2E PASS. Durable generation between transactions and after crashes remains open, as does original-interface generation.

D02: [Q25-F083 — original interface sysctl](../reports/AUDIT-Q25-SYSCTL-TARGET.md): journal v3 retains fds and refuses when evidence is lost; 3 baseline defects reproduced, 5 additional worker E2E PASS. Unsafe name-based restoration and loss of originals are closed. Durable namespace generation after crashes remains open for global journals; automatic per-interface crash recovery is not promised.

D05: [Q25-F084 — shared DNS deadline](../reports/AUDIT-Q25-DNS-BUDGET.md): dns/domain share 15 seconds from admission, retaining a partial lease for separate rollback. 3 new Linux regressions PASS. Kill-switch is addressed by the following phases below; NAT/routes and other lock waits remain open.

D05/D09: [Q05-F008 — async health probes](../reports/AUDIT-Q05-HEALTH-PROBES.md): Status/Transport health do not block the executor waiting for `--version`; four shared async slots and a deadline including queue time. 8 new ordinary + 1 privileged HTTP-router test PASS; 2 counterfactual old-behavior checks fail as expected. Whole network mutation budgets and full HTTP/systemd/fault coverage remain open.

D05/D09: [Q25-F085 — shared kill-switch cleanup deadline](../reports/AUDIT-Q25-KILL-SWITCH-BUDGET.md): 15 seconds include the operation mutex and both families; partial outcomes retain ownership for verified retry. 5 regressions PASS, 2 counterfactual old-behavior FAIL. Engage/refresh are addressed by the following phases below; NAT/routes/gateway and other lock waits remain open.

D05/D09: [Q25-F086/F087 — kill-switch refresh](../reports/AUDIT-Q25-KILL-SWITCH-REFRESH.md): shared admission/command deadline accounts for resolver time; unknown cannot authorize insertion. 6 new regressions and 5 cleanup reruns PASS; 3 counterfactual old-behavior FAIL. DNS/NSS remains synchronous; the following phase below addresses the engage command budget, while NAT/routes/gateway and other waits remain open.

D05/D09: [Q25-F088/F089 — kill-switch setup and rollback](../reports/AUDIT-Q25-KILL-SWITCH-SETUP.md): shared 15-second setup deadline plus a separate shared 15-second rollback deadline; leak overrides cannot accept incomplete rollback. 8 new regressions PASS; 2 counterfactual FAIL, then 8 setup + 6 refresh + 5 cleanup PASS. Full snapshot: 1962 Linux + 30 privileged + 8 worker E2E PASS. Engage/refresh/disengage command budgets are addressed within the stated limits; synchronous DNS/NSS, NAT/routes/gateway, other waits and full D04/D05/D09 remain open.

D05/D09: [Q14-F034 — shared NAT cleanup deadline](../reports/AUDIT-Q14-NAT-CLEANUP-BUDGET.md): profile/startup/final cleanup each share 15 seconds across admission, IPv4/IPv6, exact rules and retired DNS UDP/TCP. Late results cannot succeed; unverified records remain owned. 8 new Linux regressions PASS; 4 counterfactual FAIL, then 8 regressions + 1 privileged exact-rule PASS. Full snapshot: 1970 Linux + 30 privileged + 8 worker E2E PASS. NAT setup/rollback, DNS lease Drop/setup admission, routes/gateway, internal sysctl/I/O and full D04/D05/D09 remain open.

D05/D09: [Q14-F035 — DNS INPUT lease deadlines and retirement](../reports/AUDIT-Q14-DNS-INPUT-BUDGET.md): setup and cleanup each receive 15 seconds including queue/UDP/TCP; separate rollback after setup, retirement without lock admission and retained pending evidence. 4 new portable + 7 Linux + 1 privileged regressions PASS; 5 negative controls, then 20 domain + 7 DNS + 8 NAT + 2 native PASS. Full snapshot: 1981 Linux + 31 privileged + 8 worker E2E PASS. NAT setup/rollback, DNS REDIRECT, routes/gateway, internal sysctl/I/O and full D04/D05/D09 remain open.

D05/D09: [Q14-F036 — NAT/forwarding and DNS REDIRECT setup](../reports/AUDIT-Q14-NAT-SETUP-BUDGET.md): shared setup and exact rollback deadlines; 10 new regressions, 7 negative controls, 1991 Linux + 31 privileged + 8 E2E PASS. Client routes/gateway, scheduler isolation and full D05 remain open.

D02 closed: [Q25-F090 — namespace generation and journal v4](../reports/AUDIT-Q25-NAMESPACE-GENERATION.md). D04/D05 and other criteria remain. Windows VM, Mac/iOS and router tests are excluded from current scope by user decision, not declared PASS.

D09/D10: [17/17 Linux packet matrix PASS](../reports/AUDIT-Q34-LINUX-MATRIX.md). D13: 100 TCP handovers preserved session/fd but exceeded the RSS criterion; failure retained, debt open.

D05/D09: [Q25-F091 — shared gateway/exit-node deadline](../reports/AUDIT-Q25-GATEWAY-BUDGET.md): 6 regressions, 6 counterfactual failures, 107 restored gateway and full Linux 2001 + 32 privileged + 8 E2E PASS. Route sequences, internal locks/I/O and scheduler isolation remain open.

D13: [100 release TCP/UDP handovers each](../reports/AUDIT-Q34-RELEASE-SOAK.md): 30/30 assertions PASS, RSS growth within the unchanged 32 MiB limit; debug FAIL retained. Full resource/fault coverage and final-source measurements remain open.

D05/D09: [Q25-F092 — shared route transaction deadline](../reports/AUDIT-Q25-ROUTE-BUDGET.md): 8 regressions, 6 counterfactual FAILs, 196 restored route tests and full Linux 2009 + 32 privileged + 8 E2E PASS. Executor isolation and internal I/O remain open.

D06/D09: [Q25-F093 — strict resolver configuration check](../reports/AUDIT-Q25-RESOLVER-CONFIG.md): 3 new tests, 2 counterfactual FAILs, 18 restored DNS, full Linux 2012 + 32 privileged + 8 E2E PASS. A separate probe confirmed cross-netns mutation through a shared D-Bus; bus/service identity still needs a fix.

D06/D09/D10: [Q25-F094/F095 — resolved/D-Bus context and DNS ports](../reports/AUDIT-Q25-RESOLVER-CONTEXT.md): direct unique-owner calls with AUTH GUID, 2023 Linux + 33 privileged + 8 E2E, 17/17 packet matrix (301 assertions), 4 counterfactual FAILs and restored 29 DNS + 1 privileged PASS. Real resolved/custom ports/foreign netns and PID namespace checked. The previous open bus/service identity item is closed within these boundaries; other D06 and D10 criteria remain open.

D04/D06/D09: [Q14-F037 — worker network lease](../reports/AUDIT-Q14-WORKER-NETWORK-LEASE.md): different control/state paths can no longer bypass admission; baseline deleted 9 live-worker rules. 2027 Linux + 34 privileged + 8 lifecycle and 22 crash/admission/recovery checks PASS. D04 IN_PROGRESS: persistent exact firewall/routes and mixed nft remain open.

D04/D09/D10: [Q14-F038 — persistent server firewall](../reports/AUDIT-Q14-FIREWALL-JOURNAL.md): exact NAT/routing/DNS INPUT/REDIRECT specifications are written before mutation and recovered after SIGKILL/deleted profiles without listing. Backend/namespace/file errors abort startup and retain evidence. 2044 Linux + 35 privileged + 8 lifecycle; 27 recovery checks; 17/17 cases, 301 assertions PASS. D04 remains IN_PROGRESS: client route/DNS/kill-switch and the full mixed nft/firewalld matrix remain open.

D04/D06/D09: [Q25-F096/F097 — DNS state v2](../reports/AUDIT-Q25-DNS-MARKER-STORAGE.md): trusted held directory, validated files and SO_NETNS_COOKIE; v1 remains without automatic migration. Three file defects reproduced on baseline. 2055 Linux + 37 privileged + 8 lifecycle; 17/17 cases, 323 assertions PASS. Client route/kill-switch recovery, legacy global DNS, live persistent TUN, mixed nft/firewalld and sidecar accumulation remain open; D04 IN_PROGRESS.

D04/D09/D10: [Q25-F098 — kill-switch after crash](../reports/AUDIT-Q25-KILL-SWITCH-REBUILD.md): exact temporary DROP guards retain the prior barrier during setup/rollback and repeated SIGKILL. Real-client baseline passed 14 IPv4 + 13 IPv6 UDP probes; fixed passed 0. 2057 Linux + 39 privileged + 8 lifecycle; 14 runtime checks; 17/17 cases, 339 assertions PASS. D04 IN_PROGRESS: routes, legacy global DNS/persistent TUN and the full mixed firewall matrix remain open.

D04/D09: [Q25-F099 — route ownership attributes](../reports/AUDIT-Q25-ROUTE-ATTRIBUTES.md): usability is separate from delete/replace authority; implicit protocol/metric/source and extra attributes are checked. Baseline deleted 10 operator replacements on the kernel and a real client static bypass. 2068 Linux + 40 privileged + 8 lifecycle; 17/17 cases, 384 assertions PASS. At that stage persistent client route journaling was not implemented; see Q25-F100 below. D04 IN_PROGRESS.

D04/D09: [Q25-F100 — durable physical route journal](../reports/AUDIT-Q25-ROUTE-JOURNAL.md): intent/confirmed ownership, boot/cookie/TUN scope, shared lock and terminal roaming on I/O failure. Baseline left bypass/blackhole after SIGKILL → reconnect → stop (3 FAIL). 2087 Linux + 43 privileged + 8 lifecycle; 17/17 cases, 489 assertions PASS. Physical client route recovery is closed within the stated scope; legacy global DNS, live persistent TUN and mixed firewall keep D04 IN_PROGRESS.

D04/D05/D09: [Q25-F101 — legacy global DNS](../reports/AUDIT-Q25-LEGACY-DNS.md): unsafe automatic replay and PID refcount removed; snapshot/holders require manual recovery without reading contents or waiting on locks. Baseline changed the resolver in 4 cases; new contract 47/47 PASS. 2078 Linux + 43 privileged + 8 lifecycle; 17/17 cases, 489 assertions PASS. Legacy global recovery closed by safe refusal; live persistent TUN and mixed firewall keep D04 IN_PROGRESS.

D04/D09: [persistent TUN/TAP after SIGKILL](../reports/AUDIT-Q25-PERSISTENT-TUN.md): safe refusal preserves interface/routes/DNS/firewall; recovery succeeds after explicit removal of a verified orphan. 17/17 rows, 506 main assertions; 17 persistent scenarios with 199 detailed checks PASS. No new defect; Rust unchanged. This portion of D04 is closed within the stated boundary; D04 IN_PROGRESS for the full mixed firewall matrix.

D04/D09/D10: [server mixed nft/legacy/firewalld recovery](../reports/AUDIT-Q14-MIXED-FIREWALL.md): 16/16 scenarios, 476 checks PASS. Actual per-family backend switches, native nft parse errors, firewalld reload, partial cleanup and manual recovery verified. Unknown absence retains evidence; the lost WAN sysctl witness remains the D02 manual boundary. Rust unchanged. D04 IN_PROGRESS: mixed firewall client kill-switch/DNS/routes packet recovery is still required.

D04 closed: [Q25-F102 and client mixed firewall matrix](../reports/AUDIT-Q25-CLIENT-MIXED-FIREWALL.md): 152/152 network cells, 136 SIGKILL/recovery cases, 4880 main assertions and 3224 nested checks PASS. All 4352 direct UDP attempts under protection were blocked; 2624 allowed probes received replies. The shared classifier now handles exact legacy advice; server 16/16, 476 checks reran PASS. Persistent TUN/sysctl/legacy DNS manual boundaries remain. **Debt total: 4/15 DONE (26.7%), 9 IN_PROGRESS, 2 TODO.** This does not complete full-audit sections; D05/D06 and D09/D10/D13 remain open.

D05/D09: [Q25-F103 — shared DNS/NSS and shutdown](../reports/AUDIT-Q25-SYSTEM-RESOLVER.md): four unfinished calls, queue-inclusive deadline, retained capacity after cancellation and no blocking-pool shutdown wait. 4 baseline failures and 4 fixed PASS; 38/38 hostname cells, 34 crash/recovery, 1220 main and 806 nested checks PASS. 1537 host + 71 config; 2089 Linux + 44 privileged + 8 lifecycle PASS. D05 stays IN_PROGRESS: network-mutation scheduler isolation, internal locks/I/O and whole NetworkPlan/shutdown. **Debt: 4/15 DONE (26.7%), 9 IN_PROGRESS, 2 TODO.**

D05/D09: [Q25-F104 — resolver files before firewall](../reports/AUDIT-Q25-RESOLVER-FILES.md): one bounded snapshot, shared reader/parser with stub checks, exact keyword, comments and retained scope. 7 baseline/fixed pairs; 2 old hangs, all 7 fixed runs clean. 38/38 cells, 34 crash/recovery, 1220 main and 806 nested checks PASS. 1544 host + 71 config; 2098 Linux + 44 privileged + 8 lifecycle PASS. D05 stays IN_PROGRESS: network mutations, internal locks/I/O and whole NetworkPlan/shutdown. **Debt: 4/15 DONE (26.7%), 9 IN_PROGRESS, 2 TODO.**

D05/D09: [Q25-F105 — applying NetworkPlan outside the async executor](../reports/AUDIT-Q25-NETWORK-TASK.md): a fresh thread retains ownership and the original NET/mount context until adoption or completed rollback; native ACK uses async sleep. 7 new portable + 1 Linux + 1 privileged regressions; synchronous helper counterfactual FAIL, fixed PASS. Real TCP/UDP × TUN-create/ip-up: 4/4 PASS with a responsive current-thread runtime, joined rollback and exact network restoration. 38/38 cells, 34 crash/recovery, 1220 main and 806 nested checks; 1551 host + 71 config; 2106 Linux + 45 privileged + 8 lifecycle PASS. D05 remains IN_PROGRESS: established-tunnel teardown, kill-switch mutations, locks/I/O/diagnostics and whole NetworkPlan/shutdown deadlines. Forced Drop may block until join. **Debt: 4/15 DONE (26.7%), 9 IN_PROGRESS, 2 TODO.**

D05/D09: [Q25-F106 — graceful teardown of established tunnels](../reports/AUDIT-Q25-TUN-TEARDOWN.md): shared TCP/UDP TunGuard shutdown preserves DNS → pump join → routes/forwarding and runs network cleanup on one joined worker. Worker failures enter sticky Failures. 5 new portable + 1 privileged test; baseline TCP/UDP produced 0 heartbeat ticks during held cleanup, fixed produced 7–8. 4/4 fixed scenarios and 2/2 explicit restarts after faults PASS; kill-switch retained on failure. 38/38 cells, 34 crash/recovery, 1220 main and 806 nested checks; 1556 host + 71 config; 2111 Linux + 46 privileged + 8 lifecycle PASS. D05 remains IN_PROGRESS: early error/Drop paths, kill-switch mutations, locks/I/O/diagnostics and whole NetworkPlan/shutdown deadlines. **Debt: 4/15 DONE (26.7%), 9 IN_PROGRESS, 2 TODO.**

D05/D09: [Q25-F107/F108 — firewall workers and retained DROP](../reports/AUDIT-Q25-FIREWALL-TASK.md): setup/refresh and all six terminal cleanup paths await owned workers; unconfirmed unhook forbids chain flushing. 8 new regressions, 8 baseline + 12 fixed runtime scenarios; baseline passed 6 UDP probes after cleanup failure, fixed passed 0. 38/38 cells, 34 crash/recovery, 1220 main + 806 nested checks; 1564 host + 71 config; 2119 Linux + 46 privileged + 8 lifecycle PASS. Original wildcard UDP recovery FAIL retained: explicit-bind rerun passed; multi-IP wildcard remains D06/D10. D05 remains open for startup recovery, early Drop, locks/I/O/diagnostics and whole deadlines. **Debt: 4/15 DONE (26.7%), 9 IN_PROGRESS, 2 TODO.**

D06/D09/D10: [Q15-F002 — wildcard UDP reply address](../reports/AUDIT-Q15-UDP-LOCAL-ADDRESS.md): pktinfo survives receive through immutable reply/roaming/PMTU paths. Baseline secondary IPv4 connections fail with 96 wrong-source replies; fixed has 8/8 IPv4/IPv6 × plain/obfs connections, 256/256 inner UDP echoes and original refresh-fault recovery PASS. 6 new Linux + 1 privileged regression; 1564 host + 71 config; 2125 Linux + 47 privileged + 8 lifecycle; 38/38 cells, 34 crash/recovery, 1220 + 806 checks PASS. Wildcard defect closed within report boundaries; overall D06/D10 and D05 remain open. **Debt: 4/15 DONE (26.7%), 9 IN_PROGRESS, 2 TODO.**

D05/D09: [Q25-F111 — ordered diagnostics writer](../reports/AUDIT-Q25-STATUS-WRITER.md): one thread, one pending snapshot, joined final write; file I/O outside the async executor. 5 new portable + 1 privileged test; 2 baseline + 4 fixed fsync cases, 2 baseline + 4 fixed TCP/UDP teardown and 2 recoveries PASS. 1569 host + 71 config; 2133 Linux + 48 privileged + 8 lifecycle PASS. D05 remains open: other locks/I/O, early Drop and the overall deadline. **Debt: 4/15 DONE (26.7%), 9 IN_PROGRESS, 2 TODO.**

D05/D09: [Q25-F112/F113 — identity files](../reports/AUDIT-Q25-IDENTITY-FILES.md): ID loads once on a joined worker and temporary identity survives reconnect; TOFU is capped at 1 MiB, corrupt/conflicting pins reject new trust, writes are atomic. 15 new tests; 8 baseline + 8 fixed identity cases, 6 teardown cases and 2 recoveries PASS. 1575 host + 71 config; 2148 Linux + 48 privileged + 8 lifecycle PASS. D05 remains open: synchronous TOFU, other startup I/O/Drop and overall deadlines. **Debt: 4/15 DONE (26.7%), 9 IN_PROGRESS, 2 TODO.**
