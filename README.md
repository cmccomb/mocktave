[![Github CI](https://github.com/cmccomb/mocktave/actions/workflows/tests.yml/badge.svg)](https://github.com/cmccomb/mocktave/actions)

[//]: # ([![Crates.io]&#40;https://img.shields.io/crates/v/mocktave.svg&#41;]&#40;https://crates.io/crates/mocktave&#41;)

[//]: # ([![docs.rs]&#40;https://img.shields.io/docsrs/mocktave/latest?logo=rust&#41;]&#40;https://docs.rs/mocktave&#41;)

# Bringing Octave/MATLAB into the Rust ecosystem
As much as I hate to say it, there is a lot of useful code living in .m files. Sometimes it might be nice to access that code in Rust. 

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