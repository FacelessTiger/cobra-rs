use anyhow::Result;

use crate::{Directx, IBuffer, Queue};
use std::ffi::c_void;

pub struct BufferDirectx;

impl IBuffer<Directx> for BufferDirectx {
    #[allow(unused)]
    fn set<U>(&mut self, queue: &mut Queue<Directx>, source: &[U], buffer_offset: u64) -> Result<()>
        where U: Copy {
        todo!()
    }

    fn host_address(&self) -> *mut c_void {
        todo!()
    }

    fn device_address(&self) -> u64 {
        todo!()
    }

    fn host_slice<U>(&self) -> &mut [U] {
        todo!()
    }

    fn size(&self) -> u64 {
        todo!()
    }
}