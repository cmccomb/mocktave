#[cfg(feature = "bundled")]
fn bundle() -> Result<(), Box<dyn std::error::Error>> {
    use sha2::{Digest, Sha256};
    use std::{
        env, fs,
        io::{self, Read},
        path::PathBuf,
        time::Duration,
    };

    for key in [
        "MOCKTAVE_BUNDLE_ARCHIVE",
        "MOCKTAVE_BUNDLE_SHA256",
        "DOCS_RS",
    ] {
        println!("cargo:rerun-if-env-changed={key}");
    }
    println!("cargo:rerun-if-changed=bundles/manifest.json");
    let target = env::var("TARGET")?;
    let out = PathBuf::from(env::var_os("OUT_DIR").ok_or("OUT_DIR is missing")?);
    let destination = out.join("octave-runtime.tar.gz");
    println!("cargo:rustc-env=MOCKTAVE_BUNDLE_TARGET={target}");

    // docs.rs builds APIs without downloading or executing a native runtime.
    if env::var_os("DOCS_RS").is_some() {
        fs::write(&destination, [])?;
        println!("cargo:rustc-env=MOCKTAVE_BUNDLE_SHA256=docs-only");
        return Ok(());
    }

    let registry: serde_json::Value = serde_json::from_str(include_str!("bundles/manifest.json"))?;
    let entry = &registry["targets"][&target];
    let local = env::var_os("MOCKTAVE_BUNDLE_ARCHIVE");
    let digest = if local.is_some() {
        env::var("MOCKTAVE_BUNDLE_SHA256")
            .map_err(|_| "A local bundle requires MOCKTAVE_BUNDLE_SHA256")?
    } else {
        entry["sha256"]
            .as_str()
            .ok_or_else(|| {
                format!(
            "No published bundled runtime for {target} yet. Use the native/docker backend, \
             or test a maintainer artifact with MOCKTAVE_BUNDLE_ARCHIVE and MOCKTAVE_BUNDLE_SHA256."
        )
            })?
            .to_owned()
    };
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("The bundle SHA-256 must contain 64 hexadecimal characters".into());
    }

    let bytes = if let Some(path) = local {
        let path = PathBuf::from(path);
        println!("cargo:rerun-if-changed={}", path.display());
        fs::read(path)?
    } else if destination.is_file()
        && format!("{:x}", Sha256::digest(fs::read(&destination)?)) == digest.to_lowercase()
    {
        fs::read(&destination)?
    } else {
        let url = entry["url"]
            .as_str()
            .ok_or("The runtime has not been published yet")?;
        if !url.starts_with("https://") {
            return Err("Runtime downloads require HTTPS".into());
        }
        println!("cargo:warning=Downloading the pinned self-contained Octave runtime for {target}");
        let agent = ureq::Agent::config_builder()
            .timeout_global(Some(Duration::from_secs(300)))
            .build()
            .new_agent();
        let mut response = agent.get(url).call()?;
        let mut bytes = Vec::new();
        response
            .body_mut()
            .as_reader()
            .take(512 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() > 512 * 1024 * 1024 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Runtime archive exceeds 512 MiB",
            )
            .into());
        }
        bytes
    };
    if format!("{:x}", Sha256::digest(&bytes)) != digest.to_lowercase() {
        return Err("Runtime SHA-256 mismatch; refusing to embed unverified Octave".into());
    }
    fs::write(&destination, bytes)?;
    println!(
        "cargo:rustc-env=MOCKTAVE_BUNDLE_SHA256={}",
        digest.to_lowercase()
    );
    Ok(())
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    #[cfg(feature = "bundled")]
    if let Err(error) = bundle() {
        panic!("Could not prepare bundled Octave: {error}");
    }
}
