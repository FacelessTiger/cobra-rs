use glam::UVec2;

use crate::{Directx, ISwapchain, Image};

pub struct SwapchainDirectx;

impl ISwapchain<Directx> for SwapchainDirectx {
    fn current_image(&mut self) -> &mut Image<Directx> {
        todo!()
    }

    #[allow(unused)]
    fn resize(&mut self, size: UVec2) {
        todo!()
    }

    fn size(&self) -> UVec2 {
        todo!()
    }
}