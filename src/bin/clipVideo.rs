use clap::{ArgGroup, Parser};
use std::{path::Path, process::Command};

#[derive(Parser)]
#[command(group(ArgGroup::new("time").required(true)))]
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
        let path = Path::new(f);
        let file_name = path.file_name().unwrap().to_str().unwrap();
        let mut parent = path.parent().unwrap().to_str().unwrap();
        if parent == "" {
            parent = ".";
        }

        let mut reencode_args = vec!["-i", f];
        if let Some(start) = &args.start {
            reencode_args.extend(["-ss", start]);
        }
        if let Some(to) = &args.to {
            reencode_args.extend(["-to", to]);
        }

        let output_file = format!("{}/clip_{}", parent, file_name);
        reencode_args.extend(["-c", "copy", &output_file]);

        Command::new("ffmpeg").args(reencode_args).status().unwrap();
    }
}
