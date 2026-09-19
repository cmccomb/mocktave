# Bundled runtime qualification

The intended end-user experience is one Cargo feature. The user should not need
Octave, Docker, Homebrew, BLAS/LAPACK, SuiteSparse, a C++/Fortran compiler, or
runtime network access. Release maintainers do the platform packaging work.

The Rust backend embeds a target-specific archive at compile time. At runtime it
validates the archive's target and executable, rejects archive links/traversal,
and extracts atomically into a hash-addressed user cache with cross-process
locking. The executable uses the private Octave home; no installed Octave is
selected as a fallback.

## Release status

Version 0.1.6 includes qualified macOS ARM64/Intel and Linux ARM64/x86-64 runtimes.
The [main README](../README.md#self-contained-applications) lists their minimum
OS/glibc versions and Octave versions. Each target passed the bundled suite and a
relocated consumer with an empty `PATH` and fresh cache on GitHub-hosted runners.
macOS dynamic-load tracing also verified that numerical libraries were private.

The Windows x86-64 candidate passed execution tests but remains unpublished until
its corresponding sources are assembled. Windows ARM64, Linux musl, and other
targets are not qualified. Never add an unqualified or unpublished target to
`manifest.json`.

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
   sources, and build recipes needed for redistribution. `collect_sources.py`
   retrieves pinned Homebrew sources/patches or the exact Debian source packages
   and includes the build and relocation recipes. Unix qualification uploads a
   corresponding-source archive alongside each runtime. Windows candidates need
   their corresponding-source collection completed separately before release.
4. Publish qualified archives and their corresponding source/notice artifacts
   to an immutable, versioned release. The workflow only uploads CI candidates;
   it does not publish a release or update the crate automatically.
5. Register each artifact using `tools/register_bundle.py --archive RUNTIME
   --url RUNTIME_URL --sources SOURCES --source-url SOURCES_URL`. The tool checks
   the runtime/source target and version and pins both checksums. Review and
   commit the resulting entries in `manifest.json`.
6. Test a consumer build using the published manifest without local override
   variables. Then publish the crate containing that manifest.

Octave versions may differ between platform candidates. Each is pinned in the
manifest and tested against the same API and examples. The current Linux recipe
uses the distribution's Octave on the older Ubuntu 22.04 baseline. Users requiring
one exact Octave version on every platform should wait for matching qualified
artifacts or explicitly use a controlled native/Docker environment.

## Local candidate testing

These overrides let maintainers test a new archive before publication. End users
only enable the Cargo feature; they do not set these variables.

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
