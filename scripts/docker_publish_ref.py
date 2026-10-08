"""Validate Docker release refs before login/build/push; branches publish SHA only."""
import os
from pathlib import Path
import re

SEMVER = re.compile(r"v(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(?:-([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?(?:\+([0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*))?")


def classify(ref_type, ref_name):
    if ref_type == "branch":
        if not ref_name:
            raise ValueError("empty branch ref")
        return {"release_tag": "false", "stable_release": "false"}
    if ref_type != "tag":
        raise ValueError("Docker publication requires a branch or tag")
    match = SEMVER.fullmatch(ref_name)
    if not match or (match[4] and any(part.isdecimal() and len(part) > 1 and part[0] == "0" for part in match[4].split("."))):
        raise ValueError("Docker release tag must be vMAJOR.MINOR.PATCH with valid SemVer suffixes")
    return {"release_tag": "true", "stable_release": str(match[4] is None).lower()}


if __name__ == "__main__":
    outputs = classify(os.environ["GITHUB_REF_TYPE"], os.environ["GITHUB_REF_NAME"])
    with Path(os.environ["GITHUB_OUTPUT"]).open("a", encoding="utf8") as stream:
        for key, value in outputs.items():
            stream.write(f"{key}={value}\n")
