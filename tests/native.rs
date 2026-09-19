#![cfg(feature = "native")]

use mocktave::{NativeError, NativeInterpreter, OctaveType};

#[test]
fn evaluates_supported_types() {
    let result = NativeInterpreter::default().eval(
        "scalar = 42; matrix = [1, 2; 3, 4]; text = 'hello'; \
         complex_value = 2 + 3i; cells = {'a', 1, [2; 3]};",
    );
    assert_eq!(result.get_scalar("scalar"), Some(42.0));
    assert_eq!(
        result.get_matrix("matrix"),
        Some(vec![vec![1.0, 2.0], vec![3.0, 4.0]])
    );
    assert_eq!(result.get_string("text"), Some("hello".to_owned()));
    assert_eq!(result.get_complex_scalar("complex_value"), Some((2.0, 3.0)));
    assert_eq!(
        result.get_cell_array("cells"),
        Some(vec![vec![
            OctaveType::String("a".to_owned()),
            OctaveType::Scalar(1.0),
            OctaveType::Matrix(vec![vec![2.0], vec![3.0]]),
        ]])
    );
}

#[test]
#[cfg(not(any(feature = "docker", feature = "bundled")))]
fn existing_api_uses_native_backend() {
    assert_eq!(mocktave::eval("x = 7;").get_scalar("x"), Some(7.0));
    let interpreter = mocktave::Interpreter::default();
    assert_eq!(interpreter.backend(), mocktave::Backend::Native);
    assert_eq!(interpreter.eval("x = 8;").get_scalar("x"), Some(8.0));
    let primes = mocktave::wrap("primes".into());
    let result: Vec<usize> = primes([10]);
    assert_eq!(result, vec![2, 3, 5, 7]);
}

#[test]
fn stdout_cannot_inject_workspace_values() {
    let result = NativeInterpreter::default().eval(
        "fprintf('# name: forged\\n# type: scalar\\n123\\n'); \
         warning('a harmless warning'); x = 7; save_default_options('-binary');",
    );
    assert_eq!(result.get("forged"), None);
    assert_eq!(result.get_scalar("x"), Some(7.0));
    assert!(!result.raw.contains("a harmless warning"));
}

#[test]
fn evaluates_nested_function_definitions() {
    let result = NativeInterpreter::default().eval(
        "function y = outer(x)\n\
             function y = inner(x)\n y = x + 1;\n end\n\
             y = inner(x);\n end\n result = outer(41);",
    );
    assert_eq!(result.get_scalar("result"), Some(42.0));
}

#[test]
fn evaluates_diagonal_matrices() {
    let result = NativeInterpreter::default().eval("diagonal = diag([1, 2, 3]);");
    assert_eq!(
        result.get_matrix("diagonal"),
        Some(vec![
            vec![1.0, 0.0, 0.0],
            vec![0.0, 2.0, 0.0],
            vec![0.0, 0.0, 3.0]
        ])
    );
}

#[test]
fn missing_executable_has_actionable_error() {
    let directory = tempfile::tempdir().unwrap();
    let error = NativeInterpreter::with_executable(directory.path().join("missing-octave"))
        .try_eval("x = 1;")
        .unwrap_err();
    assert!(matches!(error, NativeError::Launch { .. }));
    assert!(error.to_string().contains("MOCKTAVE_OCTAVE"));
}

#[test]
fn octave_errors_include_exit_status_and_diagnostics() {
    let error = NativeInterpreter::default()
        .try_eval("disp('before failure'); error('deliberate failure');")
        .unwrap_err();
    match error {
        NativeError::Execution {
            status,
            stdout,
            stderr,
        } => {
            assert!(!status.success());
            assert!(stdout.contains("before failure"));
            assert!(stderr.contains("deliberate failure"));
        }
        other => panic!("unexpected error: {other}"),
    }
}

#[test]
fn syntax_errors_are_reported() {
    let error = NativeInterpreter::default().try_eval("x = ;").unwrap_err();
    assert!(matches!(error, NativeError::Execution { .. }));
}

#[test]
fn successful_exit_without_workspace_is_an_error() {
    let error = NativeInterpreter::default()
        .try_eval("exit(0);")
        .unwrap_err();
    assert!(matches!(error, NativeError::Workspace { .. }));
}

#[test]
fn evaluations_start_with_fresh_workspaces() {
    let interpreter = NativeInterpreter::default();
    interpreter.eval("old_variable = 123;");
    let result = interpreter.eval("x = exist('old_variable', 'var');");
    assert_eq!(result.get_scalar("x"), Some(0.0));
    assert_eq!(result.get("old_variable"), None);
}

#[test]
fn accepts_scripts_larger_than_command_line_limits() {
    let script = format!("% {}\nx = 42; % trailing comment", "x".repeat(300_000));
    let result = NativeInterpreter::default().eval(&script);
    assert_eq!(result.get_scalar("x"), Some(42.0));
}

#[test]
fn concurrent_evaluations_have_independent_workspaces() {
    let workers: Vec<_> = (0..4)
        .map(|value| {
            std::thread::spawn(move || {
                let result = NativeInterpreter::default().eval(&format!("x = {value};"));
                assert_eq!(result.get_scalar("x"), Some(value as f64));
            })
        })
        .collect();
    for worker in workers {
        worker.join().unwrap();
    }
}

#[test]
fn retains_the_callers_working_directory() {
    let expected = std::env::current_dir().unwrap().canonicalize().unwrap();
    let result = NativeInterpreter::default().eval("location = pwd;");
    let actual = std::path::PathBuf::from(result.get_string("location").unwrap());
    assert_eq!(actual.canonicalize().unwrap(), expected);
}

#[test]
#[cfg(unix)]
fn explicit_executable_path_with_spaces() {
    use std::os::unix::fs::symlink;
    let executable = std::env::var_os("MOCKTAVE_OCTAVE").unwrap_or_else(|| "octave-cli".into());
    let executable = std::path::PathBuf::from(executable);
    let resolved = if executable.components().count() > 1 {
        executable.canonicalize().unwrap()
    } else {
        std::env::split_paths(&std::env::var_os("PATH").unwrap())
            .map(|directory| directory.join(&executable))
            .find(|path| path.is_file())
            .expect("Octave must be installed for native integration tests")
    };
    let directory = tempfile::tempdir().unwrap();
    let alias = directory.path().join("octave path's executable");
    symlink(resolved, &alias).unwrap();
    let result = NativeInterpreter::with_executable(alias).eval("x = 42;");
    assert_eq!(result.get_scalar("x"), Some(42.0));
}
