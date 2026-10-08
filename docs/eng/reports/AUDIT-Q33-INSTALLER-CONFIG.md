# Q33: installer preflight and staged INI publication

<!-- normative-sync: q33-installer-config-v1 -->

8 October 2026. F432–F436 / Q33-F035–F039. Q33 remains IN_PROGRESS.

## Findings and changes

- **F432 / Q33-F035:** explicit PUBLIC_HOST and public-panel options were checked after config/identity/users had changed. They now refuse before dependency/payload changes. Auto-host is resolved/checked before config/identity preparation, although dependencies/binary may already change. The existing character/length policy is an injection guard, not full hostname/IP/CIDR validation.
- **F433 / Q33-F036:** mistyped boolean opt-ins silently selected another policy. QELI_FORCE_RECONFIG, QELI_PANEL_PUBLIC, QELI_VERIFY_ATTESTATION and QELI_ALLOW_UNVERIFIED accept only0/1; unset/empty means0. Unsupported nonempty values refuse before changes.
- **F434 / Q33-F037:** profile generation and CLI/web rewrites changed live server.conf before final validation. All config changes now use a registered private INI stage in the same directory. After final check-config and the requested root service helper succeed, the installer backs up the previous live file and publishes by rename. Generation/identity/user/validation/backup/publication failures keep the old live config; fresh failed validation publishes no partial file. Ordinary error/EXIT cleans temporary files. External hardlink bytes are preserved; published config is owned by qeli with0600, backup is root0600.
- **F435 / Q33-F038:** missing/duplicate selected profile headers were admitted; failure of globals awk could be hidden by the following command. Exactly one selected header is required; both extraction commands explicitly propagate failure. Final Qeli check-config still validates the complete stage.
- **F436 / Q33-F039:** root service-user drop-in was requested before final config validation. It now runs after validation; helper failure prevents config publication/restart. The helper's changes are not transactional. Default QELI_RUN_AS=qeli still does not reset a prior root override; service integration remains OPEN.

## Executed evidence and limits

Command: `sudo unshare --mount --net --pid --fork python3 -B scripts/test_installer_config.py`. Requires Linux root/chroot, Bash/coreutils/dpkg fixture tooling, jq and openssl. Exact whole installer and unchanged production functions run in private chroots/network/PID namespaces.

Baseline17 methods/22 whole scripts produce22 expected assertion failures, no setup failures. Fixed21 methods PASS without skips:28 whole executions,3 positive flows/25 nonzero refusals; exact CI repeats21/28. Cases cover explicit/auto host, panel/boolean options, missing/duplicate profile, both awk calls, sed, REALITY randomness, identity, partially committed third user, validation, backup, rename and service-user failure. Positive flows cover auto-address, forced replacement with external hardlink/backup and public IPv6 endpoint/panel/listener. Baseline return-code successes do not mean passing assertions.

Filesystem bytes/inode/UID/mode, Bash/coreutils and openssl are genuine. Qeli CLI identity/users/web/check-config/service-user, accounts, HTTP, systemctl and kernel/tuning commands are explicit seams. No real INI parser/Argon2/auth/identity/daemon-health/firewall/VPN/Docker qualification. Existing secret fixture now honors --config; one stale recipe assertion on old error wording was removed and behavioral refusal/publication/restart checks are covered here.

Secrets21/admission18/binary18/updater21 regression methods PASS (78 total), ShellCheck0.10.0 warning/bash syntax,38 mocked recipe tests, native14 SHA/source/A-B provenance and docs9 checks/676 Markdown PASS. No native/Rust/ABI/core changes or heavy rebuild. Exact local CI is not a new Actions run.

Only configuration publication is staged. Users/identity/links/tuning/dependencies/binary/drop-in may already change. Recovery journals help recover passwords, not roll back accounts. Restart/health failure after publication does not restore the previous config automatically. No all-or-nothing install, fsync/SIGKILL/power-loss durability or ancestor-directory race protection is claimed. Actual CLI/kernel/service integration, prior root override and Docker remain OPEN.

Evidence: `release/certification/evidence/q33-installer-config-20261008.json`. Raw `audit-debt-20260924/q33-installer-config-20261008` SEALED:65 files/6839726 bytes; manifest SHA256 `840af7bd3d22831ecda9dff23f38192b99a3ba1f7cb978aa660cc70911ed177e`. Own .11 root SHA-backed-up/guarded-cleaned; service777/845 unchanged, .10 untouched.

Full29/37 DONE(78.4%),eight remain. Q31 3/5/capacity-deferred bootstrap/source-cut/current Actions, Q29 SIGKILL/ENONET, Q30 Apple/callback, USER_EXCLUDED platforms and D06 ACCEPTED retained. Local dev commit, no push/deploy/release.
