#!/usr/bin/env python3
"""Assemble a signed universal (arm64+x86_64) Qeli.app on the Linux lab (.10),
WITHOUT a Mac.

Inputs (built locally first — Windows/Linux with the .NET 10 SDK):
  qeli-mac/dist/osx-arm64.tar.gz   (dotnet publish -r osx-arm64 --self-contained)
  qeli-mac/dist/osx-x64.tar.gz     (dotnet publish -r osx-x64  --self-contained)

On .10 (has llvm-lipo-19 + rcodesign): merges every per-arch Mach-O into a fat
binary (the already-universal libqeli.dylib is copied as-is), assembles the .app
(Info.plist from Info.plist.in, Qeli.icns), ad-hoc-signs each Mach-O + the bundle
with rcodesign, repacks a Unix-perm zip, and pulls it back to qeli-mac/dist/.
"""
import os, sys
from pathlib import Path
sys.stdout.reconfigure(encoding="utf-8", errors="replace")

from native_lab import connect_lab, pull_verified_artifact, remote_sha256
from native_repro import DEFAULT_RCODESIGN, require_lab_password, sha256_file

ROOT = Path(__file__).resolve().parent.parent
DIST = ROOT / "qeli-mac" / "dist"
INFO_PLIST_IN = ROOT / "qeli-mac" / "Info.plist.in"
ICNS = DIST / "Qeli.icns"
CANONICAL_DYLIB = ROOT / "qeli-mac" / "QeliMac" / "native" / "libqeli.dylib"
HOST = ("10.66.116.10", os.environ.get("QELI_LAB_USER", "root"))
RDIR = "/root/mac-build"
INPUT_CACHE = "/root/qeli-mac-input-cache"
LIPO = "/usr/bin/llvm-lipo-19"
RCS = "/usr/local/bin/rcodesign"


def conn():
    return connect_lab(HOST[0], HOST[1], require_lab_password())


def r(c, cmd, t=600):
    return c.checked(cmd, "macOS universal packaging", timeout=t)


def remote_digest_if_present(c, remote_path):
    output, return_code = c.run(f'test -f "{remote_path}" && sha256sum "{remote_path}"')
    if return_code != 0:
        return None
    digest = output.split()[0].lower() if output.split() else ""
    return digest if len(digest) == 64 else None


# Remote assembly script (runs on .10).
REMOTE_PY = r'''
from pathlib import Path
import shutil
import subprocess

MACHO_MAGICS = {
    b"\xce\xfa\xed\xfe", b"\xfe\xed\xfa\xce", b"\xcf\xfa\xed\xfe", b"\xfe\xed\xfa\xcf",
    b"\xca\xfe\xba\xbe", b"\xbe\xba\xfe\xca", b"\xca\xfe\xba\xbf", b"\xbf\xba\xfe\xca",
}
REQUIRED_ARCHES = {"arm64", "x86_64"}

def macho(path):
    with path.open("rb") as stream:
        return stream.read(4) in MACHO_MAGICS

def architectures(lipo, path):
    result = subprocess.run(
        [lipo, "-archs", str(path)], check=True, capture_output=True,
        text=True, timeout=60,
    )
    names = result.stdout.split()
    if not names or len(names) != len(set(names)):
        raise RuntimeError(f"ambiguous architecture inventory: {path}")
    return set(names)

def inventory(directory):
    if directory.is_symlink() or not directory.is_dir():
        raise RuntimeError(f"expected payload directory: {directory}")
    files = {}
    for path in sorted(directory.rglob("*")):
        if path.is_symlink():
            raise RuntimeError(f"symlinked universal input: {path}")
        if path.is_file():
            files[path.relative_to(directory)] = path
        elif not path.is_dir():
            raise RuntimeError(f"unsupported universal input: {path}")
    return files

def assemble(root="/root/mac-build", lipo="/usr/bin/llvm-lipo-19"):
    root = Path(root)
    arm = inventory(root / "osx-arm64")
    x64 = inventory(root / "osx-x64")
    arm_machos = {rel for rel, path in arm.items() if macho(path)}
    x64_machos = {rel for rel, path in x64.items() if macho(path)}
    for required in ("QeliMac", "libqeli.dylib"):
        if Path(required) not in arm_machos:
            raise RuntimeError(f"missing required Mach-O: {required}")
    if x64_machos - arm_machos:
        raise RuntimeError(f"x64-only Mach-O inputs: {sorted(map(str, x64_machos - arm_machos))}")

    # Admit the complete input graph before replacing the old bundle.
    merge = set()
    for rel in sorted(arm_machos):
        arches = architectures(lipo, arm[rel])
        if arches == REQUIRED_ARCHES:
            continue
        if rel == Path("libqeli.dylib"):
            raise RuntimeError("canonical libqeli.dylib must already contain arm64 and x86_64")
        if arches != {"arm64"} or rel not in x64_machos:
            raise RuntimeError(f"missing universal pair: {rel}")
        if architectures(lipo, x64[rel]) != {"x86_64"}:
            raise RuntimeError(f"wrong x64 partner architecture: {rel}")
        merge.add(rel)

    app = root / "Qeli.app"
    if app.is_symlink():
        raise RuntimeError("refusing symlinked app output")
    if app.exists():
        shutil.rmtree(app)
    macos = app / "Contents/MacOS"
    macos.mkdir(parents=True)
    (app / "Contents/Resources").mkdir()
    for rel, path in arm.items():
        output = macos / rel
        output.parent.mkdir(parents=True, exist_ok=True)
        if rel in merge:
            subprocess.run(
                [lipo, "-create", str(path), str(x64[rel]), "-output", str(output)],
                check=True, capture_output=True, text=True, timeout=60,
            )
        else:
            shutil.copy2(path, output)
        if rel in arm_machos and architectures(lipo, output) != REQUIRED_ARCHES:
            raise RuntimeError(f"assembled Mach-O is not universal2: {rel}")
    print(f"[assemble] lipo={len(merge)} copy={len(arm)-len(merge)}; all Mach-O arm64+x86_64")
    return {"merged": len(merge), "copied": len(arm)-len(merge)}

if __name__ == "__main__":
    assemble()
'''


REMOTE_SIGN_PY = r'''
import os, subprocess, sys
RDIR="/root/mac-build"; APP=RDIR+"/Qeli.app"; MACOS=APP+"/Contents/MacOS"
RCS="/usr/local/bin/rcodesign"
MACHO_MAGICS={
    b"\xce\xfa\xed\xfe", b"\xfe\xed\xfa\xce", b"\xcf\xfa\xed\xfe", b"\xfe\xed\xfa\xcf",
    b"\xca\xfe\xba\xbe", b"\xbe\xba\xfe\xca", b"\xca\xfe\xba\xbf", b"\xbf\xba\xfe\xca",
}
def invoke(args, label):
    result=subprocess.run(args,capture_output=True,text=True)
    output=(result.stdout+result.stderr).strip()
    lowered=output.lower()
    if result.returncode or "error:" in lowered or "failed" in lowered:
        raise RuntimeError(f"{label} failed (rc={result.returncode}):\n{output}")
    return output
machos=[]
for dp,dn,fn in os.walk(MACOS):
    dn.sort()
    for name in sorted(fn):
        path=os.path.join(dp,name)
        with open(path,"rb") as stream: magic=stream.read(4)
        if magic in MACHO_MAGICS: machos.append(path)
for path in machos:
    invoke([RCS,"sign",path],f"sign {path}")
    info=invoke([RCS,"print-signature-info",path],f"inspect {path}")
    if "CodeSignatureFlags(ADHOC" not in info:
        raise RuntimeError(f"{path}: ad-hoc signature flag missing:\n{info}")
invoke([RCS,"sign",APP],"sign app bundle")
print(f"[sign] {len(machos)}/{len(machos)} Mach-O ad-hoc; bundle OK")
'''


def main():
    c = conn()
    rcodesign = r(c, f"{RCS} --version")
    if rcodesign != DEFAULT_RCODESIGN:
        raise RuntimeError(f"rcodesign is {rcodesign}, expected {DEFAULT_RCODESIGN}")
    print("[rcodesign]", rcodesign)
    print("[lipo]", r(c, f"{LIPO} -version 2>/dev/null | head -1 || echo MISSING"))

    r(c, f"mkdir -p {INPUT_CACHE}")
    sf = c.open_sftp()
    for t in ("osx-arm64.tar.gz", "osx-x64.tar.gz"):
        local = DIST / t
        expected = sha256_file(local)
        cached = f"{INPUT_CACHE}/{t}"
        if remote_digest_if_present(c, cached) != expected:
            # Reuse the prior work directory once when introducing the cache. It is subject
            # to the same digest check; an unknown or stale remote file is never trusted.
            previous = f"{RDIR}/{t}"
            if remote_digest_if_present(c, previous) == expected:
                r(c, f'cp "{previous}" "{cached}"')
                print(f"[cache seed] {t}")
            else:
                sf.put(os.fspath(local), cached)
                print(f"[upload] {t}")
        if remote_sha256(c, cached) != expected:
            raise RuntimeError(f"cached macOS input changed after upload: {t}")
        print(f"[input] {t} sha256={expected}")
    r(c, f"rm -rf {RDIR}; mkdir -p {RDIR}; cp {INPUT_CACHE}/*.tar.gz {RDIR}/")
    sf.put(os.fspath(INFO_PLIST_IN), f"{RDIR}/Info.plist.in")
    sf.put(os.fspath(ICNS), f"{RDIR}/Qeli.icns")
    sf.close()
    print("[extract]", r(c, f"cd {RDIR} && tar -xzf osx-arm64.tar.gz && tar -xzf osx-x64.tar.gz && echo OK"))

    # Assemble the universal Contents/MacOS via lipo.
    r(c, f"cat > {RDIR}/assemble.py <<'PYEOF'\n{REMOTE_PY}\nPYEOF")
    print(r(c, f"cd {RDIR} && python3 assemble.py", t=900))
    expected_dylib = sha256_file(CANONICAL_DYLIB)
    assembled_dylib = remote_sha256(c, f"{RDIR}/Qeli.app/Contents/MacOS/libqeli.dylib")
    if assembled_dylib != expected_dylib:
        raise RuntimeError(
            f"assembled app contains dylib {assembled_dylib}, expected {expected_dylib}"
        )
    print(f"[dylib] verified sha256={assembled_dylib}")

    # Info.plist (universal: both arches in LSArchitecturePriority) + icns.
    r(c, f"cd {RDIR} && sed 's|<string>__ARCH__</string>|<string>arm64</string>\\n        <string>x86_64</string>|' "
         f"Info.plist.in > Qeli.app/Contents/Info.plist")
    r(c, f"cp {RDIR}/Qeli.icns {RDIR}/Qeli.app/Contents/Resources/Qeli.icns")
    r(c, f"chmod +x {RDIR}/Qeli.app/Contents/MacOS/QeliMac")

    # One remote transaction signs and inspects every Mach-O, then signs the bundle. This
    # avoids two SSH round trips per binary while preserving the same fail-closed checks.
    r(c, f"cat > {RDIR}/sign_app.py <<'PYSIGN'\n{REMOTE_SIGN_PY}\nPYSIGN")
    print(r(c, f"cd {RDIR} && python3 sign_app.py", t=900))

    # Zip with Unix perms (executable bits survive).
    zip_py = (
        "import os,zipfile\n"
        f"root=r'{RDIR}'; app='Qeli.app'; out=os.path.join(root,'Qeli-macOS-universal.zip')\n"
        "zf=zipfile.ZipFile(out,'w',zipfile.ZIP_DEFLATED,compresslevel=6)\n"
        "for dp,dn,fn in os.walk(os.path.join(root,app)):\n"
        " for n in fn:\n"
        "  full=os.path.join(dp,n); rel=os.path.relpath(full,root)\n"
        "  zi=zipfile.ZipInfo(rel.replace(os.sep,'/')); st=os.stat(full)\n"
        "  zi.external_attr=(st.st_mode & 0xFFFF)<<16; zi.compress_type=zipfile.ZIP_DEFLATED\n"
        "  zf.writestr(zi, open(full,'rb').read())\n"
        "zf.close(); print(os.path.getsize(out))\n"
    )
    r(c, f"cat > {RDIR}/mkzip.py <<'PYZIP'\n{zip_py}PYZIP")
    zsize = r(c, f"cd {RDIR} && python3 mkzip.py")
    print(f"[zip] Qeli-macOS-universal.zip = {zsize} bytes")

    remote_zip = f"{RDIR}/Qeli-macOS-universal.zip"
    zip_digest = remote_sha256(c, remote_zip)
    sf = c.open_sftp()
    try:
        size, digest, _changes = pull_verified_artifact(
            sf,
            remote_zip,
            zip_digest,
            ROOT,
            ("qeli-mac/dist/Qeli-macOS-universal.zip",),
        )
    finally:
        sf.close()
        c.close()
    print(f"[pull] -> {DIST / 'Qeli-macOS-universal.zip'} ({size} bytes, sha256={digest})")
    print("[done]")


if __name__ == "__main__":
    main()
