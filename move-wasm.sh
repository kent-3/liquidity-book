#!/bin/bash
set -e

# Directories
TARGET_DIR="target/wasm32-unknown-unknown/release"
DEST_DIR="code"
CHECKSUM_DIR="${DEST_DIR}/checksum"

# Ensure necessary directories exist
mkdir -p "$DEST_DIR" "$CHECKSUM_DIR"

# Check if wasm-opt is installed
WASM_OPT_AVAILABLE=true
if ! command -v wasm-opt &> /dev/null; then
    echo "Warning: wasm-opt is not installed. Skipping optimization step."
    WASM_OPT_AVAILABLE=false
fi

# Move and process each .wasm file
for wasm_file in "$TARGET_DIR"/*.wasm; do
    # Extract the file name with extension
    wasm_filename=$(basename "$wasm_file")

    # Optimize and output to DEST_DIR, if wasm-opt is available
    if [ "$WASM_OPT_AVAILABLE" = true ]; then
        echo "Optimizing $wasm_filename with wasm-opt..."
        wasm-opt -Oz "$wasm_file" -o "$DEST_DIR/$wasm_filename"
    else
        echo "Skipping optimization for $wasm_filename"
        cp "$wasm_file" "$DEST_DIR/$wasm_filename"
    fi

    # Generate MD5 checksum and save to checksum file
    echo "Generating checksum for $wasm_filename..."
    md5sum "$DEST_DIR/$wasm_filename" | cut -f 1 -d " " > "${CHECKSUM_DIR}/${wasm_filename}.txt"

    # Compress the optimized or original .wasm file and save as .wasm.gz
    echo "Compressing $wasm_filename..."
    gzip -n -9 < "$DEST_DIR/$wasm_filename" > "${DEST_DIR}/${wasm_filename}.gz"

    # Remove the uncompressed .wasm file from the destination directory
    rm "$DEST_DIR/$wasm_filename"

    echo "$wasm_filename processed successfully."
done

echo "All wasm files have been processed, checksummed, and compressed."
