# cuniq

`cuniq` counts how many times each value appears in one column of a CSV file. You pick the column by its header name.

It does the same job as `cut -d, -fN file.csv | sort | uniq -c`, but you do not need to know the column number, and it is much faster on big files. It does not sort anything, and it uses all your CPU cores.

## Install

```sh
cargo install --git https://github.com/usman1100/cuniq
```

Prebuilt binaries for Linux, macOS and Windows are on the [releases page](https://github.com/usman1100/cuniq/releases/tag/latest).

## Usage

```sh
cuniq -c <COLUMN_NAME> -i <INPUT_FILE> [-o <OUTPUT_FILE>]
```

Say you have `access_logs.csv`:

```csv
timestamp,ip_address,http_method,status_code
2025-10-28,192.168.1.1,GET,200
2025-10-28,10.0.0.5,GET,200
2025-10-28,192.168.1.1,POST,404
2025-10-28,10.0.0.8,GET,200
2025-10-28,10.0.0.5,POST,201
2025-10-28,192.168.1.1,GET,200
```

Count the status codes:

```sh
$ cuniq -c status_code -i access_logs.csv
status_code,count
200,4
404,1
201,1
```

Write the result to a file instead of the terminal:

```sh
cuniq -c ip_address -i access_logs.csv -o ip_counts.csv
```

The output is itself a CSV, so you can feed it into other tools. The rows are not sorted. If you want them sorted by count, pipe the output through `sort`:

```sh
cuniq -c status_code -i access_logs.csv | tail -n +2 | sort -t, -k2 -nr
```

### Options

```
-c, --column <COLUMN_NAME>   Column to count (case-sensitive)
-i, --input <FILE_PATH>      Input CSV file
-o, --output <OUTPUT_PATH>   Output file. Prints to stdout if not given
-h, --help                   Print help
-V, --version                Print version
```

## Benchmarks

I tested on generated access-log CSVs with 7 columns. The 1M-row file is 55 MB, and the 10M-row file is 554 MB. Each number is the median of 5 runs on a 12-core M4 Pro MacBook, after one warm-up run, so the file was already in the OS cache.

I counted two columns. `status_code` has only 5 distinct values. `ip_address` is close to unique, with about 970k distinct values in the 1M file and 7.5M in the 10M file.

| File | Column | cuniq | `cut \| sort \| uniq -c` | same with `LC_ALL=C` | awk |
|---|---|---|---|---|---|
| 1M rows | status_code | **0.011s** | 2.03s | 1.75s | 0.96s |
| 1M rows | ip_address | **0.036s** | 4.35s | 2.29s | 1.60s |
| 10M rows | status_code | **0.061s** | 22.6s | 19.9s | 9.89s |
| 10M rows | ip_address | **0.40s** | 58.3s | 24.5s | 17.9s |

The awk command was `awk -F, 'NR>1{c[$N]++} END{for(k in c) print k","c[k]}'`, which uses the macOS system awk.

On the 10M-row file, `cuniq` reads the 554 MB in about 60 ms when the column has few distinct values. That is around 9 GB/s, so the speed is close to the limit of memory. On a cold cache, disk speed will be the limit instead.

### How it is fast

- The file is memory-mapped. Values are never copied, because the hash map keys point straight into the file.
- Commas and newlines are found with SIMD (`memchr`) in a single pass.
- The file is split into chunks at line boundaries, and every CPU core counts its own chunk.
- Keys are grouped by hash into 64 partitions. This lets the per-core results also be merged in parallel, which matters a lot when most values are unique.

Memory use depends mostly on how many distinct values there are. Counting `ip_address` in the 10M-row file peaked at about 1.1 GB, because it had to hold 7.5M keys. The reported memory also includes the mapped file pages. These are the OS file cache, which the OS can free when it needs to.

## Limitations

`cuniq` splits each line on commas. It does not understand quoted fields, so a value like `"Smith, John"` will be split in two and the counts after it will be wrong. If your file has quoted commas, use a full CSV tool such as [xsv](https://github.com/BurntSushi/xsv) or [qsv](https://github.com/dathere/qsv).

The input must be a regular file, because `cuniq` memory-maps it. It cannot read from a pipe or stdin. Do not let another program shorten the file while `cuniq` is reading it, because that can crash `cuniq`.

## License

MIT
