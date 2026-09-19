#!/usr/bin/env python3
"""Maintainer tool: package an installed Octave with its complete dylib closure.

Only Apple's OS libraries remain external. No Homebrew paths are modified.
The result is a relocatable numerical CLI runtime, not a GUI distribution.
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


def run(*args):
    return subprocess.check_output([str(arg) for arg in args], text=True).strip()


def libraries(path):
    return [line.strip().split(" (compatibility", 1)[0]
            for line in run("otool", "-L", path).splitlines()[1:]]


def rpaths(path):
    return re.findall(r"cmd LC_RPATH\n.*?\n\s*path (.*?) \(offset", run("otool", "-l", path))


def is_system(name):
    return name.startswith(("/usr/lib/", "/System/Library/"))


def resolve(name, origin, executable):
    if name.startswith("@loader_path/"):
        return (origin.parent / name[len("@loader_path/"):]).resolve(strict=True)
    if name.startswith("@executable_path/"):
        return (executable.parent / name[len("@executable_path/"):]).resolve(strict=True)
    if name.startswith("@rpath/"):
        for entry in rpaths(origin) + rpaths(executable):
            entry = entry.replace("@loader_path", str(origin.parent))
            entry = entry.replace("@executable_path", str(executable.parent))
            candidate = Path(entry) / name[len("@rpath/"):]
            if candidate.is_file():
                return candidate.resolve()
        raise RuntimeError(f"Cannot resolve {name} from {origin}")
    return Path(name).resolve(strict=True)


def package(prefix, destination, archive):
    prefix = prefix.resolve(strict=True)
    executable = (prefix / "bin/octave-cli").resolve(strict=True)
    version = run(executable, "--quiet", "--norc", "--eval", "disp(version)")
    machine = run("uname", "-m")
    target = {"arm64": "aarch64-apple-darwin", "x86_64": "x86_64-apple-darwin"}[machine]
    if destination.exists():
        raise RuntimeError(f"Refusing to replace existing directory: {destination}")
    destination.mkdir(parents=True)
    (destination / "bin").mkdir()
    (destination / "lib").mkdir()
    shutil.copytree(prefix / "share/octave", destination / "share/octave")
    # Headers, documentation, GUI launchers, and compiler tools are not runtime requirements.
    for folder in (destination / "share/octave").glob("*/doc"):
        shutil.rmtree(folder)

    mapping = {executable: destination / "bin/octave-cli"}
    for path in (prefix / "lib/octave").rglob("*"):
        if path.suffix == ".oct" or path.name == "PKG_ADD":
            output = destination / path.relative_to(prefix)
            output.parent.mkdir(parents=True, exist_ok=True)
            if path.suffix == ".oct":
                mapping[path.resolve()] = output
            else:
                shutil.copy2(path, output)

    queue = list(mapping)
    dependency_map = {}
    prefixes = {prefix}
    minimum_macos = (0, 0)
    for origin in queue:
        output = mapping[origin]
        output.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(origin, output)
        output.chmod(0o755)
        load_commands = run("otool", "-l", origin)
        for value in re.findall(r"\bminos (\d+(?:\.\d+)+)", load_commands):
            minimum_macos = max(minimum_macos, tuple(map(int, value.split("."))))
        dependencies = {}
        for name in libraries(origin):
            if is_system(name):
                continue
            source = resolve(name, origin, executable)
            if source == origin:  # dylib install ID
                continue
            if source not in mapping:
                output_dependency = destination / "lib" / source.name
                if output_dependency in mapping.values():
                    raise RuntimeError(f"Two different libraries have the same filename: {source}")
                mapping[source] = output_dependency
                queue.append(source)
                if "Cellar" in source.parts:
                    index = source.parts.index("Cellar")
                    prefixes.add(Path(*source.parts[:index + 3]))
            dependencies[name] = source
        dependency_map[origin] = dependencies

    for origin, output in mapping.items():
        args = ["install_name_tool"]
        if output.suffix == ".dylib":
            args += ["-id", "@loader_path/" + output.name]
        for name, source in dependency_map[origin].items():
            relative = os.path.relpath(mapping[source], output.parent)
            args += ["-change", name, "@loader_path/" + relative]
        for entry in set(rpaths(origin)):
            args += ["-delete_rpath", entry]
        if len(args) > 1:
            subprocess.run(args + [str(output)], check=True, capture_output=True)
        subprocess.run(["codesign", "--force", "--sign", "-", str(output)],
                       check=True, capture_output=True)

    for output in mapping.values():
        for name in libraries(output):
            if is_system(name):
                continue
            if not name.startswith("@loader_path/"):
                raise RuntimeError(f"External library remains: {output}: {name}")
            resolve(name, output, destination / "bin/octave-cli")
        if rpaths(output):
            raise RuntimeError(f"Unexpected runtime search path: {output}")

    notices = destination / "licenses"
    notices.mkdir()
    for source_prefix in sorted(prefixes):
        directory = notices / (source_prefix.parent.name + "-" + source_prefix.name)
        directory.mkdir()
        for path in source_prefix.iterdir():
            if path.is_file() and re.match(r"(COPYING|LICENSE|COPYRIGHT|AUTHORS|NOTICE|sbom|INSTALL_RECEIPT)", path.name, re.I):
                shutil.copy2(path, directory / path.name)
        if (source_prefix / ".brew").is_dir():
            shutil.copytree(source_prefix / ".brew", directory / "homebrew-recipes")

    metadata = {
        "schema": 1, "octave_version": version, "target": target,
        "minimum_macos": ".".join(map(str, minimum_macos)),
        "executable": "bin/octave-cli",
        "libraries": len(mapping) - 1,
        "external_dependencies": ["Apple OS libraries and frameworks"],
        "scope": "Numerical CLI; GUI, plotting tools, package compilation and external helper programs are not bundled.",
    }
    (destination / "mocktave-runtime.json").write_text(json.dumps(metadata, indent=2) + "\n")
    environment = dict(os.environ, PATH="/usr/bin:/bin", OCTAVE_HOME=str(destination),
                       OCTAVE_EXEC_HOME=str(destination), DYLD_PRINT_LIBRARIES="1")
    for name in ("DYLD_LIBRARY_PATH", "DYLD_FALLBACK_LIBRARY_PATH", "OCTAVE_PATH", "OCTAVE_EXEC_PATH"):
        environment.pop(name, None)
    result = subprocess.run([str(destination / "bin/octave-cli"), "--quiet", "--norc", "--no-history", "--eval",
                             "assert(primes(10), [2,3,5,7]); assert(sparse([3,1;1,2]) \\ [9;8], [2;3], 1e-12); assert(fft([1,0,0,0]), ones(1,4));"],
                            env=environment, text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(result.stderr)
    # Inspect actual dyld loads, including Octave modules loaded at runtime.
    for line in result.stderr.splitlines():
        match = re.search(r"dyld\[\d+\]:\s+(?:<[^>]+>\s+)?(/.*)", line)
        if match and not is_system(match[1]) and not Path(match[1]).is_relative_to(destination):
            raise RuntimeError(f"Runtime loaded a library outside the bundle: {line}")
    archive.parent.mkdir(parents=True, exist_ok=True)
    with tarfile.open(archive, "w:gz") as stream:
        for path in sorted(destination.iterdir()):
            stream.add(path, arcname=path.name)
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    manifest = dict(metadata, sha256=digest, archive=archive.name, url=None)
    archive.with_suffix(archive.suffix + ".json").write_text(json.dumps(manifest, indent=2) + "\n")
    print(json.dumps(manifest, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--prefix", required=True, type=Path)
    parser.add_argument("--destination", required=True, type=Path)
    parser.add_argument("--archive", required=True, type=Path)
    args = parser.parse_args()
    package(args.prefix, args.destination.resolve(), args.archive.resolve())
