use std::sync::Arc;
use anyhow::Result;
use ash::{self, vk};

use crate::{vulkan::internal_managers::resource_handle::{ResourceHandle, ResourceType}, ISampler, Vulkan};

use super::{cobra::SAMPLER_BINDING, CobraVulkan};

pub struct SamplerVulkan {
    sampler: vk::Sampler,
    handle: ResourceHandle,

    cobra: Arc<CobraVulkan>
}

impl ISampler<Vulkan> for SamplerVulkan {
    fn handle(&self) -> u32 {
        self.handle.id
    }
}

impl SamplerVulkan {
    pub(crate) fn new(cobra: Arc<CobraVulkan>) -> Result<Self> 
        where Self:Sized, Self:Send, Self:Sync {
        unsafe {
            let sampler = cobra.device.create_sampler(&vk::SamplerCreateInfo::default()
                .mag_filter(vk::Filter::LINEAR)
                .min_filter(vk::Filter::LINEAR)
            , None)?;

            let handle = ResourceHandle::new(cobra.clone(), ResourceType::Sampler);
            cobra.device.update_descriptor_sets(&[vk::WriteDescriptorSet::default()
                .dst_set(cobra.bindless_set)
                .dst_binding(SAMPLER_BINDING)
                .dst_array_element(handle.id)
                .descriptor_count(1)
                .descriptor_type(vk::DescriptorType::SAMPLER)
                .image_info(&[vk::DescriptorImageInfo::default()
                    .sampler(sampler)
                ])
            ], &[]);

            Ok(SamplerVulkan {
                sampler, handle, cobra
            })
        }
    }
}

impl Drop for SamplerVulkan {
    fn drop(&mut self) {
        self.cobra.push(self.sampler);
    }
}