#!/bin/bash

set -e

nasm -f elf64 userland/init.asm -o userland/init.o
ld -nostdlib -static -Ttext=0x400000 -o userland/init.elf userland/init.o