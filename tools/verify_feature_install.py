#!/usr/bin/env python3
"""Build a separate consumer using only the bundled Cargo feature, then relocate it."""

import argparse
import os
from pathlib import Path
import subprocess
import sys
import tempfile


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--offline", action="store_true", help="Resolve Cargo dependencies from the local cache")
    args = parser.parse_args()
    repository = Path(__file__).resolve().parents[1]
    target = repository / "target" / "feature-consumer"
    environment = dict(os.environ)
    environment.pop("CARGO_BUILD_TARGET", None)
    with tempfile.TemporaryDirectory(prefix="mocktave feature consumer ") as temporary:
        consumer = Path(temporary)
        subprocess.run(
            ["cargo", "init", "--bin", "--name", "mocktave-feature-consumer", "--vcs", "none", str(consumer)],
            check=True, env=environment,
        )
        command = ["cargo", "add", "mocktave", "--path", str(repository),
                   "--no-default-features", "--features", "bundled"]
        if args.offline:
            command.append("--offline")
        subprocess.run(command, cwd=consumer, check=True, env=environment)
        # The same ordinary API example must work as a downstream dependency.
        (consumer / "src" / "main.rs").write_text((repository / "examples" / "bundled.rs").read_text())
        command = ["cargo", "build", "--target-dir", str(target)]
        if args.offline:
            command.append("--offline")
        subprocess.run(command, cwd=consumer, check=True, env=environment)
        executable = target / "debug" / ("mocktave-feature-consumer.exe" if os.name == "nt" else "mocktave-feature-consumer")
        subprocess.run([sys.executable, str(repository / "tools" / "verify_standalone.py"), str(executable)],
                       check=True, env=environment)
    print("Passed: cargo add --no-default-features --features bundled with unchanged application API.")


if __name__ == "__main__":
    main()
