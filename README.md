# WildNIX Kernel

## Clone:
```bash
git clone --recurse-submodules https://github.com/wildnix/wildnix
```

## Milestones

- [x] `24.05.2026` - The white rectangle of hope
- [x] `24.05.2026` - PSF2 Font parsing
- [x] `24.05.2026` - GDT/IDT
- [x] `24.05.2026` - PMM
- [x] `25.05.2026` - Ring 3
- [x] `25.05.2026` - First syscall

## Running

1. Set up a workspace like this:
```bash
➜  wildnix ls
init-rs  libwildnix  libwildnix-macros  wildnix
➜  wildnix 
```
| Repository | URL |
|------------|-----|
| wildnix    | [wildnix/wildnix](https://github.com/wildnix/wildnix) |
| libwildnix | [wildnix/libwildnix](https://github.com/wildnix/libwildnix) |
| init-rs    | [wildnix/init-rs](https://github.com/wildnix/init-rs) |
2. Run `./scripts/make-iso.sh`
3. Run `./scripts/run.sh`
4. Enjoy or something

## Special thanks to:
- [osdev.wiki](https://osdev.wiki/wiki/Expanded_Main_Page)