use std::sync::Arc;

use anyhow::Result;
use ash::vk::{self, ImageSubresourceRange};
use glam::UVec2;
use std::ffi::c_void;

use crate::{ISwapchain, ImageFormat, Vulkan};

use super::{image::ImageVulkan, CobraVulkan};

pub struct SwapchainVulkan {
    surface: vk::SurfaceKHR,
    surface_format: vk::SurfaceFormatKHR,
    pub(crate) swapchain: vk::SwapchainKHR,

    pub(crate) images: Vec<ImageVulkan>,
    pub(crate) image_index: u32,
 
    pub(crate) semaphores: Vec<vk::Semaphore>,
    pub(crate) semaphore_index: usize,

    size: UVec2,
    pub(crate) dirty: bool,

    cobra: Arc<CobraVulkan>
}

impl ISwapchain<Vulkan> for SwapchainVulkan {
    fn current_image(&mut self) -> &mut ImageVulkan {
        &mut self.images[self.image_index as usize]
    }

    fn resize(&mut self, size: UVec2) {
        if (size.x == 0 || size.y == 0) || size == self.size { return; }
        
        self.size = size;
        self.dirty = true;
    }

    fn size(&self) -> UVec2 {
        self.size
    }
}

impl SwapchainVulkan {
    pub(crate) fn new(cobra: Arc<CobraVulkan>, window: *mut c_void, size: UVec2) -> Result<Self> {
        unsafe {
            let surface = create_surface(&cobra, window)?;
            let surface_format = Self::choose_surface_format(&cobra.surface_fn.get_physical_device_surface_formats(cobra.chosen_gpu, surface)?);
            let mut size = size;
            let (swapchain, images) = Self::create_swapchain(&cobra, surface, surface_format, None, &mut size)?;

            let mut semaphores = Vec::new();
            for _ in 0..(images.len() * 2) {
                semaphores.push(cobra.device.create_semaphore(&vk::SemaphoreCreateInfo::default(), None)?);
            }

            Ok(SwapchainVulkan {
                surface, surface_format, swapchain, images, semaphores, size, cobra,
                image_index: 0,
                semaphore_index: 0,
                dirty: false
            })
        }
    }

    fn create_swapchain(cobra: &Arc<CobraVulkan>, surface: vk::SurfaceKHR, surface_format: vk::SurfaceFormatKHR, old_swapchain: Option<vk::SwapchainKHR>, size: &mut UVec2) -> Result<(vk::SwapchainKHR, Vec<ImageVulkan>)> {
        unsafe {
            let capabilities = cobra.surface_fn.get_physical_device_surface_capabilities(cobra.chosen_gpu, surface)?;
            let extent = Self::choose_swap_extent(&capabilities, size);
            *size = UVec2::new(extent.width, extent.height);

            let swapchain = cobra.swapchain_device_fn.create_swapchain(&vk::SwapchainCreateInfoKHR::default()
                .surface(surface)
                .min_image_count(capabilities.min_image_count)
                .image_format(surface_format.format)
                .image_color_space(surface_format.color_space)
                .image_extent(extent)
                .image_array_layers(1)
                .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::TRANSFER_DST)
                .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
                .pre_transform(capabilities.current_transform)
                .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
                .present_mode(vk::PresentModeKHR::MAILBOX) // TODO: this should be chosen and effected by vsync on or off
                .clipped(true)
                .old_swapchain(match old_swapchain {
                    Some(swapchain) => swapchain,
                    None => vk::SwapchainKHR::null()
                })
            , None)?;

            // TODO: probably delay this a few frames by putting in the deletion queue instead
            if let Some(swapchain) = old_swapchain {
                cobra.swapchain_device_fn.destroy_swapchain(swapchain, None);
            }

            let vulkan_images = cobra.swapchain_device_fn.get_swapchain_images(swapchain)?;
            let images: Vec<ImageVulkan> = vulkan_images
                .iter()
                .map(|image| {
                    let image_view = cobra.device.create_image_view(&vk::ImageViewCreateInfo::default()
                        .image(*image)
                        .view_type(vk::ImageViewType::TYPE_2D)
                        .format(surface_format.format)
                        .subresource_range(ImageSubresourceRange {
                            aspect_mask: vk::ImageAspectFlags::COLOR,
                            base_mip_level: 0,
                            level_count: 1,
                            base_array_layer: 0,
                            layer_count: 1
                        })
                    , None).unwrap();
                    
                    ImageVulkan::new_swapchain_image(cobra.clone(), *image, image_view, ImageFormat::R8G8B8A8Unorm, *size)
                })
                .collect();

            Ok((swapchain, images))
        }
    }

    fn choose_surface_format(available_formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
        for format in available_formats {
            if format.format == vk::Format::R8G8B8A8_UNORM && format.color_space == vk::ColorSpaceKHR::SRGB_NONLINEAR {
                return *format;
            }
        }

        available_formats[0]
    }

    fn choose_swap_extent(capabilities: &vk::SurfaceCapabilitiesKHR, size: &UVec2) -> vk::Extent2D {
        match capabilities.current_extent.width == u32::MAX {
            true => {
                vk::Extent2D {
                    width: u32::clamp(size.x, capabilities.min_image_extent.width, capabilities.max_image_extent.width),
                    height: u32::clamp(size.y, capabilities.min_image_extent.height, capabilities.max_image_extent.height)
                }
            }
            false => capabilities.current_extent
        }
    }

    pub(crate) fn recreate(&mut self) -> Result<()> {
        let (swapchain, images) = Self::create_swapchain(&self.cobra, self.surface, self.surface_format, Some(self.swapchain), &mut self.size)?;

        self.swapchain = swapchain;
        self.images = images;
        self.dirty = false;

        Ok(())
    }
}

impl Drop for SwapchainVulkan {
    fn drop(&mut self) {
        self.cobra.push(self.swapchain);
        self.cobra.push(self.surface);

        for (_, semaphore) in self.semaphores.iter().enumerate() {
            self.cobra.push(*semaphore);
        }
    }
}

#[cfg(target_os = "windows")]
fn create_surface(cobra: &Arc<CobraVulkan>, window: *mut c_void) -> Result<vk::SurfaceKHR> {
    unsafe {
        Ok(cobra.platform_surface_fn.create_win32_surface(&ash::vk::Win32SurfaceCreateInfoKHR::default()
            .hinstance(kernel32::GetModuleHandleA(std::ptr::null()) as vk::HINSTANCE)
            .hwnd(window as vk::HWND)
        , None)?)
    }
}

#[cfg(target_os = "linux")]
fn create_surface(cobra: &Arc<CobraVulkan>, window: *mut c_void) -> Result<vk::SurfaceKHR> {
    unsafe {
        Ok(cobra.platform_surface_fn.create_wayland_surface(&ash::vk::WaylandSurfaceCreateInfoKHR::default()
            //.display(kernel32::GetModuleHandleA(std::ptr::null()) as vk::HINSTANCE)
            .surface(window as vk::wl_surface)
        , None)?)
    }
}