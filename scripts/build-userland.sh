#!/usr/bin/env bash
set -e

cd userland/init-rs

cargo build --release --target x86_64-unknown-none

mkdir -p ../../build/userland
cp target/x86_64-unknown-none/release/init-rs ../../build/userland/init.elf