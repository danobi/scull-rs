// SPDX-License-Identifier: GPL-2.0

//! LDD3 chapter 3 scull module reimplemented in rust

#![no_std]
#![feature(allocator_api, global_asm)]

use kernel::prelude::*;
use kernel::{chrdev, file_operations::FileOperations};

const NR_DEVS: usize = 4;

module! {
    type: Scull,
    name: b"scull",
    author: b"Daniel Xu",
    description: b"LDD3 chapter 3 scull module",
    license: b"GPL v2",
}

#[derive(Default)]
struct RustFile;

impl FileOperations for RustFile {
    kernel::declare_file_operations!();
}

struct Scull {
    _dev: Pin<Box<chrdev::Registration<NR_DEVS>>>,
}

impl KernelModule for Scull {
    fn init(name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust scull init\n");

        let mut chrdev_reg = chrdev::Registration::new_pinned(name, 0, module)?;
        for _ in 0..NR_DEVS {
            chrdev_reg.as_mut().register::<RustFile>()?;
        }

        Ok(Scull { _dev: chrdev_reg })
    }
}

impl Drop for Scull {
    fn drop(&mut self) {
        pr_info!("Rust scull exit\n");
    }
}
