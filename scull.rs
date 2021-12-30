// SPDX-License-Identifier: GPL-2.0

//! LDD3 chapter 3 scull module reimplemented in rust

#![no_std]
#![feature(allocator_api, global_asm, generic_associated_types)]

use kernel::prelude::*;
use kernel::{
    chrdev,
    file::File,
    file_operations::{FileOpener, FileOperations, IoctlCommand, IoctlHandler, SeekFrom},
    io_buffer::{IoBufferReader, IoBufferWriter},
    user_ptr::{UserSlicePtrReader, UserSlicePtrWriter},
};

const NR_DEVS: usize = 4;

module! {
    type: Scull,
    name: b"scull",
    author: b"Daniel Xu",
    description: b"LDD3 chapter 3 scull module",
    license: b"GPL v2",
}

#[derive(Default)]
struct ScullFile;

// Use a ZST to specialize the FileOpener cuz we want to implement a custom open()
struct ScullFileTag;
impl FileOpener<ScullFileTag> for ScullFile {
    fn open(_: &ScullFileTag, _file: &File) -> Result<Box<Self>> {
        Ok(Box::try_new(Self::default())?)
    }
}

impl FileOperations for ScullFile {
    kernel::declare_file_operations!(read, write, seek, ioctl);

    fn read(
        _this: &Self,
        _file: &File,
        _data: &mut impl IoBufferWriter,
        _offset: u64,
    ) -> Result<usize> {
        Err(Error::ENOTSUPP)
    }

    fn write(
        _this: &Self,
        _file: &File,
        _data: &mut impl IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {
        Err(Error::ENOTSUPP)
    }

    fn seek(_this: &Self, _file: &File, _offset: SeekFrom) -> Result<u64> {
        Err(Error::ENOTSUPP)
    }

    fn ioctl(this: &Self, file: &File, cmd: &mut IoctlCommand) -> Result<i32> {
        cmd.dispatch::<Self>(this, file)
    }
}

impl IoctlHandler for ScullFile {
    type Target<'a> = &'a Self;

    fn read(_this: &Self, _: &File, cmd: u32, _writer: &mut UserSlicePtrWriter) -> Result<i32> {
        match cmd {
            _ => Err(Error::ENOTSUPP),
        }
    }

    fn write(_this: &Self, _: &File, cmd: u32, _reader: &mut UserSlicePtrReader) -> Result<i32> {
        match cmd {
            _ => Err(Error::ENOTSUPP),
        }
    }
}

struct Scull {
    _dev: Pin<Box<chrdev::Registration<NR_DEVS>>>,
}

impl KernelModule for Scull {
    fn init(name: &'static CStr, module: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust scull init\n");

        let mut chrdev_reg = chrdev::Registration::new_pinned(name, 0, module)?;
        for _ in 0..NR_DEVS {
            chrdev_reg.as_mut().register::<ScullFile>()?;
        }

        Ok(Scull { _dev: chrdev_reg })
    }
}

impl Drop for Scull {
    fn drop(&mut self) {
        pr_info!("Rust scull exit\n");
    }
}
