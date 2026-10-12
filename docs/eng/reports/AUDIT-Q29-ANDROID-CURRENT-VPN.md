# Q29: current VPN integrations and Android fixture reliability

<!-- normative-sync: q29-current-vpn-v1 -->

12 October2026. Q29 IN_PROGRESS;32/37 (86.5%),5 sections open.

Seven current matching-R8 instrumented VPN integrations PASS on Android14/API34
x86_64. CLI52511ceb…, APK77c12051…/JNI01da54a3… used;126 canonical Android build
inputs match. Non-debuggable target and native bytes match qualified current
builds. Unchanged JVM/native A/B not replayed; no new large compiler caches.

| Scenario | Fresh result |
| --- | --- |
| TCP, UDP fake-TLS, QUIC × split/full | 6 actual instrumented PASS |
| IPv4/IPv6 × TCP/UDP sockets | 72 independent server echo receipts, exact size/SHA; TCP16384B and UDP32/257/1024B |
| System DNS through VPN | 6 unique names, A/AAAA question/answer receipts with TUN source |
| Full kill-switch without system lockdown | 1 actual refusal PASS; not qualification of enabled lockdown |
| Final service/TUN/AndroidRuntime | No service/TUN; no FATAL EXCEPTION |
| Guest memory | Requested1536MiB/explicit lowram, MemTotal=1505648KiB |

Final gate duration:130.93s. Ordinary sockets without process binding
or explicit Network.bindSocket in routed cases retain source/readiness preflight.
Instrumentation uses target-app context: this does not qualify a separate
non-VPN-owner UID or the first socket after CONNECTED. Instrumentation creates
profiles instead of UI/SAF. The signed R8 pair uses matching-runner rules;
ordinary production-rules APK, arm64/OEM and long physical sessions are not
qualified by this execution.

## Harness fixes

1. Emulator36.6.11 raised requested768MiB to2560MB. On the2GiB VM, owned
   cgroups1400MiB RAM/512MiB swap then1400MiB/1536MiB exhausted capacity before
   APK install and during UI import. Now1536MiB/explicit `-lowram` plus mandatory
   guest MemTotal admission before APK install. Four focused methods PASS.
   [AOSP](https://android.googlesource.com/platform/external/qemu/+/emu-master-dev/android/android-emu/android/main-common.c) explains removal of the RAM lower bound; this branch is not proven
   to be the exact installed build source.
2. The24-sample cold burst finished0.854s before CONNECTED. The helper now waits
   boundedly for CONNECTED before the existing fresh follow-up. All observed
   post-CONNECTED samples in both bursts must pass; missing cold coverage stays
   explicit without continuous-coverage PASS. Two new methods cover late
   publication and failed follow-up refusal.
3. Profiles tap had not finished transitioning when Import was declared absent.
   Up to30s/5 fresh observations require the button; retap only an unselected
   Profiles tab. Three methods reject selected-only false PASS/delayed transitions.
4. A system ANR during Always-on was misclassified as unknown VPN confirmation.
   Shared UI helper preserves original XML, selects only system Wait and requires
   a fresh ordinary screen within3 attempts. Two methods cover preservation and
   refusal on repeated ANR.

Together with prior DNS/startup fixtures:23 helper methods PASS. Memory admission
ran in current Android. The fully repaired cold UI scenario **did not pass**:
the last cold attempt stopped at unpublished `/sdcard`, with remote secure_mkdirs
failed on INI push. Previous ANR/UI/timing failures remain. One failed attempt
received actual VPN replies before aborting, without full acceptance PASS. The
seventh routed gate has its own successful scope and does not close those failures.

Historical system/data descriptions now distinguish requested768MiB from measured
RAM. Retained data/runtime-final, restart-control/r4 and resolver-diagnostic logs
report2560MB. Original packets, dates/results remain unchanged.

## Remainder

Prior SIGKILL automatic recovery also failed in minimal independent TUN control;
this step neither repeats nor closes it. Generic auto/null DnsResolver ENONET
remains: ordinary system lookup in the routed gate is a different API operation.
Current cold UI startup/lockdown/recovery remains FAIL/uncovered. Do not repeat
identical cold attempts without external-storage admission and adequate system
stability. Lowram is not standard physical-platform qualification. Historical
positive matrices retain original inputs; USER_SKIPPED/D06 unchanged. Q29 and
the whole audit remain incomplete.

All7 attempts preserved host firewall/routes/resolver, production service and
original AVD userdata. Seven owned roots removed after SHA-verified full recovery
archives and process/fd/mount/host snapshot checks. Raw:
`audit-debt-20260924/q29-current-vpn-20261012`, `q29-current-vpn-r2-20261012`…
`q29-current-vpn-r7-20261012`. [Scoped evidence](../../../release/certification/evidence/q29-current-vpn-20261012.json).
`NO_PUSH_DEPLOY_RELEASE`.
