// Run with: cargo run --no-default-features --features bundled --example bundled
fn main() -> Result<(), mocktave::Error> {
    // Select the backend in Cargo.toml; application code uses the ordinary API.
    let results = mocktave::try_eval(
        "dense = [3, 1; 1, 2] \\ [9; 8]; sparse_result = sparse([3, 1; 1, 2]) \\ [9; 8]; \
         spectrum = fft([1, 0, 0, 0]); p = primes(10); runtime = OCTAVE_HOME;",
    )?;
    for name in ["dense", "sparse_result"] {
        let value = results.get_matrix(name).unwrap();
        assert!((value[0][0] - 2.0).abs() < 1e-12);
        assert!((value[1][0] - 3.0).abs() < 1e-12);
    }
    assert_eq!(
        results.get_matrix("p"),
        Some(vec![vec![2.0, 3.0, 5.0, 7.0]])
    );
    assert_eq!(results.get_matrix("spectrum"), Some(vec![vec![1.0; 4]]));
    println!(
        "Bundled numerical checks passed. Runtime: {}",
        results.get_string("runtime").unwrap()
    );
    Ok(())
}
