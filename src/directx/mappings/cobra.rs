use std::sync::Arc;

use anyhow::Result;
use glam::UVec2;

use crate::{Buffer, BufferFlags, Directx, ICobra, Image, ImageFormat, ImageUsage, Queue, QueueType, Sampler, Swapchain};
use std::ffi::c_void;

pub struct CobraDirectx;

impl ICobra<Directx> for CobraDirectx {
    fn new() -> Result<Arc<Self>> {
        todo!()
    }

    #[allow(unused)]
    fn new_buffer(&self, cobra: Arc<Self>, size: u64, flags: BufferFlags) -> Result<Buffer<Directx>> {
        todo!()
    }

    #[allow(unused)]
    fn new_image(&self, cobra: Arc<Self>, size: impl Into<UVec2>, format: ImageFormat, usage: ImageUsage) -> Result<Image<Directx>> {
        todo!()
    }

    #[allow(unused)]
    fn new_sampler(&self, cobra: Arc<Self>) -> Result<Sampler<Directx>> {
        todo!()
    }

    #[allow(unused)]
    fn new_swapchain(&self, cobra: Arc<Self>, window: *mut c_void, size: UVec2) -> Result<Swapchain<Directx>> {
        todo!()
    }

    #[allow(unused)]
    fn queue(&self, ty: QueueType) -> &Queue<Directx> {
        todo!()
    }

    fn supports_resizable_bar(&self) -> bool {
        todo!()
    }
}