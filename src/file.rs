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

const UNSUPPORTED_VIDEO_EXTENSIONS: [&str; 1] = ["qt"];

pub fn replace_unsupported_video_exts(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let mut result = path.to_path_buf();

    let ext = path.extension().unwrap_or_default().to_str().unwrap_or("");

    if UNSUPPORTED_VIDEO_EXTENSIONS
        .into_iter()
        .any(|blacklisted| ext.eq_ignore_ascii_case(blacklisted))
    {
        result.set_extension("mp4");
    }

    result
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn replace_extension_if_blacklisted_changes_blacklisted_suffix() {
        let path = Path::new("video.qt");
        let result = replace_unsupported_video_exts(path);
        assert_eq!(result, Path::new("video.mp4"));
    }

    #[test]
    fn replace_extension_if_blacklisted_keeps_non_blacklisted_suffix() {
        let path = Path::new("audio.mp3");
        let result = replace_unsupported_video_exts(path);
        assert_eq!(result, Path::new("audio.mp3"));
    }

    #[test]
    fn replace_extension_if_blacklisted_is_case_insensitive() {
        let path = Path::new("VIDEO.QT");
        let result = replace_unsupported_video_exts(path);
        assert_eq!(result, Path::new("VIDEO.mp4"));
    }
}
