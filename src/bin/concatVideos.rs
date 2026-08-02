use clap::Parser;
use cmd_utils::{concat_differences, probe_media};
use std::{
    env,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    process::{self, Command, ExitCode},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    output: Option<PathBuf>,
    #[arg(short, long)]
    force: bool,
    #[arg(required = true, num_args = 2..)]
    input: Vec<PathBuf>,
}

struct TempConcatList {
    path: Option<PathBuf>,
}

impl TempConcatList {
    fn path(&self) -> &Path {
        self.path.as_deref().unwrap()
    }

    fn remove(mut self) -> Result<(), String> {
        let path = self.path.take().unwrap();
        fs::remove_file(&path).map_err(|error| {
            format!(
                "failed to remove temporary concat list <{}>: {error}",
                path.display()
            )
        })
    }
}

impl Drop for TempConcatList {
    fn drop(&mut self) {
        if let Some(path) = self.path.take()
            && let Err(error) = fs::remove_file(&path)
        {
            eprintln!(
                "warning: failed to remove temporary concat list <{}>: {error}",
                path.display()
            );
        }
    }
}

fn escape_ffconcat_path(path: &Path) -> String {
    let mut normalized = path.to_string_lossy().replace('\\', "/");
    if normalized.starts_with("//") {
        normalized.insert_str(0, "file:");
    }
    format!("'{}'", normalized.replace('\'', "'\\''"))
}

fn absolute_path(path: &Path) -> Result<PathBuf, String> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        env::current_dir()
            .map(|directory| directory.join(path))
            .map_err(|error| format!("failed to resolve <{}>: {error}", path.display()))
    }
}

fn create_concat_list(inputs: &[PathBuf]) -> Result<TempConcatList, String> {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = env::temp_dir().join(format!("concatVideos-{}-{nonce}.ffconcat", process::id()));
    create_concat_list_at(inputs, path)
}

fn create_concat_list_at(inputs: &[PathBuf], path: PathBuf) -> Result<TempConcatList, String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .map_err(|error| format!("failed to create <{}>: {error}", path.display()))?;
    let concat_list = TempConcatList { path: Some(path) };

    writeln!(file, "ffconcat version 1.0").map_err(|error| {
        format!(
            "failed to write <{}>: {error}",
            concat_list.path().display()
        )
    })?;
    for input in inputs {
        writeln!(file, "file {}", escape_ffconcat_path(input)).map_err(|error| {
            format!(
                "failed to write <{}>: {error}",
                concat_list.path().display()
            )
        })?;
    }
    file.flush().map_err(|error| {
        format!(
            "failed to flush <{}>: {error}",
            concat_list.path().display()
        )
    })?;

    Ok(concat_list)
}

fn run(args: Args) -> Result<(), String> {
    for input in &args.input {
        if !input.is_file() {
            return Err(format!("input is not a file: <{}>", input.display()));
        }
    }

    let output = args
        .output
        .unwrap_or_else(|| cmd_utils::add_suffix_to_file(&args.input[0], "_concat"));
    if output.exists() {
        return Err(format!("output already exists: <{}>", output.display()));
    }

    let reference = probe_media(&args.input[0]);
    let mut incompatible = false;
    for input in &args.input[1..] {
        let candidate = probe_media(input);
        let differences = concat_differences(&reference, &candidate);
        if differences.is_empty() {
            continue;
        }

        incompatible = true;
        eprintln!("incompatible media properties in <{}>:", input.display());
        for difference in differences {
            match difference.stream_index {
                Some(index) => eprintln!(
                    "  stream {index} {}: expected {}, actual {}",
                    difference.field, difference.expected, difference.actual
                ),
                None => eprintln!(
                    "  {}: expected {}, actual {}",
                    difference.field, difference.expected, difference.actual
                ),
            }
        }
    }

    if incompatible && !args.force {
        return Err("media properties are incompatible; pass --force to continue".to_string());
    }
    if incompatible {
        eprintln!("warning: continuing despite incompatible media properties");
    }

    let absolute_inputs = args
        .input
        .iter()
        .map(|input| absolute_path(input))
        .collect::<Result<Vec<_>, _>>()?;
    let concat_list = create_concat_list(&absolute_inputs)?;
    let status_result = Command::new("ffmpeg")
        .args(["-f", "concat", "-safe", "0", "-i"])
        .arg(concat_list.path())
        .args(["-map", "0", "-c", "copy", "-n"])
        .arg(&output)
        .status()
        .map_err(|error| format!("failed to run ffmpeg: {error}"));
    let cleanup_result = concat_list.remove();
    let status = match (status_result, cleanup_result) {
        (Ok(status), Ok(())) => status,
        (Err(ffmpeg_error), Ok(())) => return Err(ffmpeg_error),
        (Ok(_), Err(cleanup_error)) => return Err(cleanup_error),
        (Err(ffmpeg_error), Err(cleanup_error)) => {
            return Err(format!("{ffmpeg_error}; {cleanup_error}"));
        }
    };

    if !status.success() {
        return Err(format!("ffmpeg exited with status {status}"));
    }

    Ok(())
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn escape_ffconcat_path_normalizes_windows_separators_and_quotes() {
        assert_eq!(
            escape_ffconcat_path(Path::new(r"C:\clips\Bob's video.mp4")),
            r"'C:/clips/Bob'\''s video.mp4'"
        );
    }

    #[test]
    fn escape_ffconcat_path_uses_file_protocol_for_unc_paths() {
        assert_eq!(
            escape_ffconcat_path(Path::new(r"\\server\share\clip.mp4")),
            r"'file://server/share/clip.mp4'"
        );
    }

    #[test]
    fn concat_list_collision_does_not_delete_existing_file() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "concatVideos-collision-test-{}-{nonce}.ffconcat",
            process::id()
        ));
        fs::write(&path, "owned elsewhere").unwrap();

        let result = create_concat_list_at(&[], path.clone());

        assert!(result.is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), "owned elsewhere");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn concat_list_remove_reports_cleanup_failure() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!(
            "concatVideos-missing-test-{}-{nonce}.ffconcat",
            process::id()
        ));
        let concat_list = TempConcatList { path: Some(path) };

        let error = concat_list.remove().unwrap_err();

        assert!(error.starts_with("failed to remove temporary concat list"));
    }
}
