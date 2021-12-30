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
