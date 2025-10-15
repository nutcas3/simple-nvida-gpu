use crate::{memory::{GemHandle, GemObject}, Error, Result};

pub mod ioctl {
    pub const GETPARAM: u32 = 0x00;
    pub const GEM_CREATE: u32 = 0x01;
    pub const GEM_INFO: u32 = 0x02;
    pub const GEM_MMAP: u32 = 0x03;
}

pub mod param {
    pub const CHIPSET_ID: u32 = 0x00;
    pub const VRAM_SIZE: u32 = 0x01;
    pub const DRIVER_VERSION: u32 = 0x02;
    pub const FIRMWARE_VERSION: u32 = 0x03;
}

pub mod flags {
    pub const RENDER_ALLOW: u32 = 1 << 0;
    pub const AUTH: u32 = 1 << 1;
    pub const UNLOCKED: u32 = 1 << 2;
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GetParamRequest {
    pub param: u32,
    pub value: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GemCreateRequest {
    pub size: u64,
    pub handle: u32,
    pub flags: u32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GemInfoRequest {
    pub handle: u32,
    pub size: u64,
    pub offset: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct GemMmapRequest {
    pub handle: u32,
    pub offset: u64,
    pub size: u64,
    pub addr: u64,
}

pub struct DrmFile {
    pub fd: u32,
    pub authenticated: bool,
}

impl DrmFile {
    pub fn new(fd: u32) -> Self {
        Self {
            fd,
            authenticated: false,
        }
    }

    pub fn authenticate(&mut self) {
        self.authenticated = true;
    }

    pub fn is_authenticated(&self) -> bool {
        self.authenticated
    }
}

pub struct NovaDrm {
    device_id: u16,
    vram_size: u64,
    driver_version: u32,
    firmware_version: u32,
    gem_objects: Vec<GemObject>,
    next_gem_handle: u32,
}

impl NovaDrm {
    pub fn new(device_id: u16, vram_size: u64, firmware_version: u32) -> Self {
        Self {
            device_id,
            vram_size,
            driver_version: 1,
            firmware_version,
            gem_objects: Vec::new(),
            next_gem_handle: 1,
        }
    }

    pub fn handle_getparam(&self, req: &mut GetParamRequest) -> Result<()> {
        req.value = match req.param {
            param::CHIPSET_ID => self.device_id as u64,
            param::VRAM_SIZE => self.vram_size,
            param::DRIVER_VERSION => self.driver_version as u64,
            param::FIRMWARE_VERSION => self.firmware_version as u64,
            _ => return Err(Error::Invalid),
        };

        Ok(())
    }

    pub fn handle_gem_create(&mut self, req: &mut GemCreateRequest) -> Result<()> {
        if req.size == 0 {
            return Err(Error::Invalid);
        }
        let handle = GemHandle(self.next_gem_handle);
        self.next_gem_handle += 1;

        let gem_obj = GemObject::new(handle, req.size as usize);
        self.gem_objects.push(gem_obj);

        req.handle = handle.0;
        Ok(())
    }

    pub fn handle_gem_info(&self, req: &mut GemInfoRequest) -> Result<()> {
        let handle = GemHandle(req.handle);

        let gem_obj = self
            .gem_objects
            .iter()
            .find(|obj| obj.handle() == handle)
            .ok_or(Error::Invalid)?;

        req.size = gem_obj.size() as u64;
        req.offset = 0;

        Ok(())
    }

    pub fn handle_gem_mmap(&self, req: &mut GemMmapRequest) -> Result<()> {
        let handle = GemHandle(req.handle);

        let gem_obj = self
            .gem_objects
            .iter()
            .find(|obj| obj.handle() == handle)
            .ok_or(Error::Invalid)?;

        if req.size as usize > gem_obj.size() {
            return Err(Error::Invalid);
        }

        // TODO would map to userspace
        req.addr = 0; // Placeholder

        Ok(())
    }

    pub fn device_id(&self) -> u16 {
        self.device_id
    }

    pub fn vram_size(&self) -> u64 {
        self.vram_size
    }

    pub fn driver_version(&self) -> u32 {
        self.driver_version
    }

    pub fn firmware_version(&self) -> u32 {
        self.firmware_version
    }

    pub fn gem_object_count(&self) -> usize {
        self.gem_objects.len()
    }
}

pub trait DrmOps {
    fn open(&mut self, file: &mut DrmFile) -> Result<()>;
    fn close(&mut self, file: &DrmFile) -> Result<()>;
    fn ioctl(&mut self, file: &DrmFile, cmd: u32, arg: u64) -> Result<()>;
}

impl DrmOps for NovaDrm {
    fn open(&mut self, file: &mut DrmFile) -> Result<()> {
        Ok(())
    }

    fn close(&mut self, file: &DrmFile) -> Result<()> {
        Ok(())
    }

    fn ioctl(&mut self, file: &DrmFile, cmd: u32, arg: u64) -> Result<()> {
        match cmd {
            ioctl::GETPARAM => {
                let mut req = GetParamRequest {
                    param: arg as u32,
                    value: 0,
                };
                self.handle_getparam(&mut req)?;
                Ok(())
            }
            ioctl::GEM_CREATE => {
                let mut req = GemCreateRequest {
                    size: arg,
                    handle: 0,
                    flags: 0,
                };
                self.handle_gem_create(&mut req)?;
                Ok(())
            }
            ioctl::GEM_INFO => {
                let mut req = GemInfoRequest {
                    handle: arg as u32,
                    size: 0,
                    offset: 0,
                };
                self.handle_gem_info(&mut req)?;
                Ok(())
            }
            ioctl::GEM_MMAP => {
                let mut req = GemMmapRequest {
                    handle: arg as u32,
                    offset: 0,
                    size: 0,
                    addr: 0,
                };
                self.handle_gem_mmap(&mut req)?;
                Ok(())
            }
            _ => Err(Error::NotSupported),
        }
    }
}
