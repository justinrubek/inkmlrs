#[macro_use]
extern crate vulkano;
extern crate winit;
extern crate xml;

use std::sync::Arc;

use vulkano::device::DeviceExtensions;
use vulkano::instance::{Instance, InstanceExtensions, PhysicalDevice};

mod inkml;
mod parse;
mod render;

use std::error::Error;
use std::fs::File;
use std::io::BufReader;

use self::inkml::{Ink,Node,Traces};

fn main() -> Result<(), Box<dyn Error>> {
    /*
    let file = File::open("file1.inkml")?;
    let file = BufReader::new(file);
    let document = parse::parse_inkml(file)?;
    */
    // document.iter().for_each(|n| println!("{:?}", n));
    let document: Ink = Default::default();
    
    draw(document)
}

fn draw(document: Ink) -> Result<(), Box<dyn Error>> {
    let instance = Instance::new(None, &InstanceExtensions::none(), None)?;
    let physical = PhysicalDevice::enumerate(&instance).next().expect("Physical");
    let context = render::Context::new(physical, &DeviceExtensions::none())?;
    
    let mut renderer = {
        render::Renderer::new(context.clone())?
    };

    use vulkano::format::Format;
    use vulkano::framebuffer::Framebuffer;
    use vulkano::image::{Dimensions,StorageImage};
    let image = StorageImage::new(context.device.clone(),
        Dimensions::Dim2d { width: 1024, height: 1024 },
        Format::R8G8B8A8Unorm, Some(context.queue.family()))?;
    let framebuffer = Arc::new(Framebuffer::start(renderer.render_pass.clone())
                               .add(image.clone())?
                               .build()?);
    
    use vulkano::command_buffer::AutoCommandBufferBuilder;
    use vulkano::format::ClearValue;
    let command_buffer = AutoCommandBufferBuilder::new(
        context.device.clone(),
        context.queue.family())?
        .begin_render_pass(framebuffer.clone(), false, vec![[1.0, 1.0, 1.0, 1.0].into()])?;

    document.iter().filter_map(|n| {
        match n {
            inkml::Node::Traces(inkml::Traces::Trace(inkml::Trace { ref vertices })) => {
                Some(vertices)
            }
            _ => None
        }
    }).for_each(|vertices| renderer.add_stroke(vertices.clone()));
    
    let command_buffer = renderer.draw(command_buffer)?
        .end_render_pass()?
        .build()?;
    
    Ok(())
}

fn window() {
    let mut events_loop = winit::EventsLoop::new();
    let mut lpressed = false;
    let _window = winit::Window::new(&events_loop).unwrap();

    println!(r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<inkml:ink xmlns:inkml="http://www.w3.org/TR/InkML">
  <inkml:definitions>
    <inkml:context xml:id="ctx0">
      <inkml:inkSource xml:id="inkSrc0">
        <inkml:traceFormat>
          <inkml:channel name="X" type="decimal"/>
          <inkml:channel name="Y" type="decimal"/>
        </inkml:traceFormat>
      </inkml:inkSource>
    </inkml:context>
    <inkml:brush xml:id="br0">
      <inkml:brushProperty name="width" value=".07" units="mm"/>
      <inkml:brushProperty name="color" value="\#00AA24" />
    </inkml:brush>
  </inkml:definitions>
  <inkml:traceGroup>"#);

    events_loop.run_forever(|event| {
        // println!("{:?}", event);
        use winit::Event::WindowEvent;
        use winit::ElementState;
        use winit::WindowEvent::*;
        use winit::ControlFlow;
        
        match event {
            WindowEvent {
                event: CloseRequested,
                ..
            } => ControlFlow::Break,
            
            /*
            WindowEvent {
                event: winit::WindowEvent::Touch(touch),
                ..
            } => {
                println("       {} {},", touch.location.x, touch.location.y);
                ControlFlow::Continue
            }
            */
            WindowEvent {
                event: CursorMoved { position, .. },
                ..
            } => {
                if lpressed {
                    println!("      {} {},", position.x, position.y);
                }
                ControlFlow::Continue    
            },

            WindowEvent {
                event: MouseInput { state, button: winit::MouseButton::Left, .. },
                ..
            } => {
                match state {
                    ElementState::Pressed => {
                        lpressed = true;
                        println!("    <inkml:trace contextRef=\"#ctx0\" brushRef=\"#br0\">");
                    },
                    ElementState::Released => { 
                        lpressed = false;
                        println!("    </inkml:trace>");
                    },
                    
                }
                ControlFlow::Continue
            },
            _ => ControlFlow::Continue
        }
    });
             
    println!("  </inkml:traceGroup>
</inkml:ink>");

}
