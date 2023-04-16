#[cfg(any(feature = "brew-local", feature = "brew-src"))]
const BREW_VERSION: &str = "4.0.13";

#[cfg(all(
    feature = "docker",
    not(feature = "brew-local"),
    not(feature = "brew-src")
))]
mod build {
    pub(crate) fn go() {}
}

#[cfg(all(feature = "brew-local", not(feature = "brew-src")))]
mod build {
    const BREW_COMMAND: &str = "brew";

    pub(crate) fn go() {
        super::brew_install(BREW_COMMAND, "octave", false, false, false, None);
    }
}

#[cfg(feature = "brew-src")]
mod build {
    use std::{
        fs::{remove_file, File},
        io::copy,
        io::{Read, Write},
        ops::Not,
        process::Output,
        str::FromStr,
    };

    const BREW_COMMAND: &str = "bin/brew";

    // Download from a url to file_name
    fn download(url: &str, file_name: &str) -> usize {
        let file_contents = minreq::get(url).send().expect("Could not open URL");
        let path =
            &std::path::PathBuf::from_str(&*(std::env::var("OUT_DIR").unwrap() + "/" + file_name))
                .unwrap();
        std::fs::File::create(path)
            .expect("creation failed")
            .write(file_contents.as_bytes())
            .expect("TODO: panic message")
    }

    // Unzip archive_name
    fn unzip(archive_name: &str) {
        use decompress::{DecompressError, Decompression};
        let archive_path = &std::path::PathBuf::from_str(&std::env::var("OUT_DIR").unwrap())
            .unwrap()
            .join(archive_name);
        let out_dir = &std::path::PathBuf::from_str(&std::env::var("OUT_DIR").unwrap()).unwrap();
        if archive_path.is_dir().not() {
            let _ = decompress::decompress(
                archive_path,
                out_dir,
                &decompress::ExtractOptsBuilder::default().build().unwrap(),
            );
        }
    }

    // Prepare brew environment (not sure what do)
    fn brew_shellenv() {
        std::process::Command::new(BREW_COMMAND)
            .arg("shellenv")
            .current_dir(
                std::env::var("OUT_DIR").unwrap() + &format!("/brew-{}/", super::BREW_VERSION),
            )
            .status()
            .expect("failed to execute process");
    }

    fn brew_deps(package_name: &str) -> Vec<String> {
        // Update homebrew
        let string_output = String::from_utf8(
            std::process::Command::new(BREW_COMMAND)
                .arg("deps")
                .arg(package_name)
                .arg("-n")
                .current_dir(
                    std::env::var("OUT_DIR").unwrap() + &format!("/brew-{}/", super::BREW_VERSION),
                )
                .output()
                .unwrap()
                .stdout,
        )
        .unwrap();

        string_output[0..string_output.len() - 1]
            .split("\n")
            .into_iter()
            .map(String::from)
            .collect()
    }

    fn brew_update() {
        // Update homebrew
        std::process::Command::new(BREW_COMMAND)
            .arg("update")
            .arg("--force")
            .arg("--quiet")
            .arg("--verbose")
            .current_dir(
                std::env::var("OUT_DIR").unwrap() + &format!("/brew-{}/", super::BREW_VERSION),
            )
            .status()
            .expect("failed to execute process");
    }

    pub(crate) fn go() {
        // Download and unzip homebrew
        download(
            &format!(
                "https://github.com/Homebrew/brew/archive/refs/tags/{}.tar.gz",
                super::BREW_VERSION
            ),
            "brew.tar.gz",
        );
        unzip("brew.tar.gz");

        // Prepare homebrew environment
        brew_shellenv();
        brew_update();

        // Install octave deps
        for dep in brew_deps("octave") {
            super::brew_install(BREW_COMMAND, &dep, true, false, false, None);
        }

        // Install SED and move to the right location
        super::brew_install(BREW_COMMAND, "gnu-sed", true, false, false, None);
        std::process::Command::new("cp")
            .arg(
                std::env::var("OUT_DIR").unwrap()
                    + "/brew-"
                    + super::BREW_VERSION
                    + "/Cellar/gnu-sed/4.9/libexec/gnubin/sed",
            )
            .arg(
                std::env::var("OUT_DIR").unwrap()
                    + "/brew-"
                    + super::BREW_VERSION
                    + "/Library/Homebrew/shims/mac/super",
            )
            .output()
            .expect("oops");

        // Copy gfortran
        std::process::Command::new("cp")
            .arg(
                std::env::var("OUT_DIR").unwrap()
                    + "/brew-"
                    + super::BREW_VERSION
                    + "/Cellar/gcc/12.2.0/bin/gfortran-12",
            )
            .arg(
                std::env::var("OUT_DIR").unwrap()
                    + "/brew-"
                    + super::BREW_VERSION
                    + "/Library/Homebrew/shims/mac/super",
            )
            .output()
            .expect("oops");

        super::brew_install(
            BREW_COMMAND,
            "octave",
            true,
            true,
            false,
            Some(
                &*(std::env::var("OUT_DIR").unwrap()
                    + "/brew-"
                    + super::BREW_VERSION
                    + "/opt/gnu-sed/libexec/gnubin"),
            ),
        );
    }
}

#[cfg(any(feature = "brew-src", feature = "brew-local"))]
fn brew_install(
    brew_command: &str,
    package_name: &str,
    build_from_source: bool,
    ignore_dependencies: bool,
    only_dependencies: bool,
    optional_path: Option<&str>,
) {
    let mut stub = std::process::Command::new(brew_command);
    let mut cmd = stub
        .arg("install")
        .arg(package_name)
        .arg("--verbose")
        .arg("--force");
    if cfg!(feature = "brew-src") {
        cmd =
            cmd.current_dir(std::env::var("OUT_DIR").unwrap() + &format!("/brew-{BREW_VERSION}/"));
    }
    match optional_path {
        Some(path) => cmd = cmd.env("PATH", path.to_string() + ":" + env!("PATH")),
        None => {}
    }
    if ignore_dependencies {
        cmd = cmd.arg("--ignore-dependencies");
    }
    if build_from_source {
        cmd = cmd.arg("--build-from-source");
    }
    if only_dependencies {
        cmd = cmd.arg("--only-dependencies");
    }
    cmd.status().expect("failed to execute process");
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");

    build::go()
}
