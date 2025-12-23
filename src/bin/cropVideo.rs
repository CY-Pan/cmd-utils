use clap::Parser;
use std::{path::Path, process::Command};

#[derive(Parser)]
struct Args {
    #[arg(short, long)]
    x: i32,
    #[arg(short, long)]
    y: i32,
    #[arg(short, long)]
    width: i32,
    #[arg(short, long)]
    height: i32,
    files: Vec<String>,
}

fn main() {
    let args = Args::parse();
    for f in &args.files {
        let path = Path::new(f);
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let mut parent = path.parent().unwrap().to_str().unwrap();
        if parent == "" {
            parent = ".";
        }

        let vf_arg = format!("crop={}:{}:{}:{}", args.width, args.height, args.x, args.y);

        let output_file = format!("{}/crop_{}", parent, file_name);

        let reencode_args = vec![
            "-i",
            f,
            "-vf",
            &vf_arg,
            "-c:v",
            if cfg!(target_os = "macos") {
                "h264_videotoolbox"
            } else {
                "h264_nvenc"
            },
            &output_file,
        ];

        Command::new("ffmpeg").args(reencode_args).status().unwrap();
    }
}
