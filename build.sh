#!/bin/bash

set -e  # Exit immediately if a command fails

# Step 1: Compile Rust project with Cargo
echo "Building Rust project with cargo..."
cargo build

FILE="./syntax/operation.l"

# Step 2: Run the compiled Rust binary
RUST_BINARY="./target/debug/lex"
if [ -f "$RUST_BINARY" ]; then
    echo "Running Rust binary..."
    "$RUST_BINARY" $FILE
else
    echo "Rust binary not found at $RUST_BINARY"
    exit 1
fi

# Step 3: Compile lex.yy.c if it exists
if [ -f lex.yy.c ]; then
    echo "Compiling lex.yy.c with cc..."
    cc lex.yy.c -o lex

    # Step 4: Run the compiled lex executable
    echo "Running lex executable..."
    ./lex
else
    echo "lex.yy.c not found. Skipping C compilation."
fi

