#!/usr/bin/env python3
"""Check the built simulator test host's effective Keychain entitlements.

This is not device/IPA release admission. Real Keychain unit tests still run
after this check; unsigned or wrongly signed hosts must not qualify them.
"""
from __future__ import annotations

import argparse
from pathlib import Path
import plistlib
import subprocess
import sys

from verify_ios_ipa import KEYCHAIN_GROUP_KEY, _signed_entitlements, _string_list


def validate_host(bundle: Path) -> list[str]:
    if sys.platform != "darwin":
        return ["simulator signature verification requires macOS codesign"]
    try:
        if bundle.is_symlink() or not bundle.is_dir():
            return ["simulator host must be a real app directory"]
        info_path = bundle / "Info.plist"
        if info_path.is_symlink() or not info_path.is_file():
            return ["simulator host Info.plist is missing or not a regular file"]
        info = plistlib.loads(info_path.read_bytes())
        if not isinstance(info, dict) or info.get("CFBundlePackageType") != "APPL":
            return ["simulator host is not an application bundle"]
        group = info.get("QeliKeychainAccessGroup")
        if not isinstance(group, str) or not group or "$(" in group:
            return ["simulator host Keychain access group is missing or unresolved"]
        verified = subprocess.run(
            ["codesign", "--verify", "--strict", str(bundle)],
            capture_output=True, timeout=30,
        )
        if verified.returncode:
            return ["simulator host code signature verification failed"]
        entitlements, error = _signed_entitlements(bundle)
        if error:
            return [error]
        if group not in _string_list(entitlements.get(KEYCHAIN_GROUP_KEY)):
            return ["simulator host signed Keychain entitlement differs from QeliKeychainAccessGroup"]
    except (OSError, ValueError, plistlib.InvalidFileException, subprocess.TimeoutExpired) as error:
        return [f"cannot verify simulator host: {error}"]
    return []


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("app", type=Path)
    args = parser.parse_args(argv)
    errors = validate_host(args.app)
    for error in errors:
        print(f"FAIL: {error}", file=sys.stderr)
    if errors:
        return 1
    print("PASS: built simulator host signature and shared-Keychain entitlement verified")
    return 0


if __name__ == "__main__":
    sys.exit(main())
