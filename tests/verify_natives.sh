#!/usr/bin/env sh

# Path to the directory where you want to start searching
if [ -d "./tests/QuantumLauncher/instances" ]; then
    root_dir="./tests/QuantumLauncher/instances"
elif [ -d "./QuantumLauncher/instances" ]; then
    root_dir="./QuantumLauncher/instances"
else
    echo "Error: Neither './tests/QuantumLauncher/instances' nor './QuantumLauncher/instances' exists. Exiting."
    exit 1
fi

for entry in $(ls -1S "$root_dir"); do
    entry="$root_dir/$entry"
    if [ -d "$entry" ]; then # if directory
        find "$entry/libraries/natives" -type f \( -iname "*.so" -o -iname "*.dylib" -o -iname "*.dll" \) | while read dylib; do
            file_output=$(file "$dylib")
            stripped_output=$(echo "$file_output" | sed "s|$root_dir/||")
            echo "$stripped_output"
        done
    fi
done
