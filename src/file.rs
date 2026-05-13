use std::path::{Path, PathBuf};

pub fn add_prefix_to_file(path: impl AsRef<Path>, prefix: impl AsRef<str>) -> PathBuf {
    let path = path.as_ref();
    path.parent().unwrap_or(Path::new(".")).join(format!(
        "{}{}.{}",
        prefix.as_ref(),
        path.file_stem().unwrap().to_str().unwrap(),
        path.extension().unwrap().to_str().unwrap()
    ))
}

pub fn add_suffix_to_file(path: impl AsRef<Path>, suffix: impl AsRef<str>) -> PathBuf {
    let path = path.as_ref();
    path.parent().unwrap_or(Path::new(".")).join(format!(
        "{}{}.{}",
        path.file_stem().unwrap().to_str().unwrap(),
        suffix.as_ref(),
        path.extension().unwrap().to_str().unwrap()
    ))
}

pub fn make_unique_filename(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    if !path.exists() {
        return path.to_path_buf();
    }

    for i in 2.. {
        let new_path = add_suffix_to_file(path, format!("_{}", i));
        if !new_path.exists() {
            return new_path;
        }
    }
    unreachable!();
}
