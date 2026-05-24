cargo build
wsl -d ubuntu -- bash -c "./scripts/make-iso.sh"
qemu-system-x86_64 -cdrom wildnix-x86_64.iso -m 256M -serial stdio -no-reboot -no-shutdown -vga std