# Duplicate Finder

Duplicate Finder is a Rust utility that scans a directory for identical files, groups them together, and stores each set in clearly labeled folders while producing a single index for reference.

## Features

- Dialog prompt to scan the current directory or enter a custom path
- Recursive file discovery with metadata validation
- SHA-256 hashing to ensure accurate duplicate detection
- `duplicates/<hash>_<original_filename>` output folders for each duplicate group
- `duplicate_files_index.txt` summary listing every duplicate file and hash

## How It Works

1. Recursively scans all files under the selected directory.
2. Computes SHA-256 hashes and groups files that share the same hash.
3. Moves duplicate sets into dedicated folders and writes a comprehensive index at the directory root.
