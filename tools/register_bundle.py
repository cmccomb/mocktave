#!/usr/bin/env python3
"""Register a qualified, already-published archive in the crate's pinned manifest."""
import argparse
import hashlib
import json
from pathlib import Path
import tarfile
from urllib.parse import urlparse

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--archive", type=Path, required=True)
parser.add_argument("--url", required=True)
parser.add_argument("--sources", type=Path, required=True)
parser.add_argument("--source-url", required=True)
parser.add_argument("--manifest", type=Path, default=Path("bundles/manifest.json"))
args = parser.parse_args()
if any(urlparse(url).scheme != "https" for url in (args.url, args.source_url)):
    parser.error("Use a versioned HTTPS release URL")
metadata = json.loads(args.archive.with_suffix(args.archive.suffix + ".json").read_text())
if hashlib.sha256(args.archive.read_bytes()).hexdigest() != metadata["sha256"]:
    parser.error("Archive SHA-256 does not match its metadata")
with tarfile.open(args.sources) as archive:
    source_metadata = json.load(archive.extractfile("sources.json"))
if any(source_metadata["runtime"][key] != metadata[key] for key in ("target", "octave_version")):
    parser.error("Corresponding-source metadata does not match this runtime")
with args.sources.open("rb") as stream:
    metadata["source_sha256"] = hashlib.file_digest(stream, "sha256").hexdigest()
metadata["source_url"] = args.source_url
registry = json.loads(args.manifest.read_text())
metadata["url"] = args.url
registry["targets"][metadata["target"]] = metadata
args.manifest.write_text(json.dumps(registry, indent=2, sort_keys=True) + "\n")
print(f'Registered {metadata["target"]}: {metadata["sha256"]}')
