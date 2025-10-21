extern crate alloc;
use alloc::vec::Vec;
use nutcase::{
    core::{NovaCore, Bar0},
    drm::{NovaDrm, GetParamRequest, GemCreateRequest, param},
    gsp::{GspClient, JobType},
    memory::{DmaBuffer, MemoryFlags},
    ChipsetSpec, GpuGeneration, Result,
};

/// Example: Initialize the GPU driver
fn example_init_driver() -> Result<()> {
    println!("=== Initializing Nova GPU Driver ===");
    
    // Simulate BAR0 mapping (in real kernel, this comes from PCI)
    let bar_addr = core::ptr::null_mut();
    let bar_size = 16 * 1024 * 1024; // 16MB
    
    // Create Nova-Core driver
    let core = NovaCore::probe(bar_addr, bar_size)?;
    
    println!("✓ GPU detected: {}", core.gpu().spec().name);
    println!("✓ Generation: {:?}", core.gpu().spec().generation);
    println!("✓ Device ID: 0x{:04x}", core.gpu().spec().device_id);
    println!("✓ Firmware loaded: {}", core.gpu().firmware().is_loaded());
    
    Ok(())
}

/// Example: DRM operations
fn example_drm_operations() -> Result<()> {
    println!("\n=== DRM Operations ===");
    
    // Create DRM driver
    let mut drm = NovaDrm::new(0x2204, 8 * 1024 * 1024 * 1024, 1);
    
    // Query parameters
    let mut getparam = GetParamRequest {
        param: param::CHIPSET_ID,
        value: 0,
    };
    drm.handle_getparam(&mut getparam)?;
    println!("✓ Chipset ID: 0x{:04x}", getparam.value);
    
    // Create GEM object
    let mut gem_create = GemCreateRequest {
        size: 4096,
        handle: 0,
        flags: 0,
    };
    drm.handle_gem_create(&mut gem_create)?;
    println!("✓ Created GEM object: handle={}", gem_create.handle);
    
    println!("✓ Total GEM objects: {}", drm.gem_object_count());
    
    Ok(())
}

/// Example: GSP communication
fn example_gsp_communication() -> Result<()> {
    println!("\n=== GSP Communication ===");
    
    // Create DMA buffer for ring buffer
    let ring_buffer = DmaBuffer::new(
        core::ptr::null_mut(),
        0x1000_0000,
        4096,
    );
    
    // Create GSP client
    let mut gsp = GspClient::new(ring_buffer, core::ptr::null_mut());
    gsp.set_context_handle(1);
    
    // Create command buffer
    let cmd_buffer = DmaBuffer::new(
        core::ptr::null_mut(),
        0x2000_0000,
        1024,
    );
    
    // Submit a graphics job
    gsp.submit_job(JobType::Graphics, &cmd_buffer, 42)?;
    println!("✓ Submitted graphics job with fence_id=42");
    
    // Allocate memory
    let handle = gsp.alloc_memory(8192, MemoryFlags::COHERENT)?;
    println!("✓ Allocated memory: handle={}", handle);
    
    // Poll for responses
    let responses = gsp.poll_responses()?;
    println!("✓ Received {} responses", responses.len());
    
    Ok(())
}

/// Example: Chipset detection
fn example_chipset_detection() -> Result<()> {
    println!("\n=== Chipset Detection ===");
    
    // Simulate different chipset IDs
    let test_cases = vec![
        (0x16200000, "TU102", GpuGeneration::Turing),
        (0x17200000, "GA102", GpuGeneration::Ampere),
        (0x18800000, "AD102", GpuGeneration::Ada),
    ];
    
    for (pmc_boot_0, expected_name, expected_gen) in test_cases {
        let spec = ChipsetSpec::detect_from_register(pmc_boot_0)?;
        println!("✓ Detected: {} ({:?})", spec.name, spec.generation);
        assert_eq!(spec.name, expected_name);
        assert_eq!(spec.generation, expected_gen);
    }
    
    Ok(())
}

/// Example: Memory management
fn example_memory_management() -> Result<()> {
    println!("\n=== Memory Management ===");
    
    // Create DMA buffer
    let mut buffer = DmaBuffer::new(
        core::ptr::null_mut(),
        0x3000_0000,
        4096,
    );
    
    // Write data
    let data: u32 = 0xDEADBEEF;
    buffer.write(0, &data)?;
    println!("✓ Wrote 0x{:08x} to buffer", data);
    
    // Read data back
    let read_data: u32 = buffer.read(0)?;
    println!("✓ Read 0x{:08x} from buffer", read_data);
    assert_eq!(data, read_data);
    
    // Buffer info
    println!("✓ Buffer DMA address: 0x{:016x}", buffer.dma_addr());
    println!("✓ Buffer size: {} bytes", buffer.size());
    
    Ok(())
}

fn main() -> Result<()> {
    println!("╔════════════════════════════════════════╗");
    println!("║  Nutcase Nova GPU Driver Examples     ║");
    println!("╚════════════════════════════════════════╝\n");
    
    // Run examples
    example_chipset_detection()?;
    example_memory_management()?;
    
    // Note: These would fail without actual hardware
    // example_init_driver()?;
    // example_drm_operations()?;
    // example_gsp_communication()?;
    
    println!("\n✓ All examples completed successfully!");
    
    Ok(())
}

#[cfg(not(test))]
fn println(s: &str) {
    // In no_std, would use kernel logging
    #[cfg(feature = "std")]
    std::println!("{}", s);
}
