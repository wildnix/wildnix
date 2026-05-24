#!/usr/bin/env bash
set -e

cd ../init-rs

cargo build --release --target x86_64-unknown-none

mkdir -p ../wildnix/build/userland
cp target/x86_64-unknown-none/release/init-rs ../wildnix/build/userland/init.elf

cd ../wildnix