use anyhow::Result;
use glam::{IVec2, UVec2};

use crate::{BlendFactor, BlendOp, Buffer, ClearValue, CompareOperation, Directx, ICommandList, Image, ImagePrimitive, IndexType, PipelineStage, Swapchain};

pub struct CommandListDirectx;

impl ICommandList<Directx> for CommandListDirectx {
    #[allow(unused)]
    fn clear(&self, image: &mut Image<Directx>, color: impl Into<ClearValue>) {
        todo!()
    }

    #[allow(unused)]
    fn clear_color_attachment(&self, attachment: u32, color: impl Into<ClearValue>, size: impl Into<UVec2>) {
        todo!()
    }

    #[allow(unused)]
    fn clear_depth_attachment(&self, depth: f32, size: impl Into<UVec2>) {
        todo!()
    }

    #[allow(unused)]
    fn present(&self, swapchain: &mut Swapchain<Directx>) {
        todo!()
    }

    #[allow(unused)]
    fn copy_buffer_region(&self, src: &Buffer<Directx>, dst: &Buffer<Directx>, size: u64, src_offset: u64, dst_offset: u64) {
        todo!()
    }

    #[allow(unused)]
    fn copy_buffer_to_image(&self, src: &Buffer<Directx>, dst: &Image<Directx>, src_offset: u64) {
        todo!()
    }

    #[allow(unused)]
    fn copy_image_to_buffer(&self, src: &mut Image<Directx>, dst: &Buffer<Directx>, dst_offset: u64) {
        todo!()
    }

    #[allow(unused)]
    fn blit_image(&self, src: &mut Image<Directx>, dst: &mut Image<Directx>, src_size: Option<impl Into<UVec2>>) {
        todo!()
    }

    #[allow(unused)]
    fn begin_rendering<'a>(&mut self, region: impl Into<UVec2>, color_attachment: &mut Image<Directx>, depth_attachment: impl Into<Option<&'a mut Image<Directx>>>)
        where <Directx as ImagePrimitive<Directx>>::Inner: 'a {
        todo!()
    }

    #[allow(unused)]
    fn end_rendering(&self) {
        todo!()
    }
    
    #[allow(unused)]
    fn barrier(&self, src: PipelineStage, dst: PipelineStage) {
        todo!()
    }

    #[allow(unused)]
    fn buffer_barrier(&self, buffer: &Buffer<Directx>, src: PipelineStage, dst: PipelineStage) {
        todo!()
    }

    #[allow(unused)]
    fn push_constant<U>(&self, value: &U) {
        todo!()
    }

    #[allow(unused)]
    fn bind_shaders(&mut self, shaders: &[&'static [u8]]) {
        todo!()
    }

    #[allow(unused)]
    fn bind_index_buffer(&self, buffer: &Buffer<Directx>, ty: IndexType, offset: u64) {
        todo!()
    }

    #[allow(unused)]
    fn set_default_state(&self) {
        todo!()
    }

    #[allow(unused)]
    fn set_viewport(&self, size: impl Into<IVec2>) {
        todo!()
    }

    #[allow(unused)]
    fn set_scissor(&self, size: impl Into<UVec2>, offset: impl Into<IVec2>) {
        todo!()
    }

    #[allow(unused)]
    fn enable_color_blend(&mut self, src_blend: BlendFactor, dst_blend: BlendFactor, blend_op: BlendOp, src_blend_alpha: BlendFactor, dst_blend_alpha: BlendFactor, blend_alpha: BlendOp) {
        todo!()
    }

    #[allow(unused)]
    fn enable_depth_test(&self, write_enabled: bool, op: CompareOperation) {
        todo!()
    }

    #[allow(unused)]
    fn draw(&self, vertex_count: u32, instance_count: u32, first_vertex: u32, first_instance: u32) -> Result<()> {
        todo!()
    }

    #[allow(unused)]
    fn draw_indirect(&self, buffer: &Buffer<Directx>, offset: u64, draw_count: u32, stride: u32) -> Result<()> {
        todo!()
    }

    #[allow(unused)]
    fn draw_indirect_count(&self, buffer: &Buffer<Directx>, offset: u64, count_buffer: &Buffer<Directx>, count_buffer_offset: u64, max_draw_count: u32, stride: u32) -> Result<()> {
        todo!()
    }

    #[allow(unused)]
    fn draw_indexed(&self, index_count: u32, instance_count: u32, first_index: u32, vertex_offset: i32, first_instance: u32) -> Result<()> {
        todo!()
    }

    #[allow(unused)]
    fn draw_indexed_indirect(&self, buffer: &Buffer<Directx>, offset: u64, draw_count: u32, stride: u32) -> Result<()> {
        todo!()
    }

    #[allow(unused)]
    fn draw_indexed_indirect_count(&self, buffer: &Buffer<Directx>, offset: u64, count_buffer: &Buffer<Directx>, count_buffer_offset: u64, max_draw_count: u32, stride: u32) -> Result<()> {
        todo!()
    }

    #[allow(unused)]
    fn dispatch(&self, work_x: u32, work_y: u32, work_z: u32) {
        todo!()
    }

    #[allow(unused)]
    fn dispatch_indirect(&self, buffer: &Buffer<Directx>, offset: u64) {
        todo!()
    }
}