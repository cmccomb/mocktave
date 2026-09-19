use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use crate::{Error, NativeInterpreter};

static ARCHIVE: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/octave-runtime.tar.gz"));
const DIGEST: &str = env!("MOCKTAVE_BUNDLE_SHA256");
const TARGET: &str = env!("MOCKTAVE_BUNDLE_TARGET");

pub(crate) fn interpreter() -> Result<NativeInterpreter, Error> {
    if ARCHIVE.is_empty() {
        return Err(Error::Bundle(
            "This documentation-only build contains no runtime".into(),
        ));
    }
    let cache = match std::env::var_os("MOCKTAVE_CACHE_DIR") {
        Some(path) => PathBuf::from(path),
        None => directories::BaseDirs::new()
            .ok_or_else(|| Error::Bundle("Cannot locate a user cache directory".into()))?
            .cache_dir()
            .join("mocktave"),
    };
    prepare(ARCHIVE, &cache, DIGEST, TARGET)
}

fn validate(root: &Path, target: &str) -> Result<PathBuf, Error> {
    let metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("mocktave-runtime.json"))?)
            .map_err(|error| Error::Bundle(format!("Invalid runtime metadata: {error}")))?;
    if metadata["schema"] != 1 || metadata["target"].as_str() != Some(target) {
        return Err(Error::Bundle(format!(
            "Runtime target/schema does not match {target}"
        )));
    }
    let name = metadata["executable"]
        .as_str()
        .ok_or_else(|| Error::Bundle("Runtime has no executable path".into()))?;
    let path = Path::new(name);
    if !safe_path(path) || !root.join(path).is_file() {
        return Err(Error::Bundle(
            "Runtime executable is missing or outside its directory".into(),
        ));
    }
    Ok(root.join(path))
}

fn safe_path(path: &Path) -> bool {
    !path.as_os_str().is_empty()
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
}

fn prepare(
    archive: &[u8],
    cache: &Path,
    digest: &str,
    target: &str,
) -> Result<NativeInterpreter, Error> {
    fs::create_dir_all(cache)?;
    let cache = cache.canonicalize()?;
    let runtime = cache.join(digest);
    let lock = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(cache.join(format!("{digest}.lock")))?;
    fs2::FileExt::lock_exclusive(&lock)?;
    if runtime.is_dir()
        && fs::read_to_string(runtime.join(".complete"))
            .ok()
            .as_deref()
            == Some(digest)
    {
        let executable = validate(&runtime, target)?;
        return Ok(NativeInterpreter::with_runtime(executable, runtime));
    }
    // Extract to a sibling first so another process never sees a partial runtime.
    let staging = tempfile::Builder::new()
        .prefix("extract-")
        .tempdir_in(&cache)?;
    let stream = flate2::read::GzDecoder::new(archive);
    let mut archive = tar::Archive::new(stream);
    let mut total = 0_u64;
    for entry in archive.entries()? {
        let mut entry = entry?;
        let path = entry.path()?.into_owned();
        if !safe_path(&path)
            || !(entry.header().entry_type().is_file() || entry.header().entry_type().is_dir())
        {
            return Err(Error::Bundle(format!(
                "Unsafe runtime archive entry: {}",
                path.display()
            )));
        }
        total = total
            .checked_add(entry.size())
            .ok_or_else(|| Error::Bundle("Runtime is too large".into()))?;
        if total > 2 * 1024 * 1024 * 1024 {
            return Err(Error::Bundle("Runtime exceeds 2 GiB unpacked".into()));
        }
        entry.unpack_in(staging.path())?;
    }
    validate(staging.path(), target)?;
    fs::write(staging.path().join(".complete"), digest)?;
    // Only an incomplete, hash-addressed cache owned by this backend is replaced.
    if runtime.exists() {
        fs::remove_dir_all(&runtime)?;
    }
    fs::rename(staging.path(), &runtime)?;
    let executable = validate(&runtime, target)?;
    Ok(NativeInterpreter::with_runtime(executable, runtime))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn archive(target: &str, symlink: bool) -> Vec<u8> {
        let gzip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
        let mut tar = tar::Builder::new(gzip);
        let data = format!(r#"{{"schema":1,"target":"{target}","executable":"bin/octave-cli"}}"#);
        for (path, bytes) in [
            ("mocktave-runtime.json", data.as_bytes()),
            ("bin/octave-cli", b"test"),
        ] {
            let mut header = tar::Header::new_gnu();
            header.set_size(bytes.len() as u64);
            header.set_mode(0o755);
            header.set_cksum();
            tar.append_data(&mut header, path, bytes).unwrap();
        }
        if symlink {
            let mut header = tar::Header::new_gnu();
            header.set_entry_type(tar::EntryType::Symlink);
            header.set_size(0);
            tar.append_link(&mut header, "escape", "../outside")
                .unwrap();
        }
        tar.into_inner().unwrap().finish().unwrap()
    }

    #[test]
    fn rejects_a_bundle_for_another_target_without_installing_it() {
        let cache = tempfile::tempdir().unwrap();
        assert!(prepare(
            &archive("wrong-target", false),
            cache.path(),
            "test",
            TARGET
        )
        .is_err());
        assert!(!cache.path().join("test").exists());
    }

    #[test]
    fn rejects_archive_links() {
        let cache = tempfile::tempdir().unwrap();
        assert!(prepare(&archive(TARGET, true), cache.path(), "test", TARGET).is_err());
        assert!(!cache.path().join("test").exists());
    }

    #[test]
    fn concurrent_initialization_installs_once_and_reuses_the_cache() {
        let cache = tempfile::tempdir().unwrap();
        let workers: Vec<_> = (0..4)
            .map(|_| {
                let path = cache.path().to_owned();
                std::thread::spawn(move || {
                    prepare(&archive(TARGET, false), &path, "test", TARGET).unwrap()
                })
            })
            .collect();
        for worker in workers {
            worker.join().unwrap();
        }
        let executable = cache.path().join("test/bin/octave-cli");
        fs::OpenOptions::new()
            .append(true)
            .open(&executable)
            .unwrap()
            .write_all(b" marker")
            .unwrap();
        prepare(&archive(TARGET, false), cache.path(), "test", TARGET).unwrap();
        assert_eq!(fs::read(executable).unwrap(), b"test marker");
    }

    #[test]
    fn rejects_absolute_and_parent_paths() {
        assert!(!safe_path(Path::new("/tmp/escape")));
        assert!(!safe_path(Path::new("../escape")));
        assert!(safe_path(Path::new("bin/octave-cli")));
    }
}
