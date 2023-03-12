[![Github CI](https://github.com/cmccomb/mocktave/actions/workflows/tests.yml/badge.svg)](https://github.com/cmccomb/mocktave/actions)
[![Crates.io](https://img.shields.io/crates/v/mocktave.svg)](https://crates.io/crates/mocktave)
[![docs.rs](https://img.shields.io/docsrs/mocktave/latest?logo=rust)](https://docs.rs/mocktave)

# Access Octave/MATLAB from Rust
As much as I hate to say it, there is a lot of useful code living in .m files. Sometimes it can be nice to access that 
code through Rust. There are at least two use cases I can think of:
1. __Rapid Development__: There might be a simple function in Octave that would require significant development effort to replicate in Rust.
This crate serves as a stopgap measure to enable further development.
2. __Robust Testing__: We all know that the better option is to rewrite those nasty .m files in Rust so they're 🚀Blazingly Fast™️🚀! This create is still useful for testing 
purposes, allowing direction comparison to legacy Octave/MATLAB code. 

# Requirements
This crate uses a disgusting hack: Octave is run in the background in Docker. For that reason, *__you must have a working installation of [Docker](https://docs.docker.com/get-docker/).__*

# Example Usage
Let's say we need a function to compute prime numbers, but we're too lazy to write one ourselves. Let's make a thin 
wrapper around the Octave `primes` function! That function will look like this:
```rust
fn primes(less_than_n: usize) -> Vec<Vec<f64>> {
    mocktave::eval(                // Start an evaluation
            &format!(              // Format the command
                "x = primes({});", // This is where we call `primes` from Octave
                less_than_n        // Pass through the argument
            )
        )
        .get_matrix_named("x")     // Extract the results matrix. 
        .unwrap()                  // Unwrap to get the value     
}

let all_primes_less_than_100 = primes(100);
```
Its important to note that this function is definitely *__NOT__* 🚀Blazingly Fast™️🚀, since it starts, runs, and closes 
a Docker container every time its run.