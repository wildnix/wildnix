#!/usr/bin/env bash
set -e

KERNEL=target/x86_64-unknown-none/debug/wildnix
ISO_DIR=iso_root
ISO=wildnix-x86_64.iso
LIMINE_DIR="${LIMINE_DIR:-$HOME/limine-bin}"

if [ ! -d "$LIMINE_DIR" ]; then
    CWD=$(pwd)
    git clone https://github.com/limine-bootloader/limine.git \
        --branch=v8.x-binary --depth=1 "$LIMINE_DIR"
    cd "$LIMINE_DIR"
    make -j$(nproc)
    cd "$CWD"
fi

cargo build

mkdir -p "$ISO_DIR/boot/limine"
mkdir -p "$ISO_DIR/EFI/BOOT"

cp "$KERNEL"                          "$ISO_DIR/kernel"
cp limine.conf                        "$ISO_DIR/boot/limine/limine.conf"
cp "$LIMINE_DIR/limine-bios.sys"      "$ISO_DIR/boot/limine/"
cp "$LIMINE_DIR/limine-bios-cd.bin"   "$ISO_DIR/boot/limine/"
cp "$LIMINE_DIR/limine-uefi-cd.bin"   "$ISO_DIR/boot/limine/"
cp "$LIMINE_DIR/BOOTX64.EFI"          "$ISO_DIR/EFI/BOOT/"
cp "$LIMINE_DIR/BOOTIA32.EFI"         "$ISO_DIR/EFI/BOOT/"

xorriso -as mkisofs \
    -b boot/limine/limine-bios-cd.bin \
    -no-emul-boot -boot-load-size 4 -boot-info-table \
    --efi-boot boot/limine/limine-uefi-cd.bin \
    -efi-boot-part --efi-boot-image \
    --protective-msdos-label \
    "$ISO_DIR" -o "$ISO"

"$LIMINE_DIR/limine" bios-install "$ISO"

echo "ISO ready: $ISO"