use std::collections::HashMap;
use std::sync::Arc;

use vulkano::descriptor_set::layout::DescriptorType;
use vulkano::descriptor_set::persistent::PersistentDescriptorSet;
use vulkano::pipeline::PipelineBindPoint;
use vulkano::{
    command_buffer::{AutoCommandBufferBuilder, PrimaryAutoCommandBuffer},
    descriptor_set::layout::DescriptorSetLayout,
    pipeline::{
        graphics::color_blend::{AttachmentBlend, ColorBlendState, ColorBlendAttachmentState, BlendFactor, BlendOp, LogicOp},
        graphics::vertex_input::VertexDefinition,
        GraphicsPipeline,
    },
    shader::EntryPoint,
    render_pass::Subpass,
};
use vulkano::pipeline::Pipeline;
use vulkano::pipeline::graphics::viewport::ViewportState;
use vulkano::pipeline::StateMode;
use vulkano::pipeline::graphics::color_blend::ColorComponents;

use crate::graphics::{context::GraphicsContext, BlendMode, PipelineData};

pub enum VertexOrder {
    LineList,
    LineStrip,
    PointList,
    TriangleFan,
    TriangleList,
    TriangleStrip,
}

pub enum ShaderType {
    Vertex,
    Fragment,
    TessellationControl,
    TessellationEval,
    Geometry,
    Default,
}

pub type ShaderId = usize;

pub struct ShaderProgram {
    pipelines: PipelineObjectSet,
    current_mode: BlendMode,
}

pub trait ShaderHandle {
    fn draw(
        &self,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        pipe_data: &PipelineData,
    );
    // fn set_blend_mode(&mut self, mode: BlendMode);
    fn blend_mode(&self) -> BlendMode;
    fn layout(&self) -> &[Arc<DescriptorSetLayout>];
    fn pipeline(&self) -> Arc<GraphicsPipeline>;
}

impl ShaderHandle for ShaderProgram {
    fn draw(
        &self,
        command_buffer: &mut AutoCommandBufferBuilder<PrimaryAutoCommandBuffer>,
        pipe_data: &PipelineData,
    ) {
        command_buffer.bind_pipeline_graphics(self.pipeline().clone());

        let layout = self.layout()[0].clone();

        let mut builder = PersistentDescriptorSet::start(layout.clone());
        for i in 0..layout.num_bindings() {
            match layout.descriptor(i).unwrap().ty {
                DescriptorType::UniformBuffer => {
                    builder
                        .add_buffer(pipe_data.uniform_buffers.get(&i).unwrap().clone())
                        .unwrap();
                }
                DescriptorType::CombinedImageSampler { .. } => {
                    let image_sampler = pipe_data.sampled_images.get(&i).unwrap();
                    builder
                        .add_sampled_image(image_sampler.0.clone(), image_sampler.1.clone())
                        .unwrap();
                }
                _ => {
                    panic!(
                        "Unsupported descriptor type ({:?}) in shader.",
                        layout.descriptor(i).unwrap().ty
                    )
                }
            }
        }

        let set = builder.build().unwrap();

        command_buffer.bind_descriptor_sets(
            PipelineBindPoint::Graphics,
            self.pipeline().layout().clone(),
            0,
            set,
        );

        command_buffer.bind_vertex_buffers(
            0,
            (
                pipe_data.vertex_buffer.clone(),
                pipe_data.instance_buffer.clone(),
            ),
        );

        command_buffer
            .draw(pipe_data.vertex_count, pipe_data.instance_count, 0, 0)
            .unwrap();
    }

    // fn set_blend_mode(&mut self, mode: BlendMode) {
    //     let _ = self.pipelines.mode(&mode)?;
    //     self.current_mode = mode;
    // }

    fn blend_mode(&self) -> BlendMode {
        self.current_mode
    }

    fn layout(&self) -> &[Arc<DescriptorSetLayout>] {
        self.pipelines
            .get(&self.current_mode)
            .unwrap()
            .layout()
            .descriptor_set_layouts()
    }

    fn pipeline(&self) -> Arc<GraphicsPipeline> {
        self.pipelines.get(&self.current_mode).unwrap().clone()
    }
}

impl ShaderProgram {
    pub fn new<Vd>(
        context: &mut GraphicsContext,
        vertex_type: Vd,
        vertex_order: VertexOrder,
        vertex_shader: EntryPoint,
        fragment_shader: EntryPoint,
        blend: BlendMode,
    ) -> Self
    where
        Vd: VertexDefinition + 'static + Sync + Send,
    {
        let po = new_pipeline(
            context,
            vertex_type,
            vertex_order,
            vertex_shader,
            fragment_shader,
            blend,
        );

        let mut pos = PipelineObjectSet::new(16);
        pos.insert(blend, po);

        Self {
            pipelines: pos,
            current_mode: blend,
        }
    }

    pub fn from_pipeline(mode: BlendMode, pipeline: Arc<GraphicsPipeline>) -> Self {
        let mut pipeline_os = PipelineObjectSet::new(16);
        pipeline_os.insert(mode, pipeline);
        Self {
            pipelines: pipeline_os,
            current_mode: mode,
        }
    }
}

// This structure is to store multiple pipelines for different blend modes.
pub struct PipelineObjectSet {
    pipelines: HashMap<BlendMode, Arc<GraphicsPipeline>>,
}

impl PipelineObjectSet {
    pub fn new(cap: usize) -> Self {
        Self {
            pipelines: HashMap::with_capacity(cap),
        }
    }

    pub fn insert(&mut self, blend_mode: BlendMode, pipeline: Arc<GraphicsPipeline>) {
        self.pipelines.insert(blend_mode, pipeline);
    }

    pub fn get(&self, blend_mode: &BlendMode) -> Option<&Arc<GraphicsPipeline>> {
        self.pipelines.get(blend_mode)
    }

    // pub fn mode(&self, mode: &BlendMode) -> Result<&GraphicsPipeline, GraphicsError> {
    //     match self.pipelines.get(&mode) {
    //         Some(po) => Ok(po),
    //         None => {},
    //     }
    // }
}

pub fn new_pipeline<Vd>(
    context: &mut GraphicsContext,
    vertex_type: Vd,
    vertex_order: VertexOrder,
    vertex_shader: EntryPoint,
    fragment_shader: EntryPoint,
    blend: BlendMode,
) -> Arc<GraphicsPipeline>
where
    Vd: VertexDefinition + 'static + Sync + Send,
{
    let mut pipeline = GraphicsPipeline::start()
        .vertex_input_state::<Vd>(vertex_type)
        .vertex_shader(vertex_shader, ())
        .viewport_state(ViewportState::viewport_dynamic_scissor_irrelevant())
        .fragment_shader(fragment_shader, ())
        .color_blend_state(blend.into())
        .render_pass(Subpass::from(context.render_pass.clone(), 0).unwrap());

    pipeline = match vertex_order {
        VertexOrder::LineList => pipeline.line_list(),
        VertexOrder::LineStrip => pipeline.line_strip(),
        VertexOrder::PointList => pipeline.point_list(),
        VertexOrder::TriangleFan => pipeline.triangle_fan(),
        VertexOrder::TriangleList => pipeline.triangle_list(),
        VertexOrder::TriangleStrip => pipeline.triangle_strip(),
    };

    pipeline.build(context.device.clone()).unwrap()
}

impl From<BlendMode> for ColorBlendState {
    fn from(blend_mode: BlendMode) -> Self {

        let mut logic_op: Option<StateMode<LogicOp>> = None;
        let mut attach: Option<AttachmentBlend> = None;
        let blend_constants: [f32; 4] = [1.0, 1.0, 1.0, 1.0]; // TODO implement these.

        match blend_mode {
            BlendMode::Add => {
                attach = Some(AttachmentBlend {
                    color_op: BlendOp::Add,
                    color_source: BlendFactor::One,
                    color_destination: BlendFactor::One,
                    alpha_op: BlendOp::Add,
                    alpha_source: BlendFactor::One,
                    alpha_destination: BlendFactor::One,
                });
            }
            BlendMode::Subtract => {
                attach = Some(AttachmentBlend {
                    color_op: BlendOp::Subtract,
                    color_source: BlendFactor::One,
                    color_destination: BlendFactor::One,
                    alpha_op: BlendOp::Subtract,
                    alpha_source: BlendFactor::One,
                    alpha_destination: BlendFactor::One,
                });
            }
            BlendMode::Alpha => {
                attach = Some(AttachmentBlend::alpha());
            }
            BlendMode::Invert => {
                logic_op = Some(StateMode::Fixed(LogicOp::Invert));
            }
        };

        return ColorBlendState {
            logic_op: logic_op,
            attachments: vec![ColorBlendAttachmentState {
                blend: attach,
                color_write_mask: ColorComponents::all(),
                color_write_enable: StateMode::Fixed(true),
            }],
            blend_constants: StateMode::Fixed(blend_constants),
        };
    }
}
