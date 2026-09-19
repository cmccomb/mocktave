# Bundled runtime qualification

The intended end-user experience is one Cargo feature. The user should not need
Octave, Docker, Homebrew, BLAS/LAPACK, SuiteSparse, a C++/Fortran compiler, or
runtime network access. Release maintainers do the platform packaging work.

The Rust backend embeds a target-specific archive at compile time. At runtime it
validates the archive's target and executable, rejects archive links/traversal,
and extracts atomically into a hash-addressed user cache with cross-process
locking. The executable uses the private Octave home; no installed Octave is
selected as a fallback.

## Current status

- The Apple Silicon prototype includes Octave 11.3.0 and 79 dependent libraries.
  It requires macOS 26 because that is the baseline of the packaging host's
  binaries. This artifact does not establish compatibility with older macOS.
- A relocated Rust executable has passed dense solves, sparse solves, FFTs, and
  prime generation with an empty `PATH` and a fresh cache. Its extracted runtime
  was audited to load only bundled libraries and Apple OS libraries.
- `manifest.json` deliberately has no published targets. It must not advertise
  nonexistent releases or unqualified platforms.
- The manually triggered `bundled.yml` workflow qualifies macOS ARM64/Intel,
  Linux ARM64/x86-64, and Windows x86-64 candidates. Only the local Apple Silicon
  candidate has been run so far. Windows ARM64, Linux musl, and other targets
  are not supported by these initial recipes.

## Maintainer workflow

1. Run `.github/workflows/bundled.yml`. macOS and Linux packagers copy the full
   numerical/loader dependency closure, rewrite runtime library paths, and audit
   the result. Windows uses the signed upstream distribution and audits DLL
   dependencies. Only OS libraries may remain external.
2. Require the bundled tests, existing doctests, and relocated-executable test
   to pass for every candidate. Record each actual OS/glibc minimum from the
   artifact metadata; the runner label alone is not a compatibility guarantee.
   `tools/verify_feature_install.py` also creates a separate Cargo application,
   adds Mocktave with `--no-default-features --features bundled`, and tests the
   ordinary `try_eval` API after relocating the application with an empty `PATH`.
3. Retain the dependency license notices, package provenance, corresponding
   sources, and build recipes needed for redistribution. The prototypes include
   available notices and provenance but are not a completed source distribution.
   Complete this release work before distributing runtime binaries publicly.
4. Publish qualified archives and their corresponding source/notice artifacts
   to an immutable, versioned release. The workflow only uploads CI candidates;
   it does not publish a release or update the crate automatically.
5. Register each artifact using `tools/register_bundle.py`. Review and commit
   the resulting URL, target, version, and SHA-256 entries in `manifest.json`.
6. Test a consumer build using the published manifest without local override
   variables. Then publish the crate containing that manifest.

Octave versions may differ between platform candidates. Each is pinned in the
manifest and tested against the same API and examples. The current Linux recipe
uses the distribution's Octave on the older Ubuntu 22.04 baseline. Users requiring
one exact Octave version on every platform should wait for matching qualified
artifacts or explicitly use a controlled native/Docker environment.

## Local candidate testing

These overrides are for maintainers before an archive is published. They are not
part of the intended end-user installation.

```sh
python3 tools/package_macos.py \
  --prefix "$(brew --prefix octave)" \
  --destination work/runtime \
  --archive work/artifacts/octave-runtime.tar.gz

export MOCKTAVE_BUNDLE_ARCHIVE="$PWD/work/artifacts/octave-runtime.tar.gz"
export MOCKTAVE_BUNDLE_SHA256="<sha256 printed by the packager>"
cargo test --no-default-features --features bundled
cargo build --no-default-features --features bundled --example bundled
python3 tools/verify_standalone.py target/debug/examples/bundled
python3 tools/verify_feature_install.py --offline
```

`DOCS_RS` skips runtime acquisition only for documentation generation. Such a
build contains no runtime and returns a clear error if bundled execution is
attempted. Normal builds cannot use this as a working runtime substitute.
