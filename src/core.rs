use crate::{ChipsetSpec, Error, Result, NVIDIA_VENDOR_ID};
use core::ptr;

pub mod regs {
    pub const PMC_BOOT_0: u32 = 0x000000;
    pub const PMC_ENABLE: u32 = 0x000200;
    pub const PMC_STATUS: u32 = 0x000204;
    pub const PMC_STATUS_IDLE: u32 = 1 << 0;
}

pub struct Bar0 {
    base_addr: *mut u8,
    size: usize,
}

impl Bar0 {
    pub fn new(base_addr: *mut u8, size: usize) -> Self {
        Self { base_addr, size }
    }

    pub fn read_u32(&self, offset: u32) -> Result<u32> {
        if offset as usize + 4 > self.size {
            return Err(Error::Invalid);
        }

        unsafe {
            let ptr = self.base_addr.add(offset as usize) as *const u32;
            Ok(ptr::read_volatile(ptr))
        }
    }

    pub fn write_u32(&self, offset: u32, value: u32) -> Result<()> {
        if offset as usize + 4 > self.size {
            return Err(Error::Invalid);
        }

        unsafe {
            let ptr = self.base_addr.add(offset as usize) as *mut u32;
            ptr::write_volatile(ptr, value);
        }

        Ok(())
    }

    pub fn base_addr(&self) -> *mut u8 {
        self.base_addr
    }

    /// Get the size
    pub fn size(&self) -> usize {
        self.size
    }
}

/// GPU firmware management
pub struct Firmware {
    loaded: bool,
    version: u32,
}

impl Firmware {
    /// Create a new firmware instance
    pub fn new() -> Self {
        Self {
            loaded: false,
            version: 0,
        }
    }

    /// Load firmware for the given chipset
    pub fn load(&mut self, spec: &ChipsetSpec) -> Result<()> {
        // Placeholder for firmware loading logic
        // In a real implementation, this would load GSP firmware
        self.loaded = true;
        self.version = 1; // Dummy version
        Ok(())
    }

    pub fn is_loaded(&self) -> bool {
        self.loaded
    }

    pub fn version(&self) -> u32 {
        self.version
    }
}

impl Default for Firmware {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Gpu {
    spec: ChipsetSpec,
    bar: Bar0,
    fw: Firmware,
}

impl Gpu {
    pub fn new(bar: Bar0) -> Result<Self> {
        let pmc_boot_0 = bar.read_u32(regs::PMC_BOOT_0)?;
        let spec = ChipsetSpec::detect_from_register(pmc_boot_0)?;
        let mut fw = Firmware::new();
        fw.load(&spec)?;

        Ok(Self { spec, bar, fw })
    }

    pub fn spec(&self) -> &ChipsetSpec {
        &self.spec
    }
    pub fn bar(&self) -> &Bar0 {
        &self.bar
    }
    pub fn firmware(&self) -> &Firmware {
        &self.fw
    }

    pub fn reset(&mut self) -> Result<()> {
        self.bar.write_u32(regs::PMC_ENABLE, 0)?;
        self.wait_for_idle()?;
        self.bar.write_u32(regs::PMC_ENABLE, 0xffffffff)?;
        Ok(())
    }

    pub fn wait_for_idle(&self) -> Result<()> {
        const MAX_RETRIES: u32 = 1000;

        for _ in 0..MAX_RETRIES {
            let status = self.bar.read_u32(regs::PMC_STATUS)?;
            if status & regs::PMC_STATUS_IDLE != 0 {
                return Ok(());
            }
            // todo, would use proper delay
        }

        Err(Error::Timeout)
    }

    pub fn init_hardware(&mut self) -> Result<()> {
        self.reset()?;
        Ok(())
    }
}

pub struct NovaCore {
    gpu: Gpu,
}

impl NovaCore {
    pub fn new(bar_addr: *mut u8, bar_size: usize) -> Result<Self> {
        let bar = Bar0::new(bar_addr, bar_size);
        let gpu = Gpu::new(bar)?;

        Ok(Self { gpu })
    }

    pub fn gpu(&self) -> &Gpu {
        &self.gpu
    }

    pub fn gpu_mut(&mut self) -> &mut Gpu {
        &mut self.gpu
    }

    pub fn probe(bar_addr: *mut u8, bar_size: usize) -> Result<Self> {
        let mut core = Self::new(bar_addr, bar_size)?;
        core.gpu_mut().init_hardware()?;
        Ok(core)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PciDeviceId {
    pub vendor: u16,
    pub device: u16,
}

impl PciDeviceId {
    pub const fn new(vendor: u16, device: u16) -> Self {
        Self { vendor, device }
    }
}

pub fn generate_pci_table() -> &'static [PciDeviceId] {
    use crate::chipset::SUPPORTED_DEVICES;

    const fn make_table() -> [PciDeviceId; 15] {
        [
            PciDeviceId::new(NVIDIA_VENDOR_ID, 0x1e02),
            PciDeviceId::new(NVIDIA_VENDOR_ID, 0x1e04),
            PciDeviceId::new(NVIDIA_VENDOR_ID, 0x1e07),
            PciDeviceId::new(NVIDIA_VENDOR_ID, 0x1e82),
            PciDeviceId::new(NVIDIA_VENDOR_ID, 0x1e84),

            PciDeviceId::new(NVIDIA_VENDOR_ID, 0x2200),
            PciDeviceId::new(NVIDIA_VENDOR_ID, 0x2204),
            PciDeviceId::new(NVIDIA_VENDOR_ID, 0x2206),
            PciDeviceId::new(NVIDIA_VENDOR_ID, 0x2208),
            PciDeviceId::new(NVIDIA_VENDOR_ID, 0x2216),
            PciDeviceId::new(NVIDIA_VENDOR_ID, 0x2220),

            PciDeviceId::new(NVIDIA_VENDOR_ID, 0x2684),
            PciDeviceId::new(NVIDIA_VENDOR_ID, 0x2704),
            PciDeviceId::new(NVIDIA_VENDOR_ID, 0x2782),
            PciDeviceId::new(NVIDIA_VENDOR_ID, 0x2786),
        ]
    }

    static TABLE: [PciDeviceId; 15] = make_table();
    &TABLE
}
