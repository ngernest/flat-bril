#!/bin/bash

# Create a hyperfine command with all benchmarks
CMD="hyperfine --warmup 3 --export-markdown json_roundtrip_bench.md"

for file in test/*.bril; do
  # Extract just the benchmark name (without path or extension)
  name=$(basename "$file" .bril)
  # Add to hyperfine command
  CMD+=" --command-name \"$name\" \"bril2json < $file | cargo run -- --json\""
done

# Execute the command
eval "$CMD"