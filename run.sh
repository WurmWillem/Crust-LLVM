#!/usr/bin/env bash

set -e  # stop on error

# Step 1: run your compiler (adjust if needed)
cargo run

# Step 2: assemble
nasm -f elf64 out.asm -o out.o

# Step 3: link
ld out.o -o out

# Step 4: run
./out
