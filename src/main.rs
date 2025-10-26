use clap::Parser;
use std::{
    fs::File,
    io::{BufRead, BufReader},
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    function: String,
}

fn main() {
    let args = Args::parse();
    let file = match File::open("./data.csv") {
        Ok(data) => data,
        Err(err) => panic!("{}", err),
    };
    let reader = BufReader::new(file);

    let mut lines = reader.lines();

    let head = match lines.next() {
        Some(data) => data,
        None => panic!("{}", "Error reading file head"),
    };

    let column_string = match head {
        Ok(data) => data,
        Err(err) => panic!("{}", err),
    };

    let columns_iter = column_string.trim().split(",");

    let columns: Vec<String> = columns_iter.map(|s| s.to_string()).collect();

    if columns.is_empty() {
        panic!("{}", "CSV file must contain headers")
    }

    println!("{:?}", columns);

    println!("{}", args.function);
}
