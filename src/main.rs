use clap::Parser;
use memchr::{memchr, memchr2_iter};
use rayon::prelude::*;
use rustc_hash::{FxBuildHasher, FxHashMap};
use std::{
    fs::File,
    hash::BuildHasher,
    io::{self, BufWriter, Write},
    process::exit,
};

#[derive(Parser, Debug)]
#[command(
    version,
    about = "Counts the occurrences of values in a specific CSV column."
)]
struct Args {
    /// Name of the CSV column to count unique values for
    #[arg(
        short,
        long,
        value_name = "COLUMN_NAME",
        help = "Column name to count (case-sensitive)"
    )]
    column: String,

    /// Path to the input CSV file
    #[arg(
        short,
        long,
        value_name = "FILE_PATH",
        help = "Path to the input CSV file"
    )]
    input: String,

    /// Optional path to write the result as CSV
    ///
    /// If not provided, results are printed to stdout.
    #[arg(
        short,
        long,
        value_name = "OUTPUT_PATH",
        help = "Path to output file (optional)"
    )]
    output: Option<String>,
}

/// Number of hash partitions. Keys are split by hash so that merging the
/// per-thread results can also run in parallel.
const PARTS: usize = 64;

/// Keys borrow directly from the memory-mapped file, so no value is copied.
type Counts<'a> = FxHashMap<&'a [u8], u64>;

fn fail(msg: impl std::fmt::Display) -> ! {
    eprintln!("{}", msg);
    exit(1);
}

/// Counts the values of column `idx` in `data`, which must start at the
/// beginning of a line. Returns one map per hash partition.
fn count_chunk(data: &[u8], idx: usize) -> Vec<Counts<'_>> {
    let mut parts: Vec<Counts> = (0..PARTS).map(|_| Counts::default()).collect();
    let mut add = |key| {
        // Use the top 6 bits of the hash to pick a partition.
        let p = (FxBuildHasher.hash_one(key) >> 58) as usize;
        *parts[p].entry(key).or_insert(0) += 1;
    };

    let mut field = 0;
    let mut start = 0;
    let mut counted = false;
    // A single SIMD pass over every comma and newline, tracking which field we are in.
    for pos in memchr2_iter(b',', b'\n', data) {
        if !counted && field == idx {
            add(data[start..pos].trim_ascii());
            counted = true;
        }
        if data[pos] == b'\n' {
            field = 0;
            counted = false;
        } else {
            field += 1;
        }
        start = pos + 1;
    }
    // The last line may not end with a newline.
    if !counted && field == idx && start < data.len() {
        add(data[start..].trim_ascii());
    }
    parts
}

/// Splits `body` into about `n` chunks. Every chunk ends at a line boundary.
fn split_lines(body: &[u8], n: usize) -> Vec<&[u8]> {
    let target = body.len() / n + 1;
    let mut chunks = Vec::with_capacity(n);
    let mut start = 0;
    while start < body.len() {
        let mut end = (start + target).min(body.len());
        if end < body.len() {
            end = memchr(b'\n', &body[end..]).map_or(body.len(), |i| end + i + 1);
        }
        chunks.push(&body[start..end]);
        start = end;
    }
    chunks
}

/// Merges one partition's maps and formats it as CSV rows.
fn merge_and_format(maps: Vec<Counts>) -> Vec<u8> {
    let mut maps = maps.into_iter();
    let mut merged = maps.next().unwrap_or_default();
    for map in maps {
        for (key, count) in map {
            *merged.entry(key).or_insert(0) += count;
        }
    }

    let mut buf = Vec::with_capacity(merged.len() * 16);
    for (key, count) in merged {
        buf.extend_from_slice(key);
        buf.push(b',');
        buf.extend_from_slice(count.to_string().as_bytes());
        buf.push(b'\n');
    }
    buf
}

fn main() {
    let args = Args::parse();

    let file = File::open(&args.input).unwrap_or_else(|e| fail(format!("{}: {}", args.input, e)));
    let len = file.metadata().map(|m| m.len()).unwrap_or(0);
    if len == 0 {
        fail("Error reading file head: file is empty");
    }
    // SAFETY: the file must not be truncated by another process while we read it.
    let mmap = unsafe { memmap2::Mmap::map(&file) }.unwrap_or_else(|e| fail(e));
    let data: &[u8] = &mmap;

    let head_end = memchr(b'\n', data).unwrap_or(data.len());
    let search_index = data[..head_end]
        .trim_ascii()
        .split(|&b| b == b',')
        .position(|name| name == args.column.as_bytes())
        .unwrap_or_else(|| fail(format!("column {} not found", args.column)));
    let body = &data[(head_end + 1).min(data.len())..];

    // Phase 1: count each chunk in parallel.
    let chunks = split_lines(body, rayon::current_num_threads());
    let per_chunk: Vec<Vec<Counts>> = chunks
        .par_iter()
        .map(|chunk| count_chunk(chunk, search_index))
        .collect();

    // Group the maps by partition, then merge each partition in parallel.
    let mut by_part: Vec<Vec<Counts>> = (0..PARTS).map(|_| Vec::with_capacity(chunks.len())).collect();
    for parts in per_chunk {
        for (p, map) in parts.into_iter().enumerate() {
            by_part[p].push(map);
        }
    }
    let rows: Vec<Vec<u8>> = by_part.into_par_iter().map(merge_and_format).collect();

    let sink: Box<dyn Write> = match &args.output {
        Some(path) => Box::new(File::create(path).unwrap_or_else(|e| fail(format!("{}: {}", path, e)))),
        None => Box::new(io::stdout().lock()),
    };
    let mut out = BufWriter::with_capacity(1 << 16, sink);
    let result = (|| {
        writeln!(out, "{},count", args.column)?;
        for buf in &rows {
            out.write_all(buf)?;
        }
        out.flush()
    })();
    if let Err(e) = result {
        fail(format!("Error writing output. {}", e));
    }
}
