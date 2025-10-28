use clap::Parser;
use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Write},
    process::exit,
};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    column: String,

    #[arg(short, long)]
    input: String,

    #[arg(short, long)]
    output: Option<String>,
}

fn print_counter(counter: HashMap<String, u32>) -> () {
    for (key, value) in counter {
        println!("{},{}", key, value)
    }
}

fn write_count_to_output(path: String, counter: HashMap<String, u32>, column: String) -> () {
    let mut file = match File::create(path) {
        Ok(data) => data,
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    };

    match file.write(format!("{},count\n", column).as_bytes()) {
        Ok(_) => (),
        Err(e) => {
            eprintln!("Error printing line. {}", e);
            exit(1);
        }
    }

    for (key, value) in counter {
        match file.write(format!("{},{}\n", key, value).as_bytes()) {
            Ok(_) => continue,
            Err(e) => eprintln!("Error printing line. {}", e),
        }
    }
}

fn main() {
    let args = Args::parse();
    let column = args.column;
    let file = match File::open(args.input) {
        Ok(data) => data,
        Err(err) => panic!("{}", err),
    };
    let reader = BufReader::with_capacity(1024 * 1024, file);
    let mut lines = reader.lines();

    let head = match lines.next() {
        Some(data) => data,
        None => panic!("{}", "Error reading file head"),
    };

    let column_string = match head {
        Ok(data) => data,
        Err(err) => {
            eprintln!("{}", err);
            exit(1);
        }
    };

    let columns_iter = column_string.trim().split(",");
    let columns: Vec<String> = columns_iter.map(|s| s.to_string()).collect();

    let search_index = match columns.iter().position(|i| i == &column) {
        Some(data) => data,
        None => {
            eprintln!("column {} not found", column);
            exit(1);
        }
    };

    if columns.is_empty() {
        panic!("{}", "CSV file must contain headers")
    }

    let mut counter: HashMap<String, u32> = HashMap::new();

    for line in lines {
        let cells_string = match line {
            Ok(data) => data,
            Err(_err) => continue,
        };

        if let Some(cell_value) = cells_string.split(",").nth(search_index) {
            let cell_value = cell_value.trim();

            counter
                .entry(cell_value.to_string())
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
    }

    match args.output {
        Some(data) => write_count_to_output(data, counter, column),
        None => {
            println!("{},count", column);
            print_counter(counter);
        }
    }
}
