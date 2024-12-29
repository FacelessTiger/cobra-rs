use std::{ops::Deref, sync::RwLock};

use ash::vk;

use crate::{BlendFactor, BlendOp, CompareOperation, ImageFormat, ImageUsage};

// Converters
pub(crate) fn image_format_to_vulkan(format: ImageFormat) -> vk::Format {
    match format {
       ImageFormat::Unknown => vk::Format::UNDEFINED,
       ImageFormat::R32Sint => vk::Format::R32_SINT,
       ImageFormat::R16G16B16A16Sfloat => vk::Format::R16G16B16A16_SFLOAT,
       ImageFormat::R16G16B16A16Unorm => vk::Format::R16G16B16A16_UNORM,
       ImageFormat::R8G8B8A8Unorm => vk::Format::R8G8B8A8_UNORM,
       ImageFormat::B8G8R8A8Srgb => vk::Format::B8G8R8A8_SRGB,
       ImageFormat::D32SFloat => vk::Format::D32_SFLOAT
    }
 }
 
 pub(crate) fn image_usage_to_vulkan(usages: ImageUsage) -> vk::ImageUsageFlags {
   let mut ret = vk::ImageUsageFlags::empty();
   for usage in usages {
      ret |= match usage {
         ImageUsage::None => vk::ImageUsageFlags::empty(),
         ImageUsage::ColorAttachment => vk::ImageUsageFlags::COLOR_ATTACHMENT,
         ImageUsage::DepthStencilAttachment => vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
         ImageUsage::TransferSrc => vk::ImageUsageFlags::TRANSFER_SRC,
         ImageUsage::TransferDst => vk::ImageUsageFlags::TRANSFER_DST,
         ImageUsage::Storage => vk::ImageUsageFlags::STORAGE,
         ImageUsage::Sampled => vk::ImageUsageFlags::SAMPLED,
         _ => unreachable!()
      };
   }

   ret
}

pub(crate) fn blend_factor_to_vulkan(blend_factor: BlendFactor) -> vk::BlendFactor {
   match blend_factor {
      BlendFactor::Zero => vk::BlendFactor::ZERO,
      BlendFactor::One => vk::BlendFactor::ONE,
      BlendFactor::SrcAlpha => vk::BlendFactor::SRC_ALPHA,
      BlendFactor::DstAlpha => vk::BlendFactor::DST_ALPHA,
      BlendFactor::OneMinusSrcAlpha => vk::BlendFactor::ONE_MINUS_SRC_ALPHA
   }
}

pub(crate) fn blend_op_to_vulkan(blend_op: BlendOp) -> vk::BlendOp {
   match blend_op {
       BlendOp::Add => vk::BlendOp::ADD
   }
}

pub(crate) fn compare_op_to_vulkan(compare_op: CompareOperation) -> vk::CompareOp {
   match compare_op {
      CompareOperation::None => vk::CompareOp::NEVER,
      CompareOperation::Greater => vk::CompareOp::GREATER,
      CompareOperation::GreaterEqual => vk::CompareOp::GREATER_OR_EQUAL,
      CompareOperation::LesserEqual => vk::CompareOp::LESS_OR_EQUAL
   }
}

// Thread safe wrappers over pointer types
pub(crate) struct AllocationInfo(RwLock<vk_mem::AllocationInfo>);

impl AllocationInfo {
   pub fn new(allocation_info: vk_mem::AllocationInfo) -> Self {
      AllocationInfo(RwLock::new(allocation_info))
   }
}

impl Deref for AllocationInfo {
   type Target = RwLock<vk_mem::AllocationInfo>;

   fn deref(&self) -> &Self::Target {
       &self.0
   }
}

unsafe impl Send for AllocationInfo { }
unsafe impl Sync for AllocationInfo { }