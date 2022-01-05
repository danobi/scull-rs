// SPDX-License-Identifier: GPL-2.0

//! LDD3 chapter 3 scull module reimplemented in rust

#![no_std]
#![feature(allocator_api, global_asm, generic_associated_types)]

use kernel::prelude::*;
use kernel::{
    file::{File, FileFlags},
    file_operations::{FileOperations, IoctlCommand, IoctlHandler, SeekFrom},
    io_buffer::{IoBufferReader, IoBufferWriter},
    miscdev, mutex_init,
    sync::{Mutex, Ref, UniqueRef},
    user_ptr::{UserSlicePtrReader, UserSlicePtrWriter},
};

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
    _data: Vec<Option<Vec<u8>>>,
}

struct ScullDevInner {
    data: Vec<ScullQuantum>,
    quantum: i32,
    qset: i32,
}

struct ScullDev {
    inner: Mutex<ScullDevInner>,
}

impl ScullDev {
    fn try_new() -> Result<Ref<Self>> {
        let inner = ScullDevInner {
            data: Vec::new(),
            quantum: *scull_quantum.read(),
            qset: *scull_qset.read(),
        };

        // SAFETY: we will call mutex_init!() after this
        let mut scull_dev = Pin::from(UniqueRef::try_new(ScullDev {
            inner: unsafe { Mutex::new(inner) },
        })?);

        // SAFETY: `inner` is pinned when `scull_dev` is
        let pinned = unsafe { scull_dev.as_mut().map_unchecked_mut(|s| &mut s.inner) };
        mutex_init!(pinned, "ScullDev::inner");

        Ok(scull_dev.into())
    }

    fn trim(&self) {
        let mut inner = self.inner.lock();
        inner.data = Vec::new();
        inner.quantum = *scull_quantum.read();
        inner.qset = *scull_qset.read();
    }
}

struct ScullFile {
    _dev: Ref<ScullDev>,
}

impl FileOperations for ScullFile {
    type OpenData = Ref<ScullDev>;
    type Wrapper = Box<Self>;
    kernel::declare_file_operations!(read, write, seek, ioctl);

    fn open(dev: &Ref<ScullDev>, file: &File) -> Result<Box<Self>> {
        if (file.flags() & FileFlags::O_ACCMODE) == FileFlags::O_WRONLY {
            dev.trim();
        }

        Ok(Box::try_new(ScullFile { _dev: dev.clone() })?)
    }

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
    _dev: Pin<Box<miscdev::Registration<ScullFile>>>,
}

impl KernelModule for Scull {
    fn init(name: &'static CStr, _: &'static ThisModule) -> Result<Self> {
        pr_info!("Rust scull init\n");

        let scull_dev = ScullDev::try_new()?;
        let scull = Scull {
            _dev: miscdev::Registration::new_pinned(name, None, scull_dev)?,
        };

        Ok(scull)
    }
}

impl Drop for Scull {
    fn drop(&mut self) {
        pr_info!("Rust scull exit\n");
    }
}
