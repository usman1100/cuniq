# cuniq

`cuniq` counts how many times each value appears in one column of a CSV file. You pick the column by its header name.

It does the same job as `cut -d, -fN file.csv | sort | uniq -c`, but you do not need to know the column number, and it is faster on big files because it does not sort anything.

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

I tested on generated access-log CSVs with 7 columns. The 1M-row file is 55 MB, and the 10M-row file is 554 MB. Each number is the median of 5 runs on an M4 Pro MacBook, after one warm-up run, so the file was already in the OS cache.

I counted two columns. `status_code` has only 5 distinct values. `ip_address` is close to unique, with about 970k distinct values in the 1M file and 7.5M in the 10M file.

| File | Column | cuniq | `cut \| sort \| uniq -c` | same with `LC_ALL=C` | awk |
|---|---|---|---|---|---|
| 1M rows | status_code | **0.11s** | 2.10s | 1.56s | 0.97s |
| 1M rows | ip_address | **0.91s** | 4.30s | 2.35s | 1.68s |
| 10M rows | status_code | **1.07s** | 21.5s | 21.9s | 9.56s |
| 10M rows | ip_address | **8.46s** | 57.2s | 25.8s | 17.5s |

The awk command was `awk -F, 'NR>1{c[$N]++} END{for(k in c) print k","c[k]}'`, which uses the macOS system awk.

The gain is largest when the column has few distinct values. In that case `cuniq` is about 20x faster than the shell pipeline and about 9x faster than awk. When almost every value is unique, the hash map grows large, so the gain drops to about 3x over `LC_ALL=C sort` and 2x over awk.

Memory use depends on how many distinct values there are, not on the file size. Counting `status_code` in the 10M-row file used about 3 MB. Counting `ip_address` in the same file used about 1.2 GB, because it had to hold 7.5M keys.

## Limitations

`cuniq` splits each line on commas. It does not understand quoted fields, so a value like `"Smith, John"` will be split in two and the counts after it will be wrong. If your file has quoted commas, use a full CSV tool such as [xsv](https://github.com/BurntSushi/xsv) or [qsv](https://github.com/dathere/qsv).

## License

MIT
