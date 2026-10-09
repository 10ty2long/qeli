# Q34: release source admission and Docker notices

<!-- normative-sync: q34-publication-gates-v4 -->

Qualification completed9October2026; packet started8October.

## F474 / Q34-F028: actual tag and exact-commit CI

The old shell used short `git rev-list -n1 "$TAG"`, HEAD and version checks. Real
Git reproduces admission of branch `v0.8.2` without a tag. The release workflow also
lacked exact-commit CI admission before build/upload. Checkout now uses `refs/tags/`;
a shared read-only helper requires the actual tag on HEAD, committed Cargo version,
clean source and no assume-unchanged/skip-worktree flags hiding changes. Successful
CI for that exact commit is required before building. Query uses `--commit`; missing,
stale,failed or unavailable CI refuses admission. Source identity is rechecked after
the query; outputs are written only after admission. Tests cover real lightweight/
annotated tags,branch collisions,dirty/untracked/hidden source,HEAD drift,tool timeout
and negative CI. Full multi-platform/SDK/certification preflight remains separate.
Old tags missing the new admission helper/notice policy receive no legacy bypass.

## F475 / Q34-F029: release/Docker compiler pin

Rust action lacked a toolchain value and Docker used moving `rust:1-bookworm`,
diverging from reviewed Rust1.97.0 notices/native policy. Release now takes the
shared DEFAULT_RUST_TOOLCHAIN. Docker1.97.0 builder and Debian runtime bases bind
multiarch digests from official Docker Hub HTTPS tag metadata, listing amd64,
arm64/v8 andarm/v7. Two registry manifest-body attempts failed401 and are retained;
no whole manifest/layer pull qualification is claimed. Docker verifies its pull at
a future build. Live APT repositories are not snapshots; no full image reproducibility.

## F476 / Q34-F030: Docker and extracted DEB common notices

Docker runtime omitted project AGPL/common Cargo/toolchain notices. It now copies
LICENSE/LICENSING and the whole533-file bundle to `/usr/share/doc/qeli`. Context
ignore keeps these paths. Docker publication validates the common bundle before
login/build. Release smoke checks actual extracted DEB resources before checksum/
attestation/upload. This batch qualifies source recipes/common bundle; it does not
claim a fresh whole Docker/native build or container VPN runtime.

## F477 / Q34-F031: CI triggers for every workflow

Both push/PR paths previously covered only `ci.yml`; release-attest/docker-publish
only changes could miss CI. Both now cover `.github/workflows/**`; release tag/CI
contract suites are part of CI. Actual YAML parsing checked all three changed
workflows/fields, without claiming GitHub expression or hosted trigger execution.

## Qualification and limits

- 86methods PASS:17release tag,19release CI,12whole preflight,38existing native recipe contracts. Final17 includes a discovered Git trust-flag bypass/refusal; remaining final inputs did not change after their execution.
- Actual baseline shell/real Git branch-only acceptance reproduced;final CLI branch-only/version negatives and matching tag positive PASS.
- SHA-qualified official PyYAML6.0.3 pure Python BaseLoader parsed three workflows;private scratch only,no system package changes.
- Official Docker Hub metadata/digests/platforms retained;registry401 not relabelled PASS. Read-only hosted CI for the starting local dev commit3cd67555 returned no run;current-source Actions remains OPEN,no dispatch/push.
- Common native/toolchain/managed bundles,protected files,14native artifact hashes/nativee3fb checked. Docs/diff/input scope in raw evidence. Labs untouched;no new core/APK/publish/SDK/router/Mac/device build.

GitHub already provides source archives for release tags; no duplicate source asset
is introduced. Exact source commit is retained in build evidence.
[GitHub source archives](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases).
This does not prove matching third-party binary sources/downstream patches.
Matching source publication,immutable OpenWrt SDK source cut,MicroCom exact binding,
whole Docker pull/build/current-source Actions/signatures/fullrelease remain OPEN.
Q29SIGKILLFAIL-ENONET/Q30/Q31/F459braces/D06accepted/USER_EXCLUDED preserved.
Q34IN_PROGRESS30/37DONE81.1percent7remain.LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.

## Evidence

`q34-publication-gates-20261008`: SEALED 73 files / 1725534 bytes; manifest SHA256 `ce00677e40dae1fa6d7fee5067f2df5a2d5c5c8db5f3154c8c779411070df9cd`.

## 9 October: F490 / Q34-F044 — prevent an old Docker release from replacing latest

A real Git repository with `v0.8.2` and `v0.9.0` reproduces the defect: the
baseline CLI emits `stable_release=true` for `v0.8.2`, and the workflow uses
that flag for `latest`. Re-running an old stable version could roll back the channel.

`docker_publish_ref.py` now emits a separate `latest_release` flag. Stable tags
require a complete Git repository, the candidate tag in the fetched inventory,
and no higher stable MAJOR/MINOR/PATCH version. Comparison is numeric; build
metadata has equal precedence. Branches and RCs return false without querying
Git; unrelated or invalid tags do not define the stable channel. Older stable
releases retain version and SHA publication, without `latest`. Git failures,
timeout, shallow checkout or a missing tag refuse publication before outputs
and before login/build. Existing Cargo version / exact-commit CI admission
remains separate and mandatory for release tags.

The workflow uses a shared concurrency group across refs, with
`cancel-in-progress: false` to preserve an executing run. Serialization alone
would be insufficient because GitHub does not guarantee queue ordering;
numeric version admission is also required. GitHub may replace a pending run
when another arrives; publication of every queued version is not guaranteed.
See [official concurrency rules](https://docs.github.com/en/actions/concepts/workflows-and-actions/concurrency).
Automatic metadata-action `latest` remains disabled; the explicit tag uses
the new output. See [metadata-action](https://github.com/docker/metadata-action).

Sixteen methods passed on Windows and the same sixteen on Linux: old/new tags,
numeric order, RC/unrelated tags, equal build metadata precedence, branches
without Git queries, missing tags/repositories, a real shallow clone, Git
failure/timeout preserving outputs, and the actual CLI. Another 59 related
release admission / CI / preflight / workflow gate methods passed on Windows.
Official SHA-verified actionlint1.7.12 accepted all four workflows. Linux inputs
and the full output were recovered and SHA checked; the owned fixture was
removed after process/fd/mount guards using only an exact file list and
nonrecursive deletion. The .11 service, PID777/845 and installed Qeli SHA
were preserved; .10 was unused.

Scope: this selects the highest fetched stable Git tag, not GHCR registry state.
A newer tag blocks an older version from updating `latest` even before its build
succeeds. Complete current fetching and retained stable tags are prerequisites;
tag deletion/movement, external registry writers and executing old workflow
revisions are not qualified. Old refs use their own workflow and do not acquire
this fix retroactively. No push, hosted Actions, signing or multiarch build ran.
Core/native artifacts and the existing OpenWrt source cut are unchanged;
previous qualification is inherited, not newly executed.

F490 is closed within this scope. Q34 IN_PROGRESS; 30/37 DONE (81.1%), seven
sections remain. Q29 SIGKILL FAIL/ENONET, Q30/Q31, F459 braces OPEN, D06
ACCEPTED_LIMITATION and USER_EXCLUDED are retained. LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.

F490 evidence: `q34-docker-latest-20261009`; SEALED 31 files / 1049840 bytes; manifest SHA256 `61d93c90e2640e781acf6ef239b72ce1a0cce1fd5e3f5a7eafb8ba24816581d2`.


## 9 October: F494 / Q34-F048 — common release source admission

Real Git reproduced a bypass in baseline preflight and certification:
assume-unchanged and skip-worktree hid a tracked worktree edit from status.
Preflight admitted the old HEAD and certification returned the committed-tree
digest although the worktree file differed. Release tag admission already
rejected these flags.

All three entry points now use release_source.py. Existing commit, index trust
flags and clean source are checked before external queries, using NUL-separated
Git paths. Either flag refuses admission even without an edit. Preflight checks
again between phases and after certification. Certification queries ls-tree for
the admitted commit and checks identity again: HEAD drift, a newly hidden flag
or visible source edit refuses the result. Git queries have a 30-second limit;
missing Git, timeout and nonzero exit produce a certification refusal.

New untracked files are allowed only within release/certification/ for the normal
run_ipv6_release_matrix.py --record writer; that subtree is excluded from digest.
Tracked evidence edits require a commit; untracked source outside that subtree
refuses certification. Ignored build caches remain allowed. Preflight and tag
admission require no new untracked evidence: commit reports before these gates.
A custom --evidence outside certification must be committed or kept in ignored
output. Clean committed-source digest serialization is unchanged; hidden
manifest/evidence edits also refuse. Privileged concurrent ignored-cache writers
remain outside the guard.

165 Windows and 33 Linux methods PASS: actual Git fixtures, hidden edits/flags,
HEAD drift, late flags, query failures, new evidence, tracked evidence changes,
ignored caches, orchestration and prior tag/CI/router/OpenWrt/recipe contracts.
Existing CI suites already cover the new tests; workflows unchanged. External
CI/lab builds in orchestration are controlled; no actual current-source CI
publication, SDK/router compilation or runtime VPN qualification is claimed.

The Linux source archive was SHA-checked before execution; 16 known inputs were
removed after SHA/process/fd/mount guards without recursive deletion. The initial
cleanup generator failed with SyntaxError before execution due to a broad FILES
replacement; a helper encoding typo refused before SSH. Both failures retained,
corrected cleanup completed. Original .11 service777/845 and /usr/bin/qeli75bc
retained; .10 unused. Native14/source digest/qeli tree/protected inputs unchanged,
prior execution proofs inherited without Rust/APK/Maven rebuild.41 certification
cases and95 prior supplements preserved modulo source_digest bookkeeping;
supplement96 records this scoped check. Q34 IN_PROGRESS,30/37 DONE81.1%,7remain;
F459/Q29 SIGKILL FAIL-ENONET/Q30/Q31,D06accepted,USER_EXCLUDED and expected release
publisher OPEN retained.LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.

F494 evidence: q34-release-source-trust-20261009; SEALED 51 files / 1287941 bytes; manifest SHA256 2c7e1d8ea5e65a74c96f6db525eed2fe05310a1ba8febdafc41d908a11fd39dc.


## 9 October: F495 / Q34-F049 — certification in actual release workflows

DEB and release-tag Docker publishers called release_tag.py --require-ci without
validating the certification manifest. CI ran validator unit tests, not manifest
admission. The exact baseline publisher command returned PASS/rc0 and wrote
GITHUB_OUTPUT with a real private Git tag v0.8.2 and a committed failed mandatory
linux.outer4.inner4.tcp.full result under controlled successful CI. The validator
rejected the same manifest. This reproduces the bypass without publication.

Both publisher entrypoints now pass --require-certification with --require-ci.
After tag/version/source admission the shared validator checks
release/certification/VERSION.json against current source digest and mandatory
runtime case statuses. Missing, malformed, stale and failed evidence refuse before
CI queries, DEB compilation, Docker login/build/push or writing outputs. Source/tag
identity is checked again after reading evidence; HEAD drift and tracked manifest
changes also refuse. I/O and CertificationError failures return rc1. Admission
requires both certification and successful CI for the exact commit.

Unavailable physical cases remain advisory under existing policy; an explicit
failed physical result blocks release. USER_EXCLUDED devices are not claimed
qualified. Branch/SHA Docker publication retains its separate policy; the new gate
is mandatory for release tags. A normal read-only release_tag.py call without
--require-certification checks metadata, not full release admission. Manual gate:
python scripts/release_tag.py --tag vVERSION --require-ci --require-certification
Replace VERSION with committed Cargo version. Full release_preflight.py still
checks additional SDK/router/source gates and is not replaced by this command.

132 Windows and41 Linux methods PASS. Actual offline CLI, Git tags and committed
JSON manifests cover failed automated/physical results, unavailable advisory,
missing/stale/malformed evidence, I/O refusal, HEAD/manifest drift, retained outputs
on refusal, positive metadata and CI refusal. Successful CI responses are
controlled; no actual GH query/build/upload/OIDC in these tests. Pinned semantic
actionlint accepts all4workflows; existing CI suite includes the new tests.
Both workflows explicitly verify admission precedes compilation/login.

Only private Python/Git fixtures ran on .11. Eight source inputs plus archive
were SHA-checked; nine known files removed nonrecursively after process/fd/mount
guards. Original777/845 service and CLI75bc retained; .10unused. Native14/qeli tree/
protected inputs/OpenWrt source cut unchanged; prior qualification inherited
without Rust/APK/Maven rebuild.41 cases and96 prior supplements preserved modulo
source_digest bookkeeping; append97. This is not newly qualified actual release/
OIDC/signatures/current-source Actions or a rerun of the full runtime matrix.
The [primary advisory](https://github.com/advisories/GHSA-vfj7-8cjw-p6xm) still lists no published F459 patch; no suppression.
Q34 IN_PROGRESS30/37DONE81.1%,7remain;Q29SIGKILLFAIL-ENONET,Q30/Q31,D06accepted,
USER_EXCLUDED and other open items retained.LOCAL_DEV_COMMIT;NO_PUSH_DEPLOY_RELEASE.

F495 evidence: q34-publisher-certification-20261009; SEALED 42 files / 1211464 bytes; manifest SHA256 d11345555e523dc6296de351daf95a0a2d8b942c4d672527a7cbd43fd5c80837.
