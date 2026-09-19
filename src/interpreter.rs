use std::{fmt, io};

use crate::InterpreterResults;

/// Select one of the backends enabled by Cargo features.
///
/// The default is Docker when enabled, then bundled Octave, then native Octave.
/// Select a backend explicitly when your dependency graph enables multiple features.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Backend {
    /// Run Octave in a Docker container (`docker` feature).
    #[cfg(feature = "docker")]
    Docker,
    /// Run an installed Octave executable (`native` feature).
    #[cfg(feature = "native")]
    Native,
    /// Run the private runtime embedded in the application (`bundled` feature).
    #[cfg(feature = "bundled")]
    Bundled,
}

impl Default for Backend {
    fn default() -> Self {
        #[cfg(feature = "docker")]
        {
            Self::Docker
        }
        #[cfg(all(not(feature = "docker"), feature = "bundled"))]
        {
            Self::Bundled
        }
        #[cfg(all(not(feature = "docker"), not(feature = "bundled"), feature = "native"))]
        {
            Self::Native
        }
    }
}

/// An Octave interpreter with an explicit backend and a common result API.
///
/// Each evaluation starts a fresh Octave process. Docker containers can be reused;
/// variables do not persist between evaluations with any backend.
pub struct Interpreter {
    backend: Backend,
    engine: Engine,
}

enum Engine {
    #[cfg(feature = "docker")]
    Docker(crate::docker::Interpreter),
    #[cfg(feature = "native")]
    Native(crate::NativeInterpreter),
}

impl Interpreter {
    /// Initialize a backend, reporting errors instead of panicking.
    ///
    /// Bundled initialization extracts the embedded runtime once into a private
    /// user cache. It requires no network access or installed Octave at runtime.
    pub fn new(backend: Backend) -> Result<Self, Error> {
        let engine = match backend {
            #[cfg(feature = "docker")]
            Backend::Docker => Engine::Docker(crate::docker::Interpreter::try_new()?),
            #[cfg(feature = "native")]
            Backend::Native => Engine::Native(crate::NativeInterpreter::default()),
            #[cfg(feature = "bundled")]
            Backend::Bundled => Engine::Native(crate::bundled::interpreter()?),
        };
        Ok(Self { backend, engine })
    }

    /// Select a particular system Octave executable, without invoking a shell.
    #[cfg(feature = "native")]
    pub fn with_executable(executable: impl AsRef<std::ffi::OsStr>) -> Self {
        crate::NativeInterpreter::with_executable(executable).into()
    }

    /// Return the selected backend.
    pub fn backend(&self) -> Backend {
        self.backend
    }

    /// Evaluate code and return execution errors with their diagnostics.
    pub fn try_eval(&self, input: &str) -> Result<InterpreterResults, Error> {
        match &self.engine {
            #[cfg(feature = "docker")]
            Engine::Docker(interpreter) => interpreter.try_eval(input),
            #[cfg(feature = "native")]
            Engine::Native(interpreter) => interpreter.try_eval(input).map_err(Error::Native),
        }
    }

    /// Evaluate code, panicking on failure as in the original Mocktave API.
    pub fn eval(&self, input: &str) -> InterpreterResults {
        self.try_eval(input)
            .unwrap_or_else(|error| panic!("{error}"))
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new(Backend::default()).unwrap_or_else(|error| panic!("{error}"))
    }
}

#[cfg(feature = "native")]
impl From<crate::NativeInterpreter> for Interpreter {
    fn from(value: crate::NativeInterpreter) -> Self {
        Self {
            backend: Backend::Native,
            engine: Engine::Native(value),
        }
    }
}

/// Initialization or evaluation errors from the selected backend.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// Host I/O or runtime initialization failure.
    Io(io::Error),
    /// Native Octave launch, execution, or workspace error.
    #[cfg(feature = "native")]
    Native(crate::NativeError),
    /// Docker API error.
    #[cfg(feature = "docker")]
    Docker(bollard::errors::Error),
    /// Docker's Octave process exited unsuccessfully.
    #[cfg(feature = "docker")]
    DockerExecution {
        /// Exit code, or `None` if Docker did not provide one.
        status: Option<i64>,
        /// Captured Octave output and diagnostics.
        output: String,
    },
    /// The private runtime could not be validated or prepared.
    #[cfg(feature = "bundled")]
    Bundle(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(f, "{error}"),
            #[cfg(feature = "native")]
            Self::Native(error) => write!(f, "{error}"),
            #[cfg(feature = "docker")]
            Self::Docker(error) => write!(f, "Docker: {error}"),
            #[cfg(feature = "docker")]
            Self::DockerExecution { status, output } => {
                write!(f, "Octave in Docker failed ({status:?}): {output}")
            }
            #[cfg(feature = "bundled")]
            Self::Bundle(message) => write!(f, "Bundled Octave: {message}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            #[cfg(feature = "native")]
            Self::Native(error) => Some(error),
            #[cfg(feature = "docker")]
            Self::Docker(error) => Some(error),
            #[cfg(feature = "docker")]
            Self::DockerExecution { .. } => None,
            #[cfg(feature = "bundled")]
            Self::Bundle(_) => None,
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

#[cfg(feature = "docker")]
impl From<bollard::errors::Error> for Error {
    fn from(error: bollard::errors::Error) -> Self {
        Self::Docker(error)
    }
}
