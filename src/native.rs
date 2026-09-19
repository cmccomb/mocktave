use std::ffi::{OsStr, OsString};
use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::process::{Command, ExitStatus, Stdio};

use crate::InterpreterResults;

/// Evaluate code using an installed GNU Octave executable, without Docker.
///
/// Each call starts a fresh process in the caller's working directory. Variables
/// do not persist between calls. Startup files and history are disabled.
/// `Default` uses `MOCKTAVE_OCTAVE`, or `octave-cli` on `PATH` if unset.
/// This backend executes with the caller's permissions and is not a sandbox.
#[derive(Clone, Debug)]
pub struct NativeInterpreter {
    executable: OsString,
    runtime_home: Option<PathBuf>,
}

impl Default for NativeInterpreter {
    fn default() -> Self {
        Self::with_executable(
            std::env::var_os("MOCKTAVE_OCTAVE").unwrap_or_else(|| "octave-cli".into()),
        )
    }
}

impl NativeInterpreter {
    /// Select an executable by name or path, overriding `MOCKTAVE_OCTAVE`.
    ///
    /// The value is a single executable, not a shell command or argument list.
    /// It is launched only when an evaluation is requested.
    pub fn with_executable(executable: impl AsRef<OsStr>) -> Self {
        Self {
            executable: executable.as_ref().to_owned(),
            runtime_home: None,
        }
    }

    #[cfg(feature = "bundled")]
    pub(crate) fn with_runtime(executable: PathBuf, home: PathBuf) -> Self {
        Self {
            executable: executable.into_os_string(),
            runtime_home: Some(home),
        }
    }

    /// Evaluate Octave code and return the saved workspace.
    ///
    /// Panics if Octave cannot be run, exits unsuccessfully, or fails to save its
    /// workspace. Use [`Self::try_eval`] to handle these errors without panicking.
    pub fn eval(&self, input: &str) -> InterpreterResults {
        self.try_eval(input)
            .unwrap_or_else(|error| panic!("{error}"))
    }

    /// Evaluate code, reporting launch, execution, and workspace I/O errors.
    ///
    /// A temporary script avoids command-line length limits. Workspace data is
    /// saved separately from stdout and stderr so printed text cannot be mistaken
    /// for variables. Temporary files are removed when this method returns.
    /// The existing result parser and its supported Octave types are reused.
    pub fn try_eval(&self, input: &str) -> Result<InterpreterResults, NativeError> {
        let directory = tempfile::Builder::new()
            .prefix("mocktave-")
            .tempdir()
            .map_err(NativeError::Io)?;
        self.eval_in(input, &directory)
    }

    fn eval_in(
        &self,
        input: &str,
        directory: &tempfile::TempDir,
    ) -> Result<InterpreterResults, NativeError> {
        let workspace = directory.path().join("workspace.txt");
        let script = directory.path().join("evaluation.m");
        let workspace_name = workspace.to_str().ok_or_else(|| {
            NativeError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Octave's temporary workspace path must be valid UTF-8",
            ))
        })?;
        // Single-quoted Octave strings escape apostrophes by doubling them.
        // No shell is involved, and backslashes remain literal (including on Windows).
        let workspace_name = workspace_name.replace('\'', "''");
        fs::write(
            &script,
            format!("{input}\n\nbuiltin('save', '-text', '{workspace_name}', '*');\n"),
        )
        .map_err(NativeError::Io)?;

        let script_name = script.to_str().ok_or_else(|| {
            NativeError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Octave's temporary script path must be valid UTF-8",
            ))
        })?;
        let script_name = script_name.replace('\'', "''");

        let mut command = Command::new(&self.executable);
        if let Some(home) = &self.runtime_home {
            command
                .env("OCTAVE_HOME", home)
                .env("OCTAVE_EXEC_HOME", home)
                .env_remove("OCTAVE_PATH")
                .env_remove("OCTAVE_EXEC_PATH")
                .env_remove("DYLD_LIBRARY_PATH")
                .env_remove("DYLD_FALLBACK_LIBRARY_PATH")
                .env_remove("LD_LIBRARY_PATH")
                .env_remove("LD_PRELOAD");
        }
        let output = command
            .args(["--quiet", "--no-gui", "--no-history", "--norc"])
            // Evaluate the file's contents as code, matching the Docker backend's
            // --eval semantics for function definitions (including nested functions).
            .arg("--eval")
            .arg(format!("eval(fileread('{script_name}'));"))
            .stdin(Stdio::null())
            .output()
            .map_err(|source| NativeError::Launch {
                executable: self.executable.clone(),
                source,
            })?;
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        if !output.status.success() {
            return Err(NativeError::Execution {
                status: output.status,
                stdout,
                stderr,
            });
        }
        let saved = fs::read_to_string(workspace).map_err(|source| NativeError::Workspace {
            source,
            stdout,
            stderr,
        })?;
        // Octave's text format uses platform line endings; the shared parser uses LF.
        Ok(InterpreterResults::from(saved.replace("\r\n", "\n")))
    }
}

/// Errors produced by the native Octave backend.
#[derive(Debug)]
pub enum NativeError {
    /// Could not prepare the temporary script or workspace path.
    Io(io::Error),
    /// Could not start the configured Octave executable.
    Launch {
        /// Executable that was requested.
        executable: OsString,
        /// Underlying process-launch error.
        source: io::Error,
    },
    /// Octave exited unsuccessfully.
    Execution {
        /// Exit status reported by the process.
        status: ExitStatus,
        /// Captured standard output.
        stdout: String,
        /// Captured standard error, including Octave diagnostics.
        stderr: String,
    },
    /// Octave exited successfully but did not leave a readable text workspace.
    Workspace {
        /// Underlying workspace-read error.
        source: io::Error,
        /// Captured standard output.
        stdout: String,
        /// Captured standard error.
        stderr: String,
    },
}

impl fmt::Display for NativeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(source) => write!(f, "Could not prepare an Octave evaluation: {source}"),
            Self::Launch { executable, source } => write!(
                f,
                "Could not start Octave executable {executable:?}: {source}. Install GNU Octave \
                 and put octave-cli on PATH, set MOCKTAVE_OCTAVE, or use \
                 NativeInterpreter::with_executable."
            ),
            Self::Execution {
                status,
                stdout,
                stderr,
            } => {
                write!(f, "Octave failed ({status}).\n{stderr}{stdout}")
            }
            Self::Workspace {
                source,
                stdout,
                stderr,
            } => write!(
                f,
                "Octave did not produce a readable workspace: {source}. \
                 The script may have exited before saving.\n{stderr}{stdout}"
            ),
        }
    }
}

impl std::error::Error for NativeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(source) | Self::Launch { source, .. } | Self::Workspace { source, .. } => {
                Some(source)
            }
            Self::Execution { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn temporary_paths_with_apostrophes_and_spaces() {
        let directory = tempfile::Builder::new()
            .prefix("mocktave path's ")
            .tempdir()
            .unwrap();
        let result = NativeInterpreter::default()
            .eval_in("x = 42;", &directory)
            .unwrap();
        assert_eq!(result.get_scalar("x"), Some(42.0));
    }
}
