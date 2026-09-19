[![Github CI](https://github.com/cmccomb/mocktave/actions/workflows/tests.yml/badge.svg)](https://github.com/cmccomb/mocktave/actions)
[![Crates.io](https://img.shields.io/crates/v/mocktave.svg)](https://crates.io/crates/mocktave)
[![docs.rs](https://img.shields.io/docsrs/mocktave/latest?logo=rust)](https://docs.rs/mocktave)

![](https://raw.githubusercontent.com/cmccomb/mocktave/cb1c11a7bf512f3cc2b392bf53a99c7e70a5bbab/mocktave.png)

# Access Octave/MATLAB from Rust
As much as I hate to say it, there is a lot of useful code living in .m files. Sometimes it can be nice to access that 
code through Rust. There are at least two use cases I can think of:
1. __Rapid Development__: There might be a simple function in Octave that would require significant development effort to replicate in Rust.
This crate serves as a stopgap measure to enable further development.
2. __Robust Testing__: We all know that the better option is to rewrite those nasty .m files in Rust so they're 🚀Blazingly Fast™️🚀! This create is still useful for testing 
purposes, allowing direction comparison to legacy Octave/MATLAB code. 

# Installation

Choose the Octave runtime when adding the dependency. Application code uses the
same API for each backend:

```rust
let result = mocktave::try_eval("x = 6 * 7;")?;
assert_eq!(result.get_scalar("x"), Some(42.0));
# Ok::<(), mocktave::Error>(())
```

Starting with version 0.1.6, select one of:

```sh
# Self-contained: includes Octave and its numerical libraries.
cargo add mocktave --no-default-features --features bundled

# Use an Octave installation already on the machine.
cargo add mocktave --no-default-features --features native

# Keep the existing Docker behavior.
cargo add mocktave
```

For development against this checkout, add `--path /path/to/mocktave`.

## Runtime requirements

- **Docker (default):** requires a working [Docker](https://docs.docker.com/get-docker/)
  installation and runs the `gnuoctave/octave:8.1.0` image.
- **Native:** runs an installed GNU Octave executable directly, without Docker or
  its Rust dependencies. Cargo does not download or install Octave.
- **Bundled:** embeds a verified, prebuilt Octave runtime and
  its numerical libraries in your Rust application. No Octave installation,
  package manager, or numerical development libraries are required on the end
  user's machine. See [bundled runtime qualification](bundles/README.md) for the
  current release status and platform limits.

`try_eval`, `eval`, `wrap`, and `Interpreter::default()` use the backend selected
by these features. No backend selection is required in application code.
`try_eval` returns errors; existing `eval` calls retain their panic-on-error behavior.
There is no silent fallback to a different backend.

## Self-contained applications

The `bundled` feature supports these qualified targets:

| Platform | Rust target | Minimum OS / libc | Octave |
| --- | --- | --- | --- |
| Apple Silicon macOS | `aarch64-apple-darwin` | macOS 14 | 11.3.0 |
| Intel macOS | `x86_64-apple-darwin` | macOS 15 | 11.3.0 |
| ARM64 Linux | `aarch64-unknown-linux-gnu` | glibc 2.35 | 6.4.0 |
| x86-64 Linux | `x86_64-unknown-linux-gnu` | glibc 2.35 | 6.4.0 |

Windows bundles passed execution tests but are not published pending
corresponding-source packaging. Windows, musl Linux, and other unsupported
targets can use an installed Octave with `native`, or the Docker backend.
Unsupported bundled targets fail at build time with an actionable error.

The equivalent dependency declaration is:

```toml
[dependencies]
mocktave = { version = "0.1.6", default-features = false, features = ["bundled"] }
```

`default-features = false` disables the existing Docker default.

Cargo downloads the archive for its target and verifies a pinned SHA-256. The
archive is embedded into the resulting executable. At first use it extracts
once into a user cache, protected by a cross-process lock and atomic installation.
All runtime use is offline, including the first extraction. The application can be moved without carrying Cargo's
build directory or a separate Octave install. It needs a writable, executable
cache directory; `MOCKTAVE_CACHE_DIR` optionally overrides the default location.

This is a larger application: runtimes carry tens of megabytes of compressed
data. They provide the numerical CLI, with private BLAS/LAPACK,
SuiteSparse, FFTW, and compiler runtime libraries. The OS ABI remains a dependency;
GUI tools, plotting executables, package compilation, and third-party Octave
packages are outside the bundled scope. Runtime redistribution must retain
Octave's GPL and each bundled dependency's license/source obligations. Runtime
archives, notices, and corresponding source archives are available in the
[versioned runtime release](https://github.com/cmccomb/mocktave/releases/tag/octave-runtime-0.1.6).

## Without Docker

Install [GNU Octave](https://octave.org/download), for example with
`brew install octave` on macOS or `sudo apt-get install octave` on Debian/Ubuntu.
On Windows, install GNU Octave and set `MOCKTAVE_OCTAVE` to its `octave-cli.exe`
path if it is not on `PATH`.

Select the native feature in your dependency declaration:

```toml
[dependencies]
mocktave = { version = "0.1.6", default-features = false, features = ["native"] }
```

The existing `mocktave::eval`, `mocktave::wrap`, and `mocktave::Interpreter` APIs
then use native Octave. Run the examples and tests without Docker:

```sh
cargo run --no-default-features --features native --example primes1
cargo test --no-default-features --features native
```

The default executable is `octave-cli` on `PATH`. Set `MOCKTAVE_OCTAVE` to an
executable name or full path to override it, or configure an interpreter directly:

```rust,no_run
# #[cfg(feature = "native")]
# {
let interpreter = mocktave::NativeInterpreter::with_executable("/opt/homebrew/bin/octave-cli");
let result = interpreter.try_eval("x = primes(10);")?;
assert_eq!(result.get_matrix("x"), Some(vec![vec![2.0, 3.0, 5.0, 7.0]]));
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

`try_eval` reports missing executables, failed evaluations (with captured stdout,
stderr, and exit status), and missing workspace output. `eval` retains the
existing panic-on-failure API. Each evaluation starts a fresh Octave process;
variables do not persist between calls. The caller's working directory is
preserved, startup files and history are disabled, and temporary script/workspace
files are cleaned up after evaluation. `InterpreterResults::raw` contains the
saved text workspace; ordinary stdout/stderr are not included in successful results.
The native backend runs code with your user permissions and is not a sandbox.

## Applications with multiple backends

Cargo features are additive, including features enabled by other dependencies.
The default priority is Docker, then bundled, then native. Applications that
enable several backends can select one explicitly:

```rust,no_run
# #[cfg(feature = "native")]
# {
use mocktave::{Backend, Interpreter};
let octave = Interpreter::new(Backend::Native)?;
let result = octave.try_eval("x = 6 * 7;")?;
assert_eq!(result.get_scalar("x"), Some(42.0));
# }
# Ok::<(), Box<dyn std::error::Error>>(())
```

Use `Backend::Docker`, `Backend::Native`, or `Backend::Bundled` when the
corresponding feature is enabled. This explicit selector is optional when the
dependency feature already selects the desired default.

# Example Usage
Let's say we need a function to compute prime numbers, but we're too lazy to write one ourselves. Let's make a thin 
wrapper around the Octave `primes` function! That function will look like this:

But hey, let's say we're even lazier! We love shortcuts around here:
```rust
fn primes(less_than_n: usize) -> Vec<Vec<f64>> {
    mocktave::eval(                // Start an evaluation
            &format!(              // Format the command
                "x = primes({});", // This is where we call `primes` from Octave
                less_than_n        // Pass through the argument
            )
        )
        .get_matrix("x")           // Extract the results matrix. 
        .unwrap()                  // Unwrap to get the value     
}

let all_primes_less_than_100 = primes(100);

assert_eq!(all_primes_less_than_100, 
           vec![vec![2.0, 3.0, 5.0, 7.0, 11.0, 13.0, 17.0, 19.0, 23.0, 
                     29.0, 31.0, 37.0, 41.0, 43.0, 47.0, 53.0, 59.0, 
                     61.0, 67.0, 71.0, 73.0, 79.0, 83.0, 89.0, 97.0]]);
```
```rust
let primes = mocktave::wrap("primes".into());
let all_primes_less_than_100: Vec<usize> = primes([100]);

assert_eq!(all_primes_less_than_100, vec![2_usize, 3, 5, 7,
    11, 13, 17, 19, 23, 29, 31, 37, 41, 43, 47, 53, 59, 61, 67,
    71, 73, 79, 83, 89, 97]);
```

Each call to `eval` starts a new Octave process. With Docker it also creates and
removes a container; reuse `Interpreter` to avoid that container startup cost.
Native execution avoids the container cost but still starts Octave for each call.
