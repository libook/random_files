# Random Files

Respond random files in a directory.
- Create a directory named `files`.
- Put some files in it.
- Run ./random_files
- Access `127.0.0.1:3030`
- Get a random file.

## Features
- Responds with a random file from a directory.
- Support subdirectories. e.g. `http://127.0.0.1:3030/subdir1`
- Use boolean query `refresh_cache` to refresh the file list cache of a directory. e.g. `http://127.0.0.1:3030/subdir1?refresh_cache=true`
- Use environment variable `RUST_LOG` to set log level. e.g. `RUST_LOG=info ./random_files`

## Installation

### Rust

Make sure you have Rust toolchains installed.

Clone code and `cd` to the project directory.

Run directly:
```bash
cargo run
# or for logging
RUST_LOG=info cargo run
```
Compile and run:
```bash
cargo build --release
RUST_LOG=info ./target/release/random_files
```

### Docker

Make sure you have Docker installed.

Clone code and `cd` to the project directory.

Run `./build.sh`

Create container:
```bash
docker run -d -p 3030:3030 -v $(pwd):/files --name random_files libook/random_files:latest
# or for subdirectories
docker run -d -p 3030:3030 -v $(pwd):/files/subdir1 --name random_files libook/random_files:latest
```
