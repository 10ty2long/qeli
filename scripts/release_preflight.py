#!/usr/bin/env python3
"""Release preflight — run this BEFORE cutting a release, from the release branch.

Gates that a 64-bit-only lab build misses (this is exactly how the 0.7.12
mipsel/armv7 router regression reached asset-packaging: CI was red on dev for four
commits, but the local jemalloc gate stayed green and nobody looked at CI):

  1. CI green  — the latest CI run must cover this exact local commit and be
                 `success`; missing Git/CI identity or failed queries refuse release.
                 Since the
                 keenetic-cross matrix builds every shipped router arch
                 (aarch64 + armv7 + mipsel), a green CI already guarantees the
                 32-bit clients compile. Needs `gh` authenticated.
  2. 32-bit    — optional belt-and-suspenders: cross-build the mipsel + armv7
                 router client on the lab, independent of CI. Runs only when
                 QELI_LAB_PASS is set; host from QELI_LAB_SERVER (default .10).
                 Builds a private immutable Git snapshot with the common pinned
                 router recipe, then verifies artifact SHA256 and static target ELF.
                 Dirty source/recipe inputs fail before SSH; no artifacts are published.
  3. OpenWrt   — the feed Makefile's PKG_SOURCE_VERSION must carry the qeli/ tree
                 being released and PKG_MIRROR_HASH must be a real sha256, not the
                 unmatchable placeholder (which makes the router package unbuildable).
                 Local-only; no network or lab needed.
  4. Evidence  — every mandatory automated IPv6/roaming case must be recorded
                 against this source tree and an exact artifact SHA-256. Physical
                 checks are advisory when unavailable; a recorded failure blocks.

Exit non-zero if any gate fails, so it can front a release script.

  python scripts/release_preflight.py [branch]      # branch default: current

Environment:
  QELI_LAB_PASS           enables gate 2 (SSH password for root on the lab host)
  QELI_LAB_SERVER         lab host (default 10.66.116.10)
  QELI_LAB_SRC            obsolete; private committed-source checkout is always used
  QELI_LAB_TRUST_NEW_HOST=1
                          accept a host key that is not in known_hosts (off by default)
"""
import os
import subprocess
import sys

sys.stdout.reconfigure(encoding="utf-8", errors="replace")

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

BRANCH = sys.argv[1] if len(sys.argv) > 1 else subprocess.run(
    ["git", "rev-parse", "--abbrev-ref", "HEAD"], cwd=ROOT, capture_output=True, text=True, timeout=30
).stdout.strip()

from native_lab import connect_lab
from router_release_gate import verify_router_release
from release_ci_gate import check_release_ci


failures = []

# ── Gate 1: CI green on the branch ───────────────────────────────────────────
print(f"=== Gate 1: latest CI run on '{BRANCH}' ===")
failures.extend(check_release_ci(ROOT, BRANCH))

# ── Gate 2: 32-bit router cross-build on the lab (optional) ──────────────────
lab_pass = os.environ.get("QELI_LAB_PASS", "")
if not lab_pass:
    print("\n=== Gate 2: 32-bit lab build — SKIPPED (QELI_LAB_PASS unset) ===")
else:
    print("\n=== Gate 2: 32-bit router cross-build on the lab ===")
    connection = None
    try:
        # Reject dirty source/recipes before connecting or preparing lab tools.
        from router_release_gate import committed_crate_archive
        snapshot = committed_crate_archive(ROOT)
        host = os.environ.get("QELI_LAB_SERVER", "10.66.116.10")
        connection = connect_lab(host, "root", lab_pass)
        if os.environ.get("QELI_LAB_SRC"):
            print("  QELI_LAB_SRC is obsolete: using a private committed-source checkout")
        report = verify_router_release(connection, ROOT, snapshot=snapshot)
        print(f"  source commit: {report['source_commit']}")
        print(f"  private checkout: {report['remote_root']}")
        print(f"  archive SHA256: {report['archive_sha256']}")
        for result in report["results"]:
            print(f"  {result['arch']}: {result['status']}")
            if result["status"] != "PASS":
                failures.append(f"32-bit {result['arch']} build/admission failed")
                print("  ! " + result["error"])
            else:
                print("  artifact SHA256: " + result["artifact_sha256"])
    except Exception as error:
        failures.append(f"32-bit router gate failed: {error}")
        print(f"  ! {error}")
    finally:
        if connection is not None:
            try:
                connection.close()
            except Exception as error:
                failures.append(f"lab close failed: {error}")
                print(f"  ! {error}")

# ── Gate 3: OpenWrt package pin ──────────────────────────────────────────────
# The feed Makefile pins the source by SHA and verifies the generated tarball by
# PKG_MIRROR_HASH. Both are release-time facts: the SHA is the commit being tagged, and
# the hash only exists once that tarball has been produced. Left stale, the package
# either builds a DIFFERENT tree than the release or refuses to build at all — and the
# all-zero placeholder is deliberately unmatchable, so nothing catches it until someone
# tries to build for a router. Checked here because this is the one script that runs
# with the release commit in hand. (Audit 2026-07-30, #2.)
print("\n=== Gate 3: OpenWrt package pin ===")
OWRT_MK = os.path.join(ROOT, "qeli-openwrt", "Makefile")
PLACEHOLDER_HASH = "0" * 64
try:
    with open(OWRT_MK, encoding="utf-8") as fh:
        mk = fh.read()

    def mk_var(name):
        for line in mk.splitlines():
            if line.startswith(f"{name}:="):
                return line.split(":=", 1)[1].strip()
        return None

    pkg_hash = mk_var("PKG_MIRROR_HASH")
    pkg_sha = mk_var("PKG_SOURCE_VERSION")
    pkg_ver = mk_var("PKG_VERSION")
    # A git failure must FAIL the check, not skip it.
    #
    # The exit code was ignored and only `stdout` read, so anything that stopped git from
    # answering — a dubious-ownership refusal, a detached worktree, git missing from PATH —
    # left `head` empty, and the `if head and ...` below then quietly skipped the comparison.
    # The one gate that catches a stale PKG_SOURCE_VERSION was therefore disabled by an
    # unrelated environment problem, and disabled SILENTLY: preflight printed the Makefile's
    # SHA and reported no mismatch, which reads as "the pin is correct".
    # (Audit 2026-08-02, §8 of the follow-up.)
    head_proc = subprocess.run(
        ["git", "rev-parse", "HEAD"], capture_output=True, text=True, cwd=ROOT
    )
    head = head_proc.stdout.strip()
    if head_proc.returncode != 0 or not head:
        detail = head_proc.stderr.strip() or f"exit {head_proc.returncode}"
        failures.append(
            "release_preflight: cannot read HEAD via git "
            f"({detail}) — refusing to skip the OpenWrt PKG_SOURCE_VERSION comparison, "
            "which is the only check that catches the router package building an older tree"
        )

    print(f"  PKG_VERSION={pkg_ver}  PKG_SOURCE_VERSION={(pkg_sha or '')[:8]}  "
          f"PKG_MIRROR_HASH={'<placeholder>' if pkg_hash == PLACEHOLDER_HASH else (pkg_hash or '')[:16]}")

    if pkg_hash is None or pkg_sha is None or pkg_ver is None:
        failures.append("qeli-openwrt/Makefile: could not read PKG_VERSION/PKG_SOURCE_VERSION/PKG_MIRROR_HASH")
    else:
        if pkg_hash == PLACEHOLDER_HASH:
            failures.append(
                "qeli-openwrt/Makefile: PKG_MIRROR_HASH is still the placeholder — the router "
                "package cannot build. Produce it from an OpenWrt buildroot with "
                "`make package/qeli/download V=s` and hash the Source archive from package metadata "
                "(or `make package/qeli/check FIXUP=1`). Never set it to `skip`."
            )
        elif pkg_hash.lower() == "skip":
            failures.append("qeli-openwrt/Makefile: PKG_MIRROR_HASH=skip disables tarball verification")
        elif len(pkg_hash) != 64 or not all(ch in "0123456789abcdef" for ch in pkg_hash.lower()):
            failures.append(f"qeli-openwrt/Makefile: PKG_MIRROR_HASH is not a sha256 ({pkg_hash!r})")

        if head and pkg_sha != head:
            # The pin can never equal the commit that sets it: writing a SHA into this file
            # produces a new commit with a different SHA. Demanding equality made this gate
            # unsatisfiable, and an unsatisfiable gate gets bypassed — which is worse than a
            # gate that asks the decidable question.
            #
            # What the check is FOR is that the router package compiles the sources being
            # released. The package builds only qeli/ (Build/Compile cd's into
            # $(PKG_BUILD_DIR)/qeli; the init/config files come from the feed clone, not the
            # tarball), so the question is whether the pinned commit carries the same qeli/
            # tree as the release. A pin left over from an earlier version still fails — that
            # tree differs — which is the case this gate exists to catch.
            def tree_of(rev):
                # `<commit>:<path>` already resolves to the subtree's OID; appending
                # ^{tree} makes git look for a PATH literally named "qeli^{tree}".
                p = subprocess.run(["git", "rev-parse", rev],
                                   capture_output=True, text=True, cwd=ROOT)
                return p.stdout.strip() if p.returncode == 0 else ""

            pin_tree, head_tree = tree_of(f"{pkg_sha}:qeli"), tree_of("HEAD:qeli")
            if not pin_tree:
                failures.append(
                    f"qeli-openwrt/Makefile: PKG_SOURCE_VERSION pins {pkg_sha[:8]}, which this "
                    f"repository does not contain — the package would build an unknown tree"
                )
            elif not head_tree:
                failures.append("cannot read HEAD:qeli — refusing to skip the OpenWrt pin comparison")
            elif pin_tree != head_tree:
                failures.append(
                    f"qeli-openwrt/Makefile: PKG_SOURCE_VERSION pins {pkg_sha[:8]}, whose qeli/ "
                    f"tree ({pin_tree[:8]}) is not the one being released ({head_tree[:8]}) — "
                    f"the router package would build different sources"
                )
            else:
                print(f"  pin {pkg_sha[:8]} != HEAD {head[:8]}, but its qeli/ tree matches "
                      f"({pin_tree[:8]}) — the package builds the released sources")

        # The version in the feed Makefile has to name the release it ships.
        crate_ver = None
        try:
            with open(os.path.join(ROOT, "qeli", "Cargo.toml"), encoding="utf-8") as fh:
                for line in fh:
                    if line.startswith("version"):
                        crate_ver = line.split("=", 1)[1].strip().strip('"')
                        break
        except OSError:
            pass
        if crate_ver and pkg_ver != crate_ver:
            failures.append(
                f"qeli-openwrt/Makefile: PKG_VERSION={pkg_ver} but the crate is {crate_ver}"
            )
except OSError as e:
    failures.append(f"qeli-openwrt/Makefile unreadable: {e}")
    print(f"  ! {e}")

# ── Gate 4: release certification evidence ──────────────────────────────────
print("\n=== Gate 4: IPv6/roaming release certification ===")
certification = subprocess.run(
    [sys.executable, os.path.join(ROOT, "scripts", "release_certification.py"), "--quiet"],
    cwd=ROOT,
    capture_output=True,
    text=True,
    encoding="utf-8",
    errors="replace",
)
if certification.stdout.strip():
    print(certification.stdout.rstrip())
if certification.stderr.strip():
    print(certification.stderr.rstrip())
if certification.returncode != 0:
    failures.append(
        "IPv6/roaming certification is incomplete or stale; see Gate 4 details above"
    )

# ── verdict ──────────────────────────────────────────────────────────────────
print("\n===== PREFLIGHT =====")
if failures:
    print("FAIL:")
    for f in failures:
        print(f"  - {f}")
    sys.exit(1)
print("PASS — safe to release")
