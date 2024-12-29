use anyhow::Result;

use crate::{CommandList, Directx, IQueue, Swapchain, SyncPoint};

pub struct QueueDirectx;

impl IQueue<Directx> for QueueDirectx {
    #[allow(unused)]
    fn acquire(&self, swapchain: &mut Swapchain<Directx>) -> anyhow::Result<Option<SyncPoint<Directx>>> {
        todo!()
    }

    #[allow(unused)]
    fn present(&self, swapchain: &mut Swapchain<Directx>, wait: Option<&mut SyncPoint<Directx>>) -> Result<()> {
        todo!()
    }

    fn begin(&self) -> Result<CommandList<Directx>> {
        todo!()
    }

    #[allow(unused)]
    fn submit(&self, cmd: CommandList<Directx>, wait: Option<&mut SyncPoint<Directx>>) -> Result<SyncPoint<Directx>> {
        todo!()
    }
}