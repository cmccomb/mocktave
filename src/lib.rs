#![warn(clippy::all)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]
#![warn(clippy::missing_docs_in_private_items)]
#![doc = include_str!("../README.md")]

use bollard::container::{Config, RemoveContainerOptions};
use bollard::Docker;

use bollard::exec::{CreateExecOptions, StartExecResults};
use bollard::image::CreateImageOptions;
use futures_util::stream::StreamExt;
use futures_util::TryStreamExt;

/// This is the only function ou need. It evaluates octave syntax.
/// ```
/// use mocktave::eval;
/// let should_be_seven = eval("5+2");
/// assert_eq!(should_be_seven, "ans =  7\n");
/// ```
#[tokio::main]
pub async fn eval(input: &str) -> String {
    const IMAGE: &str = "mtmiller/octave:latest";
    let docker = Docker::connect_with_socket_defaults().unwrap();

    docker
        .create_image(
            Some(CreateImageOptions {
                from_image: IMAGE,
                ..Default::default()
            }),
            None,
            None,
        )
        .try_collect::<Vec<_>>()
        .await
        .expect("Await fails?");

    let alpine_config = Config {
        image: Some(IMAGE),
        tty: Some(true),
        ..Default::default()
    };

    let id = docker
        .create_container::<&str, &str>(None, alpine_config)
        .await
        .expect("Await fails?")
        .id;
    docker
        .start_container::<String>(&id, None)
        .await
        .expect("Await fails?");

    // non interactive
    let exec = docker
        .create_exec(
            &id,
            CreateExecOptions {
                attach_stdout: Some(true),
                attach_stderr: Some(true),
                cmd: Some(vec![
                    "octave",
                    "--eval",
                    &(input.to_string() + "save(\"-\", \"*\");"),
                ]),
                ..Default::default()
            },
        )
        .await
        .expect("Await fails?")
        .id;

    let mut output_text = vec!["".to_string(); 0];

    if let StartExecResults::Attached { mut output, .. } =
        docker.start_exec(&exec, None).await.expect("Await fails?")
    {
        while let Some(Ok(msg)) = output.next().await {
            output_text.push(msg.to_string());
            print!("{}", msg);
        }
    } else {
        unreachable!();
    }

    docker
        .remove_container(
            &id,
            Some(RemoveContainerOptions {
                force: true,
                ..Default::default()
            }),
        )
        .await
        .expect("Await fails?");

    output_text.join("")
}
