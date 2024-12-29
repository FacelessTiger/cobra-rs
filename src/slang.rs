use crate::{CobraType, IImage, ISampler, Image, Sampler};

#[repr(transparent)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Ptr<T> {
    _phantom: std::marker::PhantomData<T>
}

#[repr(transparent)]
#[derive(Clone, Copy, Default, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ImageHandle<T> {
    _handle: u32,
    _phantom: std::marker::PhantomData<T>
}

impl<T> ImageHandle<T> 
    where T: CobraType<T> {
    pub fn new_storage(image: &Image<T>) -> ImageHandle<T> {
        Self::new(image.handle().unwrap())
    }

    pub fn new_sampled<C>(image: &Image<T>, sampler: &Sampler<T>) -> ImageHandle<T> {
        Self::new(image.handle().unwrap() | (sampler.handle() << 20))
    }

    pub fn new_storage_from_handle(handle: u32) -> ImageHandle<T> {
        Self::new(handle)
    }

    fn new(handle: u32) -> ImageHandle<T> {
        ImageHandle {
            _handle: handle,
            _phantom: std::marker::PhantomData
        }
    }
}

#[allow(non_camel_case_types)]
pub type float2 = glam::Vec2;
#[allow(non_camel_case_types)]
pub type float3 = glam::Vec3;
#[allow(non_camel_case_types)]
pub type float4 = glam::Vec4;

#[allow(non_camel_case_types)]
pub type uint32_t = u32;

#[allow(non_camel_case_types)]
pub type float4x4 = glam::Mat4;

pub use slang_struct::slang_struct;
pub use cps;

#[macro_export]
#[cps::cps]
macro_rules! slang_include {
    ($source:literal) =>
    let $($lisp_source:tt)* = cps::include!($source) in
    {
        slang_struct!($($lisp_source)*);
    }
}