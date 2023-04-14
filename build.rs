use std::{
    fs::{remove_file, File},
    io::copy,
    io::{Read, Write},
    ops::Not,
    path::PathBuf,
    process::Output,
    str::FromStr,
};

// Download from a url to file_name
#[cfg(feature = "brew")]
fn download(url: &str, file_name: &str) -> usize {
    let file_contents = minreq::get(url).send().expect("Could not open URL");
    let path = &PathBuf::from_str(&*(std::env::var("OUT_DIR").unwrap() + "/" + file_name)).unwrap();
    File::create(path)
        .expect("creation failed")
        .write(file_contents.as_bytes())
        .expect("TODO: panic message")
}

// Unzip archive_name
#[cfg(feature = "brew")]
fn unzip(archive_name: &str) {
    use decompress::{DecompressError, Decompression};
    let archive_path = &PathBuf::from_str(&std::env::var("OUT_DIR").unwrap())
        .unwrap()
        .join(archive_name);
    let out_dir = &PathBuf::from_str(&std::env::var("OUT_DIR").unwrap()).unwrap();
    if archive_path.is_dir().not() {
        let _ = decompress::decompress(
            archive_path,
            out_dir,
            &decompress::ExtractOptsBuilder::default().build().unwrap(),
        );
    }
}

// Prepare brew environment (not sure what do)
#[cfg(feature = "brew")]
fn brew_shellenv() {
    std::process::Command::new("bin/brew")
        .arg("shellenv")
        .current_dir(std::env::var("OUT_DIR").unwrap() + "/Homebrew-brew-9912dca/")
        .status()
        .expect("failed to execute process");
}

#[cfg(feature = "brew")]
fn brew_deps(package_name: &str) -> Vec<String> {
    // Update homebrew
    let string_output = String::from_utf8(
        std::process::Command::new("bin/brew")
            .arg("deps")
            .arg(package_name)
            .arg("-n")
            .current_dir(std::env::var("OUT_DIR").unwrap() + "/Homebrew-brew-9912dca/")
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

#[cfg(feature = "brew")]
fn brew_update() {
    // Update homebrew
    std::process::Command::new("bin/brew")
        .arg("update")
        .arg("--force")
        .arg("--quiet")
        .arg("--verbose")
        .current_dir(std::env::var("OUT_DIR").unwrap() + "/Homebrew-brew-9912dca/")
        .status()
        .expect("failed to execute process");
}

#[cfg(feature = "brew")]
fn brew_install(package_name: &str, build_from_source: bool, ignore_dependencies: bool) {
    let mut stub = std::process::Command::new("bin/brew");
    let cmd = stub
        .arg("install")
        .arg(package_name)
        .arg("--verbose")
        .current_dir(std::env::var("OUT_DIR").unwrap() + "/Homebrew-brew-9912dca/");
    if ignore_dependencies {
        cmd.arg("--ignore-dependencies");
    }
    if build_from_source {
        cmd.arg("--build-from-source");
    }
    cmd.status().expect("failed to execute process");
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=src");

    // Download and unzip homebrew
    #[cfg(feature = "brew")]
    {
        download(
            "https://github.com/Homebrew/brew/tarball/master",
            "brew.tar.gz",
        );
        unzip("brew.tar.gz");

        // Prepare homebrew environment
        brew_shellenv();
        brew_update();

        // for package in brew_deps("octave") {
        //     brew_install(&package, true, false);
        // }
        brew_install("octave", true, true);
    }
}
