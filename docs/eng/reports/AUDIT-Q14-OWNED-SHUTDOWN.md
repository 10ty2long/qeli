# Q14: final DNS and IPv6 sysctl verification during worker shutdown

Date: 23 September 2026. Baseline commit: `44f19ce8`.
Sections 14, 17, 18 and 19: **IN_PROGRESS**. Q14-F027 is **partially** addressed.

## Confirmed problem

After Q14-F028, exact DNS rule specifications survived a failed Drop; IPv6 sysctl leases
also remained after restoration failures. Yet signal-driven worker shutdown exited 0
when accounting was flushed successfully. Resource errors remained in logs, while
`Server shutdown complete` appeared even after an accounting flush failure.
Concurrent worker and flush failures returned only the first cause.

## Change

After profile supervisors, ordinary tag sweeps and post_down finish,
`nat::finish_owned_cleanup()` retries all pending DNS records and all remaining IPv6
sysctl leases owned by this worker, under the common firewall mutex. A DNS failure
does not skip sysctls; a profile failure does not skip other profiles. Failed cleanup
retains ownership for retry; verified successful cleanup removes the record.

The DNS registry also checks for active leases: none should remain after profiles stop.
An active lease produces `DNS INPUT lease still active at worker shutdown`; its rules
are not deleted by this check. Incomplete ownership is reported without assuming that
an active record is already unused.

`qeli/src/server/shutdown.rs` combines the original worker failure, the final failure
of known network resources and the usage flush failure. Each category is preserved,
with its text bounded to 2048 characters. Accounting is flushed even after network
failure; the exclusive control socket lease remains held until that attempt finishes.

On SIGINT/SIGTERM shutdown the worker exits 1 when the final result is an error.
Non-signal shutdown returns the same Result. `Server shutdown complete` is logged only
for success; otherwise the log says `Server shutdown failed: ...`. A previously observed
transient failure alone does not fail shutdown if final retry confirms release.
Earlier errors remain in logs but do not become a sticky failure flag.

This check deliberately covers **ownership known to the current worker**. Generic NAT
sweeps retain their existing behavior: unavailable `iptables-nft -S` on mixed native nft
does not become an unconditional startup failure. No new INI keys; ABI and wire format
are unchanged.

## Validation

14 new host tests: four DNS registry, four orchestration and six final worker outcome
cases. Coverage includes active/pending entries, permanent failure and successful retry,
retained specs, continuing after failures, all error categories and bounded diagnostics.

**971 host unit + 52 editor/policy + 7 examples + 12 server INI = 1042 Rust tests PASS.**
Linux all-targets Clippy, client-only, client without roaming, server-only, minimal FFI,
compatibility without features and rustfmt PASS. Existing feature-specific warnings
remain; Clippy keeps the prior chunks_exact_to_as_chunks exception for ndp_proxy.

Additionally, eight isolated host-process scenarios use the extracted production
`run_worker` tail, actual `process::exit`, production NAT adapters and portable registry.
Firewall/sysctl, accounting and the control lease are fixtures. No real Linux worker
or signal delivery was run; a parameter selects the signal branch.
Six baseline scenarios expose defects and two controls pass; all eight pass with the fix.
They verify exit 1 for permanent DNS/sysctl failure or an active lease, exit 0 after
successful retry, all error causes and flush before lease release. These scenarios are
not included in 1042. Evidence and logs:
C:/Users/litvi/OneDrive/Documents/qeli/owned-shutdown-audit-20260923.

## Open boundaries

Q14-F027 **remains open**: generic NAT sweep errors, TUN deletion, queue timeouts/panics
and shutdown JoinSet results do not yet reach a complete aggregate outcome. Success of
the new check does not verify those resources. Worker exit also does not establish
correct reporting by the outer supervisor/systemd.

Only in-memory worker leases are checked. A failed partial acquire before IPv6 lease
registration, previous-worker journal records and IPv4 forwarding held for the worker
lifetime are not separately covered by this pass. `sysctl::recover()` can still return
Ok after persisting unresolved stale entries; its contract needs a separate pass.
The DNS registry does not survive crashes/restarts; persistent journaling is absent.

Firewall commands lack an overall deadline; a stalled command can delay final
verification too. Linux runtime, actual firewall/TUN/DNS, devices, native release,
SSH/systemd/Actions and benchmarks were not run.

Previous pass: [retained DNS ownership](AUDIT-Q14-DNS-OWNERSHIP.md).
