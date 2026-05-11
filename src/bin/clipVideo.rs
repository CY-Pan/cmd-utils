use clap::{ArgGroup, Parser};
use cmd_utils::{VideoClipConfig, VideoEncodeInfo};

#[derive(Parser)]
#[command(group(ArgGroup::new("time").required(true).multiple(true)))]
struct Args {
    #[arg(short, long, group = "time")]
    start: Option<String>,
    #[arg(short, long, group = "time")]
    to: Option<String>,
    files: Vec<String>,
}

fn main() {
    let args = Args::parse();

    for f in &args.files {
        let output_file =
            cmd_utils::make_unique_filename(cmd_utils::add_prefix_to_file(f, "clip_"));

        cmd_utils::reencode_video(
            f,
            output_file.to_str().unwrap(),
            VideoEncodeInfo {
                only_copy: true,
                clip_config: Some(VideoClipConfig {
                    from: args.start.clone(),
                    to: args.to.clone(),
                }),
                ..Default::default()
            },
        );
    }
}
