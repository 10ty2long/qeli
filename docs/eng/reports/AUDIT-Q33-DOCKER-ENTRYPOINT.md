# Q33: Docker entrypoint configuration and first-run publication

<!-- normative-sync: q33-docker-entrypoint-v1 -->

8 October 2026. F400–F403 / Q33-F003–F006. Q33 remains IN_PROGRESS: this batch covers the container entrypoint only. Full Debian install/upgrade/remove, updater rollback, Docker build/image/recreation/health, remaining file-permission and command paths still require review and integration.

## Reproduced failures and changes

Four old behavior checks fail on both dash and ash (eight expected failures):

- Empty argv reaches `shift` and exits instead of choosing the documented server default. The script now shifts only a supplied role.
- A supplied `--config` is appended after an injected default option. Clap rejects duplicates, and seeding also touches the wrong file. The entrypoint resolves `--config PATH`, `--config=PATH`, `-c PATH` and `-cPATH` before seeding, preserves argv and injects a path only when the CLI supplies none. An explicit CLI path overrides QELI_CONFIG. Missing/empty/duplicate paths refuse before writes. Help/version options go directly to the executable without seeding.
- First-run config and users files inherit umask022 and are0644. New files now use umask077 and mktemp0600; existing files retain their bytes and modes.
- A dangling users.conf symlink passes the old missing-file check and the empty-file redirection creates its target. Config and users seeding now refuse links and nonregular names. They write a private same-directory temporary file and publish with GNU `ln -T`, which refuses replacement and directory destinations. A concurrent regular creator is preserved; a raced link/directory refuses startup. Failed copy/link leaves no partially published final file and cleans the temporary file.

Publication means complete visible bytes, not fsync/power-loss durability. SIGKILL may leave a temporary file. The volume must support hard links for first-run creation; unsupported/read-only filesystems refuse safely. Privileged concurrent changes to parent directories or final reads are outside this cooperative publication check. Existing files are not chmodded or rewritten.

## Executed checks and scope

`python scripts/test_docker_entrypoint.py` requires Linux root/chroot. Optional QELI_ENTRYPOINT_ASH_ROOT supplies extracted ash/musl; QELI_ENTRYPOINT_TEST_SHELL chooses the shell and QELI_ENTRYPOINT_SOURCE selects baseline/current bytes. This is an argv-recording qeli executable seam, not daemon execution.

The exact whole entrypoint ran on .11 inside disposable chroots in private mount/net/PID namespaces:17 methods per dash/ash,34 scenarios and46 invocations PASS with no skips. Cases include CLI forms/precedence, option-only/no-argument calls, preserved state/modes, help without writes, links/FIFO, deterministic competing file/link/directory creation, copy/link failure cleanup and shell metacharacters as path data. Both shells use actual Debian coreutils for publication; ash is retained SHA-admitted OpenWrt userspace, not a BusyBox-utilities compatibility claim. Windows's17 platform skips do not qualify runtime.

The existing Linux CI job now runs `sudo unshare --mount --net --pid --fork python3 -B scripts/test_docker_entrypoint.py`. The same argument sequence executed as root on .11 in an exact two-file fixture:17 dash methods/23 invocations PASS without skips. Fixture backup/input SHA and cleanup guards passed; PID777/845 unchanged. This local check does not claim a new Actions run. Separate raw packet: `audit-debt-20260924/q33-docker-entrypoint-ci-20261008`.

Initial fixture failures (PATH omitted chroot, generator context, repeat mkdir and missing cp) are retained separately and are not product verdicts. After repair, all four baseline defects reproduced on both shells and the final suite passed. Final remote input SHA matches the committed candidate scripts. Backup SHA was verified locally before process/fd/mount/resolved-path guarded removal of the exact owned root. Service PID777/worker845 stayed unchanged; no VPN, host firewall, Docker daemon or production service was started/reconfigured.

Rust/native recipes, ABI and packaged cores are unchanged; no unrelated native rebuild is required. Evidence: `release/certification/evidence/q33-docker-entrypoint-20261008.json`. Raw packet: `audit-debt-20260924/q33-docker-entrypoint-20261008`,25 files/2072139 bytes, manifest SHA256 `8596d49f7129b2b32856cadd0f60ab6fd6c3ea0d288d21b1fdcc80effe283e46`.

Q31 SDK upstream bootstrap is capacity-deferred, not PASS/compile FAIL; release source-cut/current Actions remain OPEN. Q29/Q30/user exclusions and D06 accepted limitation remain. Full plan29/37(78.4%),eight remain. No push/deploy/release.
