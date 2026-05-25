#!/usr/bin/env bash
set -e

qemu-system-x86_64 \
    -M q35 \
    -m 256M \
    -cdrom wildnix-x86_64.iso \
    -serial stdio \
    -usb \
    -device usb-kbd \
    -no-reboot \
    -no-shutdown \
    -vga std