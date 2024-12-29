use anyhow::Result;
use ash::vk::{self, Rect2D};
use glam::{IVec2, UVec2};

use crate::{vulkan::internal_managers::{pipeline_manager::GraphicsPipelineKey, utils}, BlendFactor, BlendOp, ClearValue, CompareOperation, ICommandList, ISwapchain, IndexType, PipelineStage, Vulkan};

use super::{image::ImageVulkan, swapchain::SwapchainVulkan, BufferVulkan, CobraVulkan};

pub struct CommandAllocator {
    pub(crate) command_pool: vk::CommandPool,
    pub(crate) available_command_lists: Vec<CommandListVulkan>,
}

pub struct CommandListVulkan {
    pub(crate) command_buffer: vk::CommandBuffer,
    pub(crate) allocator: *mut CommandAllocator,

    pub(crate) graphics_key: GraphicsPipelineKey,
    pub(crate) graphics_state_changed: bool,

    pub(crate) cobra: *const CobraVulkan
}

impl ICommandList<Vulkan> for CommandListVulkan {
    fn clear(&self, image: &mut ImageVulkan, color: impl Into<ClearValue>) {
        unsafe {
            let cobra = &*self.cobra;
            image.transition_layout(self.command_buffer, vk::ImageLayout::TRANSFER_DST_OPTIMAL);
            
            cobra.device.cmd_clear_color_image(self.command_buffer, image.allocation.0, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &match color.into() {
                ClearValue::Vec4(value) => vk::ClearColorValue { float32: value.to_array() },
                ClearValue::IVec4(value) => vk::ClearColorValue { int32: value.to_array() },
                ClearValue::UVec4(value) => vk::ClearColorValue { uint32: value.to_array() }
            }, &[vk::ImageSubresourceRange::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .level_count(1)
                .layer_count(1)
            ]);
        }
    }

    fn clear_color_attachment(&self, attachment: u32, color: impl Into<ClearValue>, size: impl Into<UVec2>) {
        unsafe {
            let cobra = &*self.cobra;
            let size = size.into();

            cobra.device.cmd_clear_attachments(self.command_buffer, &[vk::ClearAttachment::default()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .color_attachment(attachment)
                .clear_value(vk::ClearValue {
                    color: match color.into() {
                        ClearValue::Vec4(value) => vk::ClearColorValue { float32: value.to_array() },
                        ClearValue::IVec4(value) => vk::ClearColorValue { int32: value.to_array() },
                        ClearValue::UVec4(value) => vk::ClearColorValue { uint32: value.to_array() }
                    }
                })
            ], &[vk::ClearRect::default()
                .rect(Rect2D::default().extent(vk::Extent2D { width: size.x, height: size.y }))
                .layer_count(1)
            ]);
        }
    }

    fn clear_depth_attachment(&self, depth: f32, size: impl Into<UVec2>) {
        unsafe {
            let cobra = &*self.cobra;
            let size = size.into();

            cobra.device.cmd_clear_attachments(self.command_buffer, &[vk::ClearAttachment::default()
                .aspect_mask(vk::ImageAspectFlags::DEPTH)
                .clear_value(vk::ClearValue { depth_stencil: { vk::ClearDepthStencilValue { depth, stencil: 0 } } })
            ], &[vk::ClearRect::default()
                .rect(Rect2D::default().extent(vk::Extent2D { width: size.x, height: size.y }))
                .layer_count(1)
            ]);
        }
    }

    fn present(&self, swapchain: &mut SwapchainVulkan) {
        swapchain.current_image().transition_layout( self.command_buffer, vk::ImageLayout::PRESENT_SRC_KHR);
    }

    fn copy_buffer_region(&self, src: &BufferVulkan, dst: &BufferVulkan, size: u64, src_offset: u64, dst_offset: u64) {
        unsafe {
            let cobra = &*self.cobra;
            cobra.device.cmd_copy_buffer(self.command_buffer, src.allocation.0, dst.allocation.0, &[vk::BufferCopy::default()
                .src_offset(src_offset)
                .dst_offset(dst_offset)
                .size(size)
            ]);
        }
    }

    fn copy_buffer_to_image(&self, src: &BufferVulkan, dst: &ImageVulkan, src_offset: u64) {
        unsafe {
            let cobra = &*self.cobra;
            dst.transition_layout(self.command_buffer, vk::ImageLayout::TRANSFER_DST_OPTIMAL);

            cobra.device.cmd_copy_buffer_to_image(self.command_buffer, src.allocation.0, dst.allocation.0, vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[vk::BufferImageCopy::default()
                .buffer_offset(src_offset)
                .image_subresource(vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .layer_count(1)
                )
                .image_extent(vk::Extent3D { width: dst.size.x, height: dst.size.y, depth: 1 })
            ]);
        }
    }

    fn copy_image_to_buffer(&self, src: &mut ImageVulkan, dst: &BufferVulkan, dst_offset: u64) {
        unsafe {
            let cobra = &*self.cobra;
            src.transition_layout(self.command_buffer, vk::ImageLayout::TRANSFER_SRC_OPTIMAL);

            cobra.device.cmd_copy_image_to_buffer(self.command_buffer, src.allocation.0, vk::ImageLayout::TRANSFER_SRC_OPTIMAL, dst.allocation.0, &[vk::BufferImageCopy::default()
                .buffer_offset(dst_offset)
                .image_subresource(vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .layer_count(1)
                )
                .image_extent(vk::Extent3D { width: src.size.x, height: src.size.y, depth: 1 })
            ]);
        }
    }

    fn blit_image(&self, src: &mut ImageVulkan, dst: &mut ImageVulkan, src_size: Option<impl Into<UVec2>>) {
        unsafe {
            let cobra = &*self.cobra;
            src.transition_layout(self.command_buffer, vk::ImageLayout::TRANSFER_SRC_OPTIMAL);
            dst.transition_layout(self.command_buffer, vk::ImageLayout::TRANSFER_DST_OPTIMAL);

            let src_size = match src_size {
                Some(size) => size.into(),
                None => src.size
            };

            cobra.device.cmd_blit_image2(self.command_buffer, &vk::BlitImageInfo2::default()
                .src_image(src.allocation.0)
                .src_image_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                .dst_image(dst.allocation.0)
                .dst_image_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .regions(&[vk::ImageBlit2::default()
                    .src_subresource(vk::ImageSubresourceLayers::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .layer_count(1)
                    )
                    .src_offsets([vk::Offset3D::default(), vk::Offset3D { x: src_size.x as i32, y: src_size.y as i32, z: 1 }])
                    .dst_subresource(vk::ImageSubresourceLayers::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .layer_count(1)
                    )
                    .dst_offsets([vk::Offset3D::default(), vk::Offset3D { x: dst.size.x as i32, y: dst.size.y as i32, z: 1 }])
                ])
                .filter(vk::Filter::NEAREST)
            );
        }
    }

    fn begin_rendering<'a>(&mut self, region: impl Into<UVec2>, color_attachment: &mut ImageVulkan, depth_attachment: impl Into<Option<&'a mut ImageVulkan>>)
        where ImageVulkan: 'a {
        unsafe {
            let cobra = &*self.cobra;
            let region = region.into();

            self.graphics_state_changed = true;
            self.graphics_key.color_attachment = color_attachment.format;

            let mut depth_info = vk::RenderingAttachmentInfo::default();
            if let Some(image) = depth_attachment.into() {
                image.transition_layout(self.command_buffer, vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL);

                self.graphics_key.depth_attachment = image.format;
                depth_info = depth_info.image_view(image.view).image_layout(vk::ImageLayout::DEPTH_ATTACHMENT_OPTIMAL);
            }

            color_attachment.transition_layout(self.command_buffer, vk::ImageLayout::ATTACHMENT_OPTIMAL);
            cobra.device.cmd_begin_rendering(self.command_buffer, &vk::RenderingInfo::default()
                .render_area(Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: vk::Extent2D { width: region.x, height: region.y }
                })
                .layer_count(1)
                .color_attachments(&[vk::RenderingAttachmentInfo::default()
                    .image_view(color_attachment.view)
                    .image_layout(vk::ImageLayout::ATTACHMENT_OPTIMAL)
                ])
                .depth_attachment(&depth_info)
            );
        }
    }

    fn end_rendering(&self) {
        unsafe {
            let cobra = &*self.cobra;
            cobra.device.cmd_end_rendering(self.command_buffer);
        }
    }

    fn barrier(&self, src: PipelineStage, dst: PipelineStage) {
        unsafe {
            let cobra = &*self.cobra;
            cobra.device.cmd_pipeline_barrier2(self.command_buffer, &vk::DependencyInfo::default()
                .memory_barriers(&[vk::MemoryBarrier2::default()
                    .src_stage_mask(pipeline_stage_to_vulkan(src))
                    .src_access_mask(vk::AccessFlags2::MEMORY_READ | vk::AccessFlags2::MEMORY_WRITE)
                    .dst_stage_mask(pipeline_stage_to_vulkan(dst))
                    .dst_access_mask(vk::AccessFlags2::MEMORY_READ | vk::AccessFlags2::MEMORY_WRITE)
                ])
            );
        }
    }

    fn buffer_barrier(&self, buffer: &BufferVulkan, src: PipelineStage, dst: PipelineStage) {
        unsafe {
            let cobra = &*self.cobra;
            cobra.device.cmd_pipeline_barrier2(self.command_buffer, &vk::DependencyInfo::default()
                .buffer_memory_barriers(&[vk::BufferMemoryBarrier2::default()
                    .src_stage_mask(pipeline_stage_to_vulkan(src))
                    .src_access_mask(vk::AccessFlags2::MEMORY_READ | vk::AccessFlags2::MEMORY_WRITE)
                    .dst_stage_mask(pipeline_stage_to_vulkan(dst))
                    .dst_access_mask(vk::AccessFlags2::MEMORY_READ | vk::AccessFlags2::MEMORY_WRITE)
                    .buffer(buffer.allocation.0)
                    .size(vk::WHOLE_SIZE)
                ])
            );
        }
    }

    fn bind_shaders(&mut self, shaders: &[&'static [u8]]) {
        self.graphics_state_changed = true;
        for i in 0..shaders.len() {
            self.graphics_key.shaders[i] = Some(shaders[i]);
        }
    }

    fn bind_index_buffer(&self, buffer: &BufferVulkan, ty: IndexType, offset: u64) {
        unsafe {
            let cobra = &*self.cobra;
            cobra.device.cmd_bind_index_buffer(self.command_buffer, buffer.allocation.0, offset, match ty {
                IndexType::U16 => vk::IndexType::UINT16,
                IndexType::U32 => vk::IndexType::UINT32
            });
        }
    }

    fn push_constant<T>(&self, value: &T) {
        unsafe {
            let cobra = &*self.cobra;
            cobra.device.cmd_push_constants(self.command_buffer, cobra.bindless_pipeline_layout, vk::ShaderStageFlags::ALL, 0, core::slice::from_raw_parts(
                (value as *const T) as *const u8, 
                core::mem::size_of::<T>()
            ));
        }
    }

    fn set_default_state(&self) {
        unsafe {
            let cobra = &*self.cobra;
            cobra.device.cmd_set_depth_test_enable(self.command_buffer, false);
            cobra.device.cmd_set_depth_write_enable(self.command_buffer, false);
            cobra.device.cmd_set_depth_compare_op(self.command_buffer, vk::CompareOp::NEVER);
        }
    }

    fn set_viewport(&self, size: impl Into<IVec2>) {
        unsafe {
            let cobra = &*self.cobra;
            let size = size.into();

            let y_offset = if size.y < 0 { 0.0f32 } else { size.y as f32 };
            cobra.device.cmd_set_viewport_with_count(self.command_buffer, &[vk::Viewport::default()
                .x(0.0)
                .y(y_offset)
                .width(size.x as f32)
                .height(-size.y as f32)
                .min_depth(0.0)
                .max_depth(1.0)
            ]);
        }
    }

    fn set_scissor(&self, size: impl Into<UVec2>, offset: impl Into<IVec2>) {
        unsafe {
            let cobra = &*self.cobra;
            let size = size.into();
            let offset = offset.into();

            cobra.device.cmd_set_scissor_with_count(self.command_buffer, &[vk::Rect2D::default()
                .offset(vk::Offset2D { x: offset.x, y: offset.y })
                .extent(vk::Extent2D { width: size.x, height: size.y })
            ]);
        }
    }

    fn enable_color_blend(&mut self, src_blend: BlendFactor, dst_blend: BlendFactor, blend_op: BlendOp, src_blend_alpha: BlendFactor, dst_blend_alpha: BlendFactor, blend_alpha: BlendOp) {
        self.graphics_key.blend_enable = true;
        self.graphics_key.src_blend = src_blend;
        self.graphics_key.dst_blend = dst_blend;
        self.graphics_key.blend_op = blend_op;
        self.graphics_key.src_blend_alpha = src_blend_alpha;
        self.graphics_key.dst_blend_alpha = dst_blend_alpha;
        self.graphics_key.blend_alpha = blend_alpha;
        self.graphics_state_changed = true;
    }

    fn enable_depth_test(&self, write_enabled: bool, op: CompareOperation) {
        unsafe {
            let cobra = &*self.cobra;
            cobra.device.cmd_set_depth_test_enable(self.command_buffer, true);
            cobra.device.cmd_set_depth_write_enable(self.command_buffer, write_enabled);
            cobra.device.cmd_set_depth_compare_op(self.command_buffer, utils::compare_op_to_vulkan(op));
        }
    }

    fn draw(&self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) -> Result<()> {
        unsafe  {
            let cobra = &*self.cobra;
            self.bind_pipeline_if_needed()?;
            cobra.device.cmd_draw(self.command_buffer, vertex_count, instance_count, first_vertex, first_instance);

            Ok(())
        }
    }

    fn draw_indirect(&self, buffer: &BufferVulkan, offset: u64, draw_count: u32, stride: u32) -> Result<()> {
        unsafe {
            let cobra = &*self.cobra;
            self.bind_pipeline_if_needed()?;
            cobra.device.cmd_draw_indirect(self.command_buffer, buffer.allocation.0, offset, draw_count, stride);

            Ok(())
        }
    }

    fn draw_indirect_count(&self, buffer: &BufferVulkan, offset: u64, count_buffer: &BufferVulkan, count_buffer_offset: u64, max_draw_count: u32, stride: u32) -> Result<()> {
        unsafe {
            let cobra = &*self.cobra;
            self.bind_pipeline_if_needed()?;
            cobra.device.cmd_draw_indirect_count(self.command_buffer, buffer.allocation.0, offset, count_buffer.allocation.0, count_buffer_offset, max_draw_count, stride);

            Ok(())
        }
    }

    fn draw_indexed(&self, index_count: u32, instance_count: u32, first_index: u32, vertex_offset: i32, first_instance: u32) -> Result<()> {
        unsafe  {
            let cobra = &*self.cobra;
            self.bind_pipeline_if_needed()?;
            cobra.device.cmd_draw_indexed(self.command_buffer, index_count, instance_count, first_index, vertex_offset, first_instance);

            Ok(())
        }
    }

    fn draw_indexed_indirect(&self, buffer: &BufferVulkan, offset: u64, draw_count: u32, stride: u32) -> Result<()> {
        unsafe {
            let cobra = &*self.cobra;
            self.bind_pipeline_if_needed()?;
            cobra.device.cmd_draw_indexed_indirect(self.command_buffer, buffer.allocation.0, offset, draw_count, stride);

            Ok(())
        }
    }

    fn draw_indexed_indirect_count(&self, buffer: &BufferVulkan, offset: u64, count_buffer: &BufferVulkan, count_buffer_offset: u64, max_draw_count: u32, stride: u32) -> Result<()> {
        unsafe {
            let cobra = &*self.cobra;
            self.bind_pipeline_if_needed()?;
            cobra.device.cmd_draw_indexed_indirect_count(self.command_buffer, buffer.allocation.0, offset, count_buffer.allocation.0, count_buffer_offset, max_draw_count, stride);

            Ok(())
        }
    }

    fn dispatch(&self, work_x: u32, work_y: u32, work_z: u32) {
        unsafe {
            let cobra = &*self.cobra;
            cobra.device.cmd_dispatch(self.command_buffer, work_x, work_y, work_z);
        }
    }

    fn dispatch_indirect(&self, buffer: &BufferVulkan, offset: u64) {
        unsafe {
            let cobra = &*self.cobra;
            cobra.device.cmd_dispatch_indirect(self.command_buffer, buffer.allocation.0, offset);
        }
    }
}

impl CommandListVulkan {
    pub(crate) fn new(cobra: *const CobraVulkan, command_buffer: vk::CommandBuffer, allocator: *mut CommandAllocator) -> CommandListVulkan {
        CommandListVulkan {
            cobra, command_buffer, allocator,
            graphics_key: GraphicsPipelineKey::new(), graphics_state_changed: false
        }
    }

    fn bind_pipeline_if_needed(&self) -> Result<()> {
        let cobra = unsafe { &*self.cobra };
        if !self.graphics_state_changed { return Ok(()); }

        cobra.bind_graphics_pipeline(self.command_buffer, self.graphics_key)?;
        Ok(())
    }
}

fn pipeline_stage_to_vulkan(stages: PipelineStage) -> vk::PipelineStageFlags2 {
    let mut ret = vk::PipelineStageFlags2::empty();
    for stage in stages {
        ret |= match stage {
            PipelineStage::Compute => vk::PipelineStageFlags2::COMPUTE_SHADER,
            PipelineStage::Transfer => vk::PipelineStageFlags2::ALL_TRANSFER,
            PipelineStage::Graphics => vk::PipelineStageFlags2::ALL_GRAPHICS,
            _ => unreachable!()
        };
    }

    ret
}