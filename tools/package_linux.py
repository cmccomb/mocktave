#!/usr/bin/env python3
"""Maintainer tool: collect an installed Debian/Ubuntu Octave and its shared libraries.

Build on the oldest supported glibc baseline. Numerical and compiler libraries
are copied privately; only glibc/the dynamic loader remain host dependencies.
Requires patchelf, readelf, ldd, and dpkg-query on the packaging machine.
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

OS_LIBRARIES = {"libc.so.6", "libm.so.6", "libdl.so.2", "libpthread.so.0",
                "librt.so.1", "libresolv.so.2", "libutil.so.1", "libanl.so.1"}


def run(*args):
    return subprocess.check_output([str(arg) for arg in args], text=True).strip()


def dependencies(path):
    result = {}
    for line in run("ldd", path).splitlines():
        if "not found" in line:
            raise RuntimeError(f"Unresolved dependency: {path}: {line}")
        match = re.match(r"\s*(\S+) => (/\S+) \(", line)
        if match:
            result[match[1]] = Path(match[2]).resolve()
    return result


def package(prefix, destination, archive):
    prefix = prefix.resolve(strict=True)
    executable = (prefix / "bin/octave-cli").resolve(strict=True)
    version = run(executable, "--quiet", "--norc", "--eval", "disp(version)")
    arch = {"x86_64": "x86_64", "aarch64": "aarch64"}[run("uname", "-m")]
    target = arch + "-unknown-linux-gnu"
    if destination.exists():
        raise RuntimeError(f"Refusing to replace existing directory: {destination}")
    (destination / "bin").mkdir(parents=True)
    (destination / "lib/runtime").mkdir(parents=True)
    shutil.copytree(prefix / "share/octave", destination / "share/octave", symlinks=False)
    mapping = {executable: destination / "bin/octave-cli"}
    for path in (prefix / "lib").glob("**/octave/**/*.oct"):
        mapping[path.resolve()] = destination / path.relative_to(prefix)
    for path in (prefix / "lib").glob("**/octave/**/PKG_ADD"):
        output = destination / path.relative_to(prefix)
        output.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(path, output)
    queue = list(mapping)
    names = {}
    minimum_glibc = (2, 0)
    for origin in queue:
        output = mapping[origin]
        output.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(origin, output)
        output.chmod(0o755)
        for version_number in re.findall(r"GLIBC_(\d+\.\d+)", run("readelf", "--version-info", origin)):
            minimum_glibc = max(minimum_glibc, tuple(map(int, version_number.split("."))))
        for name, source in dependencies(origin).items():
            if name in OS_LIBRARIES:
                continue
            if name in names and names[name] != source:
                raise RuntimeError(f"Conflicting libraries named {name}")
            names[name] = source
            if source not in mapping:
                mapping[source] = destination / "lib/runtime" / name
                queue.append(source)
    for origin, output in mapping.items():
        for name in run("patchelf", "--print-needed", output).splitlines():
            if "/" in name:
                subprocess.run(["patchelf", "--replace-needed", name, Path(name).name, str(output)], check=True)
        relative = os.path.relpath(destination / "lib/runtime", output.parent)
        subprocess.run(["patchelf", "--set-rpath", "$ORIGIN/" + relative, str(output)], check=True)
    for output in mapping.values():
        for name, source in dependencies(output).items():
            if name not in OS_LIBRARIES and not source.is_relative_to(destination):
                raise RuntimeError(f"External library remains: {output}: {source}")
    notices = destination / "licenses"
    notices.mkdir()
    package_names = set()
    for source in mapping:
        ownership = subprocess.run(["dpkg-query", "-S", str(source)], capture_output=True, text=True)
        if ownership.returncode == 0:
            package_names.update(line.split(": /", 1)[0] for line in ownership.stdout.splitlines())
    versions = []
    for name in sorted(package_names):
        versions.append(run("dpkg-query", "-W", "-f=${binary:Package} ${Version} ${source:Package} ${source:Version}\n", name))
        copyright_file = Path("/usr/share/doc") / name.split(":")[0] / "copyright"
        if copyright_file.exists():
            shutil.copy2(copyright_file, notices / (name.replace(":", "-") + "-copyright"))
    (notices / "debian-package-provenance.txt").write_text("\n".join(versions) + "\n")
    metadata = {"schema": 1, "target": target, "octave_version": version,
                "minimum_glibc": ".".join(map(str, minimum_glibc)),
                "executable": "bin/octave-cli", "libraries": len(mapping)-1,
                "external_dependencies": ["Linux kernel and glibc"], "scope": "Numerical CLI"}
    (destination / "mocktave-runtime.json").write_text(json.dumps(metadata, indent=2) + "\n")
    environment = dict(os.environ, OCTAVE_HOME=str(destination), OCTAVE_EXEC_HOME=str(destination), PATH="/usr/bin:/bin")
    for key in ("LD_LIBRARY_PATH", "LD_PRELOAD", "OCTAVE_PATH", "OCTAVE_EXEC_PATH"):
        environment.pop(key, None)
    subprocess.run([str(destination / "bin/octave-cli"), "--quiet", "--norc", "--no-history", "--eval",
                    "assert(primes(10),[2,3,5,7]); assert(sparse([3,1;1,2]) \\ [9;8],[2;3],1e-12);"], env=environment, check=True)
    archive.parent.mkdir(parents=True, exist_ok=True)
    with tarfile.open(archive, "w:gz", dereference=True) as stream:
        for path in sorted(destination.iterdir()):
            stream.add(path, arcname=path.name)
    manifest = dict(metadata, sha256=hashlib.sha256(archive.read_bytes()).hexdigest(), archive=archive.name, url=None)
    archive.with_suffix(archive.suffix + ".json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps(manifest, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--prefix", type=Path, default=Path("/usr"))
    parser.add_argument("--destination", type=Path, required=True)
    parser.add_argument("--archive", type=Path, required=True)
    args = parser.parse_args()
    package(args.prefix, args.destination.resolve(), args.archive.resolve())
