# Q34: release source admission and Docker notices

<!-- normative-sync: q34-publication-gates-v1 -->

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
