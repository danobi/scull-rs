// SPDX-License-Identifier: GPL-2.0

//! Rust out-of-tree sample

#![no_std]
#![feature(allocator_api, global_asm)]

use kernel::prelude::*;

module! {
    type: RustOutOfTree,
    name: b"rust_out_of_tree",
    author: b"Rust for Linux Contributors",
    description: b"Rust out-of-tree sample",
    license: b"GPL v2",
}

struct RustOutOfTree {
    message: String,
}

impl KernelModule for RustOutOfTree {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust out-of-tree sample (init)\n");

        Ok(RustOutOfTree {
            message: "on the heap!".try_to_owned()?,
        })
    }
}

impl Drop for RustOutOfTree {
    fn drop(&mut self) {
        pr_info!("My message is {}\n", self.message);
        pr_info!("Rust out-of-tree sample (exit)\n");
    }
}
