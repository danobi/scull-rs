// SPDX-License-Identifier: GPL-2.0

//! LDD3 chapter 3 scull module reimplemented in rust

#![no_std]
#![feature(allocator_api, global_asm)]

use kernel::prelude::*;

module! {
    type: Scull,
    name: b"scull",
    author: b"Daniel Xu",
    description: b"LDD3 chapter 3 scull module",
    license: b"GPL v2",
}

struct Scull {
    message: String,
}

impl KernelModule for Scull {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust scull init\n");

        Ok(Scull {
            message: "on the heap!".try_to_owned()?,
        })
    }
}

impl Drop for Scull {
    fn drop(&mut self) {
        pr_info!("My message is {}\n", self.message);
        pr_info!("Rust scull exit\n");
    }
}
