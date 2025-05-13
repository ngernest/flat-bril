#!/bin/bash

# Loop through all .bril files in the test directory
for bril_file in test/*.bril; do
    # Check if any files were found
    if [ ! -f "$bril_file" ]; then
        echo "No .bril files found in test directory"
        exit 1
    fi
    
    # Extract the base filename without extension
    base_name=$(basename "$bril_file" .bril)
    
    # Define the output .out file
    out_file="test/${base_name}.out"
    
    # Run the conversion command
    bril2json < "$bril_file" | brili > "$out_file"
    
    # Check if conversion was successful
    if [ $? -eq 0 ]; then
        echo "Successfully converted $bril_file to $out_file"
    else
        echo "Error converting $bril_file"
    fi
done

echo "All .bril files have been converted to .json"


