"""Read-only exact release tag/source/version admission; no publishing or builds."""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys
import tomllib
from docker_publish_ref import classify
from native_repro import DEFAULT_RUST_TOOLCHAIN
from release_ci_gate import check_release_ci

ROOT = Path(__file__).resolve().parent.parent
OID = re.compile(r"[0-9a-f]{40}|[0-9a-f]{64}")


def git(root, *args):
    result = subprocess.run(["git", *args], cwd=root, capture_output=True,
                            text=True, encoding="utf-8", errors="strict", timeout=30)
    if result.returncode:
        raise ValueError(result.stderr.strip() or "Release Git query failed")
    return result.stdout.strip()


def admit_tag(root, tag):
    classify("tag", tag)
    if git(root, "status", "--porcelain", "--untracked-files=all"):
        raise ValueError("Release publication requires a clean committed source")
    flags = git(root, "ls-files", "-v").splitlines()
    if any(line and (line[0].islower() or line[0] == "S") for line in flags):
        raise ValueError("Release source has assume-unchanged/skip-worktree Git flags")
    commit = git(root, "rev-parse", "--verify", "HEAD^{commit}")
    tagged = git(root, "rev-parse", "--verify", "refs/tags/" + tag + "^{commit}")
    if not OID.fullmatch(commit) or tagged != commit:
        raise ValueError("Release tag does not identify the checked-out commit")
    package = tomllib.loads(git(root, "show", commit + ":qeli/Cargo.toml")).get("package", {})
    version = package.get("version")
    if not isinstance(version, str) or tag != "v" + version:
        raise ValueError("Release tag differs from committed Cargo version")
    return {"commit": commit, "version": version, "rust_toolchain": DEFAULT_RUST_TOOLCHAIN}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, default=ROOT)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--require-ci", action="store_true", help="Require successful CI on this exact commit")
    parser.add_argument("--github-output", type=Path)
    args = parser.parse_args(argv)
    try:
        identity = admit_tag(args.repo, args.tag)
        if args.require_ci:
            failures = check_release_ci(args.repo, args.tag, by_commit=True)
            if failures:
                raise ValueError("; ".join(failures))
            if admit_tag(args.repo, args.tag) != identity:
                raise ValueError("Release source changed during CI admission")
        if args.github_output:
            with args.github_output.open("a", encoding="utf-8", newline="\n") as stream:
                for key, value in identity.items():
                    stream.write(f"{key}={value}\n")
        print(json.dumps({"status": "PASS", **identity}, sort_keys=True))
        return 0
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        print(f"Release publication refused: {error}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
