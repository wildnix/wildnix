$ErrorActionPreference = "Stop"

Push-Location userland/init-rs

cargo build --release --target x86_64-unknown-none

Pop-Location

New-Item -ItemType Directory -Force -Path build/userland | Out-Null

Copy-Item `
    "userland/init-rs/target/x86_64-unknown-none/release/init-rs" `
    "build/userland/init.elf" `
    -Force

Write-Host "Userland build complete."