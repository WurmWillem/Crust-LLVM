#!/usr/bin/env bash

set -e  # stop on error

nasm -f elf64 out.asm -o out.o

ld out.o -o out

./out
