# cuniq

`cuniq` (from **c**olumn-**uniq**) is a simple, blazing-fast command-line utility for counting the occurrences of unique values in a specific CSV column.

It's a specialized replacement for common shell pipelines like `cut | sort | uniq -c`. `cuniq` is often much faster, finds the column by its **header name** (not its index), and is built in Rust for performance and memory safety.

## 🚀 Features

  * **Fast:** Written in Rust, it processes files much faster than many scripted equivalents.
  * **Header-Aware:** You specify the column by its name (e.g., `user_id`), not its position (e.g., `f-2`). This makes your scripts more robust to changes in the CSV structure.
  * **Simple:** Follows the GNU utility philosophy. It does one thing and does it well.
  * **Flexible:** Output to standard out (`stdout`) or to a new file.

## ⚙️ Usage

The basic command structure is:

```sh
cuniq --column <COLUMN_NAME> --input <INPUT_FILE> [OPTIONS]
```

### Examples

Let's say you have a file named `access_logs.csv`:

```csv
timestamp,ip_address,http_method,status_code,user_agent
2025-10-28,192.168.1.1,GET,200,"Mozilla..."
2025-10-28,10.0.0.5,GET,200,"Chrome..."
2025-10-28,192.168.1.1,POST,404,"Mozilla..."
2025-10-28,10.0.0.8,GET,200,"Chrome..."
2025-10-28,10.0.0.5,POST,201,"Chrome..."
2025-10-28,192.168.1.1,GET,200,"Mozilla..."
```

-----

#### 1\. Count values and print to terminal

To count the occurrences of each `status_code`:

```sh
$ cuniq --column status_code --input access_logs.csv

status_code,count
200,4
404,1
201,1
```

-----

#### 2\. Count values and save to a new file

To count the occurrences of each `ip_address` and save the result:

```sh
$ cuniq -c ip_address -i access_logs.csv -o ip_counts.csv
```

The new file `ip_counts.csv` will contain:

```csv
ip_address,count
192.168.1.1,3
10.0.0.5,2
10.0.0.8,1
```

-----

## CLI Options

```
Usage: cuniq --column <COLUMN_NAME> --input <FILE_PATH> [OPTIONS]

Options:
  -c, --column <COLUMN_NAME>
          Column name to count (case-sensitive)
  -i, --input <FILE_PATH>
          Path to the input CSV file
  -o, --output <OUTPUT_PATH>
          Path to output file (optional). If not provided, results are printed to stdout
  -h, --help
          Print help
  -V, --version
          Print version
```

## 📜 License

This project is licensed under the MIT License.
