use crate::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GpuGeneration {
    Turing,
    Ampere,
    Ada,
}

pub struct ChipsetSpec {
    pub generation: GpuGeneration,
    pub device_id: u16,
    pub name: &'static str,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChipsetSpec {
    pub generation: GpuGeneration,
    pub device_id: u16,
    pub name: &'static str,
}

impl ChipsetSpec {
    pub const fn new(generation: GpuGeneration, device_id: u16, name: &'static str) -> Self {
        Self {
            generation,
            device_id,
            name,
        }
    }

    pub fn detect_from_register(pmc_boot_0: u32) -> Result<Self> {
        let device_id = ((pmc_boot_0 >> 20) & 0xfff) as u16;
        
        match device_id {
            0x162 => Ok(Self::new(GpuGeneration::Turing, 0x1e02, "TU102")),
            0x164 => Ok(Self::new(GpuGeneration::Turing, 0x1e04, "TU104")),
            0x166 => Ok(Self::new(GpuGeneration::Turing, 0x1e07, "TU106")),
            0x167 => Ok(Self::new(GpuGeneration::Turing, 0x1e82, "TU117")),
            0x168 => Ok(Self::new(GpuGeneration::Turing, 0x1e84, "TU116")),
            
            0x170 => Ok(Self::new(GpuGeneration::Ampere, 0x2200, "GA100")),
            0x172 => Ok(Self::new(GpuGeneration::Ampere, 0x2204, "GA102")),
            0x173 => Ok(Self::new(GpuGeneration::Ampere, 0x2206, "GA103")),
            0x174 => Ok(Self::new(GpuGeneration::Ampere, 0x2208, "GA104")),
            0x176 => Ok(Self::new(GpuGeneration::Ampere, 0x2216, "GA106")),
            0x177 => Ok(Self::new(GpuGeneration::Ampere, 0x2220, "GA107")),
            
            0x188 => Ok(Self::new(GpuGeneration::Ada, 0x2684, "AD102")),
            0x189 => Ok(Self::new(GpuGeneration::Ada, 0x2704, "AD103")),
            0x190 => Ok(Self::new(GpuGeneration::Ada, 0x2782, "AD104")),
            0x191 => Ok(Self::new(GpuGeneration::Ada, 0x2786, "AD106")),
            0x192 => Ok(Self::new(GpuGeneration::Ada, 0x2820, "AD107")),
            
            _ => Err(Error::NoDevice),
        }
    }
}

pub const SUPPORTED_DEVICES: &[(u16, &str)] = &[
    (0x1e02, "TU102"),
    (0x1e04, "TU104"),
    (0x1e07, "TU106"),
    (0x1e82, "TU117"),
    (0x1e84, "TU116"),
    
    (0x2200, "GA100"),
    (0x2204, "GA102"),
    (0x2206, "GA103"),
    (0x2208, "GA104"),
    (0x2216, "GA106"),
    (0x2220, "GA107"),
    
    (0x2684, "AD102"),
    (0x2704, "AD103"),
    (0x2782, "AD104"),
    (0x2786, "AD106"),
    (0x2820, "AD107"),
];
