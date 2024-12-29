use ash::vk;
use super::super::mappings::CobraVulkan;

pub enum DeleteValue {
    Swapchain(vk::SwapchainKHR),
    Surface(vk::SurfaceKHR),
    CommandPool(vk::CommandPool),
    Semaphore(vk::Semaphore),
    ImageView(vk::ImageView),
    Image((vk::Image, vk_mem::Allocation)),
    Buffer((vk::Buffer, vk_mem::Allocation)),
    Sampler(vk::Sampler),
    ShaderModule(vk::ShaderModule)
}

impl From<vk::SwapchainKHR> for DeleteValue {
    fn from(value: vk::SwapchainKHR) -> Self {
        DeleteValue::Swapchain(value)
    }
}

impl From<vk::SurfaceKHR> for DeleteValue {
    fn from(value: vk::SurfaceKHR) -> Self {
        DeleteValue::Surface(value)
    }
}

impl From<vk::CommandPool> for DeleteValue {
    fn from(value: vk::CommandPool) -> Self {
        DeleteValue::CommandPool(value)
    }
}

impl From<vk::Semaphore> for DeleteValue {
    fn from(value: vk::Semaphore) -> Self {
        DeleteValue::Semaphore(value)
    }
}

impl From<vk::ImageView> for DeleteValue {
    fn from(value: vk::ImageView) -> Self {
        DeleteValue::ImageView(value)
    }
}

impl From<(vk::Image, vk_mem::Allocation)> for DeleteValue {
    fn from(value: (vk::Image, vk_mem::Allocation)) -> Self {
        DeleteValue::Image(value)
    }
}

impl From<(vk::Buffer, vk_mem::Allocation)> for DeleteValue {
    fn from(value: (vk::Buffer, vk_mem::Allocation)) -> Self {
        DeleteValue::Buffer(value)
    }
}

impl From<vk::Sampler> for DeleteValue {
    fn from(value: vk::Sampler) -> Self {
        DeleteValue::Sampler(value)
    }
}

impl From<vk::ShaderModule> for DeleteValue {
    fn from(value: vk::ShaderModule) -> Self {
        DeleteValue::ShaderModule(value)
    }
}

impl CobraVulkan {

    pub(crate) fn push(&self, to_delete: impl Into<DeleteValue>) { self.deletion_queue.lock().unwrap().push(to_delete.into()); }

    pub(crate) fn flush(&mut self) {
        unsafe {
            let mut deletion_queue = self.deletion_queue.lock().unwrap();
            for value in deletion_queue.iter_mut() {
                match value {
                    DeleteValue::Swapchain(swapchain) => self.swapchain_device_fn.destroy_swapchain(*swapchain, None),
                    DeleteValue::Surface(surface) => self.surface_fn.destroy_surface(*surface, None),
                    DeleteValue::CommandPool(pool) => self.device.destroy_command_pool(*pool, None),
                    DeleteValue::Semaphore(semaphore) => self.device.destroy_semaphore(*semaphore, None),
                    DeleteValue::ImageView(image_view) => self.device.destroy_image_view(*image_view, None),
                    DeleteValue::Image(image) => self.allocator.destroy_image(image.0, &mut image.1),
                    DeleteValue::Buffer(buffer) => self.allocator.destroy_buffer(buffer.0, &mut buffer.1),
                    DeleteValue::Sampler(sampler) => self.device.destroy_sampler(*sampler, None),
                    DeleteValue::ShaderModule(module) => self.device.destroy_shader_module(*module, None)
                }
            }
            deletion_queue.clear();
        }
    }

}