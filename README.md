# SdFat-rs

A minimal, async port of [SdFat](https://github.com/greiman/SdFat) for Rust.
The initial target is to support FAT32.

## Instructions to build

This project requires a custom build of rustc/LLVM with [this patch](https://github.com/drmorr0/sdfat32-rs/blob/master/patch2.diff) applied.  You can follow the instructions [here](https://objectdisoriented.evokewonder.com/posts/patching-llvm/) to build your custom version of rustc/LLVM.

Build with `cargo build --release`.  You'll need some way to flash your .elf file, either ravedude or ATMEL studio.
