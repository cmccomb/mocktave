#!/usr/bin/env python3
"""Run the bundled Rust example from a new location and an empty executable PATH."""
import argparse
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("executable", type=Path)
args = parser.parse_args()
with tempfile.TemporaryDirectory(prefix="mocktave-standalone-") as temporary:
    root = Path(temporary).resolve()
    executable = root / args.executable.name
    shutil.copy2(args.executable, executable)
    empty_path = root / "empty-path"
    empty_path.mkdir()
    environment = dict(os.environ, PATH=str(empty_path), MOCKTAVE_CACHE_DIR=str(root / "cache"))
    for key in ("MOCKTAVE_BUNDLE_ARCHIVE", "MOCKTAVE_BUNDLE_SHA256", "MOCKTAVE_OCTAVE",
                "OCTAVE_HOME", "OCTAVE_EXEC_HOME", "OCTAVE_PATH", "OCTAVE_EXEC_PATH",
                "LD_LIBRARY_PATH", "LD_PRELOAD", "DYLD_LIBRARY_PATH", "DYLD_FALLBACK_LIBRARY_PATH"):
        environment.pop(key, None)
    result = subprocess.run([str(executable)], cwd=root, env=environment, text=True, capture_output=True)
    if result.returncode:
        raise RuntimeError(result.stderr[-6000:] + result.stdout)
    if "Bundled numerical checks passed. Runtime: " not in result.stdout:
        raise RuntimeError("The executable did not confirm bundled numerical checks: " + result.stderr + result.stdout)
    if sys.platform == "darwin":
        # Successful child diagnostics are captured by the Rust API. Audit the
        # same extracted Octave directly to inspect its numerical libraries.
        runtime = Path(result.stdout.split("Runtime: ", 1)[1].strip())
        assert runtime.is_relative_to(root / "cache")
        trace_environment = dict(environment, OCTAVE_HOME=str(runtime), OCTAVE_EXEC_HOME=str(runtime), DYLD_PRINT_LIBRARIES="1")
        trace = subprocess.run([str(runtime / "bin/octave-cli"), "--quiet", "--norc", "--no-history", "--eval",
                                "assert(sparse([3,1;1,2]) \\ [9;8],[2;3],1e-12); assert(fft([1,0,0,0]),ones(1,4));"],
                               cwd=root, env=trace_environment, text=True, capture_output=True)
        if trace.returncode:
            raise RuntimeError(trace.stderr[-6000:])
        loaded = []
        for line in trace.stderr.splitlines():
            match = re.search(r"dyld\[\d+\]:\s+(?:<[^>]+>\s+)?(/.*)", line)
            if match:
                loaded.append(match[1])
        if not loaded:
            raise RuntimeError("No dyld dependency trace was captured")
        if not any("libopenblas" in path and Path(path).is_relative_to(runtime) for path in loaded):
            raise RuntimeError("No private OpenBLAS library was observed")
        for path in loaded:
            if not path.startswith(("/usr/lib/", "/System/Library/")) and not Path(path).is_relative_to(root):
                raise RuntimeError(f"Standalone binary loaded an external dependency: {path}")
        print(f"Audited {len(loaded)} dynamic loads: only the extracted runtime and Apple OS libraries.")
    print(result.stdout.strip())
    print("Passed with a relocated Rust executable, a fresh runtime cache, and an empty PATH.")
