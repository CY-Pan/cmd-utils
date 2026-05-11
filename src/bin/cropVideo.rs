use clap::Parser;
use cmd_utils::{VideoCropConfig, VideoEncodeInfo};

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    x: u32,
    #[arg(short, long)]
    y: u32,
    #[arg(short, long)]
    width: u32,
    #[arg(short, long)]
    height: u32,
    files: Vec<String>,
}

fn main() {
    let args = Args::parse();
    for f in &args.files {
        let output_file =
            cmd_utils::make_unique_filename(cmd_utils::add_prefix_to_file(f, "crop_"));

        cmd_utils::reencode_video(
            f,
            output_file.to_str().unwrap(),
            VideoEncodeInfo {
                crop_config: Some(VideoCropConfig {
                    x: args.x,
                    y: args.y,
                    width: args.width,
                    height: args.height,
                }),
                ..VideoEncodeInfo::hardware_default()
            },
        )
    }
}
