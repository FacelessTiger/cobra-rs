use anyhow::Result;
use glam::UVec2;

use crate::{Directx, IImage};

pub struct ImageDirectx;

impl IImage<Directx> for ImageDirectx {
    #[allow(unused)]
    fn set(&mut self, data: &[u8]) -> Result<()> {
        todo!()
    }

    fn handle(&self) -> Result<u32> {
        todo!()
    }

    fn size(&self) -> UVec2 {
        todo!()
    }
}