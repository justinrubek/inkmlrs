use std::error::Error;
use std::ops::Range;
use std::sync::Arc;

use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::AutoCommandBufferBuilder;
use vulkano::device::{Device, DeviceExtensions, Features, Queue};
use vulkano::format::Format;
use vulkano::framebuffer::{FramebufferAbstract, RenderPassAbstract, Subpass};
use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice};
use vulkano::pipeline::{GraphicsPipeline, GraphicsPipelineAbstract};
use vulkano::pipeline::vertex::{VertexMember, VertexMemberTy};

mod shaders;

pub struct Context {
    pub device: Arc<Device>,
    pub queue: Arc<Queue>,
}

impl Context {
    pub fn new(physical: PhysicalDevice, exts: &DeviceExtensions) -> Result<Arc<Context>, Box<dyn Error>> {
        let queue_family = physical.queue_families()
            .find(|&q| q.supports_graphics())
            .expect("Unable to find a queue supporting graphics");

        let (device, mut queues) = {
            Device::new(physical, &Features::none(), exts,
                        [(queue_family, 0.5)].iter().cloned() )?
        };
        let queue = queues.next().unwrap();

        Ok(Arc::new(Context {
            device,
            queue,
        }))
    }
}

#[derive(Clone, Debug)]
struct Section {
    range: Range<usize>,
}

pub struct Renderer {
    pub context: Arc<Context>,
    pub pipeline: Arc<GraphicsPipelineAbstract + Send + Sync>,
    pub render_pass: Arc<RenderPassAbstract + Send + Sync>,
    queued: Vec<Vertex>,
    sections: Vec<Section>,
}

impl Renderer {
    pub fn new(context: Arc<Context>) -> Result<Renderer, Box<dyn Error>> {
        let vs = shaders::lines_vsm::Shader::load(context.device.clone())?;
        let fs = shaders::lines_fsm::Shader::load(context.device.clone())?;

        let render_pass = Arc::new(vulkano::single_pass_renderpass!(context.device.clone(),
                attachments: {
                    color: {
                        load: Clear,
                        store: Store,
                        format: Format::R8G8B8A8Unorm,
                        samples: 1,
                    }
                },

                pass: {
                    color: [color],
                    depth_stencil: {}
                }
        )?);

        let pipeline = Arc::new(GraphicsPipeline::start()
                                .vertex_input_single_buffer::<Vertex>()
                                .vertex_shader(vs.main_entry_point(), ())
                                .triangle_list()
                                .viewports_dynamic_scissors_irrelevant(1)
                                .fragment_shader(fs.main_entry_point(), ())
                                .render_pass(Subpass::from(render_pass.clone(), 0).ok_or("no subpass")?)
                                .build(context.device.clone())?
        );
        Ok(Renderer {
            context,
            pipeline,
            render_pass,
            queued: Vec::new(),
            sections: Vec::new(),
        })
    }

    pub fn add_stroke(&mut self, endpoints: Vec<Point>) {
         
        let vertices = trace_to_tri(endpoints);
        
        let start = self.queued.len();
        self.queued
            .extend(vertices);
        let end = self.queued.len();
        self.sections.push(Section { range: start..end } );
    }

    pub fn draw(&mut self, cmd: AutoCommandBufferBuilder) -> Result<AutoCommandBufferBuilder, Box<dyn Error>> {
        Ok(cmd)
    }
}

// TODO: Find a way to perform this on the GPU, and do it better
fn trace_to_tri(points: Vec<Point>) -> Vec<Vertex> {
    // TODO: Determine the vertices for the triangles to represent this line segment
    let mut vertices = Vec::new();
    let w: f32 = 5.0;
    
    // Loop through points in pairs
    for i in 0..(points.len()-1) {
        // Determine the rise and run differences between the points
        let mut p0 = &points[i];
        let mut p1 = &points[i+1];
        
        let mut run = p0.0 - p1.0;
        let mut rise = p0.1 - p1.1;
        // Figure out which point is top/bottom in order to create the rectangle
        // If run is negative, p0 is to the left of p1
        // if rise is negative, p0 is below p1
        if rise < 0.0 {
            if run < 0.0 {
                // p0 is below and to the right
                rise = rise.abs();
                run = -(run.abs());
            } else {
                // p0 is below and to the left
                rise = -(rise.abs());
                run = run.abs();
            }
        } else {
            if run < 0.0 {
                // p1 is below and to the left
                rise = rise.abs();
                run = -(run.abs());
            } else {
                // p1 is below and to the right
                rise = -(rise.abs());
                run = run.abs();
            }
            p0 = &points[i+1];
            p1 = &points[i];

        }

        run *= w;
        rise *= w;

        vertices.push(Vertex { position: Point(p0.0+run, p0.1+rise) });
        vertices.push(Vertex { position: Point(p0.0-run, p0.1-rise) });
        vertices.push(Vertex { position: Point(p1.0+run, p1.1+rise) });
        vertices.push(Vertex { position: Point(p1.0-run, p1.1-rise) });
    }
    //println!("{:?}", vertices.len());
    // println!("{:?}", vertices);
    
    vertices 
}

#[derive(Debug)]
struct Vertex {
    position: Point, 
}

impl Vertex {
    fn new(position: Point) -> Self {
        Vertex { position }
    }
}

impl_vertex!(Vertex, position);

#[repr(C)]
#[derive(Clone, Debug)]
pub struct Point(pub f32, pub f32);

unsafe impl VertexMember for Point {
    fn format() -> (VertexMemberTy, usize) {
        (VertexMemberTy::F32, 2)
    }
}
