use anyhow::Result;

use crate::{Directx, IFence};

pub struct FenceDirectx;

impl IFence<Directx> for FenceDirectx {
    #[allow(unused)]
    fn wait(&self, value: Option<u64>) -> Result<()> {
        todo!()
    }

    fn pending_value(&self) -> u64 {
        todo!()
    }

    fn current_value(&self) -> Result<u64> {
        todo!()
    }
}