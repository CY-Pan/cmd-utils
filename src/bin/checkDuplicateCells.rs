use clap::Parser;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Parser)]
struct Args {
    path: Vec<String>,
}

fn main() {
    let mut cell_set = std::collections::HashSet::<String>::new();
    let args = Args::parse_from(wild::args());

    for f in &args.path {
        let reader = BufReader::new(File::open(f).unwrap());
        for line in reader.lines() {
            let line = line.unwrap().trim().to_string();
            if !line.ends_with("\\\\") {
                continue;
            }
            let columns: Vec<&str> = line
                .split('&')
                .map(|s| s.trim_end_matches('\\').trim())
                .collect();
            if columns.len() < 2 {
                continue;
            }
            columns.into_iter().for_each(|cell| {
                if !cell_set.is_empty()
                    && !cell.starts_with("\\textbf")
                    && !cell_set.insert(cell.to_string())
                {
                    panic!("Duplicate cell found: [{}], in line [{}]", cell, line);
                }
            });
        }
    }
    println!("No duplicate cells found.");
}
