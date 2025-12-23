use clap::Parser;
use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::{self, File},
    io::{BufWriter, Write},
};

#[derive(Parser)]
struct Args {
    host: String,
    task_file: String,
}

#[derive(Deserialize)]
struct Task {
    #[serde(flatten)]
    task: HashMap<String, Vec<String>>,
}

fn main() {
    let args = Args::parse();

    let content = fs::read_to_string(&args.task_file).unwrap();
    let task: Task = serde_json::from_str(&content).unwrap();

    let out_file = File::create("batch_scp.sh").unwrap();
    let mut out_writer = BufWriter::new(out_file);

    for (k, v) in task.task {
        let target = if k.ends_with('/') { k } else { format!("{k}/") };
        for file in v {
            let cmd = format!("scp -T {}:'{file}' {target}", args.host);
            out_writer.write_all(cmd.as_bytes()).unwrap();
            out_writer.write_all(b"\n").unwrap();
        }
    }
}
