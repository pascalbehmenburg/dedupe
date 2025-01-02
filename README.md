# Dedupe

A cli tool written in rust to find exactly identical files (by their file content) in a folder and optionally it's subfolders using blake3 hashes and multithreading.

## Features

- Fast blake3 content hashing
- Persistent caching of results to re-use / re-index based of last file modification
- Interactive duplicate management
- Configurable duplicate file handling

## Roadmap

- [ ] Add option to delete duplicates instead of moving them
  - [ ] Guess as to there being no ui users won't really like us selecting which of the duplicates to delete so maybe we need a terminal ui at some point
- [ ] Better document usage especially the caching as it is not really obvious yet.
- [ ] Filter logs to be more relevant to the actual program and not some thread infos
- [ ] Add better logs to the program itself
- [ ] Add an optional output log file

## Installation

Clone repo and navigate into it:
```sh
git clone https://github.com/pascalbehmenburg/dedupe
cd dedupe
```

Install dedupe executable using cargo:
```sh
cargo install --path .
```

## Example Usage

```sh
# Recursive scan, without cache file and default ./duplicates folder
dedupe -p /path/to/scan

# Non-recursive
dedupe -p /path/to/scan --no-recursive

# With cache file
dedupe -p /path/to/scan -c index.db

# Use existing cache without reindexing
dedupe -p /path/to/scan -c index.db --reindex false

# Custom duplicates folder
dedupe -p /path/to/scan -d /path/to/duplicates
```

## Options

Prefer to use the help command as it will be more up to date:
- `--help`: To show all options

Other options:
- `-p, --path <path>`: Path to scan for duplicates
- `-c, --cache <cache>`: Path to cache file
- `-d, --duplicates <duplicates>`: Path to store duplicates
- `--reindex <reindex>`: Reindex files
- `--no_recursive`: Whether to include subfolders
- `-v, --verbose`: Whether to verbosely print logs


## License

This software is licensed under MIT [License](LICENSE).
