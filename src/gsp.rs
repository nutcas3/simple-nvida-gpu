use crate::{memory::DmaBuffer, Error, Result};
use core::mem::size_of;

pub mod category {
    pub const JOB_SUBMIT: u8 = 0x01;
    pub const JOB_COMPLETE: u8 = 0x02;
    pub const MEMORY: u8 = 0x03;
    pub const MEMORY_RESPONSE: u8 = 0x04;
    pub const CONTEXT: u8 = 0x05;
    pub const SYNC: u8 = 0x06;
    pub const ERROR: u8 = 0xFF;
}
pub mod memory_op {
    pub const ALLOC: u8 = 0x01;
    pub const FREE: u8 = 0x02;
    pub const MAP: u8 = 0x03;
    pub const UNMAP: u8 = 0x04;
}

pub mod mailbox {
    pub const DOORBELL: u32 = 0x00;
    pub const STATUS: u32 = 0x04;
}

#[repr(u8)]
pub enum JobType {
    Graphics = 0,
    Compute = 1,
    Copy = 2,
    VideoDecode = 3,
    VideoEncode = 4,
}

#[repr(C)]
pub struct GspMessage {
    pub sequence: u32,
    pub unit_id: u8,
    pub size: u16,
    pub category: u8,
    pub status: u32,
}

impl GspMessage {
    pub fn new(sequence: u32, category: u8, size: u16) -> Self {
        Self {
            sequence,
            unit_id: 0,
            size,
            category,
            status: 0,
        }
    }
}
pub struct JobSubmitMessage {
    pub header: GspMessage,
    pub job_type: JobType,
    pub context_handle: u32,
    pub cmd_buffer_addr: u64,
    pub cmd_buffer_size: u32,
    pub fence_id: u64,
}
pub struct JobCompleteMessage {
    pub header: GspMessage,
    pub fence_id: u64,
    pub status: u32,
}
pub struct MemoryAllocMessage {
    pub header: GspMessage,
    pub op: u8,
    pub size: u64,
    pub flags: u32,
}
pub struct MemoryResponseMessage {
    pub header: GspMessage,
    pub handle: u32,
    pub gpu_addr: u64,
}
pub struct ErrorMessage {
    pub header: GspMessage,
    pub error_code: u32,
    pub error_info: u64,
}
pub enum GspResponse {
    JobComplete { fence_id: u64, status: u32 },
    MemoryAlloc { handle: u32, addr: u64 },
    Error { code: u32, info: u64 },
}
pub struct RingBuffer {
    buffer: DmaBuffer,
    read_pos: usize,
    write_pos: usize,
    capacity: usize,
}

impl RingBuffer {
    pub fn new(buffer: DmaBuffer) -> Self {
        let capacity = buffer.size();
        Self {
            buffer,
            read_pos: 0,
            write_pos: 0,
            capacity,
        }
    }

    pub fn write<T: Copy>(&mut self, data: &T) -> Result<()> {
        let size = size_of::<T>();
        if self.available_write() < size {
            return Err(Error::Busy);
        }

        self.buffer.write(self.write_pos, data)?;
        self.write_pos = (self.write_pos + size) % self.capacity;
        Ok(())
    }

    pub fn read<T: Copy>(&mut self) -> Result<Option<T>> {
        let size = size_of::<T>();
        if self.available_read() < size {
            return Ok(None);
        }

        let data = self.buffer.read::<T>(self.read_pos)?;
        self.read_pos = (self.read_pos + size) % self.capacity;
        Ok(Some(data))
    }

    fn available_write(&self) -> usize {
        if self.write_pos >= self.read_pos {
            self.capacity - (self.write_pos - self.read_pos) - 1
        } else {
            self.read_pos - self.write_pos - 1
        }
    }

    fn available_read(&self) -> usize {
        if self.write_pos >= self.read_pos {
            self.write_pos - self.read_pos
        } else {
            self.capacity - (self.read_pos - self.write_pos)
        }
    }
}

pub struct MailboxRegs {
    base_addr: *mut u8,
}

impl MailboxRegs {
    pub fn new(base_addr: *mut u8) -> Self {
        Self { base_addr }
    }

    pub fn write(&self, offset: u32, value: u32) -> Result<()> {
        unsafe {
            let ptr = self.base_addr.add(offset as usize) as *mut u32;
            core::ptr::write_volatile(ptr, value);
        }
        Ok(())
    }

    pub fn read(&self, offset: u32) -> Result<u32> {
        unsafe {
            let ptr = self.base_addr.add(offset as usize) as *const u32;
            Ok(core::ptr::read_volatile(ptr))
        }
    }
}


pub struct GspClient {
    queue: RingBuffer,
    mailbox: MailboxRegs,
    sequence: u32,
    context_handle: u32,
}

impl GspClient {
    pub fn new(queue_buffer: DmaBuffer, mailbox_addr: *mut u8) -> Self {
        Self {
            queue: RingBuffer::new(queue_buffer),
            mailbox: MailboxRegs::new(mailbox_addr),
            sequence: 0,
            context_handle: 0,
        }
    }

    fn next_sequence(&mut self) -> u32 {
        let seq = self.sequence;
        self.sequence = self.sequence.wrapping_add(1);
        seq
    }

    fn context_handle(&self) -> u32 {
        self.context_handle
    }
    pub fn set_context_handle(&mut self, handle: u32) {
        self.context_handle = handle;
    }

    pub fn submit_job(
        &mut self,
        job_type: JobType,
        cmd_buffer: &DmaBuffer,
        fence_id: u64,
    ) -> Result<()> {
        let msg = JobSubmitMessage {
            header: GspMessage::new(
                self.next_sequence(),
                category::JOB_SUBMIT,
                size_of::<JobSubmitMessage>() as u16,
            ),
            job_type,
            context_handle: self.context_handle(),
            cmd_buffer_addr: cmd_buffer.dma_addr(),
            cmd_buffer_size: cmd_buffer.size() as u32,
            fence_id,
        };

        self.queue.write(&msg)?;
        self.mailbox.write(mailbox::DOORBELL, 1)?;

        Ok(())
    }

    pub fn alloc_memory(&mut self, size: usize, flags: u32) -> Result<u32> {
        let msg = MemoryAllocMessage {
            header: GspMessage::new(
                self.next_sequence(),
                category::MEMORY,
                size_of::<MemoryAllocMessage>() as u16,
            ),
            op: memory_op::ALLOC,
            size: size as u64,
            flags,
        };

        self.queue.write(&msg)?;
        self.mailbox.write(mailbox::DOORBELL, 1)?;

        // TODO would wait for response
        Ok(0) // Placeholder handle
    }

    pub fn poll_responses(&mut self) -> Result<Vec<GspResponse>> {
        let mut responses = Vec::new();

        while let Some(header) = self.queue.read::<GspMessage>()? {
            let response = match header.category {
                category::JOB_COMPLETE => {
                    if let Some(job_msg) = self.queue.read::<JobCompleteMessage>()? {
                        GspResponse::JobComplete {
                            fence_id: job_msg.fence_id,
                            status: job_msg.status,
                        }
                    } else {
                        continue;
                    }
                }

                category::MEMORY_RESPONSE => {
                    if let Some(mem_msg) = self.queue.read::<MemoryResponseMessage>()? {
                        GspResponse::MemoryAlloc {
                            handle: mem_msg.handle,
                            addr: mem_msg.gpu_addr,
                        }
                    } else {
                        continue;
                    }
                }

                category::ERROR => {
                    if let Some(err_msg) = self.queue.read::<ErrorMessage>()? {
                        GspResponse::Error {
                            code: err_msg.error_code,
                            info: err_msg.error_info,
                        }
                    } else {
                        continue;
                    }
                }

                _ => continue,
            };

            responses.push(response);
        }

        Ok(responses)
    }
}
