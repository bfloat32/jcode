use jcode_storage as storage;

#[cfg(not(test))]
pub(super) const fn telemetry_available() -> bool {
    false
}

#[cfg(test)]
pub(super) fn telemetry_available() -> bool {
    std::env::var_os("JCODE_TEST_ENABLE_TELEMETRY").is_some()
}

pub fn enforce_disabled_policy() {
    if telemetry_available() {
        return;
    }

    let Ok(root) = storage::jcode_dir() else {
        return;
    };
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };

    for entry in entries.filter_map(Result::ok) {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if !name.starts_with("telemetry_")
            && name != "install_conversion_id"
            && name != "no_telemetry"
        {
            continue;
        }

        let path = entry.path();
        if entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            let _ = std::fs::remove_dir_all(path);
        } else {
            let _ = std::fs::remove_file(path);
        }
    }
}
