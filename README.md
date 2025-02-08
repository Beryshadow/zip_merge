# Merge Arrays Algorithm with Duplicate Removal

## Overview

This Rust-based algorithm is designed to merge two arrays of objects while removing patterns of duplicates. It utilizes parallel processing via the Rayon library to efficiently handle large datasets. The algorithm first deduplicates the arrays by eliminating repeating patterns and then merges them, removing any further duplicate patterns that appear post-merge.

## Features

- **Deduplication**: The algorithm identifies and removes repeating patterns in arrays.
- **Merge Arrays**: It merges two arrays by identifying common subsequences and combining them in a meaningful way.
- **Parallel Processing**: Rayon is used to parallelize the deduplication process, improving performance on larger datasets.
- **File Handling**: Reads arrays from input files and writes the merged result to an output file.
- **Color Output**: Output is colored to visually differentiate the input and merged arrays.

## Example Usage
```bash
cargo run --release -- file1.txt file2.txt merged_output.txt
```
or
```bash
zip_merge file1.txt file2.txt merged_output.txt
```

## Prerequisites

- Rust 2021
