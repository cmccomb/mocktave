[![Github CI](https://github.com/cmccomb/mocktave/actions/workflows/tests.yml/badge.svg)](https://github.com/cmccomb/mocktave/actions)
[![Crates.io](https://img.shields.io/crates/v/mocktave.svg)](https://crates.io/crates/mocktave)
[![docs.rs](https://img.shields.io/docsrs/mocktave/latest?logo=rust)](https://docs.rs/mocktave)

# Access Octave/MATLAB in the Rust Ecosystem
As much as I hate to say it, there is a lot of useful code living in .m files. Sometimes it might be nice to access that code in Rust. 

# Requirements
*__You must have a working installation of [Docker](https://docs.docker.com/get-docker/).__*

# Example Usage
```rust
let script = "              \
    z = 5.24;               \
    m = z*inv(eye(5, 5));   \
    m(1, 2) = 5;            \
    a = 5;                  \
    ";

let y = mocktave::eval(script);

println!("{y:#?}");
```