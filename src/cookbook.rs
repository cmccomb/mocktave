//! # A Cookbook of Common Tasks
//! ## Wrap an Octave function
//! ```rust
#![doc = include_str ! ("../examples/primes.rs")]
//! ```
//! ## Access linear algebra
//! Linear algebra is one of the areas where rust is (currently) immature. Accessing Octave can enable better testing
//! as the Rust community evolves. Here's an example of an ODE solver:
//! ```rust
#![doc = include_str ! ("../examples/linalg.rs")]
//! ```
//! ## Do topology optimization
//! ### top88.m
//! The famous top88.m code, adapted from [here](https://github.com/blademwang11/Topopt/blob/master/top88.m)
//! ```rust
#![doc = include_str ! ("../examples/top88.rs")]
//! ```
//! ### top99.m
//! The famous top99.m code, adapted from [here](https://www.topopt.mek.dtu.dk/apps-and-software/a-99-line-topology-optimization-code-written-in-matlab)
//! ```rust
#![doc = include_str ! ("../examples/top99.rs")]
//! ```
