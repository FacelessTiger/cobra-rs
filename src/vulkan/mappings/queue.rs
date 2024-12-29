use std::sync::Mutex;
use std::{collections::VecDeque, alloc::Layout, alloc::alloc, alloc::dealloc};
use anyhow::Result;
use ash::vk;

use crate::{IFence, IQueue, SyncPoint, Vulkan};

use super::command_list::{CommandAllocator, CommandListVulkan};
use super::fence::FenceVulkan;
use super::swapchain::SwapchainVulkan;
use super::CobraVulkan;

const LAYOUT: Layout = Layout::new::<CommandAllocator>();

pub struct QueueVulkan {
    queue: vk::Queue,
    queue_family: u32,
    fence: FenceVulkan,

    allocators: Mutex<VecDeque<*mut CommandAllocator>>,
    pending_command_lists: Mutex<VecDeque<(CommandListVulkan, u64)>>,

    cobra: *const CobraVulkan
}
unsafe impl Send for QueueVulkan { }
unsafe impl Sync for QueueVulkan { }

impl IQueue<Vulkan> for QueueVulkan {
    fn acquire(&self, swapchain: &mut SwapchainVulkan) -> Result<Option<SyncPoint<Vulkan>>> {
        unsafe {
            let cobra = &*self.cobra;

            if swapchain.dirty { swapchain.recreate()? }
            match cobra.swapchain_device_fn.acquire_next_image(swapchain.swapchain, u64::MAX, swapchain.semaphores[swapchain.semaphore_index], vk::Fence::null()) {
                Ok((image_index, suboptimal)) => {
                    if suboptimal {
                        swapchain.dirty = true;
                        return Ok(None);
                    } else {
                        swapchain.image_index = image_index;
                    }
                },
                Err(err) => {
                    if err == vk::Result::ERROR_OUT_OF_DATE_KHR {
                        swapchain.dirty = true;
                        return Ok(None);
                    } else {
                        return Err(err.into());
                    }
                }
            }

            cobra.device.queue_submit2(self.queue, &[vk::SubmitInfo2::default()
                .wait_semaphore_infos(&[vk::SemaphoreSubmitInfo::default()
                    .semaphore(swapchain.semaphores[swapchain.semaphore_index])
                    .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
                ])
                .signal_semaphore_infos(&[vk::SemaphoreSubmitInfo::default()
                    .semaphore(self.fence.timeline_semaphore)
                    .value(cobra.advance())
                    .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
                ])
            ], vk::Fence::null())?;

            swapchain.semaphore_index = (swapchain.semaphore_index + 1) % swapchain.semaphores.len();
            Ok(Some(SyncPoint::new_from_fence(&self.fence)))
        }
    }

    fn present(&self, swapchain: &mut SwapchainVulkan, wait: Option<&mut SyncPoint<Vulkan>>) -> Result<()> {
        unsafe {
            let cobra = &*self.cobra;

            let mut wait_info = Vec::new();
            if let Some(sync) = wait {
                wait_info.push(vk::SemaphoreSubmitInfo::default()
                    .semaphore((*(sync.fence.unwrap() as *const FenceVulkan)).timeline_semaphore)
                    .value(sync.value())
                    .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
                );
            }

            cobra.device.queue_submit2(self.queue, &[vk::SubmitInfo2::default()
                .wait_semaphore_infos(&wait_info)
                .signal_semaphore_infos(&[vk::SemaphoreSubmitInfo::default()
                    .semaphore(swapchain.semaphores[swapchain.semaphore_index])
                    .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
                ])
            ], vk::Fence::null())?;

            let binary_wait = swapchain.semaphores[swapchain.semaphore_index];
            swapchain.semaphore_index = (swapchain.semaphore_index + 1) % swapchain.semaphores.len();

            match cobra.swapchain_device_fn.queue_present(self.queue, &vk::PresentInfoKHR::default()
                .wait_semaphores(&[binary_wait])
                .swapchains(&[swapchain.swapchain])
                .image_indices(&[swapchain.image_index])
            ) {
                Ok(suboptimal) => if suboptimal { swapchain.dirty = true; },
                Err(err) => {
                    if err == vk::Result::ERROR_OUT_OF_DATE_KHR { 
                        swapchain.dirty = true; 
                    } else {
                        return Err(err.into());
                    }
                } 
            }

            Ok(())
        }
    }

    fn submit(&self, cmd: CommandListVulkan, wait: Option<&mut SyncPoint<Vulkan>>) -> Result<SyncPoint<Vulkan>> {
        unsafe {
            let cobra = &*self.cobra;
            cobra.device.end_command_buffer(cmd.command_buffer)?;
            
            let mut wait_info = Vec::new();
            if let Some(sync) = wait {
                //sync.fence.unwrap() as *const Fence;
                wait_info.push(vk::SemaphoreSubmitInfo::default()
                    .semaphore((*(sync.fence.unwrap() as *const FenceVulkan)).timeline_semaphore)
                    .value(sync.value())
                    .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
                );
            }

            cobra.device.queue_submit2(self.queue, &[vk::SubmitInfo2::default()
                .command_buffer_infos(&[vk::CommandBufferSubmitInfo::default()
                    .command_buffer(cmd.command_buffer)
                ])
                .wait_semaphore_infos(&wait_info)
                .signal_semaphore_infos(&[vk::SemaphoreSubmitInfo::default()
                    .semaphore(self.fence.timeline_semaphore)
                    .value(cobra.advance())
                    .stage_mask(vk::PipelineStageFlags2::ALL_COMMANDS)
                ])
            ], vk::Fence::null())?;
            
            self.allocators.lock().unwrap().push_back(cmd.allocator);
            self.pending_command_lists.lock().unwrap().push_back((cmd, self.fence.pending_value()));

            Ok(SyncPoint::new_from_fence(&self.fence))
        }
    }

    fn begin(&self) -> Result<CommandListVulkan> {
        unsafe {
            let cobra = &*self.cobra;
            let current_value = self.fence.current_value()?;

            {
                let mut pending_command_list = self.pending_command_lists.lock().unwrap();
                while !pending_command_list.is_empty() {
                    let pending = pending_command_list.front().unwrap();
                    if current_value >= pending.1 {
                        cobra.device.reset_command_buffer(pending.0.command_buffer, vk::CommandBufferResetFlags::empty())?;

                        let pending = pending_command_list.pop_front().unwrap().0;
                        let allocator = pending.allocator;
                        (*allocator).available_command_lists.push(pending);
                    } else {
                        break;
                    }
                }
            }

            let allocator = self.acquire_command_allocator()?;
            let cmd = match (*allocator).available_command_lists.is_empty() {
                true => {
                    let cmd = cobra.device.allocate_command_buffers(&vk::CommandBufferAllocateInfo::default()
                        .command_pool((*allocator).command_pool)
                        .level(vk::CommandBufferLevel::PRIMARY)
                        .command_buffer_count(1)
                    )?[0];

                    CommandListVulkan::new(self.cobra, cmd, allocator)
                }
                false => (*allocator).available_command_lists.pop().unwrap()
            };

            cobra.device.begin_command_buffer(cmd.command_buffer, &vk::CommandBufferBeginInfo::default())?;
            cobra.device.cmd_bind_descriptor_sets(cmd.command_buffer, vk::PipelineBindPoint::GRAPHICS, cobra.bindless_pipeline_layout, 0, &[cobra.bindless_set], &[]);

            Ok(cmd)
        }
    }

}

impl QueueVulkan {
    pub(crate) fn new() -> QueueVulkan {
        QueueVulkan {
            queue: vk::Queue::null(),
            queue_family: 0,
            cobra: std::ptr::null(),
            fence: FenceVulkan::new(),

            allocators: Mutex::new(VecDeque::new()),
            pending_command_lists: Mutex::new(VecDeque::new())
        }
    }

    pub(crate) fn init(&mut self, cobra: *const CobraVulkan, queue: vk::Queue, queue_family: u32) -> Result<()> {
        self.fence.init(cobra)?;
        self.cobra = cobra;
        self.queue = queue;
        self.queue_family = queue_family;

        Ok(())
    }

    pub(crate) fn destroy(&mut self) {
        unsafe {
            let cobra = &*self.cobra;
            for allocator in self.allocators.lock().unwrap().iter() {
                cobra.push((**allocator).command_pool);

                allocator.drop_in_place();
                dealloc(*allocator as *mut u8, LAYOUT);
            }

            cobra.push(self.fence.timeline_semaphore);
        }
    }

    fn acquire_command_allocator(&self) -> Result<*mut CommandAllocator> {
        unsafe {
            let cobra = &*self.cobra;

            let mut allocators = self.allocators.lock().unwrap();
            Ok(match allocators.is_empty() {
                true => {
                    let ptr = alloc(LAYOUT) as *mut CommandAllocator;
                    std::ptr::write(ptr, CommandAllocator {
                        command_pool: cobra.device.create_command_pool(&vk::CommandPoolCreateInfo::default()
                            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                            .queue_family_index(self.queue_family)
                        , None)?,
                        available_command_lists: Vec::new()
                    });

                    ptr
                }
                false => allocators.pop_back().unwrap()
            })
        }
    }
}