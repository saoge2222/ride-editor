use std::sync::Arc;

use vulkano::buffer::{Buffer, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::descriptor_set::allocator::{
    StandardDescriptorSetAllocator, StandardDescriptorSetAllocatorCreateInfo,
};
use vulkano::descriptor_set::layout::DescriptorSetLayout;
use vulkano::descriptor_set::{DescriptorSet, WriteDescriptorSet};
use vulkano::device::Device;
use vulkano::memory::allocator::{AllocationCreateInfo, MemoryTypeFilter, StandardMemoryAllocator};
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::subpass::PipelineSubpassType;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Scissor, Viewport, ViewportState};
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::layout::PipelineDescriptorSetLayoutCreateInfo;
use vulkano::pipeline::{GraphicsPipeline, PipelineBindPoint, PipelineLayout, PipelineShaderStageCreateInfo};
use vulkano::render_pass::RenderPass;

use crate::vulkano_base::vulkano_base_render_loop::CommandBufferBuilder;
use super::render_glyph::GlyphAtlas;
use super::render_pipeline::{record_draw, PushConstants};
use super::render_shader;
use super::render_vertex::TexturedVertex;

const NDC_OFFSET: f32 = -1.0;
const PUSH_CONSTANT_OFFSET: u32 = 0;
const VERTEX_BUFFER_BINDING: u32 = 0;
const DESCRIPTOR_SET_INDEX: u32 = 0;
const SAMPLER_BINDING: u32 = 0;
const COLOR_ATTACHMENT_COUNT: u32 = 1;

pub struct TextRenderer {
    pipeline: Arc<GraphicsPipeline>,
    pipeline_layout: Arc<PipelineLayout>,
    descriptor_set_layout: Arc<DescriptorSetLayout>,
    descriptor_allocator: Arc<StandardDescriptorSetAllocator>,
    atlas: GlyphAtlas,
    memory_allocator: Arc<StandardMemoryAllocator>,
    vertex_buffer: Option<(Subbuffer<[TexturedVertex]>, u32)>,
}

impl TextRenderer {
    pub fn new(
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        extent: [u32; 2],
        memory_allocator: Arc<StandardMemoryAllocator>,
        atlas: GlyphAtlas,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let shaders = render_shader::load_text_shaders(device.clone())?;
        let vertex_entry = shaders
            .vertex
            .entry_point("main")
            .ok_or("text vertex shader entry point not found")?;
        let fragment_entry = shaders
            .fragment
            .entry_point("main")
            .ok_or("text fragment shader entry point not found")?;

        let vertex_input_state = [TexturedVertex::per_vertex()].definition(&vertex_entry)?;

        let vertex_stage = PipelineShaderStageCreateInfo::new(vertex_entry);
        let fragment_stage = PipelineShaderStageCreateInfo::new(fragment_entry);

        let pipeline_layout_create_info = PipelineDescriptorSetLayoutCreateInfo::from_stages([
            &vertex_stage,
            &fragment_stage,
        ])
        .into_pipeline_layout_create_info(device.clone())?;
        let pipeline_layout = PipelineLayout::new(device.clone(), pipeline_layout_create_info)?;
        let descriptor_set_layout = pipeline_layout.set_layouts()[DESCRIPTOR_SET_INDEX as usize].clone();

        let mut create_info = GraphicsPipelineCreateInfo::layout(pipeline_layout.clone());
        create_info.stages = [vertex_stage, fragment_stage].into_iter().collect();
        create_info.vertex_input_state = Some(vertex_input_state);
        create_info.input_assembly_state = Some(InputAssemblyState::default());
        create_info.viewport_state = Some(ViewportState {
            viewports: vec![Viewport {
                offset: [0.0, 0.0],
                extent: [extent[0] as f32, extent[1] as f32],
                depth_range: 0.0..=1.0,
            }]
            .into(),
            scissors: vec![Scissor {
                offset: [0, 0],
                extent,
            }]
            .into(),
            ..Default::default()
        });
        create_info.rasterization_state = Some(RasterizationState::default());
        create_info.multisample_state = Some(MultisampleState::default());
        create_info.color_blend_state = Some(ColorBlendState::with_attachment_states(
            COLOR_ATTACHMENT_COUNT,
            ColorBlendAttachmentState::default(),
        ));
        create_info.subpass = Some(PipelineSubpassType::BeginRenderPass(
            render_pass.clone().first_subpass(),
        ));

        let pipeline = GraphicsPipeline::new(device.clone(), None, create_info)?;
        let descriptor_allocator = Arc::new(StandardDescriptorSetAllocator::new(
            device,
            StandardDescriptorSetAllocatorCreateInfo::default(),
        ));

        Ok(Self {
            pipeline,
            pipeline_layout,
            descriptor_set_layout,
            descriptor_allocator,
            atlas,
            memory_allocator,
            vertex_buffer: None,
        })
    }

    pub fn draw(
        &mut self,
        builder: &mut CommandBufferBuilder,
        extent: [u32; 2],
        x: f32,
        y: f32,
        text: &str,
        color: [f32; 4],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut vertices = Vec::new();
        let mut pen_x = x;
        for ch in text.chars() {
            let Some(placement) = self.atlas.glyph(ch) else {
                continue;
            };
            if placement.width_px > 0 && placement.height_px > 0 {
                let x0 = pen_x + placement.left_px;
                let y0 = y - placement.top_px;
                let x1 = x0 + placement.width_px as f32;
                let y1 = y0 + placement.height_px as f32;
                let [u0, v0, u1, v1] = placement.uv;
                vertices.push(TexturedVertex {
                    position: [x0, y0],
                    uv: [u0, v0],
                    color,
                });
                vertices.push(TexturedVertex {
                    position: [x1, y0],
                    uv: [u1, v0],
                    color,
                });
                vertices.push(TexturedVertex {
                    position: [x0, y1],
                    uv: [u0, v1],
                    color,
                });
                vertices.push(TexturedVertex {
                    position: [x0, y1],
                    uv: [u0, v1],
                    color,
                });
                vertices.push(TexturedVertex {
                    position: [x1, y0],
                    uv: [u1, v0],
                    color,
                });
                vertices.push(TexturedVertex {
                    position: [x1, y1],
                    uv: [u1, v1],
                    color,
                });
            }
            pen_x += placement.advance_px;
        }
        if vertices.is_empty() {
            return Ok(());
        }

        let vertex_buffer = Buffer::from_iter(
            self.memory_allocator.clone(),
            BufferCreateInfo {
                usage: BufferUsage::VERTEX_BUFFER,
                ..Default::default()
            },
            AllocationCreateInfo {
                memory_type_filter: MemoryTypeFilter::PREFER_DEVICE
                    | MemoryTypeFilter::HOST_SEQUENTIAL_WRITE,
                ..Default::default()
            },
            vertices,
        )?;
        let vertex_count = vertex_buffer.len() as u32;

        let descriptor_set = DescriptorSet::new(
            self.descriptor_allocator.clone(),
            self.descriptor_set_layout.clone(),
            [WriteDescriptorSet::image_view_sampler(
                SAMPLER_BINDING,
                self.atlas.texture.image_view.clone(),
                self.atlas.sampler.clone(),
            )],
            [],
        )?;

        let push_constants = PushConstants {
            scale: [2.0 / extent[0] as f32, 2.0 / extent[1] as f32],
            offset: [NDC_OFFSET, NDC_OFFSET],
        };

        builder.bind_pipeline_graphics(self.pipeline.clone())?;
        builder.bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            self.pipeline_layout.clone(),
            DESCRIPTOR_SET_INDEX,
            descriptor_set,
        )?;
        builder.push_constants(
            self.pipeline_layout.clone(),
            PUSH_CONSTANT_OFFSET,
            push_constants,
        )?;
        builder.bind_vertex_buffers(VERTEX_BUFFER_BINDING, vertex_buffer.clone())?;
        record_draw(builder, vertex_count)?;

        self.vertex_buffer = Some((vertex_buffer, vertex_count));
        Ok(())
    }
}
