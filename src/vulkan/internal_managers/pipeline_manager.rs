use anyhow::Result;
use ash::vk;
use spirv_cross2::spirv::ExecutionModel;

use crate::{vulkan::mappings::CobraVulkan, BlendFactor, BlendOp, ImageFormat};

use super::utils;

#[derive(Hash, Eq, PartialEq, Clone, Copy)]
pub struct GraphicsPipelineKey {
    pub(crate) color_attachment: ImageFormat,
    pub(crate) depth_attachment: ImageFormat,
    pub(crate) shaders: [Option<&'static [u8]>; 2],

    pub(crate) blend_enable: bool,
    pub(crate) src_blend: BlendFactor,
    pub(crate) dst_blend: BlendFactor,
    pub(crate) blend_alpha: BlendOp,
    pub(crate) blend_op: BlendOp,
    pub(crate) src_blend_alpha: BlendFactor,
    pub(crate) dst_blend_alpha: BlendFactor
}

impl GraphicsPipelineKey {
    pub(crate) fn new() -> GraphicsPipelineKey {
        GraphicsPipelineKey { 
            color_attachment: ImageFormat::Unknown,
            depth_attachment: ImageFormat::Unknown,
            shaders: [None; 2],

            blend_enable: false, 
            blend_op: BlendOp::Add, src_blend: BlendFactor::Zero, dst_blend: BlendFactor::Zero,
            blend_alpha: BlendOp::Add, src_blend_alpha: BlendFactor::Zero, dst_blend_alpha: BlendFactor::Zero
        }
    }
}

impl CobraVulkan {

    pub(crate) fn bind_graphics_pipeline(&self, cmd: vk::CommandBuffer, key: GraphicsPipelineKey) -> Result<()> {
        unsafe {
            // TODO: this requires two hashes, figure out how to do one without deadlocking
            let graphics_pipelines = self.graphics_pipelines.read().unwrap();
            self.device.cmd_bind_pipeline(cmd, vk::PipelineBindPoint::GRAPHICS, match graphics_pipelines.contains_key(&key) {
                true => *graphics_pipelines.get(&key).unwrap(),
                false => {
                    drop(graphics_pipelines);

                    let mut shader_module_infos = Vec::new();
                    let mut shader_stages = Vec::new();

                    for shader in key.shaders {
                        let shader = <[u8]>::align_to::<u32>(match shader {
                            Some(shader) => shader,
                            None => break
                        }).1;

                        let module = spirv_cross2::Module::from_words(shader);
                        let compiler = spirv_cross2::Compiler::<spirv_cross2::targets::None>::new(module)?;
                        for entry in compiler.entry_points()? {
                            shader_module_infos.push(vk::ShaderModuleCreateInfo::default().code(shader));
            
                            shader_stages.push(vk::PipelineShaderStageCreateInfo::default()
                                .stage(match entry.execution_model {
                                    ExecutionModel::Vertex => vk::ShaderStageFlags::VERTEX,
                                    ExecutionModel::Fragment => vk::ShaderStageFlags::FRAGMENT,
                                    _ => todo!("Implement other shader types")
                                })
                                .name(c"main")
                                .push_next(&mut *(shader_module_infos.last_mut().unwrap() as *mut _)) // borrow checker is wrong and its safe, so we need to "convince" it
                            );
                        }
                    }

                    let pipeline = self.device.create_graphics_pipelines(vk::PipelineCache::null(), &[vk::GraphicsPipelineCreateInfo::default()
                        .stages(&shader_stages)
                        .vertex_input_state(&vk::PipelineVertexInputStateCreateInfo::default())
                        .input_assembly_state(&vk::PipelineInputAssemblyStateCreateInfo::default()
                            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
                        )
                        .viewport_state(&vk::PipelineViewportStateCreateInfo::default())
                        .rasterization_state(&vk::PipelineRasterizationStateCreateInfo::default()
                            .polygon_mode(vk::PolygonMode::FILL)
                            .cull_mode(vk::CullModeFlags::NONE)
                            .front_face(vk::FrontFace::CLOCKWISE)
                            .line_width(1.0)
                        )
                        .multisample_state(&vk::PipelineMultisampleStateCreateInfo::default()
                            .rasterization_samples(vk::SampleCountFlags::TYPE_1)
                            .min_sample_shading(1.0)
                        )
                        .depth_stencil_state(&vk::PipelineDepthStencilStateCreateInfo::default()
                            .min_depth_bounds(0.0)
                            .max_depth_bounds(1.0)
                        )
                        .color_blend_state(&vk::PipelineColorBlendStateCreateInfo::default()
                            .attachments(&[vk::PipelineColorBlendAttachmentState::default()
                                .blend_enable(key.blend_enable)
                                .src_color_blend_factor(utils::blend_factor_to_vulkan(key.src_blend))
                                .dst_color_blend_factor(utils::blend_factor_to_vulkan(key.dst_blend))
                                .alpha_blend_op(utils::blend_op_to_vulkan(key.blend_alpha))
                                .color_blend_op(utils::blend_op_to_vulkan(key.blend_op))
                                .src_alpha_blend_factor(utils::blend_factor_to_vulkan(key.src_blend_alpha))
                                .dst_alpha_blend_factor(utils::blend_factor_to_vulkan(key.dst_blend_alpha))
                                .color_write_mask(vk::ColorComponentFlags::RGBA)
                            ])
                        )
                        .dynamic_state(&vk::PipelineDynamicStateCreateInfo::default()
                            .dynamic_states(&[
                                vk::DynamicState::VIEWPORT_WITH_COUNT, vk::DynamicState::SCISSOR_WITH_COUNT,
                                vk::DynamicState::DEPTH_TEST_ENABLE, vk::DynamicState::DEPTH_WRITE_ENABLE, vk::DynamicState::DEPTH_COMPARE_OP
                            ])
                        )
                        .layout(self.bindless_pipeline_layout)
                        .push_next(&mut vk::PipelineRenderingCreateInfo::default()
                            .color_attachment_formats(&[utils::image_format_to_vulkan(key.color_attachment)])
                            .depth_attachment_format(utils::image_format_to_vulkan(key.depth_attachment))
                        )
                    ], None).unwrap()[0];

                    self.graphics_pipelines.write().unwrap().insert(key, pipeline);
                    pipeline
                }
            });
            Ok(())
        }
    }

}