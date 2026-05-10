use clap::Parser;

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
            cmd_utils::VideoEncodeInfo {
                hardware_encode: true,
                crop_config: Some(cmd_utils::VideoCropConfig {
                    x: args.x,
                    y: args.y,
                    width: args.width,
                    height: args.height,
                }),
                ..Default::default()
            },
        )
    }
}
