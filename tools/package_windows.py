#!/usr/bin/env python3
"""Package the signed upstream Windows distribution as a numerical CLI runtime.

The caller must verify the upstream ZIP signature before invoking this tool.
Run on Windows, where the result is validated before an artifact is produced.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tarfile


def package(prefix, destination, archive):
    prefix = prefix.resolve(strict=True)
    executable = prefix / "bin/octave-cli.exe"
    if os.name != "nt":
        raise RuntimeError("Package and validate Windows runtimes on Windows")
    if destination.exists():
        raise RuntimeError(f"Refusing to replace existing directory: {destination}")
    (destination / "bin").mkdir(parents=True)
    for path in (prefix / "bin").iterdir():
        if path.suffix.lower() == ".dll" or path.name == "octave-cli.exe":
            shutil.copy2(path, destination / "bin" / path.name)
    shutil.copytree(prefix / "share/octave", destination / "share/octave")
    for path in (prefix / "lib/octave").rglob("*"):
        if path.is_file() and (path.suffix.lower() in (".oct", ".dll") or path.name == "PKG_ADD"):
            output = destination / path.relative_to(prefix)
            output.parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(path, output)
    notices = destination / "licenses"
    notices.mkdir()
    if (prefix / "share/licenses").is_dir():
        shutil.copytree(prefix / "share/licenses", notices / "upstream")
    for path in prefix.parent.glob("COPYING*"):
        shutil.copy2(path, notices / path.name)
    objdump = prefix / "bin/objdump.exe"
    if not objdump.exists():
        raise RuntimeError("The upstream distribution must include objdump for the DLL audit")
    bundled = {path.name.lower() for path in (destination / "bin").iterdir()}
    system = Path(os.environ["SystemRoot"]) / "System32"
    for binary in destination.rglob("*"):
        if binary.suffix.lower() not in (".exe", ".dll", ".oct"):
            continue
        output = subprocess.check_output([str(objdump), "-p", str(binary)], text=True)
        for name in re.findall(r"DLL Name:\s*(\S+)", output):
            if name.lower() in bundled or name.lower().startswith(("api-ms-", "ext-ms-")) or (system / name).is_file():
                continue
            raise RuntimeError(f"External DLL remains: {binary}: {name}")
    environment = dict(os.environ, OCTAVE_HOME=str(destination), OCTAVE_EXEC_HOME=str(destination), PATH=str(system))
    for key in ("OCTAVE_PATH", "OCTAVE_EXEC_PATH"):
        environment.pop(key, None)
    runtime = destination / "bin/octave-cli.exe"
    version = subprocess.check_output([str(runtime), "--quiet", "--norc", "--eval", "disp(version)"], env=environment, text=True).strip()
    subprocess.run([str(runtime), "--quiet", "--norc", "--no-history", "--eval",
                    "assert(primes(10),[2,3,5,7]); assert(sparse([3,1;1,2]) \\ [9;8],[2;3],1e-12);"], env=environment, check=True)
    metadata = {"schema": 1, "target": "x86_64-pc-windows-msvc", "octave_version": version,
                "executable": "bin/octave-cli.exe", "external_dependencies": ["Windows OS DLLs"],
                "scope": "Numerical CLI; no MSYS shell or development tools"}
    (destination / "mocktave-runtime.json").write_text(json.dumps(metadata, indent=2) + "\n")
    archive.parent.mkdir(parents=True, exist_ok=True)
    with tarfile.open(archive, "w:gz") as stream:
        for path in sorted(destination.iterdir()):
            stream.add(path, arcname=path.name)
    manifest = dict(metadata, sha256=hashlib.sha256(archive.read_bytes()).hexdigest(), archive=archive.name, url=None)
    archive.with_suffix(archive.suffix + ".json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps(manifest, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--prefix", type=Path, required=True)
    parser.add_argument("--destination", type=Path, required=True)
    parser.add_argument("--archive", type=Path, required=True)
    args = parser.parse_args()
    package(args.prefix, args.destination.resolve(), args.archive.resolve())
