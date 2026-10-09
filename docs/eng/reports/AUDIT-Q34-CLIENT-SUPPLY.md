# Q34: client build dependencies and Windows signatures

<!-- normative-sync: q34-client-supply-v2 -->

8 October 2026. F455–F458 / Q34-F009–F012 are closed in the scope below. F459 / Q34-F013 remains OPEN. Full Q34 remains IN_PROGRESS; full audit 30/37 DONE (81.1%), 7 remain. Local changes only; no push, deployment or release.

## Findings and fixes

| Finding | Result |
|---|---|
| F455 / Q34-F009 | Actual npm audit found selector-parser GHSA-rj75-hqrm-r3gf and source-map-js GHSA-68fv-2mgg-jv7q. Exact overrides 7.1.6 and 1.2.2 replace only those two lock records. Clean npm ci --ignore-scripts and CSS build PASS; embedded CSS byte-identical. |
| F456 / Q34-F010 | Gradle wrapper version was pinned, but its downloaded archive lacked a checksum. Added official Gradle9.7.1 SHA256 acd53f1edaf02f1a8ff99879f8a34b302661a057d9b063ae9e35b552f804d20a. Real wrapper accepts the verified official ZIP and refuses a byte-modified ZIP before running Gradle. Existing wrapper-JAR CI validation retained. |
| F457 / Q34-F011 | NuGet audit warnings did not stop normal restore/build. Repository Directory.Build.props audits all direct/transitive packages from low severity; NU1900–NU1905 are errors. NuGet.Config fixes an explicit official advisory source without overriding package sources. Three actual Release restores/builds PASS; known vulnerable fixture and unavailable advisory source both refuse restore. |
| F458 / Q34-F012 | Windows signature gate covered only Wintun. Extended existing gate to both official WinDivert2.2.2-A copies: pinned DLL/driver/license/per-path notices, driver file version2.2, Valid Authenticode and signer thumbprint043589F75FCE2795E7F2CC3E526D46784D5DDAB3. Positive and four real corrupt/missing-file refusals PASS. |
| F459 / Q34-F013 OPEN | braces GHSA-vfj7-8cjw-p6xm has no published patched version as of this check; npm latest3.0.3 and Tailwind v3-lts3.4.19 still depend on it. Current npm audit returns1: five affected graph entries from this one advisory. No suppression or claim of zero vulnerabilities; npm hard gate remains pending. |

These npm packages build trusted repository CSS/templates; they do not ship as the VPN/panel runtime parser. No client INI format changes. A Tailwind4 migration would change the build/configuration contract and requires separate CSS compatibility qualification. A speculative braces3.0.4 trial was refused by npm and never entered the repository. Open finding must be revisited when an upstream fix or qualified replacement is available.

## Actual checks and boundaries

NuGet fresh package/audit cache, SDK10.0.300: Release Windows, managed macOS cross-build and QeliConformance build/restore PASS. The three Release graphs have no reported known vulnerabilities, including transitive packages. This does not qualify Debug/headless/Linux-specific graphs, SDK freshness, or absence of undisclosed defects. Mandatory shared conformance with the full tracked corpus PASS; Windows adapters selftest PASS. PacketCodec conformance benchmark:1400bytes,20000iterations,260.953ms,102.328MiB/s; this is a local codec gate, not VPN throughput. Initial incomplete scratch copy failed eight required fixture checks; complete corpus rerun retained the required checks.

NuGet real System.Text.Json8.0.0 fixture refuses with NU1903. Deliberately unavailable child-only audit source refuses with NU1900/NU1905. No global NuGet source or OS networking configuration was edited. The initial SDK first-use log reports a development HTTPS certificate setup. Audit settings apply to projects built inside this repository; stripped standalone source copies must retain these root build-policy files. .NET package locks and SDK policy remain separate Q34 work.

Official [WinDivert2.2.2-A release](https://github.com/basil00/WinDivert/releases/tag/v2.2.2) ZIP downloaded independently; both bundled x64 files exactly match upstream SHA256. Official ZIP contains an unsigned user-space DLL and a Valid signed SYS; gate authenticates SYS and pins DLL bytes. Notices intentionally differ by path and are checked individually. No driver installation, WinDivert filter, VPN routing or Windows network VM qualification. Wrapper positive run executes only Gradle --version in private GRADLE_USER_HOME; no APK/runtime claim.

Scratch data/caches and outputs are owned local paths. No SSH/lab changes or secrets used. Rust/native source digest remains e3fb04a80fc8c2e7334fa5187dacb497ca47b2a79954fcddae45b5d5a533052c; committed four-target A/B, copies and 14 SHAs remain valid without rebuilding unchanged Rust inputs. app.css, CHANGELOG and users.html remain byte-identical. Initial fixture/encoding/notice assumptions are recorded as harness failures rather than product regressions.

`q34-client-supply-20261008`: SEALED 88 files / 404870232 bytes; manifest SHA256 `049420ad624cd500958d4b0a9dacefa18e4142f32ece4d814259da8e36e0e66a`.

SDK policy, .NET/Android dependency locking/verification, other dependency graphs and release payload/current exact-SHA Actions admission remain open Q34 coverage. Old14 debtDONE+D06accepted; Q29SIGKILLFAIL/ENONET, Q30Apple/callbackOPEN, Q31capacity/source-cut/currentActions and USER_EXCLUDED Mac/iOS/router/Windows network VM are preserved. No fresh Android runtime/soak or physical Mac claim.

Policy references: [Gradle wrapper checksums](https://docs.gradle.org/current/userguide/gradle_wrapper.html#sec:verification), [NuGet audit](https://learn.microsoft.com/en-us/nuget/concepts/auditing-packages), [selector-parser advisory](https://github.com/postcss/postcss-selector-parser/security/advisories/GHSA-rj75-hqrm-r3gf), [braces advisory](https://github.com/advisories/GHSA-vfj7-8cjw-p6xm).

## 9 October: F491 / Q34-F045 — APK builder must not kill another Gradle build

`scripts/rebuild_apk.py` ran global `pkill -9 -f GradleDaemon` and removed
`/root/.gradle/caches/journal-1` before building, despite its Gradle invocation
already using `--no-daemon`. A process name does not identify build ownership;
this could terminate another task and damage its cache. Both global operations
are removed. Gradle owns its locks. Rust/Gradle commands, offline mode, one worker,
source mirroring, release-signature verification and SHA-qualified APK download
are preserved.

Execution now lives in `build`/`main`: import does not connect to the lab,
invalid arguments refuse before SSH, and `finally` closes SFTP/SSH after open,
build or close failures. The CLI retains default debug and `--release` modes;
compilation/signature failure still exits unsuccessfully. New tests run in CI.

Inside owned .11 PID/mount namespaces, the exact baseline command killed a
controlled foreign-named `GradleDaemon` and its own shell, both returncode -9.
The shared-cache path was covered by a private bind mount; host processes and
the real cache were inaccessible to the probe. Baseline journal deletion is
not claimed because the shell died first. The actual fixed orchestration
invoked a controlled failing `gradlew`, correctly refused compile failure7,
and preserved the foreign worker and byte-identical cache marker. This is
process/failure qualification, not actual Gradle/compiler/APK execution.

Windows:7 new lifecycle/orchestration methods,38 native recipe methods and11
workflow methods PASS. Linux: the same7 new methods and the separate actual
concurrency probe PASS. Official SHA-verified actionlint1.7.12 accepted four
workflows. Inputs/full Linux output were recovered with SHA, and the owned root
was removed using an exact file list after process/fd/mount guards. The .11
service,PID777/845 and installed CLI75bc were preserved; .10 was unused.
Core/native recipe inputs,14 native SHA,protected files and the existing OpenWrt
source cut are unchanged; no new APK was built.

Scope: this removes global process termination and shared-cache damage.
The builder retains its fixed source directories; concurrent APK builds from
different sources using those same directories are not qualified by this packet.
Release `apksigner` verification proves a valid signature, not the expected
publisher; current signed release payload and publisher qualification remain
open requirements. F459 braces remains OPEN without suppression. Q34 IN_PROGRESS;
30/37 DONE81.1%,seven sections remain;Q29 SIGKILL FAIL/ENONET,Q30/Q31,D06
ACCEPTED_LIMITATION and USER_EXCLUDED are preserved. LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.

F491 evidence: `q34-apk-builder-20261009`; SEALED 36 files / 1205550 bytes; manifest SHA256 `bf6a65063b2ee57f87813097cb2601d15d075dffa25ebf0a9007139abcaa8bf9`.
