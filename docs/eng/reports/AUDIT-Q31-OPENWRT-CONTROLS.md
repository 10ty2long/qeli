# Q31: LuCI control, INI publication and firewall failures

<!-- normative-sync: q31-openwrt-controls-v34 -->

6 October 2026. F307–F309 product fixes and F310 test reconciliation. Q31 moves
from TODO to IN_PROGRESS; no full checklist criterion or router runtime is closed.
Overall28/37(75.7%),9 remain. Q29 SIGKILL FAIL/auto-null ENONET, Q30 Apple runtime
exclusions and callback-drain OPEN, and D06 ACCEPTED_LIMITATION remain unchanged.

## F307: service commands ran before committed UCI intent

Connect/Disconnect staged `enabled` with `uci.save()`, issued service commands,
then invoked the modal apply helper. Session-local staged settings are not the
committed configuration read by the root init script. A disabled profile could
therefore ignore Connect; simultaneous controls could interleave an old start
with a new stop. The modal helper is not an awaitable commit barrier.

All three controls now share a promise queue. Connect/Disconnect save and await
`uci.apply()` commit/confirmation before enable/start or stop/disable. An apply/save
failure issues no later service commands; a failed operation does not poison the
queue. Restart waits its turn. This serializes controls in one loaded view only,
not independent tabs/RPC clients or unrelated form saves. Service failure after
commit does not roll back the saved autostart intent; the error is propagated.
Like the previous apply action, UCI apply can include other staged packages.
Unsaved form fields still require Save & Apply before the status controls.

The distinction and awaitable API were checked against primary LuCI sources:
[UCI save/apply](https://openwrt.github.io/luci/jsapi/uci.js.html) and
[modal changes.apply](https://openwrt.github.io/luci/jsapi/ui.js.html).
Seven Node fixtures execute the actual module with controlled UCI/RPC promises:
commit ordering, stop intent, save/apply failure, queue recovery, overlapping three
controls, failed enable and invalid action. They do not execute real rpcd/procd.

## F308: firewall synchronization reported success after failed mutation

The live init path discarded delete/add-list/set/commit/reload errors. Even after
adding error propagation, a retry can see already-staged UCI values and incorrectly
skip commit/reload. The root-only tmpfs `firewall-pending` marker is created before
mutation, retained on failure and cleared only after successful commit/reload.
A retry reconciles the current desired device and repeats pending commit/reload.
A clean no-op still performs neither commit nor reload. Init returns before procd
instance admission on a synchronization error. Persistent state-directory creation
failure is propagated as well. No existing firewall service was invoked by tests.

This is not a firewall transaction or rollback: partially staged changes can remain
for the retry, and other administrators/independent init invocations are not locked.
The one-shot install defaults and actual fw4 lifecycle require the next Q31 pass.

## F309: incomplete INI could replace the previous configuration

Rendering truncated the live file and ignored earlier I/O failures. Invalid UCI MTU
could disappear instead of failing (including malformed text), while out-of-range
values reached the core. Rendering now uses a0600 sibling tmpfs file, checks writes
and publishes by atomic rename; ordinary failure removes the temporary file and
preserves the previous INI. MTU must be decimal0(auto) or576..16602, matching the
core/LuCI range. Process kill during rendering may leave a0600 temporary file until
reboot; cleanup after SIGKILL and concurrent lifecycle ownership are not qualified.

Attacker-influenced values use printf with fixed formats. Literal backslash escapes
remain literal; raw C0/DEL controls are stripped from nonsecret INI values. CLI
secrets reject DEL too, matching rpcd validation. BusyBox baseline preserved literal
backslashes already; this is not a claim of a reproduced OpenWrt escape injection.
The printf change also closes echo interpretation on shells such as dash.
Passwords remain separate0600 tmpfs files; no JSON configuration format is added.

Ten Python fixtures source the actual init functions with mock UCI and a mock
firewall executable, each under a fresh temporary directory. Current BusyBox ash
and dash both PASS: render/DNS/router guard, safe value output, malformed/boundary
MTU, earlier write/publish failure, exact4096-byte secret and invalid secrets,
legacy migration no-op, firewall no-op, five mutation/reload failure-and-retry paths,
and reserved device namespace. Baseline BusyBox failures were retained separately.
Actual OpenWrt UCI parsing, RPC secret stdin and real firewall/network behavior are
not replaced by these fixtures. No /etc, running service, TUN or management route
was changed; tests ran only in an isolated `/var/tmp` directory on lab .11.

## F310: a stale Android source assertion broke the recipe gate

The37-test native recipe suite still expected the former nullable username chain.
At the packet base, Android already captures `activeConfig` once, checks stopping/
null/current core under the service monitor, then logs `logValue(config.username)`.
Both source and test matched the base before correction, proving this failure
predates the Q31 patch. The assertion now checks the current capture/sanitizer;
Android product code is unchanged. All37 recipe tests PASS. This source assertion
is not an Android runtime logging test or a new Android bug fix.

## Scope and remaining work

Reviewed initial source paths: init renderer/live firewall sync, LuCI controls,
package ACL/rpcd secret transport, package defaults and router build entrypoints.
The ACL remains scoped to qeli UCI and package-owned controls; service list is read
only. RPC secrets use stdin, not argv. No additional deletion was justified in
this packet. Broader source/dead-code review is still open.

Next: install/upgrade failure and rollback, cached UI status and unsaved form
ownership, cross-process secret/lifecycle races, build-helper failure/provenance
and available cross-build checks. Current artifacts were not rebuilt; Rust/native/
Android/Apple/Windows client implementation inputs are unchanged. Actual router
WAN renewal/reboot, memory/throughput and network runtime are USER_EXCLUDED;
cross-build results will not be presented as device qualification.

Documentation/bindings/diff checks accompany the fixture results. Prior automated
Linux case statuses/artifacts/execution dates/acceptance_basis stay intact; the
source digest is rebound with this scoped qualification supplement only.

Raw packet: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-openwrt-controls-20261006.
Evidence: release/certification/evidence/q31-openwrt-controls-20261006.json.

## F311: preparation errors and manifest recovery were outside cleanup

Router toolchain commands hid rustup failures behind tail/true; Zig exit status was
also ignored. Both helpers now check each command and the installed cargo-zigbuild
0.23.0 inventory. Only selected MIPS builds prepare nightly/rust-src; other selected
targets do not install an unused nightly. Cargo builds use --locked. Unknown arch,
extra argument and unknown flag are rejected before SSH rather than building all
targets after a typo. These fixes do not pin the rolling nightly/stable toolchain
or qualify reproducibility, ABI or current MIPS target support.

The entire connected lifetime now has a finally-close barrier. An interrupted
manifest backup is recovered before --sync, preventing a later restore from
replacing the newly uploaded Cargo.toml with the previous version. A successfully
restricted manifest is restored after build/transfer, and connection close still
runs when restoration fails. SFTP sync closes on put/inventory failure, propagates
cleanup/mkdir errors and requires Cargo.toml/Cargo.lock. This does not make source
sync atomic or exact: deleted remote modules and .cargo/config.toml reconciliation,
concurrent use of the shared checkout and full source provenance remain OPEN.

## F312: a failed download could still print KEENETIC_BUILD PASS

The Keenetic pull helper swallowed IOError and left build result zero. The original
main path was reproduced with a failed SFTP download: exit0 and PASS. Preparation
failure also left its SSH connection open; toolchain errors returned normally.
The corrected main returns1 on transfer failure, closes the connection and checks
setup failure. Both helpers reuse native_lab's verified atomic transfer: SHA256
is read over strict SSH, downloaded bytes must match before destination replacement,
and the empty digest is rejected before transfer/publication. Read failure, corrupt
payload and empty artifact preserve the previous local file. An architecture failure
is reported while the other selected architectures still run. This is transport
identity, not ELF/ABI/device/reproducibility attestation. No real binary was built
or published in this packet.

Fourteen executable Python tests load both real helpers with fake SSH/SFTP/toolchain
responses, exercising invalid CLI, toolchain/pin failures, non-MIPS admission,
manifest/sync order, cleanup failures, per-arch result propagation, verified artifact
publication and lockfile build commands. Current tests PASS; baseline reproduction
records are separate. Mock responses are not a successful real toolchain install.

## F313: repeated LuCI load read the same cached enabled flag

LuCI uci.load caches the package. Polling it again never refreshed external edits;
unloading it would discard staged form changes. A scoped read-only service_status
RPC now calls the package's status_enabled init command. That command reads the same
root UCI context used by service start, rather than the browser/session form cache.
Root CLI deltas may still be visible before commit; this is init-visible intent, not
a claim of a disk-only committed read. No form cache is loaded/unloaded by polling.
Exit0/1 mean enabled/disabled; load error, missing service or invalid RPC response
become unknown, displayed explicitly. The read ACL permits service_status only;
service actions and secret writes remain in the write ACL. Service process liveness
continues to come from procd and is not an authenticated connectivity measurement.

The9 LuCI caller fixtures plus one status/ACL adapter fixture PASS. The latter parses
the compatible adapter in JavaScript with mocked system/fs, not an ucode interpreter;
real ucode/rpcd and rc.common dispatch are NOT_RUN. The added init status case and
prior shell cases make11 tests per BusyBox ash/dash, both PASS in the isolated .11
directory. Official sources: [LuCI cache semantics](https://openwrt.github.io/luci/jsapi/uci.js.html),
[ucode system exit codes](https://ucode-lang.org/module-core.html).

Current packet, F311–F313: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-router-build-failures-20261006.
Evidence: release/certification/evidence/q31-router-build-failures-20261006.json.
The previous packet/seal is unchanged. Install defaults/upgrade rollback, broader
source review, secret/lifecycle races and real available cross-build qualification
remain OPEN. The next installation pass must exercise interrupted creation and
reload errors; the one-shot defaults have not been qualified by the live-sync tests.
Q31 remains IN_PROGRESS, overall28/37(75.7%),9 remain.


## F314: interrupted firewall defaults could report success

The first-install defaults ignored individual UCI and reload errors, then exited0.
If only the qeli zone name had been created, a later run saw that name and skipped
repair. Baseline fixtures reproduce exit0 after commit/reload/input failures;
commit failure leaves no persisted zone, reload is not retried, and an input failure
leaves an incomplete zone. These are shell/UCI-model reproductions, not libuci/fw4.

Defaults now check load, each mutation, commit and live-service reload. They create
named package-owned zone/forwarding sections and a0600 pending marker in a0700 tmpfs
runtime directory before mutation. Retry completes only those owned sections and
replaces the device list without duplicates. Existing administrator zones remain
unchanged; reserved-name collision, foreign pending ownership, multiple qeli zones
and invalid devices fail before mutation. Image preparation without a firewall
service remains supported. While firewall-install-pending exists, init refuses live
sync/start and tells the administrator to rerun /etc/uci-defaults/99-qeli-firewall.

This is recoverable installation, not automatic rollback or an interprocess
transaction. Root CLI staged deltas, concurrent changes, process death and reboot
are not qualified by the model. The temporary marker does not persist across boot;
normal failed mutations remain visible and must be corrected/retried. Nine new
installation cases plus12 init cases PASS in both BusyBox ash and dash on lab .11.
All test paths/services are redirected into temporary fixture directories; no live
/etc configuration, firewall or service is changed.

## F315: --sync retained deleted sources and omitted Cargo config

Both router helpers only removed src/bin and uploaded the remaining files additively.
A deleted module survived and the old .cargo/config.toml continued to influence the
build. Both baseline helper fixtures reproduce these stale inputs. A shared
router_source wrapper now delegates to the existing native_lab sync owner: replace
src and .cargo, upload current config/assets/sources/Cargo.toml/Cargo.lock, preserve
target cache. Mandatory local inputs are checked before remote mutation.

A0600 .router-sync-incomplete marker is written before replacement and removed only
after all uploads and SFTP close succeed. Both helpers check it before toolchain/
build admission, including a later invocation without --sync. Transfer/open/close/
remote-command failures retain it; successful --sync repairs the managed inputs.
Seven real-shell/filesystem adapter tests PASS in isolated Linux temporary roots,
14 router helper fault tests and12 unchanged shared native-lab tests PASS. Repeated
sync removes stale inputs and preserves target cache; missing local files cause no
remote commands. Baseline proof uses a bounded adapter for its historical hardcoded
cleanup, never the live /opt checkout.

This replacement is not atomic and has no cross-process lock. Without --sync, an
existing unmarked checkout is still intentionally reused. No claim is made for
concurrent users, mutable local snapshots, complete provenance, reproducibility,
ELF/ABI or a fresh real cross-build. Cargo config scope is the current single
.cargo/config.toml. A failed initial baseline attempt due to absent Paramiko in the
Linux fixture is retained; source-only imports were corrected with a connection-
forbidden lab stub before the successful reproduction, without installing packages.

Current packet F314–F315: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-install-source-sync-20261006.
Evidence: release/certification/evidence/q31-install-source-sync-20261006.json.
37 native recipe checks,10 Node caller/adapter cases and docs/bindings/diff checks
also PASS. Real libuci/fw4/procd/ucode/rpcd/package install and cross-build NOT_RUN;
router runtime USER_EXCLUDED. Earlier seals and runtime statuses/artifacts/dates/
acceptance_basis are retained. Full upgrade/rollback, cross-process lifecycle/secret
races, broader source review and actual build qualification remain OPEN.
Q31 IN_PROGRESS, overall28/37(75.7%),9 remain.

## F316–F318: deletion, migration and UCI admission failure

clear_secrets previously logged success after a failed rm. RPC clear_secret ignored
fs.unlink's null error return and always returned result=true. CLI deletion now
propagates failure, including all-secret partial failures; removing an absent file
remains idempotent. RPC validates the two literal names, delegates to the init
owner and returns result=false for nonzero or invalid command results. A compatible
Node fixture reproduces the original false success with the documented null return;
this is not actual ucode execution. [Primary fs.unlink contract](https://ucode-lang.org/module-fs.html#unlink).
Clearing a saved file does not stop a running VPN or erase copies already loaded
by the client/rendered obfs configuration; restart applies credential changes.

Legacy migration deleted staged UCI options before commit. After commit failure,
a retry saw no options and returned0 without retrying: old credentials remained on
persistent storage. A0600 secret-migration-pending marker is now created before
staged deletes and retained through failure. Retry commits even when no staged
legacy option remains; removal failure retains retry intent. Migration reads each
legacy value once, preserves an existing nonempty runtime credential, copies before
delete, and scrubs empty legacy values. Copy/marker/delete/commit/remove failures
are covered. The marker is tmpfs; no crash/reboot transaction or secure flash erase
is claimed. Rotate old credentials after migration as previously documented.

Both initial/reloaded qeli config_load calls, firewall load and restoration of the
qeli context now fail admission explicitly. Multiple qeli firewall zones are rejected
before mutation instead of silently selecting the last one. No-zone valid behavior
is retained. Baseline model probes reproduce false success on clear/load/ambiguous
zone and retained persistent secrets after migration retry in both interpreters.

## F319: rc.common hid callback errors from the service command

Review of the saved [upstream rc.common](https://raw.githubusercontent.com/openwrt/openwrt/master/package/base-files/files/etc/rc.common)
showed rc_procd invokes the callback, closes the service message, then start/stop
run optional hooks; the callback result is otherwise lost. A saved dispatcher
snapshot with fixture-only lib/functions/procd adapters reproduces direct callback
failure with CLI exit0. This uses an intermediate F316–F318 source before hooks,
recorded separately from the Git baseline, to isolate the dispatcher defect.

The init owner now retains preparation/cleanup status and returns it through the
standard service_started/service_stopped hooks. Failed stop cleanup still permits
procd_kill to be requested, but forbids subsequent start preparation in the same
restart/reload invocation. Valid disabled start succeeds. This covers callback
results, not ubus publication/delete failure, daemon acceptance, native process join
or authenticated connectivity. The real procd library's flock/admission behavior
is source-reviewed only; no custom locking mechanism was introduced.

Five saved-dispatcher cases plus23 init and9 installation cases make37 PASS per
BusyBox ash/dash on .11. The snapshot is byte/hash-pinned; tests require the explicit
QELI_OPENWRT_RC_COMMON_FIXTURE path and otherwise skip. All paths/adapters are in
temporary roots; actual /etc, firewall, rc.d and daemons are unchanged. Two result-
hook cases and the9 preceding init cases are new. Eleven Node fixtures PASS; one
new compatible RPC clear case. Native recipes37 and docs/bindings/diff PASS.

Current packet F316–F319: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-secrets-upgrade-20261006.
Evidence: release/certification/evidence/q31-secrets-upgrade-20261006.json.
Real libuci/fw4/procd/ucode/rpcd/opkg and cross-build NOT_RUN; router USER_EXCLUDED.
Full package upgrade/rollback, interprocess/daemon failure and lifecycle checks,
broader source review and build provenance remain OPEN. Previous seals/runtime
statuses/artifacts/dates/acceptance_basis and retained Q29/Q30/D06 observations stay
intact. Q31 IN_PROGRESS, overall28/37(75.7%),9 remain.

## F320: Keenetic installer consumed obsolete artifact names and hid dependencies

The build helper publishes qeli-client-keenetic-aarch64/mipsel, but the installer
looked only for qeli-client-aarch64/mipsel. A canonical-only bundle failed before
installation. It also ignored mandatory ip-full/iptables install failures with
||true and could publish a client and report completion with missing preparation.
Both behaviors are reproduced with the original Git source and isolated command
models. Documentation now lists canonical artifacts and --sync; old manually
prepared filenames remain a fallback, with canonical taking precedence.

The installer checks nonempty binary/init/new-config inputs before dependency
changes, propagates required update/install failures, checks ip/iptables command
availability and warns explicitly when optional ip6tables installation fails.
It prepares binary/init/config copies before publication, cleans ordinary temporary
files on exit and publishes each file by sibling rename. Existing INI content is
preserved; its mode becomes0600 and the credential directory0700. Copy/preparation
failure retains old installed binary/init. Publication failure can leave a partial
bundle, because multiple renames are not a transaction; correct the error and retry.
There is no forced service restart, concurrent-installer lock, fsync/power-loss or
actual architecture/ELF/package-manager qualification.

Eleven installer cases cover canonical and legacy names, both architecture choices,
existing config, required/optional dependency failures, missing command, incomplete
bundle/unsupported arch, copy failure and publication retry. Model artifacts are
small fixture files, not executed VPN binaries. Initial baseline reproduction used
ambient PATH and failed in the harness; that attempt is preserved. Final fixtures
restrict PATH for both sources; lab inventory confirms actual opkg is absent.

## F321: OpkgTun ignored ndm failures and mixed NetworkPlan reads

Each ndmc mutation previously continued after failure, then saved configuration and
logged interface up. Original-source model injection reproduces exit0/up/save after
failed ip global. The hook now checks every L3/configuration-save mutation, returns
failure, logs retry required and never logs up after failed apply. A0600 pending
checkpoint is created before mutations and removed only after successful save. A
failed apply bypasses the address-only no-op on retry, including when ndm already
shows matching connected addresses. Checkpoint create/remove errors reject apply
completion; no cross-process lock or durable/fsync transaction is implied. Earlier successful
ndm mutations are not rolled back; after fixing the error, rerun the hook or use the
manual registration sequence and check each command result. Deferred interface/
missing-plan behavior remains0 with a waiting diagnostic, rather than success-up.

The interface marker now requires opkgtun plus a nonempty all-decimal suffix and a
maximum15-character name. The prior glob accepted opkgtun0garbage; the model proves
it reached ndm. Rejected markers issue no ndm commands. This is a local marker grammar
fix, not a demonstrated server-to-ndm exploit. A plan is captured once per read and
IPv4/IPv6/MTU are extracted from that same snapshot. Forced replacement after the
first read reproduced old IPv4 from A with IPv6/MTU from B; corrected commands all
use A. This does not serialize a concurrent stop, mark a plan generation or validate
the whole NetworkPlan as a second client parser.

Nine hook cases cover dual-stack apply, all nine mutation failures and connected retry,
checkpoint creation/removal failures,
invalid marker, missing plan/interface, existing address no-op and forced plan
replacement. Existing optimistic address/status idempotence, MTU-only updates,
process ownership/join, legacy NAT/sysctl recovery and wan.d concurrency remain OPEN.
Current tests do not qualify real ndmc error/status output, event recursion, ndm
rollback or firmware compatibility. No router network/proc/sys/service was changed.

Current packet F320–F321: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-keenetic-install-hook-20261006.
Evidence: release/certification/evidence/q31-keenetic-install-hook-20261006.json.
20 cases per BusyBox ash/dash PASS in .11 temporary roots with substituted /opt,
PATH and opkg/ndmc commands. Native recipes37 and docs/bindings/diff PASS; native
implementation, OpenWrt adapters and build helpers are unchanged. Real opkg/ndm and
cross-build NOT_RUN; routers USER_EXCLUDED. Previous sealed evidence and runtime
statuses/artifacts/dates/acceptance_basis are retained; Q29/Q30/D06 observations
unchanged. Q31 IN_PROGRESS, overall28/37(75.7%),9 remain.

## F322: forwarding snapshots and failed restoration lost recovery state

Both Keenetic init templates wrote an unchecked snapshot directly to the final
file. A failed sysctl read could become an empty saved value; the original snapshot
function still returned success. Upstream route-query errors were also hidden by a
pipeline. Restoration ignored write errors and removed the checkpoint even when
recovery was incomplete. Isolated original/current comparisons reproduce these
cases; the failed read comparison calls the snapshot function, because the old
nat_up subsequently failed at its write after already creating the bad snapshot.

Both templates now check reads, allowed sysctl values and WAN query status, then
publish a0600 sibling checkpoint by rename. Only negotiated families are read/probed: an IPv4-only plan does not require IPv6
sysctl/WAN querying, and an IPv6-only plan does not require IPv4 sysctl/iptables. The version2 checkpoint records original values, TUN/LAN names and
families and RA journalled before mutation. Apply rejects a checkpoint for different rule
interfaces. Restoration validates saved fields, touches only journalled families
and checks reads/writes. A failed restore retains the checkpoint for retry; only
successful restoration removes it. Values changed away from the wrapper's1/2 are
preserved, but same-value administrator changes cannot be distinguished. This is
not a cross-process lock, fsync/power-loss transaction or ownership of kernel sysctls.

## F323: legacy rule cleanup could delete administrator rules and hide failures

Old rules had no owner tag: nat_up reused an administrator's matching rule, and
nat_down deleted it. Cleanup ignored firewall errors and could then discard the
saved state. Switching GATEWAY/OPKGTUN skipped recovery entirely. Original-source
models reproduce lost matching admin rules, false completion after failed delete/
check, retained stale state after mode change and restart reaching start after
incomplete legacy-state recovery.

New rules use comment qeli-keenetic-legacy. Checked add/delete helpers distinguish
model check status1 (absent) from higher errors; add/delete errors propagate. Only
tagged rules for journalled families and saved interfaces are cleaned. Recovery
runs from the checkpoint even after GATEWAY/OPKGTUN/TUN/LAN changes; without a
checkpoint no rules are deleted. Partial cleanup keeps retry state. Start rejects
failed stale recovery before clearing the plan or launching, and stop/restart
propagate cleanup failure. New legacy mode needs the iptables comment capability;
actual Entware/kernel behavior, including error-code differences, is NOT_RUN.

Versionless/unknown old checkpoints reject automatic cleanup and remain intact for
manual review. Before replacing an active legacy template, stop it with its old
script and inspect its rules/sysctls; if a checkpoint remains, keep its saved values
and identify old rule ownership before manual recovery. Do not discard that state
as an automatic migration. New code intentionally does not guess ownership of old
untagged firewall rules. No such migration was performed on a real router.

Twenty-two isolated state cases per template (44 new) cover capture/publication,
WAN failure, family/interface admission, journal failures, exact matching admin
rules, duplicate apply, check/add/delete/missing-command failures and retry,
restoration read/write failure, corrupt/old checkpoints, mode changes and start/
stop/actual dispatcher restart gating. Together with the20 prior installer/hook
cases,64 tests per BusyBox ash/dash PASS. Shared forwarding/firewall helper blocks are kept
identical by the scoped source review. Files replace proc/sys, all firewall/ip
commands are models and every potential signal is intercepted. No actual client
process, kernel forwarding, firewall or router service was operated. One initial
fixture TUN expectation FAIL and one reproduction-scope assertion FAIL are retained
and explained; final comparisons use corrected identical isolation for both sources.
Native recipes37/docs/bindings/diff PASS.

Packet F322–F323: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-keenetic-forwarding-state-20261006.
Evidence: release/certification/evidence/q31-keenetic-forwarding-state-20261006.json.
Actual router/iptables/comment module/sysctl runtime and cross-build NOT_RUN;
router USER_EXCLUDED. PID identity, TERM/join ordering, instant launch failure,
INI/core semantic parity, OpkgTun idempotence/concurrency and broader build/source
review remain OPEN. Earlier runtime statuses/artifacts/dates/acceptance_basis and
Q29/Q30/D06 observations are retained. Q31 IN_PROGRESS, overall28/37(75.7%),9 remain.

## F324: a numeric PID file could target a foreign process or process group

Both init templates trusted cat(PIDFILE) and kill -0/kill without identifying the
process. A stale PID pointing at an explicitly spawned sleep victim was accepted
and terminated by the original script. Invalid PID0 could reach a group-signal
attempt; that comparison always intercepted kill and sent no group signal.

New private PID records contain decimal PID and Linux proc start ticks. Both
values, exact record shape, positive non-system PID and executable identity are
checked before process admission/signalling. A replaced executable's deleted
suffix is accepted for the original generation. Start ticks distinguish PID reuse within one boot;
comm spaces/parentheses do not break stat parsing. Corrupt/legacy single-PID records,
foreign executables and mismatched ticks reject start/stop and retain state for
review. Status is0 live,3 stopped and4 unverified. These are best-effort shell/proc
checks: the check-to-kill race is not atomically closed by pidfd, and interprocess
locking, PID namespaces/reboot identity and actual router proc/readlink remain
unqualified. No atomic PID-identity protection is claimed.

PID publication uses a0600 sibling file and rename. Failure stops and joins the
known launched process when its identity is verified. If termination/publication
cannot finish, the pending record remains and both start/stop reject it for review.
The installer now rejects an existing PID or pending record before dependencies
or file replacement; stop the installed template and verify completion before
upgrade. There is no automatic migration of the former single-PID format. A
proc-read failure after launch may retain an incomplete pending record plus a
printed PID; do not discard it and launch another generation blindly.

## F325: TERM was followed by cleanup before exit, and launch failure looked successful

Original stop performed compatibility cleanup before TERM, immediately removed PID/
plan and deleted TUN without waiting. A delayed native helper's EXIT observed both
records gone; an ignored TERM still produced successful stop and forgotten state.
Original core-managed start reported success when the executable exited immediately
or when the PID path was a directory. Original NAT-failure unwind also cleaned before
the delayed helper exited. Safe original/current comparisons reproduce each case.

Both templates now require executable/config/readlink, check directories, recover
stale compatibility state, prepare/publish the process record and check post-exec
liveness. This is process admission, not authenticated NetworkPlan readiness for
core-managed/OpkgTun startup. TERM targets the verified process and polls for exit
up to15 one-second iterations. It treats the matching zombie as terminated; this
is observed process exit, not waitpid reaping of a process started by another shell.
Signal/identity/timeout failure retains PID/plan/marker and skips network cleanup;
restart does not launch another client. After exit, compatibility recovery runs,
then plan/markers/PID are cleared with errors checked. Startup plan/NAT/marker failures
use the same joined stop path. PID-removal/cleanup failure supports a later retry.

The wrapper no longer runs ip link del: kernel/core/ndm own TUN lifetime, and a
persistent or reused link must not be blindly removed. Startup grants no extra
claim about actual Qeli thread drain, kill-switch lifetime or connectivity. Exact
wall-clock deadlines, SIGKILL/power loss, a concurrently running wan.d handler and
all service operations/administrator changes under an interprocess lock remain OPEN.

Linux-owned native test executable: raw packet process-fixture.c, compiled once with
cc and hash recorded; no Qeli release binary rebuilt.43 native-process cases (21 base,
22 OpkgTun),44 state-file models and21 installer/hook cases total108 per BusyBox
ash/dash PASS;44 cases are new including the installer upgrade gate. The native
helper publishes a fake plan, delays exit or ignores TERM. Its unique per-test path
and explicitly spawned sleep children bound signal/cleanup targets. Network callbacks
are substituted; no actual kernel network/firewall/router service mutation. Polling
is shortened to3*50ms (startup50ms, plan timeout2 polls), qualifying control flow,
not the production15-second duration. Old state fixtures now model proc and provide
an executable/readlink so stale-recovery admission is exercised under the new format.
One initial original-source harness assertion FAIL is retained: it selected the
first nat_up and accidentally appended current lifecycle code; corrected comparison
appends only the final fixture callbacks, preserving the old implementation. The
invalid/group PID scenario always models signals, never sends a group signal.
Native recipes37/docs/bindings/diff PASS; actual router/core-OS integration NOT_RUN.

Packet F324–F325: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-keenetic-process-lifecycle-20261006.
Evidence: release/certification/evidence/q31-keenetic-process-lifecycle-20261006.json.
INI/core semantic parity, OpkgTun idempotence/generation/concurrency, packaging and
broader source/build ABI/provenance remain OPEN. Earlier runtime statuses/artifacts/
dates/acceptance_basis and Q29/Q30/D06 observations are retained. Router USER_EXCLUDED;
Q31 IN_PROGRESS, overall28/37(75.7%),9 remain.

## F326: optimistic OpkgTun no-op skipped changed MTU and incomplete application

Connected status and address substring/regex matches did not establish a complete
application. Original-source comparisons skip a changed MTU, accept IPv4 .20 for
.2 or regex lookalikes and IPv6 ::20 for ::2, and silently return with no current
plan. Matching addresses could also precede successful global/MSS/security/save.

The hook now requires a private qeli.opkgtun.applied receipt containing the exact
interface and complete captured plan. It publishes a0600 sibling by rename only
after all mutations and save succeed. Missing/changed receipts force application;
MTU-only changes reapply, and the next matching event is a no-op. Connected status
and literal address tokens are checked, including show exit status. Empty plans
cannot qualify no-op. Receipt publication or pending removal failure retains the
retry marker and cannot log success. Stop removes the receipt after client exit.

The receipt records successful commands; it is not a generation token, concurrency
lock or readback of every ndm setting. External MTU/MSS/security changes, obsolete
handlers after stop, address-family removal, equivalent IPv6 spelling and real
firmware output/event behavior remain unqualified. No firmware syntax is invented
to delete addresses or prove every applied setting. Existing snapshot consistency
remains; applying a snapshot does not prove it is the latest concurrent plan.

## F327: exit between proc stat and executable lookup was rejected as changed identity

A matching live stat can become a zombie before readlink(exe). Its proc directory
still exists while exe disappears. Old code returned unverified rather than exited,
retaining state after successful termination. A deterministic direct-child helper
keeps that zombie until the Python owner reaps it and reproduces failure in both
init templates. A separate unreadable-live-exe case must continue to reject signals.

On failed exe lookup, both templates reread stat and require the same start ticks
before accepting Z/X as exited. A live, unreadable, replaced or unverified process
still rejects ownership; disappearance retains the existing exited behavior. This
does not close the general check-to-signal race or provide pidfd protection.

Final120 cases per BusyBox ash/dash PASS:43 existing native-process cases plus four
new race/live-exe checks,44 state models,12 installer and17 hook cases. Eight hook
cases are new. Seven focused original/current hook methods reproduce eight old
assertion failures including subtests; fixed methods PASS. Deterministic old race
fails twice, fixed race and unreadable-live tests PASS on both shells. Initial dash
suite FAIL is retained; its exact cause was not instrumented. The initial orphan
race diagnostic did not reproduce because PID1 reaped it; direct-child qualification
is separately recorded. Native recipes37/docs/bindings/diff PASS. Commands/files
and network callbacks remain models; only owned Linux helper processes are real.
Helper bytes are unchanged from F324/F325; no Qeli/router binary rebuilt. Router
runtime USER_EXCLUDED, cross-build NOT_RUN.

Packet F326–F327: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-keenetic-opkgtun-receipt-20261006.
Evidence: release/certification/evidence/q31-keenetic-opkgtun-receipt-20261006.json.
INI/core parity, OpkgTun generations/concurrency and broader package/build/source
review remain OPEN. Q29 FAIL/ENONET, Q30 skips/drain OPEN and D06 retained.
Q31 IN_PROGRESS; overall28/37(75.7%),9 remain.

## F328: a build verifier stopped live lab services and reused their source directory

keenetic_verify.py unconditionally stopped qeli-server and issued pkill -9 qeli,
even though compilation and dependency inspection need no running-service change.
The old additive sync also preserved deleted modules and stale Cargo configuration.
Modeled old pipeline calls reproduce the service commands; actual file-adapter
comparison in an owned temporary directory reproduces retained stale inputs. No
old service or compiler command was executed against the live lab.

The verifier now uses a new0700 mktemp checkout under /var/tmp for each run, validates
the returned path, pins CARGO_TARGET_DIR to that run and delegates replacement to
router_source/native_lab. Its incomplete-sync marker and checked SFTP finalization
are reused; no local INI parser or separate source-sync implementation is added.
It never stops, kills or restarts services. Builds use --locked and one compiler
job, require release server jemalloc, and keep client-only features isolated.
The directory is retained for inspection; it must be removed when no longer needed.
No simultaneous shared source/target checkout is used. Toolchain pinning and full
router artifact provenance are separate OPEN criteria.

## F329: verifier failures could return success, and error text qualified ring absence

Old main printed FAIL but returned None, so the script exited0 after build failure.
It accepted a nonzero reverse dependency query containing did-not-match text as
ring absence, ignored missing-artifact inspection and could leave SSH unclosed on
sync exception. Controlled old runs reproduce false PASS and failed-summary exit0.

Every stage now uses checked commands. A successful forward normal/build dependency
graph is required; ring as an exact package name, empty/wrong-root graphs and error
text are rejected. Dev-only dependencies are excluded. Nonempty/executable artifact
checks and readelf ELF magic are mandatory, followed by checked SHA256. Connection
is closed in finally on success/failure, including sync failure. Missing credentials
exit2, verification/connect/close failures exit1, full success exit0.

Nineteen focused gate tests PASS on Windows and Linux, including failure injection
at every command, sync/readiness/hash/metadata/close failures and service isolation.
Seven shared-sync tests use actual shell/files with a temporary SFTP adapter on
Linux and PASS. Actual installed Cargo executes offline fixture graphs with local
crates: client normal/build graph excludes dev-only ring; positive server graph
contains ring and is rejected. No compiler, network dependency download, Qeli
host/cross build, client artifact or firmware is qualified by that fixture.37
native recipe tests/docs/bindings/diff PASS. Historical June11 host/ABI results
remain historical and do not certify current sources.

Packet F328–F329: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-keenetic-verification-gate-20261006.
Evidence: release/certification/evidence/q31-keenetic-verification-gate-20261006.json.
INI/core gateway parity, OpkgTun generation/concurrency, packaging and broader
source/build ABI/provenance remain OPEN. Q29 FAIL/ENONET,Q30 skips/drain OPEN,D06
retained. Real router USER_EXCLUDED; Q31 IN_PROGRESS,28/37(75.7%),9 remain.

Initial native-recipe FAIL is retained: its old literal server_build= source check
no longer matched dynamic stage output. The recipe now requires the named locked
server command, and the execution test asserts server_build=OK after success.
Final recipes PASS; no product command was weakened to satisfy the old assertion.

## F330: duplicated init INI parsing disagreed with the core

Both templates had an awk parser plus their own true/yes/1 table. Valid quoted,
uppercase/BOM and on values accepted by the canonical client parser were treated
as false in the wrapper. Original/current comparisons reproduce unwanted legacy
NAT/forwarding for quoted gateway_nat, uppercase keys and forward=on.

Both shell parsers are removed. qeli-client --print-gateway-owner uses the existing
bounded config_source loader and strict shared INI parser and returns only core or
legacy, before logging initialization or client execution. No config secrets are
serialized and hooks/device identity/TOFU/TUN/network setup are not executed. The
new API is gated Linux AND client-bin; outside that added region, client/mod.rs is
identical to the baseline. Cargo/native FFI recipes/default feature set and all
other native implementations are unchanged; this is not a new FFI ABI.

## F331: exit-node ownership and query failures could fall into legacy LAN NAT

Old core_manages_gateway inspected gateway_nat/forward only. Valid exit_node=true
therefore enabled the opposite-direction legacy LAN masquerade too. It is now core
ownership alongside gateway_nat/forward. All flags false/absent preserve the legacy
fallback. Invalid/duplicate/conflicting configurations, nonzero/unknown/empty/multiline
query results reject start before launch or network recovery. Unknown ownership
never admits nat_up. One validated pre-launch answer is cached for wrapper decisions;
GATEWAY=no and active OpkgTun require no legacy ownership query. Stop recovery remains
independent of current INI flags and saved-state rules are unchanged.

Upgrade the standalone binary and template together. A binary lacking the new
command rejects gateway startup rather than silently invoking the old shell parser.
The query is a config snapshot, not authentication readiness or proof of ownership
of a running generation. The process subsequently reloads the path: concurrent
configuration replacement between inspection/start and concurrent service/wan.d
operations remain OPEN. No atomic config-generation lock is claimed.

Fresh Rust1.97.0 offline/locked/jobs1 host client-bin debug build, eight binary
unit tests, pinned formatter and strict Clippy PASS.155 cases per BusyBox ash/dash
PASS:47 owned native-helper process cases,44 state models,12 installer,17 hook,19
verifier and16 new owner cases.22 actual inspector/INI admission subcases run per
shell across both templates.16 original/current records per shell reproduce the
three syntax mismatches and exit-node legacy ownership; only new-source queries
use the actual host inspector, old-source decisions use original awk. Firewall,
proc/sys, network and service callbacks remain models. The C process helper now
models the metadata response without a parser; actual core parsing is qualified
by Rust tests and the host inspector. Existing state fixtures were corrected to
admit metadata inspection so stale-recovery assertions reach recovery, not an
earlier unsupported-command failure; both final suites rerun.37 recipes/docs/
bindings/diff PASS. Windows rustfmt was unavailable, so the pinned lab formatter
was used before compilation. No Qeli release/cross-build, actual router/firmware
networking or native FFI artifact rebuild; historical statuses/dates remain.

Packet F330–F331: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-keenetic-core-gateway-20261006.
Evidence: release/certification/evidence/q31-keenetic-core-gateway-20261006.json.
Core gateway INI decision parity is now scoped PASS. Config/generation concurrency,
OpkgTun ownership, packaging and broader source/build ABI/provenance remain OPEN.
Q29 FAIL/ENONET,Q30 skips/drain OPEN,D06 retained; router USER_EXCLUDED.
Q31 IN_PROGRESS;28/37(75.7%),9 remain.

## F332: shell lifecycle operations and wan.d callbacks could overlap

Two handlers could interleave ndm commands and pending/receipt publication. Stop
could remove marker/plan while a captured old hook continued and recreated receipt
files. Concurrent start/stop/restart and installation had no common exclusion.

All three entry points now source one bundled lifecycle.sh. Atomic mkdir of
/var/run/qeli.lifecycle.lock admits one action; competitors return nonzero before
service/network/publication work and must retry. Init holds it across the whole
restart and internal cleanup; hook rechecks the marker under the lock. Installer
uses the same library from its bundle, prepares a0600 installed copy and publishes
it before binary/init. Existing older scripts do not cooperate: upgrade library,
both relevant templates and hook together after stopping/reviewing the old client.

Normal exit and HUP/INT/TERM release the lock. SIGKILL leaves it for manual review;
no stale-owner guessing, PID-based automatic deletion or process killing is added.
The directory is in the router's volatile /var/run rather than persistent /opt.
After verifying no init/hook/install owner remains, remove only the empty lock
directory with rmdir; preserve PID/plan/forwarding/pending records and retry the
normal action. Busy hook events are not queued and need a later event/manual retry.
A hanging ndmc can retain exclusion; no new firmware timeout guarantee is claimed.

## F333: a replaced core plan could still receive a complete applied receipt

The hook now compares its captured marker/whole plan before each checked L3/save
mutation and after save, before receipt publication. An observed replacement stops
the sequence and retains pending for retry. The coherent-snapshot test now checks
rejection and reapplication of the replacement, rather than qualifying old-plan
success. Core publication does not share the shell lock: replacement after the
last check, identical-byte ABA, generation authentication and family removal remain
OPEN. Commands already completed are not rolled back or called atomic.

## F334: installer rename into a directory falsely reported publication

mv source destination-directory returns success while putting a temporary basename
inside the directory. Binary/init/config/helper directory targets are rejected
before dependency work. Library preparation/publication failure keeps binary/init
unpublished and cleans its temporary sibling; whole-bundle rollback remains OPEN.

Eight new concurrent lifecycle methods (both init variants inside one method) and
two installer methods qualify165 total tests per BusyBox ash/dash:47 owned native
helper process,44 state models,14 installer,17 hook,19 verifier,16 owner and8
lifecycle. Actual owned shell barriers reproduce three baseline failures per shell:
interleaved hooks, stop during hook and plan rotation during save; original installer
directory publication also fails the new admission assertion. Current suites PASS.
Network/ndm/opkg/sysctl remain command/file models. C helper and actual host debug
inspector bytes match F330/F331; Rust/core/native implementations, ABI/artifacts and
recipes are unchanged, so no new Rust/FFI/release/cross build is claimed.37 recipes,
docs/bindings/diff PASS. Real router runtime USER_EXCLUDED.

Packet F332–F334: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-keenetic-lifecycle-lock-20261006.
Evidence: release/certification/evidence/q31-keenetic-lifecycle-lock-20261006.json.
Cooperating shell exclusion and observed replacement rejection are scoped PASS;
core/config generations, firmware ordering, packaging/cross ABI/source provenance
and broader review remain OPEN. Q29 FAIL/ENONET,Q30 skips/drain OPEN,D06 retained.
Q31 IN_PROGRESS;28/37(75.7%),9 remain.

Initial directory baseline subtests also produced two fixture errors because an earlier successful old installation affected later targets. Those raw logs are retained and do not qualify those targets. Separate fresh fixtures now reproduce false old success for binary/init/config and checked new rejection: six old/current records per shell PASS. No product implementation or final165-suite input changed for that comparison.

## F335: shared cached checkout and manifest backup contaminated router builds

Both helpers reused /opt/qeli-src without --sync. A successful build could thus
contain older sources; simultaneous runs also edited/restored one Cargo.toml and
shared target. Each run now creates a checked private0700 mktemp root, always
uploads current managed inputs, requires completed sync and restricts only that
copy to rlib. No shared manifest backup/restore remains. CARGO_TARGET_DIR belongs
to the run; CARGO_INCREMENTAL=0 and --jobs1 bound Qeli compiler concurrency per run.
--sync is accepted for compatibility. The printed owned directory is retained
for diagnosis and needs reviewed cleanup after completion; old roots are untouched.
Installed toolchains and Cargo downloads may still be shared; concurrent toolchain
installation and local source mutation during upload are not qualified.

## F336: matching SHA admitted incompatible/non-ELF router artifacts

A valid transfer hash did not establish ELF architecture or static admission.
Both helpers now share router_artifact.py: read one SFTP snapshot, close transport,
compare its SHA, inspect that same snapshot and pass it to the existing atomic
publisher. Invalid data cannot replace a previous binary or bypass admission via
a matching local hash. The gate bounds headers/segments, requires little endian,
target class/machine and executable entry, rejects PT_INTERP/DT_NEEDED and checks
ARMv7 EABI5 hard-float. It permits ET_EXEC/static PIE; unsupported extended program
headers fail. This is metadata admission, not proof of musl, CPU instructions,
full MIPS float ABI, kernel/firmware behavior or complete binary validity.

## F337: SDK installation did not enforce the checked-in dependency graph

The OpenWrt package now requires Cargo.lock and checks the client-only graph with
cargo metadata --locked before cargo install --locked --jobs1. The actual fixture
showed that install --locked --path alone still admitted missing/stale lockfiles
for its local path dependency; the guard rejects both before publication. No
dependencies were downloaded. The source SHA/mirror hash remain release-cut
obligations. Empty SDK include stubs only expose the checked-in Build/Compile to
GNU make; this is not an OpenWrt SDK or real package/cross build.

45 router tests PASS on .11:15 helper failure/admission,16 ELF/snapshot,7 private
checkout and7 sync methods. Three POSIX methods perform actual owned directory/
manifest operations; compiler/SSH helper cases are models with a rejecting
test-only Paramiko stub because the lab lacks Paramiko.20 old/current comparison
records reproduce cached-source admission and invalid matching-SHA publication in
both helpers. Five actual offline GNU make/Cargo tiny-crate cases show old missing/
stale success, new missing/stale refusal and new valid success. Tiny real x86_64
glibc-static ELF passes the metadata gate; the unchanged prior GNU debug Qeli ELF
is rejected for PT_INTERP. Neither is a fresh router Qeli/musl cross build or a
firmware execution. Initial import/fixture-assumption failures remain retained
and unqualified.37 recipe tests and docs/bindings/diff PASS.

Packet F335–F337: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-router-build-isolation-20261006.
Evidence: release/certification/evidence/q31-router-build-isolation-20261006.json.
Rust/core/native compile inputs, native libraries, existing router binaries and
their historical qualification remain unchanged. No fresh Qeli/FFI/release/cross
build, toolchain pinning/reproducibility or full ABI qualification is claimed.
Core/config generation, ABA, last-check replacement, firmware ordering and
whole-bundle rollback stay OPEN. Q29 FAIL/ENONET,Q30 exclusions/drainOPEN,D06 retained.
Q31 IN_PROGRESS;28/37(75.7%),9 remain; no whole Q31 checklist row is closed.

Metadata references: [ELF program headers](https://refspecs.linuxfoundation.org/elf/gabi4%2B/ch5.pheader.html),
[Arm ELF32 ABI](https://github.com/ARM-software/abi-aa/blob/main/aaelf32/aaelf32.rst),
[Cargo install](https://doc.rust-lang.org/cargo/commands/cargo-install.html).

## F338: default Rust, floating nightly and unverified Zig changed build identity

Both helpers independently installed/selected toolchains. Ordinary builds used
default Cargo/Rust, MIPS used +nightly and any Zig version was only printed. Current
read-only inventory confirms both labs default to Rust1.96 while pinned native
recipes use1.97.0. The new router_toolchain.py owns target aliases, setup and build
commands for both helpers: Rust1.97.0, nightly-2026-06-10 for MIPS, Zig0.13.0 and
cargo-zigbuild0.23.0. The nightly archive date is taken from the existing .10
manifest (compiler commit date is June9); no new latest-nightly selection.

Wrong Zig fails before installer work. Named Rust pins/versions, target std and
nightly rust-src are verified after successful installers; missing results reject
setup. cargo-zigbuild inventory and its top-level executable version both must
match. Install uses explicit Rust and one job; default Rust is unchanged. Dead
helper checked/feature definitions and duplicate OpenWrt build_std data/argument
are removed. Architecture/toolchain/flags have one owner.

## F339: inherited encoded/compiler flags shadowed the intended recipe

CARGO_ENCODED_RUSTFLAGS takes precedence over RUSTFLAGS, including the old MIPS
soft-float assignment. The shared Qeli command removes encoded flags, RUSTC,
CARGO_BUILD_RUSTC and CARGO_BUILD_RUSTFLAGS; assigns its own RUSTFLAGS (empty for
ordinary targets, explicit MIPS linker soft-float), disables Rust wrappers and
uses the pin with a private target/noincremental/jobs1. This is a bounded ambient
override fix, not a sanitization of every Cargo profile/config/PATH/cache input.

52 Linux router tests PASS:12 helpers,10 shared policy,16 ELF,7 source and7 checkout.
Four formerly duplicated setup test methods become one delegation method plus10
shared semantic failure methods; coverage is moved, not discarded.12 old/current
model records reproduce wrong Zig admission and unpinned/unisolated commands in
both helpers. A real offline tiny host Cargo/Rust fixture runs old/new prefixes:
old encoded cfg is active under default Rust1.96, new cfg is absent under Rust1.97.
No cargo-zigbuild/musl/cross/MIPS/nightly/Qeli build is exercised by that fixture.
37 recipes/docs/bindings/diff PASS. .10 inventory is read-only; execution uses .11.
No global tools/defaults/service/network/installed binaries were changed.

The initial draft used cargo +pin zigbuild --version, which the real tool rejects;
the final check uses cargo-zigbuild --version. A transient uncommitted BIN removal
was caught by helper tests and repaired. A test-only import context also unloaded the policy module and broke comparison
mock identity; pre-importing it fixed the harness. These initial failed attempts
are retained, do not qualify success, and final exact-source tests rerun. Version pinning does
not prove hermeticity, full ABI/ISA, independent A/B reproducibility or router
runtime. Shared tool installation/cache, global config/PATH and upload races
remain. Real router USER_EXCLUDED; SDK Rust feed/source/mirror release obligations
are separate. Core/native/Rust implementation inputs/artifacts unchanged.

Packet F338–F339: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-router-toolchain-policy-20261006.
Evidence: release/certification/evidence/q31-router-toolchain-policy-20261006.json.
Generation/ABA/last-check/firmware/whole-bundle rollback OPEN. Q29 FAIL/ENONET,
Q30 exclusions/drainOPEN,D06 retained. Q31 IN_PROGRESS;28/37(75.7%),9 remain;
no whole checklist row is closed.
Policy references: [rustup pins](https://rust-lang.github.io/rustup/concepts/toolchains.html),
[Cargo environment](https://doc.rust-lang.org/cargo/reference/environment-variables.html).

## F340: failed code publication left a mixed installer generation

The old installer replaced helper/binary/init independently. A failed final init
rename or TERM after binary publication left new executable/helper with old init.
The new installer prepares same-directory old-file backups, preserves bytes/modes
and writes a0600 recovery record before publication. Rollback intent precedes
each rename; a failed same-directory rename retaining its source is treated as
unpublished. Ordinary failure/handled termination restores changed code in reverse
order; targets originally absent are removed. Failed restore retains remaining
backups and install-pending rather than silently retiring recovery evidence.

One shared qeli_install_ready guards retry, both updated init starts and active
wan.d callbacks. Stop is still allowed. The lock is released after handled
rollback. A stale pending record, including a dangling symlink, refuses admission.
Older scripts need manual stop/review and do not automatically honor the guard;
an old init restored during a partially failed rollback cannot be called protected.
Whole-bundle crash atomicity is not claimed: SIGKILL/power loss needs owner review
and manual recovery; no fsync or automatic stale deletion is added. Existing INI
contents remain; tightened config600 or a newly published example may remain after
a late failure. opkg/package changes and external root/admin replacement are
outside code rollback. Missing named backups may have been consumed by a successful
partial restore; inspect the complete set, do not blindly delete targets/markers.

## F341: linked/special targets bypassed regular-file installation assumptions

The former directory check admitted symlinks/FIFO. In particular, chmod600 of
client.conf followed a symlink and changed another file; chmod700 of a linked
config directory changed the outside directory. Current installed code/config
targets must be regular or absent, never symlink; /opt/etc/qeli must not be linked.
Rejection occurs before package updates. This does not close root/admin path races.

176 tests PASS per BusyBox ash/dash:47 owned native process,44 state,14 installer,
17 hook,19 verifier,16 gateway owner,8 lifecycle and11 new install rollback/admission
methods. New cases cover every code rename, old/absent/new code, restore/copy/marker
failures, TERM after a real owned rename, regular/dangling linked targets,FIFO,
linked config directory and both init/hook pending guards. Five old/current
scenarios produce10 records per shell: init rename, TERM, config symlink, directory
symlink and failed restore. Old mixed code/outside chmod/no recovery marker are
reproduced; new restore/refusal/retained recovery assertions PASS.

Initial11-method suites fail only on a fixture assertion counting read-only opkg
print-architecture as a dependency mutation during refused retry. The assertion
now compares mutating package calls, with full176 rerun per interpreter; original
FAIL logs/inputs retained. State function fixtures explicitly source the shared
library and all library paths map into their owned root. No fixture relies on a
real /opt. Network/ndm/opkg/proc-sys remain models; actual owned shell/rename/signal/
native helper and existing host inspector execute. Reused helper/inspector hashes
match F330–F331; Rust/core/native inputs/artifacts/recipes unchanged.37 recipes,
docs/bindings/diff PASS; no new Qeli/FFI/release/cross build or real router runtime.

Packet F340–F341: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-keenetic-install-rollback-20261006.
Evidence: release/certification/evidence/q31-keenetic-install-rollback-20261006.json.
Ordinary executable-bundle failure recovery is scoped PASS. Power-loss transaction,
older scripts/admin, core/config generation/ABA/last-check, firmware/full ABI and
cross-build qualification remain OPEN. Q29 FAIL/ENONET,Q30 exclusions/drainOPEN,
D06 retained; real router USER_EXCLUDED. Q31 IN_PROGRESS;28/37(75.7%),9 remain;
no whole checklist row is closed.

## F342: UCI → INI changed literal strings before the shared parser

The renderer wrote string values without quotes. The shared INI parser trims
edge whitespace from bare values and removes one surrounding quote pair with
backslash-quote unescaping. A username with edge spaces or a volatile obfs_key
starting and ending with quotes could therefore differ from the operator input.
Preserving the secret file alone did not protect subsequent key serialization.
This is a data conversion defect; this packet did not run a VPN handshake.

One ini_write now wraps strings in double quotes and escapes only double quotes,
matching the shared core serializer. Bare backslashes remain literal. ini_kv
keeps omission of absent optional values; required server/proto/password_file
and logging use the same writer. ASCII controls are still stripped before
writing, bool/MTU remain unchanged. UCI text is not evaluated; the parameter
surface and external configuration format remain INI.

Four new tests exercise40 round-trip operations:13 variants each for the writer,
user and obfs_key plus one required/logging/control case. Unmodified
config/format.rs is compiled into a separate GNU host inspector with the existing
log rlib; the fixture supplies generated INI on stdin and compares UTF-8 bytes
through hex fields. This executes the actual shared parser, not another Python
implementation. The Git baseline init with the same new tests reproduces13
mismatches per shell. Full current41 =23 init +9 defaults +5 saved rc.common
dispatcher +4 round-trip PASS separately under BusyBox ash and dash. Files, init
functions and the Rust parser execute in owned temporary paths; UCI/procd/firewall
remain models. The saved dispatcher snapshot/hash is unchanged; no real OpenWrt
daemons run. Initial standalone compilation lacked the log dependency; that
failure is retained, then the existing GNU log rlib was linked without parser edits.

11 LuCI Node fixtures,37 native recipes, docs/bindings/diff PASS. Core/native/
router build recipes and existing artifacts are unchanged; no fresh Qeli,
FFI/cross/firmware/ABI build or network obfs handshake is claimed. Value format
round-trip is tested separately from ClientConfig semantic validity.

Packet: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-openwrt-ini-roundtrip-20261006.
Evidence: release/certification/evidence/q31-openwrt-ini-roundtrip-20261006.json.
Q31 IN_PROGRESS,28/37(75.7%),9 remain; no whole checklist row closed.
Generation/ABA/last-check/firmware ordering and cross qualification remain OPEN;
Q29 FAIL/ENONET,Q30 exclusions/drainOPEN,D06 retained, router USER_EXCLUDED.

## F343: wrapper did not check dev and the L3 owner before launch

Both init templates used their own TUN for legacy firewall and the OpkgTun marker,
but gateway inspection did not check that name against INI dev. With GATEWAY=no
or active OpkgTun it skipped inspection entirely. A mismatched dev/dev_attach
could therefore launch another interface or attempt to create a device owned by
ndm; gateway flags could introduce an additional owner.

Metadata CLI now accepts paired --expect-router-device and --expect-router-attach
only with --print-gateway-owner. One read through the shared bounded loader and
strict core parser checks matching dev, device_type=tun and dev_attach. Attached
router mode requires gateway_nat/forward/exit_node disabled because ndm owns
L3/gateway. The old metadata CLI without expectations keeps its original function;
updated templates require the new binary. Unsupported flags on an older binary
reject before launch instead of silently falling back.

Both templates perform this query once before old state cleanup, NetworkPlan/PID
removal and process creation; GATEWAY=no still requires interface admission.
No shell INI parser was added. The check does not pin the snapshot for later
startup: edits between inspection and launch, generation/ABA remain OPEN.
Do not edit client.conf during start/restart. After fixing names/modes, align INI
and template and retry start; inspection itself creates no TUN and executes no hooks.

Fresh GNU client-bin debug build Rust1.97.0,12 targeted units (4 new), strict
Clippy and fmt PASS. 184 shell tests per BusyBox ash/dash PASS:
47 process +44 state +14 installer +17 hook +19 verifier +24 gateway owner
+8 lifecycle +11 rollback. Eight new shell methods add34 actual inspector
cells; with the previous22 this is56, not network connections.
The same new tests with old Git template bodies and old host inspector produce
20 false-admission mismatches across eight methods per interpreter.

The first full suite hung with the older C process fixture: argc==4 did not
recognize additional CLI flags and metadata inspection entered its modeled
lifetime. This timeout is not a qualified full result; owned helper/suite
processes were stopped after exe hash/cwd checks, two owned temp roots cleaned,
final lookup empty. A separate C helper recognizes only old4 and new8 arguments;
its metadata reply remains a model, process lifetime tests execute actual native
processes. Old/new helper source/hashes are retained; the old executable is unchanged.
Windows PATH lacked rustfmt; the installed pinned .11 formatter was used.

317 compile inputs and17 fixture Git files match supplied bytes; only private
Cargo.toml is restricted to rlib, with the original manifest retained.
37 recipes, docs/bindings/diff PASS. Rust edits are confined to Linux client-bin
metadata admission/CLI; normal data plane, FFI/JNI exports/config schema unchanged.
Previous native/release/router artifacts remain; no fresh FFI/release/cross/
firmware build or router ABI/runtime is claimed. .10 untouched; .11 executed
only owned file/compiler/process fixtures.

Packet: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-keenetic-interface-admission-20261006.
Evidence: release/certification/evidence/q31-keenetic-interface-admission-20261006.json.
Q31 IN_PROGRESS,28/37(75.7%),9 remain; no whole checklist row closed.
Q29 FAIL/ENONET,Q30 exclusions/drainOPEN,D06 retained, router USER_EXCLUDED.

## F344–F346: actual router cross matrix and compile regressions

Previous host/tiny-crate gates did not compile current Qeli against musl. All four
actual baseline release client-only builds failed: musl statfs.f_type is unsigned
while PROC_SUPER_MAGIC is signed. MIPS additionally lacks std AtomicU64 for the
roaming RPF lease counter. F344 compares both through lossless i128, retaining the
regular-procfs check. F345 uses existing portable-atomic;64-bit width, Relaxed
fetch_update, checked_add and exhaustion refusal remain. Shared Linux code changes;
no fresh FFI/native release artifact qualification.

All four fixed locked/client-bin/jobs1 builds PASS in private0700 source/target,
with a private rlib-only manifest. Rust1.97.0/MIPS nightly-2026-06-10,
Zig0.13.0/cargo-zigbuild0.23.0. Each SHA-bound SFTP snapshot passed common ELF
admission/readelf: little endian, matching class/machine, executable entry,
no PT_INTERP/DT_NEEDED; ARM EABI5 hard-float. MIPS declares O32/MIPS32r2/soft-float.
These are file declarations, not firmware/other-ISA/kernel runtime or clean A/B proof.

| Target | Bytes | SHA-256 |
|---|---:|---|
| aarch64-unknown-linux-musl | 4818040 | 60641ddcdb447c347240e173a341923d3eadc34b9f72fb30954fc4ff00327351 |
| x86_64-unknown-linux-musl | 5574984 | ec567377e46cd8a2ed629bccfb83318fba4e19bf26905df3f49ca77c254a5925 |
| mipsel-unknown-linux-musl | 6846056 | b365638cfc715086052b04ee6a3f5ad3e826077a858387bfc28d3f0d3c8faeaa |
| armv7-unknown-linux-musleabihf | 4981980 | ba223fb91a931aad1c631ae71b111f568cdcb8830a69ece684854e03c591910f |

GNU build/strict Clippy/fmt PASS on .11;79 sysctl tests PASS with3 inherited ignored,
plus1 real descriptor recreation test in a private network namespace. Fresh static
x86_64-musl runs24 gateway tests each BusyBox ash/dash without skips,56 actual
metadata cells per interpreter.52 Linux router helper tests without skips,
37 recipe checks/docs/bindings PASS.

F346 aligns the existing four-target CI with named pins instead of floating
stable/nightly, explicit Cargo/rustup toolchains, jobs1, cleared compiler overrides
and common static ELF admission instead of file alone. PyYAML6.0.3 installed only
in the audit directory parsed the workflow;16 rendered shell steps pass bash -n.
The exact Python ELF step passes4 actual binaries and rejects4 truncated copies,
8 expected outcomes. GitHub Actions NOT_RUN.

The first as _ cast revision failed compilation and is retained; it was replaced
by lossless conversion. Initial library-test harness omitted conformance siblings;
12 exact Git fixtures and the correct sysctl:: filter produce79 tests. Two zero-match
filters are NOT_RUN, not PASS. Windows helper run had10 POSIX skips; qualification
uses the fresh Linux run without skips. Verified inputs:317 compile/8 gateway files
on .10; two317-input source layouts/12 conformance/16 helpers on .11. Only private
Cargo manifests become rlib; reproduction scripts use environment credentials.

.10 ran owned sequential compilation/metadata/filesystem fixtures and installed
named nightly/rust-src/missing pinned std targets; defaults and live services stayed
unchanged. .11 ran owned GNU/private-namespace checks. Repository binaries were
not replaced or deployed.

Current Q31 remainder:

| Area | State |
|---|---|
| Actual standalone four-target cross builds/ELF | DONE in the stated scope |
| UCI/INI, shell lifecycle/rollback and LuCI | Scoped suites PASS; model boundaries retained |
| Actual ucode/fs, rpcd/ubus/UCI and session ACL | Scoped native PASS F347–F351 below; SDK Kconfig/LuCI packaging scoped PASS F353–F354; retained-binary SDK packaging/offline opkg Q31-V003 scoped PASS; stock Rust1.89/x86 SDK macro compile/link F358 scoped PASS; bootstrap/release pipeline OPEN; actual non-PID1 procd scope F352 and HTTP/session API Q31-V002 qualified |
| Keenetic preflight→launch INI; core plan generation/ABA | F355/F361 scoped PASS; F362–F363: late receipt and counter reset on fresh OpkgTun init launch scoped PASS; writer/lock, direct launch and admin replacement OPEN |
| SIGKILL/power-loss installer recovery | Manual recovery; no full bundle transaction claim |
| Firmware WAN/reboot/DNS/firewall/RSS/throughput | USER_EXCLUDED; cross-build is not a device test |
| Hermetic shared caches/config/PATH/clean A/B | F356–F357 compiler/intermediate-directory and F364 five standalone release fields scoped PASS; F365/Q31-V006 all four standalone Qeli A/B scoped PASS; full hermetic policy OPEN |

This current table supersedes historical cross-build NOT_RUN statements without
rewriting original evidence. Packet:
C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-router-cross-matrix-20261006.
Evidence: release/certification/evidence/q31-router-cross-matrix-20261006.json.
Q31 IN_PROGRESS;28/37(75.7%),9 remain. Q29 SIGKILL FAIL/auto-null ENONET,
Q30 Apple exclusions/callback-drain OPEN and D06 ACCEPTED retained.

## F347–F349: actual ucode/fs and volatile secret transport

Unmodified upstream ucode and fs were built on .11 at the revisions used by
OpenWrt24.10 (3f64c8089bf3ea4847c96b91df09fbfcaec19e1d) and25.12
(85922056ef7abeace3cca3ab28bc1ac2d88e31b1). This is actual GNU host interpreter,
filesystem and process execution, without rpcd/ubus daemons, firmware, UCI/procd
or package SDK qualification. Those scopes remain OPEN.

F347: array arguments to fs.popen are unsupported in these revisions, returning
null/EINVAL. Fixing the regex alone still left both secret names unable to write.
The RPC now selects one of two literal init commands and sends the secret only
through stdin. Caller-controlled names/values never enter shell text or argv.
Strict write/close comparisons reject short writes, nonzero exit and null status.

F348: the JavaScript-escape regex \x00..\x1f/\x7f failed in actual ucode when
checking a nonempty secret. An ord byte loop now rejects C0/DEL, preserves literal
Unicode/metacharacters and enforces 1..4096 bytes. JSON remains internal RPC/test
transport; configuration remains INI.

F349: mv interpreted a directory destination as a container, placed .secret.*
inside it and returned success. A linked runtime directory could change external
permissions/credentials. Init now rejects linked/non-directory runtime leaves
and linked/non-regular secret targets before publication. Clear also rejects a linked runtime leaf to
prevent external deletion; an absent runtime directory remains a successful no-op. Status is true only
for readable regular files of 1..4096 bytes in an unlinked runtime directory.
Empty/oversize/directory/FIFO/link targets are not configured secrets. These are
stable filesystem states; root/admin replacement between check and rename remains
outside the guarantee.

All 14 new test_openwrt_ucode.py methods PASS without skips on both versions,
each with BusyBox ash and dash: four combinations. Cases cover both names/exact
bytes, Unicode byte boundaries, all 33 controls, invalid types/names, stdin shell
injection sentinel and argv secrecy, child exit17/preserved previous file, four
invalid publication targets, five invalid status targets, linked runtime without
write/chmod, linked runtime clear refusal, clear filesystem failure and closed service verbs/status. Service
responses are exit-code adapters, not real procd. The full 55-method OpenWrt suite
(41 existing +14 new) PASS in both shells with ucode24.10, hash-checked saved
rc.common and the reused exact-source Rust INI inspector.11 LuCI Node fixtures,
37 recipes, docs/bindings/diff PASS.

The original Git module with corrected loader fixture produced40 assertion
failures in11 methods for each of four combinations. A regex-only intermediate
isolated the array-popen failure; the first fix exposed directory false success.
Three new status/runtime-link write/delete methods against old init/status give 7 independent
failures per version. Initial include() loader and cascading subtest cleanup
errors are harness failures, not extra product defects; original inputs/logs are
retained. Final subtests clean their owned paths even after failed assertions.

Dependencies were downloaded/extracted only inside the private0700 root, without
system installation: CMake3.31.6, json-c0.18 and recorded Debian library packages.
Single-job builds enable only ucode/fs. Source archives, commit/source/executable/
shared-library SHA256, CMake config/runtime ldd and exact Qeli inputs are saved.
Initial private CMake dependency failures are retained. .10 was unchanged; no
services, /etc, firewall or network on .11 were changed. Rust/native/router
artifacts/recipes are unchanged; a new Qeli/FFI/cross build is unnecessary for
these shell/ucode changes and is not claimed.

API/pin sources: [OpenWrt24.10 recipe](https://github.com/openwrt/openwrt/blob/openwrt-24.10/package/utils/ucode/Makefile),
[OpenWrt25.12 recipe](https://github.com/openwrt/openwrt/blob/openwrt-25.12/package/utils/ucode/Makefile),
[pinned fs.popen](https://github.com/jow-/ucode/blob/85922056ef7abeace3cca3ab28bc1ac2d88e31b1/lib/fs.c),
[ucode core API](https://ucode-lang.org/module-core.html).

Packet: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-openwrt-ucode-runtime-20261006.
Evidence: release/certification/evidence/q31-openwrt-ucode-runtime-20261006.json.
ucode/fs platform API is qualified in this scope; daemon/SDK/firmware,
config preflight-to-launch generation/ABA and clean A/B remain separately OPEN.
Q31 IN_PROGRESS;28/37(75.7%),9 remain. Q29 SIGKILL FAIL/auto-null ENONET,
Q30 Apple exclusions/callback-drain OPEN and D06 ACCEPTED are preserved.

## F350–F351: package dependencies and actual RPC/UCI/ACL

F350: the LuCI package declared only qeli/rpcd-mod-ucode despite requiring the
UI framework and importing fs. luci.mk maps LUCI_DEPENDS into runtime DEPENDS;
rpcd-mod-ucode requires libucode but not fs. Existing LuCI installs mask missing
requirements. luci-base and ucode-mod-fs are now explicit dependencies. Actual
rpcd without fs.so cannot register luci.qeli; with it all five methods exist.
A full SDK package resolver/install was not run.

F351: the form sent secrets without caller validation. Native ubus CLI/JSON→blob
truncates NUL and its suffix before Qeli: left<NUL>right was stored as left with
success. This is unsupported input for the transport's C-string API; a handler
cannot detect bytes already lost. The official caller now shares a form/setSecret
validator rejecting all C0/DEL before RPC, invalid types, over4096 UTF-8 bytes
and lone surrogates. Empty fields preserve existing secrets; Unicode/quotes/edge
spaces remain literal and storage errors propagate. Arbitrary direct RPC callers
must reject NUL before serialization; server rejection of already lost NUL is not
claimed. The probe is UNSUPPORTED_NUL_INPUT_LOST_BEFORE_QELI, not supported input.

A pristine GNU host stack was built from OpenWrt24.10 recipes at recorded commit
58584c6f2829a7e5d77376ab68aae3dcc4a392d8: libubox49056d17,ubus60e04048,
uci16ff0bad,rpcdbba95191 and ucode3f64c808; full SHAs are in evidence. CMake3.31.6,
json-c0.18 and existing native runtime libraries, single compiler job and private
DESTDIR/sysroot only, no system install. rpcd file/rpcsys/iwinfo are disabled;
ucode plugin/fs are real. The independent new ucode build has ZLIB OFF. In F347,
ZLIB remained default ON but its target was not built or used, clarifying the
previous brief modules description without changing old artifact bytes. Initial
install failed on unbuilt zlib; a separate copy was built. Initial rpcd compile
lacked private json-c headers; only build include flags were corrected.

Seven new integration methods PASS without skips. Each starts owned ubusd/rpcd
inside a chroot with a separate Unix socket, unmodified Qeli module/init/ACL,
actual OpenWrt functions.sh/uci.sh/libuci and private config/runtime paths.
Cases cover signature/type policy, exit2/9 and VM recovery, both secret bytes/
4096byte Unicode/child17, filesystem/linked-leaf guards, actual reader/writer ACL
allow/deny decisions for the five methods and UCI qeli/firewall, session-specific
UCI set/commit and unknown status for missing config.11 literal/control/Unicode/
injection values also pass real UCI→init renderer→reused exact-source Rust INI
inspector, SHA256 afc80db3… matches F342 and format.rs is unchanged. Service
start/stop/enable verbs and logger are adapters; procd is unqualified. Actual
session.access decisions are tested; HTTP/uhttpd authentication bridge is not.

Initial chroot devices in /tmp(nodev) could not open; fixtures moved to owned
/var/tmp without remounting. Initial global CLI staging assertion was wrong:
global UCI deltas are init-visible and do not model LuCI session staging. Final
checks use real session SID uci.set/commit: status stays unchanged until commit.
Initial NUL assertion was qualified after the separate transport probe. These
setup/fixture assumptions are not extra product findings; original FAIL inputs/
logs are retained. Final tests/cleanup PASS; no owned daemons/runtime roots remain.
Host /etc, services, network/firewall and .10 were unchanged.

14 Node fixtures PASS,3 new:66 control/name cases, invalid types, ASCII/Unicode
byte boundaries, lone surrogates, literal values, blank/no-op and storage failure.
Old JS with the same new tests reproduces NUL transmission before validation;
initial incomplete baseline without qeli.uc is excluded.37 recipes, docs/bindings/
diff PASS. Core/native/router inputs/artifacts are unchanged; no new Qeli/FFI/
cross build is claimed. Pinned source archives, CMake configs, runtime template/
dependencies, executable/module SHA and exact Git input proof are saved.

Sources: [pinned rpcd package dependencies](https://github.com/openwrt/openwrt/blob/58584c6f2829a7e5d77376ab68aae3dcc4a392d8/package/system/rpcd/Makefile),
[LuCI runtime dependency mapping](https://github.com/openwrt/luci/blob/openwrt-24.10/luci.mk),
[pinned rpcd ucode dispatch](https://github.com/openwrt/rpcd/blob/bba95191ff2f22c9118a1ba1355b83afaa277ae3/ucode.c).
Packet: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-openwrt-rpcd-package-20261006.
Evidence: release/certification/evidence/q31-openwrt-rpcd-package-20261006.json.
Native RPC/UCI/session ACL is qualified; these tests do not replace procd/SDK/
HTTP/firmware. Config preflight/core generation/ABA and clean A/B remain OPEN.
Q31 IN_PROGRESS;28/37(75.7%),9 remain. Q29 SIGKILL FAIL/auto-null ENONET,
Q30 Apple exclusions/callback-drain OPEN and D06 ACCEPTED are preserved.

## F352: init hid submission errors from the actual procd supervisor

On the official OpenWrt24.10.4 x86/64 rootfs, old init returned exit0 for
start/stop/restart/reload without procd's service object even though ubus reported
Not found. Missing Unix socket errors were also lost. procd.sh cleans up JSON
and restores its namespace after ubus; rc.common invokes result hooks afterwards.
The old hooks preserved only preparation/rendered-INI cleanup failures, allowing
LuCI to report success without an available supervisor.

Init now observes command ubus call service set/add/delete in its own invocation,
forwarding the identical argument array and preserving the actual exit code.
The wrapper is installed by start_service/stop_service after procd.sh is loaded;
it does not replace library internals. service_started returns preparation or
submission failure; service_stopped returns INI removal or delete submission
failure. The latter becomes QELI_STOP_RESULT, blocking a new instance within a
failed-stop restart/reload. Other ubus calls do not alter this receipt. Successful
submission does not establish VPN connectivity, cleanup completion or acceptance
of configuration by the client itself.

Official rootfs.tar.gz24.10.4 SHA256:
e7f8ab84eef55c7eb23492de7b0517dbc23fbedaa4abbb680fe64720c309dc03.
It contains actual musl procd2024.12.22~42d39376-r1, ubusd/ubus/libuci,
BusyBox ash, rc.common/procd.sh/jshn. Archive/runtime identities are retained.
Each method copies it into an owned0700 chroot on .11 with its own Unix socket.
Procd runs as an ordinary non-PID1 process; normal boot/reboot is never invoked.
Host /etc, system services, network/firewall and .10 remain unchanged.

7 new integration methods PASS without skips: actual start argv/env/private0600
INI; stop removes rendered config and retains the volatile secret; restart/reload
produce a new PID and updated INI; owned child SIGKILL triggers real respawn;
disabled no-instance/enable-disable rc.d symlinks; missing secret/binary admission;
missing procd and missing socket failures for all four verbs. Multiple states
inside a method are not additional checklist criteria. Client executable is an
inert shell receipt followed by exec sleep; /dev/net/tun is only an existence
marker file, firewall UCI is empty and no actual firewall reload runs. Qeli data
plane/NetworkPlan cleanup/WAN/reboot/DNS/firewall and firmware boot are not tested.

Old Git init with the same7 methods gives8 assertion failures: two unavailable
states × four verbs. Initial6-method/4-failure baseline is also retained. Full
OpenWrt69 = previous55 +7rpcd +7procd PASS separately with BusyBox ash/dash;
all optional runtime environment variables are set, no skips. Previously qualified
ucode24.10/fs GNU runtime, rpcd/UCI template, saved rc.common and exact-source INI
inspector are reused with their prior hashes. Procd tests always execute rootfs
BusyBox; earlier shell-dependent fixtures use the selected interpreter. No owned
processes/runtime roots remain.14 Node/37recipes/docs/bindings/diff PASS. Qeli
core/native/router sources/recipes/artifacts are unchanged; no new Rust/FFI/cross
build is claimed for this shell fix. SDK was not run: read-only .11 inventory
showed1.7GB free, so this batch used the small rootfs rather than SDK251MB plus
extraction/feeds/build. SDK remains an outstanding item, not a technical PASS.

Sources: [official rootfs/hash](https://downloads.openwrt.org/releases/24.10.4/targets/x86/64/),
[release procd recipe](https://github.com/openwrt/openwrt/blob/v24.10.4/package/system/procd/Makefile),
[pinned non-PID1 daemon](https://github.com/openwrt/procd/blob/42d3937654508b04da64969f9d764ac2ec411904/procd.c),
[release rc.common](https://github.com/openwrt/openwrt/blob/v24.10.4/package/base-files/files/etc/rc.common).
Packet: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-openwrt-procd-20261006.
Evidence: release/certification/evidence/q31-openwrt-procd-20261006.json.
Actual procd supervision API is qualified within this scope; SDK/package install,
HTTP/config preflight/core generation/ABA/clean A-B remain OPEN; firmware is
USER_EXCLUDED. Q31 IN_PROGRESS28/37(75.7%),9remain. Q29 SIGKILL FAIL/auto-null
ENONET, Q30 Apple exclusions/callback-drain OPEN and D06 ACCEPTED are preserved.

## F353–F354: actual SDK and LuCI package checks (7 October)

F353: unconditional `+ip6tables` creates recursive dependencies for qeli and
luci-app-qeli in the actual24.10.4 SDK with pinned feeds and alternate virtual
providers. `make defconfig` can return0 despite these errors. The dependency is
now `+IPV6:ip6tables`, preserving virtual backend choice and requiring the IPv6
provider only with IPV6. Direct scripts/config/conf and diagnostic checks reject
false-success make results. The old recipe reproduces the cycle; the conditional
recipe removes it.

F354: luci.mk reads LUCI_MAINTAINER/LUCI_URL; the old PKG_MAINTAINER did not
set the IPK owner. The actual old package recorded OpenWrt LuCI community and
the upstream LuCI URL. Both variables are fixed; the new control records Alexandr
and https://github.com/litvinovtd/qeli. The same ownership test fails on the retained
old IPK and passes on the fixed one. Source-hash guidance uses actual SDK metadata:
this SDK produces tar.zst, not the assumed tar.xz. Release-cut hashes remain OPEN.

The official x86/64 SDK24.10.4 archive matches release SHA256
229e871f734a2cee5ce3ad6a3e98d3836b0899bfdeaea4d9c2c5cc7b1fce1407.
Pinned LuCI75e41cba5160281b2d1dca922719efef29f7ffd8, packages Rust
f1470815f41fe199bfdafe9a91ad115bc2e91203 and base v24.10.4 archives/inputs are
retained with hashes. Only required Rust recipe files were fetched from packages;
this is not a full packages feed. GNU awk debs were extracted privately without
host installation. Actual luci-base host-compile built jsmin/po2lmo; the direct
LuCI package compile -j1 produced a genuinely minified IPK. This goal does not
qualify full dependency resolution, Rust bootstrap or the Qeli client build.

`scripts/test_openwrt_sdk.py`:3 SDK methods PASS without skips (actual metadata,
stock IPV6=y, private Kconfig IPV6=n);3 IPK methods PASS without skips and14 existing
LuCI scenarios PASS on packaged minified JS. The SDK fixes its hidden IPV6 symbol
to default y: the initial requested n profile silently became y and is not IPv4-only
qualification. The final n test temporarily changes only owned Config-build.in,
disables ALL/ALL_KMODS/ALL_NONSHARED, checks actual symbols and restores both files.
This is private Kconfig branch coverage, not real IPv4-only firmware runtime.
The IPK has exactly four expected0644 files; RPC/ACL/menu bytes equal Git, while
JS is smaller than source.37 native recipe/14 source LuCI/docs/bindings/diff PASS.

opkg installation into an empty owned offline-root is NOT PASS. After fixing
its lock directory, five runtime dependencies are absent and opkg rejects the
candidate even with force-depends/nodeps/add-arch. Initial and final logs remain;
temporary roots are removed, no fake installed dependency records were created.
Actual installation/resolver qualification remains OPEN. Initial missing awk,
wrong feed environment, missing jsmin and incorrect IPV6=n fixture expectations
remain recorded harness failures, not product defects or successful results.

Only a completed owned Q31 Keenetic compiler cache on .11 was archived into
target-cache.tar.zst. zstd/tar compare PASS, previous qeli-client retained with
its original SHA; the recoverable target cache was removed after verification.
SDK/feed archives, IPKs, inputs and logs are retained. No active owned processes
or temporary install roots; host services/network/firewall and .10 untouched.
Rust/core/native implementation, ABI and published binaries are unchanged;
no fresh Rust/FFI/cross build is claimed.

Cargo.toml declares no rust-version. Pinned SDK Rust1.89.0 cannot be called
incompatible merely because standalone recipes were tested with1.97.0; an initial
minimum1.96 inference was incorrect. Actual SDK compiler compatibility, source
pin/mirror release-cut and clean A/B remain OPEN. HTTP, preflight/core generation/
ABA and administrator replacement races remain OPEN; firmware USER_EXCLUDED.
All41 historic certification cases/acceptance are preserved; the scoped supplement
does not replace their artifact/date/status. Q31 IN_PROGRESS;28/37(75.7%),9 remain.
Q29 SIGKILL FAIL/auto-null ENONET, Q30 Apple exclusions/callback-drain OPEN and
D06 ACCEPTED remain unchanged.

Sources: [official SDK/hash](https://downloads.openwrt.org/releases/24.10.4/targets/x86/64/),
[pinned LuCI package mapping](https://github.com/openwrt/luci/blob/75e41cba5160281b2d1dca922719efef29f7ffd8/luci.mk),
[pinned Rust recipe](https://github.com/openwrt/packages/blob/f1470815f41fe199bfdafe9a91ad115bc2e91203/lang/rust/Makefile).
Raw: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-openwrt-sdk-20261007.
Evidence: release/certification/evidence/q31-openwrt-sdk-20261007.json.

## Q31-V002: actual HTTP/session bridge (7 October)

The prior HTTP API NOT_RUN boundary is now qualified in a limited scope.
Official musl uhttpd2025.07.06~7e64e8ba-r4 and uhttpd_ubus.so from rootfs24.10.4
connect to the previously built native GNU rpcd/ucode/fs/UCI runtime over a private
Unix socket. Each test runs inside fresh .11 mount/net namespaces; HTTP listens
only on namespace loopback. Two owned chroots share only the socket directory
through a private bind mount. No -a authentication bypass. Host /etc, services,
network/firewall and .10 are unchanged.

6 scripts/test_openwrt_http.py methods PASS without skips: anonymous/invalid
session/wrong password denials; reader can read qeli/service status but cannot
access secrets, service mutations or other UCI configs; writer forwards five closed
service verbs and literal Unicode/metacharacter secret over stdin, receives boolean
secret status and clears it; actual session UCI set→apply→confirm changes the
init-visible committed enabled state; embedded ubus_rpc_session impersonation is
rejected; local session revocation and idle expiration deny further HTTP RPC;
HTTP serves exact minified JS bytes from the previous SDK IPK and recovers after
malformed requests. Writes to firewall/rpcd remain denied.

Real packaged Qeli RPC/ACL/menu/JS from the F353–F354 IPK and genuine unmodified
luci-base ACL are used. Mandatory LuCI framework ACL supplies UCI transport methods;
Qeli ACL supplies granular config/object scope. Restricted logins need corresponding
read/write luci-base permissions; luci-app-qeli alone does not grant framework UCI
methods. apply/confirm follows LuCI, rather than substituting HTTP commit. No Qeli
ACL expansion. Initial missing-base-ACL failures, expiry polling that refreshed
sessions, incorrect parse-error expectation and the authorized read receipt were
retained and corrected. Cleanup inventory initially classified saved runtime-template
as temporary; it is excluded by exact path and preserved. No new product defect or
claimed baseline product regression.

All959 official-rootfs regular files/215 links match the prior SHA-verified archive;
28 GNU rpcd template files/19 links and7 transferred Git inputs match. No owned
processes or temporary mount/runtime roots after final run; namespaces exited.
uhttpd/plugin/ACL/status hashes and previous IPK retained.14 source LuCI/37recipes/
docs9/bindings/diff PASS. Rust/core/native/ABI/build inputs and artifacts unchanged;
prior SDK/cross results retain execution dates and scopes.

Service verbs target an inert init adapter, not live VPN/procd. Browser HTML login/
DOM, whole-LuCI TLS/cookies/CORS/CSRF policy, OpenWrt PID1, firmware and real VPN
networking are not qualified by this HTTP API run. Full SDK Rust build/resolver/
install, source release-cut/compiler compatibility, preflight/config/core generation/
ABA and clean A/B remain OPEN.41 historic certification cases/acceptance preserved;
scoped supplement only. Q31 IN_PROGRESS;28/37(75.7%),9 remain. Q29 SIGKILL FAIL/
auto-null ENONET, Q30 Apple exclusions/callback-drain OPEN and D06 ACCEPTED retained.

Sources: [release uhttpd recipe](https://github.com/openwrt/openwrt/blob/v24.10.4/package/network/services/uhttpd/Makefile),
[pinned HTTP/session bridge](https://github.com/openwrt/uhttpd/blob/7e64e8bad2415cec0a6d9770ec379db54273c7a7/ubus.c).
Raw: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-openwrt-http-20261007.
Evidence: release/certification/evidence/q31-openwrt-http-20261007.json.

## F355: Keenetic preflight and launch opened different INI revisions (7 October)

Both templates inspected the editable original INI and later reopened it in an
asynchronous process. The actual retained core inspector plus an owned native
process reproduced atomic replacement immediately after inspection: both baseline
variants checked A and launched B (two subtest failures). Interface/gateway
admission therefore did not describe the launched configuration.

The common lifecycle helper now publishes one bounded private runtime copy before
inspection; both inspection and launch use /opt/var/run/qeli-launch/client.conf.
Original owner/mode are preserved, avoiding command-trust promotion. Original and
runtime paths reject links/nonregular targets; directories must be root-owned0700.
The installer checks tools and an existing config directory's owner before package
updates. Repeated active start retains the copy; restart recaptures the original.
Stop clears it only after verified join and network cleanup. Hook QELI_CONFIG_PATH
points to the copy; manuals now explain editing, recovery and changed path semantics.
No shell INI parser, configuration format/schema or native ABI was added.

8 new methods exercise both templates with actual core preflight and native process
identity: rotation, metadata, path admission, bounded copy, publication failures,
failed join, repeated start/restart and installer failures. Full192 Keenetic methods
PASS without skips in dash and BusyBox ash on .11. The first full run had four
fixture failures per shell: existing PID-publication tests failed every mv and hit
the new earlier config publication. Fault injection now targets the PID destination;
separate new tests cover copy publication failure. Initial full baseline7 methods
produced14failures/12errors for absent new contracts; these are not26 product bugs.
All initial/final logs are retained.37recipes/14LuCI/docs9/bindings/diff PASS.

The core inspector is the previous SHA-bound host binary:315/317 compile inputs
match current Git exactly. Two later portability edits (MIPS atomic fallback and
procfs integer cast) are outside unchanged CLI/parser/inspection; exact diffs verified.
No fresh current-core build is claimed. Only the tiny owned C receipt helper was
built freshly with GCC -Werror.
Filesystem/network command models remain models; no live router, VPN or hook command
execution is claimed. Product Rust/core/native/ABI/artifacts are unchanged; previous
cross results retain their original dates and scopes. Owned fixture cleanup verified;
only private .11 files/processes, no host service/network/firewall or .10 mutation. An older6October model-only fixture was separately archived/compared and removed by exact path; an initial self-matching verifier guard stopped without mutation.

This closes editable-original replacement between Keenetic preflight and launch.
Root/admin replacement of private runtime paths, core plan generation/ABA, OpenWrt
config generation, SIGKILL/power-loss transactions, SDK full build/install/compiler
compatibility/source release-cut and clean A/B remain OPEN. Physical firmware remains
USER_EXCLUDED.41 historic cases/acceptance preserved; scoped supplement only.
Q31 IN_PROGRESS28/37(75.7%),9remain; Q29 SIGKILL FAIL/auto-null ENONET, Q30 Apple
exclusions/callback-drain OPEN and D06 ACCEPTED retained. No push/deploy.

Raw: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-keenetic-launch-config-20261007.
Evidence: release/certification/evidence/q31-keenetic-launch-config-20261007.json.

## F356–F357: Cargo config bypassed compiler and private intermediates (7 October)

F356: cargo +pin pins Cargo, but unset RUSTC/CARGO_BUILD_RUSTC allowed Cargo's
build.rustc config to select another executable. F357: setting CARGO_TARGET_DIR
alone left build.build-dir/CARGO_BUILD_BUILD_DIR free to write intermediates outside
the owned target. The host gate also used the default toolchain and ambient flags.
Both router variants and the captured actual host-gate client command reproduce
these defects with real Cargo on dependency-free private Linux probes: configured
compiler adapter changes the compiled receipt, and config/environment redirect
intermediates. Final same-harness baseline4methods:9 expected subtest failures,
no errors; these represent two defects, not nine independent product bugs.

Shared router_toolchain now resolves and checks rustc from the selected pin before
Cargo and explicitly sets RUSTC plus both output directories. Both maintainer
helpers consume it; keenetic_verify uses the same environment with Rust1.97.0,
including test/Clippy/graph commands. The four-target router CI step follows it.
Missing/nonexecutable compiler lookup fails before Cargo. Existing target-specific
flags and MIPS nightly remain. Manuals explain the named compiler/Clippy prerequisite
and scoped limits. Project Rust/native/schema/ABI/published artifacts are unchanged.

59 related router methods (7 real-Cargo runtime methods, one new host-prefix model)
and19 host-gate model methods PASS without skips on .11. Nine fixed compile/execute
receipts report pinned rather than redirected; actual tiny test has1PASS, Clippy -D
warnings and graph succeed under the configured compiler trap. This validates command
selection, not Qeli's own tests/dependency graph. Both failure guards cover all three
roots. Real rustc/cargo1.97.0/Clippy identities and rustc SHA retained. PyYAML6.0.3
parses actual CI;16 rendered four-target shell steps pass bash -n.37recipes/14LuCI/
docs9/bindings/diff PASS. No GitHub Actions, actual Zig or fresh full Qeli/cross/SDK
build is claimed. .11 had532MiB free and2GiB RAM; probes need no dependency download.

Initial raw preparation stopped for a missing local baseline directory before tests;
first baseline had8FAIL/1harnessERROR because tempfile emitted an underscore refused
by production root admission. Owned mktemp naming fixes it; all logs retained.
One test-source CRLF/Git mismatch was retained, normalized to canonical bytes and the59-method suite repeated. Final17 transferred Git inputs verified; no probe processes/run roots remain,
private CARGO_HOME only. Host services/network/firewall/global config and .10 untouched.

This closes ordinary build.rustc and config/environment intermediate-directory
redirection in these commands. Full hermetic Cargo home/ancestor configuration,
forced env/profile/source/linker/PATH/cache policy and clean independent Qeli A/B
remain OPEN. Source release-cut/full SDK build/install/compiler compatibility,
core plan generation/ABA/admin replacement and SIGKILL/power-loss transactions
remain OPEN; firmware USER_EXCLUDED.41historic cases/acceptance preserved; scoped
supplement only. Q31 IN_PROGRESS28/37(75.7%),9remain; Q29 SIGKILL FAIL/ENONET,
Q30 callback-drain OPEN/Apple exclusions and D06 ACCEPTED retained. No push/deploy.

Reference: [Cargo configuration](https://doc.rust-lang.org/cargo/reference/config.html).
Raw: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-router-cargo-20261007.
Evidence: release/certification/evidence/q31-router-cargo-20261007.json.

## Q31-V003: actual offline IPK installation, upgrade and removal (October7)

Seven actual opkg2024.10.16~38eccbb1-r1 integration methods PASS without skips
in the official OpenWrt24.10.4 x86/64 rootfs on .11. Qeli and the SDK LuCI IPK
install with genuine nft and legacy providers of virtual iptables/ip6tables.
Exact payload bytes, root ownership, executable modes, UCI0600 and native
qeli-client --version are verified. No force-depends or fabricated installed
records: initial state is the official rootfs;22 missing packages come from
matching release core/base/kernel feeds. Actual usign validates all3 index
signatures using rootfs release keys; every IPK SHA matches its signed index.

Upgrade uses only an audit0.8.2-r2 fixture with a changed default comment, built
by actual SDK ipkg-build. Product version is unchanged. Modified UCI survives;
new defaults appear in qeli-opkg, both0600. Unmodified defaults are replaced.
Removing Qeli with LuCI installed is refused. Removing LuCI then Qeli deletes
binary/init/defaults/RPC/ACL while retaining modified UCI0600. Missing dependencies
and a truncated archive are refused without changing installed status/client
payload. These early admission failures do not establish power-loss atomicity.

Actual SDK Package/qeli/install and BuildPackage/ipkg packaging run. An explicit
audit overlay disables source download/prepare/configure/Rust compile and supplies
the previous SHA-bound standalone musl binary built October6. Original recipe,
overlay, artifacts and logs are retained; original SDK recipe is restored. There
is no fresh SDK Qeli Rust compile: source/mirror release-cut and SDK Rust1.89
compatibility remain OPEN. Published artifacts/Rust/core/native/ABI are unchanged.

Initial harness errors are retained: empty Download/default in packaging overlay,
copytree over existing symlinks, missing kmods feed, malformed lists_dir and missing
lock directory. Initial suite5PASS/1FAIL/1ERROR came from error-text expectations
and offline-root / breaking conffile lookup. Correct /target inside private chroot
resolves both; final suite7PASS. Initial r2 packaging emitted a locale timestamp
warning; it was rebuilt with LC_ALL=C/fixed epoch, then all7 tests repeated.
No new product defect was found.

Operations use owned rootfs and mount/network/PID namespaces, without host
proc/dev mounts, module loading, VPN startup or host service changes. Offline
opkg does not qualify firmware postinst/fw4/procd. Physical-router WAN/reboot/
DNS/firewall/performance USER_EXCLUDED. Full SDK Rust build, core plan generation/
ABA/admin replacement and full hermetic clean Qeli A/B remain OPEN. Q31 IN_PROGRESS,
28/37(75.7%),9remain; Q29 SIGKILL FAIL/auto-null ENONET, Q30 callback-drain OPEN/
Apple exclusions and D06 ACCEPTED are preserved.

Raw: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-openwrt-opkg-20261007.
Evidence: release/certification/evidence/q31-openwrt-opkg-20261007.json.
Upstream reference: [opkg conffile removal](https://lxr.openwrt.org/source/opkg-lede/libopkg/opkg_remove.c).

Verified959 official rootfs files/215 links and6 transferred source inputs. All317
previous fixed musl build inputs match current Git; the existing router crate-type
rlib derivation retains its original scope. SDK strip transforms the binary SHA:
input ec567377e46cd8a2ed629bccfb83318fba4e19bf26905df3f49ca77c254a5925, packaged
915b113abcb47ce37f40c79be70927b8d330217cb075ae161fc36ae9fffeac24; both retained.
37recipes/14LuCI/docs9/bindings PASS. No final owned processes/case/smoke/mount
roots; host services/network/firewall and .10 untouched. No push/deployment.

## F358 / Q31-V004: SDK compiled unused FFI libraries (October7)

Current client-bin was freshly tested using actual Rust1.89.0, the pinned SDK feed
version. GNU cargo check --locked --offline --release PASS. Full musl cargo install
with official SDK24.10.4 GCC13.3.0 and actually expanded SDK flags/profile PASS.
These are prebuilt upstream rustc/std, not source-built SDK Rust bootstrap.
Compiler/linker/locked dependencies/source inputs are recorded. Owned .10
RUSTUP_HOME/CARGO_HOME/target and jobs1; global defaults/cache unchanged. SDK GCC
and its bundled host loader/libs use private paths; both archives SHA/tar compared.

F358: baseline actual cargo install built rlib, cdylib and staticlib although only
the CLI is installed. Both unused outputs are confirmed in compiler argv/artifacts.
Build/Compile now checks lock and projects only private SDK Cargo.toml to rlib;
unknown declaration/missing inputs refuse Cargo. Repository manifest/Rust/core/
schema/native ABI unchanged. Make $$ preserves sed's regex anchor. Both production
macros are expanded by actual SDK; exact output and explicit private compiler/path
substitutions retained. Tiny actual Cargo/musl baseline1expected FAIL due to .a/.so;
fixed5methods PASS without skips, including existing rlib, unknown declaration and
missing lock/manifest. Current full client then compiled again using fixed macro,
without replacing Compile with a copy. Only rlib/rmeta, no unused FFI outputs;
source manifest restored afterwards. Dependency cache reused, not independent
clean A/B. SDK3 Kconfig/metadata methods PASS with fixed recipe.

Baseline/fixed ELF each pass9 executable cases in official24.10.4 rootfs: --version,
legacy/core ownership, BOM/quotes, attached TUN, wrong device/attach, TAP and malformed
INI refusals.18 executions, not18 unit methods. Verified ELF64/x86-64 PIE, interpreter
/lib/ld-musl-x86_64.so.1, libc.so/libgcc_s.so.1, BIND_NOW/RELRO and NX stack. Owned
mount/network/PID namespaces, no host proc/dev mounts. No VPN/TUN/firewall startup;
physical firmware USER_EXCLUDED. ELF/projection/inputs/logs retained; published
binaries unchanged. No global all-feature/platform MSRV or SDK Rust bootstrap/
source-download/target-matrix qualification is claimed.

Cargo warns about locked der0.8.0; primary RustCrypto changelog explains the yank
as a minimal-versions CI correction. Lock stays unchanged; dependency stack review
belongs to Q34. This is not an established vulnerability or compiler failure.
Initial standalone GCC lacked its bundled SDK host libs; correct runtime was
transferred before compilation and diagnostics retained. Rust1.89 compatibility
is confirmed for the tested client-bin/x86 path. Helper1.97/MIPS nightly results
retain their original dates/scope.

Raw: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-openwrt-compiler-20261007.
Evidence: release/certification/evidence/q31-openwrt-compiler-20261007.json.
Primary yank source: [RustCrypto der changelog](https://github.com/RustCrypto/formats/blob/master/der/CHANGELOG.md).
Q31 IN_PROGRESS28/37(75.7%),9remain. Full SDK bootstrap/release pipeline, core plan
ABA/admin replacement and full hermetic Qeli A/B OPEN; Q29 SIGKILL FAIL/ENONET,
Q30 callback-drain OPEN/Apple exclusions and D06 ACCEPTED preserved. No push/deploy.

Final provenance/cleanup:364 Git crate inputs verified before/after the full build;
3228 compiler/SDK tool files and links inventoried. Both SDK recipe copies match
current source. Official rootfs959regular/215links verified against its archive.
Private toolchain/cache/build dirs and runtime rootfs removed; no remaining owned
processes, mounts or compile-case dirs. Global Rust toolchain inventory unchanged.
Free disk after cleanup:.10=2588MiB,.11=467MiB.37 recipe methods,14 LuCI fixtures,
documentation9 checks, bindings and diff checks PASS. No host services/network/
firewall changes.41 historic certification cases preserved; scoped supplement only.

## F359–F360: repeated start and rejected OpenWrt preparation (7 October)

F359: init atomically rewrote client.conf but submitted the same command/path to
procd. Repeated start with a changed server or password returned success while
retaining the process with old data. The instance now declares INI and password
paths through procd's file parameter. Native content comparison replaces the
instance when data changes; rendering identical bytes preserves its PID. Writing
a secret alone does not restart the VPN; subsequent start/restart/reload applies it.

F360: rc.common closes and submits the service description even when start_service
fails. An empty set removed the running instance after malformed MTU, invalid dev
or missing server. The ubus wrapper now refuses set/add following unsuccessful
preparation and returns its error. Intentional disabling and stop/delete retain
their normal behavior.

Actual procd 2024.12.22~42d39376, ubus, UCI and rc.common from official OpenWrt
24.10.4: baseline yields five expected failures across three methods; fixed suite
passes all 11 methods without skips. Four new methods cover changed INI/password,
idempotent start and preservation of process/INI after three render refusals. Seven
existing methods cover stop/reload/respawn/autostart/transport failures. Related
55 shell/renderer/dispatcher/ucode methods pass on dash and BusyBox ash. Retained
INI inspector and ucode identities are SHA-verified; shared parser source matches
current Git. Rust/FFI rebuild and published binary replacement are unnecessary.

Owned mount/network/PID namespaces use their own procfs for correct PID checks;
no host proc/dev mounts enter the chroot. Client is an inert receipt/sleep adapter,
TUN is an existence marker. No firewall or real VPN starts; procd is non-PID1. Initial
CRLF, mismatched procfs/PID namespace and import-path diagnostics are retained as
preparation errors, not product failures. This is not an atomic UCI/firewall/INI/core
plan transaction: rollback after successful render, core generation/ABA and admin
replacement remain OPEN.

Mechanism checked against [the exact procd source version](https://github.com/openwrt/procd/blob/42d39376/service/instance.c).
Raw: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-openwrt-config-generation-20261007.
Evidence: release/certification/evidence/q31-openwrt-config-generation-20261007.json.
Q31 IN_PROGRESS; 28/37 (75.7%), 9 remain. Q29 SIGKILL FAIL/auto-null ENONET,
Q30 callback-drain OPEN/Apple exclusions and D06 ACCEPTED preserved. No push/deploy.

Final: 10 source inputs and 4 retained artifacts SHA-verified; rootfs 959 files/215
links rechecked and owned template removed. No residual owned processes, mounts
or runtime case directories. 37 recipe checks, 14 LuCI fixtures, documentation
9 checks, bindings and diff PASS. Historical 41 case/acceptance records preserved;
scoped supplement added. .10 unused.

## F361: runtime NetworkPlan file generation (7 October)

The core published addresses and MTU only. Reauthentication with identical addresses
produced identical bytes, allowing Keenetic to treat the previous APPLIED receipt
as current. A→B→A during ndm apply/save was also invisible to file comparison.
The Linux writer now appends `generation=<NetworkPlan.generation>` after MTU,
preserving the first address line and existing named fields. It uses the core's
existing authenticated generation; no new INI parameter or JSON configuration.
Existing whole-file hook checks and APPLIED receipts automatically include it.

This distinguishes identical addresses and ABA within one adapter's monotonic
plan generations. The counter is not global: adapter/process reset, writer/shell
lock coordination, changes after the final check, atomic ndm configuration,
removal of old address families and root/admin replacement remain OPEN.
Legacy runtime files without generation remain readable with their old guarantees.

On .10, Rust1.97 with private Cargo home/target, one job and offline/locked:
a derived baseline (same formatter seam/tests, only the generation line omitted)
reproduced 2 assertion FAIL and 1 PASS; fixed passed 11 lifecycle unit tests,
client-bin GNU check, production lib/bin Clippy `-D warnings` and fmt.
Three new unit methods cover legacy/IPv4, same-address reauth/ABA/u64 max
and dual-stack/IPv6-only. The baseline is not an unchanged original source file.
376 Git/current compile inputs were verified before/after; the private rlib
manifest projection was restored. No fresh FFI/SDK1.89/cross or published binary build.

On .11,194 Keenetic methods without skips passed on each dash/BusyBox ash
(192 previous plus 2 new). New methods cover reapplying identical addresses with
a new generation and A→B→A during apply/save: stale receipt refusal, pending
retention and successful retry of the new generation. ndmc is modelled; retained
native helper and metadata inspector retain their scoped roles, and the inspected
current core region matches the retained binary's source. No firmware/VPN claim.
37 recipes,14 LuCI fixtures,9 docs checks,bindings and diff checks passed.
Initially missing conformance inputs and a wrong-edition formatter invocation
are retained as preparation errors; the final gate uses edition2021.
Owned .10 compiler/cache directories were cleaned after preserving results.

Raw: `audit-debt-20260924/q31-plan-generation-20261007/`; seal, input SHA,
commands, times and results are recorded in evidence. 41 historic cases/acceptance
are retained. Q31 IN_PROGRESS; Q29 SIGKILL FAIL/auto-null ENONET, Q30 callback drain
OPEN/Apple exclusions and D06 ACCEPTED remain unchanged. Total28/37(75.7%),9 remain.

## F362–F363: late stale receipts and a new process generation (7 October)

F362: cached no-op used a plan snapshot preceding `ndmc show` without rechecking
it afterwards. After the last pre-receipt check, `mv APPLIED` and `rm PENDING`
could also overlap independent plan/marker publication. The hook returned success
for stale state. A shared check now rereads marker/plan before no-op, after receipt
publication and after pending removal. Observed replacement returns failure and
preserves/recreates pending for the next event. An old receipt file may remain
after `mv`; pending prevents it qualifying no-op. This does not undo ndm commands
already executed or queue events.

F363: fresh start after the old client exited removed the plan but retained APPLIED.
The new adapter's local counter starts at1, allowing identical plan/address bytes
to reuse the previous process receipt. OpkgTun init now removes APPLIED before
fresh launch, after successful INI/owner inspection and before PID/process creation.
Removal failure refuses launch. Start of an already verified live client preserves
the receipt; failed preflight preserves recovery state. This step does not remove
incomplete pending. Base init and Rust writer are unchanged.

On .11, the same5 new methods reproduced baseline8 assertion failures in4 methods
on each dash/BusyBox ash: show×plan/marker(2), mv/rm×plan/marker(4), inheriting the
exited process receipt(1), ignoring invalidation failure(1). Preflight preservation
already passed baseline. Fixed199 methods per interpreter without skips passed
(194 previous +5 new). Coverage includes successful retry, no false success log,
repeated start without restarting the live native helper and directory-receipt
launch refusal preserving canary/old plan. Processes and /proc identity are real;
ndmc/network are models. Generation=1 after launch is fixture publication: the
C-helper qualifies process admission, not a new Rust handshake. Retained metadata
inspector stays in its previously qualified role.382 Git/current inputs and SHA2
retained artifacts verified before/after; no owned process/mount/temp roots remain.
37 recipes,14 LuCI,docs9,bindings/diff passed. No fresh Rust/FFI/cross/SDK build or
firmware/VPN qualification.

Counter reset is closed within the updated OpkgTun init's fresh-launch protocol.
Direct binary launch/adapter replacement outside it, writer/shell lock coordination,
replacement after the final check, atomic ndm and old address families remain OPEN.
The core writer cannot simply acquire the shell lock: init holds it during start,
and its legacy branch can wait for plan publication, creating a wait cycle without
protocol changes. Generation and bounded checks do not promise synchronization.
Raw: `audit-debt-20260924/q31-keenetic-receipt-20261007/`; source SHA, commands,
times, baseline/fixed and seal in evidence.41 historic cases/acceptance and34 previous
supplements preserved. Q31 IN_PROGRESS,28/37(75.7%),9 remain;Q29/Q30/D06 retained.

## F364: ambient inputs changed the standalone release profile (7 October)

Pinned Cargo/rustc and cleared RUSTFLAGS did not pin profile.release. Ambient
variables from an adjacent FFI build (such as CARGO_PROFILE_RELEASE_PANIC=unwind),
Cargo home and ancestor config could override the five explicit qeli/Cargo.toml
fields: lto=fat,codegen-units=1,opt-level=3,strip=true,panic=abort. Builds succeeded
with different panic/optimization/size. This is reproduced build contamination,
not an observed crash or a new performance measurement.

Shared router_toolchain now supplies these five CARGO_PROFILE_RELEASE_* values
to child Cargo. Both standalone helpers and the Linux host gate use it.
`--print-release-profile` exports the same set for a separate keenetic-cross CI
step with working-directory=.; other jobs, FFI unwind and SDK feed profiles stay
unchanged. A new canonical Cargo.toml comparison checks exporter/helper/CI agreement.
[Cargo documents environment precedence over config](https://doc.rust-lang.org/cargo/reference/config.html#environment-variables);
actual lab tool behavior is qualified by the fresh checks below.

On .10 Rust1.97: the same new method exercises12cells (Keenetic/OpenWrt/host ×
environment/Cargo home/owned ancestor/forced child env). Baseline9 assertion FAIL:
the program printed unwind|0 instead of abort|3; forced child env3cells already PASS.
Fixed12cells all passed and five actual rustc flags were checked. The full real-Cargo
suite8methods without skips passed (7 previous +1 new).72 router/host-gate Linux
methods without skips passed (53 helper +19 gate, including the new manifest/export/
CI contract). Initial Windows checks skipped10 POSIX methods; Linux ran those methods.

Actual cargo-zigbuild0.23.0/Zig0.13.0/stock x86_64-musl std1.97 also executed the exact
router_build_command on two separate dependency-free mini-crates: baseline unwind|0;
fixed abort|3 and five compiler flags PASS. Both native static ELFs passed the shared
target validator and were actually executed. These are small probes, not Qeli,
not clean identical A/B builds or a new four-target/current-Qeli ABI qualification.
Each has private HOME/Cargo/target/Zig/XDG caches, removed after preserving ELF/logs.
Pins/global caches/defaults/services/network and .11 were unchanged.372 primary+1
supplemental Git/current inputs verified; source/tool SHA, commands, results and
timestamps in raw.37recipes/14LuCI/docs9/bindings/diff passed. CI exporter/placement
was locally checked; no new GitHub Actions run was dispatched.

Full hermetic policy (other profile keys, forced env, source/linker/PATH, registry/
tool caches), clean Qeli A/B and SDK bootstrap/source release-cut remain OPEN.
This qualifies five explicit standalone fields, not full Cargo isolation.
Raw: `audit-debt-20260924/q31-router-release-profile-20261007/`; seal/scoped evidence.
41 historic cases/acceptance and35 previous supplements preserved. Q31 IN_PROGRESS,
28/37(75.7%),9 remain;Q29 SIGKILL FAIL/auto-null ENONET,Q30 callback-drain OPEN/
Apple exclusions,D06 ACCEPTED retained. Published binaries were not replaced.

## F365 / Q31-V005: independent builds depended on temporary source paths

Two full Qeli x86_64-musl builds used the exact shared standalone recipe, Rust
1.97.0, cargo-zigbuild0.23.0 and Zig0.13.0. Each started with a fresh target and
Zig/XDG cache, private HOME/Cargo home and an identical offline copy of the existing
registry. Compilation was sequential with one job. All364 crate inputs were
identical and remained unchanged; this was a full client build, not a mini-probe.

Both baseline builds succeeded and produced5582856-byte static ELF executables,
but96458 bytes differed. Each embedded227 occurrences of its random mktemp source
root. Merely substituting strings did not make them equal; the failure was retained.
The shared router/host helper now passes `--remap-path-prefix=<owned root>=/usr/src/qeli`.
This changes compiled source locations, including `file!()` and panic diagnostics;
it does not create that directory or change configuration paths. MIPS soft-float
flags are preserved. Canonical manifests, Rust/core/FFI and SDK/CI recipes are unchanged.
See the [rustc option](https://doc.rust-lang.org/rustc/command-line-arguments.html#--remap-path-prefix-remap-source-names-in-output).

The same new actual-Cargo method reproduced three baseline failures, one per
Keenetic/OpenWrt/host caller; all9 fixed actual-Cargo methods and72 Linux helper/gate
methods PASS without skips. Two fresh full fixed builds are byte-identical:
5577864 bytes, SHA256 `5132852d8b56ec588a503d52b4cc6cc4aa959d2c537e6ed47a2daa4518c2427b`.
Neither ELF contains its private build root. Actual ELF admission, help/version
and24 INI metadata cases per pair PASS (48 across baseline/fixed), without connecting.
Five canonical release flags, no unused FFI outputs and fresh compilation were checked.
Dependency warnings and the deprecated linker-optimization warning are retained,
not suppressed. Global registry/tool contents stayed unchanged; owned roots were
removed and no owned process/mount remained. Published binaries were not replaced.

This qualifies clean independent x86_64 standalone A/B using a verified existing
registry seed. Other-ISA A/B, arbitrary Cargo/config/linker/PATH/cache hermeticity,
SDK bootstrap/source release-cut and firmware runtime are not qualified here.
Firmware remains USER_EXCLUDED. Q31 remains IN_PROGRESS,28/37(75.7%); Q29 SIGKILL
FAIL/auto-null ENONET, Q30 callback-drain OPEN/Apple exclusions and D06 ACCEPTED remain.
Packet: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-router-clean-ab-20261007.
Evidence: release/certification/evidence/q31-router-clean-ab-20261007.json.

## Q31-V006: independent full builds for the remaining three router targets

Six fresh full Qeli client builds qualify two independent builds per target.
AArch64/ARMv7 use Rust 1.97.0; MIPS uses nightly-2026-06-10 and actual
`build-std=std,panic_abort`. All use the shared pinned standalone recipe, one job,
sequential execution and offline identical verified registry copies. Each has a
separate owned source root, HOME/Cargo home, target and Zig/XDG/TMP caches.
All 364 crate inputs stayed unchanged; the temporary rlib projection was restored.
Verbose logs confirm the five release fields, source-path remapping, fresh
compilation and absence of unused FFI outputs/private source roots in the ELF.

| Target | Bytes per ELF | SHA256 of both independent builds |
| --- | ---: | --- |
| aarch64-unknown-linux-musl | 4820920 | `6a0ae5622a9dd4ce58289e2ab0066d0d1600e1b6dec0269cf1199209452ea62a` |
| armv7-unknown-linux-musleabihf | 4984836 | `2a445c055db3ffc23f26e3adb59c8ecb8e9fe89ae6f74f251c8437530b3fe812` |
| mipsel-unknown-linux-musl | 6849048 | `591f593741b167832aa2bc7b06f9e3476ea336e8195246920915b0367ef4deb9` |

All three pairs are byte-identical and pass shared static target-ELF admission.
The earlier F365 x86_64 pair is reused only after verifying its entire 74-file seal
and equality of all 380 current compiled-packet inputs with that fixed packet.
Together these qualify independent A/B for all four standalone router targets;
they do not claim a new x86_64 execution or complete Cargo hermeticity.

Every new ELF ran help/version and 12 owner/interface INI metadata cases under
private QEMU user mode: 12 CLI calls and 72 metadata cases PASS, no skips.
The three QEMU 10.0.2 executables were privately extracted from the official Debian
security package, whose 71008176-byte archive matched the SHA from existing apt
metadata before extraction:
`ac60ebf92f482731e6a89ff5428c805a74f3e66c062670b89c81d2f7512d6c38`.
No package, binfmt handler or system emulator was installed. The first URL from
stale main-repository metadata returned 404; this preparation failure is retained.
[QEMU user mode](https://www.qemu.org/docs/master/user/main.html) runs the target
userspace against host kernel services; these CLI/parser checks do not qualify
router kernels, CPU-specific crypto, firmware, TUN, firewall or a VPN connection.

All six compiler roots were removed; no owned runner/process/mount remained.
Global registry/toolchain/nightly-source contents were unchanged. Own source,
ELF and tool evidence remain in the inspection packet. Build warnings are retained;
no new full-cross Clippy or performance benchmark is claimed. Published binaries,
Rust/core/ABI, manifests and lockfiles were not changed.

## F366–F367: release preflight bypassed the common router build policy

F366: the old source fingerprint removed every CR byte, including binary font
bytes. Two different real fixtures therefore produced the same fingerprint.
The old gate also accepted dirty source inputs with only a warning. Both failures
were reproduced by executing the exact baseline function/gate AST, using real
binary fixture files and explicit SSH/subprocess models; this was not a live old
release-preflight execution.

F367: gate 2 built in a shared checkout with unpinned default Rust/rolling nightly,
without the common locked one-job recipe, release fields, path remapping, rlib
projection or strict ELF admission. A configured lab with missing Paramiko could
silently skip, and a modeled execution error left the SSH connection open.
Baseline command captures, skip result and leaked-close result are retained.

`router_release_gate.py` now freezes one committed `qeli/` Git archive, preserving
binary bytes, and uploads it to an owned checkout through the existing source
helpers. Dirty source or participating recipe/helper inputs fail before SSH.
Gate 2 uses the common pinned toolchains/build command, rlib projection and strict
static target-ELF admission; the fetched artifact must match its remote SHA256.
A configured lab failure fails preflight; the caller closes SSH in `finally`.
`QELI_LAB_SRC` is obsolete. Gate 2 remains optional when `QELI_LAB_PASS` is absent.
No artifact is published; the private remote checkout is retained for inspection.
Tool readiness follows the existing helper and may install missing named pins
when the maintainer runs the gate; no such installation occurred in this audit.

Final Linux validation: 15 unique gate methods using real temporary Git repos,
binary CR/NUL bytes, immutable snapshots and explicit transport/ELF models;
72 existing related methods PASS without skips. All 750 final model inputs were
verified before/after execution. CI now invokes the new gate tests in the native
job; the exact command was also checked locally. Initial test-fixture escaping
and inherited duplicate-method preparation were corrected before final evidence.
The six full builds qualify the shared compiler recipe. The new caller wiring
is covered by these models, not a full live preflight/GitHub Actions dispatch.

Full SDK Rust bootstrap, source-pin/mirror release-cut, arbitrary environment/
Cargo/linker/PATH/cache isolation and remaining Q31 source/dead-code review remain
OPEN. Physical firmware stays USER_EXCLUDED. Q31 remains IN_PROGRESS; overall
28/37 (75.7%), nine remain. Q29 SIGKILL FAIL/auto-null ENONET, Q30 callback-drain
OPEN/Apple exclusions and D06 ACCEPTED remain unchanged. No push/deploy.
Packet: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-router-cross-ab-20261007.
Evidence: release/certification/evidence/q31-router-cross-ab-20261007.json.

## F368–F369 / Q31-V007: SSH receive-window and diagnostic connection ownership

Four exact baseline implementations (native_lab, lab_common, Keenetic/OpenWrt
build helpers) read stdout to EOF before stderr. A real SSH command on .10 wrote
3 MiB stderr before 64 KiB stdout and reproduced timeout in all four callers.
The fixture had an external timeout; the test harness closed failed baseline
channels. No services, firewall or network settings changed.

F368: shared `ssh_command.run_ssh_command` combines stderr into stdout through
Paramiko, sends EOF to unused stdin, drains output before exit status and closes
only the command channel in `finally`. Four separate sequential-drain implementations
were removed. Whitespace/tuple/exit-code contracts are retained. Output is combined
in arrival order; stdout-then-stderr ordering is no longer promised.
[Paramiko contract](https://docs.paramiko.org/en/stable/api/channel.html#paramiko.channel.Channel.set_combine_stderr).
Timeout remains a Paramiko operation timeout. Channel closure does not prove
remote-process termination and does not add a total wall-clock deadline.

F369: connect_lab closes an allocated client after connection failure and preserves
the original error even if close fails. The diagnostic probe closes SSH in finally
and checks only installed named pins. Rolling nightly/default Rust/Cargo proxies
were replaced with guarded inventory/pinned-executable queries. Exact old AST
models reproduce missing close on connect/probe error; no rolling-nightly download
or deliberately failed lab authentication was attempted.

Fixed code passes 16 actual SSH checks across four callers: all pressure bytes,
stdin EOF, nonzero exit, timeout and closed command channels. The same connection
remained usable after four timeouts. All eight actual probe commands pass; installed
toolchain/package inventories and tool hashes match before/after. Linux:35 native
plus90 router methods PASS without skips, including seven new transport and three
probe methods. All764 model inputs verified before/after. Local native contract78
(including37 recipes), release-gate15, docs9 and bindings PASS. The Windows probe
suite has one expected POSIX skip; qualification uses Linux without skips. CI
includes transport through native tests and an explicit probe command; no new
GitHub Actions run was dispatched.

## Q31 source review reconciliation

28 router-slice files reviewed:21 production/declaration files,three configuration
examples and four manuals. Coverage includes UCI/INI publication, volatile secrets/
ACL/LuCI, procd lifecycle, Keenetic PID/join/legacy recovery, OpkgTun plan/receipt,
SDK package declarations, standalone source/tool/build/artifact helpers and
maintainer gates. Shared transport/connect/recipe-identity changes are reviewed
separately. External rc.common/procd callbacks, rpcd methods, LuCI entry points and
Make macros remain; a single text reference was not treated as dead-code proof.
Thin compatibility wrappers are used by entry points/tests. No further confirmed
unused production implementation was found; separate SSH implementations now use
one helper. Source inventory/hash/classification retained. The Review/dead-code
criterion is complete in this scope. The other four Q31 criteria retain their
runtime/pipeline limits; this is not full Q31 PASS.

The new SSH helper is included in native recipe inputs and release-gate dirty
admission. Native provenance was already STALE at the base commit and remains
STALE: sources/recipes do not match published cores. PROVENANCE, A/B evidence and
binaries were not rewritten; full native A/B/provenance refresh remains mandatory
Q34 release work. Core/Rust/ABI, router compiler flags and artifacts unchanged;
no new Rust/cross build claimed. Certification retains41 historical cases/acceptance
and38 prior supplements; scoped evidence does not imply green CI. No owned pressure/
model processes or test temporary roots remain; own fixture/source packet retained.
SDK bootstrap/source release-cut, arbitrary hermetic policy and writer/lock/atomic
ndm remain OPEN; firmware USER_EXCLUDED. Q31 IN_PROGRESS,28/37(75.7%),9remain;
Q29/Q30/D06 retained. No push/deploy.
Packet: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/q31-router-ssh-20261007.
Evidence: release/certification/evidence/q31-router-ssh-20261007.json.


## F370–F371: safe native preflight before provenance refresh

October7: real lab capacity blocks fresh native A/B: about2.3GiB free on .10,
450MiB on .11, versus the required8GiB. Installed Rust1.97.0/targets, desktop
Zig0.13.0/cargo-zigbuild0.23.0/GNUld2.44/rcodesign0.29.0 and Android
NDK26.3.11579264/cargo-ndk4.1.2 passed actual inventory without installation.

F370: old reset removed pass A/B before checking free space. The exact baseline
function on .10 removed two tiny retained fixture files, then refused low disk.
The fixed function refused on the same host and preserved both files. Capacity
is now checked on the build root if present, otherwise on the existing `/var/tmp`
parent, before mkdir/rm; malformed
inventory also prevents cleanup. This is preflight, not space reservation or
protection against concurrent disk consumption.

F371: desktop/Android source sync moved from shared `/opt/qeli-src` and
`/root/qeli-src` into separate `desktop-source`/`android-source` under owned native
build roots. All three Windows/macOS/Android Cargo commands use `--jobs 1` for
2GiB hosts; no OOM reproduction or RSS ceiling is claimed. Concurrent runs of
the same group are unsupported; no new global lock or general hermetic policy.
Services and global tools were unchanged.

36 native Linux methods PASS without skips;80 local native methods PASS,
including actual capture of three build commands and malformed-inventory refusal
before cleanup.764 exact model inputs checked before/after. Initial Linux
discover80 had20 missing-input errors for the broader GUI/recipe suites in the
incomplete packet; that preparation result is retained separately. Linux
qualification uses four complete native helper suites; full80 passed locally.

Owned old lab data is saved in two local archives:18,231 entries from .10 and
138,114 from .11,plus metadata for918 inactive sockets. All regular-file bytes,
links and FIFO metadata match; remote inventories before/after match, and
decompressed secret scans found no password. Before possible cleanup, automatic
review rejected broad removal without separate approval of exact paths; no
removal occurred. Archive outcomes are recorded in the raw packet; inactive
Unix sockets retain metadata separately, while tar supports FIFO metadata.
Until capacity is recovered and real fresh A/B succeeds, native provenance
remains STALE and release-blocking Q34 stays open. PROVENANCE/evidence/binaries
were not rewritten. No fresh native compilation, export/signature gate or
emulator runtime is claimed. Q31 IN_PROGRESS,28/37(75.7%),9remain;Q29/Q30/D06 and
platform exclusions retained.
Packet: C:/Users/litvi/OneDrive/Documents/qeli/audit-debt-20260924/native-refresh-20261007.
Evidence: release/certification/evidence/native-preflight-20261007.json.
