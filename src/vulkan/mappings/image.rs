use anyhow::{Error, Result};
use ash::vk;
use glam::UVec2;
use vk_mem::Alloc;
use std::sync::{atomic::{AtomicI32, Ordering}, Arc};

use crate::{vulkan::internal_managers::{resource_handle::{ResourceHandle, ResourceType}, utils}, IBuffer, ICommandList, IImage, IQueue, ImageFormat, ImageUsage, Vulkan};

use super::{cobra::{SAMPLED_IMAGE_BINDING, STORAGE_IMAGE_BINDING}, CobraVulkan};

pub struct ImageVulkan {
   pub(crate) allocation: (vk::Image, Option<vk_mem::Allocation>),
   pub(crate) view: vk::ImageView,
   pub(crate) layout: AtomicI32,
   pub(crate) format: ImageFormat,
   pub(crate) size: UVec2,
   handle: Option<ResourceHandle>,

   cobra: Arc<CobraVulkan>
}

impl IImage<Vulkan> for ImageVulkan {
   fn set(&mut self, data: &[u8]) -> Result<()> {
      let staging_buffer = self.cobra.staging_buffer.read().unwrap();
      staging_buffer.as_ref().unwrap().host_slice()[0..data.len()].copy_from_slice(data);
      
      // TODO: use staging queue if available
      let cmd = self.cobra.graphics_queue.begin()?;
      cmd.copy_buffer_to_image(
         staging_buffer.as_ref().unwrap(),
         self, 0);
      self.cobra.graphics_queue.submit(cmd, None)?.wait()?;

      Ok(())
   }

   fn handle(&self) -> Result<u32> {
      match &self.handle {
         Some(handle) => Ok(handle.id),
         None => Err(Error::msg("Tried to get handle from an image with without Storage or Sampled usage"))
      }
   }

   fn size(&self) -> UVec2 {
       self.size
   }
}

impl ImageVulkan {
   pub(crate) fn new(cobra: Arc<CobraVulkan>, size: impl Into<UVec2>, format: ImageFormat, usage: ImageUsage) -> Result<Self> {
      unsafe {
         let size: UVec2 = size.into();

         let vulkan_format = utils::image_format_to_vulkan(format);
         let mut allocation_info = vk_mem::AllocationCreateInfo::default();
         allocation_info.usage = vk_mem::MemoryUsage::AutoPreferDevice;
         allocation_info.required_flags = vk::MemoryPropertyFlags::DEVICE_LOCAL;
         
         let allocation = cobra.allocator.create_image(&vk::ImageCreateInfo::default()
            .image_type(vk::ImageType::TYPE_2D)
            
            .format(vulkan_format)
            .extent(vk::Extent3D { width: size.x, height: size.y, depth: 1 })
            
            .mip_levels(1)
            .array_layers(1)
            .samples(vk::SampleCountFlags::TYPE_1)

            .tiling(vk::ImageTiling::OPTIMAL)
            .usage(utils::image_usage_to_vulkan(usage))
         , &allocation_info)?;
         let allocation = (allocation.0, Some(allocation.1));

         let view = cobra.device.create_image_view(&vk::ImageViewCreateInfo::default()
            .image(allocation.0)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(vulkan_format)
            .subresource_range(vk::ImageSubresourceRange::default()
               .aspect_mask(match usage.contains(ImageUsage::DepthStencilAttachment) {
                  true => vk::ImageAspectFlags::DEPTH,
                  false => vk::ImageAspectFlags::COLOR
               })
               .level_count(1)
               .layer_count(1)
            )
         , None)?;

         // Update descriptor
         let mut handle = None;
         if usage.contains(ImageUsage::Storage) {
            handle.get_or_insert(ResourceHandle::new(cobra.clone(), ResourceType::Image));

            cobra.device.update_descriptor_sets(&[vk::WriteDescriptorSet::default()
               .dst_set(cobra.bindless_set)
               .dst_binding(STORAGE_IMAGE_BINDING)
               .dst_array_element(handle.as_ref().unwrap().id)
               .descriptor_count(1)
               .descriptor_type(vk::DescriptorType::STORAGE_IMAGE)
               .image_info(&[vk::DescriptorImageInfo::default()
                  .image_view(view)
                  .image_layout(vk::ImageLayout::GENERAL) // Todo??
               ])
            ], &[]);
         }

         if usage.contains(ImageUsage::Sampled) {
            handle.get_or_insert(ResourceHandle::new(cobra.clone(), ResourceType::Image));

            cobra.device.update_descriptor_sets(&[vk::WriteDescriptorSet::default()
               .dst_set(cobra.bindless_set)
               .dst_binding(SAMPLED_IMAGE_BINDING)
               .dst_array_element(handle.as_ref().unwrap().id)
               .descriptor_count(1)
               .descriptor_type(vk::DescriptorType::SAMPLED_IMAGE)
               .image_info(&[vk::DescriptorImageInfo::default()
                  .image_view(view)
                  .image_layout(vk::ImageLayout::READ_ONLY_OPTIMAL)
               ])
            ], &[]);
         }

         // return
         Ok(ImageVulkan {
            allocation, view, format, handle, size, cobra,
            layout: AtomicI32::new(vk::ImageLayout::UNDEFINED.as_raw())
         })
      }
   }

   pub(crate) fn new_swapchain_image(cobra: Arc<CobraVulkan>, image: vk::Image, view: vk::ImageView, format: ImageFormat, size: UVec2) -> ImageVulkan {
      ImageVulkan {
         view, format, size, cobra,
         allocation: (image, None),
         layout: AtomicI32::new(vk::ImageLayout::UNDEFINED.as_raw()),
         handle: None
      }
   }

   pub(crate) fn transition_layout(&self, cmd: vk::CommandBuffer, new_layout: vk::ImageLayout) {
      unsafe {
         let layout = vk::ImageLayout::from_raw(self.layout.load(Ordering::SeqCst));
         if layout == new_layout { return; }

         self.cobra.device.cmd_pipeline_barrier2(cmd, &vk::DependencyInfo::default()
            .image_memory_barriers(&[vk::ImageMemoryBarrier2::default()
               .src_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
               .src_access_mask(vk::AccessFlags2::MEMORY_READ | vk::AccessFlags2::MEMORY_WRITE)
               .dst_stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
               .dst_access_mask(vk::AccessFlags2::MEMORY_READ | vk::AccessFlags2::MEMORY_WRITE)

               .old_layout(layout)
               .new_layout(new_layout)

               .image(self.allocation.0)
               .subresource_range(vk::ImageSubresourceRange::default()
                  .aspect_mask(match new_layout == vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL {  
                     true => vk::ImageAspectFlags::DEPTH,
                     false => vk::ImageAspectFlags::COLOR
                  })
                  .level_count(1)
                  .layer_count(1)
               )
            ])
         );

         self.layout.store(new_layout.as_raw(), Ordering::SeqCst);
      }
   }
}

impl Drop for ImageVulkan {
   fn drop(&mut self) {
      self.cobra.push(self.view);
      if self.allocation.1.is_some() {
         self.cobra.push((self.allocation.0, self.allocation.1.unwrap()));
      }
   }
}