use crate::{Directx, ISampler};

pub struct SamplerDirectx;

impl ISampler<Directx> for SamplerDirectx {
    fn handle(&self) -> u32 {
        todo!()
    }
}