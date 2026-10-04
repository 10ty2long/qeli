# Q24: multipath, bonding and shared budgets

<!-- normative-sync: audit-q24-bonding-v1 -->

**IN_PROGRESS. Overall plan: 23/37 (62.2%).** 4 October 2026.
[Evidence](../../../release/certification/evidence/q24-bonding-20261004.json).

## Q24-F001: one stalled carrier stopped healthy flows (P2)

The isolated Linux baseline delivered 480/480 UDP echoes across 16 inner flows.
After blackholing one secondary TCP carrier and starting a bounded UDP flood,
only 11 echoes arrived; the last five seconds received zero. Removing the rule
restored traffic without replacing the process or TUN.

The client's 4096-entry queue retained TunPacket values from the small shared pool.
A stalled writer could exhaust it, stopping TUN reads for the other carriers.
The server queue could likewise retain its entire shared 4 MiB wire pool.

The common core now snapshots bonded uplink into one fixed 4 MiB pool. Queues
retain PooledBuffer, immediately releasing native TUN packets, including FIFO
Wintun reservations. Each queue receives a share of the budget, leaving room
for other streams and a temporary resume candidate. There is no fallback allocation;
overload drops the packet. Server queues use the same share calculation.
Single-stream TCP keeps its former path. No new INI option, wire format or public ABI.

## Layer review

Reviewed primary AUTH/server push, bearer and authenticated JOIN proofs, fresh
transcript/epoch, stream caps, atomic admission/revocation, fixed/adaptive actors,
logical-slot scheduling, IPv4/IPv6 fragment affinity, independent directional rate
buckets, shared cover/stealth budgets, quota counters and task/socket release.
Bearer JOIN remains necessary for compatibility; no confirmed dead production
code was found in this pass.

The manual now describes the 1–16 range, fixed targets with retries, adaptive
three-second ramp with no downscaling, negotiated stable slots versus legacy
modulo scheduling, and the fact that one inner flow is not striped across carriers.

## Qualification and remaining work

Before the production fix, fixed/adaptive handover and resume, grace expiry,
and shared 8 Mbps cap/one-carrier reset/revocation passed in fake-tls and plain.
A fixture incorrectly expecting 1/1 instead of the bonded width was corrected.
Persistent 15% loss in REALITY-TLS delayed iperf's final control exchange until
timeout; the new fixture bounds loss duration and keeps separate phase logs.
This outcome is not classified as a confirmed production regression.

Q24-F001 is committed in the audit worktree. Final Linux units/Clippy/release,
bidirectional baseline/fixed, runtime scenarios and native A/B/ABI are still being
qualified. Previous 2405 units and native bytes are not fresh validation of the
changed core. Q24 stays open until these gates finish.

Lab .11 accepts TCP/22 without an SSH banner; no new authentication attempts.
The available .10 host runs sequentially with one compiler job; its working service
and executable are preserved. Mac/iOS/router/Windows VM are user exclusions.
Profiles remain INI-only; service/API JSON remains permitted.
