use clap::Parser;
use cmd_utils::{VideoCropConfig, VideoEncodeInfo};

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value_t = 0)]
    x: u32,
    #[arg(short, long, default_value_t = 0)]
    y: u32,
    #[arg(short, long, default_value_t = u32::MAX)]
    width: u32,
    #[arg(short, long, default_value_t = u32::MAX)]
    height: u32,
    files: Vec<String>,
}

fn main() {
    let args = Args::parse();
    for f in &args.files {
        let media_info = cmd_utils::probe_media(f);
        let video_info = cmd_utils::get_video_stream(&media_info).unwrap();

        let width = if args.width == u32::MAX {
            video_info.width
        } else {
            args.width
        };

        let height = if args.height == u32::MAX {
            video_info.height
        } else {
            args.height
        };

        let output_file = cmd_utils::make_unique_filename(cmd_utils::add_prefix_to_file(
            cmd_utils::replace_unsupported_video_exts(f),
            "crop_",
        ));

        cmd_utils::reencode_video(
            f,
            output_file.to_str().unwrap(),
            VideoEncodeInfo {
                crop_config: Some(VideoCropConfig {
                    x: args.x,
                    y: args.y,
                    width: width,
                    height: height,
                }),
                ..VideoEncodeInfo::hardware_default()
            },
        )
    }
}
