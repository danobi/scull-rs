// SPDX-License-Identifier: GPL-2.0

//! LDD3 chapter 3 scull module reimplemented in rust

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

struct ScullQset {
    data: Vec<Vec<u8>>,
}

struct ScullDevInner {
    data: Vec<ScullQset>,
    quantum: i32,
    qset: i32,
    size: u64,
}

impl ScullDevInner {
    /// Given the scull set index, return the scull set.
    ///
    /// Will allocate and initialize any missing scull sets (but not the quantums).
    fn follow(&mut self, n: usize) -> Result<&mut ScullQset> {
        // Resize self to have at least as many elements as the requested index
        //
        // Note we cannot use `Vec::try_resize()` because `Vec` does not implement
        // `Clone` with `cfg(no_global_oom_handling)`
        if self.data.len() < (n + 1) {
            for _ in self.data.len()..(n + 1) {
                let mut qset: Vec<Vec<u8>> = Vec::try_with_capacity(self.qset.try_into()?)?;
                for _ in 0..self.qset {
                    qset.try_push(Vec::new())?;
                }

                self.data.try_push(ScullQset { data: qset })?;
            }
        }

        Ok(&mut self.data[n])
    }
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
            size: 0,
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

    fn read(&self, data: &mut impl IoBufferWriter, offset: u64) -> Result<usize> {
        let mut inner = self.inner.lock();

        if offset > inner.size {
            return Ok(0);
        }

        // Do not read past end of device
        let mut to_write = data.len() as u64;
        if (offset + to_write) > inner.size {
            to_write = inner.size - offset;
        }

        // Calculate offets into nested data structure
        let qset_size: u64 = inner.qset.try_into()?;
        let quantum_size: u64 = inner.quantum.try_into()?;
        let itemsize: u64 = quantum_size * qset_size;
        let item: u64 = offset / itemsize;
        let rest: u64 = offset % itemsize;
        let s_pos: u64 = rest / quantum_size;
        let q_pos: u64 = rest % quantum_size;

        // Find quantum to read from
        let qset: &mut ScullQset = inner.follow(item as usize)?;
        let quantum: &mut Vec<u8> = &mut qset.data[s_pos as usize];
        if quantum.is_empty() {
            // Do not fill any holes, unlike write() path
            return Ok(0);
        }

        // Cap reads to a single quantum
        if to_write > (quantum_size - q_pos) {
            to_write = quantum_size - q_pos;
        }

        // Write data for user
        let slice_start: usize = q_pos as usize;
        let slice_end: usize = (q_pos + to_write) as usize;
        let dest = &mut quantum[slice_start..slice_end];
        data.write_slice(dest)?;

        Ok(to_write as usize)
    }

    fn write(&self, data: &mut impl IoBufferReader, offset: u64) -> Result<usize> {
        let mut inner = self.inner.lock();

        // Calculate offets into nested data structure
        let qset_size: u64 = inner.qset.try_into()?;
        let quantum_size: u64 = inner.quantum.try_into()?;
        let itemsize: u64 = quantum_size * qset_size;
        let item: u64 = offset / itemsize;
        let rest: u64 = offset % itemsize;
        let s_pos: u64 = rest / quantum_size;
        let q_pos: u64 = rest % quantum_size;

        // Cap writes to a single quantum
        let mut to_read = data.len() as u64;
        if to_read > (quantum_size - q_pos) {
            to_read = quantum_size - q_pos;
        }

        // Find quantum to write to
        let qset: &mut ScullQset = inner.follow(item as usize)?;
        let quantum: &mut Vec<u8> = &mut qset.data[s_pos as usize];
        if quantum.is_empty() {
            quantum.try_resize(quantum_size.try_into()?, 0)?;
        }

        // Read user data in
        let slice_start: usize = q_pos as usize;
        let slice_end: usize = (q_pos + to_read) as usize;
        let dest = &mut quantum[slice_start..slice_end];
        data.read_slice(dest)?;

        // Update accounting
        let new_offset = offset + to_read;
        if inner.size < new_offset {
            inner.size = new_offset;
        }

        // Return number of bytes read
        Ok(to_read as usize)
    }
}

struct ScullFile {
    dev: Ref<ScullDev>,
}

impl FileOperations for ScullFile {
    type OpenData = Ref<ScullDev>;
    type Wrapper = Box<Self>;
    kernel::declare_file_operations!(read, write, seek, ioctl);

    fn open(dev: &Ref<ScullDev>, file: &File) -> Result<Box<Self>> {
        if (file.flags() & FileFlags::O_ACCMODE) == FileFlags::O_WRONLY {
            dev.trim();
        }

        Ok(Box::try_new(ScullFile { dev: dev.clone() })?)
    }

    fn read(
        this: &Self,
        _file: &File,
        data: &mut impl IoBufferWriter,
        offset: u64,
    ) -> Result<usize> {
        this.dev.read(data, offset)
    }

    fn write(
        this: &Self,
        _file: &File,
        data: &mut impl IoBufferReader,
        offset: u64,
    ) -> Result<usize> {
        this.dev.write(data, offset)
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
