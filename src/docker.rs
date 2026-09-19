use crate::{Error, InterpreterResults};

/// Reuse a Docker container for multiple evaluations.
///
/// Each evaluation starts a fresh Octave process; variables do not persist between calls.
/// ```
/// let interp = mocktave::Interpreter::default();
/// let res1 = interp.eval("a = 5+2");
/// assert_eq!(res1.get_scalar("a").unwrap(), 7_f64);
/// let res2 = interp.eval("a = ones(2, 2)");
/// assert_eq!(res2.get_matrix("a").unwrap(), vec![vec![1.0_f64; 2]; 2]);
/// let res3 = interp.eval("a = 'asdf'");
/// assert_eq!(res3.get_string("a").unwrap(), "asdf");
/// ```
pub struct Interpreter {
    docker: bollard::Docker,
    id: String,
}

impl Interpreter {
    pub(crate) fn try_new() -> Result<Self, Error> {
        use futures_util::TryStreamExt;
        tokio::runtime::Runtime::new()?.block_on(async {
            let docker = bollard::Docker::connect_with_local_defaults()?;
            docker
                .create_image(
                    Some(bollard::image::CreateImageOptions {
                        from_image: "gnuoctave/octave",
                        tag: "8.1.0",
                        ..Default::default()
                    }),
                    None,
                    None,
                )
                .try_collect::<Vec<_>>()
                .await?;

            let alpine_config = bollard::container::Config {
                image: Some("gnuoctave/octave:8.1.0"),
                tty: Some(true),
                ..Default::default()
            };

            let id = docker
                .create_container::<&str, &str>(None, alpine_config)
                .await?
                .id;

            docker.start_container::<String>(&id, None).await?;

            Ok(Interpreter { docker, id })
        })
    }
}

impl Interpreter {
    /// This function does the heavy lifting in the interpreter struct.
    pub(crate) fn try_eval(&self, input: &str) -> Result<InterpreterResults, Error> {
        use futures_util::stream::StreamExt;
        tokio::runtime::Runtime::new()?.block_on(async {
            // non interactive
            let exec = self
                .docker
                .create_exec(
                    &self.id.clone(),
                    bollard::exec::CreateExecOptions {
                        attach_stdout: Some(true),
                        attach_stderr: Some(true),
                        cmd: Some(vec![
                            "octave",
                            "--eval",
                            &(input.to_string() + "\n\nsave(\"-\", \"*\");"),
                        ]),
                        ..Default::default()
                    },
                )
                .await?
                .id;

            let mut output_text = vec!["".to_string(); 0];

            if let bollard::exec::StartExecResults::Attached { mut output, .. } =
                self.docker.start_exec(&exec, None).await?
            {
                while let Some(msg) = output.next().await {
                    let msg = msg?;
                    output_text.push(msg.to_string());
                    print!("{}", msg);
                }
            } else {
                unreachable!();
            }

            let output = output_text.join("");
            let status = self.docker.inspect_exec(&exec).await?.exit_code;
            if status != Some(0) {
                return Err(Error::DockerExecution { status, output });
            }
            Ok(InterpreterResults::from(output))
        })
    }
}

impl Drop for Interpreter {
    fn drop(&mut self) {
        if let Ok(runtime) = tokio::runtime::Runtime::new() {
            let _ = runtime.block_on(self.docker.remove_container(
                &self.id,
                Some(bollard::container::RemoveContainerOptions {
                    force: true,
                    ..Default::default()
                }),
            ));
        }
    }
}
