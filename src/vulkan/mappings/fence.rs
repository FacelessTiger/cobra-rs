use std::sync::atomic::{AtomicU64, Ordering};
use anyhow::Result;
use ash::vk;

use crate::{IFence, Vulkan};

use super::CobraVulkan;

pub struct FenceVulkan {
    pub(crate) timeline_semaphore: vk::Semaphore,
    last_seen_value: AtomicU64,

    cobra: *const CobraVulkan
}

impl IFence<Vulkan> for FenceVulkan {
    fn wait(&self, value: Option<u64>) -> Result<()> {
        let last_seen = self.last_seen_value.load(Ordering::SeqCst);
        let value = match value {
            Some(v) => v,
            None => last_seen
        };
        if last_seen >= value { return Ok(()); }

        unsafe {
            (*self.cobra).device.wait_semaphores(&vk::SemaphoreWaitInfo::default()
                .semaphores(&[self.timeline_semaphore])
                .values(&[value])
            , u64::MAX)?;
        }

        self.last_seen_value.store(value, Ordering::SeqCst);
        Ok(())
    }

    fn pending_value(&self) -> u64 {
        unsafe {
            (*self.cobra).timeline_value.load(Ordering::SeqCst)
        }
    }

    fn current_value(&self) -> Result<u64> {
        let cobra = unsafe { &*self.cobra };
        let last_seen = self.last_seen_value.load(Ordering::SeqCst);
        if last_seen >= cobra.timeline_value.load(Ordering::SeqCst) { return Ok(last_seen); }

        unsafe {
            let last_seen = cobra.device.get_semaphore_counter_value(self.timeline_semaphore)?;
            self.last_seen_value.store(last_seen, Ordering::SeqCst);
            Ok(last_seen)
        }
    }
}

impl FenceVulkan {
    pub(crate) fn new() -> FenceVulkan {
        FenceVulkan {
            timeline_semaphore: vk::Semaphore::null(),
            last_seen_value: AtomicU64::new(0),
            cobra: std::ptr::null()
        }
    }

    pub(crate) fn init(&mut self, cobra: *const CobraVulkan) -> Result<()> {
        unsafe {
            self.timeline_semaphore = (*cobra).device.create_semaphore(&vk::SemaphoreCreateInfo::default()
                .push_next(&mut vk::SemaphoreTypeCreateInfo::default()
                    .semaphore_type(vk::SemaphoreType::TIMELINE)
                    .initial_value(0)
                )
            , None)?;
            self.cobra = cobra;

            Ok(())
        }
    }
}