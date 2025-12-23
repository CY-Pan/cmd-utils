use clap::{Parser, ValueEnum};
use serde::Deserialize;
use std::{collections::HashMap, fs, process::Command};

#[derive(ValueEnum, Clone)]
enum Browser {
    Chrome,
    Edge,
}

#[derive(Parser)]
struct Args {
    #[arg(short, long, default_value = "chrome")]
    browser: Browser,
    url_file: String,
}

#[derive(Deserialize)]
struct URLs {
    #[serde(flatten)]
    data: HashMap<String, String>,
}

fn main() {
    let args = Args::parse();
    let content = fs::read_to_string(&args.url_file).unwrap();
    let urls: URLs = serde_json::from_str(&content).unwrap();

    let exe_path = if cfg!(target_os = "macos") {
        match args.browser {
            Browser::Chrome => "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
            Browser::Edge => "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
        }
    } else {
        match args.browser {
            Browser::Chrome => "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
            Browser::Edge => "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
        }
    };

    if urls.data.is_empty() {
        println!("No URL found in the file");
        return;
    }

    let mut cmd = Command::new(exe_path);
    for (_, v) in urls.data {
        cmd.arg(v);
    }
    cmd.output().unwrap();
}
