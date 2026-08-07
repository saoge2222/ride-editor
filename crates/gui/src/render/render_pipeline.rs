use std::sync::Arc;

use vulkano::buffer::{Buffer, BufferContents, BufferCreateInfo, BufferUsage, Subbuffer};
use vulkano::device::Device;
use vulkano::memory::allocator::{
    AllocationCreateInfo, StandardMemoryAllocator, MemoryTypeFilter,
};
use vulkano::pipeline::graphics::color_blend::{ColorBlendAttachmentState, ColorBlendState};
use vulkano::pipeline::graphics::input_assembly::InputAssemblyState;
use vulkano::pipeline::graphics::multisample::MultisampleState;
use vulkano::pipeline::graphics::rasterization::RasterizationState;
use vulkano::pipeline::graphics::subpass::PipelineSubpassType;
use vulkano::pipeline::graphics::vertex_input::{Vertex, VertexDefinition};
use vulkano::pipeline::graphics::viewport::{Scissor, Viewport, ViewportState};
use vulkano::pipeline::layout::PipelineLayoutCreateInfo;
use vulkano::pipeline::graphics::GraphicsPipelineCreateInfo;
use vulkano::pipeline::{
    GraphicsPipeline, PipelineLayout, PipelineShaderStageCreateInfo,
};
use vulkano::render_pass::RenderPass;

use crate::vulkano_base::vulkano_base_render_loop::CommandBufferBuilder;
use super::render_draw::DrawList;
use super::render_shader;
use super::render_vertex::Vertex2D;

const NDC_OFFSET: f32 = -1.0;
const PUSH_CONSTANT_OFFSET: u32 = 0;
const DRAW_INSTANCE_COUNT: u32 = 1;
const DRAW_FIRST_VERTEX: u32 = 0;
const DRAW_FIRST_INSTANCE: u32 = 0;
const VERTEX_BUFFER_BINDING: u32 = 0;

#[repr(C)]
#[derive(Clone, Copy, Debug, BufferContents)]
pub struct PushConstants {
    pub scale: [f32; 2],
    pub offset: [f32; 2],
}

pub struct RenderPipelineContext {
    pub pipeline: Arc<GraphicsPipeline>,
    pub pipeline_layout: Arc<PipelineLayout>,
    device: Arc<Device>,
    render_pass: Arc<RenderPass>,
    memory_allocator: Arc<StandardMemoryAllocator>,
    vertex_buffer: Option<(Subbuffer<[Vertex2D]>, u32)>,
}

impl RenderPipelineContext {
    pub fn new(
        device: Arc<Device>,
        render_pass: Arc<RenderPass>,
        extent: [u32; 2],
        memory_allocator: Arc<StandardMemoryAllocator>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (pipeline, pipeline_layout) = create_pipeline(&device, &render_pass, extent)?;

        Ok(Self {
            pipeline,
            pipeline_layout,
            device,
            render_pass,
            memory_allocator,
            vertex_buffer: None,
        })
    }

    pub fn recreate(&mut self, extent: [u32; 2]) -> Result<(), Box<dyn std::error::Error>> {
        let (pipeline, pipeline_layout) =
            create_pipeline(&self.device, &self.render_pass, extent)?;
        self.pipeline = pipeline;
        self.pipeline_layout = pipeline_layout;
        Ok(())
    }

    pub fn record(
        &mut self,
        builder: &mut CommandBufferBuilder,
        extent: [u32; 2],
        draw_list: &DrawList,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if draw_list.is_empty() {
            return Ok(());
        }

        let vertices = draw_list.build_mesh();
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

        let push_constants = PushConstants {
            scale: [2.0 / extent[0] as f32, 2.0 / extent[1] as f32],
            offset: [NDC_OFFSET, NDC_OFFSET],
        };

        builder.bind_pipeline_graphics(self.pipeline.clone())?;
        builder.push_constants(
            self.pipeline_layout.clone(),
            PUSH_CONSTANT_OFFSET,
            push_constants,
        )?;
        builder.bind_vertex_buffers(VERTEX_BUFFER_BINDING, vertex_buffer.clone())?;
        builder.draw(
            vertex_count,
            DRAW_INSTANCE_COUNT,
            DRAW_FIRST_VERTEX,
            DRAW_FIRST_INSTANCE,
        )?;

        self.vertex_buffer = Some((vertex_buffer, vertex_count));
        Ok(())
    }
}

fn create_pipeline(
    device: &Arc<Device>,
    render_pass: &Arc<RenderPass>,
    extent: [u32; 2],
) -> Result<(Arc<GraphicsPipeline>, Arc<PipelineLayout>), Box<dyn std::error::Error>> {
    let shaders = render_shader::load(device.clone())?;
    let vertex_entry = shaders
        .vertex
        .entry_point("main")
        .ok_or("vertex shader entry point not found")?;
    let fragment_entry = shaders
        .fragment
        .entry_point("main")
        .ok_or("fragment shader entry point not found")?;

    let input_interface = &vertex_entry.info().input_interface;
    let vertex_input_state = [Vertex2D::per_vertex()].definition(input_interface)?;

    let pipeline_layout = PipelineLayout::new(
        device.clone(),
        PipelineLayoutCreateInfo {
            push_constant_ranges: vertex_entry
                .info()
                .push_constant_requirements
                .into_iter()
                .collect(),
            ..Default::default()
        },
    )?;

    let mut create_info = GraphicsPipelineCreateInfo::layout(pipeline_layout.clone());
    create_info.stages = [
        PipelineShaderStageCreateInfo::new(vertex_entry),
        PipelineShaderStageCreateInfo::new(fragment_entry),
    ]
    .into_iter()
    .collect();
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
        1,
        ColorBlendAttachmentState::default(),
    ));
    create_info.subpass = Some(PipelineSubpassType::BeginRenderPass(
        render_pass.clone().first_subpass(),
    ));

    let pipeline = GraphicsPipeline::new(device.clone(), None, create_info)?;

    Ok((pipeline, pipeline_layout))
}
