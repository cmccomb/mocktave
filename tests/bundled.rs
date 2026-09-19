#![cfg(feature = "bundled")]

use mocktave::{Backend, Interpreter};

#[test]
#[cfg(not(feature = "docker"))]
fn cargo_feature_selects_bundled_for_the_ordinary_api() {
    let interpreter = Interpreter::default();
    assert_eq!(interpreter.backend(), Backend::Bundled);
    let private_home = interpreter
        .eval("runtime = OCTAVE_HOME;")
        .get_string("runtime");
    assert_eq!(
        mocktave::try_eval("runtime = OCTAVE_HOME;")
            .unwrap()
            .get_string("runtime"),
        private_home
    );
    assert_eq!(
        mocktave::eval("runtime = OCTAVE_HOME;").get_string("runtime"),
        private_home
    );
    let primes = mocktave::wrap("primes".into());
    let values: Vec<usize> = primes([10]);
    assert_eq!(values, vec![2, 3, 5, 7]);
}

#[test]
fn bundled_backend_has_its_own_runtime_and_numerical_libraries() {
    let interpreter = Interpreter::new(Backend::Bundled).unwrap();
    assert_eq!(interpreter.backend(), Backend::Bundled);
    let results = interpreter
        .try_eval("runtime = OCTAVE_HOME; x = sparse([3,1;1,2]) \\ [9;8]; p = primes(10);")
        .unwrap();
    assert!(!results
        .get_string("runtime")
        .unwrap()
        .contains("/opt/homebrew"));
    let value = results.get_matrix("x").unwrap();
    assert!((value[0][0] - 2.0).abs() < 1e-12);
    assert!((value[1][0] - 3.0).abs() < 1e-12);
    assert_eq!(
        results.get_matrix("p"),
        Some(vec![vec![2.0, 3.0, 5.0, 7.0]])
    );
}

#[test]
fn native_remains_a_separate_explicit_option() {
    let native = Interpreter::new(Backend::Native).unwrap();
    assert_eq!(native.backend(), Backend::Native);
}
