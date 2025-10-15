use nutcase::{
    chipset::{ChipsetSpec, GpuGeneration},
    drm::{NovaDrm, GetParamRequest, GemCreateRequest, param},
    memory::{DmaBuffer, GemHandle, GemObject, MemoryFlags},
    Error, Result,
};

#[test]
fn test_chipset_detection() {
    let spec = ChipsetSpec::detect_from_register(0x16200000).unwrap();
    assert_eq!(spec.name, "TU102");
    assert_eq!(spec.generation, GpuGeneration::Turing);
    assert_eq!(spec.device_id, 0x1e02);
    
    let spec = ChipsetSpec::detect_from_register(0x17200000).unwrap();
    assert_eq!(spec.name, "GA102");
    assert_eq!(spec.generation, GpuGeneration::Ampere);
    
    let spec = ChipsetSpec::detect_from_register(0x18800000).unwrap();
    assert_eq!(spec.name, "AD102");
    assert_eq!(spec.generation, GpuGeneration::Ada);
    
    let result = ChipsetSpec::detect_from_register(0xFFFFFFFF);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::NoDevice);
}

#[test]
fn test_drm_getparam() {
    let drm = NovaDrm::new(0x2204, 8_589_934_592, 1);
    
    let mut req = GetParamRequest {
        param: param::CHIPSET_ID,
        value: 0,
    };
    
    drm.handle_getparam(&mut req).unwrap();
    assert_eq!(req.value, 0x2204);
    
    req.param = param::VRAM_SIZE;
    drm.handle_getparam(&mut req).unwrap();
    assert_eq!(req.value, 8_589_934_592);
}

#[test]
fn test_drm_gem_create() {
    let mut drm = NovaDrm::new(0x2204, 8_589_934_592, 1);
    
    let mut req = GemCreateRequest {
        size: 4096,
        handle: 0,
        flags: 0,
    };
    
    drm.handle_gem_create(&mut req).unwrap();
    assert_ne!(req.handle, 0);
    assert_eq!(drm.gem_object_count(), 1);
    
    req.size = 8192;
    req.handle = 0;
    drm.handle_gem_create(&mut req).unwrap();
    assert_eq!(drm.gem_object_count(), 2);
}

#[test]
fn test_drm_gem_create_invalid() {
    let mut drm = NovaDrm::new(0x2204, 8_589_934_592, 1);
    
    let mut req = GemCreateRequest {
        size: 0,
        handle: 0,
        flags: 0,
    };
    
    let result = drm.handle_gem_create(&mut req);
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), Error::Invalid);
}

#[test]
fn test_memory_flags() {
    let flags = MemoryFlags::new(MemoryFlags::COHERENT | MemoryFlags::CACHED);
    
    assert!(flags.is_coherent());
    assert!(flags.is_cached());
    assert!(!flags.is_write_combined());
    
    let flags2 = MemoryFlags::new(MemoryFlags::WRITE_COMBINED);
    assert!(!flags2.is_coherent());
    assert!(flags2.is_write_combined());
}

#[test]
fn test_gem_object() {
    let handle = GemHandle(42);
    let obj = GemObject::new(handle, 4096);
    
    assert_eq!(obj.handle(), handle);
    assert_eq!(obj.size(), 4096);
    assert!(obj.dma_buffer().is_none());
}

#[test]
fn test_dma_buffer_operations() {
    let mut buffer = DmaBuffer::new(
        core::ptr::null_mut(),
        0x1000_0000,
        4096,
    );
    
    assert_eq!(buffer.dma_addr(), 0x1000_0000);
    assert_eq!(buffer.size(), 4096);
    
    let data: u32 = 0xDEADBEEF;
    buffer.write(0, &data).unwrap();
    let read_data: u32 = buffer.read(0).unwrap();
    assert_eq!(data, read_data);
}

#[test]
fn test_error_display() {
    assert_eq!(format!("{}", Error::NoMem), "Out of memory");
    assert_eq!(format!("{}", Error::Invalid), "Invalid argument");
    assert_eq!(format!("{}", Error::NoDevice), "No such device");
    assert_eq!(format!("{}", Error::Timeout), "Timeout");
}

#[test]
fn test_error_codes() {
    assert_eq!(Error::NoMem.to_errno(), -12);
    assert_eq!(Error::Invalid.to_errno(), -22);
    assert_eq!(Error::NoDevice.to_errno(), -19);
    assert_eq!(Error::Timeout.to_errno(), -110);
}

#[test]
fn test_supported_devices() {
    use nutcase::chipset::SUPPORTED_DEVICES;
    
    assert_eq!(SUPPORTED_DEVICES.len(), 15);
    
    assert_eq!(SUPPORTED_DEVICES[0].0, 0x1e02);
    assert_eq!(SUPPORTED_DEVICES[0].1, "TU102");
    
    assert_eq!(SUPPORTED_DEVICES[14].0, 0x2820);
    assert_eq!(SUPPORTED_DEVICES[14].1, "AD107");
}

#[test]
fn test_pci_device_table() {
    use nutcase::core::generate_pci_table;
    use nutcase::NVIDIA_VENDOR_ID;
    
    let table = generate_pci_table();
    
    assert_eq!(table.len(), 15);
    
    for entry in table {
        assert_eq!(entry.vendor, NVIDIA_VENDOR_ID);
    }
    
    assert_eq!(table[0].device, 0x1e02);
}
