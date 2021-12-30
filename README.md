# scull-rs

This repository implements Linux Device Drivers 3rd edition's scull driver from
chapter 3 as an out of tree rust kernel module.

### Requirements

See README from https://github.com/Rust-for-Linux/rust-out-of-tree-module on
requirements.

### Build

```sh
$ make KDIR=../linux LLVM=1
```

Example:

```sh
$ make KDIR=.../linux-with-rust-support LLVM=1
make -C ../linux M=$PWD
  RUSTC [M] /home/dxu/dev/scull-rs/rust_out_of_tree.o
  MODPOST /home/dxu/dev/scull-rs/Module.symvers
  CC [M]  /home/dxu/dev/scull-rs/rust_out_of_tree.mod.o
  LD [M]  /home/dxu/dev/scull-rs/rust_out_of_tree.ko
  BTF [M] /home/dxu/dev/scull-rs/rust_out_of_tree.ko
Skipping BTF generation for /home/dxu/dev/scull-rs/rust_out_of_tree.ko because it's a Rust module

$ sudo insmod ./scull.ko
```
