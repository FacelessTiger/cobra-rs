use anyhow::Result;
use ash::vk;
use glam::UVec2;
use std::ffi::c_void;
use std::mem::ManuallyDrop;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use crate::vulkan::internal_managers::deletion_queue::DeleteValue;
use crate::vulkan::internal_managers::pipeline_manager::GraphicsPipelineKey;
use crate::vulkan::internal_managers::resource_handle::ResourceType;
use crate::{Buffer, BufferFlags, ICobra, ImageFormat, ImageUsage, QueueType, Vulkan};

use super::buffer::BufferVulkan;
use super::queue::QueueVulkan;
use super::swapchain::SwapchainVulkan;
use super::{ImageVulkan, SamplerVulkan};

pub(crate) const SAMPLER_BINDING: u32 = 0;
pub(crate) const STORAGE_IMAGE_BINDING: u32 = 1;
pub(crate) const SAMPLED_IMAGE_BINDING: u32 = 2;

pub(crate) struct IDInfo {
    pub id_counter: u32,
    pub recycled_ids: Vec<u32>
}

pub struct CobraVulkan {
    pub(crate) deletion_queue: Mutex<Vec<DeleteValue>>,

    pub(crate) _entry: ash::Entry,
    pub(crate) instance: ash::Instance,
    pub(crate) chosen_gpu: vk::PhysicalDevice,
    pub(crate) device: ash::Device,

    pub(crate) allocator: ManuallyDrop<vk_mem::Allocator>,
    pub(crate) timeline_value: AtomicU64,
    pub(crate) graphics_queue: QueueVulkan,

    pub(crate) bindless_pool: vk::DescriptorPool,
    pub(crate) bindless_set_layout: vk::DescriptorSetLayout,
    pub(crate) bindless_set: vk::DescriptorSet,
    pub(crate) bindless_pipeline_layout: vk::PipelineLayout,

    pub(crate) surface_fn: ash::khr::surface::Instance,
    pub(crate) swapchain_device_fn: ash::khr::swapchain::Device,
    #[cfg(target_os = "windows")]
    pub(crate) platform_surface_fn: ash::khr::win32_surface::Instance,
    #[cfg(target_os = "linux")]
    pub(crate) platform_surface_fn: ash::khr::wayland_surface::Instance,

    pub(crate) graphics_pipelines: RwLock<HashMap<GraphicsPipelineKey, vk::Pipeline>>,
    pub(crate) id_infos: Mutex<HashMap<ResourceType, IDInfo>>,

    // The only point of ManuallyDrop here is to inhibit the destructor on this buffer, since the Arc for Cobra will be dead when it tries to be deleted so we have to do it manually
    pub(crate) staging_buffer: RwLock<Option<ManuallyDrop<BufferVulkan>>>,
    pub(crate) resizable_bar: bool
}

impl ICobra<Vulkan> for CobraVulkan {
    fn new() -> Result<Arc<Self>> {
        unsafe {
            let (entry, instance) = Self::create_instance()?;
            let chosen_gpu = Self::pick_gpu(&instance)?;
            let (device, graphics_queue) = Self::create_device_and_queues(&instance, &chosen_gpu)?;
            let (bindless_pool, bindless_set_layout, bindless_set, bindless_pipeline_layout) = Self::setup_bindless(&device)?;
            
            let surface_fn = ash::khr::surface::Instance::new(&entry, &instance);
            let swapchain_device_fn = ash::khr::swapchain::Device::new(&instance, &device);
            #[cfg(target_os = "windows")]
            let platform_surface_fn = ash::khr::win32_surface::Instance::new(&entry, &instance);
            #[cfg(target_os = "linux")]
            let platform_surface_fn = ash::khr::wayland_surface::Instance::new(&entry, &instance);
            
            let mut allocator_info = vk_mem::AllocatorCreateInfo::new(&instance, &device, chosen_gpu);
            allocator_info.flags = vk_mem::AllocatorCreateFlags::BUFFER_DEVICE_ADDRESS;
            let allocator = vk_mem::Allocator::new(allocator_info)?;
 
            let resizable_bar = {
                let memory_properties = instance.get_physical_device_memory_properties(chosen_gpu);

                let mut max_device_memory = 0;
                for heap in memory_properties.memory_heaps_as_slice() {
                    if heap.flags.contains(vk::MemoryHeapFlags::DEVICE_LOCAL) {
                        max_device_memory = std::cmp::max(max_device_memory, heap.size);
                    }
                }

                let mut max_host_visible_device_memory = 0;
                for ty in memory_properties.memory_types_as_slice() {
                    if ty.property_flags.contains(vk::MemoryPropertyFlags::DEVICE_LOCAL | vk::MemoryPropertyFlags::HOST_VISIBLE) {
                        max_host_visible_device_memory = std::cmp::max(max_host_visible_device_memory, memory_properties.memory_heaps[ty.heap_index as usize].size);
                    }
                }

                max_device_memory == max_host_visible_device_memory
            };

            let ret = Arc::new(CobraVulkan {
                deletion_queue: Mutex::new(Vec::new()),
                _entry: entry,
                instance, chosen_gpu, device,

                allocator: ManuallyDrop::new(allocator),
                timeline_value: AtomicU64::new(0),
                graphics_queue: QueueVulkan::new(),

                bindless_pool, bindless_set_layout, bindless_set, bindless_pipeline_layout,

                surface_fn, swapchain_device_fn, platform_surface_fn,

                graphics_pipelines: RwLock::new(HashMap::new()),
                id_infos: Mutex::new(HashMap::new()),

                staging_buffer: RwLock::new(None),
                resizable_bar
            });
            let ptr = Arc::as_ptr(&ret) as *mut CobraVulkan;
            (*ptr).graphics_queue.init(ptr, graphics_queue.0, graphics_queue.1)?;

            ret.staging_buffer.write().unwrap().replace(ManuallyDrop::new(BufferVulkan::new_weak(&Arc::downgrade(&ret), 64 * 1024 * 1024, BufferFlags::Upload)?));
            Ok(ret)
        }
    }

    fn new_buffer(&self, cobra: Arc<Self>, size: u64, flags: BufferFlags) -> Result<Buffer<Vulkan>> {
        BufferVulkan::new(cobra, size, flags)
    }

    fn new_image(&self, cobra: Arc<Self>, size: impl Into<UVec2>, format: ImageFormat, usage: ImageUsage) -> Result<ImageVulkan> {
        ImageVulkan::new(cobra, size, format, usage)
    }

    fn new_sampler(&self, cobra: Arc<Self>) -> Result<SamplerVulkan> {
        SamplerVulkan::new(cobra)
    }

    fn new_swapchain(&self, cobra: Arc<Self>, window: *mut c_void, size: UVec2) -> Result<SwapchainVulkan> {
        SwapchainVulkan::new(cobra, window, size)
    }

    fn queue(&self, ty: QueueType) -> &QueueVulkan {
        match ty {
            QueueType::Graphics => &self.graphics_queue
        }
    }

    fn supports_resizable_bar(&self) -> bool {
        self.resizable_bar
    }
}

impl CobraVulkan {

    pub fn advance(&self) -> u64 {
        self.timeline_value.fetch_add(1, Ordering::SeqCst) + 1
    }

    fn create_instance() -> Result<(ash::Entry, ash::Instance)> {
        unsafe {
            let entry = ash::Entry::load()?;
            let mut extensions = vec![vk::KHR_SURFACE_NAME.as_ptr()];
            extensions.append(&mut platform_surface_extensions());

            let instance = entry.create_instance(&vk::InstanceCreateInfo::default()
                .application_info(&vk::ApplicationInfo::default()
                    .api_version(vk::API_VERSION_1_3)
                )
                .enabled_extension_names(&extensions)
            , None)?;

            Ok((entry, instance))
        }
    }

    fn pick_gpu(instance: &ash::Instance) -> Result<vk::PhysicalDevice> {
        unsafe {
            let physical_devices = instance.enumerate_physical_devices()?;

            let mut chosen_gpu = physical_devices[0];
            for physical_device in physical_devices {
                let properties = instance.get_physical_device_properties(physical_device);
                if properties.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
                    chosen_gpu = physical_device;
                    break;
                }
            }

            Ok(chosen_gpu)
        }
    }

    fn create_device_and_queues(instance: &ash::Instance, chosen_gpu: &vk::PhysicalDevice) -> Result<(ash::Device, (vk::Queue, u32))> {
        unsafe {
            let mut graphics_queue_family: u32 = 0;

            let queue_families = instance.get_physical_device_queue_family_properties(*chosen_gpu);
            for (i, &queue_family) in queue_families.iter().enumerate() {
                if queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    graphics_queue_family = i as u32;
                }
            }

            // TODO: only using GPL for renderdoc support with shader module deprecation
            let extensions = [ash::khr::swapchain::NAME.as_ptr(), ash::ext::graphics_pipeline_library::NAME.as_ptr()];
            let device = instance.create_device(*chosen_gpu, &vk::DeviceCreateInfo::default()
                .queue_create_infos(&[vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(graphics_queue_family)
                    .queue_priorities(&[1.0])
                ])
                .enabled_extension_names(&extensions)
                .push_next(&mut vk::PhysicalDeviceVulkan11Features::default()
                    .variable_pointers(true)
                    .variable_pointers_storage_buffer(true)
                )
                .push_next(&mut vk::PhysicalDeviceVulkan12Features::default()
                    .descriptor_binding_sampled_image_update_after_bind(true)
                    .descriptor_binding_storage_image_update_after_bind(true)
                    .descriptor_binding_partially_bound(true)
                    .runtime_descriptor_array(true)
                    .scalar_block_layout(true)
                    .timeline_semaphore(true)
                    .buffer_device_address(true)
                )
                .push_next(&mut vk::PhysicalDeviceVulkan13Features::default()
                    .synchronization2(true)
                    .dynamic_rendering(true)
                )
                .push_next(&mut vk::PhysicalDeviceGraphicsPipelineLibraryFeaturesEXT::default()
                    .graphics_pipeline_library(true)
                )
            , None)?;

            let graphics_queue = device.get_device_queue(graphics_queue_family, 0);

            Ok((device, (graphics_queue, graphics_queue_family)))
        }
    }

    fn setup_bindless(device: &ash::Device) -> Result<(vk::DescriptorPool, vk::DescriptorSetLayout, vk::DescriptorSet, vk::PipelineLayout)> {
        unsafe {
            const BINDING_INFOS: [(vk::DescriptorType, u32, u32); 3] = [
                (vk::DescriptorType::SAMPLER, 1 << 12, SAMPLER_BINDING),
                (vk::DescriptorType::STORAGE_IMAGE, 1 << 20, STORAGE_IMAGE_BINDING),
                (vk::DescriptorType::SAMPLED_IMAGE, 1 << 20, SAMPLED_IMAGE_BINDING)
            ];

            let mut pool_sizes = Vec::new();
            let mut bindings = Vec::new();
            let mut binding_flags = Vec::new();

            for binding_info in BINDING_INFOS {
                pool_sizes.push(vk::DescriptorPoolSize::default()
                    .ty(binding_info.0)
                    .descriptor_count(binding_info.1)
                );
                bindings.push(vk::DescriptorSetLayoutBinding::default()
                    .binding(binding_info.2)
                    .descriptor_type(binding_info.0)
                    .descriptor_count(binding_info.1)
                    .stage_flags(vk::ShaderStageFlags::ALL)
                );

                binding_flags.push(vk::DescriptorBindingFlags::UPDATE_AFTER_BIND | vk::DescriptorBindingFlags::PARTIALLY_BOUND);
            }

            let bindless_pool = device.create_descriptor_pool(&vk::DescriptorPoolCreateInfo::default()
                .flags(vk::DescriptorPoolCreateFlags::UPDATE_AFTER_BIND)
                .max_sets(1)
                .pool_sizes(&pool_sizes)
            , None)?;
            let bindless_set_layout = device.create_descriptor_set_layout(&vk::DescriptorSetLayoutCreateInfo::default()
                .flags(vk::DescriptorSetLayoutCreateFlags::UPDATE_AFTER_BIND_POOL)
                .bindings(&bindings)
                .push_next(&mut vk::DescriptorSetLayoutBindingFlagsCreateInfo::default()
                    .binding_flags(&binding_flags)
                )
            , None)?;
            let bindless_set = device.allocate_descriptor_sets(&vk::DescriptorSetAllocateInfo::default()
                .descriptor_pool(bindless_pool)
                .set_layouts(&[bindless_set_layout])
            )?[0];
            let bindless_pipeline_layout = device.create_pipeline_layout(&vk::PipelineLayoutCreateInfo::default()
                .set_layouts(&[bindless_set_layout])
                .push_constant_ranges(&[vk::PushConstantRange::default()
                    .stage_flags(vk::ShaderStageFlags::ALL)
                    .offset(0)
                    .size(128)
                ])
            , None)?;

            Ok((bindless_pool, bindless_set_layout, bindless_set, bindless_pipeline_layout))
        }
    }

}

impl Drop for CobraVulkan {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().expect("Failed to wait idle on shutdowm");

            for pipeline in self.graphics_pipelines.read().unwrap().iter() {
                self.device.destroy_pipeline(*pipeline.1, None);
            }

            self.graphics_queue.destroy();
            self.push(self.staging_buffer.read().unwrap().as_ref().unwrap().allocation);
            self.flush();

            self.device.destroy_descriptor_set_layout(self.bindless_set_layout, None);
            self.device.destroy_pipeline_layout(self.bindless_pipeline_layout, None);
            self.device.destroy_descriptor_pool(self.bindless_pool, None);

            ManuallyDrop::drop(&mut self.allocator);

            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}

#[cfg(target_os = "windows")]
fn platform_surface_extensions() -> Vec<*const i8> {
    vec![vk::KHR_WIN32_SURFACE_NAME.as_ptr()]
}

#[cfg(target_os = "linux")]
fn platform_surface_extensions() -> Vec<*const i8> {
    vec![vk::KHR_WAYLAND_SURFACE_NAME.as_ptr()]
}