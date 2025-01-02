# File Deduplicator

A Rust command line tool to find and manage duplicate files using blake3 hashes.

## Features

- Fast blake3 content hashing
- Persistent caching of results
- File modification time tracking
- Interactive duplicate management
- Configurable duplicate file handling

## Installation

```sh
cargo install --path .
```

## Usage

```sh
# Recursive scan, without cache file and default ./duplicates folder
file-dedup -p /path/to/scan

# Non-recursive
file-dedup -p /path/to/scan --no-recursive

# With cache file
file-dedup -p /path/to/scan -c index.db

# Use existing cache without reindexing
file-dedup -p /path/to/scan -c index.db --reindex false

# Custom duplicates folder
file-dedup -p /path/to/scan -d /path/to/duplicates
```

## Options

- `-p, --path <path>`: Path to scan for duplicates
- `-c, --cache <cache>`: Path to cache file
- `-d, --duplicates <duplicates>`: Path to store duplicates
- `--reindex <reindex>`: Reindex files

## License

This software is licensed under MIT [License](LICENSE).
