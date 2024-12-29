pub mod slang;
pub mod traits;
pub use traits::*;

#[cfg(feature="vulkan")]
pub mod vulkan;
#[cfg(feature="directx")]
pub mod directx;

pub use anyhow::{Result, Error};
pub use glam::*;

use paste::paste;

#[cfg(feature="vulkan")]
pub struct Vulkan;
#[cfg(feature="directx")]
pub struct Directx;

macro_rules! create_primitive {
    ($name:ident) => {
        paste! {
            pub trait [<$name Primitive>]<T>
                where Self::Inner: [<I $name>]<T>, T: CobraType<T> {
                type Inner;
            }

            #[cfg(feature="vulkan")]
            impl [<$name Primitive>]<Vulkan> for Vulkan { type Inner = vulkan::mappings::[<$name Vulkan>]; }
            #[cfg(feature="directx")]
            impl [<$name Primitive>]<Directx> for Directx { type Inner = directx::mappings::[<$name Directx>]; }
            pub type $name<T> = <T as [<$name Primitive>]<T>>::Inner;
        }
    };
}

pub trait CobraType<T>: CobraPrimitive<T> + 
    BufferPrimitive<T> + ImagePrimitive<T> + SamplerPrimitive<T> +
    CommandListPrimitive<T> + QueuePrimitive<T> + FencePrimitive<T> + 
    SwapchainPrimitive<T>
    where T: CobraType<T> { }
#[cfg(feature="vulkan")]
impl CobraType<Vulkan> for Vulkan { }
#[cfg(feature="directx")]
impl CobraType<Directx> for Directx { }

create_primitive!(Cobra);

create_primitive!(Buffer);
create_primitive!(Image);
create_primitive!(Sampler);

create_primitive!(CommandList);
create_primitive!(Queue);
create_primitive!(Fence);

create_primitive!(Swapchain);