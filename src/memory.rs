use crate::{Error, Result};
use core::ptr;

pub struct DmaBuffer {
    virt_addr: *mut u8,
    dma_addr: u64,
    size: usize,
}

impl DmaBuffer {
    pub fn new(virt_addr: *mut u8, dma_addr: u64, size: usize) -> Self {
        Self {
            virt_addr,
            dma_addr,
            size,
        }
    }

    pub fn dma_addr(&self) -> u64 {
        self.dma_addr
    }

    pub fn size(&self) -> usize {
        self.size
    }
    pub fn write<T: Copy>(&mut self, offset: usize, data: &T) -> Result<()> {
        if offset + core::mem::size_of::<T>() > self.size {
            return Err(Error::Invalid);
        }

        unsafe {
            let ptr = self.virt_addr.add(offset) as *mut T;
            ptr::write_volatile(ptr, *data);
        }

        Ok(())
    }

    pub fn read<T: Copy>(&self, offset: usize) -> Result<T> {
        if offset + core::mem::size_of::<T>() > self.size {
            return Err(Error::Invalid);
        }

        unsafe {
            let ptr = self.virt_addr.add(offset) as *const T;
            Ok(ptr::read_volatile(ptr))
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        unsafe { core::slice::from_raw_parts(self.virt_addr, self.size) }
    }
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        unsafe { core::slice::from_raw_parts_mut(self.virt_addr, self.size) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GemHandle(pub u32);
pub struct GemObject {
    handle: GemHandle,
    size: usize,
    dma_buffer: Option<DmaBuffer>,
}

impl GemObject {
    pub fn new(handle: GemHandle, size: usize) -> Self {
        Self {
            handle,
            size,
            dma_buffer: None,
        }
    }
    pub fn handle(&self) -> GemHandle {
        self.handle
    }

    pub fn size(&self) -> usize {
        self.size
    }
    pub fn attach_dma_buffer(&mut self, buffer: DmaBuffer) {
        self.dma_buffer = Some(buffer);
    }
    pub fn dma_buffer(&self) -> Option<&DmaBuffer> {
        self.dma_buffer.as_ref()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MemoryFlags(pub u32);

impl MemoryFlags {
    pub const COHERENT: u32 = 1 << 0;
    pub const CACHED: u32 = 1 << 1;
    pub const WRITE_COMBINED: u32 = 1 << 2;

    pub fn new(flags: u32) -> Self {
        Self(flags)
    }

    pub fn bits(&self) -> u32 {
        self.0
    }

    pub fn is_coherent(&self) -> bool {
        self.0 & Self::COHERENT != 0
    }

    pub fn is_cached(&self) -> bool {
        self.0 & Self::CACHED != 0
    }

    pub fn is_write_combined(&self) -> bool {
        self.0 & Self::WRITE_COMBINED != 0
    }
}
