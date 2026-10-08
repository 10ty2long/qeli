# Q34: exact local OpenWrt source cut

<!-- normative-sync: q34-openwrt-source-cut-v1 -->

Date: 9 October 2026.

## F483 / Q34-F037

The old pin dd0b736276b9ad5b89d117abd1bfe324210f3b26 contained a different qeli
source tree and the production source gate correctly refused it. The recipe now
pins the original full local dev commit f4b80a9a3052e9ec12c640036928d543f931dafa and actual SDK-generated
qeli-0.8.2.tar.zst SHA256 d6c112641de77b446d3a32a702221b247556b873c11f648087aebb57d3b209c5. No synthetic commit was created: the
original commit and every tree object travelled in a Git pack; missing parents
were marked as a shallow boundary. All 3131 tracked prepared files matched
Git, including source, licenses and notices. Native source and14artifactSHA stay unchanged.

Official OpenWrt24.10.4 SDK SHA256
229e871f734a2cee5ce3ad6a3e98d3836b0899bfdeaea4d9c2c5cc7b1fce1407 was verified.
Its metadata/scripts/host tools were byte-preserved; unused target compiler/build
trees were excluded. Private extracted gawk/dependency and pinned Rust feed/config
supported download/prepare only; no host package install, compiler, Cargo, Rust
bootstrap or IPK invocation. [Official checksum](https://downloads.openwrt.org/releases/24.10.4/targets/x86/64/sha256sums).

## Qualification and limits

Two actual SDK downloads, distinct owned Git file URLs and empty DL_DIR caches
produced identical 26381218-byte archives and SHA256. Initial Git download with PKG_SOURCE_MIRROR=0 did not check the old hash;
it did not admit prepare. The measured hash was then verified by production
verify-source.sh during SDK prepare. The second recipe held the correct hash;
archive bytes were independently compared after generation. A hand-produced tar or git-archive
was not substituted for the actual SDK source export.

Public URL stays https://github.com/litvinovtd/qeli.git. Read-only GitHub query for
f4b80a9a3052e9ec12c640036928d543f931dafa returned HTTP422 No commit found; published dev was
83c07f5c31178c67a005f20f59975e14528ab73a. Publish the exact pinned commit before
using the public feed. The source cut is qualified locally; public clone/source
availability and current Actions remain OPEN. Later qeli tree changes require a
new cut/hash. Package version remains0.8.2.

75methodsPASS:25source gate,12whole-preflight,38recipes;three existing
source-suite skips (POSIX symlink/FIFO and opt-in baseline) remain. The actual
committed production gate admitted the real SDK archive and rejected modified
and absent archives. Initial direct-entry fixture lacked sdk/tmp;FAIL66 retained,
only owned tmp directory created. Two SSH launch-channel timeout10s retained as
controller diagnostics; remote jobs continued and completed successfully without
duplicate launch. No compiler/runtime qualification was substituted by fixtures.

Only an owned temporary root on lab .11, jobs1, with no production service,
host package/firewall/sysctl changes. Original service/PID/binary SHA verified
afterward; recoverable archive and guarded cleanup are in raw evidence. Lab .10
was not used. Credential supplied through stdin only; raw and decoded archive credential scan PASS.

Q34 IN_PROGRESS;30/37DONE81.1percent7remain. Q29 SIGKILL FAIL/ENONET, Q30/Q31,
F459 braces OPEN, D06 ACCEPTED_LIMITATION and USER_EXCLUDED preserved. No new
compiler/nativeA-B/platform/runtime/hostedCI/signature/release claim.
LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.

## Evidence

`q34-openwrt-source-cut-20261009`: SEALED 81 files / 261832009 bytes; manifest SHA256 `3e57c16820e13a9177f2e8339779d50d09f41573357ae6eed5e6bfa21b8aaa98`.
