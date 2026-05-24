#!/usr/bin/env bash
set -e

qemu-system-x86_64 \
    -M q35 \
    -m 256M \
    -cdrom wildnix-x86_64.iso \
    -serial stdio \
    -no-reboot \
    -no-shutdown \
    -vga std