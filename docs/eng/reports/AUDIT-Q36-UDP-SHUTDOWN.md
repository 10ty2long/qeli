# Q36-F018: UDP shutdown and retained task failures

<!-- normative-sync: q36-udp-shutdown-v1 -->

11 October 2026. Baseline: `2c5fcd5e`. Status: **source fix and regression tests
PASS; full Linux runtime verification OPEN**. Q36 remains IN_PROGRESS.

A private UDP server returned exit 1 on normal SIGTERM during the expanded AUTH
matrix: `UDP receive pump stopped`. ProfileTasks cancels the child receiver first;
its channel can close before listener cancellation takes effect. The previous
handler treated every closed channel as a failure.

The receiver now distinguishes owner shutdown from channel loss in a live profile
generation. The supervisor accepts normal UDP-worker completion during shutdown.
The shared `supervise_udp_workers` joins all remaining workers and records completed
errors and panics with the existing bounded failure collector. Deliberate cancellation
is expected; an empty set and normal completion in a live profile remain failures.
INI, ABI and wire format are unchanged.

## Verification and evidence

- Two negative controls use the exact original source branches. The receive-pump
  owner-cancellation test fails; both worker shutdown/drain tests also fail.
  Adapters only supply the invocation environment.
- 38 exact-source tests pass on Windows (Rust 1.98.0) and Linux `.10` (Rust 1.97.0).
  A small crate includes the full tasks/shutdown modules, exact wait/receive-pump
  functions and tests. This is not the complete Qeli library. Six regression tests
  cover cancellation, live profiles, completed errors/panics and an empty worker set.
- 77 benchmark-tool tests pass on each OS, checking actual native-client injection
  boundaries, proof/AuthOK, PID, carrier mode and SUN_LEN. The private instrumented
  client exits before NetworkPlan application.
- The Linux fixture was archived and its SHA verified before removal. The active
  `.10` service, PIDs and binary SHA were unchanged.
- The full `.10` host gate **FAILS**: two vpn0 FORWARD rules and a 10.8.0.0/24
  MASQUERADE through ens18 appeared between snapshots. These previously observed
  rules were neither hidden by normalization nor removed. Tests did not change firewall.

[Machine-readable evidence](../../../release/audit_q36_udp_shutdown_2026-10-11.json)
contains source/log/archive SHA values, both original-code controls, and complete
before/after host snapshots. Raw evidence remains in
`audit-debt-20260924/q36-native-auth-20261011`.

## Remaining work

The pre-fix AUTH attempt retained 23 successful bursts /152 peers. The next UDP
burst admitted 16 peers but failed shutdown. These measurements will not be pooled
with fixed-server runs or certified as a complete carrier matrix.

An earlier private CLI build contains the first receive-pump fix but predates final
worker-error collection. The full fat-LTO test-library build on `.11` is incomplete;
the guest stopped issuing SSH banners during that build. Memory pressure is suspected,
not a confirmed OOM. Compiler termination and final `.11` state are unverified;
no hypervisor console access was found.

Once access returns: inspect resources and only this packet's processes/cache,
build the current native CLI, run 108 fresh AUTH bursts, and verify cleanup/host state.
TCP asymmetry and Q35 soaks remain separate obligations. No completed full AUTH
result or Q36 closure is claimed. LOCAL_DEV_COMMIT; NO_PUSH_DEPLOY_RELEASE.
