use anyhow::Result;
use ash::vk;
use vk_mem::Alloc;
use std::sync::Weak;
use std::{ffi::c_void, sync::Arc};

use crate::vulkan::internal_managers::utils::AllocationInfo;
use crate::{BufferFlags, IBuffer, ICommandList, IQueue, Vulkan};

use super::queue::QueueVulkan;
use super::CobraVulkan;

enum PtrType {
    Arc(Arc<CobraVulkan>),
    Weak(Weak<CobraVulkan>)
}

impl PtrType {
    fn get(&self) -> Arc<CobraVulkan> {
        match self {
            PtrType::Arc(val) => val.clone(),
            PtrType::Weak(val) => val.upgrade().unwrap().clone()
        }
    }
}

pub struct BufferVulkan {
    pub(crate) allocation: (vk::Buffer, vk_mem::Allocation),
    allocation_info: AllocationInfo,
    size: u64,
    address: u64,

    cobra: PtrType
}

impl IBuffer<Vulkan> for BufferVulkan {
    fn set<T>(&mut self, queue: &mut QueueVulkan, source: &[T], buffer_offset: u64) -> Result<()> 
        where T: Copy {
        let cobra = self.cobra.get();
        let staging_buffer = cobra.staging_buffer.read().unwrap();
        let staging_buffer = staging_buffer.as_ref().unwrap();
        staging_buffer.host_slice().copy_from_slice(source);
        
        let cmd = queue.begin()?;
        cmd.copy_buffer_region(staging_buffer, self, source.len() as u64, 0, buffer_offset);
        queue.submit(cmd, None)?.wait()?;

        Ok(())
    }

    fn host_address(&self) -> *mut c_void {
        let address = self.allocation_info.read().unwrap().mapped_data;
        assert!(!address.is_null());

        address
    }

    fn device_address(&self) -> u64 {
        self.address
    }

    fn host_slice<T>(&self) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(self.allocation_info.read().unwrap().mapped_data.cast(), self.size as usize / std::mem::size_of::<T>())
        }
    }

    fn size(&self) -> u64 {
        self.size
    }
}

impl BufferVulkan {
    pub(crate) fn new(cobra: Arc<CobraVulkan>, size: u64, flags: BufferFlags) -> Result<Self> {
        Self::init(PtrType::Arc(cobra), size, flags)
    }

    pub(crate) fn new_weak(cobra: &Weak<CobraVulkan>, size: u64, flags: BufferFlags) -> Result<BufferVulkan> {
        Self::init(PtrType::Weak(cobra.clone()), size, flags)
    }

    fn init(cobra: PtrType, size: u64, flags: BufferFlags) -> Result<BufferVulkan> {
        unsafe {
            let cb = cobra.get();
            let mut allocation_info = vk_mem::AllocationCreateInfo::default();
            allocation_info.usage = vk_mem::MemoryUsage::Auto;

            match flags {
                BufferFlags::Default => {
                    allocation_info.required_flags = vk::MemoryPropertyFlags::DEVICE_LOCAL;
                },
                BufferFlags::Upload | BufferFlags::Readback => {
                    allocation_info.flags = vk_mem::AllocationCreateFlags::MAPPED | vk_mem::AllocationCreateFlags::HOST_ACCESS_RANDOM;
                    allocation_info.required_flags = vk::MemoryPropertyFlags::HOST_COHERENT;
                },
                BufferFlags::DeviceUpload => {
                    allocation_info.flags = vk_mem::AllocationCreateFlags::MAPPED | vk_mem::AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE;
                    allocation_info.required_flags = vk::MemoryPropertyFlags::DEVICE_LOCAL | vk::MemoryPropertyFlags::HOST_COHERENT;
                }
            }
            
            let (buffer, allocation) = cb.allocator.create_buffer(&vk::BufferCreateInfo::default()
                .size(size)
                .usage(
                    vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST |
                    vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::SHADER_DEVICE_ADDRESS |
                    vk::BufferUsageFlags::INDEX_BUFFER | vk::BufferUsageFlags::INDIRECT_BUFFER 
                )
            , &allocation_info)?;
           let allocation_info = AllocationInfo::new(cb.allocator.get_allocation_info(&allocation));
           let address = cb.device.get_buffer_device_address(&vk::BufferDeviceAddressInfo::default().buffer(buffer));

            drop(cb);
            Ok(BufferVulkan {
                allocation: (buffer, allocation), allocation_info, address, size,
                cobra
            })
        }
    }
}

impl Drop for BufferVulkan {
    fn drop(&mut self) {
        self.cobra.get().push(self.allocation);
    }
}