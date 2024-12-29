use std::{ffi::c_void, sync::Arc};

use anyhow::Result;
use glam::{IVec2, IVec4, UVec2, UVec4, Vec4};

use crate::{Buffer, CobraType, CommandList, Fence, Image, ImagePrimitive, Queue, Sampler, Swapchain};

// Buffer info
pub enum BufferFlags {
    Default, // device_local
    Upload, // host_local | host_visible | host_coherent (pref host_cached)
    Readback, // host_local | host_visible | host_coherent | host_cached
    DeviceUpload // device_local | host_visible | host_coherent
}

// Image info
#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum ImageFormat {
   Unknown,
   R32Sint,
   R16G16B16A16Sfloat,
   R16G16B16A16Unorm,
   R8G8B8A8Unorm,
   B8G8R8A8Srgb,
   D32SFloat
}

bitflags::bitflags! {
   #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct ImageUsage: u32 {
      const None = 0;
      const ColorAttachment = 1;
      const DepthStencilAttachment = 2;
      const TransferSrc = 4;
      const TransferDst = 8;
      const Storage = 16;
      const Sampled = 32;
    }
}

// Shader info
bitflags::bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct ShaderStage: u32 {
        const Vertex = 1;
        const Pixel = 2;
    }
}

// CommandList info
pub enum ClearValue {
    Vec4(Vec4),
    IVec4(IVec4),
    UVec4(UVec4)
}

impl From<Vec4> for ClearValue {
    fn from(value: Vec4) -> Self {
        ClearValue::Vec4(value)
    }
}

impl From<IVec4> for ClearValue {
    fn from(value: IVec4) -> Self {
        ClearValue::IVec4(value)
    }
}

impl From<UVec4> for ClearValue {
    fn from(value: UVec4) -> Self {
        ClearValue::UVec4(value)
    }
}

pub enum CompareOperation {
    None,
    Greater,
    GreaterEqual,
    LesserEqual
}

pub enum IndexType {
    U16,
    U32
}

bitflags::bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq)]
    pub struct PipelineStage: u32 {
        const None = 0;
        const Compute = 1;
        const Transfer = 2;
        const Graphics = 4;
        const All = PipelineStage::Compute.union(PipelineStage::Transfer).union(PipelineStage::Graphics).bits();
    }
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum BlendFactor {
    Zero,
    One,
    SrcAlpha,
    DstAlpha,
    OneMinusSrcAlpha
}

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub enum BlendOp {
    Add
}

// Queue info
pub enum QueueType {
    Graphics
}

// Traits
// Context
pub trait ICobra<T>:
    where T: CobraType<T>, Self:Sized, Self:Send, Self:Sync {

    fn new() -> Result<Arc<Self>>;
    fn new_buffer(&self, cobra: Arc<Self>, size: u64, flags: BufferFlags) -> Result<Buffer<T>>;
    fn new_image(&self, cobra: Arc<Self>, size: impl Into<UVec2>, format: ImageFormat, usage: ImageUsage) -> Result<Image<T>>;
    fn new_sampler(&self, cobra: Arc<Self>) -> Result<Sampler<T>>;
    fn new_swapchain(&self, cobra: Arc<Self>, window: *mut c_void, size: UVec2) -> Result<Swapchain<T>>;

    fn queue(&self, ty: QueueType) -> &Queue<T>;

    fn supports_resizable_bar(&self) -> bool;
}

// Resources
pub trait IBuffer<T> 
    where T: CobraType<T>, Self:Sized, Self:Send, Self:Sync {
    // TODO: maybe allow batching copies together?
    fn set<U>(&mut self, queue: &mut Queue<T>, source: &[U], buffer_offset: u64) -> Result<()>
        where U: Copy;

    fn host_address(&self) -> *mut c_void;
    fn device_address(&self) -> u64;

    fn host_slice<U>(&self) -> &mut [U];
    fn size(&self) -> u64;
}

pub trait IImage<T> 
    where T: CobraType<T>, Self:Sized, Self:Send, Self:Sync {
    fn set(&mut self, data: &[u8]) -> Result<()>;

    fn handle(&self) -> Result<u32>;
    fn size(&self) -> UVec2;
}

pub trait ISampler<T>
    where T: CobraType<T>, Self:Sized, Self:Send, Self:Sync {
    fn handle(&self) -> u32;
}

// Commands and execution
pub trait ICommandList<T>
    where T: CobraType<T> {
    fn clear(&self, image: &mut Image<T>, color: impl Into<ClearValue>);
    fn clear_color_attachment(&self, attachment: u32, color: impl Into<ClearValue>, size: impl Into<UVec2>);
    fn clear_depth_attachment(&self, depth: f32, size: impl Into<UVec2>);
    fn present(&self, swapchain: &mut Swapchain<T>);

    fn copy_buffer_region(&self, src: &Buffer<T>, dst: &Buffer<T>, size: u64, src_offset: u64, dst_offset: u64);
    fn copy_buffer_to_image(&self, src: &Buffer<T>, dst: &Image<T>, src_offset: u64);
    fn copy_image_to_buffer(&self, src: &mut Image<T>, dst: &Buffer<T>, dst_offset: u64);
    fn blit_image(&self, src: &mut Image<T>, dst: &mut Image<T>, src_size: Option<impl Into<UVec2>>);

    fn begin_rendering<'a>(&mut self, region: impl Into<UVec2>, color_attachment: &mut Image<T>, depth_attachment: impl Into<Option<&'a mut Image<T>>>)
        where <T as ImagePrimitive<T>>::Inner: 'a;
    fn end_rendering(&self);
    fn barrier(&self, src: PipelineStage, dst: PipelineStage);
    fn buffer_barrier(&self, buffer: &Buffer<T>, src: PipelineStage, dst: PipelineStage);
    fn push_constant<U>(&self, value: &U);

    fn bind_shaders(&mut self, shaders: &[&'static [u8]]);
    fn bind_index_buffer(&self, buffer: &Buffer<T>, ty: IndexType, offset: u64);

    fn set_default_state(&self);
    fn set_viewport(&self, size: impl Into<IVec2>);
    fn set_scissor(&self, size: impl Into<UVec2>, offset: impl Into<IVec2>);
    fn enable_color_blend(&mut self, src_blend: BlendFactor, dst_blend: BlendFactor, blend_op: BlendOp, src_blend_alpha: BlendFactor, dst_blend_alpha: BlendFactor, blend_alpha: BlendOp);
    fn enable_depth_test(&self, write_enabled: bool, op: CompareOperation);

    fn draw(&self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) -> Result<()>;
    fn draw_indirect(&self, buffer: &Buffer<T>, offset: u64, draw_count: u32, stride: u32) -> Result<()>;
    fn draw_indirect_count(&self, buffer: &Buffer<T>, offset: u64, count_buffer: &Buffer<T>, count_buffer_offset: u64, max_draw_count: u32, stride: u32) -> Result<()>;

    fn draw_indexed(&self, index_count: u32, instance_count: u32, first_index: u32, vertex_offset: i32, first_instance: u32) -> Result<()>;
    fn draw_indexed_indirect(&self, buffer: &Buffer<T>, offset: u64, draw_count: u32, stride: u32) -> Result<()>;
    fn draw_indexed_indirect_count(&self, buffer: &Buffer<T>, offset: u64, count_buffer: &Buffer<T>, count_buffer_offset: u64, max_draw_count: u32, stride: u32) -> Result<()>;

    fn dispatch(&self, work_x: u32, work_y: u32, work_z: u32);
    fn dispatch_indirect(&self, buffer: &Buffer<T>, offset: u64);
}

pub trait IQueue<T> 
    where T: CobraType<T>, Self:Sized {
    fn acquire(&self, swapchain: &mut Swapchain<T>) -> Result<Option<SyncPoint<T>>>;
    fn present(&self, swapchain: &mut Swapchain<T>, wait: Option<&mut SyncPoint<T>>) -> Result<()>;

    fn begin(&self) -> Result<CommandList<T>>;
    fn submit(&self, cmd: CommandList<T>, wait: Option<&mut SyncPoint<T>>) -> Result<SyncPoint<T>>;
}

pub trait IFence<T>: 'static 
    where T: CobraType<T> {
    fn wait(&self, value: Option<u64>) -> Result<()>;

    fn pending_value(&self) -> u64;
    fn current_value(&self) -> Result<u64>;
}

// Output
pub trait ISwapchain<T> 
    where T: CobraType<T>, Self:Sized, Self:Send, Self:Sync {
    fn current_image(&mut self) -> &mut Image<T>;
    fn resize(&mut self, size: UVec2);
    
    fn size(&self) -> UVec2;
}

// SyncPoint
pub struct SyncPoint<T>
    where T: CobraType<T> {
    pub fence: Option<*const Fence<T>>,
    pub value: Option<u64>
}

impl<T> SyncPoint<T> 
    where T: CobraType<T> {
    pub fn new() -> SyncPoint<T> {
        SyncPoint {
            fence: None,
            value: None
        }
    }

    pub fn new_from_fence(fence: &Fence<T>) -> SyncPoint<T> {
        SyncPoint {
            fence: Some(fence as *const _),
            value: Some(fence.pending_value())
        }
    }

    pub fn wait(&mut self) -> Result<()> {
        if let Some(fence) = self.fence {
            unsafe { (*fence).wait(self.value)?; }
        }

        Ok(())
    }

    pub fn value(&mut self) -> u64 {
        match self.value.is_none() && !self.fence.is_some() {
            true => unsafe { (*self.fence.unwrap()).pending_value() },
            false => self.value.unwrap()
        }
    }
}