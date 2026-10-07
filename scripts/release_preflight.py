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
                 being released; recipe variables must be unambiguous literals. The SDK-generated
                 Source archive SHA256 must match PKG_MIRROR_HASH; missing archive,
                 mutable ref, skip/placeholder and dirty inputs refuse release.
                 Local-only; no network or lab needed.
  4. Evidence  — every mandatory automated IPv6/roaming case must be recorded
                 against this source tree and an exact artifact SHA-256. Physical
                 checks are advisory when unavailable; a recorded failure blocks.

Exit non-zero if any gate fails, so it can front a release script.

  python scripts/release_preflight.py [branch]      # branch default: current

Environment:
  QELI_LAB_PASS           enables gate 2 (SSH password for root on the lab host)
  QELI_LAB_SERVER         lab host (default 10.66.116.10)
  QELI_OPENWRT_SOURCE_ARCHIVE
                          local SDK-generated Source archive for mandatory gate 3
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
from openwrt_source_gate import check_openwrt_source


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

# ── Gate 3: immutable OpenWrt source and actual SDK mirror archive ────────────
print("\n=== Gate 3: OpenWrt package source ===")
failures.extend(check_openwrt_source(ROOT, os.environ.get("QELI_OPENWRT_SOURCE_ARCHIVE")))

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
