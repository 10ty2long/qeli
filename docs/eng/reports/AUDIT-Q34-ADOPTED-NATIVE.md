# Q34 — native libraries after adopting the panel fixes

<!-- normative-sync: audit-q34-adopted-native-v1 -->

13 October 2026. The panel CSS build fixes, unsafe legacy entrypoint removal
and Bash marker fix were adopted locally in commit
`da776fb892bd6c02b34194a8e772770744832a77`. Its native source digest is
`27b85c8164e85def83c7c08575e5c3ac1b17d638b52f1cb8c8cf83a1fb5ea17e`.

Before adoption, Rustfmt, strict Clippy, two targeted embedded-assets tests
and the full factory release build passed on .11. The resulting CLI
`15c34473` passed 131 actual HTTP checks covering pages, assets, auth, CSP,
CSRF and INI. Initial incomplete-source-cut, empty-test-filter and CRLF-wrapper
failures are retained; final results bind the corrected inputs and filter.

Fresh independent A/B builds produced Windows x64 and macOS universal
x64/arm64 on .10, and Android ARM64/x86_64 on .11. They used the pinned stock
recipes, Rust 1.97.0, offline/locked/jobs1, separate target directories and
`panic=unwind`. All 327 source inputs were verified. All four library pairs
were byte-identical. Exact exports were checked: 33 C exports for Windows and
each macOS slice; 33 C, 21 TransportCore JNI and one ConfigCore JNI export,
55 total, for each Android ABI. The new DLL passed 22 actual ABI and localhost
checks. Windows network driver execution and physical macOS execution are
outside this result.

All four new libraries differ from their previous SHA values. Old APK and JNI
executions were therefore not restamped. All 679 Android inputs were projected
from the adopted commit; fresh instrumented release app/test APKs and an
ordinary unsigned production APK were built. Both embedded SOs, laboratory
APK signatures, non-debuggable metadata, production rules and two R8 ABI
checks passed. The unchanged 201 JVM tests and lint were not repeated:
applicability was verified using inputs and original SHA-bound XML reports,
retaining the original execution dates.

On .11, the new SO passed 23 JNI checks in a disposable read-only API34
emulator; the installed new release APK passed two INI tests. Original SDK and
userdata bytes were preserved. Logs were archived and independently qualified.
This is not a new VPN data-plane, SIGKILL recovery or generic/null DnsResolver
qualification; the corresponding Q29 limitations remain open.

Both build groups passed owned root/archive cleanup, temporary swap removal
and host restoration checks. The emulator runtime root was also removed;
services, PIDs, binaries, routes and firewall were preserved. Initial desktop
staging failures remain archived: substituting a string inode-cookie into an
`is None` check caused Python to print SyntaxWarning before JSON. External
controllers were corrected; Qeli transport, budgets, recipes and ownership
requirements were unchanged.

The local checkout now contains matching four canonical and four consumed
copies, updated A/B evidence, SHA256SUMS and PROVENANCE. Pair and provenance
checks passed. The [machine-readable result](../../../release/certification/evidence/q34-adopted-native-20261013.json)
records the actual build commit, APK hashes, runtime receipt and retained local
archive bindings. Packets: `audit-debt-20260924/q34-native-*-adopted-*`,
`q34-adopted-android-package-20261013`, `q34-adopted-android-runtime-20261013`.

The native copy/provenance debt caused by patch adoption is closed.
Q34 remains IN_PROGRESS: global release certification applicability to the new
tree, the public source cut and hosted CI are not qualified by this result.
The plan remains 32/37 (86.5%). `NO_PUSH_DEPLOY_RELEASE`.
