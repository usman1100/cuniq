use clap::Parser;
use std::{
    collections::HashMap,
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

    let search_index = match columns.iter().position(|i| i == &"UNITPRICE") {
        Some(data) => data,
        None => panic!("UNITPRICE header not found"),
    };

    if columns.is_empty() {
        panic!("{}", "CSV file must contain headers")
    }

    let mut counter: HashMap<String, i128> = HashMap::new();

    for line in lines {
        let cells_string = match line {
            Ok(data) => data,
            Err(_err) => continue,
        };
        let cells_iter = cells_string.trim().split(",");
        let cells: Vec<String> = cells_iter.map(|i| i.to_string()).collect();

        let cell_value = match cells.get(search_index) {
            Some(data) => data,
            None => continue,
        };

        match counter.get(cell_value) {
            Some(data) => counter.insert(cell_value.to_string(), data + 1),
            None => counter.insert(cell_value.to_string(), 0),
        };
    }

    println!("{:?}", counter);
    println!("{}", args.function);
}
