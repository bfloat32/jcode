fn publish_no_replace(
    output_dir: &Path,
    final_path: &Path,
    content_key: &str,
    bytes: &[u8],
    publication: PublicationMode,
) -> Result<(), EvalError> {
    let cleanup = OwnedTemp::create(output_dir, content_key, bytes)?;
    match publication {
        PublicationMode::Publish => {}
        #[cfg(test)]
        PublicationMode::FailBeforeLink => {
            return Err(EvalError::new(
                EvalErrorKind::Io,
                "injected pre-publication failure",
            ));
        }
    }
    match fs::hard_link(&cleanup.path, final_path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return Err(EvalError::new(
                EvalErrorKind::OverwriteRefused,
                format!("baseline already exists: {}", final_path.display()),
            ));
        }
        Err(error) => return Err(EvalError::io("publish", final_path, &error)),
    }
    cleanup.remove()?;
    sync_directory(output_dir)
}

struct OwnedTemp {
    path: PathBuf,
    armed: bool,
}

impl OwnedTemp {
    fn create(output_dir: &Path, content_key: &str, bytes: &[u8]) -> Result<Self, EvalError> {
        static NEXT_TEMP: AtomicU64 = AtomicU64::new(0);
        const MAX_ATTEMPTS: u8 = 16;
        for _ in 0..MAX_ATTEMPTS {
            let nonce = NEXT_TEMP.fetch_add(1, Ordering::Relaxed);
            let path = output_dir.join(format!(
                ".intel-eval-{content_key}-{}-{nonce}.tmp",
                std::process::id()
            ));
            let open = OpenOptions::new().write(true).create_new(true).open(&path);
            let mut file = match open {
                Ok(file) => file,
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(EvalError::io("create temporary artifact", &path, &error));
                }
            };
            if let Err(error) = file.write_all(bytes) {
                drop(file);
                return Err(cleanup_failed_temp("write temporary artifact", &path, &error));
            }
            if let Err(error) = file.sync_all() {
                drop(file);
                return Err(cleanup_failed_temp("sync temporary artifact", &path, &error));
            }
            drop(file);
            return Ok(Self { path, armed: true });
        }
        Err(EvalError::new(
            EvalErrorKind::Io,
            "temporary artifact collision retry limit reached",
        ))
    }

    fn remove(mut self) -> Result<(), EvalError> {
        fs::remove_file(&self.path)
            .map_err(|error| EvalError::io("remove temporary artifact", &self.path, &error))?;
        self.armed = false;
        Ok(())
    }
}

impl Drop for OwnedTemp {
    fn drop(&mut self) {
        if self.armed {
            match fs::remove_file(&self.path) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(_) => {}
            }
        }
    }
}

fn cleanup_failed_temp(action: &str, path: &Path, error: &std::io::Error) -> EvalError {
    match fs::remove_file(path) {
        Ok(()) => EvalError::io(action, path, error),
        Err(cleanup) => EvalError::new(
            EvalErrorKind::Io,
            format!("{action} {}: {error}; cleanup failed: {cleanup}", path.display()),
        ),
    }
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> Result<(), EvalError> {
    File::open(path)
        .and_then(|directory| directory.sync_all())
        .map_err(|error| EvalError::io("sync output directory", path, &error))
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> Result<(), EvalError> {
    Ok(())
}
