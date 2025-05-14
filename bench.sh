#!/bin/bash

# Create a hyperfine command with all benchmarks
CMD="hyperfine -w3 --shell=none --show-output --export-markdown json_roundtrip_bench.md"

for file in test/*.json; do
  # Extract just the benchmark name (without path or extension)
  name=$(basename "$file" .json)
  # Add to hyperfine command
  CMD+=" --command-name \"$name\" \"./target/release/flat-bril --json --filename $file\""
done

# Execute the command
eval "$CMD"