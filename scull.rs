// SPDX-License-Identifier: GPL-2.0

//! LDD3 chapter 3 scull module reimplemented in rust

#![no_std]
#![feature(allocator_api, global_asm, generic_associated_types)]

use kernel::prelude::*;
use kernel::{
    chrdev,
    file::{AccessMode, File},
    file_operations::{FileOpener, FileOperations, IoctlCommand, IoctlHandler, SeekFrom},
    io_buffer::{IoBufferReader, IoBufferWriter},
    mutex_init,
    sync::Mutex,
    user_ptr::{UserSlicePtrReader, UserSlicePtrWriter},
};

const NR_DEVS: usize = 4;

module! {
    type: Scull,
    name: b"scull",
    author: b"Daniel Xu",
    description: b"LDD3 chapter 3 scull module",
    license: b"GPL v2",
    params: {
        scull_quantum: i32 {
            default: 4000,
            permissions: 0o444,
            description: b"Size of scull quantum",
        },
        scull_qset: i32 {
            default: 1000,
            permissions: 0o444,
            description: b"Number of quantums per node",
        },
    },
}

struct ScullQuantum {
    data: Vec<Option<Vec<u8>>>,
}

struct ScullFileInner {
    data: Vec<ScullQuantum>,
    quantum: i32,
    qset: i32,
}

struct ScullFile {
    inner: Mutex<ScullFileInner>,
}

impl ScullFile {
    fn trim(&self) {
        let mut inner = self.inner.lock();
        inner.data = Vec::new();
        inner.quantum = *scull_quantum.read();
        inner.qset = *scull_qset.read();
    }
}

impl FileOpener<()> for ScullFile {
    fn open(_: &(), file: &File) -> Result<Pin<Box<Self>>> {
        // XXX: this needs to be stored globally, not per-fd
        let inner = ScullFileInner {
            data: Vec::new(),
            quantum: *scull_quantum.read(),
            qset: *scull_qset.read(),
        };

        // SAFETY: we will call mutex_init!() after this
        let mut scull_file = Pin::from(Box::try_new(ScullFile {
            inner: unsafe { Mutex::new(inner) },
        })?);
        // SAFETY: `inner` is pinned when `scull_file` is
        let pinned = unsafe { scull_file.as_mut().map_unchecked_mut(|s| &mut s.inner) };
        mutex_init!(pinned, "ScullFile::inner");

        if file.flags().access_mode() == AccessMode::WriteOnly {
            scull_file.trim();
        }

        Ok(scull_file)
    }
}

impl FileOperations for ScullFile {
    type Wrapper = Pin<Box<Self>>;
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
